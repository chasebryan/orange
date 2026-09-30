/*
 * D-011 v0.1 stand-in kernels: the crypto feature profile, built only when
 * the laboratory defines D011_CRYPTO_PROFILE and targets a CPU with the
 * extension. Contributor-written C, NOT Orange output.
 *
 * D011_PART_ACCEL is the only target-specific kernel code: AES rounds and
 * a 64x64-bit carry-less product through AES-NI and PCLMULQDQ (x86-64),
 * Armv8 AES and PMULL (AArch64), or scalar Zkne and Zbkc (RV64).
 * D011_PART_GCM_GLUE is portable C that builds AES and AES-GCM on them.
 */
#include "d011_kernels.h"

/* ======================================================================
 * Crypto-profile AES and AES-GCM glue, portable C. The AES rounds and
 * the 64x64-bit carry-less product come from D011_PART_ACCEL. GHASH here uses
 * a different algorithm from the portable kernel: operands are
 * bit-reversed into polynomial order, multiplied with four carry-less
 * products, reduced modulo x^128 + x^7 + x^2 + x + 1 with shifts, and
 * reversed back. Built only in the crypto feature profile.
 */
#if defined(D011_PART_GCM_GLUE)

static uint64_t load_be64(const uint8_t *p)
{
    uint64_t v = 0;
    for (int i = 0; i < 8; i++) {
        v = (v << 8) | p[i];
    }
    return v;
}

static void store_be64(uint8_t *p, uint64_t v)
{
    for (int i = 7; i >= 0; i--) {
        p[i] = (uint8_t)v;
        v >>= 8;
    }
}

static uint64_t bitrev64(uint64_t v)
{
    v = ((v >> 1) & 0x5555555555555555ULL) | ((v & 0x5555555555555555ULL) << 1);
    v = ((v >> 2) & 0x3333333333333333ULL) | ((v & 0x3333333333333333ULL) << 2);
    v = ((v >> 4) & 0x0f0f0f0f0f0f0f0fULL) | ((v & 0x0f0f0f0f0f0f0f0fULL) << 4);
    v = ((v >> 8) & 0x00ff00ff00ff00ffULL) | ((v & 0x00ff00ff00ff00ffULL) << 8);
    v = ((v >> 16) & 0x0000ffff0000ffffULL) | ((v & 0x0000ffff0000ffffULL) << 16);
    return (v >> 32) | (v << 32);
}

static void ghash_mul_clmul(uint8_t out[16], const uint8_t x[16], const uint8_t y[16])
{
    uint64_t a0 = bitrev64(load_be64(x)), a1 = bitrev64(load_be64(x + 8));
    uint64_t b0 = bitrev64(load_be64(y)), b1 = bitrev64(load_be64(y + 8));
    uint64_t p0h, p0l, p1h, p1l, p2h, p2l, p3h, p3l;
    uint64_t w0, w1, w2, w3, t0, t1, ov, r0, r1;
    d011_accel_clmul64(a0, b0, &p0h, &p0l);
    d011_accel_clmul64(a0, b1, &p1h, &p1l);
    d011_accel_clmul64(a1, b0, &p2h, &p2l);
    d011_accel_clmul64(a1, b1, &p3h, &p3l);
    w0 = p0l;
    w1 = p0h ^ p1l ^ p2l;
    w2 = p3l ^ p1h ^ p2h;
    w3 = p3h;
    t0 = w2 ^ (w2 << 1) ^ (w2 << 2) ^ (w2 << 7);
    t1 = w3 ^ ((w3 << 1) | (w2 >> 63)) ^ ((w3 << 2) | (w2 >> 62)) ^ ((w3 << 7) | (w2 >> 57));
    ov = (w3 >> 63) ^ (w3 >> 62) ^ (w3 >> 57);
    r0 = w0 ^ t0 ^ ov ^ (ov << 1) ^ (ov << 2) ^ (ov << 7);
    r1 = w1 ^ t1;
    store_be64(out, bitrev64(r0));
    store_be64(out + 8, bitrev64(r1));
}

int d011_accel_aes_encrypt_block(uint8_t out[16], const uint8_t *key, size_t key_len,
                                 const uint8_t in[16])
{
    uint8_t rk[240];
    int nr = d011_aes_expand_key(rk, key, key_len);
    if (nr < 0) {
        return -1;
    }
    d011_accel_aes_rounds(out, rk, nr, in);
    d011_wipe(rk, sizeof rk);
    return 0;
}

static void ghash_update(uint8_t acc[16], const uint8_t h[16], const uint8_t *data, size_t len)
{
    uint8_t block[16];
    while (len > 0) {
        size_t take = len < 16 ? len : 16;
        for (size_t i = 0; i < 16; i++) {
            block[i] = 0;
        }
        for (size_t i = 0; i < take; i++) {
            block[i] = data[i];
        }
        for (int i = 0; i < 16; i++) {
            block[i] ^= acc[i];
        }
        ghash_mul_clmul(acc, block, h);
        data += take;
        len -= take;
    }
}

static void gcm_ctr(uint8_t *out, const uint8_t *in, size_t len, const uint8_t *rk, int nr,
                    const uint8_t j0[16])
{
    uint8_t ctr[16];
    uint8_t ks[16];
    uint32_t count = ((uint32_t)j0[12] << 24) | ((uint32_t)j0[13] << 16) |
                     ((uint32_t)j0[14] << 8) | (uint32_t)j0[15];
    for (int i = 0; i < 12; i++) {
        ctr[i] = j0[i];
    }
    while (len > 0) {
        size_t take = len < 16 ? len : 16;
        count++;
        ctr[12] = (uint8_t)(count >> 24);
        ctr[13] = (uint8_t)(count >> 16);
        ctr[14] = (uint8_t)(count >> 8);
        ctr[15] = (uint8_t)count;
        d011_accel_aes_rounds(ks, rk, nr, ctr);
        for (size_t i = 0; i < take; i++) {
            out[i] = (uint8_t)(in[i] ^ ks[i]);
        }
        out += take;
        in += take;
        len -= take;
    }
    d011_wipe(ks, sizeof ks);
}

static void gcm_tag(uint8_t tag[16], const uint8_t *rk, int nr, const uint8_t j0[16],
                    const uint8_t *aad, size_t aad_len, const uint8_t *ct, size_t ct_len)
{
    static const uint8_t zero[16] = {0};
    uint8_t h[16];
    uint8_t acc[16] = {0};
    uint8_t lengths[16];
    uint8_t ek[16];
    d011_accel_aes_rounds(h, rk, nr, zero);
    ghash_update(acc, h, aad, aad_len);
    ghash_update(acc, h, ct, ct_len);
    store_be64(lengths, (uint64_t)aad_len << 3);
    store_be64(lengths + 8, (uint64_t)ct_len << 3);
    ghash_update(acc, h, lengths, 16);
    d011_accel_aes_rounds(ek, rk, nr, j0);
    for (int i = 0; i < 16; i++) {
        tag[i] = (uint8_t)(acc[i] ^ ek[i]);
    }
    d011_wipe(h, sizeof h);
    d011_wipe(ek, sizeof ek);
}

static void gcm_j0(uint8_t j0[16], const uint8_t iv[12])
{
    for (int i = 0; i < 12; i++) {
        j0[i] = iv[i];
    }
    j0[12] = 0;
    j0[13] = 0;
    j0[14] = 0;
    j0[15] = 1;
}

int d011_accel_aes_gcm_seal(uint8_t *out, const uint8_t *key, size_t key_len,
                            const uint8_t iv[12], const uint8_t *aad, size_t aad_len,
                            const uint8_t *pt, size_t pt_len)
{
    uint8_t rk[240];
    uint8_t j0[16];
    int nr = d011_aes_expand_key(rk, key, key_len);
    if (nr < 0) {
        return -1;
    }
    gcm_j0(j0, iv);
    gcm_ctr(out, pt, pt_len, rk, nr, j0);
    gcm_tag(out + pt_len, rk, nr, j0, aad, aad_len, out, pt_len);
    d011_wipe(rk, sizeof rk);
    return 0;
}

int d011_accel_aes_gcm_open(uint8_t *out, const uint8_t *key, size_t key_len,
                            const uint8_t iv[12], const uint8_t *aad, size_t aad_len,
                            const uint8_t *ct, size_t ct_len)
{
    uint8_t rk[240];
    uint8_t j0[16];
    uint8_t tag[16];
    uint32_t bad;
    int nr;
    if (ct_len < 16) {
        return -1;
    }
    nr = d011_aes_expand_key(rk, key, key_len);
    if (nr < 0) {
        return -1;
    }
    gcm_j0(j0, iv);
    gcm_tag(tag, rk, nr, j0, aad, aad_len, ct, ct_len - 16);
    bad = d011_ct_differs(tag, ct + ct_len - 16, 16);
    if (bad == 0) {
        gcm_ctr(out, ct, ct_len - 16, rk, nr, j0);
    }
    d011_wipe(rk, sizeof rk);
    d011_wipe(tag, sizeof tag);
    return bad == 0 ? 0 : -1;
}

#endif /* D011_PART_GCM_GLUE */

/* ======================================================================
 * The target-specific instructions.
 */
#if defined(D011_PART_ACCEL)

#if defined(__x86_64__)
/* x86-64: AES-NI and PCLMULQDQ intrinsics. */
#include <emmintrin.h>
#include <wmmintrin.h>

void d011_accel_aes_rounds(uint8_t out[16], const uint8_t *rk, int rounds, const uint8_t in[16])
{
    __m128i s = _mm_loadu_si128((const __m128i *)(const void *)in);
    s = _mm_xor_si128(s, _mm_loadu_si128((const __m128i *)(const void *)rk));
    for (int r = 1; r < rounds; r++) {
        s = _mm_aesenc_si128(s, _mm_loadu_si128((const __m128i *)(const void *)(rk + 16 * r)));
    }
    s = _mm_aesenclast_si128(s, _mm_loadu_si128((const __m128i *)(const void *)(rk + 16 * rounds)));
    _mm_storeu_si128((__m128i *)(void *)out, s);
}

void d011_accel_clmul64(uint64_t a, uint64_t b, uint64_t *hi, uint64_t *lo)
{
    __m128i p = _mm_clmulepi64_si128(_mm_cvtsi64_si128((long long)a),
                                     _mm_cvtsi64_si128((long long)b), 0x00);
    *lo = (uint64_t)_mm_cvtsi128_si64(p);
    *hi = (uint64_t)_mm_cvtsi128_si64(_mm_unpackhi_epi64(p, p));
}
#elif defined(__aarch64__)
/* AArch64: Armv8 AES and PMULL intrinsics. */
#include <arm_neon.h>

void d011_accel_aes_rounds(uint8_t out[16], const uint8_t *rk, int rounds, const uint8_t in[16])
{
    uint8x16_t s = vld1q_u8(in);
    for (int r = 0; r < rounds - 1; r++) {
        s = vaesmcq_u8(vaeseq_u8(s, vld1q_u8(rk + 16 * r)));
    }
    s = vaeseq_u8(s, vld1q_u8(rk + 16 * (rounds - 1)));
    s = veorq_u8(s, vld1q_u8(rk + 16 * rounds));
    vst1q_u8(out, s);
}

void d011_accel_clmul64(uint64_t a, uint64_t b, uint64_t *hi, uint64_t *lo)
{
    uint64x2_t p = vreinterpretq_u64_p128(vmull_p64((poly64_t)a, (poly64_t)b));
    *lo = vgetq_lane_u64(p, 0);
    *hi = vgetq_lane_u64(p, 1);
}
#elif defined(__riscv) && __riscv_xlen == 64
/* RV64: scalar Zkne and Zbkc, as inline assembly so that no compiler
 * intrinsic header is needed. */
static uint64_t aes64es(uint64_t a, uint64_t b)
{
    uint64_t r;
    __asm__("aes64es %0, %1, %2" : "=r"(r) : "r"(a), "r"(b));
    return r;
}

static uint64_t aes64esm(uint64_t a, uint64_t b)
{
    uint64_t r;
    __asm__("aes64esm %0, %1, %2" : "=r"(r) : "r"(a), "r"(b));
    return r;
}

static uint64_t load_le64(const uint8_t *p)
{
    uint64_t v = 0;
    for (int i = 7; i >= 0; i--) {
        v = (v << 8) | p[i];
    }
    return v;
}

static void store_le64(uint8_t *p, uint64_t v)
{
    for (int i = 0; i < 8; i++) {
        p[i] = (uint8_t)v;
        v >>= 8;
    }
}

void d011_accel_aes_rounds(uint8_t out[16], const uint8_t *rk, int rounds, const uint8_t in[16])
{
    uint64_t s0 = load_le64(in) ^ load_le64(rk);
    uint64_t s1 = load_le64(in + 8) ^ load_le64(rk + 8);
    for (int r = 1; r < rounds; r++) {
        uint64_t n0 = aes64esm(s0, s1);
        uint64_t n1 = aes64esm(s1, s0);
        s0 = n0 ^ load_le64(rk + 16 * r);
        s1 = n1 ^ load_le64(rk + 16 * r + 8);
    }
    {
        uint64_t n0 = aes64es(s0, s1);
        uint64_t n1 = aes64es(s1, s0);
        s0 = n0 ^ load_le64(rk + 16 * rounds);
        s1 = n1 ^ load_le64(rk + 16 * rounds + 8);
    }
    store_le64(out, s0);
    store_le64(out + 8, s1);
}

void d011_accel_clmul64(uint64_t a, uint64_t b, uint64_t *hi, uint64_t *lo)
{
    uint64_t h;
    uint64_t l;
    __asm__("clmul %0, %1, %2" : "=r"(l) : "r"(a), "r"(b));
    __asm__("clmulh %0, %1, %2" : "=r"(h) : "r"(a), "r"(b));
    *hi = h;
    *lo = l;
}
#else
#error "D-011: no crypto profile for this target"
#endif /* target */

#endif /* D011_PART_ACCEL */
