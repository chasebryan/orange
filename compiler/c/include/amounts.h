#ifndef ORANGE_AMOUNTS_H
#define ORANGE_AMOUNTS_H

#include <stdint.h>

/* Shifts or rotates an n-bit word by an amount computed from data. `bits`
   is 8, 16, 32, or 64. `magnitude_known` is false when an `Int` amount's
   magnitude has more than 64 bits; the magnitude is never truncated to a
   machine word before that test. `low` is the amount modulo 2^64. A shift
   by `bits` or more is 0, and a negative amount shifts the other way. A
   rotation turns by `low` modulo `bits`. No C shift uses a count outside
   0 through 63. */
uint64_t orange_word_shift(int op, int bits, uint64_t value, int negative, int magnitude_known,
                           uint64_t magnitude, uint64_t low);

#endif
