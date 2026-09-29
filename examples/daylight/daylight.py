"""Daylight Horizon v17 backend: cryptographic arithmetic executes in Orange.

Python handles byte framing, padding, bounded batches, and verified-result control
flow. This is a research interpreter bridge, not a hardened secret-processing API.
"""
from __future__ import annotations

import argparse
from contextlib import contextmanager
from pathlib import Path
import re
import subprocess

CORE = Path(__file__).with_name('daylight.or')
IV = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
      0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]
MAX_BYTES = 1 << 20


def array(values):
    return '[' + ','.join(str(v) for v in values) + ']'


def literals(data):
    return [f'0x{x:02x}' for x in data]


def words(data):
    if len(data) % 4:
        raise ValueError('word input must be a multiple of four bytes')
    return [int.from_bytes(data[i:i+4], 'little') for i in range(0, len(data), 4)]


def packed(exprs, endian='little'):
    if len(exprs) % 4:
        raise ValueError('word expression input must be a multiple of four bytes')
    loader = 'le32' if endian == 'little' else 'be32'
    return array([loader + '(' + ','.join(exprs[i:i+4]) + ')'
                  for i in range(0, len(exprs), 4)])


def checked_bytes(data, label, length=None):
    if not isinstance(data, bytes):
        raise TypeError(f'{label} must be bytes')
    if len(data) > MAX_BYTES or (length is not None and len(data) != length):
        raise ValueError(f'invalid {label} length')
    return data


class OrangeBackend:
    """Drop-in for Horizon's four-method private AEAD backend interface."""

    def __init__(self, orangec, source=CORE):
        self.orangec = str(Path(orangec).resolve(strict=True))
        core = Path(source).read_text(encoding='utf-8')
        if not core.rstrip().endswith('}'):
            raise ValueError('invalid Orange core')
        self.core = core.rstrip()[:-1]
        self.calls = 0

    def evaluate(self, body, width, count=None):
        """Evaluate one entry point through stdin; never write inputs to disk."""
        ty = f'Word[{width}]' + (f'^{count}' if count is not None else '')
        source = self.core + f'\n spec result() -> {ty} {{\n{body}\n}}\n}}\n'
        self.calls += 1
        try:
            proc = subprocess.run([self.orangec, 'eval', '-'], input=source,
                                  text=True, capture_output=True, timeout=30, check=False)
        except (OSError, subprocess.TimeoutExpired) as exc:
            raise RuntimeError('Orange evaluator unavailable or timed out') from exc
        # Compiler diagnostics may include source values; do not relay them.
        if proc.returncode or proc.stderr:
            raise RuntimeError('Orange rejected the generated computation')
        prefix = f'daylight::result: {ty} = '
        if not proc.stdout.startswith(prefix) or not proc.stdout.endswith('\n'):
            raise RuntimeError('unexpected Orange result')
        value = proc.stdout[len(prefix):-1]
        if count is None:
            parts = [value]
        elif value.startswith('[') and value.endswith(']'):
            parts = value[1:-1].split(', ')
        else:
            raise RuntimeError('malformed Orange array')
        if len(parts) != (count if count is not None else 1):
            raise RuntimeError('incorrect Orange result length')
        if any(not re.fullmatch(r'0x[0-9a-f]{' + str(width // 4) + '}', p) for p in parts):
            raise RuntimeError('malformed Orange word')
        result = [int(p, 16) for p in parts]
        return result if count is not None else result[0]

    def _sha_expr(self, data):
        n = len(data)
        if n > MAX_BYTES + 128:
            raise ValueError('hash input exceeds research limit')
        padded = list(data) + ['0x80'] + ['0'] * ((55 - n) % 64)
        padded += literals((8 * n).to_bytes(8, 'big'))
        state = IV
        for base in range(0, len(padded), 1024):
            chunk = padded[base:base+1024]
            body = [f'let h0: Word[32]^8 = {array(state)};']
            for i, offset in enumerate(range(0, len(chunk), 64), 1):
                block = packed(chunk[offset:offset+64], 'big')
                body.append(f'let h{i}: Word[32]^8 = sha_compress(h{i-1},{block});')
            body.append(f'h{i}')
            state = self.evaluate('\n'.join(body), 32, 8)
        return b''.join(x.to_bytes(4, 'big') for x in state)

    def sha256(self, data):
        return self._sha_expr(literals(checked_bytes(data, 'hash input')))

    def hmac_sha256(self, key, data):
        checked_bytes(key, 'HMAC key')
        checked_bytes(data, 'HMAC input')
        if len(key) > 64:
            key = self.sha256(key)
        k = literals(key.ljust(64, b'\0'))
        inner = self._sha_expr([f'({v} ^ 0x36)' for v in k] + literals(data))
        return self._sha_expr([f'({v} ^ 0x5c)' for v in k] + literals(inner))

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
        block = self.evaluate(f'chacha_block({array(words(key))},{counter},{array(words(nonce))})',32,16)
        return b''.join(x.to_bytes(4, 'little') for x in block)

    def chacha20(self, key, counter, nonce, data):
        checked_bytes(key, 'key', 32)
        checked_bytes(nonce, 'nonce', 12)
        checked_bytes(data, 'message')
        blocks = (len(data) + 63) // 64
        if type(counter) is not int or counter < 0 or counter >= 2**32 or counter + blocks > 2**32:
            raise ValueError('ChaCha counter would wrap')
        out = bytearray()
        for base in range(0, len(data), 256):
            chunk = data[base:base+256]
            body = []
            for i in range((len(chunk)+63)//64):
                body.append(f'let b{i}: Word[32]^16 = chacha_block({array(words(key))},{counter+base//64+i},{array(words(nonce))});')
            exprs = []
            for i, byte in enumerate(chunk):
                word = f'b{i//64}[{(i%64)//4}]'
                if i % 4:
                    word = f'({word} >> {8*(i%4)})'
                exprs.append(f'0x{byte:02x} ^ ({word} as Word[8])')
            body.append(array(exprs))
            out.extend(self.evaluate('\n'.join(body),8,len(chunk)))
        return bytes(out)

    def poly1305_mac(self, message, key):
        checked_bytes(message, 'Poly1305 input')
        checked_bytes(key, 'Poly1305 key', 32)
        state = [0] * 5
        for base in range(0, len(message), 1024):
            chunk = message[base:base+1024]
            body = [f'let r: Word[64]^5 = poly_r({array(words(key[:16]))});',
                    f'let h0: Word[64]^5 = {array(state)};']
            for i, offset in enumerate(range(0, len(chunk), 16), 1):
                block = chunk[offset:offset+16]
                high = 1 << 24 if len(block) == 16 else 0
                if len(block) < 16:
                    block += b'\x01'
                block = block.ljust(16,b'\0')
                body.append(f'let h{i}: Word[64]^5 = poly_step(h{i-1},r,{array(words(block))},{high});')
            body.append(f'h{i}')
            state = self.evaluate('\n'.join(body),64,5)
        result = self.evaluate(f'poly_finish({array(state)},{array(words(key[16:]))})',32,4)
        return b''.join(x.to_bytes(4,'little') for x in result)

    def _tag(self, key, nonce, aad, ciphertext):
        checked_bytes(aad, 'associated data')
        checked_bytes(ciphertext, 'ciphertext')
        mac_data = (aad + bytes(-len(aad) % 16) + ciphertext + bytes(-len(ciphertext) % 16)
                    + len(aad).to_bytes(8,'little') + len(ciphertext).to_bytes(8,'little'))
        checked_bytes(mac_data, 'padded MAC input')
        return self.poly1305_mac(mac_data, self.chacha20_block(key,0,nonce)[:32])

    def chacha20_poly1305_encrypt(self, key, nonce, aad, plaintext):
        # Check combined bound before computing ciphertext.
        checked_bytes(aad, 'associated data'); checked_bytes(plaintext, 'plaintext')
        if ((len(aad)+15)//16 + (len(plaintext)+15)//16)*16 + 16 > MAX_BYTES:
            raise ValueError('combined AEAD input exceeds research limit')
        ciphertext = self.chacha20(key,1,nonce,plaintext)
        return ciphertext, self._tag(key,nonce,aad,ciphertext)

    def chacha20_poly1305_decrypt(self, key, nonce, aad, ciphertext, tag):
        checked_bytes(tag, 'tag', 16)
        expected = self._tag(key,nonce,aad,ciphertext)
        valid = self.evaluate(f'tag_equal({array(words(expected))},{array(words(tag))})',8)
        if valid != 1:
            return None
        # No plaintext computation occurs until Orange accepts the tag.
        return self.chacha20(key,1,nonce,ciphertext)

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
    parser = argparse.ArgumentParser(description='Run a public Daylight Orange test vector')
    parser.add_argument('--orangec', required=True)
    args = parser.parse_args()
    engine = OrangeBackend(args.orangec)
    key = bytes(range(0x80,0xa0))
    nonce = bytes.fromhex('070000004041424344454647')
    aad = bytes.fromhex('50515253c0c1c2c3c4c5c6c7')
    plaintext = (b"Ladies and Gentlemen of the class of '99: If I could offer you only "
                 b'one tip for the future, sunscreen would be it.')
    ciphertext, tag = engine.chacha20_poly1305_encrypt(key,nonce,aad,plaintext)
    if tag.hex() != '1ae10b594f09e26a7e902ecbd0600691':
        raise RuntimeError('RFC 8439 known-answer failure')
    if engine.chacha20_poly1305_decrypt(key,nonce,aad,ciphertext,tag) != plaintext:
        raise RuntimeError('roundtrip failure')
    print('RFC 8439 ciphertext:',ciphertext.hex())
    print('RFC 8439 tag:',tag.hex())
    print('Authenticated roundtrip: PASS')


if __name__ == '__main__':
    main()
