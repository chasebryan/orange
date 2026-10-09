#include "amounts.h"

#include "orange.h"

#include <stdint.h>

static uint64_t width_mask(int bits) {
    if (bits >= 64) {
        return UINT64_MAX;
    }
    if (bits <= 0) {
        return 0;
    }
    return (UINT64_C(1) << (unsigned)bits) - 1u;
}

/* `amount` is 0 through 63. A count of 64 or more is not a C shift. */
static uint64_t shift_left_bits(uint64_t value, uint32_t amount) {
    if (amount == 0 || amount >= 64u) {
        return amount == 0 ? value : 0;
    }
    return value << amount;
}

static uint64_t shift_right_bits(uint64_t value, uint32_t amount) {
    if (amount == 0 || amount >= 64u) {
        return amount == 0 ? value : 0;
    }
    return value >> amount;
}

/* `amount` is 0 through bits-1, and bits is 8, 16, 32, or 64. */
static uint64_t rotate_left_bits(uint64_t value, int bits, uint64_t mask, uint32_t amount) {
    uint32_t down;
    if (amount == 0) {
        return value & mask;
    }
    down = (uint32_t)bits - amount;
    return (shift_left_bits(value, amount) | shift_right_bits(value, down)) & mask;
}

uint64_t orange_word_shift(int op, int bits, uint64_t value, int negative, int magnitude_known,
                           uint64_t magnitude, uint64_t low) {
    uint64_t mask;
    uint32_t turn;
    int within;
    int leftward;
    if (bits != 8 && bits != 16 && bits != 32 && bits != 64) {
        return 0;
    }
    mask = width_mask(bits);
    value &= mask;
    turn = (uint32_t)(low & (uint64_t)(bits - 1));
    within = magnitude_known && magnitude < (uint64_t)bits;
    if (op == TK_ROL) {
        return rotate_left_bits(value, bits, mask, turn);
    }
    if (op == TK_ROR) {
        uint32_t by = turn == 0 ? 0u : (uint32_t)bits - turn;
        return rotate_left_bits(value, bits, mask, by);
    }
    if (!within) {
        return 0;
    }
    leftward = (op == TK_LSHIFT) != (negative != 0);
    if (leftward) {
        return shift_left_bits(value, (uint32_t)magnitude) & mask;
    }
    return shift_right_bits(value, (uint32_t)magnitude);
}
