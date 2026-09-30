"""Daylight Horizon v17 backend: the cryptographic arithmetic runs in Orange.

Python frames bytes, pads, batches and checks results. Hashing, HMAC and HKDF,
the cipher, the authenticator and the tag comparison are the specifications of
daylight-horizon.or, evaluated by `orangec eval` on source sent through stdin.
This is a research interpreter bridge, not a hardened secret-processing API.
"""
from __future__ import annotations

import argparse
from contextlib import contextmanager
from pathlib import Path
import re
import subprocess

PROGRAM = Path(__file__).with_name('daylight-horizon.or')
MARKER = '  // ---- Examples ----'
MAX_BYTES = 1 << 20
BLOCKS_PER_CALL = 16
POLY_BLOCKS_PER_CALL = 64


def array(values):
    return '[' + ', '.join(str(v) for v in values) + ']'


def literals(data):
    return [f'0x{x:02x}' for x in data]


def checked_bytes(data, label, length=None):
    if not isinstance(data, bytes):
        raise TypeError(f'{label} must be bytes')
    if len(data) > MAX_BYTES or (length is not None and len(data) != length):
        raise ValueError(f'invalid {label} length')
    return data


def sha256_padding(length):
    """FIPS 180-4 section 5.1.1: a 1 bit, zeros, and the bit length."""
    return b'\x80' + bytes((55 - length) % 64) + (8 * length).to_bytes(8, 'big')


class OrangeBackend:
    """Drop-in for Horizon's four-method private AEAD backend interface."""

    def __init__(self, orangec, source=PROGRAM):
        self.orangec = str(Path(orangec).resolve(strict=True))
        text = Path(source).read_text(encoding='utf-8')
        library, marker, _ = text.partition(MARKER)
        if not marker:
            raise ValueError('invalid Orange program: no examples marker')
        # Everything before the marker is the module's specifications with
        # parameters and its named constants; the examples stay out, because
        # `orangec eval` prints every specification without parameters.
        self.library = library
        self.calls = 0

    def evaluate(self, body, ty):
        """Evaluate one entry point of type `ty` through stdin; never write inputs to disk."""
        source = self.library + f'\n  spec result() -> {ty} {{\n{body}\n  }}\n}}\n'
        self.calls += 1
        try:
            proc = subprocess.run([self.orangec, 'eval', '-'], input=source,
                                  text=True, capture_output=True, timeout=30, check=False)
        except (OSError, subprocess.TimeoutExpired) as exc:
            raise RuntimeError('Orange evaluator unavailable or timed out') from exc
        # Compiler diagnostics may include source values; do not relay them.
        if proc.returncode or proc.stderr:
            raise RuntimeError('Orange rejected the generated computation')
        if not proc.stdout.endswith('\n'):
            raise RuntimeError('unexpected Orange result')
        # `result` is the last specification, so it is the last line printed.
        prefix = f'daylight::result: {ty} = '
        line = proc.stdout[:-1].rsplit('\n', 1)[-1]
        if not line.startswith(prefix):
            raise RuntimeError('unexpected Orange result')
        return self.parse(line[len(prefix):], ty)

    @staticmethod
    def parse(value, ty):
        if ty == 'Bool':
            if value not in ('true', 'false'):
                raise RuntimeError('malformed Orange truth value')
            return value == 'true'
        if ty == 'Int':
            if not re.fullmatch(r'-?[0-9]+', value):
                raise RuntimeError('malformed Orange integer')
            return int(value)
        match = re.fullmatch(r'Word\[(\d+)\](?:\^(\d+))?', ty)
        if not match:
            raise RuntimeError('unsupported Orange type')
        width, count = int(match.group(1)), match.group(2)
        if count is None:
            parts = [value]
        elif value.startswith('[') and value.endswith(']'):
            parts = value[1:-1].split(', ')
        else:
            raise RuntimeError('malformed Orange array')
        if len(parts) != (int(count) if count is not None else 1):
            raise RuntimeError('incorrect Orange result length')
        if any(not re.fullmatch(r'0x[0-9a-f]{' + str(width // 4) + '}', p) for p in parts):
            raise RuntimeError('malformed Orange word')
        result = [int(p, 16) for p in parts]
        return result if count is not None else result[0]

    def _sha256_blocks(self, prefix_blocks, data):
        """SHA-256 of `prefix_blocks`, whole 64-byte block expressions, followed by `data`."""
        n = 64 * len(prefix_blocks) + len(data)
        if n > MAX_BYTES + 128:
            raise ValueError('hash input exceeds research limit')
        padded = data + sha256_padding(n)
        blocks = list(prefix_blocks)
        blocks += [array(literals(padded[i:i + 64])) for i in range(0, len(padded), 64)]
        state = 'initial_hash()'
        while blocks:
            batch, blocks = blocks[:BLOCKS_PER_CALL], blocks[BLOCKS_PER_CALL:]
            body = [f'    let h0: Word[32]^8 = {state};']
            for i, block in enumerate(batch, 1):
                body.append(f'    let h{i}: Word[32]^8 = compress(h{i - 1}, {block});')
            if blocks:
                body.append(f'    h{len(batch)}')
                state = array(f'0x{x:08x}' for x in self.evaluate('\n'.join(body), 'Word[32]^8'))
            else:
                body.append(f'    digest(h{len(batch)})')
                return bytes(self.evaluate('\n'.join(body), 'Word[8]^32'))
        raise AssertionError('unreachable')

    def sha256(self, data):
        return self._sha256_blocks([], checked_bytes(data, 'hash input'))

    def hmac_sha256(self, key, data):
        """RFC 2104: the padded key blocks are formed in Orange by keyed_block."""
        checked_bytes(key, 'HMAC key')
        checked_bytes(data, 'HMAC input')
        if len(key) > 64:
            key = self.sha256(key)
        padded = array(literals(key.ljust(64, b'\0')))
        inner = self._sha256_blocks([f'keyed_block({padded}, 0x36)'], data)
        return self._sha256_blocks([f'keyed_block({padded}, 0x5c)'], inner)

    def hkdf_sha256(self, ikm, *, salt=b'', info=b'', length=32):
        checked_bytes(ikm, 'HKDF input')
        checked_bytes(salt, 'HKDF salt')
        checked_bytes(info, 'HKDF info')
        if type(length) is not int or not 0 <= length <= 255 * 32:
            raise ValueError('invalid HKDF output length')
        if len(info) > MAX_BYTES - 33:
            raise ValueError('HKDF info exceeds research limit')
        prk = self.hmac_sha256(salt or bytes(32), ikm)
        output, block = b'', b''
        for counter in range(1, (length + 31) // 32 + 1):
            block = self.hmac_sha256(prk, block + info + bytes([counter]))
            output += block
        return output[:length]

    def chacha20_block(self, key, counter, nonce):
        checked_bytes(key, 'key', 32)
        checked_bytes(nonce, 'nonce', 12)
        if type(counter) is not int or not 0 <= counter < 2**32:
            raise ValueError('invalid ChaCha counter')
        stream = f'keystream({array(literals(key))}, {counter}, {array(literals(nonce))})'
        return bytes(self.evaluate(f'    {stream}', 'Word[8]^64'))

    def chacha20(self, key, counter, nonce, data):
        checked_bytes(key, 'key', 32)
        checked_bytes(nonce, 'nonce', 12)
        checked_bytes(data, 'message')
        blocks = (len(data) + 63) // 64
        if type(counter) is not int or counter < 0 or counter >= 2**32 or counter + blocks > 2**32:
            raise ValueError('ChaCha counter would wrap')
        out = bytearray()
        for i in range(blocks):
            chunk = data[64 * i:64 * i + 64]
            padded = array(literals(chunk.ljust(64, b'\0')))
            stream = f'keystream({array(literals(key))}, {counter + i}, {array(literals(nonce))})'
            block = f'xor_64({padded}, {stream})'
            out.extend(self.evaluate(f'    {block}', 'Word[8]^64')[:len(chunk)])
        return bytes(out)

    def poly1305_mac(self, message, key):
        """RFC 8439 section 2.5: the accumulator is an exact integer modulo 2^130 - 5."""
        checked_bytes(message, 'Poly1305 input')
        checked_bytes(key, 'Poly1305 key', 32)
        blocks = []
        for offset in range(0, len(message), 16):
            block = message[offset:offset + 16]
            whole = len(block) == 16
            if not whole:
                block = (block + b'\x01').ljust(16, b'\0')
            blocks.append(f'block_value({array(literals(block))}, {1 if whole else 0})')
        accumulator = '0'
        while blocks:
            batch, blocks = blocks[:POLY_BLOCKS_PER_CALL], blocks[POLY_BLOCKS_PER_CALL:]
            body = [f'    let r: Int = poly_r({array(literals(key))});',
                    f'    let a0: Int = {accumulator};']
            for i, block in enumerate(batch, 1):
                body.append(f'    let a{i}: Int = absorb(a{i - 1}, r, {block});')
            body.append(f'    a{len(batch)}')
            accumulator = str(self.evaluate('\n'.join(body), 'Int'))
        final = f'    tag({accumulator} + poly_s({array(literals(key))}))'
        return bytes(self.evaluate(final, 'Word[8]^16'))

    def _tag(self, key, nonce, aad, ciphertext):
        """RFC 8439 section 2.8: the authenticated data, then Poly1305 under the one-time key."""
        checked_bytes(aad, 'associated data')
        checked_bytes(ciphertext, 'ciphertext')
        mac_data = (aad + bytes(-len(aad) % 16) + ciphertext + bytes(-len(ciphertext) % 16)
                    + len(aad).to_bytes(8, 'little') + len(ciphertext).to_bytes(8, 'little'))
        checked_bytes(mac_data, 'padded MAC input')
        return self.poly1305_mac(mac_data, self.chacha20_block(key, 0, nonce)[:32])

    def chacha20_poly1305_encrypt(self, key, nonce, aad, plaintext):
        # Check the combined bound before computing the ciphertext.
        checked_bytes(aad, 'associated data')
        checked_bytes(plaintext, 'plaintext')
        if ((len(aad) + 15) // 16 + (len(plaintext) + 15) // 16) * 16 + 16 > MAX_BYTES:
            raise ValueError('combined AEAD input exceeds research limit')
        ciphertext = self.chacha20(key, 1, nonce, plaintext)
        return ciphertext, self._tag(key, nonce, aad, ciphertext)

    def chacha20_poly1305_decrypt(self, key, nonce, aad, ciphertext, tag):
        checked_bytes(tag, 'tag', 16)
        expected = self._tag(key, nonce, aad, ciphertext)
        same = f'    same_tag({array(literals(expected))}, {array(literals(tag))})'
        if self.evaluate(same, 'Bool') is not True:
            return None
        # No plaintext computation occurs until Orange accepts the tag.
        return self.chacha20(key, 1, nonce, ciphertext)

    @contextmanager
    def horizon(self, module):
        """Temporarily substitute arithmetic beneath unchanged Horizon policy.

        The upstream private interface is pinned in README.md. Module substitution
        is process-global: use only in a single-threaded reference/research process.
        """
        old = module._AEAD
        module._AEAD = self
        try:
            yield
        finally:
            module._AEAD = old


def main():
    parser = argparse.ArgumentParser(description='Run the public RFC 8439 AEAD vector through Orange')
    parser.add_argument('--orangec', required=True)
    args = parser.parse_args()
    engine = OrangeBackend(args.orangec)
    key = bytes(range(0x80, 0xa0))
    nonce = bytes.fromhex('070000004041424344454647')
    aad = bytes.fromhex('50515253c0c1c2c3c4c5c6c7')
    plaintext = (b"Ladies and Gentlemen of the class of '99: If I could offer you only "
                 b'one tip for the future, sunscreen would be it.')
    ciphertext, tag = engine.chacha20_poly1305_encrypt(key, nonce, aad, plaintext)
    if tag.hex() != '1ae10b594f09e26a7e902ecbd0600691':
        raise RuntimeError('RFC 8439 known-answer failure')
    if engine.chacha20_poly1305_decrypt(key, nonce, aad, ciphertext, tag) != plaintext:
        raise RuntimeError('roundtrip failure')
    print('RFC 8439 ciphertext:', ciphertext.hex())
    print('RFC 8439 tag:', tag.hex())
    print('Authenticated roundtrip: PASS')


if __name__ == '__main__':
    main()
