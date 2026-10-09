#include "tests.h"

#include "bigint.h"
#include "pack.h"

#include <stdio.h>
#include <string.h>

enum { MAX_TEST_TITLE_BYTES = 128 };

static int word_width(TypeKind type) {
    switch (type) {
    case TY_W8: return 8;
    case TY_W16: return 16;
    case TY_W32: return 32;
    case TY_W64: return 64;
    default: return 0;
    }
}

static int bulk_element(TypeKind type) {
    return type == TY_BOOL || word_width(type) != 0;
}

static int charge_steps(Compiler *c, uint64_t cost) {
    uint64_t limit = c->step_limit == 0 ? MAX_STEPS : c->step_limit;
    if (c->failed) {
        return 0;
    }
    if (cost > limit || c->steps > limit - cost) {
        c->failed = 1;
        c->step_hit = 1;
        return 0;
    }
    c->steps += cost;
    return 1;
}

static uint32_t modulus_digit_count(const Big *modulus) {
    uint32_t bits = big_bits(modulus);
    return bits == 0 ? 1u : (bits + 31u) / 32u;
}

static const Big *modulus_of(const Compiler *c, uint16_t index) {
    if (c->moduli == NULL || index == 0 || index >= c->nmoduli) {
        return NULL;
    }
    return &c->moduli[index];
}

static int is_array(const Value *value) {
    return !value->is_tuple && (value->length > 0 || value->pack != NULL);
}

static int element_word(const Value *array, uint32_t index, uint64_t *word) {
    if (array->pack != NULL) {
        if (index >= array->pack->length) {
            return 0;
        }
        *word = pack_get(array->pack, index);
        return 1;
    }
    if (array->elems == NULL || index >= array->length) {
        return 0;
    }
    if (array->elems[index].type == TY_BOOL || word_width(array->elems[index].type) != 0) {
        *word = array->elems[index].word;
        return 1;
    }
    if (array->elems[index].type == TY_MOD || array->elems[index].type == TY_INT) {
        const Big *big = &array->elems[index].big;
        if (big->negative || big->nlimbs > 2) {
            return 0;
        }
        *word = 0;
        if (big->nlimbs > 0) {
            *word = big->limbs[0];
        }
        if (big->nlimbs > 1) {
            *word |= (uint64_t)big->limbs[1] << 32;
        }
        return 1;
    }
    return 0;
}

static int same_value(const Value *left, const Value *right);

static int same_array_elements(const Value *left, const Value *right) {
    uint32_t index;
    if (left->type != right->type || left->length != right->length) {
        return 0;
    }
    if (bulk_element(left->type) || left->pack != NULL || right->pack != NULL) {
        for (index = 0; index < left->length; index++) {
            uint64_t left_word = 0;
            uint64_t right_word = 0;
            if (!element_word(left, index, &left_word) || !element_word(right, index, &right_word) ||
                left_word != right_word) {
                return 0;
            }
        }
        return 1;
    }
    if (left->elems == NULL || right->elems == NULL) {
        return left->length == 0;
    }
    for (index = 0; index < left->length; index++) {
        if (!same_value(&left->elems[index], &right->elems[index])) {
            return 0;
        }
    }
    return 1;
}

static int same_value(const Value *left, const Value *right) {
    uint32_t index;
    if (left->is_tuple || right->is_tuple) {
        if (!left->is_tuple || !right->is_tuple || left->length != right->length || left->elems == NULL ||
            right->elems == NULL) {
            return 0;
        }
        for (index = 0; index < left->length; index++) {
            if (!same_value(&left->elems[index], &right->elems[index])) {
                return 0;
            }
        }
        return 1;
    }
    if (is_array(left) || is_array(right)) {
        if (!is_array(left) || !is_array(right)) {
            return 0;
        }
        return same_array_elements(left, right);
    }
    if (left->type != right->type) {
        return 0;
    }
    if (left->type == TY_INT || left->type == TY_MOD) {
        return big_cmp(&left->big, &right->big) == 0;
    }
    return left->word == right->word;
}

static uint32_t decode_utf8(const char *text, size_t length, size_t index, size_t *next) {
    unsigned char lead;
    size_t width;
    uint32_t codepoint;
    if (index >= length) {
        *next = index;
        return 0;
    }
    lead = (unsigned char)text[index];
    if (lead < 0x80u) {
        *next = index + 1;
        return lead;
    }
    if ((lead & 0xe0u) == 0xc0u) {
        width = 2;
    } else if ((lead & 0xf0u) == 0xe0u) {
        width = 3;
    } else if ((lead & 0xf8u) == 0xf0u) {
        width = 4;
    } else {
        *next = index + 1;
        return lead;
    }
    if (index + width > length) {
        *next = index + 1;
        return lead;
    }
    if (width == 2) {
        codepoint = ((uint32_t)(lead & 0x1fu) << 6) | ((unsigned char)text[index + 1] & 0x3fu);
    } else if (width == 3) {
        codepoint = ((uint32_t)(lead & 0x0fu) << 12) | (((unsigned char)text[index + 1] & 0x3fu) << 6) |
                    ((unsigned char)text[index + 2] & 0x3fu);
    } else {
        codepoint = ((uint32_t)(lead & 0x07u) << 18) | (((unsigned char)text[index + 1] & 0x3fu) << 12) |
                    (((unsigned char)text[index + 2] & 0x3fu) << 6) | ((unsigned char)text[index + 3] & 0x3fu);
    }
    *next = index + width;
    return codepoint;
}

int orange_title_fault(const char *text, size_t length, size_t *offset, uint32_t *codepoint) {
    size_t index = 0;
    if (length == 0) {
        *offset = 0;
        *codepoint = 0;
        return TITLE_EMPTY;
    }
    while (index < length) {
        size_t next = index;
        uint32_t decoded = decode_utf8(text, length, index, &next);
        if (decoded == '\\') {
            *offset = index;
            *codepoint = decoded;
            return TITLE_BACKSLASH;
        }
        if (decoded < 0x20u || decoded > 0x7eu) {
            *offset = index;
            *codepoint = decoded;
            return TITLE_CHAR;
        }
        if (next <= index) {
            break;
        }
        index = next;
    }
    if (length > MAX_TEST_TITLE_BYTES) {
        *offset = length;
        *codepoint = 0;
        return TITLE_LONG;
    }
    return TITLE_OK;
}

int orange_values_equal(Compiler *c, const Value *left, const Value *right, int *equal) {
    uint32_t index;
    if (c->failed) {
        return 0;
    }
    if (left->is_tuple || right->is_tuple) {
        int all = 1;
        if (!left->is_tuple || !right->is_tuple || left->length != right->length || left->elems == NULL ||
            right->elems == NULL) {
            c->failed = 1;
            return 0;
        }
        for (index = 0; index < left->length; index++) {
            int part = 0;
            if (!orange_values_equal(c, &left->elems[index], &right->elems[index], &part)) {
                return 0;
            }
            all = all && part;
        }
        *equal = all;
        return 1;
    }
    if (is_array(left) || is_array(right)) {
        int all = 1;
        if (!is_array(left) || !is_array(right) || left->type != right->type || left->length != right->length) {
            c->failed = 1;
            return 0;
        }
        if (bulk_element(left->type)) {
            uint64_t cost = left->length <= 64u ? 1u : ((uint64_t)left->length + 63u) / 64u;
            if (!charge_steps(c, cost)) {
                return 0;
            }
            for (index = 0; index < left->length; index++) {
                uint64_t left_word = 0;
                uint64_t right_word = 0;
                if (!element_word(left, index, &left_word) || !element_word(right, index, &right_word)) {
                    c->failed = 1;
                    return 0;
                }
                all = all && (left_word == right_word);
            }
            *equal = all;
            return 1;
        }
        if (left->pack != NULL || right->pack != NULL) {
            uint16_t mod_index = left->mod_index != 0 ? left->mod_index : (left->pack != NULL ? left->pack->mod_index : 0);
            const Big *modulus = modulus_of(c, mod_index);
            uint64_t cost;
            if (left->type != TY_MOD || modulus == NULL) {
                c->failed = 1;
                return 0;
            }
            cost = 1u + modulus_digit_count(modulus);
            for (index = 0; index < left->length; index++) {
                uint64_t left_word = 0;
                uint64_t right_word = 0;
                if (!charge_steps(c, cost) || !element_word(left, index, &left_word) ||
                    !element_word(right, index, &right_word)) {
                    if (!c->failed) {
                        c->failed = 1;
                    }
                    return 0;
                }
                all = all && (left_word == right_word);
            }
            *equal = all;
            return 1;
        }
        if (left->elems == NULL || right->elems == NULL) {
            c->failed = 1;
            return 0;
        }
        for (index = 0; index < left->length; index++) {
            int part = 0;
            if (!orange_values_equal(c, &left->elems[index], &right->elems[index], &part)) {
                return 0;
            }
            all = all && part;
        }
        *equal = all;
        return 1;
    }
    if (left->type != right->type) {
        c->failed = 1;
        return 0;
    }
    if (left->type == TY_INT) {
        uint32_t left_limbs = big_limbs(&left->big);
        uint32_t right_limbs = big_limbs(&right->big);
        if (!charge_steps(c, 1u + (left_limbs > right_limbs ? left_limbs : right_limbs))) {
            return 0;
        }
        *equal = big_cmp(&left->big, &right->big) == 0;
        return 1;
    }
    if (left->type == TY_MOD) {
        const Big *modulus;
        if (left->mod_index != right->mod_index) {
            c->failed = 1;
            return 0;
        }
        modulus = modulus_of(c, left->mod_index);
        if (modulus == NULL || !charge_steps(c, 1u + modulus_digit_count(modulus))) {
            if (!c->failed) {
                c->failed = 1;
            }
            return 0;
        }
        *equal = big_cmp(&left->big, &right->big) == 0;
        return 1;
    }
    if (!charge_steps(c, 1)) {
        return 0;
    }
    *equal = left->word == right->word;
    return 1;
}

int orange_first_difference(const Value *left, const Value *right, char *place, size_t cap) {
    int tuple = left->is_tuple && right->is_tuple;
    int array = !tuple && is_array(left) && is_array(right);
    uint32_t index;
    if (cap == 0) {
        return 0;
    }
    place[0] = '\0';
    if ((!tuple && !array) || left->length != right->length) {
        return 0;
    }
    for (index = 0; index < left->length; index++) {
        int differs;
        char inner[160];
        int wrote;
        inner[0] = '\0';
        if (tuple) {
            differs = !same_value(&left->elems[index], &right->elems[index]);
            if (differs) {
                orange_first_difference(&left->elems[index], &right->elems[index], inner, sizeof inner);
            }
        } else if (left->elems != NULL && right->elems != NULL && left->pack == NULL && right->pack == NULL) {
            differs = !same_value(&left->elems[index], &right->elems[index]);
            if (differs) {
                orange_first_difference(&left->elems[index], &right->elems[index], inner, sizeof inner);
            }
        } else {
            uint64_t left_word = 0;
            uint64_t right_word = 0;
            differs = !element_word(left, index, &left_word) || !element_word(right, index, &right_word) ||
                      left_word != right_word;
        }
        if (!differs) {
            continue;
        }
        if (tuple) {
            wrote = snprintf(place, cap, ".%u%s", index, inner);
        } else {
            wrote = snprintf(place, cap, "[%u]%s", index, inner);
        }
        return wrote > 0 && (size_t)wrote < cap;
    }
    return 0;
}
