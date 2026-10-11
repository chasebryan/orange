/* Exact integers for the standalone compiler.
   src/bigint.c owns the arena and the magnitude arithmetic. src/compile.c
   uses both for Int values, word-literal range checks, and `as` residues.

   Magnitudes are little-endian 32-bit limbs with no leading zero.
   Zero is an empty magnitude and is never negative.
   A value is admitted only while its magnitude has at most 16384
   significant bits. Limbs are immutable after publication. */

#ifndef ORANGE_BIGINT_H
#define ORANGE_BIGINT_H

#include <stddef.h>
#include <stdint.h>

#define ORANGE_MAX_BITS 16384u

/* Bump allocator. One arena holds every limb published for one source.
   arena_dispose frees the block. Individual Big values are not freed. */

typedef struct Arena {
    unsigned char *base;
    size_t used;
    size_t cap;
} Arena;

/* negative is 0 or 1. A zero magnitude forces negative to 0. */
typedef struct Big {
    uint32_t *limbs;
    uint32_t nlimbs;
    int negative;
} Big;

int arena_init(Arena *arena, size_t cap);
void arena_dispose(Arena *arena);
void *arena_alloc(Arena *arena, size_t size, size_t align);

Big big_zero(void);
uint32_t big_bits(const Big *value);
uint32_t big_limbs(const Big *value);

int big_from_u64(Arena *arena, uint64_t value, Big *out);

/* Decimal, `0x` hex, or `0b` binary digits. Underscores are skipped.
   Digits accumulate in a fixed scratch; only the finished magnitude is
   stored in the arena. Returns 0 for a bad digit, an exhausted arena, or
   a magnitude past ORANGE_MAX_BITS. */
int big_from_digits(Arena *arena, const char *text, size_t length, int negative, Big *out);

/* Arithmetic returns 0 when the arena is exhausted or the result would
   exceed ORANGE_MAX_BITS. big_neg flips the sign of a non-zero value and
   does not allocate. */
int big_add(Arena *arena, const Big *left, const Big *right, Big *out);
int big_sub(Arena *arena, const Big *left, const Big *right, Big *out);
int big_mul(Arena *arena, const Big *left, const Big *right, Big *out);
int big_neg(const Big *value, Big *out);
/* value * 2^amount. Fails when the result would exceed 16384 significant bits. */
int big_shl(Arena *arena, const Big *value, uint32_t amount, Big *out);

/* Signed comparison: -1 when left < right, 0 when equal, 1 when left > right. */
int big_cmp(const Big *left, const Big *right);

/* Euclidean quotient and remainder. The remainder is in 0 through
   |divisor| - 1. A zero divisor returns 0 and writes no result. */
int big_div_euclid(Arena *arena, const Big *dividend, const Big *divisor, Big *quot, Big *rem);

/* Non-negative residue of value modulo 2^width, for width from 1 through 64.
   A negative magnitude wraps, so -1 at width 8 is 255. */
int big_mod_pow2(const Big *value, uint32_t width, uint64_t *out);

/* Decimal spelling, with a leading '-' when negative. `buffer` is NUL-terminated. */
int big_format(const Big *value, char *buffer, size_t capacity);

/* Fixed known values used by `orangec --self-test`. Returns 1 on success. */
int bigint_self_test(void);

#endif
