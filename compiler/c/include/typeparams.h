#ifndef ORANGE_TYPEPARAMS_H
#define ORANGE_TYPEPARAMS_H

/* Declarations for the type-parameter translation unit.
   The implementation is compiler/c/src/typeparams.c, compiled on its own. */

#include "orange.h"

TokenKind peek_kind(const Compiler *c);
Token peek_token(const Compiler *c);
void advance_token(Compiler *c);
int same_span(const Compiler *c, uint32_t a0, uint32_t a1, uint32_t b0, uint32_t b1);
int span_is(const Compiler *c, uint32_t start, uint32_t end, const char *word);
void found_token_label(TokenKind kind, char *buf, size_t cap);
void span_copy(char *dest, size_t cap, const char *text, uint32_t start, uint32_t end);
void copy_text(char *dest, size_t cap, const char *src);
void add_diag(Compiler *c, const char *code, uint32_t start, uint32_t end, const char *message,
              const char *label, const char *note, int phase);
void diag_add_secondary(Compiler *c, uint32_t start, uint32_t end, const char *label);
void resource_diag(Compiler *c, const char *code, uint32_t start, uint32_t end, const char *message);
int ensure_cap(void **ptr, size_t *cap, size_t need, size_t elem, size_t max);
int parse_type(Compiler *c, DeclaredType *type, int allow_array);
int push_site(Compiler *c, const DeclaredType *type, const char *role, uint32_t *site_out);
int canonical_array_length(const char *text, uint32_t start, uint32_t end, uint32_t *value);
Sz eval_size(Compiler *c, uint32_t index);
void report_size_fault(Compiler *c, Sz fault);
int find_leaf(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, TypeKind *type,
              uint32_t *length, uint32_t *leaf, int *silent);
void array_parts(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, int *have_len,
                 uint32_t *len, int *have_elem, TypeKind *elem);
void spell_type(const Compiler *owner, char *buffer, size_t cap, TypeKind type, uint32_t length, uint16_t mod,
                uint32_t tup0, uint16_t tup_n);
int same_tuple(const Compiler *left_owner, uint32_t left0, uint16_t left_n, const Compiler *right_owner,
               uint32_t right0, uint16_t right_n);
void copy_func_name(const Compiler *c, const Func *func, char *name, size_t cap);
int size_length(Compiler *c, uint32_t index, int report, uint32_t *length);
void reject_type(Compiler *c, TypeKind type, int ok, uint32_t start, uint32_t end);
void bind_modulus(Compiler *c, TypeSite *site);
int decode_size_bound(Compiler *c, uint32_t start, uint32_t end, int64_t *out, int *too_big);
int builtin_type_name(const Compiler *c, uint32_t start, uint32_t end);
void resolve_site(Compiler *c, TypeSite *site, int from_decl, uint32_t earlier_limit);

int tp_func_has_types(const Func *func);
int tp_starts_call(const Compiler *c);
int tp_brackets_open_type(const Compiler *c);
void tp_impl_span(const Compiler *c, uint32_t *start, uint32_t *end);
int tp_parse_params(Compiler *c, Func *func);
void tp_admit(Compiler *c, Func *func);
int tp_bind_use(Compiler *c, TypeSite *site);
void tp_materialize(Compiler *c, TypeSite *site, int report);
int tp_lookup_call(Compiler *c, Expr *expr, Compiler *target, uint32_t callee, int report, uint32_t caller_func,
                   uint32_t locals, uint32_t *inst_id);
void tp_format_label(const Compiler *c, uint32_t inst, char *buf, size_t cap);
void tp_write_values(const Compiler *c, const Func *func, const Instance *inst, char *buf, size_t cap);
void tp_name_diags(Compiler *c, const Func *func, uint32_t inst, uint32_t from);
void tp_refresh_convs(Compiler *c, uint32_t func_index);
void tp_note_expr(Compiler *c, Expr *expr);
void tp_apply_stamps(Compiler *c);
int tp_names_type_param(const Compiler *c, uint32_t func_index, uint32_t start, uint32_t end);
void tp_type_param_note(const Compiler *c, uint32_t start, uint32_t end, char *buf, size_t cap);

#endif
