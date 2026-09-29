"""Standards, cross-implementation, boundary, tamper and policy tests.

    python3 test_daylight.py --orangec /path/to/orangec [--upstream /path/to/-wuci-ji]

Without --upstream, the primitives and the standalone program are checked
against RFC and FIPS vectors, Python's hashlib and hmac, and a pure-Python
reference of RFC 8439 and the Horizon frame adapted from Daylight v15
Meridian. With --upstream, a chasebryan/-wuci-ji checkout at the pinned
revision, the frame is also compared with the Horizon source and the vault
tests run beneath its evidence checks.
"""
import argparse
import hashlib
import hmac
import json
from pathlib import Path
import random
import struct
import subprocess
import sys
import tempfile
import unittest

from daylight import MAX_BYTES, PROGRAM, OrangeBackend, array, literals

ENGINE = None
UPSTREAM = None

MAGIC = b'DLTHV1A'
KDF_INFO = b'DAYLIGHT-HORIZON-ALPHA-AEAD-KEY:'
ROOT = bytes(range(32))
NONCE = bytes.fromhex('000000000000000000000001')
AUTHORIZATION_TAG = bytes.fromhex('0123456789abcdef' * 4)
PLAINTEXT = b'Daylight Horizon runs in Orange.'
PRIME = (1 << 130) - 5


class Reference:
    """RFC 8439, RFC 5869 and the Horizon frame in pure Python, for comparison."""

    @staticmethod
    def _quarter_round(s, a, b, c, d):
        rotl = lambda v, n: ((v << n) | (v >> (32 - n))) & 0xffffffff
        s[a] = (s[a] + s[b]) & 0xffffffff; s[d] = rotl(s[d] ^ s[a], 16)
        s[c] = (s[c] + s[d]) & 0xffffffff; s[b] = rotl(s[b] ^ s[c], 12)
        s[a] = (s[a] + s[b]) & 0xffffffff; s[d] = rotl(s[d] ^ s[a], 8)
        s[c] = (s[c] + s[d]) & 0xffffffff; s[b] = rotl(s[b] ^ s[c], 7)

    @classmethod
    def chacha20_block(cls, key, counter, nonce):
        state = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, *struct.unpack('<8I', key),
                 counter & 0xffffffff, *struct.unpack('<3I', nonce)]
        working = list(state)
        for _ in range(10):
            for quad in [(0, 4, 8, 12), (1, 5, 9, 13), (2, 6, 10, 14), (3, 7, 11, 15),
                         (0, 5, 10, 15), (1, 6, 11, 12), (2, 7, 8, 13), (3, 4, 9, 14)]:
                cls._quarter_round(working, *quad)
        return struct.pack('<16I', *[(w + s) & 0xffffffff for w, s in zip(working, state)])

    @classmethod
    def chacha20(cls, key, counter, nonce, data):
        out = bytearray()
        for offset in range(0, len(data), 64):
            block = cls.chacha20_block(key, counter + offset // 64, nonce)
            out.extend(x ^ y for x, y in zip(data[offset:offset + 64], block))
        return bytes(out)

    @staticmethod
    def poly1305_mac(message, key):
        r = int.from_bytes(key[:16], 'little') & 0x0ffffffc0ffffffc0ffffffc0fffffff
        s = int.from_bytes(key[16:], 'little')
        acc = 0
        for offset in range(0, len(message), 16):
            chunk = message[offset:offset + 16]
            acc = ((acc + int.from_bytes(chunk, 'little') + (1 << (8 * len(chunk)))) * r) % PRIME
        return ((acc + s) & ((1 << 128) - 1)).to_bytes(16, 'little')

    @classmethod
    def _tag(cls, key, nonce, aad, ciphertext):
        data = (aad + bytes(-len(aad) % 16) + ciphertext + bytes(-len(ciphertext) % 16)
                + struct.pack('<QQ', len(aad), len(ciphertext)))
        return cls.poly1305_mac(data, cls.chacha20_block(key, 0, nonce)[:32])

    @classmethod
    def chacha20_poly1305_encrypt(cls, key, nonce, aad, plaintext):
        ciphertext = cls.chacha20(key, 1, nonce, plaintext)
        return ciphertext, cls._tag(key, nonce, aad, ciphertext)

    @classmethod
    def chacha20_poly1305_decrypt(cls, key, nonce, aad, ciphertext, tag):
        if not hmac.compare_digest(cls._tag(key, nonce, aad, ciphertext), tag):
            return None
        return cls.chacha20(key, 1, nonce, ciphertext)

    @staticmethod
    def hkdf_sha256(ikm, *, salt=b'', info=b'', length=32):
        prk = hmac.new(salt or bytes(32), ikm, hashlib.sha256).digest()
        output, block, counter = b'', b'', 1
        while len(output) < length:
            block = hmac.new(prk, block + info + bytes([counter]), hashlib.sha256).digest()
            output += block
            counter += 1
        return output[:length]

    @staticmethod
    def header(nonce, authorization_tag):
        header = {'nonce': nonce.hex(), 'authorization': {'authorization_tag': authorization_tag.hex()},
                  'plaintext_len': 32}
        return json.dumps(header, sort_keys=True, separators=(',', ':'), ensure_ascii=True).encode()

    @classmethod
    def seal(cls, root, nonce, authorization_tag, plaintext):
        """horizon_crypto.seal_framed for the fixture-shaped header."""
        header_bytes = cls.header(nonce, authorization_tag)
        salt = hashlib.sha256(MAGIC + header_bytes).digest()
        info = KDF_INFO + MAGIC + b':' + authorization_tag.hex().encode()
        key = cls.hkdf_sha256(root, salt=salt, info=info, length=32)
        aad = MAGIC + struct.pack('<I', len(header_bytes)) + header_bytes
        ciphertext, tag = cls.chacha20_poly1305_encrypt(key, nonce, aad, plaintext)
        return aad + ciphertext + tag


def orange_bytes(expression, count):
    return bytes(ENGINE.evaluate(f'    {expression}', f'Word[8]^{count}'))


def orange_bool(expression):
    return ENGINE.evaluate(f'    {expression}', 'Bool')


def lit(data):
    return array(literals(data))


class PrimitiveTests(unittest.TestCase):
    def test_sha256_padding_and_batch_boundaries(self):
        for size in [0, 1, 3, 55, 56, 63, 64, 65, 1023, 1024, 1025, 2048]:
            data = bytes(i % 251 for i in range(size))
            with self.subTest(size=size):
                self.assertEqual(ENGINE.sha256(data), hashlib.sha256(data).digest())

    def test_sha256_fips_examples_in_orange(self):
        # FIPS 180-4 examples: "abc" and the 56-byte two-block message, hashed
        # by the standalone program's sha256 from a three-block buffer.
        for message in [b'abc', b'abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq', bytes(183)]:
            with self.subTest(message=message[:8]):
                held = lit(message.ljust(192, b'\0'))
                self.assertEqual(orange_bytes(f'sha256({held}, {len(message)})', 32),
                                 hashlib.sha256(message).digest())

    def test_hmac_key_boundaries(self):
        for size in [0, 20, 32, 64, 65, 131]:
            key = bytes(range(size))
            with self.subTest(size=size):
                self.assertEqual(ENGINE.hmac_sha256(key, b'Daylight'), hmac.digest(key, b'Daylight', 'sha256'))

    def test_hmac_and_hkdf_in_orange(self):
        key, text = bytes(range(0x20, 0x40)), bytes(range(119))
        held = lit(text.ljust(192, b'\0'))
        self.assertEqual(orange_bytes(f'hmac({lit(key)}, {held}, 119)', 32), hmac.digest(key, text, 'sha256'))
        salt, info = bytes(range(0x40, 0x60)), bytes(range(118))
        self.assertEqual(orange_bytes(f'hkdf_32({lit(key)}, {lit(salt)}, {lit(info.ljust(192, bytes(1)))}, 118)', 32),
                         Reference.hkdf_sha256(key, salt=salt, info=info, length=32))

    def test_hkdf_rfc5869(self):
        result = ENGINE.hkdf_sha256(bytes.fromhex('0b' * 22), salt=bytes(range(13)), info=bytes(range(0xf0, 0xfa)), length=42)
        self.assertEqual(result.hex(), '3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865')
        self.assertEqual(ENGINE.hkdf_sha256(b'root', length=0), b'')
        self.assertEqual(ENGINE.hkdf_sha256(b'root', length=65), Reference.hkdf_sha256(b'root', length=65))

    def test_chacha_rfc8439(self):
        key, nonce = bytes(range(32)), bytes.fromhex('000000090000004a00000000')
        self.assertEqual(ENGINE.chacha20_block(key, 1, nonce), Reference.chacha20_block(key, 1, nonce))
        self.assertEqual(ENGINE.chacha20_block(key, 1, nonce)[:16].hex(), '10f1e7e4d13b5915500fdd1fa32071c4')
        nonce = bytes.fromhex('000000000000004a00000000')
        sunscreen = (b"Ladies and Gentlemen of the class of '99: If I could offer you only "
                     b'one tip for the future, sunscreen would be it.')
        self.assertEqual(ENGINE.chacha20(key, 1, nonce, sunscreen), Reference.chacha20(key, 1, nonce, sunscreen))

    def test_poly_rfc8439(self):
        key = bytes.fromhex('85d6be7857556d337f4452fe42d506a80103808afb0db2fd4abff6af4149f51b')
        self.assertEqual(ENGINE.poly1305_mac(b'Cryptographic Forum Research Group', key).hex(), 'a8061dc1305136c6c22b8baf0c0127a9')

    def test_poly_carries_partial_and_batch_boundaries(self):
        rng = random.Random(8439)
        cases = [(bytes(n), bytes(32)) for n in [0, 1, 16, 17]]
        cases += [(bytes([255]) * n, bytes([255]) * 32) for n in [15, 16, 17, 31, 32, 33, 1023, 1024, 1025, 2049]]
        cases += [(rng.randbytes(n), rng.randbytes(32)) for n in [1, 2, 15, 16, 17, 31, 32, 33, 64, 255, 256, 257, 4097]]
        for message, key in cases:
            with self.subTest(size=len(message), key=key[:2].hex()):
                self.assertEqual(ENGINE.poly1305_mac(message, key), Reference.poly1305_mac(message, key))

    def test_poly_reduction_and_tag_truncation(self):
        # absorb reduces modulo p after every block, and tag keeps 128 bits.
        r = 0x0ffffffc0ffffffc0ffffffc0fffffff
        block = (1 << 128) + (1 << 128) - 1
        for a in [0, 1, PRIME - 1]:
            with self.subTest(accumulator=a):
                self.assertEqual(ENGINE.evaluate(f'    absorb({a}, {r}, {block})', 'Int'), ((a + block) * r) % PRIME)
        for x in [0, (1 << 128) - 1, 1 << 128, PRIME, PRIME + (1 << 128) - 1]:
            with self.subTest(value=x):
                self.assertEqual(orange_bytes(f'tag({x})', 16), (x % (1 << 128)).to_bytes(16, 'little'))

    def test_aead_rfc8439_and_boundaries(self):
        key, nonce = bytes(range(0x80, 0xa0)), bytes.fromhex('070000004041424344454647')
        aad = bytes.fromhex('50515253c0c1c2c3c4c5c6c7')
        p = (b"Ladies and Gentlemen of the class of '99: If I could offer you only "
             b'one tip for the future, sunscreen would be it.')
        self.assertEqual(ENGINE.chacha20_poly1305_encrypt(key, nonce, aad, p), Reference.chacha20_poly1305_encrypt(key, nonce, aad, p))
        self.assertEqual(ENGINE.chacha20_poly1305_encrypt(key, nonce, aad, p)[1].hex(), '1ae10b594f09e26a7e902ecbd0600691')
        for n, a in [(0, 0), (0, 16), (1, 1), (15, 15), (16, 16), (17, 17), (63, 31), (64, 32), (65, 33), (255, 0), (256, 0), (257, 1), (1025, 1025)]:
            p, aad = bytes(i % 251 for i in range(n)), bytes(i % 241 for i in range(a))
            with self.subTest(plaintext=n, aad=a):
                sealed = ENGINE.chacha20_poly1305_encrypt(key, nonce, aad, p)
                self.assertEqual(sealed, Reference.chacha20_poly1305_encrypt(key, nonce, aad, p))
                self.assertEqual(ENGINE.chacha20_poly1305_decrypt(key, nonce, aad, *sealed), p)

    def test_rejects_before_plaintext_computation(self):
        key, nonce, aad, plain = bytes(32), bytes(12), b'header', b'message'
        c, t = ENGINE.chacha20_poly1305_encrypt(key, nonce, aad, plain)
        original = ENGINE.chacha20

        def forbidden(*args):
            self.fail('unauthenticated plaintext computation')
        ENGINE.chacha20 = forbidden
        try:
            mutations = [(b'\1' + key[1:], nonce, aad, c, t), (key, b'\1' + nonce[1:], aad, c, t),
                         (key, nonce, aad + b'x', c, t), (key, nonce, aad, bytes([c[0] ^ 1]) + c[1:], t),
                         (key, nonce, aad, c, bytes([t[0] ^ 1]) + t[1:])]
            for args in mutations:
                self.assertIsNone(ENGINE.chacha20_poly1305_decrypt(*args))
        finally:
            ENGINE.chacha20 = original

    def test_invalid_lengths_and_counter(self):
        calls = ENGINE.calls
        with self.assertRaises(ValueError):
            ENGINE.chacha20_block(bytes(31), 0, bytes(12))
        with self.assertRaises(ValueError):
            ENGINE.chacha20_block(bytes(32), 0, bytes(11))
        with self.assertRaises(ValueError):
            ENGINE.chacha20(bytes(32), 2**32 - 1, bytes(12), bytes(65))
        with self.assertRaises(ValueError):
            ENGINE.hkdf_sha256(b'x', length=8161)
        with self.assertRaises(ValueError):
            ENGINE.chacha20_poly1305_encrypt(bytes(32), bytes(12), b'', bytes(MAX_BYTES))
        with self.assertRaises(ValueError):
            ENGINE.chacha20_poly1305_decrypt(bytes(32), bytes(12), b'', b'', bytes(15))
        self.assertEqual(ENGINE.calls, calls)


class HorizonTests(unittest.TestCase):
    def test_standalone_program_prints_the_pinned_frame(self):
        proc = subprocess.run([ENGINE.orangec, 'eval', str(PROGRAM)], capture_output=True, text=True, check=True)
        self.assertEqual(proc.stderr, '')
        printed = {}
        for line in proc.stdout.splitlines():
            name, value = line.split(' = ', 1)
            printed[name] = value
        self.assertEqual(list(printed), [
            'daylight::initial_hash: Word[32]^8', 'daylight::prime: Int', 'daylight::magic: Word[8]^7',
            'daylight::root: Word[8]^32', 'daylight::nonce: Word[8]^12', 'daylight::authorization_tag: Word[8]^32',
            'daylight::plaintext: Word[8]^32', 'daylight::example: Word[8]^219', 'daylight::pinned_frame: Word[8]^219',
            'daylight::recovered: Word[8]^32'])
        frame = bytes(int(x, 16) for x in printed['daylight::example: Word[8]^219'][1:-1].split(', '))
        self.assertEqual(printed['daylight::example: Word[8]^219'], printed['daylight::pinned_frame: Word[8]^219'])
        self.assertEqual(frame, Reference.seal(ROOT, NONCE, AUTHORIZATION_TAG, PLAINTEXT))
        self.assertEqual(frame[-48:].hex(), '52a6747ccae6ef1548b86f1eabbfa9569f815f01841a4b7f17384facd1772fb5dbff1531ab89bb4161a5afca9ea0717c')
        self.assertEqual(printed['daylight::recovered: Word[8]^32'], printed['daylight::plaintext: Word[8]^32'])
        self.assertEqual(printed['daylight::root: Word[8]^32'], lit(ROOT))
        self.assertEqual(printed['daylight::nonce: Word[8]^12'], lit(NONCE))
        self.assertEqual(printed['daylight::authorization_tag: Word[8]^32'], lit(AUTHORIZATION_TAG))
        self.assertEqual(printed['daylight::plaintext: Word[8]^32'], lit(PLAINTEXT))
        if UPSTREAM is not None:
            from src import horizon_crypto as hc
            header = {'nonce': NONCE.hex(), 'authorization': {'authorization_tag': AUTHORIZATION_TAG.hex()},
                      'plaintext_len': 32}
            self.assertEqual(frame, hc.seal_framed(magic=MAGIC, header=header, plaintext=PLAINTEXT, root_key=ROOT))

    def test_seal_open_and_authentic_in_orange(self):
        rng = random.Random(2026)
        for trial in range(3):
            root, nonce = rng.randbytes(32), rng.randbytes(12)
            authorization_tag, plaintext = rng.randbytes(32), rng.randbytes(32)
            with self.subTest(trial=trial):
                frame = orange_bytes(f'seal({lit(root)}, {lit(nonce)}, {lit(authorization_tag)}, {lit(plaintext)})', 219)
                self.assertEqual(frame, Reference.seal(root, nonce, authorization_tag, plaintext))
                self.assertEqual(orange_bytes(f'open({lit(root)}, {lit(frame)})', 32), plaintext)
                self.assertIs(orange_bool(f'authentic({lit(root)}, {lit(frame)})'), True)
        # A changed root, header digit, ciphertext byte or tag byte is refused,
        # and open yields zeros without decrypting.
        frame = Reference.seal(ROOT, NONCE, AUTHORIZATION_TAG, PLAINTEXT)
        wrong_root = bytes([ROOT[0] ^ 1]) + ROOT[1:]
        self.assertIs(orange_bool(f'authentic({lit(wrong_root)}, {lit(frame)})'), False)
        self.assertEqual(orange_bytes(f'open({lit(wrong_root)}, {lit(frame)})', 32), bytes(32))
        for position in [0, 8, 50, 120, 180, 203, 218]:
            tampered = bytearray(frame)
            tampered[position] ^= 1
            with self.subTest(position=position):
                self.assertIs(orange_bool(f'authentic({lit(ROOT)}, {lit(tampered)})'), False)
                self.assertEqual(orange_bytes(f'open({lit(ROOT)}, {lit(tampered)})', 32), bytes(32))

    def test_real_vault_framing_and_evidence_checks(self):
        if UPSTREAM is None:
            self.skipTest('needs --upstream')
        from src import horizon_crypto as hc, horizon_vault as hv, horizon_policy as hp
        from src import canonical_json
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'vault'
            hv.init_vault(root)
            vault = hv.HorizonVault(root)
            state = UPSTREAM / 'daylight/v17-singularity/examples/state.current.json'
            nonce = bytes.fromhex('000000000000000000000001')
            kwargs = dict(name='orange.txt', plaintext=b'Daylight running in Orange.', state_path=state, nonce=nonce)
            reference = vault.seal_bytes(**kwargs)
            parsed = hc.parse_framed(reference, magic=hv.MAGIC)
            reference_key = hc.derive_key(vault._key(), magic=hv.MAGIC, header_bytes=parsed['header_bytes'],
                                          auth_tag=parsed['header']['authorization']['authorization_tag'])
            old = hc._AEAD
            with ENGINE.horizon(hc):
                self.assertEqual(hc.derive_key(vault._key(), magic=hv.MAGIC, header_bytes=parsed['header_bytes'],
                                               auth_tag=parsed['header']['authorization']['authorization_tag']),
                                 reference_key)
                actual = vault.seal_bytes(**kwargs)
                self.assertEqual(actual, reference)
                self.assertEqual(vault.open_bytes(sealed=reference, state_path=state), kwargs['plaintext'])
                for at in [-1, -17]:
                    tampered = bytearray(actual)
                    tampered[at] ^= 1
                    with self.assertRaises(hv.HorizonVaultRefused):
                        vault.open_bytes(sealed=bytes(tampered), state_path=state)
                header = dict(parsed['header'])
                header['name'] = 'changed.txt'
                tampered = hc.aad(hv.MAGIC, hc.frame_header(header)) + parsed['ciphertext'] + parsed['tag']
                with self.assertRaises(hv.HorizonVaultRefused):
                    vault.open_bytes(sealed=tampered, state_path=state)
                header = dict(parsed['header'])
                header['authorization'] = dict(header['authorization'])
                header['authorization']['authorization_tag'] = '0' * 64
                tampered = hc.aad(hv.MAGIC, hc.frame_header(header)) + parsed['ciphertext'] + parsed['tag']
                with self.assertRaises(hv.HorizonVaultRefused):
                    vault.open_bytes(sealed=tampered, state_path=state)
                with self.assertRaises(hv.HorizonVaultRefused):
                    vault.seal_bytes(**kwargs, policy=hp.policy_for_mode('declaration'))
                fixture = dict(kwargs, state_path=UPSTREAM / 'daylight/v17-singularity/examples/state.declaration-fixture.json')
                with self.assertRaises(hv.HorizonVaultRefused):
                    vault.seal_bytes(**fixture)
                with self.assertRaises(ValueError):
                    canonical_json.loads_json_no_floats('{"nonce":"a","nonce":"b"}')
            self.assertIs(hc._AEAD, old)
            self.assertEqual(vault.open_bytes(sealed=actual, state_path=state), kwargs['plaintext'])


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--orangec', required=True)
    parser.add_argument('--upstream')
    args, remaining = parser.parse_known_args()
    ENGINE = OrangeBackend(args.orangec)
    if args.upstream is not None:
        UPSTREAM = Path(args.upstream).resolve(strict=True)
        sys.path.insert(0, str(UPSTREAM / 'daylight/v17-singularity'))
    unittest.main(argv=[sys.argv[0]] + remaining, verbosity=2)
