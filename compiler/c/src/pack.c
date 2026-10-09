#include "pack.h"

#include <stdlib.h>
#include <string.h>

uint32_t pack_stride_of(TypeKind type, uint32_t mod_bits) {
    switch (type) {
    case TY_W8: return 1;
    case TY_W16: return 2;
    case TY_W32: return 4;
    case TY_W64: return 8;
    case TY_MOD:
        if (mod_bits == 0 || mod_bits > 64) {
            return 0;
        }
        return mod_bits <= 32 ? 4u : 8u;
    default:
        return 0;
    }
}

Pack *pack_new(TypeKind type, uint32_t length, uint32_t stride, uint16_t mod_index) {
    Pack *pack;
    size_t bytes;
    if (length == 0 || stride == 0 || stride > 8 || length > SIZE_MAX / stride) {
        return NULL;
    }
    bytes = (size_t)length * (size_t)stride;
    pack = calloc(1, sizeof *pack);
    if (pack == NULL) {
        return NULL;
    }
    pack->data = calloc(1, bytes);
    if (pack->data == NULL) {
        free(pack);
        return NULL;
    }
    pack->refs = 1;
    pack->length = length;
    pack->stride = stride;
    pack->type = type;
    pack->mod_index = mod_index;
    return pack;
}

int pack_retain(Pack *pack) {
    if (pack == NULL || pack->refs == UINT32_MAX) {
        return 0;
    }
    pack->refs++;
    return 1;
}

void pack_release(Pack *pack) {
    if (pack == NULL) {
        return;
    }
    if (pack->refs > 1) {
        pack->refs--;
        return;
    }
    free(pack->data);
    pack->data = NULL;
    free(pack);
}

int pack_make_unique(Pack **slot) {
    Pack *old;
    Pack *copy;
    size_t bytes;
    if (slot == NULL || *slot == NULL) {
        return 0;
    }
    old = *slot;
    if (old->refs == 1) {
        return 1;
    }
    copy = pack_new(old->type, old->length, old->stride, old->mod_index);
    if (copy == NULL) {
        return 0;
    }
    bytes = (size_t)old->length * (size_t)old->stride;
    memcpy(copy->data, old->data, bytes);
    pack_release(old);
    *slot = copy;
    return 1;
}

uint64_t pack_get(const Pack *pack, uint32_t index) {
    const unsigned char *slot;
    uint64_t word = 0;
    uint32_t byte;
    if (pack == NULL || pack->data == NULL || index >= pack->length) {
        return 0;
    }
    slot = pack->data + (size_t)index * pack->stride;
    for (byte = 0; byte < pack->stride; byte++) {
        word |= (uint64_t)slot[byte] << (8u * byte);
    }
    return word;
}

void pack_set(Pack *pack, uint32_t index, uint64_t word) {
    unsigned char *slot;
    uint32_t byte;
    if (pack == NULL || pack->data == NULL || pack->refs != 1 || index >= pack->length) {
        return;
    }
    slot = pack->data + (size_t)index * pack->stride;
    for (byte = 0; byte < pack->stride; byte++) {
        slot[byte] = (unsigned char)(word >> (8u * byte));
    }
}

void pack_fill(Pack *pack, uint64_t word) {
    uint32_t index;
    if (pack == NULL || pack->data == NULL || pack->refs != 1) {
        return;
    }
    if (word == 0) {
        memset(pack->data, 0, (size_t)pack->length * (size_t)pack->stride);
        return;
    }
    for (index = 0; index < pack->length; index++) {
        pack_set(pack, index, word);
    }
}

void pack_write_from(Pack *dst, uint32_t at, const Pack *src) {
    size_t bytes;
    if (dst == NULL || src == NULL || dst->data == NULL || src->data == NULL || dst->refs != 1) {
        return;
    }
    if (dst->stride != src->stride || at > dst->length || src->length > dst->length - at) {
        return;
    }
    bytes = (size_t)src->length * (size_t)src->stride;
    memcpy(dst->data + (size_t)at * dst->stride, src->data, bytes);
}

Pack *pack_slice(const Pack *src, uint32_t start, uint32_t count) {
    Pack *out;
    if (src == NULL || src->data == NULL || count == 0 || start > src->length || count > src->length - start) {
        return NULL;
    }
    out = pack_new(src->type, count, src->stride, src->mod_index);
    if (out == NULL) {
        return NULL;
    }
    memcpy(out->data, src->data + (size_t)start * src->stride, (size_t)count * src->stride);
    return out;
}

Pack *pack_concat(const Pack *left, const Pack *right) {
    Pack *out;
    size_t left_bytes;
    size_t right_bytes;
    if (left == NULL || right == NULL || left->data == NULL || right->data == NULL) {
        return NULL;
    }
    if (left->stride != right->stride || left->type != right->type || left->length > UINT32_MAX - right->length) {
        return NULL;
    }
    out = pack_new(left->type, left->length + right->length, left->stride, left->mod_index);
    if (out == NULL) {
        return NULL;
    }
    left_bytes = (size_t)left->length * left->stride;
    right_bytes = (size_t)right->length * right->stride;
    memcpy(out->data, left->data, left_bytes);
    memcpy(out->data + left_bytes, right->data, right_bytes);
    return out;
}

int pack_self_test(void) {
    Pack *pack;
    Pack *shared;
    Pack *slice;
    uint64_t word;
    pack = pack_new(TY_W8, 4, 1, 0);
    if (pack == NULL) {
        return 0;
    }
    pack_fill(pack, 0);
    pack_set(pack, 1, 0xab);
    if (!pack_retain(pack)) {
        pack_release(pack);
        return 0;
    }
    shared = pack;
    if (!pack_make_unique(&pack) || pack == shared || pack_get(shared, 1) != 0xab || pack_get(pack, 1) != 0xab) {
        pack_release(pack);
        pack_release(shared);
        return 0;
    }
    pack_set(pack, 1, 0x10);
    if (pack_get(shared, 1) != 0xab || pack_get(pack, 1) != 0x10 || pack->refs != 1) {
        pack_release(pack);
        pack_release(shared);
        return 0;
    }
    slice = pack_slice(shared, 1, 2);
    word = slice == NULL ? 0 : pack_get(slice, 0);
    pack_release(slice);
    pack_release(pack);
    pack_release(shared);
    return word == 0xab;
}
