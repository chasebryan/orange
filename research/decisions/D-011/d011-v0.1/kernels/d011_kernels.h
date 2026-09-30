/*
 * D-011 v0.1 stand-in kernels: the C interface.
 *
 * These are contributor-written portable C reference kernels for the
 * proposed D-015 flagship families. They are NOT Orange output: Orange has
 * no native backend. The D-011 laboratory builds them for each candidate
 * target tuple only to measure target feasibility (known-answer agreement,
 * instruction inventory, control-flow trace equivalence, ABI agreement and
 * cost). This header also stands in for the "stable generated C ABI" of the
 * D-011 recommendation: fixed-width integers, pointer plus length, explicit
 * status returns, no structs by value.
 */
#ifndef D011_KERNELS_H
#define D011_KERNELS_H

#include <stddef.h>
#include <stdint.h>

/* SHA-2 (FIPS 180-4). */
void d011_sha256(uint8_t out[32], const uint8_t *msg, size_t len);
/* w[16], w[17], then the state after rounds 0 and 1 from the initial hash:
 * 18 words, each big-endian. The Orange oracle computes the same values. */
void d011_sha256_probe(uint8_t out[72], const uint8_t block[64]);
void d011_sha512(uint8_t out[64], const uint8_t *msg, size_t len);

/* HMAC and HKDF over SHA-256 (RFC 2104, RFC 5869). */
void d011_hmac_sha256(uint8_t out[32], const uint8_t *key, size_t key_len,
                      const uint8_t *msg, size_t msg_len);
int d011_hkdf_sha256(uint8_t *out, size_t out_len, const uint8_t *ikm,
                     size_t ikm_len, const uint8_t *salt, size_t salt_len,
                     const uint8_t *info, size_t info_len);

/* ChaCha20, Poly1305 and the AEAD (RFC 8439). */
void d011_chacha20_block(uint8_t out[64], const uint8_t key[32],
                         uint32_t counter, const uint8_t nonce[12]);
void d011_chacha20_xor(uint8_t *out, const uint8_t *in, size_t len,
                       const uint8_t key[32], uint32_t counter,
                       const uint8_t nonce[12]);
void d011_poly1305(uint8_t tag[16], const uint8_t *msg, size_t len,
                   const uint8_t key[32]);
void d011_aead_seal(uint8_t *out, const uint8_t key[32],
                    const uint8_t nonce[12], const uint8_t *aad,
                    size_t aad_len, const uint8_t *pt, size_t pt_len);
int d011_aead_open(uint8_t *out, const uint8_t key[32],
                   const uint8_t nonce[12], const uint8_t *aad,
                   size_t aad_len, const uint8_t *ct, size_t ct_len);

/* X25519 (RFC 7748). */
void d011_x25519(uint8_t out[32], const uint8_t scalar[32],
                 const uint8_t u[32]);

/* AES (FIPS 197) and GCM (SP 800-38D), table-free portable code. */
int d011_aes_expand_key(uint8_t rk[240], const uint8_t *key, size_t key_len);
int d011_aes_encrypt_block(uint8_t out[16], const uint8_t *key,
                           size_t key_len, const uint8_t in[16]);
int d011_aes_gcm_seal(uint8_t *out, const uint8_t *key, size_t key_len,
                      const uint8_t iv[12], const uint8_t *aad,
                      size_t aad_len, const uint8_t *pt, size_t pt_len);
int d011_aes_gcm_open(uint8_t *out, const uint8_t *key, size_t key_len,
                      const uint8_t iv[12], const uint8_t *aad,
                      size_t aad_len, const uint8_t *ct, size_t ct_len);

/* SHA-3 and SHAKE (FIPS 202): the symmetric core of ML-KEM and ML-DSA. */
void d011_sha3_256(uint8_t out[32], const uint8_t *msg, size_t len);
void d011_shake128(uint8_t *out, size_t out_len, const uint8_t *msg,
                   size_t len);

/* Crypto feature profile only: AES rounds and 64x64 carry-less multiply
 * through the target's crypto extension (AES-NI and PCLMULQDQ, Armv8 AES
 * and PMULL, RISC-V Zkne and Zbkc), with portable glue for GCM. */
void d011_accel_aes_rounds(uint8_t out[16], const uint8_t *rk, int rounds,
                           const uint8_t in[16]);
void d011_accel_clmul64(uint64_t a, uint64_t b, uint64_t *hi, uint64_t *lo);
int d011_accel_aes_encrypt_block(uint8_t out[16], const uint8_t *key,
                                 size_t key_len, const uint8_t in[16]);
int d011_accel_aes_gcm_seal(uint8_t *out, const uint8_t *key, size_t key_len,
                            const uint8_t iv[12], const uint8_t *aad,
                            size_t aad_len, const uint8_t *pt, size_t pt_len);
int d011_accel_aes_gcm_open(uint8_t *out, const uint8_t *key, size_t key_len,
                            const uint8_t iv[12], const uint8_t *aad,
                            size_t aad_len, const uint8_t *ct, size_t ct_len);

/* Shared byte helpers (D011_PART_CT). */
void d011_wipe(void *p, size_t len);
uint32_t d011_ct_differs(const uint8_t *a, const uint8_t *b, size_t len);

/* Laboratory-internal: the streaming SHA-256 that HMAC and HKDF share. */
struct d011_sha256_ctx {
    uint32_t h[8];
    uint8_t buf[64];
    uint64_t total;
    size_t used;
};

void d011_sha256_init(struct d011_sha256_ctx *ctx);
void d011_sha256_update(struct d011_sha256_ctx *ctx, const uint8_t *msg,
                        size_t len);
void d011_sha256_final(struct d011_sha256_ctx *ctx, uint8_t out[32]);

#endif
