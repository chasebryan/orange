#ifndef ORANGE_TESTS_H
#define ORANGE_TESTS_H

#include "orange.h"

#include <stddef.h>
#include <stdint.h>

/* What is wrong with a test title, if anything. */
enum {
    TITLE_OK = 0,
    TITLE_EMPTY = 1,
    TITLE_BACKSLASH = 2,
    TITLE_CHAR = 3,
    TITLE_LONG = 4
};

/* `text` is the title without its quotes. A fault's byte offset is stored
   in `*offset`; a non-ASCII character's code point is stored in `*codepoint`. */
int orange_title_fault(const char *text, size_t length, size_t *offset, uint32_t *codepoint);

/* Compares every element. Arrays of words or `Bool` cost one step per 64
   elements; a number or a residue costs what comparing it alone costs; a
   tuple adds nothing. A difference does not stop the walk. Returns 0 when
   the step budget is exhausted or the values are not a pair of one type. */
int orange_values_equal(Compiler *c, const Value *left, const Value *right, int *equal);

#ifdef ORANGEC_TEST
/* Element visits of `==` / `!=`. `orange_eq_audit` requires n visits for a
   difference at either end of an n-element array or tuple of words,
   residues, or big integers. */
int orange_eq_audit(void);
#endif

/* The first `[i]` or `.k` where two arrays or tuples differ, followed into
   nested rows. Returns 0 for scalars and for values that match. */
int orange_first_difference(const Value *left, const Value *right, char *place, size_t cap);

#endif
