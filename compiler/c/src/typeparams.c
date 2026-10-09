#include "typeparams.h"

#include <stdio.h>
#include <string.h>

static const char TYPE_PARAMETER_NOTE[] =
    "a type parameter is written `K in {F, L}` and names each type its function is checked for, as in "
    "`spec square[K in {F, L}](x: K) -> K { x * x }`";
static const char SIZE_PARAMETER_NOTE[] =
    "a sized function is written `spec f[n in 1..5](x: Word[8]^n) -> Type { ... }` and checked once for each n from 1 "
    "up to, but not including, 5";
static const char SIZE_RANGE_NOTE[] =
    "a size parameter `n in a..b` takes each value from a up to, but not including, b, with a < b <= 65536, and a "
    "function has at most 256 instances";
static const char TYPE_PARAMETER_CALL_NOTE[] =
    "a call gives each type parameter one of the types it lists, in brackets, as in `pow[F](x, e)`";
static const char TYPE_ARGUMENT_NOTE[] =
    "a type in a call's brackets is `Int`, `Bool`, `Word[n]`, an array of one of them such as `Word[8]^4`, the name of "
    "a `type` declaration, or a type parameter of the calling function";
static const char TYPE_CALL_NOTE[] =
    "a function with type parameters is called with one entry in brackets for each of its sizes and types, in order, "
    "as in `pow[F](x, e)`, or without brackets when its arguments choose one instance";
static const char NO_BRACKET_NOTE[] =
    "a call that writes no brackets calls the one instance of its function whose parameters have its arguments' "
    "types, and among several, the one whose result has the type its place expects; any other call writes its types "
    "in brackets, as in `pow[F](x, e)`";
static const char MIXED_NOTE[] =
    "a call that writes no brackets calls the one instance of its function whose parameters have its arguments' "
    "types, and among several, the one whose result has the type its place expects; any other call writes its sizes "
    "and types in brackets, as in `sum[Q, 3](xs)`";
static const char INSTANCE_COUNT_NOTE[] =
    "a function has one instance for each combination of its sizes' values and its type parameters' types, at most "
    "256 in all";

typedef struct TpTy {
    int known;
    int len_only;
    TypeKind kind;
    uint32_t length;
    uint16_t mod;
    uint32_t tup0;
    uint16_t tup_n;
    uint32_t alen;
    const Compiler *owner;
} TpTy;

int tp_func_has_types(const Func *func) {
    uint8_t slot;
    if (func == NULL) {
        return 0;
    }
    for (slot = 0; slot < func->nsizes; slot++) {
        if (func->sz_kind[slot]) {
            return 1;
        }
    }
    return 0;
}

static int tp_has_sizes(const Func *func) {
    uint8_t slot;
    for (slot = 0; slot < func->nsizes; slot++) {
        if (!func->sz_kind[slot]) {
            return 1;
        }
    }
    return 0;
}

static int tp_name_slot(const Compiler *c, const Func *func, uint32_t start, uint32_t end, uint8_t *slot) {
    uint8_t index;
    for (index = 0; index < func->nsizes; index++) {
        if (func->sz_kind[index] && same_span(c, func->sz_name0[index], func->sz_name1[index], start, end)) {
            if (slot != NULL) {
                *slot = index;
            }
            return 1;
        }
    }
    return 0;
}

int tp_names_type_param(const Compiler *c, uint32_t func_index, uint32_t start, uint32_t end) {
    if (func_index >= c->nfuncs) {
        return 0;
    }
    return tp_name_slot(c, &c->funcs[func_index], start, end, NULL);
}

void tp_type_param_note(const Compiler *c, uint32_t start, uint32_t end, char *buf, size_t cap) {
    char name[64];
    span_copy(name, sizeof name, c->text, start, end);
    snprintf(buf, cap,
             "`%s` is a type parameter: it names a type, not a value, so it is written where a type is, as in "
             "`let x: %s = 0;`",
             name, name);
}

static void collapse_span(const Compiler *c, uint32_t start, uint32_t end, char *buf, size_t cap) {
    size_t used = 0;
    int pending = 0;
    int any = 0;
    uint32_t index;
    if (cap == 0) {
        return;
    }
    for (index = start; index < end && used + 1 < cap; index++) {
        unsigned char ch = (unsigned char)c->text[index];
        if (ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r') {
            pending = any;
            continue;
        }
        if (pending && used + 1 < cap) {
            buf[used++] = ' ';
        }
        pending = 0;
        any = 1;
        buf[used++] = (char)ch;
    }
    buf[used] = '\0';
}

static void listed_spelling(const Compiler *c, const Func *func, uint8_t slot, uint32_t index, char *buf, size_t cap) {
    const TypeSite *site;
    buf[0] = '\0';
    if (index >= func->sz_nlist[slot]) {
        return;
    }
    site = &c->sites[c->tp_sites[func->sz_list0[slot] + index]];
    collapse_span(c, site->start, site->end, buf, cap);
}

static void add_note2(Compiler *c, const char *text) {
    Diag *diag;
    if (c->ndiags == 0 || text == NULL) {
        return;
    }
    diag = &c->diags[c->ndiags - 1];
    if (strcmp(diag->code, "ORC0105") == 0 || strcmp(diag->code, "ORC0208") == 0) {
        return;
    }
    if (diag->note[0] == '\0') {
        copy_text(diag->note, sizeof diag->note, text);
    } else if (!diag->has_note2) {
        copy_text(diag->note2, sizeof diag->note2, text);
        diag->has_note2 = 1;
    }
}

static int push_listed(Compiler *c, uint32_t site) {
    if (!ensure_cap((void **)&c->tp_sites, &c->tp_site_cap, c->ntp_sites + 1, sizeof(uint32_t), MAX_TYPE_SITES)) {
        resource_diag(c, "ORC0106", 0, 0, "parser could not retain a listed type");
        return 0;
    }
    c->tp_sites[c->ntp_sites++] = site;
    return 1;
}

int tp_starts_call(const Compiler *c) {
    size_t pos;
    int depth = 0;
    int modulus = 0;
    if (c->at >= c->ntokens || c->tokens[c->at].kind != TK_LBRACKET) {
        return 0;
    }
    pos = c->at + 1;
    for (;;) {
        TokenKind kind;
        if (pos >= c->ntokens) {
            return 0;
        }
        kind = c->tokens[pos].kind;
        if (kind == TK_INT || kind == TK_IDENT || kind == TK_PLUS || kind == TK_MINUS || kind == TK_STAR ||
            kind == TK_SLASH || kind == TK_PERCENT || kind == TK_CARET || kind == TK_COMMA) {
        } else if (kind == TK_LPAREN) {
            depth++;
        } else if (kind == TK_RPAREN && depth > 0) {
            depth--;
        } else if (kind == TK_LSHIFT && modulus) {
        } else if (kind == TK_LBRACKET && !modulus && pos > 0 && c->tokens[pos - 1].kind == TK_IDENT &&
                   span_is(c, c->tokens[pos - 1].start, c->tokens[pos - 1].end, "Mod")) {
            modulus = 1;
        } else if (kind == TK_RBRACKET && modulus) {
            modulus = 0;
        } else if (kind == TK_LBRACKET && !modulus && pos > 0 && c->tokens[pos - 1].kind == TK_IDENT &&
                   pos + 2 < c->ntokens && c->tokens[pos + 1].kind == TK_INT &&
                   c->tokens[pos + 2].kind == TK_RBRACKET) {
            pos += 2;
        } else if (kind == TK_RBRACKET && depth == 0) {
            return pos + 1 < c->ntokens && c->tokens[pos + 1].kind == TK_LPAREN;
        } else {
            return 0;
        }
        pos++;
    }
}

int tp_brackets_open_type(const Compiler *c) {
    size_t pos = c->at + 1;
    int depth = 0;
    while (pos < c->ntokens) {
        TokenKind kind = c->tokens[pos].kind;
        if (kind == TK_LBRACE && depth == 0) {
            return 1;
        }
        if (kind == TK_LBRACKET || kind == TK_LPAREN || kind == TK_LBRACE) {
            depth++;
        } else if ((kind == TK_RBRACKET || kind == TK_RPAREN || kind == TK_RBRACE) && depth > 0) {
            depth--;
        } else if (kind == TK_RBRACKET && depth == 0) {
            return 0;
        }
        pos++;
    }
    return 0;
}

void tp_impl_span(const Compiler *c, uint32_t *start, uint32_t *end) {
    size_t pos = c->at + 1;
    int depth = 0;
    *start = c->tokens[c->at].start;
    *end = c->tokens[c->at].end;
    if (pos < c->ntokens && c->tokens[pos].kind == TK_IDENT) {
        *start = c->tokens[pos].start;
    }
    while (pos < c->ntokens) {
        TokenKind kind = c->tokens[pos].kind;
        if (kind == TK_LBRACE && depth == 0) {
            depth = 1;
        } else if (kind == TK_RBRACE && depth == 1) {
            *end = c->tokens[pos].end;
            return;
        } else if (kind == TK_RBRACKET && depth == 0) {
            return;
        }
        pos++;
    }
}

static int parse_type_param(Compiler *c, Func *func, Token name) {
    Token open;
    Token close;
    uint32_t list_at;
    uint16_t count = 0;
    int any_type;
    uint8_t slot;
    open = peek_token(c);
    advance_token(c);
    list_at = c->ntp_sites;
    (void)open;
    for (;;) {
        DeclaredType declared;
        uint32_t site = UINT32_MAX;
        if (peek_kind(c) == TK_RBRACE) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                     count == 0 ? "expected a listed type after `{`" : "expected a listed type after `,`",
                     "found RIGHT_BRACE", TYPE_PARAMETER_NOTE, 1);
            return 0;
        }
        if (!parse_type(c, &declared, 1)) {
            return 0;
        }
        if (!push_site(c, &declared, "listed type", &site) || !push_listed(c, site)) {
            return 0;
        }
        count++;
        if (peek_kind(c) == TK_COMMA) {
            advance_token(c);
            continue;
        }
        if (peek_kind(c) == TK_RBRACE) {
            break;
        }
        {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `,` or `}` after the listed type",
                     label, TYPE_PARAMETER_NOTE, 1);
        }
        return 0;
    }
    close = peek_token(c);
    advance_token(c);
    if (func->nsizes >= MAX_SIZES) {
        add_diag(c, "ORC0101", name.start, close.end, "a function has at most 4 size and type parameters",
                 "one parameter in brackets too many", TYPE_PARAMETER_NOTE, 1);
        return 0;
    }
    any_type = 1;
    (void)any_type;
    slot = func->nsizes;
    func->sz_kind[slot] = 1;
    func->sz_name0[slot] = name.start;
    func->sz_name1[slot] = name.end;
    func->sz_span0[slot] = name.start;
    func->sz_span1[slot] = close.end;
    func->sz_list0[slot] = list_at;
    func->sz_nlist[slot] = count;
    func->nsizes++;
    return 1;
}

static int earlier_type(const Func *func) {
    uint8_t slot;
    for (slot = 0; slot < func->nsizes; slot++) {
        if (func->sz_kind[slot]) {
            return 1;
        }
    }
    return 0;
}

static int parse_size_param(Compiler *c, Func *func, Token name) {
    Token first;
    Token second;
    uint8_t slot;
    if (peek_kind(c) != TK_INT) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected the size's first bound",
                 "expected an integer bound", SIZE_PARAMETER_NOTE, 1);
        return 0;
    }
    first = peek_token(c);
    advance_token(c);
    if (peek_kind(c) != TK_DOTDOT) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `..` between the size's bounds",
                 "expected `..`", SIZE_PARAMETER_NOTE, 1);
        return 0;
    }
    advance_token(c);
    if (peek_kind(c) != TK_INT) {
        char label[64];
        found_token_label(peek_kind(c), label, sizeof label);
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected the size's second bound", label,
                 SIZE_PARAMETER_NOTE, 1);
        return 0;
    }
    second = peek_token(c);
    advance_token(c);
    if (func->nsizes >= MAX_SIZES) {
        const char *kinds = earlier_type(func) ? "size and type parameters" : "size parameters";
        char message[160];
        snprintf(message, sizeof message, "a function has at most 4 %s", kinds);
        add_diag(c, "ORC0101", name.start, second.end, message, "one size parameter too many", SIZE_PARAMETER_NOTE, 1);
        return 0;
    }
    slot = func->nsizes;
    func->sz_kind[slot] = 0;
    func->sz_name0[slot] = name.start;
    func->sz_name1[slot] = name.end;
    func->sz_span0[slot] = name.start;
    func->sz_span1[slot] = second.end;
    func->sz_a0[slot] = first.start;
    func->sz_a1[slot] = first.end;
    func->sz_b0[slot] = second.start;
    func->sz_b1[slot] = second.end;
    func->nsizes++;
    return 1;
}

int tp_parse_params(Compiler *c, Func *func) {
    int last_type = 0;
    advance_token(c);
    if (peek_kind(c) == TK_RBRACKET) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected a size parameter", "expected a name",
                 SIZE_PARAMETER_NOTE, 1);
        return 0;
    }
    for (;;) {
        Token name;
        if (peek_kind(c) != TK_IDENT) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected a size parameter",
                     "expected a name", SIZE_PARAMETER_NOTE, 1);
            return 0;
        }
        name = peek_token(c);
        advance_token(c);
        if (!span_is(c, peek_token(c).start, peek_token(c).end, "in")) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `in` after the size's name", label,
                     SIZE_PARAMETER_NOTE, 1);
            return 0;
        }
        advance_token(c);
        if (peek_kind(c) == TK_LBRACE) {
            last_type = 1;
            if (!parse_type_param(c, func, name)) {
                return 0;
            }
        } else {
            last_type = 0;
            if (!parse_size_param(c, func, name)) {
                return 0;
            }
        }
        if (peek_kind(c) == TK_COMMA) {
            advance_token(c);
            continue;
        }
        if (peek_kind(c) == TK_RBRACKET) {
            advance_token(c);
            return 1;
        }
        {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                     last_type ? "expected `,` or `]` after the type parameter"
                               : "expected `,` or `]` after the size parameter",
                     label, last_type ? TYPE_PARAMETER_NOTE : SIZE_PARAMETER_NOTE, 1);
        }
        return 0;
    }
}

static int site_length(const TypeSite *site) {
    return site->rank <= 0 ? 0 : (int)site->length;
}

static int sites_equal(const Compiler *c, const TypeSite *left, const TypeSite *right) {
    if (!left->ok || !right->ok || left->kind != right->kind) {
        return 0;
    }
    if (left->kind == TY_TUPLE || left->is_tuple || right->kind == TY_TUPLE) {
        return same_tuple(c, left->tup0, left->tup_n, c, right->tup0, right->tup_n);
    }
    if (site_length(left) != site_length(right)) {
        return 0;
    }
    if (left->kind == TY_MOD && left->mod_index != right->mod_index) {
        return 0;
    }
    return 1;
}

static void resolve_listed(Compiler *c, TypeSite *site) {
    uint16_t elem;
    if (site->is_tuple) {
        for (elem = 0; elem < site->elem_n; elem++) {
            if (site->elem0 + elem < c->nsites) {
                bind_modulus(c, &c->sites[site->elem0 + elem]);
            }
        }
    }
    bind_modulus(c, site);
    c->admit_listed = 1;
    resolve_site(c, site, 0, c->ntypes);
    c->admit_listed = 0;
    if (!site->ok && !site->reported) {
        reject_type(c, site->kind, 0, site->start, site->end);
        site->reported = 1;
    }
}

static void report_listed_twice(Compiler *c, const Func *func, uint8_t slot, const TypeSite *first,
                                const TypeSite *again) {
    char name[64];
    char spelled[160];
    char message[384];
    span_copy(name, sizeof name, c->text, func->sz_name0[slot], func->sz_name1[slot]);
    spell_type(c, spelled, sizeof spelled, again->kind, again->rank <= 0 ? 0u : again->length, again->mod_index,
               again->tup0, again->tup_n);
    snprintf(message, sizeof message, "`%s` lists the type `%s` twice", name, spelled);
    add_diag(c, "ORC0241", again->start, again->end, message, "this is the same type as an earlier one",
             "a type parameter lists each type once, so that each instance has a type of its own", 2);
    diag_add_secondary(c, first->start, first->end, "first listed here");
}

static int check_type_slot(Compiler *c, Func *func, uint8_t slot) {
    uint16_t index;
    int valid = 1;
    char name[64];
    span_copy(name, sizeof name, c->text, func->sz_name0[slot], func->sz_name1[slot]);
    if (builtin_type_name(c, func->sz_name0[slot], func->sz_name1[slot])) {
        char message[160];
        snprintf(message, sizeof message, "`%s` is a built-in type", name);
        add_diag(c, "ORC0233", func->sz_name0[slot], func->sz_name1[slot], message,
                 "a type parameter cannot name a built-in type",
                 "a type parameter names a type of its own, so its name is not a built-in type's or a `type` "
                 "declaration's",
                 2);
        valid = 0;
    } else {
        uint32_t decl;
        for (decl = 0; decl < c->ntypes; decl++) {
            if (!c->types[decl].installed) {
                continue;
            }
            if (same_span(c, c->types[decl].name_start, c->types[decl].name_end, func->sz_name0[slot],
                          func->sz_name1[slot])) {
                char message[160];
                snprintf(message, sizeof message, "duplicate type name `%s`", name);
                add_diag(c, "ORC0233", func->sz_name0[slot], func->sz_name1[slot], message,
                         "this type parameter repeats a declared type's name",
                         "a type parameter names a type of its own, so its name is not a built-in type's or a `type` "
                         "declaration's",
                         2);
                diag_add_secondary(c, c->types[decl].name_start, c->types[decl].name_end,
                                   "the `type` declaration is here");
                valid = 0;
                break;
            }
        }
    }
    for (index = 0; index < func->sz_nlist[slot]; index++) {
        TypeSite *site = &c->sites[c->tp_sites[func->sz_list0[slot] + index]];
        uint16_t earlier;
        resolve_listed(c, site);
        if (!site->ok) {
            valid = 0;
        }
        for (earlier = 0; earlier < index; earlier++) {
            TypeSite *before = &c->sites[c->tp_sites[func->sz_list0[slot] + earlier]];
            if (sites_equal(c, before, site)) {
                valid = 0;
                report_listed_twice(c, func, slot, before, site);
                break;
            }
        }
    }
    func->sz_lo[slot] = 0;
    func->sz_hi[slot] = func->sz_nlist[slot];
    return valid;
}

static void report_bracket_dup(Compiler *c, uint32_t start, uint32_t end, uint32_t earlier_start, uint32_t earlier_end) {
    char name[64];
    char message[160];
    span_copy(name, sizeof name, c->text, start, end);
    snprintf(message, sizeof message, "duplicate parameter `%s`", name);
    add_diag(c, "ORC0218", start, end, message, "this name is already a parameter in brackets",
             "each size and type parameter in a function's brackets has a name of its own", 2);
    diag_add_secondary(c, earlier_start, earlier_end, "first parameter in brackets is here");
}

static void report_size_dup(Compiler *c, uint32_t start, uint32_t end, uint32_t earlier_start, uint32_t earlier_end,
                            const char *label) {
    char name[64];
    char message[160];
    span_copy(name, sizeof name, c->text, start, end);
    snprintf(message, sizeof message, "duplicate parameter `%s`", name);
    add_diag(c, "ORC0218", start, end, message, "this name is already a size parameter",
             "size parameters and parameters share one namespace, and each name is unique", 2);
    diag_add_secondary(c, earlier_start, earlier_end, label);
}

void tp_admit(Compiler *c, Func *func) {
    uint8_t slot;
    uint64_t instances = 1;
    int valid = 1;
    for (slot = 0; slot < func->nsizes; slot++) {
        uint8_t earlier;
        if (func->sz_kind[slot]) {
            if (!check_type_slot(c, func, slot)) {
                valid = 0;
            }
            if (func->sz_nlist[slot] != 0 && instances > UINT64_MAX / func->sz_nlist[slot]) {
                instances = UINT64_MAX;
            } else {
                instances *= func->sz_nlist[slot];
            }
        } else {
            int lo_big = 0;
            int hi_big = 0;
            int lo_ok = decode_size_bound(c, func->sz_a0[slot], func->sz_a1[slot], &func->sz_lo[slot], &lo_big);
            int hi_ok = decode_size_bound(c, func->sz_b0[slot], func->sz_b1[slot], &func->sz_hi[slot], &hi_big);
            if (!lo_ok || !hi_ok) {
                valid = 0;
            } else if (lo_big) {
                valid = 0;
                add_diag(c, "ORC0238", func->sz_a0[slot], func->sz_a1[slot], "a size's bound must be at most 65536",
                         "bound too large", SIZE_RANGE_NOTE, 2);
            } else if (hi_big) {
                valid = 0;
                add_diag(c, "ORC0238", func->sz_b0[slot], func->sz_b1[slot], "a size's bound must be at most 65536",
                         "bound too large", SIZE_RANGE_NOTE, 2);
            } else if (func->sz_lo[slot] >= func->sz_hi[slot]) {
                char message[160];
                valid = 0;
                snprintf(message, sizeof message, "the size range %lld..%lld is empty", (long long)func->sz_lo[slot],
                         (long long)func->sz_hi[slot]);
                add_diag(c, "ORC0238", func->sz_b0[slot], func->sz_b1[slot], message, "a size takes at least one value",
                         SIZE_RANGE_NOTE, 2);
            } else {
                uint64_t width = (uint64_t)(func->sz_hi[slot] - func->sz_lo[slot]);
                if (width != 0 && instances > UINT64_MAX / width) {
                    instances = UINT64_MAX;
                } else {
                    instances *= width;
                }
            }
        }
        for (earlier = 0; earlier < slot; earlier++) {
            if (!same_span(c, func->sz_name0[earlier], func->sz_name1[earlier], func->sz_name0[slot],
                           func->sz_name1[slot])) {
                continue;
            }
            valid = 0;
            if (func->sz_kind[slot] || func->sz_kind[earlier]) {
                report_bracket_dup(c, func->sz_name0[slot], func->sz_name1[slot], func->sz_name0[earlier],
                                   func->sz_name1[earlier]);
            } else {
                report_size_dup(c, func->sz_name0[slot], func->sz_name1[slot], func->sz_name0[earlier],
                                func->sz_name1[earlier], "first size parameter is here");
            }
            break;
        }
    }
    for (slot = 0; slot < func->nparams; slot++) {
        Param *param = &c->params[func->param0 + slot];
        uint8_t size_slot;
        for (size_slot = 0; size_slot < func->nsizes; size_slot++) {
            if (func->sz_kind[size_slot]) {
                continue;
            }
            if (!same_span(c, func->sz_name0[size_slot], func->sz_name1[size_slot], param->name_start, param->name_end)) {
                continue;
            }
            valid = 0;
            report_size_dup(c, param->name_start, param->name_end, func->sz_name0[size_slot], func->sz_name1[size_slot],
                            "the size parameter is here");
            break;
        }
    }
    if (valid && instances > MAX_INSTANCES) {
        char name[64];
        char message[384];
        valid = 0;
        copy_func_name(c, func, name, sizeof name);
        if (instances == UINT64_MAX) {
            snprintf(message, sizeof message, "`%s` has more than %u instances", name, MAX_INSTANCES);
        } else {
            snprintf(message, sizeof message, "`%s` has %llu instances, but a function has at most %u", name,
                     (unsigned long long)instances, MAX_INSTANCES);
        }
        add_diag(c, "ORC0238", func->sz_span0[0], func->sz_span1[func->nsizes - 1], message, "too many instances",
                 INSTANCE_COUNT_NOTE, 2);
    }
    func->sizes_ok = valid;
}

int tp_bind_use(Compiler *c, TypeSite *site) {
    const Func *func;
    uint8_t slot = 0;
    if (site->owner_func >= c->nfuncs || !site->named) {
        return 0;
    }
    if (site->role != NULL && strcmp(site->role, "listed type") == 0) {
        return 0;
    }
    func = &c->funcs[site->owner_func];
    if (!tp_name_slot(c, func, site->ident_start, site->ident_end, &slot)) {
        return 0;
    }
    site->param_slot = slot;
    site->param_axis = site->wrote_axis ? 1u : 0u;
    site->param_len = site->length;
    site->ok = 1;
    return 1;
}

static void fill_from_site(const Compiler *owner, const TypeSite *site, TpTy *out) {
    memset(out, 0, sizeof *out);
    out->owner = owner;
    if (!site->ok) {
        return;
    }
    out->known = 1;
    out->kind = site->kind == TY_TUPLE || site->is_tuple ? TY_TUPLE : site->kind;
    out->length = site->rank <= 0 ? 0u : site->length;
    out->mod = site->mod_index;
    out->tup0 = site->tup0;
    out->tup_n = site->tup_n;
}

void tp_materialize(Compiler *c, TypeSite *site, int report) {
    Func *func;
    TypeSite *listed;
    int64_t index;
    uint32_t axis;
    int axis_ok = 1;
    if (site->param_slot == 0xFF || site->owner_func >= c->nfuncs) {
        return;
    }
    func = &c->funcs[site->owner_func];
    if (site->param_slot >= func->nsizes || !func->sz_kind[site->param_slot]) {
        return;
    }
    index = c->ncur > site->param_slot ? c->cur_sz[site->param_slot] : 0;
    if (index < 0 || (uint32_t)index >= func->sz_nlist[site->param_slot]) {
        site->ok = 0;
        return;
    }
    listed = &c->sites[c->tp_sites[func->sz_list0[site->param_slot] + (uint32_t)index]];
    axis = site->param_len;
    if (site->param_axis && site->has_size_expr) {
        if (!size_length(c, site->length_expr, report, &axis)) {
            axis_ok = 0;
        }
    }
    site->kind = listed->kind;
    site->mod_index = listed->mod_index;
    site->tup0 = listed->tup0;
    site->tup_n = listed->tup_n;
    site->is_tuple = listed->kind == TY_TUPLE || listed->is_tuple;
    site->inner_len = listed->inner_len;
    site->ok = listed->ok && axis_ok;
    if (site->param_axis && axis_ok) {
        site->rank = listed->rank + 1;
        site->length = axis;
        if (listed->rank >= 1) {
            site->inner_len = listed->length;
        }
    } else if (!site->param_axis) {
        site->rank = listed->rank;
        site->length = listed->rank <= 0 ? 0u : listed->length;
    } else {
        site->ok = 0;
        site->rank = 0;
        site->length = 0;
    }
    if (site->is_tuple) {
        site->kind = TY_TUPLE;
        site->rank = 0;
        site->length = 0;
    }
}

static int ty_equal(const TpTy *left, const TpTy *right) {
    if (!left->known || !right->known || left->kind != right->kind || left->length != right->length) {
        return 0;
    }
    if (left->kind == TY_MOD && left->mod != right->mod) {
        return 0;
    }
    if (left->kind == TY_TUPLE) {
        return same_tuple(left->owner, left->tup0, left->tup_n, right->owner, right->tup0, right->tup_n);
    }
    return 1;
}

static void ty_from_param(const Compiler *owner, const InstParam *param, TpTy *out) {
    memset(out, 0, sizeof *out);
    out->owner = owner;
    out->known = param->type_ok;
    out->kind = param->type;
    out->length = param->length;
    out->mod = param->mod_index;
    out->tup0 = param->tup0;
    out->tup_n = param->tup_n;
}

static void arg_type(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals, TpTy *out);

static void arg_array(Compiler *c, const Expr *expr, uint32_t func_index, uint32_t locals, TpTy *out) {
    uint16_t index;
    int all = 1;
    int got = 0;
    TpTy first;
    memset(&first, 0, sizeof first);
    memset(out, 0, sizeof *out);
    out->owner = c;
    for (index = 0; index < expr->argc; index++) {
        TpTy elem;
        arg_type(c, c->args[expr->arg0 + index], func_index, locals, &elem);
        if (!elem.known) {
            all = 0;
            break;
        }
        if (!got) {
            first = elem;
            got = 1;
        } else if (!ty_equal(&first, &elem)) {
            all = 0;
            break;
        }
    }
    if (all && got && first.length == 0 && first.kind != TY_TUPLE) {
        out->known = 1;
        out->kind = first.kind;
        out->length = expr->argc;
        out->mod = first.mod;
        out->owner = first.owner;
        return;
    }
    out->len_only = 1;
    out->alen = expr->argc;
}

static void arg_type(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals, TpTy *out) {
    const Expr *expr;
    memset(out, 0, sizeof *out);
    out->owner = c;
    if (index >= c->nexprs) {
        return;
    }
    expr = &c->exprs[index];
    if (expr->kind == EX_GROUP || expr->kind == EX_UPDATE || expr->kind == EX_SLICE_UP) {
        arg_type(c, expr->left, func_index, locals, out);
        return;
    }
    if (expr->kind == EX_ARRAY) {
        arg_array(c, expr, func_index, locals, out);
        return;
    }
    if (expr->kind == EX_FILL) {
        int have_len = 0;
        int have_elem = 0;
        uint32_t len = 0;
        TypeKind elem = TY_NONE;
        TpTy value;
        array_parts(c, index, func_index, locals, &have_len, &len, &have_elem, &elem);
        arg_type(c, expr->left, func_index, locals, &value);
        if (have_len && value.known && value.length == 0 && value.kind != TY_TUPLE) {
            out->known = 1;
            out->kind = value.kind;
            out->length = len;
            out->mod = value.mod;
            out->owner = value.owner;
            return;
        }
        if (have_len) {
            out->len_only = 1;
            out->alen = len;
        }
        return;
    }
    {
        TypeKind type = TY_NONE;
        uint32_t length = 0;
        uint32_t leaf = 0;
        int silent = 0;
        int state = find_leaf(c, index, func_index, locals, &type, &length, &leaf, &silent);
        if (state == 1) {
            out->known = 1;
            out->kind = type;
            out->length = length;
            out->mod = c->leaf_mod;
            out->tup0 = c->leaf_tup0;
            out->tup_n = c->leaf_tup_n;
            out->owner = c->leaf_owner != NULL ? c->leaf_owner : c;
            return;
        }
        {
            int have_len = 0;
            int have_elem = 0;
            uint32_t len = 0;
            TypeKind elem = TY_NONE;
            array_parts(c, index, func_index, locals, &have_len, &len, &have_elem, &elem);
            if (have_elem && have_len) {
                out->known = 1;
                out->kind = elem;
                out->length = len;
                return;
            }
            if (have_len) {
                out->len_only = 1;
                out->alen = len;
            }
        }
    }
}

static int word_width(const Compiler *c, const Expr *expr, TypeKind *kind) {
    uint32_t width = 0;
    if (expr->right != UINT32_MAX || !canonical_array_length(c->text, expr->lit_start, expr->lit_end, &width)) {
        if (expr->right == UINT32_MAX && span_is(c, expr->lit_start, expr->lit_end, "8")) {
            width = 8;
        } else if (expr->right == UINT32_MAX && span_is(c, expr->lit_start, expr->lit_end, "16")) {
            width = 16;
        } else if (expr->right == UINT32_MAX && span_is(c, expr->lit_start, expr->lit_end, "32")) {
            width = 32;
        } else if (expr->right == UINT32_MAX && span_is(c, expr->lit_start, expr->lit_end, "64")) {
            width = 64;
        } else {
            return 0;
        }
    }
    if (width == 8) {
        *kind = TY_W8;
    } else if (width == 16) {
        *kind = TY_W16;
    } else if (width == 32) {
        *kind = TY_W32;
    } else if (width == 64) {
        *kind = TY_W64;
    } else {
        return 0;
    }
    return expr->right == UINT32_MAX;
}

static int type_argument(Compiler *c, uint32_t index, uint32_t caller_func, TpTy *out) {
    const Expr *expr;
    memset(out, 0, sizeof *out);
    out->owner = c;
    if (index >= c->nexprs) {
        return 1;
    }
    expr = &c->exprs[index];
    if (expr->kind == EX_GROUP) {
        return type_argument(c, expr->left, caller_func, out);
    }
    if (expr->kind == EX_NAME) {
        uint8_t slot = 0;
        if (span_is(c, expr->name_start, expr->name_end, "Int")) {
            out->known = 1;
            out->kind = TY_INT;
            return 0;
        }
        if (span_is(c, expr->name_start, expr->name_end, "Bool")) {
            out->known = 1;
            out->kind = TY_BOOL;
            return 0;
        }
        if (caller_func < c->nfuncs &&
            tp_name_slot(c, &c->funcs[caller_func], expr->name_start, expr->name_end, &slot)) {
            Func *func = &c->funcs[caller_func];
            int64_t at = c->ncur > slot ? c->cur_sz[slot] : 0;
            if (at >= 0 && (uint32_t)at < func->sz_nlist[slot]) {
                fill_from_site(c, &c->sites[c->tp_sites[func->sz_list0[slot] + (uint32_t)at]], out);
                return out->known ? 0 : 2;
            }
            return 2;
        }
        {
            uint32_t decl;
            for (decl = 0; decl < c->ntypes; decl++) {
                if (!c->types[decl].installed ||
                    !same_span(c, c->types[decl].name_start, c->types[decl].name_end, expr->name_start, expr->name_end)) {
                    continue;
                }
                fill_from_site(c, &c->sites[c->types[decl].site], out);
                return out->known ? 0 : 2;
            }
        }
        return 1;
    }
    if (expr->kind == EX_INDEX && expr->left < c->nexprs && c->exprs[expr->left].kind == EX_NAME) {
        const Expr *base = &c->exprs[expr->left];
        if (span_is(c, base->name_start, base->name_end, "Word")) {
            TypeKind kind = TY_NONE;
            if (!word_width(c, expr, &kind)) {
                return 1;
            }
            out->known = 1;
            out->kind = kind;
            return 0;
        }
        if (span_is(c, base->name_start, base->name_end, "Mod") && expr->right != UINT32_MAX) {
            TypeSite scratch;
            uint32_t before = c->ndiags;
            memset(&scratch, 0, sizeof scratch);
            scratch.param_slot = 0xFF;
            scratch.has_mod = 1;
            scratch.mod_expr = expr->right;
            bind_modulus(c, &scratch);
            if (scratch.mod_index == 0) {
                return c->ndiags > before ? 3 : 2;
            }
            out->known = 1;
            out->kind = TY_MOD;
            out->mod = scratch.mod_index;
            return 0;
        }
        return 1;
    }
    if (expr->kind == EX_BINARY && expr->op == TK_CARET) {
        const Expr *right = &c->exprs[expr->right];
        TpTy elem;
        uint32_t length = 0;
        int status;
        if (right->kind != EX_LIT ||
            !canonical_array_length(c->text, right->lit_start, right->lit_end, &length)) {
            return 1;
        }
        status = type_argument(c, expr->left, caller_func, &elem);
        if (status != 0) {
            return status;
        }
        if (!elem.known || elem.length != 0 || elem.kind == TY_TUPLE || elem.kind == TY_NONE) {
            return 1;
        }
        elem.length = length;
        *out = elem;
        return 0;
    }
    return 1;
}

static int param_fits(const Compiler *target, const InstParam *param, const TpTy *arg) {
    TpTy shape;
    if (!param->type_ok) {
        return 1;
    }
    if (arg->known) {
        ty_from_param(target, param, &shape);
        return ty_equal(&shape, arg);
    }
    if (arg->len_only && param->length > 0) {
        return param->length == arg->alen;
    }
    return 1;
}

static int result_gives(Compiler *c, Compiler *target, const Instance *inst, int set, TypeKind kind, uint32_t len,
                        uint16_t mod, uint32_t tup0, uint16_t tup_n) {
    if (!set || kind == TY_NONE) {
        return 1;
    }
    if (inst->result != kind || inst->result_len != len) {
        return 0;
    }
    if (kind == TY_MOD && inst->result_mod != mod) {
        return 0;
    }
    if (kind == TY_TUPLE && !same_tuple(target, inst->tup0, inst->tup_n, c, tup0, tup_n)) {
        return 0;
    }
    return 1;
}

static void domain_text(const Compiler *target, const Func *func, char *buf, size_t cap) {
    size_t used = 0;
    uint8_t slot;
    buf[0] = '\0';
    for (slot = 0; slot < func->nsizes && used + 8 < cap; slot++) {
        char piece[256];
        char name[64];
        int wrote;
        span_copy(name, sizeof name, target->text, func->sz_name0[slot], func->sz_name1[slot]);
        if (func->sz_kind[slot]) {
            char list[160];
            size_t list_used = 0;
            uint16_t index;
            list[0] = '\0';
            for (index = 0; index < func->sz_nlist[slot] && list_used + 2 < sizeof list; index++) {
                char spelling[64];
                int n;
                listed_spelling(target, func, slot, index, spelling, sizeof spelling);
                n = snprintf(list + list_used, sizeof list - list_used, "%s%s", index == 0 ? "" : ", ", spelling);
                if (n > 0 && (size_t)n < sizeof list - list_used) {
                    list_used += (size_t)n;
                }
            }
            snprintf(piece, sizeof piece, "`%s` in {%s}", name, list);
        } else {
            snprintf(piece, sizeof piece, "`%s` in %lld..%lld", name, (long long)func->sz_lo[slot],
                     (long long)func->sz_hi[slot]);
        }
        wrote = snprintf(buf + used, cap - used, "%s%s", slot == 0 ? "" : ", ", piece);
        if (wrote > 0 && (size_t)wrote < cap - used) {
            used += (size_t)wrote;
        }
    }
}

static const char *bracket_label(const Func *func) {
    int types = tp_func_has_types(func);
    int sizes = tp_has_sizes(func);
    if (!types) {
        return "write the sizes in brackets";
    }
    if (!sizes) {
        return "write the types in brackets";
    }
    return "write the sizes and types in brackets";
}

static void report_count(Compiler *c, Expr *expr, Compiler *target, const Func *func, const char *name) {
    char message[384];
    uint8_t types = 0;
    uint8_t sizes = 0;
    uint8_t slot;
    char taken[64];
    for (slot = 0; slot < func->nsizes; slot++) {
        if (func->sz_kind[slot]) {
            types++;
        } else {
            sizes++;
        }
    }
    if (sizes == 0 && types == 1) {
        copy_text(taken, sizeof taken, "1 type");
    } else if (sizes == 0) {
        snprintf(taken, sizeof taken, "%u types", types);
    } else {
        snprintf(taken, sizeof taken, "%u size%s and %u type%s", sizes, sizes == 1 ? "" : "s", types,
                 types == 1 ? "" : "s");
    }
    snprintf(message, sizeof message, "`%s` takes %s in brackets, but this call gives %u", name, taken, expr->nsize);
    add_diag(c, "ORC0239", expr->start, expr->end, message, "wrong number of entries in brackets", TYPE_CALL_NOTE, 2);
    (void)target;
}

static void report_unfitted(Compiler *c, Expr *expr, Compiler *target, const Func *func, const char *name,
                            const TpTy *args, int nfit, const uint32_t *fitting) {
    char domain[320];
    domain_text(target, func, domain, sizeof domain);
    if (nfit >= 2) {
        char message[512];
        char first[160];
        char second[160];
        tp_format_label(target, fitting[0], first, sizeof first);
        tp_format_label(target, fitting[1], second, sizeof second);
        snprintf(message, sizeof message, "this call fits more than one instance of `%s`, among them `%s` and `%s`",
                 name, first, second);
        add_diag(c, "ORC0239", expr->start, expr->end, message, bracket_label(func),
                 tp_has_sizes(func) && tp_func_has_types(func) ? MIXED_NOTE : NO_BRACKET_NOTE, 2);
        return;
    }
    {
        char message[384];
        char label[384];
        char shown[3][96];
        int nshown = 0;
        uint16_t param;
        snprintf(message, sizeof message, "no instance of `%s` takes arguments of these types", name);
        for (param = 0; param < func->nparams && param < expr->argc && nshown < 3; param++) {
            if (!args[param].known) {
                continue;
            }
            spell_type(args[param].owner, shown[nshown], sizeof shown[nshown], args[param].kind, args[param].length,
                       args[param].mod, args[param].tup0, args[param].tup_n);
            nshown++;
        }
        if (nshown == 0) {
            copy_text(label, sizeof label, "no instance fits these arguments");
        } else if (nshown == 1) {
            snprintf(label, sizeof label, "an argument of type `%s` is given", shown[0]);
        } else {
            snprintf(label, sizeof label, "arguments of types `%s`, `%s`%s%s are given", shown[0], shown[1],
                     nshown > 2 ? ", `" : "", nshown > 2 ? shown[2] : "");
            if (nshown > 2) {
                size_t len = strlen(label);
                if (len + 1 < sizeof label) {
                    label[len] = '`';
                    label[len + 1] = '\0';
                }
            }
        }
        {
            char note[512];
            snprintf(note, sizeof note, "`%s` is defined for %s", name, domain);
            add_diag(c, "ORC0241", expr->start, expr->end, message, label, note, 2);
            add_note2(c, tp_has_sizes(func) ? MIXED_NOTE : NO_BRACKET_NOTE);
        }
    }
}

int tp_lookup_call(Compiler *c, Expr *expr, Compiler *target, uint32_t callee, int report, uint32_t caller_func,
                   uint32_t locals, uint32_t *inst_id) {
    Func *func = &target->funcs[callee];
    char name[64];
    int saved_set;
    int saved_report;
    TypeKind saved_kind;
    uint32_t saved_len;
    uint16_t saved_mod;
    uint32_t saved_tup0;
    uint16_t saved_tup_n;
    copy_func_name(target, func, name, sizeof name);
    if (func->ninst == 0) {
        return 0;
    }
    if (expr->nsize != 0 && expr->nsize != func->nsizes) {
        if (report) {
            report_count(c, expr, target, func, name);
        }
        return 0;
    }
    if (expr->nsize > 0) {
        int64_t values[MAX_SIZES];
        uint8_t index;
        for (index = 0; index < expr->nsize; index++) {
            uint32_t entry = c->args[expr->size0 + index];
            if (func->sz_kind[index]) {
                TpTy ty;
                int status = type_argument(c, entry, caller_func, &ty);
                uint16_t listed;
                int found = 0;
                if (status == 3 || status == 2) {
                    return 0;
                }
                if (status != 0 || !ty.known) {
                    if (report) {
                        char message[384];
                        char param[64];
                        span_copy(param, sizeof param, target->text, func->sz_name0[index], func->sz_name1[index]);
                        snprintf(message, sizeof message, "`%s` takes a type for `%s` here", name, param);
                        add_diag(c, "ORC0241", c->exprs[entry].start, c->exprs[entry].end, message, "this is not a type",
                                 TYPE_ARGUMENT_NOTE, 2);
                    }
                    return 0;
                }
                for (listed = 0; listed < func->sz_nlist[index]; listed++) {
                    TpTy candidate;
                    fill_from_site(target, &target->sites[target->tp_sites[func->sz_list0[index] + listed]], &candidate);
                    if (ty_equal(&candidate, &ty)) {
                        values[index] = listed;
                        found = 1;
                        break;
                    }
                }
                if (!found) {
                    if (report) {
                        char message[384];
                        char param[64];
                        char list[192];
                        size_t used = 0;
                        uint16_t item;
                        span_copy(param, sizeof param, target->text, func->sz_name0[index], func->sz_name1[index]);
                        list[0] = '\0';
                        for (item = 0; item < func->sz_nlist[index] && used + 2 < sizeof list; item++) {
                            char spelling[64];
                            int n;
                            listed_spelling(target, func, index, item, spelling, sizeof spelling);
                            n = snprintf(list + used, sizeof list - used, "%s%s", item == 0 ? "" : ", ", spelling);
                            if (n > 0 && (size_t)n < sizeof list - used) {
                                used += (size_t)n;
                            }
                        }
                        snprintf(message, sizeof message, "`%s` is defined for `%s` in {%s}", name, param, list);
                        add_diag(c, "ORC0241", c->exprs[entry].start, c->exprs[entry].end, message,
                                 "this type is not listed", TYPE_PARAMETER_CALL_NOTE, 2);
                    }
                    return 0;
                }
            } else {
                Sz value = eval_size(c, entry);
                if (value.kind != 0) {
                    if (report) {
                        report_size_fault(c, value);
                    }
                    return 0;
                }
                if (value.value < func->sz_lo[index] || value.value >= func->sz_hi[index]) {
                    if (report) {
                        char message[384];
                        char size_name[64];
                        char label[64];
                        span_copy(size_name, sizeof size_name, target->text, func->sz_name0[index],
                                  func->sz_name1[index]);
                        snprintf(message, sizeof message, "`%s` is defined for `%s` in %lld..%lld", name, size_name,
                                 (long long)func->sz_lo[index], (long long)func->sz_hi[index]);
                        snprintf(label, sizeof label, "this size is %lld", (long long)value.value);
                        add_diag(c, "ORC0238", c->exprs[entry].start, c->exprs[entry].end, message, label,
                                 SIZE_RANGE_NOTE, 2);
                    }
                    return 0;
                }
                values[index] = value.value;
            }
        }
        {
            uint32_t cursor;
            for (cursor = 0; cursor < func->ninst; cursor++) {
                Instance *inst = &target->instances[func->inst0 + cursor];
                int match = 1;
                for (index = 0; index < func->nsizes; index++) {
                    if (inst->sz[index] != values[index]) {
                        match = 0;
                        break;
                    }
                }
                if (match) {
                    *inst_id = func->inst0 + cursor;
                    expr->inst_id = *inst_id;
                    return 1;
                }
            }
        }
        return 0;
    }
    saved_set = c->fit_set;
    saved_report = c->fit_report;
    saved_kind = c->fit_kind;
    saved_len = c->fit_len;
    saved_mod = c->fit_mod;
    saved_tup0 = c->fit_tup0;
    saved_tup_n = c->fit_tup_n;
    c->fit_set = 0;
    c->fit_report = 0;
    {
        TpTy args[MAX_PARAMS];
        uint32_t fitting[2];
        uint32_t matching[2];
        int nfit = 0;
        int nmatch = 0;
        uint32_t cursor;
        uint16_t param;
        int use_expected = saved_set && saved_kind != TY_NONE;
        for (param = 0; param < func->nparams && param < MAX_PARAMS; param++) {
            memset(&args[param], 0, sizeof args[param]);
            args[param].owner = c;
            if (param < expr->argc) {
                arg_type(c, c->args[expr->arg0 + param], caller_func, locals, &args[param]);
            }
        }
        c->fit_set = saved_set;
        c->fit_report = saved_report;
        c->fit_kind = saved_kind;
        c->fit_len = saved_len;
        c->fit_mod = saved_mod;
        c->fit_tup0 = saved_tup0;
        c->fit_tup_n = saved_tup_n;
        for (cursor = 0; cursor < func->ninst; cursor++) {
            Instance *inst = &target->instances[func->inst0 + cursor];
            int fits = 1;
            for (param = 0; param < func->nparams && param < MAX_PARAMS && fits; param++) {
                if (param >= expr->argc) {
                    continue;
                }
                fits = param_fits(target, &target->iparams[inst->param0 + param], &args[param]);
            }
            if (!fits) {
                continue;
            }
            if (nfit < 2) {
                fitting[nfit] = func->inst0 + cursor;
            }
            nfit++;
            if (result_gives(c, target, inst, use_expected, saved_kind, saved_len, saved_mod, saved_tup0, saved_tup_n)) {
                if (nmatch < 2) {
                    matching[nmatch] = func->inst0 + cursor;
                }
                nmatch++;
            }
            if (nfit > 1 && !use_expected) {
                break;
            }
        }
        if (nfit == 1 || nmatch == 1) {
            uint32_t chosen = nfit == 1 ? fitting[0] : matching[0];
            *inst_id = chosen;
            expr->inst_id = chosen;
            return 1;
        }
        if (!report) {
            return 0;
        }
        if (nmatch >= 2) {
            report_unfitted(c, expr, target, func, name, args, nmatch, matching);
        } else {
            report_unfitted(c, expr, target, func, name, args, nfit, fitting);
        }
        return 0;
    }
}

static void write_values(const Compiler *c, const Func *func, const int64_t *values, char *buf, size_t cap) {
    size_t used = 0;
    uint8_t slot;
    if (cap == 0) {
        return;
    }
    buf[0] = '\0';
    for (slot = 0; slot < func->nsizes; slot++) {
        char piece[96];
        int n;
        if (func->sz_kind[slot]) {
            char spelling[64];
            int64_t index = values[slot];
            spelling[0] = '\0';
            if (index >= 0) {
                listed_spelling(c, func, slot, (uint32_t)index, spelling, sizeof spelling);
            }
            snprintf(piece, sizeof piece, "%s%s", slot == 0 ? "" : ", ", spelling);
        } else {
            snprintf(piece, sizeof piece, "%s%lld", slot == 0 ? "" : ", ", (long long)values[slot]);
        }
        n = (int)strlen(piece);
        if (used + (size_t)n + 1 >= cap) {
            break;
        }
        memcpy(buf + used, piece, (size_t)n);
        used += (size_t)n;
        buf[used] = '\0';
    }
}

void tp_write_values(const Compiler *c, const Func *func, const Instance *inst, char *buf, size_t cap) {
    write_values(c, func, inst->sz, buf, cap);
}

void tp_format_label(const Compiler *c, uint32_t inst, char *buf, size_t cap) {
    const Instance *instance;
    const Func *func;
    char values[160];
    size_t name_len;
    if (cap == 0) {
        return;
    }
    buf[0] = '\0';
    if (inst >= c->ninstances) {
        return;
    }
    instance = &c->instances[inst];
    func = &c->funcs[instance->func];
    name_len = func->name_end - func->name_start;
    if (name_len >= cap) {
        name_len = cap - 1;
    }
    memcpy(buf, c->text + func->name_start, name_len);
    buf[name_len] = '\0';
    if (func->nsizes == 0) {
        return;
    }
    write_values(c, func, instance->sz, values, sizeof values);
    snprintf(buf + name_len, cap - name_len, "[%s]", values);
}

void tp_name_diags(Compiler *c, const Func *func, uint32_t inst, uint32_t from) {
    char label[96];
    char name[64];
    char note[320];
    const char *rule;
    uint32_t index;
    if (func->nsizes == 0) {
        return;
    }
    tp_format_label(c, inst, label, sizeof label);
    span_copy(name, sizeof name, c->text, func->name_start, func->name_end);
    if (tp_func_has_types(func) && tp_has_sizes(func)) {
        rule = "a function is checked once for each value of its sizes and each type of its type parameters";
    } else if (tp_func_has_types(func)) {
        rule = "a function is checked once for each type of its type parameters";
    } else {
        rule = "a sized function is checked once for each value of its sizes";
    }
    snprintf(note, sizeof note, "in the instance `%s`, the first of `%s` in error: %s", label, name, rule);
    for (index = from; index < c->ndiags; index++) {
        Diag *diag = &c->diags[index];
        if (strcmp(diag->code, "ORC0208") == 0 || strcmp(diag->code, "ORC0209") == 0) {
            continue;
        }
        if (diag->note[0] == '\0') {
            copy_text(diag->note, sizeof diag->note, note);
        } else if (!diag->has_note2) {
            copy_text(diag->note2, sizeof diag->note2, note);
            diag->has_note2 = 1;
        }
    }
}

void tp_refresh_convs(Compiler *c, uint32_t func_index) {
    uint32_t index;
    if (!tp_func_has_types(&c->funcs[func_index])) {
        return;
    }
    for (index = 0; index < c->nexprs; index++) {
        Expr *expr = &c->exprs[index];
        TypeSite *site;
        if (expr->kind != EX_CONV || expr->conv_site == UINT32_MAX || expr->conv_site >= c->nsites) {
            continue;
        }
        site = &c->sites[expr->conv_site];
        if (site->owner_func != func_index || site->param_slot == 0xFF) {
            continue;
        }
        tp_materialize(c, site, 0);
        expr->conv_ty = site->kind;
        expr->conv_ok = site->ok;
        expr->conv_len = site->rank <= 0 ? 0u : site->length;
        expr->conv_mod = site->mod_index;
        expr->ty_len = expr->conv_len;
        expr->ty_mod = site->mod_index;
    }
}

void tp_note_expr(Compiler *c, Expr *expr) {
    TpStamp stamp;
    if (c->cur_func >= c->nfuncs || c->cur_inst == UINT32_MAX || !tp_func_has_types(&c->funcs[c->cur_func])) {
        return;
    }
    /* Every expression records the type of this instance. Evaluation reads
       `ty` back from the expression, and the last instance checked would
       otherwise leave its type on an earlier instance's arrays and calls. */
    memset(&stamp, 0, sizeof stamp);
    stamp.expr = (uint32_t)(expr - c->exprs);
    stamp.inst = c->cur_inst;
    stamp.ty = expr->ty;
    stamp.ty_len = expr->ty_len;
    stamp.ty_mod = expr->ty_mod;
    stamp.conv_ty = expr->conv_ty;
    stamp.conv_len = expr->conv_len;
    stamp.conv_mod = expr->conv_mod;
    stamp.conv_ok = expr->conv_ok;
    if (c->nstamps > 0) {
        TpStamp *last = &c->stamps[c->nstamps - 1];
        if (last->expr == stamp.expr && last->inst == stamp.inst) {
            *last = stamp;
            return;
        }
    }
    if (!ensure_cap((void **)&c->stamps, &c->stamp_cap, c->nstamps + 1, sizeof(TpStamp), MAX_EXPRS)) {
        resource_diag(c, "ORC0209", expr->start, expr->end, "semantic analysis could not retain instance types");
        return;
    }
    c->stamps[c->nstamps++] = stamp;
}

void tp_apply_stamps(Compiler *c) {
    uint32_t index;
    if (c->cur_inst == UINT32_MAX) {
        return;
    }
    for (index = 0; index < c->nstamps; index++) {
        TpStamp *stamp = &c->stamps[index];
        Expr *expr;
        if (stamp->inst != c->cur_inst || stamp->expr >= c->nexprs) {
            continue;
        }
        expr = &c->exprs[stamp->expr];
        expr->ty = stamp->ty;
        expr->ty_len = stamp->ty_len;
        expr->ty_mod = stamp->ty_mod;
        if (expr->kind == EX_CONV) {
            expr->conv_ty = stamp->conv_ty;
            expr->conv_len = stamp->conv_len;
            expr->conv_mod = stamp->conv_mod;
            expr->conv_ok = stamp->conv_ok;
        }
    }
}
