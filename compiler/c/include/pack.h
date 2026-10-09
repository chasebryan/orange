#ifndef ORANGE_PACK_H
#define ORANGE_PACK_H

#include "orange.h"

/* A compact array of words or of residues whose modulus fits in 64 bits.
   Copies share the block. An update writes in place when refs is 1 and
   otherwise copies the block first. */
struct Pack {
    uint32_t refs;
    uint32_t length;
    uint32_t stride;
    TypeKind type;
    uint16_t mod_index;
    unsigned char *data;
};

/* Stride in bytes for a word, or for a residue of `mod_bits` when that
   modulus fits in 64 bits. Returns 0 when the value stays a `Value` array. */
uint32_t pack_stride_of(TypeKind type, uint32_t mod_bits);

Pack *pack_new(TypeKind type, uint32_t length, uint32_t stride, uint16_t mod_index);
int pack_retain(Pack *pack);
void pack_release(Pack *pack);
/* After this, `*pack` has refs 1 and may be written. The previous block is
   kept for any other owner. Returns 0 when the copy cannot be allocated. */
int pack_make_unique(Pack **pack);
uint64_t pack_get(const Pack *pack, uint32_t index);
void pack_set(Pack *pack, uint32_t index, uint64_t word);
void pack_fill(Pack *pack, uint64_t word);
void pack_write_from(Pack *dst, uint32_t at, const Pack *src);
Pack *pack_slice(const Pack *src, uint32_t start, uint32_t count);
Pack *pack_concat(const Pack *left, const Pack *right);
int pack_self_test(void);

#endif
