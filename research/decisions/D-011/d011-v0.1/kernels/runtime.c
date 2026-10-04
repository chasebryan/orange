/*
 * D-011 v0.1 laboratory runtime: the freestanding program around the
 * stand-in kernels. Contributor-written, NOT Orange output, and not part
 * of any kernel gate. D011_PART_RT is the entry point and the three Linux
 * system calls for each target; D011_PART_MEM the byte helpers a compiler
 * may call; D011_PART_DRIVER the request loop; D011_PART_CONTROL the two
 * deliberately non-constant-time detector controls.
 */
#include "d011_kernels.h"

long rt_read(int fd, void *buf, size_t len);
long rt_write(int fd, const void *buf, size_t len);
_Noreturn void rt_exit(int code);
_Noreturn void d011_start(void);
uint32_t d011_control_first_difference(const uint8_t *a, const uint8_t *b, size_t len);
uint32_t d011_control_divide(uint32_t a, uint32_t b);

/* ======================================================================
 * Entry point and system calls: _start calls d011_start with a 16-byte
 * aligned stack.
 */
#if defined(D011_PART_RT)

#if defined(__x86_64__)
/* x86-64 Linux, System V AMD64 psABI. */
static long syscall3(long n, long a, long b, long c)
{
    long ret;
    __asm__ volatile("syscall"
                     : "=a"(ret)
                     : "a"(n), "D"(a), "S"(b), "d"(c)
                     : "rcx", "r11", "memory");
    return ret;
}

long rt_read(int fd, void *buf, size_t len)
{
    return syscall3(0, fd, (long)buf, (long)len);
}

long rt_write(int fd, const void *buf, size_t len)
{
    return syscall3(1, fd, (long)buf, (long)len);
}

_Noreturn void rt_exit(int code)
{
    for (;;) {
        (void)syscall3(231, code, 0, 0);
    }
}

__asm__(".text\n"
        ".globl _start\n"
        ".type _start, @function\n"
        "_start:\n"
        "    xorl %ebp, %ebp\n"
        "    andq $-16, %rsp\n"
        "    call d011_start\n"
        "    hlt\n");
#elif defined(__aarch64__)
/* AArch64 Linux, AAPCS64. */
static long syscall3(long n, long a, long b, long c)
{
    register long x8 __asm__("x8") = n;
    register long x0 __asm__("x0") = a;
    register long x1 __asm__("x1") = b;
    register long x2 __asm__("x2") = c;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8), "r"(x1), "r"(x2) : "memory");
    return x0;
}

long rt_read(int fd, void *buf, size_t len)
{
    return syscall3(63, fd, (long)buf, (long)len);
}

long rt_write(int fd, const void *buf, size_t len)
{
    return syscall3(64, fd, (long)buf, (long)len);
}

_Noreturn void rt_exit(int code)
{
    for (;;) {
        (void)syscall3(94, code, 0, 0);
    }
}

__asm__(".text\n"
        ".globl _start\n"
        ".type _start, %function\n"
        "_start:\n"
        "    mov x29, #0\n"
        "    mov x30, #0\n"
        "    mov x9, sp\n"
        "    and x9, x9, #-16\n"
        "    mov sp, x9\n"
        "    bl d011_start\n"
        "    brk #0\n");
#elif defined(__riscv) && __riscv_xlen == 64
/* RV64 Linux, LP64D psABI. The laboratory links with relaxation
 * disabled, so the global pointer is never used and _start does not set
 * it. */
static long syscall3(long n, long a, long b, long c)
{
    register long a7 __asm__("a7") = n;
    register long a0 __asm__("a0") = a;
    register long a1 __asm__("a1") = b;
    register long a2 __asm__("a2") = c;
    __asm__ volatile("ecall" : "+r"(a0) : "r"(a7), "r"(a1), "r"(a2) : "memory");
    return a0;
}

long rt_read(int fd, void *buf, size_t len)
{
    return syscall3(63, fd, (long)buf, (long)len);
}

long rt_write(int fd, const void *buf, size_t len)
{
    return syscall3(64, fd, (long)buf, (long)len);
}

_Noreturn void rt_exit(int code)
{
    for (;;) {
        (void)syscall3(94, code, 0, 0);
    }
}

__asm__(".text\n"
        ".globl _start\n"
        ".type _start, @function\n"
        "_start:\n"
        "    li ra, 0\n"
        "    andi sp, sp, -16\n"
        "    call d011_start\n"
        "    ebreak\n");
#else
#error "D-011: no runtime for this target"
#endif /* target */

#endif /* D011_PART_RT */

/* ======================================================================
 * Freestanding byte helpers. Compilers may emit calls to memcpy, memmove, memset and memcmp for copies and
 * initializers, so the freestanding laboratory binaries define them here;
 * hosted consumers (the Rust ABI probe) take them from their C library.
 * Every loop bound is a public length. Contributor-written, not Orange
 * output.
 */
#if defined(D011_PART_MEM)

void *memcpy(void *restrict dst, const void *restrict src, size_t len);
void *memmove(void *dst, const void *src, size_t len);
void *memset(void *dst, int value, size_t len);
int memcmp(const void *a, const void *b, size_t len);

void *memcpy(void *restrict dst, const void *restrict src, size_t len)
{
    uint8_t *d = dst;
    const uint8_t *s = src;
    for (size_t i = 0; i < len; i++) {
        d[i] = s[i];
    }
    return dst;
}

void *memmove(void *dst, const void *src, size_t len)
{
    uint8_t *d = dst;
    const uint8_t *s = src;
    if (d < s) {
        for (size_t i = 0; i < len; i++) {
            d[i] = s[i];
        }
    } else {
        for (size_t i = len; i > 0; i--) {
            d[i - 1] = s[i - 1];
        }
    }
    return dst;
}

void *memset(void *dst, int value, size_t len)
{
    uint8_t *d = dst;
    for (size_t i = 0; i < len; i++) {
        d[i] = (uint8_t)value;
    }
    return dst;
}

/* Only the driver compares public bytes with memcmp; kernels use
 * d011_ct_differs for secret comparisons. */
int memcmp(const void *a, const void *b, size_t len)
{
    const uint8_t *x = a;
    const uint8_t *y = b;
    for (size_t i = 0; i < len; i++) {
        if (x[i] != y[i]) {
            return x[i] < y[i] ? -1 : 1;
        }
    }
    return 0;
}

#endif /* D011_PART_MEM */

/* ======================================================================
 * The request loop.
 *
 * Each request on standard input is one operation byte, one argument-count
 * byte, and that many arguments, each a 32-bit little-endian length and its
 * bytes. Each response on standard output is one status byte (0 ok,
 * 1 rejected, 2 unsupported in this build, 3 malformed), a 32-bit
 * little-endian length and the output bytes. End of input before a request
 * exits 0; a truncated or oversized request exits 2. Operation codes,
 * argument counts and lengths are public; the driver branches on nothing
 * else, so secret-varied requests of one shape take one path.
 */
#if defined(D011_PART_DRIVER)

#define ARENA_BYTES 65536U
#define OUT_BYTES 16400U
#define MAX_ARGS 4U

enum {
    ST_OK = 0,
    ST_REJECTED = 1,
    ST_UNSUPPORTED = 2,
    ST_MALFORMED = 3,
};

static uint8_t arena[ARENA_BYTES];
static uint8_t out[OUT_BYTES];

static int read_exact(uint8_t *buf, size_t len, int eof_ok)
{
    size_t got = 0;
    while (got < len) {
        long n = rt_read(0, buf + got, len - got);
        if (n == 0 && got == 0 && eof_ok) {
            return 1;
        }
        if (n <= 0) {
            return -1;
        }
        got += (size_t)n;
    }
    return 0;
}

static void write_all(const uint8_t *buf, size_t len)
{
    size_t done = 0;
    while (done < len) {
        long n = rt_write(1, buf + done, len - done);
        if (n <= 0) {
            rt_exit(3);
        }
        done += (size_t)n;
    }
}

static void respond(uint8_t status, size_t len)
{
    uint8_t head[5];
    head[0] = status;
    head[1] = (uint8_t)len;
    head[2] = (uint8_t)(len >> 8);
    head[3] = (uint8_t)(len >> 16);
    head[4] = (uint8_t)(len >> 24);
    write_all(head, 5);
    write_all(out, len);
}

static uint32_t le32(const uint8_t *p)
{
    return (uint32_t)p[0] | ((uint32_t)p[1] << 8) | ((uint32_t)p[2] << 16) |
           ((uint32_t)p[3] << 24);
}

struct request {
    uint8_t op;
    uint8_t count;
    const uint8_t *arg[MAX_ARGS];
    size_t len[MAX_ARGS];
};

static int shape(const struct request *rq, unsigned count)
{
    return rq->count == count;
}

static int fixed(const struct request *rq, unsigned index, size_t len)
{
    return rq->len[index] == len;
}

static void put_le32(uint8_t *p, uint32_t v)
{
    p[0] = (uint8_t)v;
    p[1] = (uint8_t)(v >> 8);
    p[2] = (uint8_t)(v >> 16);
    p[3] = (uint8_t)(v >> 24);
}

static int aes_key(const struct request *rq, unsigned index)
{
    return rq->len[index] == 16 || rq->len[index] == 32;
}

/* Run one parsed request; returns its status and sets *n to the output
 * length. */
static uint8_t run(const struct request *rq, size_t *n)
{
    const uint8_t *const *a = rq->arg;
    const size_t *l = rq->len;
    *n = 0;
    switch (rq->op) {
    case 1:
        if (!shape(rq, 1)) {
            return ST_MALFORMED;
        }
        d011_sha256(out, a[0], l[0]);
        *n = 32;
        return ST_OK;
    case 2:
        if (!shape(rq, 1) || !fixed(rq, 0, 64)) {
            return ST_MALFORMED;
        }
        d011_sha256_probe(out, a[0]);
        *n = 72;
        return ST_OK;
    case 3:
        if (!shape(rq, 1)) {
            return ST_MALFORMED;
        }
        d011_sha512(out, a[0], l[0]);
        *n = 64;
        return ST_OK;
    case 4:
        if (!shape(rq, 2)) {
            return ST_MALFORMED;
        }
        d011_hmac_sha256(out, a[0], l[0], a[1], l[1]);
        *n = 32;
        return ST_OK;
    case 5:
        if (!shape(rq, 4) || !fixed(rq, 3, 4) || le32(a[3]) > 255U * 32U) {
            return ST_MALFORMED;
        }
        if (d011_hkdf_sha256(out, le32(a[3]), a[0], l[0], a[1], l[1], a[2], l[2]) != 0) {
            return ST_MALFORMED;
        }
        *n = le32(a[3]);
        return ST_OK;
    case 6:
        if (!shape(rq, 3) || !fixed(rq, 0, 32) || !fixed(rq, 1, 4) || !fixed(rq, 2, 12)) {
            return ST_MALFORMED;
        }
        d011_chacha20_block(out, a[0], le32(a[1]), a[2]);
        *n = 64;
        return ST_OK;
    case 7:
        if (!shape(rq, 4) || !fixed(rq, 0, 32) || !fixed(rq, 1, 4) || !fixed(rq, 2, 12) ||
            l[3] > OUT_BYTES) {
            return ST_MALFORMED;
        }
        d011_chacha20_xor(out, a[3], l[3], a[0], le32(a[1]), a[2]);
        *n = l[3];
        return ST_OK;
    case 8:
        if (!shape(rq, 2) || !fixed(rq, 0, 32)) {
            return ST_MALFORMED;
        }
        d011_poly1305(out, a[1], l[1], a[0]);
        *n = 16;
        return ST_OK;
    case 9:
        if (!shape(rq, 4) || !fixed(rq, 0, 32) || !fixed(rq, 1, 12) || l[3] + 16 > OUT_BYTES) {
            return ST_MALFORMED;
        }
        d011_aead_seal(out, a[0], a[1], a[2], l[2], a[3], l[3]);
        *n = l[3] + 16;
        return ST_OK;
    case 10:
        if (!shape(rq, 4) || !fixed(rq, 0, 32) || !fixed(rq, 1, 12) || l[3] < 16 ||
            l[3] > OUT_BYTES) {
            return ST_MALFORMED;
        }
        if (d011_aead_open(out, a[0], a[1], a[2], l[2], a[3], l[3]) != 0) {
            return ST_REJECTED;
        }
        *n = l[3] - 16;
        return ST_OK;
    case 11:
        if (!shape(rq, 2) || !fixed(rq, 0, 32) || !fixed(rq, 1, 32)) {
            return ST_MALFORMED;
        }
        d011_x25519(out, a[0], a[1]);
        *n = 32;
        return ST_OK;
    case 12:
    case 17:
        if (!shape(rq, 2) || !aes_key(rq, 0) || !fixed(rq, 1, 16)) {
            return ST_MALFORMED;
        }
        if (rq->op == 12) {
            (void)d011_aes_encrypt_block(out, a[0], l[0], a[1]);
        } else {
#ifdef D011_CRYPTO_PROFILE
            (void)d011_accel_aes_encrypt_block(out, a[0], l[0], a[1]);
#else
            return ST_UNSUPPORTED;
#endif
        }
        *n = 16;
        return ST_OK;
    case 13:
    case 18:
        if (!shape(rq, 4) || !aes_key(rq, 0) || !fixed(rq, 1, 12) || l[3] + 16 > OUT_BYTES) {
            return ST_MALFORMED;
        }
        if (rq->op == 13) {
            (void)d011_aes_gcm_seal(out, a[0], l[0], a[1], a[2], l[2], a[3], l[3]);
        } else {
#ifdef D011_CRYPTO_PROFILE
            (void)d011_accel_aes_gcm_seal(out, a[0], l[0], a[1], a[2], l[2], a[3], l[3]);
#else
            return ST_UNSUPPORTED;
#endif
        }
        *n = l[3] + 16;
        return ST_OK;
    case 14:
    case 19:
        if (!shape(rq, 4) || !aes_key(rq, 0) || !fixed(rq, 1, 12) || l[3] < 16 ||
            l[3] > OUT_BYTES) {
            return ST_MALFORMED;
        }
        if (rq->op == 14) {
            if (d011_aes_gcm_open(out, a[0], l[0], a[1], a[2], l[2], a[3], l[3]) != 0) {
                return ST_REJECTED;
            }
        } else {
#ifdef D011_CRYPTO_PROFILE
            if (d011_accel_aes_gcm_open(out, a[0], l[0], a[1], a[2], l[2], a[3], l[3]) != 0) {
                return ST_REJECTED;
            }
#else
            return ST_UNSUPPORTED;
#endif
        }
        *n = l[3] - 16;
        return ST_OK;
    case 15:
        if (!shape(rq, 1)) {
            return ST_MALFORMED;
        }
        d011_sha3_256(out, a[0], l[0]);
        *n = 32;
        return ST_OK;
    case 16:
        if (!shape(rq, 2) || !fixed(rq, 1, 4) || le32(a[1]) > OUT_BYTES) {
            return ST_MALFORMED;
        }
        d011_shake128(out, le32(a[1]), a[0], l[0]);
        *n = le32(a[1]);
        return ST_OK;
    case 20:
        /* Detector control: data-dependent early exit. */
        if (!shape(rq, 2) || l[0] != l[1]) {
            return ST_MALFORMED;
        }
        put_le32(out, d011_control_first_difference(a[0], a[1], l[0]));
        *n = 4;
        return ST_OK;
    case 21:
        /* Detector control: division by a runtime value. */
        if (!shape(rq, 2) || !fixed(rq, 0, 4) || !fixed(rq, 1, 4)) {
            return ST_MALFORMED;
        }
        put_le32(out, d011_control_divide(le32(a[0]), le32(a[1])));
        *n = 4;
        return ST_OK;
    default:
        return ST_UNSUPPORTED;
    }
}

_Noreturn void d011_start(void)
{
    for (;;) {
        struct request rq;
        uint8_t head[2];
        size_t used = 0;
        size_t n;
        uint8_t status;
        int got = read_exact(head, 2, 1);
        if (got == 1) {
            rt_exit(0);
        }
        if (got < 0 || head[1] > MAX_ARGS) {
            rt_exit(2);
        }
        rq.op = head[0];
        rq.count = head[1];
        for (unsigned i = 0; i < rq.count; i++) {
            uint8_t len[4];
            size_t size;
            if (read_exact(len, 4, 0) != 0) {
                rt_exit(2);
            }
            size = le32(len);
            if (size > ARENA_BYTES - used) {
                rt_exit(2);
            }
            if (read_exact(arena + used, size, 0) != 0) {
                rt_exit(2);
            }
            rq.arg[i] = arena + used;
            rq.len[i] = size;
            used += size;
        }
        status = run(&rq, &n);
        respond(status, status == ST_OK ? n : 0);
    }
}

#endif /* D011_PART_DRIVER */

/* ======================================================================
 * Detector controls. These two functions are deliberately NOT constant-time: they exist so that every build proves its
 * detectors work. d011_control_first_difference returns at the first
 * differing byte, so its control flow depends on the data and the trace
 * comparison must see different traces. d011_control_divide divides by a
 * runtime value, so the static inventory must find a division instruction
 * (unless a target has none, which the laboratory then records). They are
 * not stand-in kernels and are excluded from every kernel gate.
 */
#if defined(D011_PART_CONTROL)

uint32_t d011_control_first_difference(const uint8_t *a, const uint8_t *b, size_t len)
{
    for (size_t i = 0; i < len; i++) {
        if (a[i] != b[i]) {
            return (uint32_t)i;
        }
    }
    return (uint32_t)len;
}

uint32_t d011_control_divide(uint32_t a, uint32_t b)
{
    return b == 0 ? 0 : a / b;
}

#endif /* D011_PART_CONTROL */
