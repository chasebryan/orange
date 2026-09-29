"""Local standards, cross-implementation, and fail-closed tests."""
import argparse
import hashlib
import hmac
import importlib.util
import os
from pathlib import Path
import random
import sys
import tempfile
import unittest

from daylight import OrangeBackend, MAX_BYTES

ENGINE = None
UPSTREAM = None
REF = None


class PrimitiveTests(unittest.TestCase):
    def test_sha256_padding_and_batch_boundaries(self):
        for size in [0, 1, 3, 55, 56, 63, 64, 65, 1023, 1024, 1025, 2048]:
            data = bytes(i % 251 for i in range(size))
            with self.subTest(size=size):
                self.assertEqual(ENGINE.sha256(data), hashlib.sha256(data).digest())

    def test_hmac_key_boundaries(self):
        for size in [0, 20, 32, 64, 65, 131]:
            key = bytes(range(size))
            with self.subTest(size=size):
                self.assertEqual(ENGINE.hmac_sha256(key,b'Daylight'), hmac.digest(key,b'Daylight','sha256'))

    def test_hkdf_rfc5869(self):
        result = ENGINE.hkdf_sha256(bytes.fromhex('0b'*22),salt=bytes(range(13)),info=bytes(range(0xf0,0xfa)),length=42)
        self.assertEqual(result.hex(),'3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865')
        self.assertEqual(ENGINE.hkdf_sha256(b'root',length=0), b'')
        self.assertEqual(ENGINE.hkdf_sha256(b'root',length=65),REF.hkdf_sha256(b'root',length=65))

    def test_chacha_rfc8439(self):
        key, nonce = bytes(range(32)), bytes.fromhex('000000090000004a00000000')
        self.assertEqual(ENGINE.chacha20_block(key,1,nonce),REF.chacha20_block(key,1,nonce))
        self.assertEqual(ENGINE.chacha20_block(key,1,nonce)[:16].hex(),'10f1e7e4d13b5915500fdd1fa32071c4')

    def test_poly_rfc8439(self):
        key=bytes.fromhex('85d6be7857556d337f4452fe42d506a80103808afb0db2fd4abff6af4149f51b')
        self.assertEqual(ENGINE.poly1305_mac(b'Cryptographic Forum Research Group',key).hex(),'a8061dc1305136c6c22b8baf0c0127a9')

    def test_poly_carries_partial_and_batch_boundaries(self):
        rng=random.Random(8439)
        cases=[(bytes(n),bytes(32)) for n in [0,1,16,17]]
        cases += [(bytes([255])*n,bytes([255])*32) for n in [15,16,17,31,32,33,1023,1024,1025,2049]]
        cases += [(rng.randbytes(n),rng.randbytes(32)) for n in [1,2,15,16,17,31,32,33,64,255,256,257,4097]]
        for message,key in cases:
            with self.subTest(size=len(message),key=key[:2].hex()):
                self.assertEqual(ENGINE.poly1305_mac(message,key),REF.poly1305_mac(message,key))

    def test_poly_final_reduction_and_addition_carry(self):
        from daylight import array, words
        prime = (1 << 130) - 5
        for accumulator in [0, prime - 1, prime, prime + 1, (1 << 130) - 1]:
            limbs = [(accumulator >> (26 * i)) & 0x3ffffff for i in range(5)]
            for pad in [bytes(16), bytes([255]) * 16]:
                result = ENGINE.evaluate(f'poly_finish({array(limbs)},{array(words(pad))})',32,4)
                actual = b''.join(x.to_bytes(4,'little') for x in result)
                expected = ((accumulator % prime + int.from_bytes(pad,'little')) % (1 << 128)).to_bytes(16,'little')
                self.assertEqual(actual,expected)

    def test_aead_rfc8439_and_boundaries(self):
        key=bytes(range(0x80,0xa0)); nonce=bytes.fromhex('070000004041424344454647')
        aad=bytes.fromhex('50515253c0c1c2c3c4c5c6c7')
        p=b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it."
        self.assertEqual(ENGINE.chacha20_poly1305_encrypt(key,nonce,aad,p),REF.chacha20_poly1305_encrypt(key,nonce,aad,p))
        self.assertEqual(ENGINE.chacha20_poly1305_encrypt(key,nonce,aad,p)[1].hex(),'1ae10b594f09e26a7e902ecbd0600691')
        for n,a in [(0,0),(0,16),(1,1),(15,15),(16,16),(17,17),(63,31),(64,32),(65,33),(255,0),(256,0),(257,1),(1025,1025)]:
            p=bytes(i%251 for i in range(n)); aad=bytes(i%241 for i in range(a))
            with self.subTest(plaintext=n,aad=a):
                sealed=ENGINE.chacha20_poly1305_encrypt(key,nonce,aad,p)
                self.assertEqual(sealed,REF.chacha20_poly1305_encrypt(key,nonce,aad,p))
                self.assertEqual(ENGINE.chacha20_poly1305_decrypt(key,nonce,aad,*sealed),p)

    def test_rejects_before_plaintext_computation(self):
        key=bytes(32); nonce=bytes(12); aad=b'header'; plain=b'message'
        c,t=ENGINE.chacha20_poly1305_encrypt(key,nonce,aad,plain)
        original=ENGINE.chacha20
        def forbidden(*args):
            self.fail('unauthenticated plaintext computation')
        ENGINE.chacha20=forbidden
        try:
            mutations=[(b'\1'+key[1:],nonce,aad,c,t),(key,b'\1'+nonce[1:],aad,c,t),
                       (key,nonce,aad+b'x',c,t),(key,nonce,aad,bytes([c[0]^1])+c[1:],t),
                       (key,nonce,aad,c,bytes([t[0]^1])+t[1:])]
            for args in mutations:
                self.assertIsNone(ENGINE.chacha20_poly1305_decrypt(*args))
        finally:
            ENGINE.chacha20=original

    def test_invalid_lengths_and_counter(self):
        calls=ENGINE.calls
        with self.assertRaises(ValueError): ENGINE.chacha20_block(bytes(31),0,bytes(12))
        with self.assertRaises(ValueError): ENGINE.chacha20_block(bytes(32),0,bytes(11))
        with self.assertRaises(ValueError): ENGINE.chacha20(bytes(32),2**32-1,bytes(12),bytes(65))
        with self.assertRaises(ValueError): ENGINE.hkdf_sha256(b'x',length=8161)
        with self.assertRaises(ValueError): ENGINE.chacha20_poly1305_encrypt(bytes(32),bytes(12),b'',bytes(MAX_BYTES))
        with self.assertRaises(ValueError): ENGINE.chacha20_poly1305_decrypt(bytes(32),bytes(12),b'',b'',bytes(15))
        self.assertEqual(ENGINE.calls,calls)


class HorizonTests(unittest.TestCase):
    def test_standalone_orange_fixture_is_reproducible(self):
        from emit_example import render, ROOT, PLAINTEXT, HEADER, MAGIC
        from src import horizon_crypto as hc
        import subprocess
        path=Path(__file__).with_name('daylight-horizon.or')
        self.assertEqual(path.read_text(),render())
        proc=subprocess.run([ENGINE.orangec,'eval',str(path)],capture_output=True,text=True,check=True)
        self.assertEqual(proc.stderr,'')
        actual=bytes(int(x,16) for x in proc.stdout.split(' = [')[1].split(']')[0].split(', '))
        self.assertEqual(actual,hc.seal_framed(magic=MAGIC,header=HEADER,plaintext=PLAINTEXT,root_key=ROOT))

    def test_real_vault_framing_and_evidence_checks(self):
        from src import horizon_crypto as hc, horizon_vault as hv, horizon_policy as hp
        from src import canonical_json
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)/'vault'
            hv.init_vault(root)
            vault=hv.HorizonVault(root)
            state=UPSTREAM/'daylight/v17-singularity/examples/state.current.json'
            nonce=bytes.fromhex('000000000000000000000001')
            kwargs=dict(name='orange.txt',plaintext=b'Daylight running in Orange.',state_path=state,nonce=nonce)
            reference=vault.seal_bytes(**kwargs)
            parsed=hc.parse_framed(reference,magic=hv.MAGIC)
            reference_key=hc.derive_key(vault._key(),magic=hv.MAGIC,header_bytes=parsed['header_bytes'],auth_tag=parsed['header']['authorization']['authorization_tag'])
            old=hc._AEAD
            with ENGINE.horizon(hc):
                self.assertEqual(hc.derive_key(vault._key(),magic=hv.MAGIC,header_bytes=parsed['header_bytes'],auth_tag=parsed['header']['authorization']['authorization_tag']),reference_key)
                actual=vault.seal_bytes(**kwargs)
                self.assertEqual(actual,reference)
                self.assertEqual(vault.open_bytes(sealed=reference,state_path=state),kwargs['plaintext'])
                for at in [-1,-17]:
                    tampered=bytearray(actual); tampered[at]^=1
                    with self.assertRaises(hv.HorizonVaultRefused): vault.open_bytes(sealed=bytes(tampered),state_path=state)
                header=dict(parsed['header']); header['name']='changed.txt'
                tampered=hc.aad(hv.MAGIC,hc.frame_header(header))+parsed['ciphertext']+parsed['tag']
                with self.assertRaises(hv.HorizonVaultRefused): vault.open_bytes(sealed=tampered,state_path=state)
                header=dict(parsed['header']); header['authorization']=dict(header['authorization']); header['authorization']['authorization_tag']='0'*64
                tampered=hc.aad(hv.MAGIC,hc.frame_header(header))+parsed['ciphertext']+parsed['tag']
                with self.assertRaises(hv.HorizonVaultRefused): vault.open_bytes(sealed=tampered,state_path=state)
                with self.assertRaises(hv.HorizonVaultRefused): vault.seal_bytes(**kwargs,policy=hp.policy_for_mode('declaration'))
                fixture=dict(kwargs,state_path=UPSTREAM/'daylight/v17-singularity/examples/state.declaration-fixture.json')
                with self.assertRaises(hv.HorizonVaultRefused): vault.seal_bytes(**fixture)
                with self.assertRaises(ValueError): canonical_json.loads_json_no_floats('{"nonce":"a","nonce":"b"}')
            self.assertIs(hc._AEAD,old)
            self.assertEqual(vault.open_bytes(sealed=actual,state_path=state),kwargs['plaintext'])


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--orangec',required=True)
    parser.add_argument('--upstream',required=True)
    args,remaining=parser.parse_known_args()
    UPSTREAM=Path(args.upstream).resolve(strict=True)
    ENGINE=OrangeBackend(args.orangec)
    refpath=UPSTREAM/'daylight/v15-meridian/src/aead.py'
    spec=importlib.util.spec_from_file_location('daylight_reference',refpath)
    REF=importlib.util.module_from_spec(spec); spec.loader.exec_module(REF)
    sys.path.insert(0,str(UPSTREAM/'daylight/v17-singularity'))
    unittest.main(argv=[sys.argv[0]]+remaining,verbosity=2)
