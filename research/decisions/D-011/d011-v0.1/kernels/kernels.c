/*
 * D-011 v0.1 stand-in kernels: the portable C reference kernels for the
 * proposed D-015 flagship families, one section per family.
 *
 * Contributor-written portable C, NOT Orange output: Orange has no native
 * backend. The laboratory compiles this file once per family, defining
 * exactly one D011_PART_* macro, so each family is its own object file
 * and its own row of the instruction inventory and size estimate. The
 * code is written for constant-time review: secret values select no
 * branch, no memory address and no division; lengths, key sizes and
 * operation codes are public. That intent is what the laboratory checks
 * on every target; it is not a claim that any build is constant-time.
 */
#include "d011_kernels.h"

/* ======================================================================
 * SHA-256 and SHA-512 (FIPS 180-4).
 * Contributor-written portable C, not Orange output. The constants were
 * derived from the prime roots of FIPS 180-4 sections 4.2 and 5.3, and
 * the SHA-256 round constants equal round_constants in
 * algorithms/sha2/sha2.or.
 * Message lengths are public; every branch below depends only on them.
 */
#if defined(D011_PART_SHA2)

static const uint32_t K256[64] = {
    0x428a2f98U, 0x71374491U, 0xb5c0fbcfU, 0xe9b5dba5U,
    0x3956c25bU, 0x59f111f1U, 0x923f82a4U, 0xab1c5ed5U,
    0xd807aa98U, 0x12835b01U, 0x243185beU, 0x550c7dc3U,
    0x72be5d74U, 0x80deb1feU, 0x9bdc06a7U, 0xc19bf174U,
    0xe49b69c1U, 0xefbe4786U, 0x0fc19dc6U, 0x240ca1ccU,
    0x2de92c6fU, 0x4a7484aaU, 0x5cb0a9dcU, 0x76f988daU,
    0x983e5152U, 0xa831c66dU, 0xb00327c8U, 0xbf597fc7U,
    0xc6e00bf3U, 0xd5a79147U, 0x06ca6351U, 0x14292967U,
    0x27b70a85U, 0x2e1b2138U, 0x4d2c6dfcU, 0x53380d13U,
    0x650a7354U, 0x766a0abbU, 0x81c2c92eU, 0x92722c85U,
    0xa2bfe8a1U, 0xa81a664bU, 0xc24b8b70U, 0xc76c51a3U,
    0xd192e819U, 0xd6990624U, 0xf40e3585U, 0x106aa070U,
    0x19a4c116U, 0x1e376c08U, 0x2748774cU, 0x34b0bcb5U,
    0x391c0cb3U, 0x4ed8aa4aU, 0x5b9cca4fU, 0x682e6ff3U,
    0x748f82eeU, 0x78a5636fU, 0x84c87814U, 0x8cc70208U,
    0x90befffaU, 0xa4506cebU, 0xbef9a3f7U, 0xc67178f2U,
};

static const uint32_t H256[8] = {
    0x6a09e667U, 0xbb67ae85U, 0x3c6ef372U, 0xa54ff53aU,
    0x510e527fU, 0x9b05688cU, 0x1f83d9abU, 0x5be0cd19U,
};

static const uint64_t K512[80] = {
    0x428a2f98d728ae22ULL, 0x7137449123ef65cdULL,
    0xb5c0fbcfec4d3b2fULL, 0xe9b5dba58189dbbcULL,
    0x3956c25bf348b538ULL, 0x59f111f1b605d019ULL,
    0x923f82a4af194f9bULL, 0xab1c5ed5da6d8118ULL,
    0xd807aa98a3030242ULL, 0x12835b0145706fbeULL,
    0x243185be4ee4b28cULL, 0x550c7dc3d5ffb4e2ULL,
    0x72be5d74f27b896fULL, 0x80deb1fe3b1696b1ULL,
    0x9bdc06a725c71235ULL, 0xc19bf174cf692694ULL,
    0xe49b69c19ef14ad2ULL, 0xefbe4786384f25e3ULL,
    0x0fc19dc68b8cd5b5ULL, 0x240ca1cc77ac9c65ULL,
    0x2de92c6f592b0275ULL, 0x4a7484aa6ea6e483ULL,
    0x5cb0a9dcbd41fbd4ULL, 0x76f988da831153b5ULL,
    0x983e5152ee66dfabULL, 0xa831c66d2db43210ULL,
    0xb00327c898fb213fULL, 0xbf597fc7beef0ee4ULL,
    0xc6e00bf33da88fc2ULL, 0xd5a79147930aa725ULL,
    0x06ca6351e003826fULL, 0x142929670a0e6e70ULL,
    0x27b70a8546d22ffcULL, 0x2e1b21385c26c926ULL,
    0x4d2c6dfc5ac42aedULL, 0x53380d139d95b3dfULL,
    0x650a73548baf63deULL, 0x766a0abb3c77b2a8ULL,
    0x81c2c92e47edaee6ULL, 0x92722c851482353bULL,
    0xa2bfe8a14cf10364ULL, 0xa81a664bbc423001ULL,
    0xc24b8b70d0f89791ULL, 0xc76c51a30654be30ULL,
    0xd192e819d6ef5218ULL, 0xd69906245565a910ULL,
    0xf40e35855771202aULL, 0x106aa07032bbd1b8ULL,
    0x19a4c116b8d2d0c8ULL, 0x1e376c085141ab53ULL,
    0x2748774cdf8eeb99ULL, 0x34b0bcb5e19b48a8ULL,
    0x391c0cb3c5c95a63ULL, 0x4ed8aa4ae3418acbULL,
    0x5b9cca4f7763e373ULL, 0x682e6ff3d6b2b8a3ULL,
    0x748f82ee5defb2fcULL, 0x78a5636f43172f60ULL,
    0x84c87814a1f0ab72ULL, 0x8cc702081a6439ecULL,
    0x90befffa23631e28ULL, 0xa4506cebde82bde9ULL,
    0xbef9a3f7b2c67915ULL, 0xc67178f2e372532bULL,
    0xca273eceea26619cULL, 0xd186b8c721c0c207ULL,
    0xeada7dd6cde0eb1eULL, 0xf57d4f7fee6ed178ULL,
    0x06f067aa72176fbaULL, 0x0a637dc5a2c898a6ULL,
    0x113f9804bef90daeULL, 0x1b710b35131c471bULL,
    0x28db77f523047d84ULL, 0x32caab7b40c72493ULL,
    0x3c9ebe0a15c9bebcULL, 0x431d67c49c100d4cULL,
    0x4cc5d4becb3e42b6ULL, 0x597f299cfc657e2aULL,
    0x5fcb6fab3ad6faecULL, 0x6c44198c4a475817ULL,
};

static const uint64_t H512[8] = {
    0x6a09e667f3bcc908ULL, 0xbb67ae8584caa73bULL,
    0x3c6ef372fe94f82bULL, 0xa54ff53a5f1d36f1ULL,
    0x510e527fade682d1ULL, 0x9b05688c2b3e6c1fULL,
    0x1f83d9abfb41bd6bULL, 0x5be0cd19137e2179ULL,
};

static uint32_t ror32(uint32_t x, unsigned n)
{
    return (x >> n) | (x << (32U - n));
}

static uint64_t ror64(uint64_t x, unsigned n)
{
    return (x >> n) | (x << (64U - n));
}

static uint32_t load_be32(const uint8_t *p)
{
    return ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) |
           ((uint32_t)p[2] << 8) | (uint32_t)p[3];
}

static void store_be32(uint8_t *p, uint32_t v)
{
    p[0] = (uint8_t)(v >> 24);
    p[1] = (uint8_t)(v >> 16);
    p[2] = (uint8_t)(v >> 8);
    p[3] = (uint8_t)v;
}

static uint64_t load_be64(const uint8_t *p)
{
    return ((uint64_t)load_be32(p) << 32) | (uint64_t)load_be32(p + 4);
}

static void store_be64(uint8_t *p, uint64_t v)
{
    store_be32(p, (uint32_t)(v >> 32));
    store_be32(p + 4, (uint32_t)v);
}

/* FIPS 180-4 section 6.2.2, step 1. */
static void sha256_schedule(uint32_t w[64], const uint8_t block[64])
{
    for (int t = 0; t < 16; t++) {
        w[t] = load_be32(block + 4 * t);
    }
    for (int t = 16; t < 64; t++) {
        uint32_t s0 = ror32(w[t - 15], 7) ^ ror32(w[t - 15], 18) ^ (w[t - 15] >> 3);
        uint32_t s1 = ror32(w[t - 2], 17) ^ ror32(w[t - 2], 19) ^ (w[t - 2] >> 10);
        w[t] = s1 + w[t - 7] + s0 + w[t - 16];
    }
}

/* FIPS 180-4 section 6.2.2, step 3: one round on a through h. */
static void sha256_round(uint32_t v[8], uint32_t k, uint32_t w)
{
    uint32_t ch = (v[4] & v[5]) ^ (~v[4] & v[6]);
    uint32_t maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
    uint32_t t1 = v[7] + (ror32(v[4], 6) ^ ror32(v[4], 11) ^ ror32(v[4], 25)) + ch + k + w;
    uint32_t t2 = (ror32(v[0], 2) ^ ror32(v[0], 13) ^ ror32(v[0], 22)) + maj;
    v[7] = v[6];
    v[6] = v[5];
    v[5] = v[4];
    v[4] = v[3] + t1;
    v[3] = v[2];
    v[2] = v[1];
    v[1] = v[0];
    v[0] = t1 + t2;
}

static void sha256_compress(uint32_t h[8], const uint8_t block[64])
{
    uint32_t w[64];
    uint32_t v[8];
    sha256_schedule(w, block);
    for (int i = 0; i < 8; i++) {
        v[i] = h[i];
    }
    for (int t = 0; t < 64; t++) {
        sha256_round(v, K256[t], w[t]);
    }
    for (int i = 0; i < 8; i++) {
        h[i] += v[i];
    }
}

void d011_sha256_init(struct d011_sha256_ctx *ctx)
{
    for (int i = 0; i < 8; i++) {
        ctx->h[i] = H256[i];
    }
    ctx->total = 0;
    ctx->used = 0;
}

void d011_sha256_update(struct d011_sha256_ctx *ctx, const uint8_t *msg, size_t len)
{
    ctx->total += (uint64_t)len;
    while (len > 0) {
        size_t take = 64 - ctx->used;
        if (take > len) {
            take = len;
        }
        for (size_t i = 0; i < take; i++) {
            ctx->buf[ctx->used + i] = msg[i];
        }
        ctx->used += take;
        msg += take;
        len -= take;
        if (ctx->used == 64) {
            sha256_compress(ctx->h, ctx->buf);
            ctx->used = 0;
        }
    }
}

void d011_sha256_final(struct d011_sha256_ctx *ctx, uint8_t out[32])
{
    uint64_t bits = ctx->total << 3;
    size_t used = ctx->used;
    ctx->buf[used++] = 0x80;
    if (used > 56) {
        while (used < 64) {
            ctx->buf[used++] = 0;
        }
        sha256_compress(ctx->h, ctx->buf);
        used = 0;
    }
    while (used < 56) {
        ctx->buf[used++] = 0;
    }
    store_be64(ctx->buf + 56, bits);
    sha256_compress(ctx->h, ctx->buf);
    for (int i = 0; i < 8; i++) {
        store_be32(out + 4 * i, ctx->h[i]);
    }
    d011_wipe(ctx, sizeof *ctx);
}

void d011_sha256(uint8_t out[32], const uint8_t *msg, size_t len)
{
    struct d011_sha256_ctx ctx;
    d011_sha256_init(&ctx);
    d011_sha256_update(&ctx, msg, len);
    d011_sha256_final(&ctx, out);
}

void d011_sha256_probe(uint8_t out[72], const uint8_t block[64])
{
    uint32_t w[64];
    uint32_t v[8];
    sha256_schedule(w, block);
    store_be32(out, w[16]);
    store_be32(out + 4, w[17]);
    for (int i = 0; i < 8; i++) {
        v[i] = H256[i];
    }
    for (int t = 0; t < 2; t++) {
        sha256_round(v, K256[t], w[t]);
        for (int i = 0; i < 8; i++) {
            store_be32(out + 8 + 32 * t + 4 * i, v[i]);
        }
    }
}

static void sha512_compress(uint64_t h[8], const uint8_t block[128])
{
    uint64_t w[80];
    uint64_t a, b, c, d, e, f, g, k;
    for (int t = 0; t < 16; t++) {
        w[t] = load_be64(block + 8 * t);
    }
    for (int t = 16; t < 80; t++) {
        uint64_t s0 = ror64(w[t - 15], 1) ^ ror64(w[t - 15], 8) ^ (w[t - 15] >> 7);
        uint64_t s1 = ror64(w[t - 2], 19) ^ ror64(w[t - 2], 61) ^ (w[t - 2] >> 6);
        w[t] = s1 + w[t - 7] + s0 + w[t - 16];
    }
    a = h[0];
    b = h[1];
    c = h[2];
    d = h[3];
    e = h[4];
    f = h[5];
    g = h[6];
    k = h[7];
    for (int t = 0; t < 80; t++) {
        uint64_t t1 = k + (ror64(e, 14) ^ ror64(e, 18) ^ ror64(e, 41)) +
                      ((e & f) ^ (~e & g)) + K512[t] + w[t];
        uint64_t t2 = (ror64(a, 28) ^ ror64(a, 34) ^ ror64(a, 39)) +
                      ((a & b) ^ (a & c) ^ (b & c));
        k = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
    }
    h[0] += a;
    h[1] += b;
    h[2] += c;
    h[3] += d;
    h[4] += e;
    h[5] += f;
    h[6] += g;
    h[7] += k;
}

void d011_sha512(uint8_t out[64], const uint8_t *msg, size_t len)
{
    uint64_t h[8];
    uint8_t buf[128];
    size_t used;
    uint64_t bits = (uint64_t)len << 3;
    for (int i = 0; i < 8; i++) {
        h[i] = H512[i];
    }
    while (len >= 128) {
        sha512_compress(h, msg);
        msg += 128;
        len -= 128;
    }
    for (used = 0; used < len; used++) {
        buf[used] = msg[used];
    }
    buf[used++] = 0x80;
    if (used > 112) {
        while (used < 128) {
            buf[used++] = 0;
        }
        sha512_compress(h, buf);
        used = 0;
    }
    while (used < 120) {
        buf[used++] = 0;
    }
    store_be64(buf + 120, bits);
    sha512_compress(h, buf);
    for (int i = 0; i < 8; i++) {
        store_be64(out + 8 * i, h[i]);
    }
    d011_wipe(buf, sizeof buf);
    d011_wipe(h, sizeof h);
}

#endif /* D011_PART_SHA2 */

/* ======================================================================
 * HMAC-SHA-256 (RFC 2104) and HKDF-SHA-256
 * (RFC 5869). Contributor-written portable C, not Orange output. Key,
 * salt, info and output lengths are public.
 */
#if defined(D011_PART_HMAC_HKDF)

struct hmac_ctx {
    struct d011_sha256_ctx inner;
    struct d011_sha256_ctx outer;
};

static void hmac_init(struct hmac_ctx *ctx, const uint8_t *key, size_t key_len)
{
    uint8_t block[64];
    uint8_t pad[64];
    for (int i = 0; i < 64; i++) {
        block[i] = 0;
    }
    if (key_len > 64) {
        d011_sha256(block, key, key_len);
    } else {
        for (size_t i = 0; i < key_len; i++) {
            block[i] = key[i];
        }
    }
    for (int i = 0; i < 64; i++) {
        pad[i] = (uint8_t)(block[i] ^ 0x36);
    }
    d011_sha256_init(&ctx->inner);
    d011_sha256_update(&ctx->inner, pad, 64);
    for (int i = 0; i < 64; i++) {
        pad[i] = (uint8_t)(block[i] ^ 0x5c);
    }
    d011_sha256_init(&ctx->outer);
    d011_sha256_update(&ctx->outer, pad, 64);
    d011_wipe(block, sizeof block);
    d011_wipe(pad, sizeof pad);
}

static void hmac_final(struct hmac_ctx *ctx, uint8_t out[32])
{
    uint8_t inner[32];
    d011_sha256_final(&ctx->inner, inner);
    d011_sha256_update(&ctx->outer, inner, 32);
    d011_sha256_final(&ctx->outer, out);
    d011_wipe(inner, sizeof inner);
}

void d011_hmac_sha256(uint8_t out[32], const uint8_t *key, size_t key_len,
                      const uint8_t *msg, size_t msg_len)
{
    struct hmac_ctx ctx;
    hmac_init(&ctx, key, key_len);
    d011_sha256_update(&ctx.inner, msg, msg_len);
    hmac_final(&ctx, out);
}

/* RFC 5869 sections 2.2 and 2.3. Returns -1 when out_len > 255 * 32. */
int d011_hkdf_sha256(uint8_t *out, size_t out_len, const uint8_t *ikm,
                     size_t ikm_len, const uint8_t *salt, size_t salt_len,
                     const uint8_t *info, size_t info_len)
{
    static const uint8_t zero_salt[32] = {0};
    uint8_t prk[32];
    uint8_t t[32] = {0};
    size_t t_len = 0;
    size_t done = 0;
    uint8_t counter = 1;
    if (out_len > 255U * 32U) {
        return -1;
    }
    if (salt_len == 0) {
        salt = zero_salt;
        salt_len = 32;
    }
    d011_hmac_sha256(prk, salt, salt_len, ikm, ikm_len);
    while (done < out_len) {
        struct hmac_ctx ctx;
        size_t take = out_len - done;
        hmac_init(&ctx, prk, 32);
        d011_sha256_update(&ctx.inner, t, t_len);
        d011_sha256_update(&ctx.inner, info, info_len);
        d011_sha256_update(&ctx.inner, &counter, 1);
        hmac_final(&ctx, t);
        t_len = 32;
        if (take > 32) {
            take = 32;
        }
        for (size_t i = 0; i < take; i++) {
            out[done + i] = t[i];
        }
        done += take;
        counter++;
    }
    d011_wipe(prk, sizeof prk);
    d011_wipe(t, sizeof t);
    return 0;
}

#endif /* D011_PART_HMAC_HKDF */

/* ======================================================================
 * ChaCha20, Poly1305 and the ChaCha20-Poly1305
 * AEAD (RFC 8439). Contributor-written portable C, not Orange output.
 * Poly1305 uses 26-bit limbs and 32x32->64-bit products. Lengths are
 * public; the tag comparison has no data-dependent branch, and only its
 * public accept/reject result selects the return path.
 */
#if defined(D011_PART_CHACHA_POLY)

static uint32_t rotl32(uint32_t x, unsigned n)
{
    return (x << n) | (x >> (32U - n));
}

static uint32_t load_le32(const uint8_t *p)
{
    return (uint32_t)p[0] | ((uint32_t)p[1] << 8) | ((uint32_t)p[2] << 16) |
           ((uint32_t)p[3] << 24);
}

static void store_le32(uint8_t *p, uint32_t v)
{
    p[0] = (uint8_t)v;
    p[1] = (uint8_t)(v >> 8);
    p[2] = (uint8_t)(v >> 16);
    p[3] = (uint8_t)(v >> 24);
}

#define QR(a, b, c, d)            \
    do {                          \
        a += b;                   \
        d = rotl32(d ^ a, 16);    \
        c += d;                   \
        b = rotl32(b ^ c, 12);    \
        a += b;                   \
        d = rotl32(d ^ a, 8);     \
        c += d;                   \
        b = rotl32(b ^ c, 7);     \
    } while (0)

/* RFC 8439 section 2.3. */
void d011_chacha20_block(uint8_t out[64], const uint8_t key[32],
                         uint32_t counter, const uint8_t nonce[12])
{
    uint32_t s[16];
    uint32_t x[16];
    s[0] = 0x61707865U;
    s[1] = 0x3320646eU;
    s[2] = 0x79622d32U;
    s[3] = 0x6b206574U;
    for (int i = 0; i < 8; i++) {
        s[4 + i] = load_le32(key + 4 * i);
    }
    s[12] = counter;
    for (int i = 0; i < 3; i++) {
        s[13 + i] = load_le32(nonce + 4 * i);
    }
    for (int i = 0; i < 16; i++) {
        x[i] = s[i];
    }
    for (int r = 0; r < 10; r++) {
        QR(x[0], x[4], x[8], x[12]);
        QR(x[1], x[5], x[9], x[13]);
        QR(x[2], x[6], x[10], x[14]);
        QR(x[3], x[7], x[11], x[15]);
        QR(x[0], x[5], x[10], x[15]);
        QR(x[1], x[6], x[11], x[12]);
        QR(x[2], x[7], x[8], x[13]);
        QR(x[3], x[4], x[9], x[14]);
    }
    for (int i = 0; i < 16; i++) {
        store_le32(out + 4 * i, x[i] + s[i]);
    }
    d011_wipe(x, sizeof x);
    d011_wipe(s, sizeof s);
}

/* RFC 8439 section 2.4. */
void d011_chacha20_xor(uint8_t *out, const uint8_t *in, size_t len,
                       const uint8_t key[32], uint32_t counter,
                       const uint8_t nonce[12])
{
    uint8_t ks[64];
    while (len > 0) {
        size_t take = len < 64 ? len : 64;
        d011_chacha20_block(ks, key, counter, nonce);
        for (size_t i = 0; i < take; i++) {
            out[i] = (uint8_t)(in[i] ^ ks[i]);
        }
        out += take;
        in += take;
        len -= take;
        counter++;
    }
    d011_wipe(ks, sizeof ks);
}

struct poly1305 {
    uint32_t r[5];
    uint32_t h[5];
    uint32_t pad[4];
};

static void poly1305_init(struct poly1305 *st, const uint8_t key[32])
{
    st->r[0] = load_le32(key) & 0x3ffffffU;
    st->r[1] = (load_le32(key + 3) >> 2) & 0x3ffff03U;
    st->r[2] = (load_le32(key + 6) >> 4) & 0x3ffc0ffU;
    st->r[3] = (load_le32(key + 9) >> 6) & 0x3f03fffU;
    st->r[4] = (load_le32(key + 12) >> 8) & 0x00fffffU;
    for (int i = 0; i < 5; i++) {
        st->h[i] = 0;
    }
    for (int i = 0; i < 4; i++) {
        st->pad[i] = load_le32(key + 16 + 4 * i);
    }
}

/* One 16-byte block; hibit is 1 << 24 for a whole block and 0 for the
 * final partial block, which the caller has already padded with 0x01. */
static void poly1305_block(struct poly1305 *st, const uint8_t m[16], uint32_t hibit)
{
    const uint32_t r0 = st->r[0], r1 = st->r[1], r2 = st->r[2], r3 = st->r[3], r4 = st->r[4];
    const uint32_t s1 = r1 * 5, s2 = r2 * 5, s3 = r3 * 5, s4 = r4 * 5;
    uint32_t h0 = st->h[0], h1 = st->h[1], h2 = st->h[2], h3 = st->h[3], h4 = st->h[4];
    uint64_t d0, d1, d2, d3, d4;
    uint32_t c;
    h0 += load_le32(m) & 0x3ffffffU;
    h1 += (load_le32(m + 3) >> 2) & 0x3ffffffU;
    h2 += (load_le32(m + 6) >> 4) & 0x3ffffffU;
    h3 += (load_le32(m + 9) >> 6) & 0x3ffffffU;
    h4 += (load_le32(m + 12) >> 8) | hibit;
    d0 = (uint64_t)h0 * r0 + (uint64_t)h1 * s4 + (uint64_t)h2 * s3 + (uint64_t)h3 * s2 + (uint64_t)h4 * s1;
    d1 = (uint64_t)h0 * r1 + (uint64_t)h1 * r0 + (uint64_t)h2 * s4 + (uint64_t)h3 * s3 + (uint64_t)h4 * s2;
    d2 = (uint64_t)h0 * r2 + (uint64_t)h1 * r1 + (uint64_t)h2 * r0 + (uint64_t)h3 * s4 + (uint64_t)h4 * s3;
    d3 = (uint64_t)h0 * r3 + (uint64_t)h1 * r2 + (uint64_t)h2 * r1 + (uint64_t)h3 * r0 + (uint64_t)h4 * s4;
    d4 = (uint64_t)h0 * r4 + (uint64_t)h1 * r3 + (uint64_t)h2 * r2 + (uint64_t)h3 * r1 + (uint64_t)h4 * r0;
    c = (uint32_t)(d0 >> 26);
    h0 = (uint32_t)d0 & 0x3ffffffU;
    d1 += c;
    c = (uint32_t)(d1 >> 26);
    h1 = (uint32_t)d1 & 0x3ffffffU;
    d2 += c;
    c = (uint32_t)(d2 >> 26);
    h2 = (uint32_t)d2 & 0x3ffffffU;
    d3 += c;
    c = (uint32_t)(d3 >> 26);
    h3 = (uint32_t)d3 & 0x3ffffffU;
    d4 += c;
    c = (uint32_t)(d4 >> 26);
    h4 = (uint32_t)d4 & 0x3ffffffU;
    h0 += c * 5;
    c = h0 >> 26;
    h0 &= 0x3ffffffU;
    h1 += c;
    st->h[0] = h0;
    st->h[1] = h1;
    st->h[2] = h2;
    st->h[3] = h3;
    st->h[4] = h4;
}

static void poly1305_finish(struct poly1305 *st, uint8_t tag[16])
{
    uint32_t h0 = st->h[0], h1 = st->h[1], h2 = st->h[2], h3 = st->h[3], h4 = st->h[4];
    uint32_t g0, g1, g2, g3, g4, c, mask;
    uint64_t f;
    c = h1 >> 26;
    h1 &= 0x3ffffffU;
    h2 += c;
    c = h2 >> 26;
    h2 &= 0x3ffffffU;
    h3 += c;
    c = h3 >> 26;
    h3 &= 0x3ffffffU;
    h4 += c;
    c = h4 >> 26;
    h4 &= 0x3ffffffU;
    h0 += c * 5;
    c = h0 >> 26;
    h0 &= 0x3ffffffU;
    h1 += c;
    /* g = h + 5 - 2^130; keep h when g is negative. */
    g0 = h0 + 5;
    c = g0 >> 26;
    g0 &= 0x3ffffffU;
    g1 = h1 + c;
    c = g1 >> 26;
    g1 &= 0x3ffffffU;
    g2 = h2 + c;
    c = g2 >> 26;
    g2 &= 0x3ffffffU;
    g3 = h3 + c;
    c = g3 >> 26;
    g3 &= 0x3ffffffU;
    g4 = h4 + c - (1U << 26);
    mask = (g4 >> 31) - 1U;
    g0 &= mask;
    g1 &= mask;
    g2 &= mask;
    g3 &= mask;
    g4 &= mask;
    mask = ~mask;
    h0 = (h0 & mask) | g0;
    h1 = (h1 & mask) | g1;
    h2 = (h2 & mask) | g2;
    h3 = (h3 & mask) | g3;
    h4 = (h4 & mask) | g4;
    h0 = h0 | (h1 << 26);
    h1 = (h1 >> 6) | (h2 << 20);
    h2 = (h2 >> 12) | (h3 << 14);
    h3 = (h3 >> 18) | (h4 << 8);
    f = (uint64_t)h0 + st->pad[0];
    h0 = (uint32_t)f;
    f = (uint64_t)h1 + st->pad[1] + (f >> 32);
    h1 = (uint32_t)f;
    f = (uint64_t)h2 + st->pad[2] + (f >> 32);
    h2 = (uint32_t)f;
    f = (uint64_t)h3 + st->pad[3] + (f >> 32);
    h3 = (uint32_t)f;
    store_le32(tag, h0);
    store_le32(tag + 4, h1);
    store_le32(tag + 8, h2);
    store_le32(tag + 12, h3);
    d011_wipe(st, sizeof *st);
}

/* RFC 8439 section 2.5. */
void d011_poly1305(uint8_t tag[16], const uint8_t *msg, size_t len,
                   const uint8_t key[32])
{
    struct poly1305 st;
    uint8_t last[16];
    poly1305_init(&st, key);
    while (len >= 16) {
        poly1305_block(&st, msg, 1U << 24);
        msg += 16;
        len -= 16;
    }
    if (len > 0) {
        for (size_t i = 0; i < 16; i++) {
            last[i] = 0;
        }
        for (size_t i = 0; i < len; i++) {
            last[i] = msg[i];
        }
        last[len] = 1;
        poly1305_block(&st, last, 0);
    }
    poly1305_finish(&st, tag);
}

/* Absorb data zero-padded to a multiple of 16 bytes (RFC 8439 2.8). */
static void poly1305_padded(struct poly1305 *st, const uint8_t *data, size_t len)
{
    uint8_t last[16];
    while (len >= 16) {
        poly1305_block(st, data, 1U << 24);
        data += 16;
        len -= 16;
    }
    if (len > 0) {
        for (size_t i = 0; i < 16; i++) {
            last[i] = 0;
        }
        for (size_t i = 0; i < len; i++) {
            last[i] = data[i];
        }
        poly1305_block(st, last, 1U << 24);
    }
}

static void aead_tag(uint8_t tag[16], const uint8_t key[32], const uint8_t nonce[12],
                     const uint8_t *aad, size_t aad_len, const uint8_t *ct, size_t ct_len)
{
    uint8_t block0[64];
    uint8_t lengths[16];
    struct poly1305 st;
    d011_chacha20_block(block0, key, 0, nonce);
    poly1305_init(&st, block0);
    poly1305_padded(&st, aad, aad_len);
    poly1305_padded(&st, ct, ct_len);
    for (int i = 0; i < 8; i++) {
        lengths[i] = (uint8_t)((uint64_t)aad_len >> (8 * i));
        lengths[8 + i] = (uint8_t)((uint64_t)ct_len >> (8 * i));
    }
    poly1305_block(&st, lengths, 1U << 24);
    poly1305_finish(&st, tag);
    d011_wipe(block0, sizeof block0);
}

/* RFC 8439 section 2.8: out receives pt_len bytes of ciphertext and the
 * 16-byte tag. */
void d011_aead_seal(uint8_t *out, const uint8_t key[32], const uint8_t nonce[12],
                    const uint8_t *aad, size_t aad_len, const uint8_t *pt, size_t pt_len)
{
    d011_chacha20_xor(out, pt, pt_len, key, 1, nonce);
    aead_tag(out + pt_len, key, nonce, aad, aad_len, out, pt_len);
}

/* Returns 0 and writes ct_len - 16 plaintext bytes when the tag verifies;
 * otherwise returns -1 and writes nothing. */
int d011_aead_open(uint8_t *out, const uint8_t key[32], const uint8_t nonce[12],
                   const uint8_t *aad, size_t aad_len, const uint8_t *ct, size_t ct_len)
{
    uint8_t tag[16];
    uint32_t bad;
    if (ct_len < 16) {
        return -1;
    }
    aead_tag(tag, key, nonce, aad, aad_len, ct, ct_len - 16);
    bad = d011_ct_differs(tag, ct + ct_len - 16, 16);
    d011_wipe(tag, sizeof tag);
    if (bad != 0) {
        return -1;
    }
    d011_chacha20_xor(out, ct, ct_len - 16, key, 1, nonce);
    return 0;
}

#endif /* D011_PART_CHACHA_POLY */

/* ======================================================================
 * X25519 (RFC 7748 section 5).
 * Contributor-written portable C, not Orange output. Field elements of
 * GF(2^255 - 19) are sixteen signed 64-bit limbs of radix 2^16, the
 * representation of the public-domain TweetNaCl; the ladder follows the
 * RFC 7748 pseudocode with a mask-based conditional swap. Right shifts of
 * negative limbs rely on the arithmetic shift that every compiler in the
 * laboratory implements.
 */
#if defined(D011_PART_X25519)

typedef int64_t fe[16];

static void fe_copy(fe o, const fe a)
{
    for (int i = 0; i < 16; i++) {
        o[i] = a[i];
    }
}

static void fe_carry(fe o)
{
    for (int i = 0; i < 16; i++) {
        int64_t c;
        o[i] += (int64_t)1 << 16;
        c = o[i] >> 16;
        if (i < 15) {
            o[i + 1] += c - 1;
        } else {
            o[0] += 38 * (c - 1);
        }
        o[i] -= c * 65536;
    }
}

/* Swap p and q when bit is 1, without a data-dependent branch. */
static void fe_cswap(fe p, fe q, int64_t bit)
{
    int64_t mask = -bit;
    for (int i = 0; i < 16; i++) {
        int64_t t = mask & (p[i] ^ q[i]);
        p[i] ^= t;
        q[i] ^= t;
    }
}

static void fe_add(fe o, const fe a, const fe b)
{
    for (int i = 0; i < 16; i++) {
        o[i] = a[i] + b[i];
    }
}

static void fe_sub(fe o, const fe a, const fe b)
{
    for (int i = 0; i < 16; i++) {
        o[i] = a[i] - b[i];
    }
}

static void fe_mul(fe o, const fe a, const fe b)
{
    int64_t t[31];
    for (int i = 0; i < 31; i++) {
        t[i] = 0;
    }
    for (int i = 0; i < 16; i++) {
        for (int j = 0; j < 16; j++) {
            t[i + j] += a[i] * b[j];
        }
    }
    for (int i = 0; i < 15; i++) {
        t[i] += 38 * t[i + 16];
    }
    for (int i = 0; i < 16; i++) {
        o[i] = t[i];
    }
    fe_carry(o);
    fe_carry(o);
}

static void fe_sq(fe o, const fe a)
{
    fe_mul(o, a, a);
}

/* a^(p-2) = a^(2^255 - 21); the exponent's bits are public constants. */
static void fe_invert(fe o, const fe a)
{
    fe c;
    fe_copy(c, a);
    for (int i = 253; i >= 0; i--) {
        fe_sq(c, c);
        if (i != 2 && i != 4) {
            fe_mul(c, c, a);
        }
    }
    fe_copy(o, c);
}

static void fe_decode(fe o, const uint8_t in[32])
{
    for (int i = 0; i < 16; i++) {
        o[i] = (int64_t)in[2 * i] + ((int64_t)in[2 * i + 1] << 8);
    }
    o[15] &= 0x7fff;
}

/* Canonical little-endian encoding: subtract p up to twice with masks. */
static void fe_encode(uint8_t out[32], const fe n)
{
    fe t;
    fe m;
    fe_copy(t, n);
    fe_carry(t);
    fe_carry(t);
    fe_carry(t);
    for (int j = 0; j < 2; j++) {
        int64_t borrow;
        m[0] = t[0] - 0xffed;
        for (int i = 1; i < 15; i++) {
            m[i] = t[i] - 0xffff - ((m[i - 1] >> 16) & 1);
            m[i - 1] &= 0xffff;
        }
        m[15] = t[15] - 0x7fff - ((m[14] >> 16) & 1);
        borrow = (m[15] >> 16) & 1;
        m[14] &= 0xffff;
        fe_cswap(t, m, 1 - borrow);
    }
    for (int i = 0; i < 16; i++) {
        out[2 * i] = (uint8_t)(t[i] & 0xff);
        out[2 * i + 1] = (uint8_t)((t[i] >> 8) & 0xff);
    }
}

void d011_x25519(uint8_t out[32], const uint8_t scalar[32], const uint8_t u[32])
{
    static const fe a24 = {0xdb41, 1};
    uint8_t k[32];
    fe x1, x2, z2, x3, z3, a, aa, b, bb, e, c, d, da, cb, t;
    int64_t swap = 0;
    for (int i = 0; i < 32; i++) {
        k[i] = scalar[i];
    }
    k[0] &= 248;
    k[31] &= 127;
    k[31] |= 64;
    fe_decode(x1, u);
    for (int i = 0; i < 16; i++) {
        x2[i] = 0;
        z2[i] = 0;
        z3[i] = 0;
    }
    x2[0] = 1;
    z3[0] = 1;
    fe_copy(x3, x1);
    for (int pos = 254; pos >= 0; pos--) {
        int64_t bit = (k[pos >> 3] >> (pos & 7)) & 1;
        swap ^= bit;
        fe_cswap(x2, x3, swap);
        fe_cswap(z2, z3, swap);
        swap = bit;
        fe_add(a, x2, z2);
        fe_sq(aa, a);
        fe_sub(b, x2, z2);
        fe_sq(bb, b);
        fe_sub(e, aa, bb);
        fe_add(c, x3, z3);
        fe_sub(d, x3, z3);
        fe_mul(da, d, a);
        fe_mul(cb, c, b);
        fe_add(t, da, cb);
        fe_sq(x3, t);
        fe_sub(t, da, cb);
        fe_sq(t, t);
        fe_mul(z3, x1, t);
        fe_mul(x2, aa, bb);
        fe_mul(t, a24, e);
        fe_add(t, aa, t);
        fe_mul(z2, e, t);
    }
    fe_cswap(x2, x3, swap);
    fe_cswap(z2, z3, swap);
    fe_invert(z2, z2);
    fe_mul(x2, x2, z2);
    fe_encode(out, x2);
    d011_wipe(k, sizeof k);
    d011_wipe(x2, sizeof x2);
    d011_wipe(z2, sizeof z2);
    d011_wipe(x3, sizeof x3);
    d011_wipe(z3, sizeof z3);
}

#endif /* D011_PART_X25519 */

/* ======================================================================
 * AES-128/256 (FIPS 197) and AES-GCM with a
 * 96-bit IV (NIST SP 800-38D). Contributor-written portable C, not Orange
 * output. The S-box is computed, never looked up: the inverse in GF(2^8)
 * is x^254 by masked shift-and-add multiplication, followed by the affine
 * map, so no memory address depends on key or data. GHASH multiplies
 * bit by bit with masks. The resulting code is slow and table-free.
 */
#if defined(D011_PART_AES_GCM)

static uint8_t gf8_mul(uint8_t a, uint8_t b)
{
    uint8_t r = 0;
    for (int i = 0; i < 8; i++) {
        r ^= (uint8_t)((0U - (uint32_t)(b & 1)) & a);
        a = (uint8_t)((a << 1) ^ ((0U - (uint32_t)(a >> 7)) & 0x1bU));
        b >>= 1;
    }
    return r;
}

static uint8_t rotl8(uint8_t x, unsigned n)
{
    return (uint8_t)((x << n) | (x >> (8U - n)));
}

/* FIPS 197 section 5.1.1: multiplicative inverse (0 maps to 0), then the
 * affine transformation. */
static uint8_t sbox(uint8_t x)
{
    uint8_t x2 = gf8_mul(x, x);
    uint8_t x4 = gf8_mul(x2, x2);
    uint8_t x8 = gf8_mul(x4, x4);
    uint8_t x16 = gf8_mul(x8, x8);
    uint8_t x32 = gf8_mul(x16, x16);
    uint8_t x64 = gf8_mul(x32, x32);
    uint8_t x128 = gf8_mul(x64, x64);
    uint8_t inv = gf8_mul(gf8_mul(gf8_mul(x2, x4), gf8_mul(x8, x16)),
                          gf8_mul(gf8_mul(x32, x64), x128));
    return (uint8_t)(inv ^ rotl8(inv, 1) ^ rotl8(inv, 2) ^ rotl8(inv, 3) ^
                     rotl8(inv, 4) ^ 0x63U);
}

/* FIPS 197 section 5.2. Returns the number of rounds, or -1 for an
 * unsupported key length. The key length is public. */
int d011_aes_expand_key(uint8_t rk[240], const uint8_t *key, size_t key_len)
{
    int nk;
    int nr;
    uint8_t rcon = 1;
    if (key_len == 16) {
        nk = 4;
        nr = 10;
    } else if (key_len == 32) {
        nk = 8;
        nr = 14;
    } else {
        return -1;
    }
    for (int i = 0; i < 4 * nk; i++) {
        rk[i] = key[i];
    }
    for (int i = nk; i < 4 * (nr + 1); i++) {
        uint8_t t[4];
        for (int j = 0; j < 4; j++) {
            t[j] = rk[4 * (i - 1) + j];
        }
        if ((i & (nk - 1)) == 0) {
            uint8_t first = t[0];
            t[0] = (uint8_t)(sbox(t[1]) ^ rcon);
            t[1] = sbox(t[2]);
            t[2] = sbox(t[3]);
            t[3] = sbox(first);
            rcon = (uint8_t)((rcon << 1) ^ ((0U - (uint32_t)(rcon >> 7)) & 0x1bU));
        } else if (nk == 8 && (i & 7) == 4) {
            for (int j = 0; j < 4; j++) {
                t[j] = sbox(t[j]);
            }
        }
        for (int j = 0; j < 4; j++) {
            rk[4 * i + j] = (uint8_t)(rk[4 * (i - nk) + j] ^ t[j]);
        }
    }
    return nr;
}

static uint8_t xtime(uint8_t x)
{
    return (uint8_t)((x << 1) ^ ((0U - (uint32_t)(x >> 7)) & 0x1bU));
}

static void aes_rounds(uint8_t out[16], const uint8_t *rk, int nr, const uint8_t in[16])
{
    uint8_t s[16];
    uint8_t t[16];
    for (int i = 0; i < 16; i++) {
        s[i] = (uint8_t)(in[i] ^ rk[i]);
    }
    for (int r = 1; r <= nr; r++) {
        /* SubBytes and ShiftRows: row j of column c moves to column c - j. */
        for (int c = 0; c < 4; c++) {
            for (int j = 0; j < 4; j++) {
                t[4 * c + j] = sbox(s[4 * ((c + j) & 3) + j]);
            }
        }
        if (r != nr) {
            for (int c = 0; c < 4; c++) {
                uint8_t a0 = t[4 * c], a1 = t[4 * c + 1], a2 = t[4 * c + 2], a3 = t[4 * c + 3];
                uint8_t all = (uint8_t)(a0 ^ a1 ^ a2 ^ a3);
                t[4 * c] = (uint8_t)(a0 ^ all ^ xtime((uint8_t)(a0 ^ a1)));
                t[4 * c + 1] = (uint8_t)(a1 ^ all ^ xtime((uint8_t)(a1 ^ a2)));
                t[4 * c + 2] = (uint8_t)(a2 ^ all ^ xtime((uint8_t)(a2 ^ a3)));
                t[4 * c + 3] = (uint8_t)(a3 ^ all ^ xtime((uint8_t)(a3 ^ a0)));
            }
        }
        for (int i = 0; i < 16; i++) {
            s[i] = (uint8_t)(t[i] ^ rk[16 * r + i]);
        }
    }
    for (int i = 0; i < 16; i++) {
        out[i] = s[i];
    }
    d011_wipe(s, sizeof s);
    d011_wipe(t, sizeof t);
}

int d011_aes_encrypt_block(uint8_t out[16], const uint8_t *key, size_t key_len,
                           const uint8_t in[16])
{
    uint8_t rk[240];
    int nr = d011_aes_expand_key(rk, key, key_len);
    if (nr < 0) {
        return -1;
    }
    aes_rounds(out, rk, nr, in);
    d011_wipe(rk, sizeof rk);
    return 0;
}

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

/* SP 800-38D section 6.3, Algorithm 1, with masks instead of branches. */
static void ghash_mul_portable(uint8_t out[16], const uint8_t x[16], const uint8_t y[16])
{
    uint64_t xh = load_be64(x), xl = load_be64(x + 8);
    uint64_t vh = load_be64(y), vl = load_be64(y + 8);
    uint64_t zh = 0, zl = 0;
    for (int half = 0; half < 2; half++) {
        uint64_t word = half == 0 ? xh : xl;
        for (int i = 63; i >= 0; i--) {
            uint64_t take = 0U - ((word >> i) & 1U);
            uint64_t reduce = 0U - (vl & 1U);
            zh ^= vh & take;
            zl ^= vl & take;
            vl = (vl >> 1) | (vh << 63);
            vh = (vh >> 1) ^ (0xe100000000000000ULL & reduce);
        }
    }
    store_be64(out, zh);
    store_be64(out + 8, zl);
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
        ghash_mul_portable(acc, block, h);
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
        aes_rounds(ks, rk, nr, ctr);
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
    aes_rounds(h, rk, nr, zero);
    ghash_update(acc, h, aad, aad_len);
    ghash_update(acc, h, ct, ct_len);
    store_be64(lengths, (uint64_t)aad_len << 3);
    store_be64(lengths + 8, (uint64_t)ct_len << 3);
    ghash_update(acc, h, lengths, 16);
    aes_rounds(ek, rk, nr, j0);
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

/* out receives pt_len bytes of ciphertext and the 16-byte tag. */
int d011_aes_gcm_seal(uint8_t *out, const uint8_t *key, size_t key_len, const uint8_t iv[12],
                      const uint8_t *aad, size_t aad_len, const uint8_t *pt, size_t pt_len)
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

int d011_aes_gcm_open(uint8_t *out, const uint8_t *key, size_t key_len, const uint8_t iv[12],
                      const uint8_t *aad, size_t aad_len, const uint8_t *ct, size_t ct_len)
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

#endif /* D011_PART_AES_GCM */

/* ======================================================================
 * SHA3-256 and SHAKE128 (FIPS 202), the
 * symmetric core that ML-KEM and ML-DSA build on. Contributor-written
 * portable C, not Orange output. The round constants, rotation offsets
 * and lane permutation were derived from FIPS 202 sections 3.2.2 to
 * 3.2.5. Index arithmetic uses tables, never division or remainder.
 */
#if defined(D011_PART_KECCAK)

static const uint64_t RC[24] = {
    0x0000000000000001ULL, 0x0000000000008082ULL,
    0x800000000000808aULL, 0x8000000080008000ULL,
    0x000000000000808bULL, 0x0000000080000001ULL,
    0x8000000080008081ULL, 0x8000000000008009ULL,
    0x000000000000008aULL, 0x0000000000000088ULL,
    0x0000000080008009ULL, 0x000000008000000aULL,
    0x000000008000808bULL, 0x800000000000008bULL,
    0x8000000000008089ULL, 0x8000000000008003ULL,
    0x8000000000008002ULL, 0x8000000000000080ULL,
    0x000000000000800aULL, 0x800000008000000aULL,
    0x8000000080008081ULL, 0x8000000000008080ULL,
    0x0000000080000001ULL, 0x8000000080008008ULL,
};

static const unsigned char ROT[24] = {
    1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14, 27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44,
};

static const unsigned char PI[24] = {
    10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1,
};

static const unsigned char NEXT1[5] = {1, 2, 3, 4, 0};
static const unsigned char NEXT2[5] = {2, 3, 4, 0, 1};
static const unsigned char PREV1[5] = {4, 0, 1, 2, 3};

static uint64_t rotl64(uint64_t x, unsigned n)
{
    return (x << n) | (x >> (64U - n));
}

static void keccak_f1600(uint64_t st[25])
{
    uint64_t bc[5];
    for (int round = 0; round < 24; round++) {
        for (int i = 0; i < 5; i++) {
            bc[i] = st[i] ^ st[i + 5] ^ st[i + 10] ^ st[i + 15] ^ st[i + 20];
        }
        for (int i = 0; i < 5; i++) {
            uint64_t t = bc[PREV1[i]] ^ rotl64(bc[NEXT1[i]], 1);
            for (int j = 0; j < 25; j += 5) {
                st[j + i] ^= t;
            }
        }
        {
            uint64_t t = st[1];
            for (int i = 0; i < 24; i++) {
                int j = PI[i];
                uint64_t next = st[j];
                st[j] = rotl64(t, ROT[i]);
                t = next;
            }
        }
        for (int j = 0; j < 25; j += 5) {
            for (int i = 0; i < 5; i++) {
                bc[i] = st[j + i];
            }
            for (int i = 0; i < 5; i++) {
                st[j + i] ^= (~bc[NEXT1[i]]) & bc[NEXT2[i]];
            }
        }
        st[0] ^= RC[round];
    }
}

static void absorb_byte(uint64_t st[25], size_t pos, uint8_t b)
{
    st[pos >> 3] ^= (uint64_t)b << (8 * (pos & 7));
}

static uint8_t squeeze_byte(const uint64_t st[25], size_t pos)
{
    return (uint8_t)(st[pos >> 3] >> (8 * (pos & 7)));
}

static void sponge(uint8_t *out, size_t out_len, const uint8_t *msg, size_t len,
                   size_t rate, uint8_t suffix)
{
    uint64_t st[25];
    size_t pos = 0;
    for (int i = 0; i < 25; i++) {
        st[i] = 0;
    }
    for (size_t i = 0; i < len; i++) {
        absorb_byte(st, pos, msg[i]);
        pos++;
        if (pos == rate) {
            keccak_f1600(st);
            pos = 0;
        }
    }
    absorb_byte(st, pos, suffix);
    absorb_byte(st, rate - 1, 0x80);
    keccak_f1600(st);
    pos = 0;
    for (size_t i = 0; i < out_len; i++) {
        if (pos == rate) {
            keccak_f1600(st);
            pos = 0;
        }
        out[i] = squeeze_byte(st, pos);
        pos++;
    }
    d011_wipe(st, sizeof st);
}

void d011_sha3_256(uint8_t out[32], const uint8_t *msg, size_t len)
{
    sponge(out, 32, msg, len, 136, 0x06);
}

void d011_shake128(uint8_t *out, size_t out_len, const uint8_t *msg, size_t len)
{
    sponge(out, out_len, msg, len, 168, 0x1f);
}

#endif /* D011_PART_KECCAK */

/* ======================================================================
 * wiping and constant-time comparison shared
 * by the kernels. Contributor-written portable C, not Orange output.
 */
#if defined(D011_PART_CT)

void d011_wipe(void *p, size_t len)
{
    volatile uint8_t *d = p;
    for (size_t i = 0; i < len; i++) {
        d[i] = 0;
    }
}

/* 0 when the byte strings are equal and 1 otherwise, without a
 * data-dependent branch. */
uint32_t d011_ct_differs(const uint8_t *a, const uint8_t *b, size_t len)
{
    uint32_t acc = 0;
    for (size_t i = 0; i < len; i++) {
        acc |= (uint32_t)(a[i] ^ b[i]);
    }
    return (acc | (0U - acc)) >> 31;
}

#endif /* D011_PART_CT */
