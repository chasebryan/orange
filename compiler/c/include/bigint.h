#ifndef ORANGE_BIGINT_H
#define ORANGE_BIGINT_H

#include <stddef.h>
#include <stdint.h>

/* Exact integers used by the standalone compiler.
   Magnitudes are little-endian 32-bit limbs with no leading zero.
   Zero is an empty magnitude and is never negative.
   A value is admitted only while its magnitude has at most 16384
   significant bits. Limbs are immutable after publication. */

#define ORANGE_MAX_BITS 16384u

typedef struct Arena {
    unsigned char *base;
    size_t used;
    size_t cap;
} Arena;

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
int big_from_digits(Arena *arena, const char *text, size_t length, int negative, Big *out);

int big_add(Arena *arena, const Big *left, const Big *right, Big *out);
int big_sub(Arena *arena, const Big *left, const Big *right, Big *out);
int big_mul(Arena *arena, const Big *left, const Big *right, Big *out);
int big_neg(const Big *value, Big *out);
/* value * 2^amount. Fails when the result would exceed 16384 significant bits. */
int big_shl(Arena *arena, const Big *value, uint32_t amount, Big *out);
int big_cmp(const Big *left, const Big *right);

/* Euclidean quotient and remainder: 0 <= rem < |divisor|, or 0 if divisor is 0. */
int big_div_euclid(Arena *arena, const Big *dividend, const Big *divisor, Big *quot, Big *rem);

/* Mathematical residue of value modulo 2^width, for width in 1..=64. */
int big_mod_pow2(const Big *value, uint32_t width, uint64_t *out);

/* Decimal spelling, with a leading '-' when negative. `buffer` is NUL-terminated. */
int big_format(const Big *value, char *buffer, size_t capacity);

int bigint_self_test(void);

#endif
