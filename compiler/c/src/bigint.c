#include "bigint.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int arena_init(Arena *arena, size_t cap) {
    arena->base = NULL;
    arena->used = 0;
    arena->cap = 0;
    if (cap == 0) {
        return 0;
    }
    arena->base = malloc(cap);
    if (arena->base == NULL) {
        return 0;
    }
    arena->cap = cap;
    return 1;
}

void arena_dispose(Arena *arena) {
    free(arena->base);
    arena->base = NULL;
    arena->used = 0;
    arena->cap = 0;
}

void *arena_alloc(Arena *arena, size_t size, size_t align) {
    size_t base;
    size_t next;
    if (align == 0 || (align & (align - 1)) != 0 || size > arena->cap) {
        return NULL;
    }
    base = (arena->used + (align - 1)) & ~(align - 1);
    if (base > arena->cap || size > arena->cap - base) {
        return NULL;
    }
    next = base + size;
    arena->used = next;
    return arena->base + base;
}

Big big_zero(void) {
    Big value;
    value.limbs = NULL;
    value.nlimbs = 0;
    value.negative = 0;
    return value;
}

static Big big_publish(uint32_t *limbs, uint32_t nlimbs, int negative) {
    Big value;
    while (nlimbs > 0 && limbs[nlimbs - 1] == 0) {
        nlimbs--;
    }
    value.limbs = nlimbs == 0 ? NULL : limbs;
    value.nlimbs = nlimbs;
    value.negative = nlimbs == 0 ? 0 : negative;
    return value;
}

uint32_t big_bits(const Big *value) {
    uint32_t top;
    uint32_t bits;
    if (value->nlimbs == 0) {
        return 0;
    }
    top = value->limbs[value->nlimbs - 1];
    bits = 0;
    while (top > 0) {
        top >>= 1;
        bits++;
    }
    return (value->nlimbs - 1) * 32u + bits;
}

uint32_t big_limbs(const Big *value) {
    return value->nlimbs;
}

static int fits_bits(const Big *value) {
    return big_bits(value) <= ORANGE_MAX_BITS;
}

static uint32_t *alloc_limbs(Arena *arena, uint32_t count) {
    if (count == 0) {
        return NULL;
    }
    return arena_alloc(arena, (size_t)count * sizeof(uint32_t), sizeof(uint32_t));
}

int big_from_u64(Arena *arena, uint64_t value, Big *out) {
    uint32_t *limbs;
    uint32_t count;
    if (value == 0) {
        *out = big_zero();
        return 1;
    }
    count = value > 0xffffffffu ? 2u : 1u;
    limbs = alloc_limbs(arena, count);
    if (limbs == NULL) {
        return 0;
    }
    limbs[0] = (uint32_t)value;
    if (count == 2) {
        limbs[1] = (uint32_t)(value >> 32);
    }
    *out = big_publish(limbs, count, 0);
    return 1;
}

static int mul_small(Arena *arena, const Big *value, uint32_t factor, Big *out) {
    uint32_t *limbs;
    uint32_t index;
    uint64_t carry;
    if (value->nlimbs == 0 || factor == 0) {
        *out = big_zero();
        return 1;
    }
    limbs = alloc_limbs(arena, value->nlimbs + 1);
    if (limbs == NULL) {
        return 0;
    }
    carry = 0;
    for (index = 0; index < value->nlimbs; index++) {
        uint64_t product = (uint64_t)value->limbs[index] * factor + carry;
        limbs[index] = (uint32_t)product;
        carry = product >> 32;
    }
    limbs[value->nlimbs] = (uint32_t)carry;
    *out = big_publish(limbs, value->nlimbs + 1, value->negative);
    return fits_bits(out);
}

static int add_small(Arena *arena, const Big *value, uint32_t addend, Big *out) {
    uint32_t *limbs;
    uint32_t count;
    uint32_t index;
    uint64_t carry;
    if (addend == 0) {
        *out = *value;
        return 1;
    }
    if (value->nlimbs == 0) {
        return big_from_u64(arena, addend, out);
    }
    count = value->nlimbs + 1;
    limbs = alloc_limbs(arena, count);
    if (limbs == NULL) {
        return 0;
    }
    carry = addend;
    for (index = 0; index < value->nlimbs; index++) {
        uint64_t sum = (uint64_t)value->limbs[index] + carry;
        limbs[index] = (uint32_t)sum;
        carry = sum >> 32;
    }
    limbs[value->nlimbs] = (uint32_t)carry;
    *out = big_publish(limbs, count, value->negative);
    return fits_bits(out);
}

static int digit_value(char character, int base) {
    int value;
    if (character >= '0' && character <= '9') {
        value = character - '0';
    } else if (character >= 'a' && character <= 'f') {
        value = character - 'a' + 10;
    } else if (character >= 'A' && character <= 'F') {
        value = character - 'A' + 10;
    } else {
        return -1;
    }
    if (value >= base) {
        return -1;
    }
    return value;
}

int big_from_digits(Arena *arena, const char *text, size_t length, int negative, Big *out) {
    size_t index = 0;
    int base = 10;
    Big value = big_zero();
    int saw_digit = 0;
    if (length >= 2 && text[0] == '0' && (text[1] == 'x' || text[1] == 'X')) {
        base = 16;
        index = 2;
    } else if (length >= 2 && text[0] == '0' && (text[1] == 'b' || text[1] == 'B')) {
        base = 2;
        index = 2;
    }
    for (; index < length; index++) {
        int digit;
        Big scaled;
        if (text[index] == '_') {
            continue;
        }
        digit = digit_value(text[index], base);
        if (digit < 0) {
            return 0;
        }
        if (!mul_small(arena, &value, (uint32_t)base, &scaled)) {
            return 0;
        }
        if (!add_small(arena, &scaled, (uint32_t)digit, &value)) {
            return 0;
        }
        saw_digit = 1;
    }
    if (!saw_digit || !fits_bits(&value)) {
        return 0;
    }
    if (negative && value.nlimbs != 0) {
        value.negative = 1;
    }
    *out = value;
    return 1;
}

static int cmp_mag(const Big *left, const Big *right) {
    uint32_t index;
    if (left->nlimbs != right->nlimbs) {
        return left->nlimbs < right->nlimbs ? -1 : 1;
    }
    index = left->nlimbs;
    while (index > 0) {
        index--;
        if (left->limbs[index] != right->limbs[index]) {
            return left->limbs[index] < right->limbs[index] ? -1 : 1;
        }
    }
    return 0;
}

static int add_mag(Arena *arena, const Big *left, const Big *right, int negative, Big *out) {
    uint32_t count = left->nlimbs > right->nlimbs ? left->nlimbs : right->nlimbs;
    uint32_t *limbs = alloc_limbs(arena, count + 1);
    uint32_t index;
    uint64_t carry = 0;
    if (limbs == NULL) {
        return 0;
    }
    for (index = 0; index < count; index++) {
        uint64_t sum = carry;
        if (index < left->nlimbs) {
            sum += left->limbs[index];
        }
        if (index < right->nlimbs) {
            sum += right->limbs[index];
        }
        limbs[index] = (uint32_t)sum;
        carry = sum >> 32;
    }
    limbs[count] = (uint32_t)carry;
    *out = big_publish(limbs, count + 1, negative);
    return fits_bits(out);
}

static int sub_mag(Arena *arena, const Big *left, const Big *right, int negative, Big *out) {
    uint32_t *limbs;
    uint32_t index;
    int32_t borrow = 0;
    if (left->nlimbs == 0) {
        *out = big_zero();
        return 1;
    }
    limbs = alloc_limbs(arena, left->nlimbs);
    if (limbs == NULL) {
        return 0;
    }
    for (index = 0; index < left->nlimbs; index++) {
        uint32_t right_limb = index < right->nlimbs ? right->limbs[index] : 0;
        uint32_t left_limb = left->limbs[index];
        uint32_t result = left_limb - right_limb - (uint32_t)borrow;
        borrow = (left_limb < right_limb) || (borrow && left_limb == right_limb);
        limbs[index] = result;
    }
    *out = big_publish(limbs, left->nlimbs, negative);
    return 1;
}

int big_add(Arena *arena, const Big *left, const Big *right, Big *out) {
    int order;
    if (left->negative == right->negative) {
        return add_mag(arena, left, right, left->negative, out);
    }
    order = cmp_mag(left, right);
    if (order == 0) {
        *out = big_zero();
        return 1;
    }
    if (order > 0) {
        return sub_mag(arena, left, right, left->negative, out);
    }
    return sub_mag(arena, right, left, right->negative, out);
}

int big_neg(const Big *value, Big *out) {
    *out = *value;
    if (out->nlimbs != 0) {
        out->negative = !out->negative;
    }
    return 1;
}

int big_cmp(const Big *left, const Big *right) {
    int magnitude;
    if (left->negative != right->negative) {
        if (left->nlimbs == 0 && right->nlimbs == 0) {
            return 0;
        }
        return left->negative ? -1 : 1;
    }
    magnitude = cmp_mag(left, right);
    return left->negative ? -magnitude : magnitude;
}

int big_sub(Arena *arena, const Big *left, const Big *right, Big *out) {
    Big negated;
    big_neg(right, &negated);
    return big_add(arena, left, &negated, out);
}

int big_mul(Arena *arena, const Big *left, const Big *right, Big *out) {
    uint32_t count;
    uint32_t *limbs;
    uint32_t i;
    int negative;
    if (left->nlimbs == 0 || right->nlimbs == 0) {
        *out = big_zero();
        return 1;
    }
    if (big_bits(left) + big_bits(right) > ORANGE_MAX_BITS + 1) {
        return 0;
    }
    count = left->nlimbs + right->nlimbs;
    limbs = alloc_limbs(arena, count);
    if (limbs == NULL) {
        return 0;
    }
    memset(limbs, 0, (size_t)count * sizeof(uint32_t));
    for (i = 0; i < left->nlimbs; i++) {
        uint64_t carry = 0;
        uint32_t j;
        for (j = 0; j < right->nlimbs; j++) {
            uint64_t prod = (uint64_t)left->limbs[i] * right->limbs[j];
            uint64_t low = (uint64_t)limbs[i + j] + (prod & 0xffffffffu) + (carry & 0xffffffffu);
            uint64_t high = (prod >> 32) + (carry >> 32) + (low >> 32);
            limbs[i + j] = (uint32_t)low;
            carry = high;
        }
        if (carry > 0xffffffffu) {
            return 0;
        }
        limbs[i + right->nlimbs] = (uint32_t)carry;
    }
    negative = left->negative != right->negative;
    *out = big_publish(limbs, count, negative);
    return fits_bits(out);
}

int big_mod_pow2(const Big *value, uint32_t width, uint64_t *out) {
    uint64_t low = 0;
    uint64_t mask;
    if (width == 0 || width > 64) {
        return 0;
    }
    if (value->nlimbs > 0) {
        low = value->limbs[0];
    }
    if (width > 32 && value->nlimbs > 1) {
        low |= (uint64_t)value->limbs[1] << 32;
    }
    mask = width == 64 ? UINT64_MAX : (UINT64_C(1) << width) - 1;
    low &= mask;
    if (value->negative && low != 0) {
        low = (uint64_t)(0 - low) & mask;
    }
    *out = low;
    return 1;
}

int big_format(const Big *value, char *buffer, size_t capacity) {
    uint32_t *scratch;
    uint32_t count;
    uint32_t groups_cap;
    uint32_t *groups;
    uint32_t ngroups = 0;
    uint32_t active;
    size_t used = 0;
    int started = 0;
    uint32_t index;
    if (capacity == 0) {
        return 0;
    }
    if (value->nlimbs == 0) {
        if (capacity < 2) {
            return 0;
        }
        buffer[0] = '0';
        buffer[1] = '\0';
        return 1;
    }
    count = value->nlimbs;
    scratch = malloc((size_t)count * sizeof(uint32_t));
    groups_cap = count * 2u + 2u;
    groups = malloc((size_t)groups_cap * sizeof(uint32_t));
    if (scratch == NULL || groups == NULL) {
        free(scratch);
        free(groups);
        return 0;
    }
    memcpy(scratch, value->limbs, (size_t)count * sizeof(uint32_t));
    active = count;
    while (active > 0) {
        uint64_t remainder = 0;
        uint32_t limb;
        if (ngroups >= groups_cap) {
            free(scratch);
            free(groups);
            return 0;
        }
        limb = active;
        while (limb > 0) {
            uint64_t current;
            limb--;
            current = (remainder << 32) | scratch[limb];
            scratch[limb] = (uint32_t)(current / 1000000000u);
            remainder = current % 1000000000u;
        }
        groups[ngroups++] = (uint32_t)remainder;
        while (active > 0 && scratch[active - 1] == 0) {
            active--;
        }
    }
    if (value->negative) {
        if (used + 1 >= capacity) {
            free(scratch);
            free(groups);
            return 0;
        }
        buffer[used++] = '-';
    }
    for (index = ngroups; index > 0; index--) {
        char chunk[16];
        int length;
        uint32_t group = groups[index - 1];
        if (!started) {
            length = snprintf(chunk, sizeof chunk, "%u", group);
            started = 1;
        } else {
            length = snprintf(chunk, sizeof chunk, "%09u", group);
        }
        if (length < 0 || used + (size_t)length >= capacity) {
            free(scratch);
            free(groups);
            return 0;
        }
        memcpy(buffer + used, chunk, (size_t)length);
        used += (size_t)length;
    }
    buffer[used] = '\0';
    free(scratch);
    free(groups);
    return 1;
}

int bigint_self_test(void) {
    Arena arena;
    Big two;
    Big value;
    Big hex;
    Big prime;
    Big wide;
    Big one;
    Big crossed;
    Big word;
    Big squared;
    uint64_t residue = 0;
    char text[8192];
    int step;
    if (!arena_init(&arena, 8u * 1024u * 1024u)) {
        return 0;
    }
    if (!big_from_u64(&arena, 2, &two)) {
        arena_dispose(&arena);
        return 0;
    }
    value = two;
    for (step = 0; step < 7; step++) {
        Big squared_step;
        if (!big_mul(&arena, &value, &value, &squared_step)) {
            arena_dispose(&arena);
            return 0;
        }
        value = squared_step;
    }
    if (!big_format(&value, text, sizeof text) ||
        strcmp(text, "340282366920938463463374607431768211456") != 0) {
        arena_dispose(&arena);
        return 0;
    }
    if (!big_from_digits(&arena, "0x80000000000000000000000000000000", 34, 0, &hex) ||
        !big_mul(&arena, &value, &hex, &prime) || !big_from_u64(&arena, 19, &wide) ||
        !big_sub(&arena, &prime, &wide, &prime) || !big_format(&prime, text, sizeof text) ||
        strcmp(text, "57896044618658097711785492504343953926634992332820282019728792003956564819949") !=
            0) {
        arena_dispose(&arena);
        return 0;
    }
    if (!big_from_u64(&arena, 1, &one) ||
        !big_from_digits(&arena, "18446744073709551617", 20, 0, &wide) ||
        !big_sub(&arena, &one, &wide, &crossed) || !big_format(&crossed, text, sizeof text) ||
        strcmp(text, "-18446744073709551616") != 0) {
        arena_dispose(&arena);
        return 0;
    }
    if (!big_from_u64(&arena, 1, &one)) {
        arena_dispose(&arena);
        return 0;
    }
    one.negative = 1;
    if (!big_mod_pow2(&one, 8, &residue) || residue != 0xff) {
        arena_dispose(&arena);
        return 0;
    }
    if (!big_from_u64(&arena, UINT64_MAX, &word) || !big_mul(&arena, &word, &word, &squared) ||
        !big_format(&squared, text, sizeof text) ||
        strcmp(text, "340282366920938463426481119284349108225") != 0) {
        arena_dispose(&arena);
        return 0;
    }
    arena_dispose(&arena);
    return 1;
}
