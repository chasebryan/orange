#ifndef ORANGE_COMPILE_H
#define ORANGE_COMPILE_H

int orange_main(int argc, char **argv);

#endif

/* Pasted into src/compile.c when ORANGE_COMPILE_REST is set. */
#ifdef ORANGE_COMPILE_REST
#include "pack.h"
static const char MODULUS_NOTE[] =
    "a modulus is a constant built from integer literals with `+`, `-`, `*`, `<<`, and parentheses, as in "
    "`Mod[(1 << 255) - 19]`";
static const char ADMITTED_TYPE_LABEL[] =
    "the admitted types are `Int`, `Bool`, `Word[8]`, `Word[16]`, `Word[32]`, `Word[64]`, `Mod[m]`, and the names of "
    "earlier `type` declarations";

static void report_bad_modulus(Compiler *c, uint32_t start, uint32_t end, const char *label) {
    add_diag(c, "ORC0232", start, end, "a modulus must be a constant from 2 through 2^521 - 1", label, MODULUS_NOTE, 2);
}

static void report_modulus_large(Compiler *c, uint32_t start, uint32_t end) {
    add_diag(c, "ORC0205", start, end, "integer magnitude exceeds the 16384-significant-bit limit",
             "this value of the modulus is too large", "the value is rejected rather than truncated or approximated", 2);
}

static int modulus_const(Compiler *c, uint32_t index, Big *out, int *ok);

static int modulus_const(Compiler *c, uint32_t index, Big *out, int *ok) {
    const Expr *expr = &c->exprs[index];
    *ok = 0;
    *out = big_zero();
    if (c->resource) {
        return 0;
    }
    if (expr->kind == EX_GROUP) {
        return modulus_const(c, expr->left, out, ok);
    }
    if (expr->kind == EX_LIT) {
        if (!decode_literal(c, expr, out)) {
            report_modulus_large(c, expr->lit_start, expr->lit_end);
            return 1;
        }
        *ok = 1;
        return 1;
    }
    if (expr->kind == EX_SHIFT && expr->op == TK_LSHIFT) {
        Big left = big_zero();
        Big right = big_zero();
        int left_ok = 0;
        int right_ok = 0;
        uint32_t amount;
        if (!modulus_const(c, expr->left, &left, &left_ok) || !left_ok) {
            return left_ok || c->resource ? 1 : 0;
        }
        if (!modulus_const(c, expr->right, &right, &right_ok) || !right_ok) {
            return right_ok || c->resource ? 1 : 0;
        }
        if (right.negative || right.nlimbs > 1 || (right.nlimbs > 0 && right.limbs[0] > 16384u)) {
            report_bad_modulus(c, c->exprs[expr->right].start, c->exprs[expr->right].end,
                               "a shift amount in a modulus is from 0 through 16384");
            return 1;
        }
        amount = right.nlimbs == 0 ? 0u : right.limbs[0];
        if (!big_shl(&c->arena, &left, amount, out)) {
            report_modulus_large(c, expr->start, expr->end);
            return 1;
        }
        *ok = 1;
        return 1;
    }
    if (expr->kind == EX_BINARY && (expr->op == TK_PLUS || expr->op == TK_MINUS || expr->op == TK_STAR)) {
        Big left = big_zero();
        Big right = big_zero();
        int left_ok = 0;
        int right_ok = 0;
        int computed = 0;
        if (!modulus_const(c, expr->left, &left, &left_ok) || !left_ok) {
            return left_ok || c->resource ? 1 : 0;
        }
        if (!modulus_const(c, expr->right, &right, &right_ok) || !right_ok) {
            return right_ok || c->resource ? 1 : 0;
        }
        if (expr->op == TK_PLUS) {
            computed = big_add(&c->arena, &left, &right, out);
        } else if (expr->op == TK_MINUS) {
            computed = big_sub(&c->arena, &left, &right, out);
        } else {
            computed = big_mul(&c->arena, &left, &right, out);
        }
        if (!computed) {
            report_modulus_large(c, expr->start, expr->end);
            return 1;
        }
        *ok = 1;
        return 1;
    }
    report_bad_modulus(c, expr->start, expr->end, "not a constant integer expression");
    return 1;
}

static int intern_modulus(Compiler *c, const Big *value, uint16_t *out) {
    uint16_t index;
    if (c->moduli == NULL) {
        c->moduli = calloc(MAX_MODULI, sizeof(Big));
        if (c->moduli == NULL) {
            resource_diag(c, "ORC0209", 0, 0, "semantic analysis could not retain moduli");
            return 0;
        }
        c->nmoduli = 1;
    }
    for (index = 1; index < c->nmoduli; index++) {
        if (big_cmp(&c->moduli[index], value) == 0) {
            *out = index;
            return 1;
        }
    }
    if (c->nmoduli >= MAX_MODULI) {
        resource_diag(c, "ORC0209", 0, 0, "semantic analysis could not retain moduli");
        return 0;
    }
    c->moduli[c->nmoduli] = *value;
    *out = c->nmoduli++;
    return 1;
}

static int admit_modulus(Compiler *c, uint32_t start, uint32_t end, const Big *value, uint16_t *mod_index) {
    char label[128];
    char digits[96];
    uint32_t bits = big_bits(value);
    if (value->negative || bits < 2) {
        if (value->negative && bits > 64) {
            copy_text(label, sizeof label, "this modulus is negative");
        } else if (big_format(value, digits, sizeof digits)) {
            snprintf(label, sizeof label, "this modulus is %s", digits);
        } else {
            copy_text(label, sizeof label, "this modulus is outside 2 through 2^521 - 1");
        }
        report_bad_modulus(c, start, end, label);
        return 0;
    }
    if (bits > MAX_MODULUS_BITS) {
        snprintf(label, sizeof label, "this modulus has %u bits", bits);
        report_bad_modulus(c, start, end, label);
        return 0;
    }
    return intern_modulus(c, value, mod_index);
}

static void walk_moduli(Compiler *c, uint32_t index);

void bind_modulus(Compiler *c, TypeSite *site) {
    Big value = big_zero();
    int ok = 0;
    if (site == NULL || !site->has_mod || site->modulus_done || c->resource) {
        return;
    }
    site->modulus_done = 1;
    if (!modulus_const(c, site->mod_expr, &value, &ok)) {
        return;
    }
    if (!ok || !admit_modulus(c, c->exprs[site->mod_expr].start, c->exprs[site->mod_expr].end, &value, &site->mod_index)) {
        site->ok = 0;
        site->reported = 1;
        site->mod_index = 0;
    }
    walk_moduli(c, site->mod_expr);
}

static void walk_moduli(Compiler *c, uint32_t index) {
    const Expr *expr;
    uint32_t arg;
    if (index == UINT32_MAX || c->resource) {
        return;
    }
    expr = &c->exprs[index];
    switch (expr->kind) {
    case EX_GROUP:
    case EX_UNARY:
    case EX_FILL:
    case EX_PROJECT:
        walk_moduli(c, expr->left);
        break;
    case EX_BINARY:
    case EX_SHIFT:
    case EX_INDEX:
    case EX_SELECT:
        walk_moduli(c, expr->left);
        walk_moduli(c, expr->right);
        break;
    case EX_UPDATE:
        walk_moduli(c, expr->left);
        walk_moduli(c, expr->right);
        walk_moduli(c, expr->callee);
        break;
    case EX_SLICE:
        walk_moduli(c, expr->left);
        walk_moduli(c, expr->right);
        walk_moduli(c, expr->callee);
        break;
    case EX_SLICE_UP:
        walk_moduli(c, expr->left);
        walk_moduli(c, expr->right);
        walk_moduli(c, expr->conv_site);
        walk_moduli(c, expr->callee);
        break;
    case EX_CALL:
    case EX_ARRAY:
    case EX_TUPLE:
        for (arg = 0; arg < expr->argc; arg++) {
            walk_moduli(c, c->args[expr->arg0 + arg]);
        }
        break;
    case EX_CONV:
        walk_moduli(c, expr->left);
        if (expr->conv_site != UINT32_MAX && expr->conv_site < c->nsites) {
            bind_modulus(c, &c->sites[expr->conv_site]);
        }
        break;
    case EX_LOOP:
        if (expr->arg0 < c->nloops && c->loops[expr->arg0].site != UINT32_MAX &&
            c->loops[expr->arg0].site < c->nsites) {
            bind_modulus(c, &c->sites[c->loops[expr->arg0].site]);
        }
        walk_moduli(c, c->loops[expr->arg0].init_expr);
        if (expr->arg0 < c->nloops) {
            LoopDesc *loop = &c->loops[expr->arg0];
            uint16_t bind;
            for (bind = 0; bind < loop->nbinds; bind++) {
                Local *local = &c->block_locals[loop->bind0 + bind];
                if (local->site != UINT32_MAX && local->site < c->nsites) {
                    bind_modulus(c, &c->sites[local->site]);
                }
                walk_moduli(c, local->value);
            }
        }
        walk_moduli(c, c->loops[expr->arg0].step_expr);
        break;
    case EX_COND:
        for (arg = 0; arg < expr->argc; arg++) {
            CondArm *item = &c->cond_arms[expr->arg0 + arg];
            uint16_t bind;
            walk_moduli(c, item->cond);
            for (bind = 0; bind < item->nbinds; bind++) {
                Local *local = &c->block_locals[item->bind0 + bind];
                if (local->site != UINT32_MAX && local->site < c->nsites) {
                    bind_modulus(c, &c->sites[local->site]);
                }
                walk_moduli(c, local->value);
            }
            walk_moduli(c, item->value);
        }
        {
            uint16_t bind;
            for (bind = 0; bind < expr->else_nbinds; bind++) {
                Local *local = &c->block_locals[expr->else_bind0 + bind];
                if (local->site != UINT32_MAX && local->site < c->nsites) {
                    bind_modulus(c, &c->sites[local->site]);
                }
                walk_moduli(c, local->value);
            }
        }
        walk_moduli(c, expr->right);
        break;
    default:
        break;
    }
}

int builtin_type_name(const Compiler *c, uint32_t start, uint32_t end) {
    return span_is(c, start, end, "Int") || span_is(c, start, end, "Bool") || span_is(c, start, end, "Word") ||
           span_is(c, start, end, "Mod");
}

static int decl_spells(const Compiler *c, const TypeDecl *decl, uint32_t start, uint32_t end) {
    return same_span(c, decl->name_start, decl->name_end, start, end);
}

static int find_installed_type(const Compiler *c, uint32_t start, uint32_t end, uint32_t limit, uint32_t *index) {
    uint32_t cursor;
    if (limit > c->ntypes) {
        limit = c->ntypes;
    }
    for (cursor = 0; cursor < limit; cursor++) {
        if (!c->types[cursor].installed || !decl_spells(c, &c->types[cursor], start, end)) {
            continue;
        }
        *index = cursor;
        return 1;
    }
    return 0;
}

void copy_ident(char *dest, size_t cap, const Compiler *c, uint32_t start, uint32_t end) {
    span_copy(dest, cap, c->text, start, end);
}

static int append_telem(Compiler *c, TypeKind kind, uint32_t length, uint16_t mod_index, int ok, uint32_t *index) {
    TupleElem *elem;
    if (!ensure_cap((void **)&c->telems, &c->telem_cap, c->ntelems + 1, sizeof(TupleElem), MAX_EXPRS)) {
        resource_diag(c, "ORC0209", 0, 0, "tuple type storage allocation failed");
        return 0;
    }
    elem = &c->telems[c->ntelems];
    memset(elem, 0, sizeof *elem);
    elem->kind = kind;
    elem->length = length;
    elem->mod_index = mod_index;
    elem->ok = ok;
    *index = c->ntelems++;
    return 1;
}

static int adopt_tuple_shape(Compiler *c, const Compiler *owner, uint32_t tup0, uint16_t tup_n, uint32_t *out0) {
    uint16_t index;
    uint32_t start = 0;
    if (owner == NULL || owner == c || tup_n == 0) {
        *out0 = tup0;
        return 1;
    }
    for (index = 0; index < tup_n; index++) {
        TupleElem src = owner->telems[tup0 + index];
        uint16_t mod = src.mod_index;
        uint32_t at = 0;
        if (src.kind == TY_MOD && !adopt_modulus(c, owner, src.mod_index, &mod)) {
            return 0;
        }
        if (!append_telem(c, src.kind, src.length, mod, src.ok, &at)) {
            return 0;
        }
        if (index == 0) {
            start = at;
        }
    }
    *out0 = start;
    return 1;
}

void resolve_site(Compiler *c, TypeSite *site, int from_decl, uint32_t earlier_limit) {
    uint32_t found = 0;
    TypeSite *target;
    char name[64];
    if (site->resolved || c->resource) {
        return;
    }
    if (site->role != NULL && strcmp(site->role, "listed type") == 0 && !c->admit_listed) {
        return;
    }
    site->resolved = 1;
    if (site->is_tuple) {
        uint16_t index;
        int ok = 1;
        int unreported = 0;
        uint32_t start = 0;
        for (index = 0; index < site->elem_n; index++) {
            TypeSite *elem;
            if (site->elem0 + index >= c->nsites) {
                ok = 0;
                unreported = 1;
                break;
            }
            elem = &c->sites[site->elem0 + index];
            resolve_site(c, elem, from_decl, earlier_limit);
            if (!elem->ok || elem->kind == TY_TUPLE) {
                ok = 0;
                if (elem->length_bad && !site->length_bad) {
                    site->length_bad = 1;
                    site->length_start = elem->length_start;
                    site->length_end = elem->length_end;
                }
                if (!elem->reported) {
                    unreported = 1;
                }
            }
        }
        site->kind = TY_TUPLE;
        site->rank = 0;
        site->length = 0;
        if (!ok) {
            site->ok = 0;
            site->reported = unreported ? 0 : 1;
            return;
        }
        for (index = 0; index < site->elem_n; index++) {
            TypeSite *elem = &c->sites[site->elem0 + index];
            uint32_t at = 0;
            uint32_t length = elem->rank <= 0 ? 0u : elem->length;
            if (!append_telem(c, elem->kind, length, elem->mod_index, elem->ok, &at)) {
                site->ok = 0;
                return;
            }
            if (index == 0) {
                start = at;
            }
        }
        site->tup0 = start;
        site->tup_n = site->elem_n;
        site->ok = 1;
        return;
    }
    if (site->length_bad) {
        site->ok = 0;
        return;
    }
    if (site->has_mod) {
        if (site->mod_index == 0) {
            site->ok = 0;
            site->reported = 1;
            return;
        }
        site->kind = TY_MOD;
        site->ok = 1;
        site->rank = site->wrote_axis ? 1 : 0;
        return;
    }
    if (site->bare_mod) {
        add_diag(c, "ORC0232", site->ident_start, site->ident_end, "`Mod` requires a modulus", "missing modulus",
                 MODULUS_NOTE, 2);
        site->ok = 0;
        site->reported = 1;
        site->kind = TY_MOD;
        return;
    }
    if (!site->named) {
        if (site->ok) {
            site->rank = site->wrote_axis ? 1 : 0;
        } else if (!span_is(c, site->ident_start, site->ident_end, "Word") && !site->reported) {
            char message[160];
            copy_ident(name, sizeof name, c, site->ident_start, site->ident_end);
            snprintf(message, sizeof message, "unsupported %s `%s`", site->role != NULL ? site->role : "type", name);
            add_diag(c, "ORC0203", site->start, site->end, message, ADMITTED_TYPE_LABEL,
                     "types are resolved contextually and never inferred by spelling similarity", 2);
            site->reported = 1;
        }
        return;
    }
    if (tp_bind_use(c, site)) {
        return;
    }
    if (!find_installed_type(c, site->ident_start, site->ident_end, from_decl ? earlier_limit : c->ntypes, &found)) {
        int later = 0;
        uint32_t cursor;
        char message[160];
        char note[256];
        copy_ident(name, sizeof name, c, site->ident_start, site->ident_end);
        snprintf(message, sizeof message, "unsupported %s `%s`", site->role != NULL ? site->role : "type", name);
        if (from_decl) {
            for (cursor = earlier_limit; cursor < c->ntypes; cursor++) {
                if (decl_spells(c, &c->types[cursor], site->ident_start, site->ident_end)) {
                    later = 1;
                    break;
                }
            }
        }
        if (later) {
            snprintf(note, sizeof note,
                     "`%s` is declared by a later `type` declaration; a `type` declaration uses only the names "
                     "declared before it",
                     name);
        } else {
            copy_text(note, sizeof note, "types are resolved contextually and never inferred by spelling similarity");
        }
        add_diag(c, "ORC0203", site->start, site->end, message, ADMITTED_TYPE_LABEL, note, 2);
        site->ok = 0;
        site->reported = 1;
        return;
    }
    target = &c->sites[c->types[found].site];
    if (!target->ok) {
        site->ok = 0;
        site->reported = 1;
        return;
    }
    if (target->kind == TY_TUPLE) {
        char message[160];
        copy_ident(name, sizeof name, c, site->ident_start, site->ident_end);
        if (site->wrote_axis) {
            snprintf(message, sizeof message, "`%s` is a tuple type, so this is an array of tuples", name);
            add_diag(c, "ORC0203", site->start, site->end, message, "arrays of tuples are not part of Orange 2026",
                     "an array's elements are `Int`, `Bool`, words, or residues", 2);
            if (site->length_end > site->length_start) {
                diag_add_secondary(c, site->length_start, site->length_end,
                                   "this length would make each element a tuple");
            }
            site->ok = 0;
            site->reported = 1;
            return;
        }
        if (site->tuple_elem) {
            snprintf(message, sizeof message, "`%s` is a tuple type, so this is a tuple of tuples", name);
            add_diag(c, "ORC0203", site->start, site->end, message, "a tuple holds no tuple",
                     "a tuple's elements are `Int`, `Bool`, words, residues, and arrays of them", 2);
            site->ok = 0;
            site->reported = 1;
            return;
        }
        site->kind = TY_TUPLE;
        site->is_tuple = 1;
        site->tup0 = target->tup0;
        site->tup_n = target->tup_n;
        site->rank = 0;
        site->length = 0;
        site->ok = 1;
        return;
    }
    if (target->rank >= 2 && site->wrote_axis) {
        char message[160];
        copy_ident(name, sizeof name, c, site->ident_start, site->ident_end);
        snprintf(message, sizeof message, "`%s` already has two array dimensions", name);
        add_diag(c, "ORC0203", site->start, site->end, message, "arrays have at most two dimensions",
                 "a row holds scalars; a matrix holds rows of the same type", 2);
        if (site->length_end > site->length_start) {
            diag_add_secondary(c, site->length_start, site->length_end, "this length would add a third dimension");
        }
        site->ok = 0;
        site->reported = 1;
        return;
    }
    site->kind = target->kind;
    site->mod_index = target->mod_index;
    if (site->wrote_axis) {
        site->rank = target->rank + 1;
        if (target->rank >= 1) {
            site->inner_len = target->length;
        }
    } else {
        site->rank = target->rank;
        site->length = target->length;
        site->inner_len = target->inner_len;
    }
    site->ok = 1;
}

static void publish_site(const Compiler *c, uint32_t site_index, TypeKind *kind, uint32_t *length, int *ok,
                         uint16_t *mod_index, int *reported) {
    const TypeSite *site;
    if (site_index == UINT32_MAX || site_index >= c->nsites) {
        return;
    }
    site = &c->sites[site_index];
    *kind = site->kind;
    *ok = site->ok;
    *mod_index = site->mod_index;
    *reported = site->reported;
    *length = site->rank <= 0 ? 0u : site->length;
}

static void publish_shape(const Compiler *c, uint32_t site_index, TypeKind kind, uint32_t *tup0, uint16_t *tup_n) {
    *tup0 = 0;
    *tup_n = 0;
    if (kind != TY_TUPLE || site_index == UINT32_MAX || site_index >= c->nsites) {
        return;
    }
    *tup0 = c->sites[site_index].tup0;
    *tup_n = c->sites[site_index].tup_n;
}

static void seal_patterns(Compiler *c, Local *locals, uint32_t count) {
    uint32_t index;
    for (index = 0; index < count; index++) {
        Local *head = &locals[index];
        uint16_t pat;
        uint32_t start = 0;
        if (head->pat_len < 2 || index + head->pat_len > count) {
            continue;
        }
        for (pat = 0; pat < head->pat_len; pat++) {
            Local *name = &locals[index + pat];
            uint32_t at = 0;
            if (!append_telem(c, name->type, name->length, name->mod_index, name->type_ok, &at)) {
                return;
            }
            if (pat == 0) {
                start = at;
            }
        }
        for (pat = 0; pat < head->pat_len; pat++) {
            locals[index + pat].tup0 = start;
            locals[index + pat].tup_n = head->pat_len;
        }
    }
}

static void prepare_types(Compiler *c) {
    uint32_t index;
    uint16_t param;
    uint16_t local;
    if (c->resource || c->parse_diags > 0) {
        return;
    }
    for (index = 0; index < c->nsites; index++) {
        TypeSite *site = &c->sites[index];
        uint16_t elem;
        if (!site->is_tuple) {
            continue;
        }
        for (elem = 0; elem < site->elem_n; elem++) {
            if (site->elem0 + elem < c->nsites) {
                bind_modulus(c, &c->sites[site->elem0 + elem]);
            }
        }
    }
    for (index = 0; index < c->ntypes; index++) {
        TypeSite *site = &c->sites[c->types[index].site];
        bind_modulus(c, site);
    }
    for (index = 0; index < c->nfuncs; index++) {
        Func *func = &c->funcs[index];
        if (!func->typed) {
            continue;
        }
        for (param = 0; param < func->nparams; param++) {
            Param *item = &c->params[func->param0 + param];
            if (item->site != UINT32_MAX && item->site < c->nsites) {
                bind_modulus(c, &c->sites[item->site]);
            }
        }
        if (func->result_site != UINT32_MAX && func->result_site < c->nsites) {
            bind_modulus(c, &c->sites[func->result_site]);
        }
        for (local = 0; local < func->nlocals; local++) {
            Local *item = &c->locals[func->local0 + local];
            if (item->site != UINT32_MAX && item->site < c->nsites) {
                bind_modulus(c, &c->sites[item->site]);
            }
            walk_moduli(c, item->value);
        }
        walk_moduli(c, func->body);
    }
    for (index = 0; index < c->ntypes; index++) {
        TypeDecl *decl = &c->types[index];
        char message[128];
        char name[64];
        uint32_t earlier = 0;
        copy_ident(name, sizeof name, c, decl->name_start, decl->name_end);
        if (builtin_type_name(c, decl->name_start, decl->name_end)) {
            snprintf(message, sizeof message, "`%s` is a built-in type", name);
            add_diag(c, "ORC0233", decl->name_start, decl->name_end, message,
                     "a `type` declaration cannot name a built-in type",
                     "the built-in types are `Int`, `Bool`, `Word[n]`, and `Mod[m]`", 2);
        } else if (find_installed_type(c, decl->name_start, decl->name_end, index, &earlier)) {
            snprintf(message, sizeof message, "duplicate type name `%s`", name);
            add_diag(c, "ORC0233", decl->name_start, decl->name_end, message, "this declaration repeats a type name",
                     "each `type` declaration of a module names a different type", 2);
            diag_add_secondary(c, c->types[earlier].name_start, c->types[earlier].name_end, "first declaration is here");
        } else {
            decl->installed = 1;
        }
        resolve_site(c, &c->sites[decl->site], 1, index);
        /* `resolve_site` leaves a bad `Word` unreported so `reject_declared`
           can underline the width of a direct signature. Diagnose the alias
           once, at this declaration. A use, a further alias, or a tuple
           element does not add another error. */
        report_alias_target(c, &c->sites[decl->site]);
    }
    for (index = 0; index < c->nsites; index++) {
        resolve_site(c, &c->sites[index], 0, c->ntypes);
    }
    for (index = 0; index < c->nfuncs; index++) {
        Func *func = &c->funcs[index];
        for (param = 0; param < func->nparams; param++) {
            Param *item = &c->params[func->param0 + param];
            publish_site(c, item->site, &item->type, &item->length, &item->type_ok, &item->mod_index, &item->type_reported);
            publish_shape(c, item->site, item->type, &item->tup0, &item->tup_n);
        }
        publish_site(c, func->result_site, &func->result, &func->result_len, &func->result_ok, &func->result_mod,
                     &func->result_reported);
        publish_shape(c, func->result_site, func->result, &func->tup0, &func->tup_n);
        for (local = 0; local < func->nlocals; local++) {
            Local *item = &c->locals[func->local0 + local];
            publish_site(c, item->site, &item->type, &item->length, &item->type_ok, &item->mod_index, &item->type_reported);
            /* Pattern heads take their shape from the names, not from one
               element's site. `seal_patterns` fills those after this loop. */
            if (item->pat_len == 0 && item->pat_i == 0) {
                publish_shape(c, item->site, item->type, &item->tup0, &item->tup_n);
            }
        }
        if (func->nlocals > 0) {
            seal_patterns(c, &c->locals[func->local0], func->nlocals);
        }
    }
    for (index = 0; index < c->nblock_locals; index++) {
        Local *item = &c->block_locals[index];
        publish_site(c, item->site, &item->type, &item->length, &item->type_ok, &item->mod_index, &item->type_reported);
        if (item->pat_len == 0 && item->pat_i == 0) {
            publish_shape(c, item->site, item->type, &item->tup0, &item->tup_n);
        }
    }
    seal_patterns(c, c->block_locals, c->nblock_locals);
    for (index = 0; index < c->nloops; index++) {
        LoopDesc *loop = &c->loops[index];
        publish_site(c, loop->site, &loop->acc_type, &loop->acc_len, &loop->acc_ok, &loop->acc_mod, &loop->acc_reported);
        publish_shape(c, loop->site, loop->acc_type, &loop->tup0, &loop->tup_n);
    }
    for (index = 0; index < c->nexprs; index++) {
        Expr *expr = &c->exprs[index];
        TypeKind kind = TY_NONE;
        uint32_t length = 0;
        int ok = 0;
        uint16_t mod_index = 0;
        int reported = 0;
        if (expr->kind != EX_CONV || expr->conv_site == UINT32_MAX) {
            continue;
        }
        publish_site(c, expr->conv_site, &kind, &length, &ok, &mod_index, &reported);
        expr->conv_ty = kind;
        expr->conv_ok = ok;
        expr->conv_len = length;
        expr->conv_mod = mod_index;
        expr->ty_len = length;
        expr->ty_mod = mod_index;
    }
}

typedef struct Applied {
    TypeKind kind;
    uint32_t length;
    int ok;
    uint16_t mod_index;
    int reported;
    uint32_t tup0;
    uint16_t tup_n;
} Applied;

static int site_uses_size(const Compiler *c, uint32_t site_index) {
    const TypeSite *site;
    uint16_t index;
    if (site_index == UINT32_MAX || site_index >= c->nsites) {
        return 0;
    }
    site = &c->sites[site_index];
    if (site->param_slot != 0xFF) {
        return 1;
    }
    if (site->has_size_expr) {
        return 1;
    }
    if (site->kind == TY_TUPLE || site->is_tuple) {
        for (index = 0; index < site->elem_n; index++) {
            if (site_uses_size(c, site->elem0 + index)) {
                return 1;
            }
        }
    }
    return 0;
}

/* Lengths written with sizes are computed for the current instance. A failure
   is reported only when `report` is set; signature capture stays silent. */
static int apply_site(Compiler *c, uint32_t site_index, int report, Applied *out) {
    TypeSite *site;
    memset(out, 0, sizeof *out);
    if (site_index == UINT32_MAX || site_index >= c->nsites) {
        return 1;
    }
    site = &c->sites[site_index];
    out->kind = site->kind;
    out->ok = site->ok;
    out->mod_index = site->mod_index;
    out->reported = site->reported;
    out->length = site->rank <= 0 ? 0u : site->length;
    out->tup0 = site->tup0;
    out->tup_n = site->tup_n;
    if (site->param_slot != 0xFF) {
        tp_materialize(c, site, report);
        out->kind = site->kind == TY_TUPLE || site->is_tuple ? TY_TUPLE : site->kind;
        out->ok = site->ok;
        out->mod_index = site->mod_index;
        out->reported = site->reported;
        out->length = site->rank <= 0 ? 0u : site->length;
        out->tup0 = site->tup0;
        out->tup_n = site->tup_n;
        if (out->kind == TY_TUPLE) {
            out->length = 0;
        }
        return 1;
    }
    if (site->kind == TY_TUPLE || site->is_tuple) {
        int all_ok = site->ok;
        int rebuild = site_uses_size(c, site_index);
        uint32_t start = 0;
        uint16_t index;
        out->kind = TY_TUPLE;
        out->length = 0;
        for (index = 0; index < site->elem_n; index++) {
            Applied elem;
            uint32_t at = 0;
            if (!apply_site(c, site->elem0 + index, report, &elem)) {
                return 0;
            }
            if (!elem.ok) {
                all_ok = 0;
            }
            if (rebuild && !append_telem(c, elem.kind, elem.length, elem.mod_index, elem.ok, &at)) {
                return 0;
            }
            if (index == 0) {
                start = at;
            }
        }
        out->ok = all_ok;
        if (rebuild && all_ok) {
            out->tup0 = start;
            out->tup_n = site->elem_n;
            site->tup0 = start;
            site->tup_n = site->elem_n;
        }
        return 1;
    }
    if (site->has_size_expr && site->ok) {
        uint32_t length = 0;
        if (!size_length(c, site->length_expr, report, &length)) {
            out->ok = 0;
            out->length = 0;
            if (report) {
                out->reported = 1;
            }
            return 1;
        }
        site->length = length;
        site->rank = 1;
        out->length = length;
        out->ok = 1;
    }
    return 1;
}

static int push_instance(Compiler *c, const Instance *inst) {
    if (!ensure_cap((void **)&c->instances, &c->instance_cap, c->ninstances + 1, sizeof(Instance), MAX_EXPRS)) {
        resource_diag(c, "ORC0209", 0, 0, "semantic analysis could not retain function instances");
        return 0;
    }
    c->instances[c->ninstances++] = *inst;
    return 1;
}

static int push_iparam(Compiler *c, const InstParam *param) {
    if (!ensure_cap((void **)&c->iparams, &c->iparam_cap, c->niparams + 1, sizeof(InstParam), MAX_EXPRS)) {
        resource_diag(c, "ORC0209", 0, 0, "semantic analysis could not retain function instances");
        return 0;
    }
    c->iparams[c->niparams++] = *param;
    return 1;
}

int decode_size_bound(Compiler *c, uint32_t start, uint32_t end, int64_t *out, int *too_big) {
    Big magnitude = big_zero();
    *too_big = 0;
    *out = 0;
    if (!big_from_digits(&c->arena, c->text + start, (size_t)(end - start), 0, &magnitude)) {
        add_diag(c, "ORC0205", start, end, "integer magnitude exceeds the 16384-significant-bit limit",
                 "this part of the size is too large", "the value is rejected rather than truncated or approximated", 2);
        return 0;
    }
    if (magnitude.negative || magnitude.nlimbs > 1 ||
        (magnitude.nlimbs == 1 && magnitude.limbs[0] > MAX_LOOP_BOUND)) {
        *too_big = 1;
        return 1;
    }
    if (magnitude.nlimbs == 1) {
        *out = (int64_t)magnitude.limbs[0];
    }
    return 1;
}

static void report_repeated_size(Compiler *c, uint32_t start, uint32_t end, uint32_t earlier_start,
                                 uint32_t earlier_end, const char *earlier_label) {
    char name[64];
    char message[160];
    span_copy(name, sizeof name, c->text, start, end);
    snprintf(message, sizeof message, "duplicate parameter `%s`", name);
    add_diag(c, "ORC0218", start, end, message, "this name is already a size parameter",
             "size parameters and parameters share one namespace, and each name is unique", 2);
    diag_add_secondary(c, earlier_start, earlier_end, earlier_label);
}

/* Ranges, distinct names, and the 256-instance cap. A function in error here
   has no instances, so its body is not checked. */
static void admit_sizes(Compiler *c, Func *func) {
    uint8_t slot;
    uint64_t instances = 1;
    int valid = 1;
    if (tp_func_has_types(func)) {
        tp_admit(c, func);
        return;
    }
    if (func->nsizes == 0) {
        func->sizes_ok = 1;
        return;
    }
    for (slot = 0; slot < func->nsizes; slot++) {
        int lo_big = 0;
        int hi_big = 0;
        int lo_ok = decode_size_bound(c, func->sz_a0[slot], func->sz_a1[slot], &func->sz_lo[slot], &lo_big);
        int hi_ok = decode_size_bound(c, func->sz_b0[slot], func->sz_b1[slot], &func->sz_hi[slot], &hi_big);
        uint8_t earlier;
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
        for (earlier = 0; earlier < slot; earlier++) {
            if (same_span(c, func->sz_name0[earlier], func->sz_name1[earlier], func->sz_name0[slot],
                          func->sz_name1[slot])) {
                valid = 0;
                report_repeated_size(c, func->sz_name0[slot], func->sz_name1[slot], func->sz_name0[earlier],
                                     func->sz_name1[earlier], "first size parameter is here");
                break;
            }
        }
    }
    for (slot = 0; slot < func->nparams; slot++) {
        Param *param = &c->params[func->param0 + slot];
        uint8_t size_slot;
        if (size_slot_of(c, (uint32_t)(func - c->funcs), param->name_start, param->name_end, &size_slot)) {
            char name[64];
            char message[160];
            valid = 0;
            span_copy(name, sizeof name, c->text, param->name_start, param->name_end);
            snprintf(message, sizeof message, "duplicate parameter `%s`", name);
            add_diag(c, "ORC0218", param->name_start, param->name_end, message, "this name is already a size parameter",
                     "size parameters and parameters share one namespace, and each name is unique", 2);
            diag_add_secondary(c, func->sz_name0[size_slot], func->sz_name1[size_slot], "the size parameter is here");
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
                 SIZE_RANGE_NOTE, 2);
    }
    func->sizes_ok = valid;
}

static void instance_values(const Func *func, uint32_t ordinal, int64_t *out) {
    uint32_t rest = ordinal;
    int slot;
    for (slot = (int)func->nsizes - 1; slot >= 0; slot--) {
        uint32_t width = (uint32_t)(func->sz_hi[slot] - func->sz_lo[slot]);
        uint32_t offset = width == 0 ? 0u : rest % width;
        rest = width == 0 ? 0u : rest / width;
        out[slot] = func->sz_lo[slot] + (int64_t)offset;
    }
}

static int capture_instance(Compiler *c, uint32_t func_index, Instance *inst) {
    Func *func = &c->funcs[func_index];
    Applied result;
    uint16_t param;
    memset(inst, 0, sizeof *inst);
    inst->func = func_index;
    memcpy(inst->sz, c->cur_sz, sizeof inst->sz);
    if (!apply_site(c, func->result_site, 0, &result)) {
        return 0;
    }
    inst->result = result.kind;
    inst->result_len = result.length;
    inst->result_mod = result.mod_index;
    inst->result_ok = result.ok;
    inst->tup0 = result.tup0;
    inst->tup_n = result.tup_n;
    inst->param0 = c->niparams;
    inst->signature_ok = result.ok;
    for (param = 0; param < func->nparams; param++) {
        Applied applied;
        InstParam shape;
        memset(&shape, 0, sizeof shape);
        if (!apply_site(c, c->params[func->param0 + param].site, 0, &applied)) {
            return 0;
        }
        shape.type = applied.kind;
        shape.length = applied.length;
        shape.mod_index = applied.mod_index;
        shape.type_ok = applied.ok;
        shape.tup0 = applied.tup0;
        shape.tup_n = applied.tup_n;
        if (!applied.ok) {
            inst->signature_ok = 0;
        }
        if (!push_iparam(c, &shape)) {
            return 0;
        }
    }
    return 1;
}

static int live_apply(Compiler *c, uint32_t func_index) {
    Func *func = &c->funcs[func_index];
    Applied applied;
    uint16_t param;
    uint16_t local;
    uint32_t index;
    if (func->result_site != UINT32_MAX && func->result_site < c->nsites) {
        if (!apply_site(c, func->result_site, 1, &applied)) {
            return 0;
        }
        func->result = applied.kind;
        func->result_len = applied.length;
        func->result_ok = applied.ok;
        func->result_mod = applied.mod_index;
        func->result_reported = applied.reported;
        func->tup0 = applied.tup0;
        func->tup_n = applied.tup_n;
    }
    for (param = 0; param < func->nparams; param++) {
        Param *item = &c->params[func->param0 + param];
        if (item->site == UINT32_MAX || item->site >= c->nsites) {
            continue;
        }
        if (!apply_site(c, item->site, 1, &applied)) {
            return 0;
        }
        item->type = applied.kind;
        item->length = applied.length;
        item->type_ok = applied.ok;
        item->mod_index = applied.mod_index;
        item->type_reported = applied.reported;
        item->tup0 = applied.tup0;
        item->tup_n = applied.tup_n;
    }
    for (local = 0; local < func->nlocals; local++) {
        Local *item = &c->locals[func->local0 + local];
        if (item->site == UINT32_MAX || item->site >= c->nsites) {
            continue;
        }
        if (!apply_site(c, item->site, 1, &applied)) {
            return 0;
        }
        item->type = applied.kind;
        item->length = applied.length;
        item->type_ok = applied.ok;
        item->mod_index = applied.mod_index;
        item->type_reported = applied.reported;
        item->tup0 = applied.tup0;
        item->tup_n = applied.tup_n;
    }
    if (func->nlocals > 0) {
        seal_patterns(c, &c->locals[func->local0], func->nlocals);
    }
    for (index = 0; index < c->nblock_locals; index++) {
        Local *item = &c->block_locals[index];
        if (item->site == UINT32_MAX || item->site >= c->nsites || c->sites[item->site].owner_func != func_index) {
            continue;
        }
        if (!apply_site(c, item->site, 1, &applied)) {
            return 0;
        }
        item->type = applied.kind;
        item->length = applied.length;
        item->type_ok = applied.ok;
        item->mod_index = applied.mod_index;
        item->type_reported = applied.reported;
        item->tup0 = applied.tup0;
        item->tup_n = applied.tup_n;
    }
    for (index = 0; index < c->nloops; index++) {
        LoopDesc *loop = &c->loops[index];
        if (loop->site == UINT32_MAX || loop->site >= c->nsites || c->sites[loop->site].owner_func != func_index) {
            continue;
        }
        if (!apply_site(c, loop->site, 1, &applied)) {
            return 0;
        }
        loop->acc_type = applied.kind;
        loop->acc_len = applied.length;
        loop->acc_ok = applied.ok;
        loop->acc_mod = applied.mod_index;
        loop->acc_reported = applied.reported;
        loop->tup0 = applied.tup0;
        loop->tup_n = applied.tup_n;
    }
    tp_refresh_convs(c, func_index);
    return 1;
}

static int build_instances(Compiler *c, uint32_t func_index) {
    Func *func = &c->funcs[func_index];
    uint32_t count = 1;
    uint32_t ordinal;
    if (!func->typed) {
        return 1;
    }
    admit_sizes(c, func);
    if (func->nsizes > 0 && !func->sizes_ok) {
        func->ninst = 0;
        return 1;
    }
    if (func->nsizes > 0) {
        uint64_t product = 1;
        uint8_t slot;
        for (slot = 0; slot < func->nsizes; slot++) {
            product *= (uint64_t)(func->sz_hi[slot] - func->sz_lo[slot]);
        }
        if (product == 0 || product > MAX_INSTANCES) {
            func->ninst = 0;
            func->sizes_ok = 0;
            return 1;
        }
        count = (uint32_t)product;
    }
    func->inst0 = c->ninstances;
    func->ninst = (uint16_t)count;
    c->cur_func = func_index;
    for (ordinal = 0; ordinal < count; ordinal++) {
        Instance inst;
        memset(c->cur_sz, 0, sizeof c->cur_sz);
        if (func->nsizes > 0) {
            instance_values(func, ordinal, c->cur_sz);
        }
        c->ncur = func->nsizes;
        if (!capture_instance(c, func_index, &inst) || !push_instance(c, &inst)) {
            return 0;
        }
    }
    c->ncur = 0;
    c->cur_func = UINT32_MAX;
    return 1;
}

static void instance_label(const Compiler *c, uint32_t inst, char *buf, size_t cap) {
    const Instance *instance;
    const Func *func;
    size_t used;
    uint8_t slot;
    if (cap == 0) {
        return;
    }
    buf[0] = '\0';
    if (inst >= c->ninstances) {
        return;
    }
    instance = &c->instances[inst];
    func = &c->funcs[instance->func];
    if (tp_func_has_types(func)) {
        tp_format_label(c, inst, buf, cap);
        return;
    }
    used = func->name_end - func->name_start;
    if (used >= cap) {
        used = cap - 1;
    }
    memcpy(buf, c->text + func->name_start, used);
    buf[used] = '\0';
    if (func->nsizes == 0 || used + 3 >= cap) {
        return;
    }
    buf[used++] = '[';
    for (slot = 0; slot < func->nsizes; slot++) {
        char num[32];
        int n = snprintf(num, sizeof num, "%s%lld", slot == 0 ? "" : ", ", (long long)instance->sz[slot]);
        if (n < 0 || used + (size_t)n + 2 >= cap) {
            break;
        }
        memcpy(buf + used, num, (size_t)n);
        used += (size_t)n;
    }
    buf[used++] = ']';
    buf[used] = '\0';
}

static void name_sized_diags(Compiler *c, const Func *func, uint32_t inst, uint32_t from) {
    char label[96];
    char name[64];
    char note[320];
    uint32_t index;
    if (tp_func_has_types(func)) {
        tp_name_diags(c, func, inst, from);
        return;
    }
    if (func->nsizes == 0) {
        return;
    }
    instance_label(c, inst, label, sizeof label);
    span_copy(name, sizeof name, c->text, func->name_start, func->name_end);
    snprintf(note, sizeof note,
             "in the instance `%s`, the first of `%s` in error: a sized function is checked once for each value of "
             "its sizes",
             label, name);
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

static void report_call_cycle(Compiler *c, const uint32_t *stack, uint32_t top, uint32_t callee, uint32_t start,
                              uint32_t end) {
    char names[8][96];
    char message[384];
    uint32_t pos = 0;
    uint32_t count;
    uint32_t index;
    size_t used = 0;
    while (pos < top && stack[pos] != callee) {
        pos++;
    }
    count = top - pos + 1;
    if (count <= 1) {
        instance_label(c, callee, names[0], sizeof names[0]);
        snprintf(message, sizeof message, "`%s` calls itself", names[0]);
    } else {
        uint32_t shown = count > 6 ? 6 : count;
        used = (size_t)snprintf(message, sizeof message, "call cycle ");
        for (index = 0; index < shown && used < sizeof message; index++) {
            char piece[120];
            int n;
            instance_label(c, stack[pos + index], names[0], sizeof names[0]);
            n = snprintf(piece, sizeof piece, "%s`%s`", index == 0 ? "" : " -> ", names[0]);
            if (n < 0 || used + (size_t)n >= sizeof message) {
                break;
            }
            memcpy(message + used, piece, (size_t)n);
            used += (size_t)n;
            message[used] = '\0';
        }
        if (count > 6 && used + 7 < sizeof message) {
            memcpy(message + used, " -> ...", 7);
            used += 7;
            message[used] = '\0';
        }
        instance_label(c, callee, names[0], sizeof names[0]);
        snprintf(message + used, sizeof message - used, " -> `%s`", names[0]);
    }
    add_diag(c, "ORC0217", start, end, message, "this call closes the cycle",
             "a `spec` may not depend on itself; recursion is not part of Orange 2026", 2);
}

static int cycle_span_reported(const Compiler *c, uint32_t start, uint32_t end) {
    uint32_t index;
    for (index = 0; index < c->ndiags; index++) {
        if (strcmp(c->diags[index].code, "ORC0217") == 0 && c->diags[index].start == start &&
            c->diags[index].end == end) {
            return 1;
        }
    }
    return 0;
}

static void analyze(Compiler *c) {
    uint32_t index;
    prepare_types(c);
    for (index = 0; index < c->nfuncs; index++) {
        if (!build_instances(c, index)) {
            return;
        }
    }
    c->cur_func = UINT32_MAX;
    c->cur_inst = UINT32_MAX;
    c->ncur = 0;
    for (index = 0; index < c->nfuncs; index++) {
        Func *func = &c->funcs[index];
        uint16_t param_index;
        uint16_t local_index;
        c->nfinished = 0;
        c->nframes = 0;
        for (uint32_t previous = 0; previous < index; previous++) {
            Func *earlier = &c->funcs[previous];
            if (earlier->is_impl == func->is_impl &&
                same_span(c, earlier->name_start, earlier->name_end, func->name_start, func->name_end)) {
                char message[128];
                char name[64];
                size_t length = func->name_end - func->name_start;
                if (length >= sizeof name) {
                    length = sizeof name - 1;
                }
                memcpy(name, c->text + func->name_start, length);
                name[length] = '\0';
                snprintf(message, sizeof message, "duplicate %s function `%s`", func->is_impl ? "impl" : "spec", name);
                add_diag(c, "ORC0201", func->name_start, func->name_end, message,
                         "this declaration repeats a name in the same namespace",
                         "`spec` and `impl` use separate declaration namespaces", 2);
                diag_add_secondary(c, earlier->name_start, earlier->name_end, "first declaration is here");
                func->duplicate = 1;
                break;
            }
        }
        if (!func->typed) {
            continue;
        }
        if (func->nsizes > 0 && !func->sizes_ok) {
            continue;
        }
        {
        uint32_t passes = func->ninst > 0 ? func->ninst : 1u;
        uint32_t pass;
        for (pass = 0; pass < passes; pass++) {
        uint32_t diags_before = c->ndiags;
        c->cur_func = index;
        if (func->ninst > 0) {
            c->cur_inst = func->inst0 + pass;
            memcpy(c->cur_sz, c->instances[c->cur_inst].sz, sizeof c->cur_sz);
            c->ncur = func->nsizes;
            if (!live_apply(c, index)) {
                return;
            }
        } else {
            c->cur_inst = UINT32_MAX;
            c->ncur = 0;
        }
        func->signature_ok = func->result_ok;
        for (param_index = 0; param_index < func->nparams; param_index++) {
            Param *param = &c->params[func->param0 + param_index];
            if (pass == 0) {
            for (uint16_t earlier = 0; earlier < param_index; earlier++) {
                Param *before = &c->params[func->param0 + earlier];
                if (!before->duplicate &&
                    same_span(c, before->name_start, before->name_end, param->name_start, param->name_end)) {
                    char message[128];
                    char name[64];
                    size_t length = param->name_end - param->name_start;
                    if (length >= sizeof name) {
                        length = sizeof name - 1;
                    }
                    memcpy(name, c->text + param->name_start, length);
                    name[length] = '\0';
                    snprintf(message, sizeof message, "duplicate parameter `%s`", name);
                    add_diag(c, "ORC0218", param->name_start, param->name_end, message,
                             "this parameter repeats an earlier name",
                             "parameter names must be unique within one function", 2);
                    diag_add_secondary(c, before->name_start, before->name_end, "first parameter is here");
                    param->duplicate = 1;
                    break;
                }
            }
            }
            if (!param->type_ok && !param->type_reported) {
                reject_declared(c, param->type, param->length_bad, param->type_start, param->type_end,
                                param->length_start, param->length_end);
                func->signature_ok = 0;
            }
        }
        if (!func->result_ok) {
            /* A rejected result is already diagnosed at its type. Do not also
               check the bindings or the body against that type (Float and
               Mod[1] must not add ORC0207 or ORC0211). A result the parser
               rejects, such as a tuple of tuples, still has its body parsed,
               so a repeated syntax error there is reported on its own. */
            if (!func->result_reported) {
                reject_declared(c, func->result, func->result_length_bad, func->result_start, func->result_end,
                                func->result_length_start, func->result_length_end);
            }
            name_sized_diags(c, func, c->cur_inst, diags_before);
            func->signature_ok = 0;
            break;
        }
        for (local_index = 0; local_index < func->nlocals; local_index++) {
            Local *local = &c->locals[func->local0 + local_index];
            int hidden = 0;
            if (local->pat_i > 0) {
                continue;
            }
            if (local->pat_len > 0) {
                int bad_type = 0;
                uint16_t pat;
                for (pat = 0; pat < local->pat_len; pat++) {
                    Local *name = &c->locals[func->local0 + local_index + pat];
                    int taken = 0;
                    uint16_t prev;
                    for (param_index = 0; param_index < func->nparams; param_index++) {
                        Param *param = &c->params[func->param0 + param_index];
                        if (!param->duplicate &&
                            same_span(c, param->name_start, param->name_end, name->name_start, name->name_end)) {
                            taken = 1;
                            break;
                        }
                    }
                    for (prev = 0; prev < local_index && !taken; prev++) {
                        Local *before = &c->locals[func->local0 + prev];
                        if (!before->duplicate &&
                            same_span(c, before->name_start, before->name_end, name->name_start, name->name_end)) {
                            taken = 1;
                        }
                    }
                    for (prev = 0; prev < pat && !taken; prev++) {
                        Local *before = &c->locals[func->local0 + local_index + prev];
                        if (!before->duplicate &&
                            same_span(c, before->name_start, before->name_end, name->name_start, name->name_end)) {
                            taken = 1;
                        }
                    }
                    if (!taken && size_slot_of(c, index, name->name_start, name->name_end, NULL)) {
                        taken = 1;
                    }
                    if (pass == 0 && taken) {
                        uint32_t within_start = 0;
                        uint32_t within_end = 0;
                        for (prev = 0; prev < pat; prev++) {
                            Local *before = &c->locals[func->local0 + local_index + prev];
                            if (!before->duplicate &&
                                same_span(c, before->name_start, before->name_end, name->name_start, name->name_end)) {
                                within_start = before->name_start;
                                within_end = before->name_end;
                                break;
                            }
                        }
                        report_binding_dup(c, index, local_index, name->name_at, name->name_end_at, name->name_start,
                                           name->name_end, within_start, within_end);
                        name->duplicate = 1;
                    }
                    if (!name->type_ok) {
                        bad_type = 1;
                    }
                }
                if (bad_type) {
                    for (pat = 0; pat < local->pat_len; pat++) {
                        Local *name = &c->locals[func->local0 + local_index + pat];
                        int unresolved = !name->type_ok;
                        name->type_ok = 0;
                        if (unresolved && !name->type_reported) {
                            reject_declared(c, name->type, name->length_bad, name->type_start, name->type_end,
                                            name->length_start, name->length_end);
                        }
                        name->type_reported = 1;
                    }
                    continue;
                }
                check_as_tuple(c, local->value, local->tup0, local->tup_n, index, local_index);
                continue;
            }
            for (param_index = 0; param_index < func->nparams; param_index++) {
                Param *param = &c->params[func->param0 + param_index];
                if (!param->duplicate &&
                    same_span(c, param->name_start, param->name_end, local->name_start, local->name_end)) {
                    hidden = 1;
                    break;
                }
            }
            for (uint16_t earlier = 0; earlier < local_index && !hidden; earlier++) {
                Local *before = &c->locals[func->local0 + earlier];
                if (!before->duplicate &&
                    same_span(c, before->name_start, before->name_end, local->name_start, local->name_end)) {
                    hidden = 1;
                }
            }
            if (!hidden && size_slot_of(c, index, local->name_start, local->name_end, NULL)) {
                hidden = 1;
            }
            if (pass == 0 && hidden) {
                report_binding_dup(c, index, local_index, local->name_at, local->name_end_at, local->name_start,
                                   local->name_end, 0, 0);
                local->duplicate = 1;
            }
            if (!local->type_ok) {
                /* A rejected binding type is already diagnosed. Do not also
                   typecheck its initializer (Float = 1 must not add ORC0207). */
                if (!local->type_reported) {
                    reject_declared(c, local->type, local->length_bad, local->type_start, local->type_end,
                                    local->length_start, local->length_end);
                }
                continue;
            }
            if (local->type == TY_TUPLE) {
                check_as_tuple(c, local->value, local->tup0, local->tup_n, index, local_index);
            } else {
                check_at(c, local->value, local->type, local->length, local->mod_index, index, local_index);
            }
        }
        if (func->body != UINT32_MAX) {
            if (func->result == TY_TUPLE) {
                check_as_tuple(c, func->body, func->tup0, func->tup_n, index, func->nlocals);
            } else {
                check_at(c, func->body, func->result, func->result_len, func->result_mod, index, func->nlocals);
            }
        }
        if (c->ndiags > diags_before) {
            name_sized_diags(c, func, c->cur_inst, diags_before);
            func->signature_ok = 0;
            break;
        }
        }
        }
        c->cur_func = UINT32_MAX;
        c->cur_inst = UINT32_MAX;
        c->ncur = 0;
    }
    {
        uint32_t nodes = c->ninstances;
        uint8_t *color = calloc(nodes ? nodes : 1, 1);
        uint32_t *stack = calloc(nodes ? nodes : 1, sizeof(uint32_t));
        if (color == NULL || stack == NULL) {
            resource_diag(c, "ORC0209", 0, 0, "semantic analysis could not retain the call graph");
            free(color);
            free(stack);
            return;
        }
        for (index = 0; index < nodes; index++) {
            uint32_t top = 0;
            uint32_t *edge_at = calloc(nodes ? nodes : 1, sizeof(uint32_t));
            if (edge_at == NULL) {
                free(color);
                free(stack);
                resource_diag(c, "ORC0209", 0, 0, "semantic analysis could not retain the call graph");
                return;
            }
            if (color[index] != 0) {
                free(edge_at);
                continue;
            }
            stack[top] = index;
            color[index] = 1;
            edge_at[index] = 0;
            while (top != UINT32_MAX) {
                uint32_t current = stack[top];
                Func *func = &c->funcs[c->instances[current].func];
                uint32_t seen = 0;
                uint32_t edge_index;
                int advanced = 0;
                for (edge_index = 0; edge_index < func->nedges; edge_index++) {
                    Edge edge = c->edges[func->edge0 + edge_index];
                    if (edge.caller_inst != current || edge.callee_inst >= nodes) {
                        continue;
                    }
                    if (seen < edge_at[current]) {
                        seen++;
                        continue;
                    }
                    edge_at[current]++;
                    advanced = 1;
                    if (color[edge.callee_inst] == 1) {
                        if (!cycle_span_reported(c, edge.start, edge.end)) {
                            report_call_cycle(c, stack, top, edge.callee_inst, edge.start, edge.end);
                        }
                    } else if (color[edge.callee_inst] == 0) {
                        color[edge.callee_inst] = 1;
                        edge_at[edge.callee_inst] = 0;
                        stack[++top] = edge.callee_inst;
                    }
                    break;
                }
                if (!advanced) {
                    color[current] = 2;
                    if (top == 0) {
                        break;
                    }
                    top--;
                }
            }
            free(edge_at);
        }
        free(color);
        free(stack);
    }
}

static void value_clear(Value *value) {
    Value *elems;
    Pack *pack;
    uint32_t length;
    uint32_t index;
    if (value == NULL) {
        return;
    }
    elems = value->elems;
    pack = value->pack;
    length = value->length;
    memset(value, 0, sizeof *value);
    if (pack != NULL) {
        pack_release(pack);
    }
    if (elems == NULL) {
        return;
    }
    for (index = 0; index < length; index++) {
        value_clear(&elems[index]);
    }
    free(elems);
}

static void value_list_clear(Value *items, uint32_t count) {
    uint32_t index;
    if (items == NULL) {
        return;
    }
    for (index = 0; index < count; index++) {
        value_clear(&items[index]);
    }
    free(items);
}

static void value_move(Value *dst, Value *src) {
    if (dst == src) {
        return;
    }
    value_clear(dst);
    *dst = *src;
    memset(src, 0, sizeof *src);
}

static int value_clone(Compiler *c, Value *dst, const Value *src, uint32_t start, uint32_t end) {
    Value *elems;
    uint32_t index;
    if (c->failed) {
        return 0;
    }
    value_clear(dst);
    if (src->pack != NULL) {
        if (!pack_retain(src->pack)) {
            c->failed = 1;
            add_diag(c, "ORC0301", start, end, "evaluation could not retain an array", "resource limit reached", NULL, 2);
            return 0;
        }
        *dst = *src;
        dst->elems = NULL;
        return 1;
    }
    if (src->length == 0) {
        *dst = *src;
        dst->elems = NULL;
        dst->pack = NULL;
        return 1;
    }
    if (src->elems == NULL) {
        c->failed = 1;
        return 0;
    }
    elems = calloc(src->length, sizeof(Value));
    if (elems == NULL) {
        c->failed = 1;
        add_diag(c, "ORC0301", start, end, "evaluation could not retain an array", "resource limit reached", NULL, 2);
        return 0;
    }
    for (index = 0; index < src->length; index++) {
        if (!value_clone(c, &elems[index], &src->elems[index], start, end)) {
            value_list_clear(elems, src->length);
            return 0;
        }
    }
    *dst = *src;
    dst->elems = elems;
    dst->length = src->length;
    return 1;
}

static int alloc_array(Compiler *c, Value **items, uint32_t length, uint32_t start, uint32_t end) {
    if (c->failed || length == 0) {
        c->failed = 1;
        return 0;
    }
    *items = calloc(length, sizeof(Value));
    if (*items == NULL) {
        c->failed = 1;
        add_diag(c, "ORC0301", start, end, "evaluation could not retain an array", "resource limit reached", NULL, 2);
        return 0;
    }
    return 1;
}

/* A charge that would pass the budget is not recorded. The diagnostic is
   reported once, on the function being evaluated, not on this expression. */
static int charge(Compiler *c, uint32_t start, uint32_t end, uint64_t cost) {
    uint64_t limit = c->step_limit == 0 ? MAX_STEPS : c->step_limit;
    (void)start;
    (void)end;
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

static int eval_expr(Compiler *c, uint32_t index, Value *params, Value *locals, int depth, Value *out);

static int eval_pattern(Compiler *c, const Local *head, Value *slot0, Value *params, Value *locals, int depth) {
    Value tuple;
    uint16_t index;
    memset(&tuple, 0, sizeof tuple);
    if (head->value == UINT32_MAX || !eval_expr(c, head->value, params, locals, depth, &tuple)) {
        value_clear(&tuple);
        return 0;
    }
    if (!tuple.is_tuple || tuple.elems == NULL || tuple.length < head->pat_len) {
        c->failed = 1;
        value_clear(&tuple);
        return 0;
    }
    for (index = 0; index < head->pat_len; index++) {
        if (!value_clone(c, &slot0[index], &tuple.elems[index], head->name_start, head->name_end)) {
            value_clear(&tuple);
            return 0;
        }
    }
    value_clear(&tuple);
    return 1;
}

static int eval_block(Compiler *c, uint32_t bind0, uint16_t nbinds, uint32_t value, Value *params, Value *locals,
                      int depth, Value *out) {
    BlockFrame *frame;
    Value *slots;
    uint16_t bind;
    int ok = 1;
    if (nbinds == 0) {
        return eval_expr(c, value, params, locals, depth, out);
    }
    if (c->nframes >= MAX_OPEN_LOOPS) {
        c->failed = 1;
        add_diag(c, "ORC0301", c->exprs[value].start, c->exprs[value].end, "evaluation could not retain block bindings",
                 "resource limit reached", NULL, 2);
        return 0;
    }
    slots = calloc(nbinds, sizeof(Value));
    if (slots == NULL) {
        c->failed = 1;
        add_diag(c, "ORC0301", c->exprs[value].start, c->exprs[value].end, "evaluation could not retain block bindings",
                 "resource limit reached", NULL, 2);
        return 0;
    }
    frame = &c->frames[c->nframes++];
    frame->bind0 = bind0;
    frame->nbinds = nbinds;
    frame->visible = 0;
    frame->slots = slots;
    for (bind = 0; bind < nbinds && ok; bind++) {
        const Local *local = &c->block_locals[bind0 + bind];
        if (local->pat_i > 0) {
            continue;
        }
        if (local->pat_len > 0) {
            ok = eval_pattern(c, local, &slots[bind], params, locals, depth);
            frame->visible = (uint16_t)(bind + local->pat_len);
            bind = (uint16_t)(bind + local->pat_len - 1);
            continue;
        }
        ok = eval_expr(c, local->value, params, locals, depth, &slots[bind]);
        frame->visible = (uint16_t)(bind + 1);
    }
    if (ok) {
        ok = eval_expr(c, value, params, locals, depth, out);
    }
    for (bind = 0; bind < nbinds; bind++) {
        value_clear(&slots[bind]);
    }
    free(slots);
    c->nframes--;
    return ok;
}

static int ensure_loops(Compiler *c) {
    if (c->nloops == 0 || c->loop_k != NULL) {
        return 1;
    }
    c->loop_k = calloc(c->nloops, sizeof *c->loop_k);
    c->loop_acc = calloc(c->nloops, sizeof *c->loop_acc);
    if (c->loop_k == NULL || c->loop_acc == NULL) {
        resource_diag(c, "ORC0106", 0, 0, "evaluation could not retain loop state");
        return 0;
    }
    return 1;
}

static int eval_function(Compiler *c, uint32_t func_index, Value *arguments, int depth, Value *out) {
    Func *func = &c->funcs[func_index];
    Value *params;
    Value *locals;
    uint16_t index;
    int ok;
    if (tp_func_has_types(func)) {
        if (!live_apply(c, func_index)) {
            c->failed = 1;
            return 0;
        }
        tp_apply_stamps(c);
    }
    if (!ensure_loops(c)) {
        c->failed = 1;
        return 0;
    }
    if (depth > MAX_CALL_DEPTH) {
        c->failed = 1;
        add_diag(c, "ORC0301", func->name_start, func->name_end, "evaluation exceeded the call depth",
                 "call depth limit reached", "a source may nest at most 256 calls", 2);
        return 0;
    }
    params = calloc(func->nparams ? func->nparams : 1, sizeof(Value));
    locals = calloc(func->nlocals ? func->nlocals : 1, sizeof(Value));
    if (params == NULL || locals == NULL) {
        free(params);
        free(locals);
        c->failed = 1;
        add_diag(c, "ORC0301", func->name_start, func->name_end, "evaluation could not retain a call frame",
                 "resource limit reached", NULL, 2);
        return 0;
    }
    ok = 1;
    for (index = 0; index < func->nparams && ok; index++) {
        ok = value_clone(c, &params[index], &arguments[index], func->name_start, func->name_end);
    }
    for (index = 0; index < func->nlocals && ok; index++) {
        const Local *local = &c->locals[func->local0 + index];
        if (local->pat_i > 0) {
            continue;
        }
        if (local->pat_len > 0) {
            ok = eval_pattern(c, local, &locals[index], params, locals, depth);
            continue;
        }
        ok = eval_expr(c, local->value, params, locals, depth, &locals[index]);
    }
    if (ok) {
        ok = eval_expr(c, func->body, params, locals, depth, out);
    }
    for (index = 0; index < func->nparams; index++) {
        value_clear(&params[index]);
    }
    for (index = 0; index < func->nlocals; index++) {
        value_clear(&locals[index]);
    }
    free(params);
    free(locals);
    return ok;
}

static uint64_t word_mask_of(int width) {
    return width >= 64 ? UINT64_MAX : (UINT64_C(1) << width) - 1;
}

static int relation_holds(TokenKind op, int ordering) {
    switch (op) {
    case TK_EQEQ: return ordering == 0;
    case TK_BANGEQ: return ordering != 0;
    case TK_LESS: return ordering < 0;
    case TK_GREATER: return ordering > 0;
    case TK_LESSEQ: return ordering <= 0;
    case TK_GREATEREQ: return ordering >= 0;
    default: return 0;
    }
}

static int word_ordering(uint64_t left, uint64_t right) {
    if (left < right) {
        return -1;
    }
    if (left > right) {
        return 1;
    }
    return 0;
}

static int binary_words(TokenKind op, uint64_t left, uint64_t right, int width, uint64_t *out) {
    uint64_t mask = word_mask_of(width);
    uint64_t a0;
    uint64_t a1;
    uint64_t b0;
    uint64_t b1;
    uint64_t p0;
    uint64_t p1;
    uint64_t p2;
    uint64_t mid;
    left &= mask;
    right &= mask;
    switch (op) {
    case TK_PLUS: *out = (left + right) & mask; return 1;
    case TK_MINUS: *out = (left - right) & mask; return 1;
    case TK_STAR:
        a0 = (uint32_t)left;
        a1 = left >> 32;
        b0 = (uint32_t)right;
        b1 = right >> 32;
        p0 = a0 * b0;
        p1 = a0 * b1;
        p2 = a1 * b0;
        mid = (p0 >> 32) + (uint32_t)p1 + (uint32_t)p2;
        *out = ((p0 & 0xffffffffu) | (mid << 32)) & mask;
        return 1;
    case TK_AMP: *out = left & right; return 1;
    case TK_PIPE: *out = left | right; return 1;
    case TK_CARET: *out = left ^ right; return 1;
    default: return 0;
    }
}

static int eval_expr_in(Compiler *c, uint32_t index, Value *params, Value *locals, int depth, Value *out);

static uint32_t modulus_digits(const Big *modulus) {
    uint32_t bits = big_bits(modulus);
    return bits == 0 ? 1u : (bits + 31u) / 32u;
}

static const Big *modulus_at(const Compiler *c, uint16_t index) {
    if (c->moduli == NULL || index == 0 || index >= c->nmoduli) {
        return NULL;
    }
    return &c->moduli[index];
}

static int array_oom(Compiler *c, uint32_t start, uint32_t end) {
    c->failed = 1;
    add_diag(c, "ORC0301", start, end, "evaluation could not retain an array", "resource limit reached", NULL, 2);
    return 0;
}

static int scalar_word(const Value *value, uint64_t *word) {
    if (value->length != 0 || value->pack != NULL || value->is_tuple) {
        return 0;
    }
    if (value->type == TY_INT || value->type == TY_MOD) {
        if (value->big.negative || value->big.nlimbs > 2) {
            return 0;
        }
        *word = 0;
        if (value->big.nlimbs > 0) {
            *word = value->big.limbs[0];
        }
        if (value->big.nlimbs > 1) {
            *word |= (uint64_t)value->big.limbs[1] << 32;
        }
        return 1;
    }
    if (type_width(value->type) > 0) {
        *word = value->word;
        return 1;
    }
    return 0;
}

static int scalar_from_word(Compiler *c, Value *out, TypeKind type, uint16_t mod_index, uint64_t word) {
    memset(out, 0, sizeof *out);
    out->type = type;
    out->mod_index = mod_index;
    out->big = big_zero();
    if (type == TY_MOD || type == TY_INT) {
        return big_from_u64(&c->arena, word, &out->big);
    }
    out->word = word;
    return 1;
}

static uint32_t array_stride(Compiler *c, TypeKind type, uint16_t mod_index) {
    uint32_t bits = 0;
    if (type == TY_MOD) {
        const Big *modulus = modulus_at(c, mod_index);
        if (modulus == NULL) {
            return 0;
        }
        bits = big_bits(modulus);
    }
    return pack_stride_of(type, bits);
}

static void transfer_array(Value *out, Value *base) {
    out->type = base->type;
    out->length = base->length;
    out->mod_index = base->mod_index;
    out->is_tuple = 0;
    out->pack = base->pack;
    out->elems = base->elems;
    out->word = 0;
    out->big = big_zero();
    base->pack = NULL;
    base->elems = NULL;
    base->length = 0;
}

static int elem_word(const Value *array, uint32_t index, uint64_t *word) {
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
    return scalar_word(&array->elems[index], word);
}

static int residue_reduce(Compiler *c, const Big *value, const Big *modulus, Big *out) {
    Big quot = big_zero();
    Big rem = big_zero();
    if (!value->negative && big_cmp(value, modulus) < 0) {
        *out = *value;
        return 1;
    }
    if (!big_div_euclid(&c->arena, value, modulus, &quot, &rem)) {
        return 0;
    }
    *out = rem;
    return 1;
}

/* Least residue r with value * r = 1 (mod m) when gcd is 1, else 0. */
static int residue_inverse(Compiler *c, const Big *value, const Big *modulus, Big *out) {
    Big prev_r = *value;
    Big prev_s = big_zero();
    Big cur_r = *modulus;
    Big cur_s = big_zero();
    Big one = big_zero();
    uint32_t guard = 0;
    if (!big_from_u64(&c->arena, 1, &prev_s) || !big_from_u64(&c->arena, 0, &cur_s) ||
        !big_from_u64(&c->arena, 1, &one)) {
        return 0;
    }
    while (cur_r.nlimbs != 0) {
        Big quot = big_zero();
        Big rem = big_zero();
        Big prod = big_zero();
        Big coeff = big_zero();
        if (++guard > 4096u || !big_div_euclid(&c->arena, &prev_r, &cur_r, &quot, &rem) ||
            !big_mul(&c->arena, &quot, &cur_s, &prod) || !big_sub(&c->arena, &prev_s, &prod, &coeff)) {
            return 0;
        }
        prev_r = cur_r;
        prev_s = cur_s;
        cur_r = rem;
        cur_s = coeff;
    }
    if (big_cmp(&prev_r, &one) != 0) {
        return big_from_u64(&c->arena, 0, out);
    }
    {
        Big quot = big_zero();
        Big rem = big_zero();
        if (!big_div_euclid(&c->arena, &prev_s, modulus, &quot, &rem)) {
            return 0;
        }
        *out = rem;
        return 1;
    }
}

static int residue_literal(Compiler *c, const Expr *expr, Big *out) {
    Big value = big_zero();
    const Big *modulus = modulus_at(c, expr->ty_mod);
    if (modulus == NULL || !decode_literal(c, expr, &value)) {
        return 0;
    }
    /* `decode_literal` already applied the sign. A negative literal denotes
       m - n, so subtract the magnitude, not the signed value. */
    value.negative = 0;
    if (expr->negative && value.nlimbs != 0) {
        return big_sub(&c->arena, modulus, &value, out);
    }
    *out = value;
    return 1;
}

/* A word index is the word's unsigned value, after one implicit conversion to Int. */
static int index_position(Compiler *c, const Value *index_value, uint32_t index_start, uint32_t index_end,
                          uint32_t *position) {
    if (index_value->type == TY_INT) {
        if (index_value->big.negative || index_value->big.nlimbs > 1) {
            c->failed = 1;
            return 0;
        }
        *position = index_value->big.nlimbs == 0 ? 0 : index_value->big.limbs[0];
        return 1;
    }
    if (type_width(index_value->type) == 0 || !charge(c, index_start, index_end, 1)) {
        c->failed = 1;
        return 0;
    }
    {
        uint64_t word = index_value->word & word_mask_of(type_width(index_value->type));
        if (word > UINT32_MAX) {
            *position = UINT32_MAX;
        } else {
            *position = (uint32_t)word;
        }
    }
    return 1;
}

/* Modulus indices are per module. A value crossing a call wears the index of
   the module that is about to use it. A shared pack is copied first so the
   caller's array keeps its own index. */
static int retag_modulus(Value *value, uint16_t mod_index) {
    uint32_t index;
    if (value->is_tuple) {
        if (value->elems == NULL) {
            return 1;
        }
        for (index = 0; index < value->length; index++) {
            if (!retag_modulus(&value->elems[index], mod_index)) {
                return 0;
            }
        }
        return 1;
    }
    if (value->pack != NULL) {
        if (value->pack->type == TY_MOD) {
            if (!pack_make_unique(&value->pack)) {
                return 0;
            }
            value->pack->mod_index = mod_index;
            value->mod_index = mod_index;
        }
        return 1;
    }
    if (value->length == 0) {
        if (value->type == TY_MOD) {
            value->mod_index = mod_index;
        }
        return 1;
    }
    if (value->elems == NULL) {
        return 1;
    }
    for (index = 0; index < value->length; index++) {
        if (value->elems[index].type == TY_MOD) {
            value->elems[index].mod_index = mod_index;
        }
    }
    return 1;
}

static int eval_expr(Compiler *c, uint32_t index, Value *params, Value *locals, int depth, Value *out) {
    int ok;
    memset(out, 0, sizeof *out);
    if (c->failed) {
        return 0;
    }
    ok = eval_expr_in(c, index, params, locals, depth, out);
    if (!ok) {
        value_clear(out);
    }
    return ok;
}

static int pack_place(uint8_t order, uint32_t index, uint32_t count, uint32_t *place) {
    if (index >= count || count == 0) {
        return 0;
    }
    if (order == 2) {
        *place = index;
        return 1;
    }
    *place = count - 1u - index;
    return 1;
}

static int pack_write_word(uint32_t *limbs, uint32_t nlimbs, uint32_t bits, uint32_t place, uint64_t word) {
    uint64_t offset = (uint64_t)bits * (uint64_t)place;
    uint32_t digit = (uint32_t)(offset / 32u);
    uint32_t shift = (uint32_t)(offset % 32u);
    if (bits == 0 || bits > 64 || digit >= nlimbs) {
        return 0;
    }
    if (bits < 64) {
        word &= (UINT64_C(1) << bits) - 1u;
    }
    limbs[digit] |= (uint32_t)word << shift;
    if (bits == 64) {
        if (digit + 1u >= nlimbs) {
            return 0;
        }
        limbs[digit + 1u] = (uint32_t)(word >> 32);
    }
    return 1;
}

static int pack_read_word(const uint32_t *limbs, uint32_t nlimbs, uint32_t bits, uint32_t place, uint64_t *word) {
    uint64_t offset = (uint64_t)bits * (uint64_t)place;
    uint32_t digit = (uint32_t)(offset / 32u);
    uint32_t shift = (uint32_t)(offset % 32u);
    uint64_t low;
    if (bits == 0 || bits > 64 || digit >= nlimbs) {
        return 0;
    }
    low = limbs[digit];
    if (bits == 64) {
        if (digit + 1u >= nlimbs) {
            return 0;
        }
        *word = low | ((uint64_t)limbs[digit + 1u] << 32);
        return 1;
    }
    *word = (low >> shift) & ((UINT64_C(1) << bits) - 1u);
    return 1;
}

/* Residue of `value` modulo 2^width, little-endian in `limbs`. A negative
   value's residue is its two's complement within that width. */
static void pack_write_residue(uint32_t *limbs, uint32_t nlimbs, const Big *value, uint32_t width) {
    uint32_t copy = value->nlimbs < nlimbs ? value->nlimbs : nlimbs;
    uint32_t index;
    memset(limbs, 0, (size_t)nlimbs * sizeof(uint32_t));
    if (copy > 0 && value->limbs != NULL) {
        memcpy(limbs, value->limbs, (size_t)copy * sizeof(uint32_t));
    }
    if (value->negative) {
        uint32_t carry = 1;
        for (index = 0; index < nlimbs; index++) {
            uint64_t sum = (uint64_t)(~limbs[index]) + carry;
            limbs[index] = (uint32_t)sum;
            carry = (uint32_t)(sum >> 32);
        }
    }
    if (nlimbs > 0 && (width % 32u) != 0) {
        uint32_t spare = width % 32u;
        limbs[nlimbs - 1u] &= (UINT32_C(1) << spare) - 1u;
    }
}

static void report_exact_limit(Compiler *c, uint32_t start, uint32_t end) {
    Diag *diag;
    add_diag(c, "ORC0301", start, end, "exact integer result exceeds the 16384-significant-bit limit",
             "result is too large for the reference evaluator",
             "`Int` is unbounded; this is a resource limit, not a finite width", 2);
    if (c->ndiags == 0) {
        return;
    }
    diag = &c->diags[c->ndiags - 1];
    copy_text(diag->note2, sizeof diag->note2, "no partial value set is returned");
    diag->has_note2 = 1;
}

static void report_packed_bits(Compiler *c, uint32_t start, uint32_t end) {
    report_exact_limit(c, start, end);
}

static int eval_pack(Compiler *c, const Expr *expr, Value *operand, Value *out) {
    int from_words = type_width(operand->type) > 0;
    int to_words = type_width(expr->conv_ty) > 0;
    uint32_t bits;
    uint32_t count;
    uint64_t width;
    uint32_t nlimbs;
    uint32_t *limbs = NULL;
    uint64_t cost;
    uint32_t index;
    if (!from_words && !to_words) {
        value_clear(operand);
        c->failed = 1;
        return 0;
    }
    if (from_words) {
        bits = (uint32_t)type_width(operand->type);
        count = operand->length == 0 ? 1u : operand->length;
    } else {
        bits = (uint32_t)type_width(expr->conv_ty);
        count = expr->conv_len == 0 ? 1u : expr->conv_len;
    }
    width = (uint64_t)bits * (uint64_t)count;
    cost = (width + 63u) / 64u;
    if (cost == 0) {
        cost = 1;
    }
    if (!charge(c, expr->op_start, expr->op_end, cost)) {
        value_clear(operand);
        return 0;
    }
    nlimbs = (uint32_t)((width + 31u) / 32u);
    if (nlimbs == 0 || width / bits != count) {
        value_clear(operand);
        c->failed = 1;
        return 0;
    }
    limbs = calloc(nlimbs, sizeof(uint32_t));
    if (limbs == NULL) {
        value_clear(operand);
        c->failed = 1;
        add_diag(c, "ORC0301", expr->start, expr->end, "evaluation could not retain an integer",
                 "resource limit reached", NULL, 2);
        return 0;
    }
    if (from_words) {
        for (index = 0; index < count; index++) {
            uint64_t word = 0;
            uint32_t place = 0;
            if (operand->length == 0) {
                word = operand->word;
            } else if (index >= operand->length || (operand->pack == NULL && operand->elems == NULL)) {
                free(limbs);
                value_clear(operand);
                c->failed = 1;
                return 0;
            } else if (operand->pack != NULL) {
                word = pack_get(operand->pack, index);
            } else {
                word = operand->elems[index].word;
            }
            if (!pack_place(expr->conv_order, index, count, &place) ||
                !pack_write_word(limbs, nlimbs, bits, place, word)) {
                free(limbs);
                value_clear(operand);
                c->failed = 1;
                return 0;
            }
        }
    } else {
        if (width > UINT32_MAX) {
            free(limbs);
            value_clear(operand);
            c->failed = 1;
            report_packed_bits(c, expr->start, expr->end);
            return 0;
        }
        pack_write_residue(limbs, nlimbs, &operand->big, (uint32_t)width);
    }
    if (expr->conv_ty == TY_INT || expr->conv_ty == TY_MOD) {
        Big value = big_zero();
        if (!big_from_limbs(&c->arena, limbs, nlimbs, 0, &value)) {
            free(limbs);
            value_clear(operand);
            c->failed = 1;
            report_packed_bits(c, expr->start, expr->end);
            return 0;
        }
        free(limbs);
        limbs = NULL;
        if (expr->conv_ty == TY_MOD) {
            const Big *modulus = modulus_at(c, expr->conv_mod);
            Big reduced = big_zero();
            uint64_t extra;
            if (modulus == NULL) {
                value_clear(operand);
                c->failed = 1;
                return 0;
            }
            extra = (uint64_t)big_limbs(&value) * modulus_digits(modulus);
            if (!charge(c, expr->op_start, expr->op_end, extra)) {
                value_clear(operand);
                return 0;
            }
            if (!residue_reduce(c, &value, modulus, &reduced)) {
                value_clear(operand);
                c->failed = 1;
                report_packed_bits(c, expr->start, expr->end);
                return 0;
            }
            out->type = TY_MOD;
            out->mod_index = expr->conv_mod;
            out->big = reduced;
            out->word = 0;
            value_clear(operand);
            return 1;
        }
        out->type = TY_INT;
        out->big = value;
        out->word = 0;
        value_clear(operand);
        return 1;
    }
    if (expr->conv_len == 0) {
        uint64_t word = 0;
        uint32_t to_bits = (uint32_t)type_width(expr->conv_ty);
        if (!pack_read_word(limbs, nlimbs, to_bits, 0, &word)) {
            free(limbs);
            value_clear(operand);
            c->failed = 1;
            return 0;
        }
        free(limbs);
        out->type = expr->conv_ty;
        out->word = word;
        out->big = big_zero();
        value_clear(operand);
        return 1;
    }
    {
        uint32_t to_bits = (uint32_t)type_width(expr->conv_ty);
        uint32_t to_count = expr->conv_len;
        uint32_t stride = pack_stride_of(expr->conv_ty, 0);
        Pack *packed = pack_new(expr->conv_ty, to_count, stride, 0);
        if (packed == NULL) {
            free(limbs);
            value_clear(operand);
            c->failed = 1;
            add_diag(c, "ORC0301", expr->start, expr->end, "evaluation could not retain an array",
                     "resource limit reached", NULL, 2);
            return 0;
        }
        for (index = 0; index < to_count; index++) {
            uint32_t place = 0;
            uint64_t word = 0;
            if (!pack_place(expr->conv_order, index, to_count, &place) ||
                !pack_read_word(limbs, nlimbs, to_bits, place, &word)) {
                free(limbs);
                pack_release(packed);
                value_clear(operand);
                c->failed = 1;
                return 0;
            }
            pack_set(packed, index, word);
        }
        free(limbs);
        out->type = expr->conv_ty;
        out->length = to_count;
        out->pack = packed;
        out->big = big_zero();
        value_clear(operand);
        return 1;
    }
}

typedef struct AccWalk {
    Compiler *c;
    uint32_t loop;
    uint32_t uses_whole;
    uint32_t plain_whole;
    uint32_t uses_comp[MAX_TUPLE];
    uint32_t plain_comp[MAX_TUPLE];
    int unsafe;
    int depth;
    /* A nested loop evaluates its body once per step, against the same
       outer accumulator. A single syntactic use there is not one owner. */
    int repeating;
} AccWalk;

static void acc_visit(AccWalk *walk, uint32_t index);

static void acc_count(AccWalk *walk, const Expr *expr, int as_update) {
    uint32_t bump = walk->repeating ? 2u : 1u;
    if (!expr->is_proj) {
        walk->uses_whole += bump;
        if (!as_update) {
            walk->plain_whole += bump;
        }
        return;
    }
    if (expr->proj_pos < MAX_TUPLE) {
        walk->uses_comp[expr->proj_pos] += bump;
        if (!as_update) {
            walk->plain_comp[expr->proj_pos] += bump;
        }
    }
}

static void acc_note(AccWalk *walk, uint32_t index, int as_update) {
    const Expr *expr;
    if (index == UINT32_MAX) {
        return;
    }
    expr = &walk->c->exprs[index];
    if (expr->kind == EX_ACCUM && expr->arg0 == walk->loop) {
        acc_count(walk, expr, as_update);
        return;
    }
    acc_visit(walk, index);
}

static void acc_binds(AccWalk *walk, uint32_t bind0, uint16_t nbinds) {
    uint16_t bind;
    for (bind = 0; bind < nbinds; bind++) {
        const Local *local = &walk->c->block_locals[bind0 + bind];
        if (local->pat_i > 0) {
            continue;
        }
        acc_visit(walk, local->value);
    }
}

static void acc_visit(AccWalk *walk, uint32_t index) {
    const Expr *expr;
    uint32_t child;
    if (index == UINT32_MAX || walk->unsafe) {
        return;
    }
    if (walk->depth > MAX_HEIGHT + 4) {
        walk->unsafe = 1;
        return;
    }
    walk->depth++;
    expr = &walk->c->exprs[index];
    switch (expr->kind) {
    case EX_NONE:
    case EX_LIT:
    case EX_NAME:
    case EX_BYTES:
    case EX_LOOP_INDEX:
        break;
    case EX_ACCUM:
        if (expr->arg0 == walk->loop) {
            acc_count(walk, expr, 0);
        }
        break;
    case EX_UNARY:
    case EX_GROUP:
    case EX_PROJECT:
    case EX_CONV:
        acc_visit(walk, expr->left);
        break;
    case EX_FILL:
        acc_visit(walk, expr->left);
        if (expr->size_expr != UINT32_MAX) {
            acc_visit(walk, expr->size_expr);
        }
        break;
    case EX_BINARY:
    case EX_SHIFT:
    case EX_INDEX:
    case EX_SELECT:
        acc_visit(walk, expr->left);
        acc_visit(walk, expr->right);
        break;
    case EX_UPDATE:
        acc_note(walk, expr->left, 1);
        acc_visit(walk, expr->right);
        acc_visit(walk, expr->callee);
        break;
    case EX_SLICE:
        acc_visit(walk, expr->left);
        if (expr->right != UINT32_MAX) {
            acc_visit(walk, expr->right);
        }
        if (expr->callee != UINT32_MAX) {
            acc_visit(walk, expr->callee);
        }
        break;
    case EX_SLICE_UP:
        acc_note(walk, expr->left, 1);
        if (expr->right != UINT32_MAX) {
            acc_visit(walk, expr->right);
        }
        if (expr->conv_site != UINT32_MAX) {
            acc_visit(walk, expr->conv_site);
        }
        acc_visit(walk, expr->callee);
        break;
    case EX_ARRAY:
    case EX_TUPLE:
    case EX_CALL:
        for (child = 0; child < expr->argc; child++) {
            acc_visit(walk, walk->c->args[expr->arg0 + child]);
        }
        break;
    case EX_COND: {
        uint16_t arm;
        for (arm = 0; arm < expr->argc; arm++) {
            CondArm *slot = &walk->c->cond_arms[expr->arg0 + arm];
            acc_visit(walk, slot->cond);
            acc_binds(walk, slot->bind0, slot->nbinds);
            acc_visit(walk, slot->value);
        }
        acc_binds(walk, expr->else_bind0, expr->else_nbinds);
        acc_visit(walk, expr->right);
        break;
    }
    case EX_LOOP: {
        const LoopDesc *nested = &walk->c->loops[expr->arg0];
        int nested_loop = expr->arg0 != walk->loop;
        if (nested_loop) {
            walk->repeating++;
        }
        acc_visit(walk, nested->init_expr);
        acc_binds(walk, nested->bind0, nested->nbinds);
        acc_visit(walk, nested->step_expr);
        if (nested_loop) {
            walk->repeating--;
        }
        break;
    }
    default:
        walk->unsafe = 1;
        break;
    }
    walk->depth--;
}

static int acc_sole(const AccWalk *walk, uint32_t uses, uint32_t plain) {
    (void)plain;
    return !walk->unsafe && uses == 1;
}

static void mark_sole_accum(Compiler *c, uint32_t loop_id) {
    LoopDesc *loop = &c->loops[loop_id];
    AccWalk walk;
    uint8_t index;
    memset(&walk, 0, sizeof walk);
    walk.c = c;
    walk.loop = loop_id;
    acc_binds(&walk, loop->bind0, loop->nbinds);
    acc_visit(&walk, loop->step_expr);
    loop->sole_whole = (uint8_t)acc_sole(&walk, walk.uses_whole, walk.plain_whole);
    for (index = 0; index < MAX_TUPLE; index++) {
        loop->sole_comp[index] = 0;
    }
    for (index = 0; index < loop->nacc && index < MAX_TUPLE; index++) {
        loop->sole_comp[index] = (uint8_t)acc_sole(&walk, walk.uses_comp[index], walk.plain_comp[index]);
    }
    loop->sole_ready = 1;
}

static int eval_expr_in(Compiler *c, uint32_t index, Value *params, Value *locals, int depth, Value *out) {
    const Expr *expr = &c->exprs[index];
    if (c->failed) {
        return 0;
    }
    switch (expr->kind) {
    case EX_GROUP:
        return eval_expr(c, expr->left, params, locals, depth, out);
    case EX_LIT: {
        Big value = big_zero();
        if (expr->ty == TY_MOD) {
            if (!charge(c, expr->start, expr->end, 1) || !residue_literal(c, expr, &value)) {
                c->failed = 1;
                return 0;
            }
            out->type = TY_MOD;
            out->mod_index = expr->ty_mod;
            out->big = value;
            out->word = 0;
            return 1;
        }
        if (!charge(c, expr->start, expr->end, 1) || !decode_literal(c, expr, &value)) {
            c->failed = 1;
            return 0;
        }
        if (expr->ty != TY_INT) {
            uint64_t word = 0;
            if (value.nlimbs > 0) {
                word = value.limbs[0];
            }
            if (value.nlimbs > 1) {
                word |= (uint64_t)value.limbs[1] << 32;
            }
            out->type = expr->ty;
            out->word = word & word_mask_of(type_width(expr->ty));
            out->big = big_zero();
            return 1;
        }
        out->type = TY_INT;
        out->word = 0;
        out->big = value;
        return 1;
    }
    case EX_NAME:
        if (!charge(c, expr->start, expr->end, expr->is_proj ? 2u : 1u)) {
            return 0;
        }
        if (expr->name_res == NAME_BLOCK) {
            int frame;
            for (frame = c->nframes - 1; frame >= 0; frame--) {
                BlockFrame *block = &c->frames[frame];
                if (block->slots != NULL && expr->name_abs >= block->bind0 &&
                    expr->name_abs < block->bind0 + block->visible) {
                    return value_clone(c, out, &block->slots[expr->name_abs - block->bind0], expr->start, expr->end);
                }
            }
            c->failed = 1;
            return 0;
        }
        if (expr->name_res == NAME_BOOL) {
            out->type = TY_BOOL;
            out->word = expr->name_index;
            out->length = 0;
            out->big = big_zero();
            return 1;
        }
        if (expr->name_res == NAME_SIZE) {
            int64_t value;
            uint64_t magnitude;
            if (expr->name_index >= c->ncur) {
                c->failed = 1;
                return 0;
            }
            value = c->cur_sz[expr->name_index];
            if (value >= 0) {
                if (!big_from_u64(&c->arena, (uint64_t)value, &out->big)) {
                    c->failed = 1;
                    return 0;
                }
            } else {
                magnitude = value == INT64_MIN ? (uint64_t)INT64_MAX + 1u : (uint64_t)(-value);
                if (!big_from_u64(&c->arena, magnitude, &out->big) || !big_neg(&out->big, &out->big)) {
                    c->failed = 1;
                    return 0;
                }
            }
            out->type = TY_INT;
            out->word = 0;
            out->length = 0;
            return 1;
        }
        return value_clone(c, out,
                           expr->name_res == NAME_LOCAL ? &locals[expr->name_index] : &params[expr->name_index],
                           expr->start, expr->end);
    case EX_CALL: {
        Value *arguments;
        uint16_t arg;
        uint16_t count;
        int ok;
        uint32_t resolved = expr->inst_id;
        /* Checking stores the instance chosen for the last value of the
           caller's sizes. Resolve again for the instance now running, so a
           call inside a sized function follows this instance.
           A callee with type parameters is resolved against the concrete type
           this place expects, including a tuple result. Stamps for the caller's
           instance have already substituted its type parameters, so `m()` inside
           `deep[Word[8]]` selects `m[Word[8]]`, and `pair()` inside a function
           that returns `(K, K)` selects `pair` at that same `K`. */
        if (c->cur_func < c->nfuncs && expr->callee != UINT32_MAX) {
            Compiler *lookup_target = c;
            uint32_t callee_index = expr->callee;
            int can_lookup = 1;
            if (expr->left != UINT32_MAX) {
                if (c->program == NULL || expr->name_index >= c->program->nmods) {
                    can_lookup = 0;
                } else {
                    lookup_target = c->program->mods[expr->name_index];
                }
            }
            if (can_lookup && callee_index < lookup_target->nfuncs &&
                lookup_target->funcs[callee_index].ninst > 0) {
                int callee_types = tp_func_has_types(&lookup_target->funcs[callee_index]);
                int caller_types = c->cur_func < c->nfuncs && tp_func_has_types(&c->funcs[c->cur_func]);
                if (!(callee_types && !caller_types)) {
                    int saved_fit = c->fit_set;
                    TypeKind saved_kind = c->fit_kind;
                    uint32_t saved_len = c->fit_len;
                    uint16_t saved_mod = c->fit_mod;
                    uint32_t saved_tup0 = c->fit_tup0;
                    uint16_t saved_tup_n = c->fit_tup_n;
                    uint32_t fresh = UINT32_MAX;
                    if (callee_types && expr->ty != TY_NONE) {
                        c->fit_set = 1;
                        c->fit_kind = expr->ty;
                        c->fit_len = expr->ty_len;
                        c->fit_mod = expr->ty_mod;
                        c->fit_tup0 = expr->ty == TY_TUPLE ? expr->ty_tup0 : 0;
                        c->fit_tup_n = expr->ty == TY_TUPLE ? expr->ty_tup_n : 0;
                    } else if (callee_types) {
                        c->fit_set = 0;
                    }
                    if (!lookup_call(c, &c->exprs[index], lookup_target, callee_index, 0, c->cur_func,
                                     c->funcs[c->cur_func].nlocals, &fresh)) {
                        c->fit_set = saved_fit;
                        c->fit_kind = saved_kind;
                        c->fit_len = saved_len;
                        c->fit_mod = saved_mod;
                        c->fit_tup0 = saved_tup0;
                        c->fit_tup_n = saved_tup_n;
                        c->failed = 1;
                        return 0;
                    }
                    c->fit_set = saved_fit;
                    c->fit_kind = saved_kind;
                    c->fit_len = saved_len;
                    c->fit_mod = saved_mod;
                    c->fit_tup0 = saved_tup0;
                    c->fit_tup_n = saved_tup_n;
                    resolved = fresh;
                }
            }
        }
        count = expr->argc == 0 ? 1u : expr->argc;
        arguments = calloc(count, sizeof(Value));
        if (arguments == NULL) {
            c->failed = 1;
            add_diag(c, "ORC0301", expr->start, expr->end, "evaluation could not retain a call frame",
                     "resource limit reached", NULL, 2);
            return 0;
        }
        for (arg = 0; arg < expr->argc; arg++) {
            if (!eval_expr(c, c->args[expr->arg0 + arg], params, locals, depth, &arguments[arg])) {
                value_list_clear(arguments, count);
                return 0;
            }
        }
        if (!charge(c, expr->start, expr->end, 1)) {
            c->failed = 1;
            value_list_clear(arguments, count);
            return 0;
        }
        if (expr->left != UINT32_MAX) {
            Compiler *target;
            Func *callee;
            if (c->program == NULL || expr->name_index >= c->program->nmods) {
                c->failed = 1;
                value_list_clear(arguments, count);
                return 0;
            }
            target = c->program->mods[expr->name_index];
            if (expr->callee >= target->nfuncs) {
                c->failed = 1;
                value_list_clear(arguments, count);
                return 0;
            }
            callee = &target->funcs[expr->callee];
            for (arg = 0; arg < expr->argc; arg++) {
                uint16_t mod_index = target->params[callee->param0 + arg].mod_index;
                TypeKind arg_type = target->params[callee->param0 + arg].type;
                if (resolved != UINT32_MAX && resolved < target->ninstances) {
                    const Instance *called = &target->instances[resolved];
                    if (arg < callee->nparams) {
                        mod_index = target->iparams[called->param0 + arg].mod_index;
                        arg_type = target->iparams[called->param0 + arg].type;
                    }
                }
                if (arg_type == TY_MOD && !retag_modulus(&arguments[arg], mod_index)) {
                    c->failed = 1;
                    add_diag(c, "ORC0301", expr->start, expr->end, "evaluation could not retain an array",
                             "resource limit reached", NULL, 2);
                    value_list_clear(arguments, count);
                    return 0;
                }
            }
            /* One step budget and one failure flag for the whole program.
               Copy them across the call so a nested module spends the same
               counter, then copy the result back. The callee's size environment
               is the instance this call resolved, then the caller's is restored. */
            {
                int64_t saved_sz[MAX_SIZES];
                uint8_t saved_ncur = target->ncur;
                uint32_t saved_func = target->cur_func;
                uint32_t saved_inst = target->cur_inst;
                memcpy(saved_sz, target->cur_sz, sizeof saved_sz);
                if (resolved != UINT32_MAX && resolved < target->ninstances) {
                    memcpy(target->cur_sz, target->instances[resolved].sz, sizeof target->cur_sz);
                    target->ncur = callee->nsizes;
                    target->cur_func = expr->callee;
                    target->cur_inst = resolved;
                }
                target->steps = c->steps;
                target->step_limit = c->step_limit;
                target->step_hit = c->step_hit;
                target->failed = c->failed;
                ok = eval_function(target, expr->callee, arguments, depth + 1, out);
                c->steps = target->steps;
                if (target->step_hit) {
                    c->step_hit = 1;
                }
                if (target->failed) {
                    c->failed = 1;
                }
                memcpy(target->cur_sz, saved_sz, sizeof target->cur_sz);
                target->ncur = saved_ncur;
                target->cur_func = saved_func;
                target->cur_inst = saved_inst;
            }
        } else {
            int64_t saved_sz[MAX_SIZES];
            uint8_t saved_ncur;
            uint32_t saved_func;
            uint32_t saved_inst;
            if (expr->callee >= c->nfuncs) {
                c->failed = 1;
                value_list_clear(arguments, count);
                return 0;
            }
            saved_ncur = c->ncur;
            saved_func = c->cur_func;
            saved_inst = c->cur_inst;
            memcpy(saved_sz, c->cur_sz, sizeof saved_sz);
            if (resolved != UINT32_MAX && resolved < c->ninstances) {
                memcpy(c->cur_sz, c->instances[resolved].sz, sizeof c->cur_sz);
                c->ncur = c->funcs[expr->callee].nsizes;
                c->cur_func = expr->callee;
                c->cur_inst = resolved;
            }
            ok = eval_function(c, expr->callee, arguments, depth + 1, out);
            memcpy(c->cur_sz, saved_sz, sizeof c->cur_sz);
            c->ncur = saved_ncur;
            c->cur_func = saved_func;
            c->cur_inst = saved_inst;
        }
        if (ok && expr->ty == TY_MOD && !retag_modulus(out, expr->ty_mod)) {
            c->failed = 1;
            add_diag(c, "ORC0301", expr->start, expr->end, "evaluation could not retain an array",
                     "resource limit reached", NULL, 2);
            value_list_clear(arguments, count);
            return 0;
        }
        value_list_clear(arguments, count);
        return ok;
    }
    case EX_UNARY: {
        Value operand;
        memset(&operand, 0, sizeof operand);
        if (!eval_expr(c, expr->left, params, locals, depth, &operand)) {
            return 0;
        }
        if (expr->op == TK_BANG) {
            if (!charge(c, expr->start, expr->end, 1)) {
                value_clear(&operand);
                return 0;
            }
            out->type = TY_BOOL;
            out->word = operand.word == 0 ? 1u : 0u;
            out->length = 0;
            out->big = big_zero();
            value_clear(&operand);
            return 1;
        }
        if (expr->op == TK_MINUS && operand.type == TY_MOD) {
            const Big *modulus = modulus_at(c, operand.mod_index);
            Big negated = big_zero();
            uint64_t cost;
            if (modulus == NULL) {
                value_clear(&operand);
                c->failed = 1;
                return 0;
            }
            cost = 1 + modulus_digits(modulus);
            if (!charge(c, expr->start, expr->end, cost)) {
                value_clear(&operand);
                return 0;
            }
            if (operand.big.nlimbs == 0) {
                negated = operand.big;
            } else if (!big_sub(&c->arena, modulus, &operand.big, &negated)) {
                value_clear(&operand);
                c->failed = 1;
                return 0;
            }
            out->type = TY_MOD;
            out->mod_index = operand.mod_index;
            out->big = negated;
            out->word = 0;
            value_clear(&operand);
            return 1;
        }
        if (expr->op == TK_MINUS) {
            uint64_t cost = 1 + big_limbs(&operand.big);
            Big negated;
            if (!charge(c, expr->start, expr->end, cost) || !big_neg(&operand.big, &negated)) {
                value_clear(&operand);
                return 0;
            }
            out->type = TY_INT;
            out->big = negated;
            out->word = 0;
            value_clear(&operand);
            return 1;
        }
        if (!charge(c, expr->start, expr->end, 1)) {
            value_clear(&operand);
            return 0;
        }
        out->type = operand.type;
        out->word = (~operand.word) & word_mask_of(type_width(operand.type));
        out->big = big_zero();
        value_clear(&operand);
        return 1;
    }
    case EX_BINARY: {
        Value left;
        Value right;
        memset(&left, 0, sizeof left);
        memset(&right, 0, sizeof right);
        if (!eval_expr(c, expr->left, params, locals, depth, &left) ||
            !eval_expr(c, expr->right, params, locals, depth, &right)) {
            value_clear(&left);
            value_clear(&right);
            return 0;
        }
        if (expr->op == TK_PLUSPLUS) {
            uint32_t total;
            uint64_t cost;
            Value *items = NULL;
            uint32_t slot;
            if (left.pack != NULL || right.pack != NULL) {
                Pack *joined;
                if (left.length == 0 || right.length == 0 || left.type != right.type || left.is_tuple || right.is_tuple ||
                    left.length > UINT32_MAX - right.length) {
                    c->failed = 1;
                    value_clear(&left);
                    value_clear(&right);
                    return 0;
                }
                total = left.length + right.length;
                cost = ((uint64_t)total + 63u) / 64u;
                if (!charge(c, expr->start, expr->end, cost == 0 ? 1 : cost)) {
                    value_clear(&left);
                    value_clear(&right);
                    return 0;
                }
                if (left.pack != NULL && right.pack != NULL) {
                    joined = pack_concat(left.pack, right.pack);
                } else {
                    uint32_t stride = left.pack != NULL ? left.pack->stride : right.pack->stride;
                    uint16_t mod_index = left.pack != NULL ? left.pack->mod_index : right.pack->mod_index;
                    uint32_t index;
                    joined = pack_new(left.type, total, stride, mod_index);
                    for (index = 0; joined != NULL && index < total; index++) {
                        uint64_t word = 0;
                        const Value *source = index < left.length ? &left : &right;
                        uint32_t at = index < left.length ? index : index - left.length;
                        if (!elem_word(source, at, &word)) {
                            pack_release(joined);
                            joined = NULL;
                            c->failed = 1;
                            break;
                        }
                        pack_set(joined, index, word);
                    }
                }
                if (joined == NULL) {
                    value_clear(&left);
                    value_clear(&right);
                    if (!c->failed) {
                        return array_oom(c, expr->start, expr->end);
                    }
                    return 0;
                }
                out->type = left.type;
                out->length = total;
                out->mod_index = joined->mod_index;
                out->pack = joined;
                value_clear(&left);
                value_clear(&right);
                return 1;
            }
            if (left.length == 0 || right.length == 0 || left.elems == NULL || right.elems == NULL ||
                left.type != right.type || left.is_tuple || right.is_tuple || left.length > UINT32_MAX - right.length) {
                c->failed = 1;
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            total = left.length + right.length;
            cost = ((uint64_t)total + 63u) / 64u;
            if (!charge(c, expr->start, expr->end, cost == 0 ? 1 : cost) ||
                !alloc_array(c, &items, total, expr->start, expr->end)) {
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            for (slot = 0; slot < left.length; slot++) {
                if (!value_clone(c, &items[slot], &left.elems[slot], expr->start, expr->end)) {
                    value_list_clear(items, total);
                    value_clear(&left);
                    value_clear(&right);
                    return 0;
                }
            }
            for (slot = 0; slot < right.length; slot++) {
                if (!value_clone(c, &items[left.length + slot], &right.elems[slot], expr->start, expr->end)) {
                    value_list_clear(items, total);
                    value_clear(&left);
                    value_clear(&right);
                    return 0;
                }
            }
            {
                TypeKind element = left.type;
                value_clear(&left);
                value_clear(&right);
                out->type = element;
                out->length = total;
                out->elems = items;
            }
            return 1;
        }
        if (is_compare_op(expr->op)) {
            int ordering = 0;
            uint64_t cost = 1;
            if (left.type != right.type || left.length != 0 || right.length != 0) {
                c->failed = 1;
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            if (left.type == TY_INT) {
                uint32_t left_limbs = big_limbs(&left.big);
                uint32_t right_limbs = big_limbs(&right.big);
                cost = 1 + (left_limbs > right_limbs ? left_limbs : right_limbs);
                ordering = big_cmp(&left.big, &right.big);
            } else if (left.type == TY_MOD) {
                const Big *modulus = modulus_at(c, left.mod_index);
                if (modulus == NULL || left.mod_index != right.mod_index) {
                    c->failed = 1;
                    value_clear(&left);
                    value_clear(&right);
                    return 0;
                }
                cost = 1 + modulus_digits(modulus);
                ordering = big_cmp(&left.big, &right.big);
            } else if (left.type == TY_BOOL || type_width(left.type) != 0) {
                ordering = word_ordering(left.word, right.word);
            } else {
                c->failed = 1;
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            if (!charge(c, expr->op_start, expr->op_end, cost)) {
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            out->type = TY_BOOL;
            out->word = relation_holds(expr->op, ordering) ? 1u : 0u;
            out->length = 0;
            out->big = big_zero();
            value_clear(&left);
            value_clear(&right);
            return 1;
        }
        if (expr->op == TK_AMPAMP || expr->op == TK_PIPEPIPE) {
            int left_true;
            int right_true;
            if (left.type != TY_BOOL || right.type != TY_BOOL) {
                c->failed = 1;
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            if (!charge(c, expr->op_start, expr->op_end, 1)) {
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            left_true = left.word != 0;
            right_true = right.word != 0;
            out->type = TY_BOOL;
            out->word = (uint64_t)(expr->op == TK_AMPAMP ? (left_true && right_true) : (left_true || right_true));
            out->length = 0;
            out->big = big_zero();
            value_clear(&left);
            value_clear(&right);
            return 1;
        }
        if ((expr->op == TK_SLASH || expr->op == TK_PERCENT) && left.type != TY_MOD) {
            if (left.type == TY_INT) {
                uint32_t dividend_limbs = big_limbs(&left.big);
                uint32_t divisor_limbs = big_limbs(&right.big);
                uint64_t divisor_cost = divisor_limbs == 0 ? 1u : divisor_limbs;
                Big quotient = big_zero();
                Big remainder = big_zero();
                if (!charge(c, expr->op_start, expr->op_end, 1 + (uint64_t)dividend_limbs * divisor_cost)) {
                    value_clear(&left);
                    value_clear(&right);
                    return 0;
                }
                if (right.big.nlimbs == 0) {
                    remainder = left.big;
                } else if (!big_div_euclid(&c->arena, &left.big, &right.big, &quotient, &remainder)) {
                    c->failed = 1;
                    report_exact_limit(c, expr->start, expr->end);
                    value_clear(&left);
                    value_clear(&right);
                    return 0;
                }
                out->type = TY_INT;
                out->word = 0;
                out->length = 0;
                out->big = expr->op == TK_SLASH ? quotient : remainder;
                value_clear(&left);
                value_clear(&right);
                return 1;
            }
            {
                int width = type_width(left.type);
                uint64_t mask = word_mask_of(width);
                uint64_t dividend = left.word & mask;
                uint64_t divisor = right.word & mask;
                if (width == 0 || !charge(c, expr->op_start, expr->op_end, 1)) {
                    c->failed = 1;
                    value_clear(&left);
                    value_clear(&right);
                    return 0;
                }
                if (divisor == 0) {
                    out->word = expr->op == TK_SLASH ? 0 : dividend;
                } else if (expr->op == TK_SLASH) {
                    out->word = dividend / divisor;
                } else {
                    out->word = dividend % divisor;
                }
                out->type = left.type;
                out->length = 0;
                out->big = big_zero();
                value_clear(&left);
                value_clear(&right);
                return 1;
            }
        }
        if (left.type == TY_MOD) {
            const Big *modulus = modulus_at(c, left.mod_index);
            Big exact = big_zero();
            Big reduced = big_zero();
            uint32_t digits;
            uint64_t square;
            uint64_t cost = 1;
            int ok = 0;
            if (modulus == NULL || left.mod_index != right.mod_index) {
                c->failed = 1;
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            digits = modulus_digits(modulus);
            square = (uint64_t)digits * (uint64_t)digits;
            if (expr->op == TK_STAR) {
                cost = 1 + 2 * square;
            } else if (expr->op == TK_SLASH) {
                cost = 1 + 64 * square;
            } else {
                cost = 1 + digits;
            }
            if (!charge(c, expr->op_start, expr->op_end, cost)) {
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            if (expr->op == TK_PLUS) {
                ok = big_add(&c->arena, &left.big, &right.big, &exact);
            } else if (expr->op == TK_MINUS) {
                ok = big_sub(&c->arena, &left.big, &right.big, &exact);
            } else if (expr->op == TK_STAR) {
                ok = big_mul(&c->arena, &left.big, &right.big, &exact);
            } else if (expr->op == TK_SLASH) {
                Big inverse = big_zero();
                ok = residue_inverse(c, &right.big, modulus, &inverse) &&
                     big_mul(&c->arena, &left.big, &inverse, &exact);
            }
            if (!ok || !residue_reduce(c, &exact, modulus, &reduced)) {
                c->failed = 1;
                report_exact_limit(c, expr->start, expr->end);
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            out->type = TY_MOD;
            out->mod_index = left.mod_index;
            out->big = reduced;
            out->word = 0;
            value_clear(&left);
            value_clear(&right);
            return 1;
        }
        if (left.type == TY_INT) {
            Big result = big_zero();
            uint64_t cost = 1;
            int ok = 0;
            if (expr->op == TK_STAR) {
                cost = 1 + (uint64_t)big_limbs(&left.big) * (uint64_t)big_limbs(&right.big);
            } else {
                uint32_t left_limbs = big_limbs(&left.big);
                uint32_t right_limbs = big_limbs(&right.big);
                cost = 1 + (left_limbs > right_limbs ? left_limbs : right_limbs);
            }
            if (!charge(c, expr->op_start, expr->op_end, cost)) {
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            if (expr->op == TK_PLUS) {
                ok = big_add(&c->arena, &left.big, &right.big, &result);
            } else if (expr->op == TK_MINUS) {
                ok = big_sub(&c->arena, &left.big, &right.big, &result);
            } else if (expr->op == TK_STAR) {
                ok = big_mul(&c->arena, &left.big, &right.big, &result);
            }
            if (!ok) {
                c->failed = 1;
                report_exact_limit(c, expr->start, expr->end);
                value_clear(&left);
                value_clear(&right);
                return 0;
            }
            out->type = TY_INT;
            out->big = result;
            out->word = 0;
            value_clear(&left);
            value_clear(&right);
            return 1;
        }
        if (!charge(c, expr->op_start, expr->op_end, 1) ||
            !binary_words(expr->op, left.word, right.word, type_width(left.type), &out->word)) {
            value_clear(&left);
            value_clear(&right);
            return 0;
        }
        out->type = left.type;
        out->big = big_zero();
        value_clear(&left);
        value_clear(&right);
        return 1;
    }
    case EX_SHIFT: {
        Value left;
        const Expr *amount;
        Big magnitude = big_zero();
        uint32_t shift;
        int width;
        uint64_t mask;
        uint64_t value;
        memset(&left, 0, sizeof left);
        if (!eval_expr(c, expr->left, params, locals, depth, &left)) {
            return 0;
        }
        amount = &c->exprs[expr->right];
        if (!decode_literal(c, amount, &magnitude) || magnitude.nlimbs == 0) {
            shift = 0;
        } else {
            shift = magnitude.limbs[0];
        }
        width = type_width(left.type);
        mask = word_mask_of(width);
        value = left.word & mask;
        if (!charge(c, expr->op_start, expr->op_end, 1)) {
            value_clear(&left);
            return 0;
        }
        if (shift == 0) {
            out->word = value;
        } else if (expr->op == TK_LSHIFT) {
            out->word = (value << shift) & mask;
        } else if (expr->op == TK_RSHIFT) {
            out->word = value >> shift;
        } else if (expr->op == TK_ROL) {
            out->word = ((value << shift) | (value >> (width - (int)shift))) & mask;
        } else {
            out->word = ((value >> shift) | (value << (width - (int)shift))) & mask;
        }
        out->type = left.type;
        out->big = big_zero();
        value_clear(&left);
        return 1;
    }
    case EX_CONV: {
        Value operand;
        uint64_t word = 0;
        uint64_t extra = 0;
        memset(&operand, 0, sizeof operand);
        if (!eval_expr(c, expr->left, params, locals, depth, &operand)) {
            return 0;
        }
        if (expr->conv_order != 0) {
            return eval_pack(c, expr, &operand, out);
        }
        if (expr->conv_ty == TY_MOD) {
            const Big *modulus = modulus_at(c, expr->conv_mod);
            uint32_t limbs = 0;
            if (modulus == NULL) {
                value_clear(&operand);
                c->failed = 1;
                return 0;
            }
            if (operand.type == TY_INT || operand.type == TY_MOD) {
                limbs = big_limbs(&operand.big);
            } else if (operand.word != 0) {
                limbs = operand.word > 0xffffffffu ? 2u : 1u;
            }
            extra = (uint64_t)limbs * modulus_digits(modulus);
        }
        if (!charge(c, expr->op_start, expr->op_end, 1 + extra)) {
            value_clear(&operand);
            return 0;
        }
        if (expr->conv_ty == TY_MOD) {
            const Big *modulus = modulus_at(c, expr->conv_mod);
            Big source = big_zero();
            Big reduced = big_zero();
            if (operand.type == TY_INT || operand.type == TY_MOD) {
                source = operand.big;
            } else if (!big_from_u64(&c->arena, operand.word, &source)) {
                value_clear(&operand);
                c->failed = 1;
                return 0;
            }
            if (modulus == NULL || !residue_reduce(c, &source, modulus, &reduced)) {
                value_clear(&operand);
                c->failed = 1;
                return 0;
            }
            out->type = TY_MOD;
            out->mod_index = expr->conv_mod;
            out->big = reduced;
            out->word = 0;
            value_clear(&operand);
            return 1;
        }
        if (operand.type == TY_INT || operand.type == TY_MOD) {
            if (expr->conv_ty == TY_INT) {
                out->type = TY_INT;
                out->big = operand.big;
                out->word = 0;
                operand.big = big_zero();
                value_clear(&operand);
                return 1;
            }
            if (!big_mod_pow2(&operand.big, (uint32_t)type_width(expr->conv_ty), &word)) {
                value_clear(&operand);
                return 0;
            }
            out->type = expr->conv_ty;
            out->word = word;
            out->big = big_zero();
            value_clear(&operand);
            return 1;
        }
        if (expr->conv_ty == TY_INT) {
            if (!big_from_u64(&c->arena, operand.word, &out->big)) {
                value_clear(&operand);
                return 0;
            }
            out->type = TY_INT;
            out->word = 0;
            value_clear(&operand);
            return 1;
        }
        out->type = expr->conv_ty;
        out->word = operand.word & word_mask_of(type_width(expr->conv_ty));
        out->big = big_zero();
        value_clear(&operand);
        return 1;
    }
    case EX_ARRAY: {
        /* Elements are evaluated first. The array then charges one step per
           element, and one step when it has none. A charge that does not fit
           in the remaining budget is dropped and is not added to the count. */
        Value *items = NULL;
        uint32_t element;
        uint64_t cost = expr->argc == 0 ? 1u : (uint64_t)expr->argc;
        if (expr->argc == 0) {
            if (!charge(c, expr->start, expr->end, cost)) {
                return 0;
            }
            out->type = expr->ty;
            out->length = 0;
            return 1;
        }
        if (!alloc_array(c, &items, expr->argc, expr->start, expr->end)) {
            return 0;
        }
        for (element = 0; element < expr->argc; element++) {
            if (!eval_expr(c, c->args[expr->arg0 + element], params, locals, depth, &items[element])) {
                value_list_clear(items, expr->argc);
                return 0;
            }
        }
        if (!charge(c, expr->start, expr->end, cost)) {
            value_list_clear(items, expr->argc);
            return 0;
        }
        {
            uint32_t stride = array_stride(c, expr->ty, items[0].mod_index);
            int scalars = stride != 0;
            uint32_t slot;
            for (slot = 0; scalars && slot < expr->argc; slot++) {
                if (items[slot].length != 0 || items[slot].pack != NULL || items[slot].type != expr->ty) {
                    scalars = 0;
                }
            }
            if (scalars) {
                Pack *packed = pack_new(expr->ty, expr->argc, stride, items[0].mod_index);
                uint32_t element;
                if (packed == NULL) {
                    value_list_clear(items, expr->argc);
                    return array_oom(c, expr->start, expr->end);
                }
                for (element = 0; element < expr->argc; element++) {
                    uint64_t word = 0;
                    if (!scalar_word(&items[element], &word)) {
                        pack_release(packed);
                        value_list_clear(items, expr->argc);
                        c->failed = 1;
                        return 0;
                    }
                    pack_set(packed, element, word);
                }
                value_list_clear(items, expr->argc);
                out->type = expr->ty;
                out->length = expr->argc;
                out->mod_index = packed->mod_index;
                out->pack = packed;
                return 1;
            }
        }
        out->type = expr->ty;
        out->length = expr->argc;
        out->elems = items;
        return 1;
    }
    case EX_INDEX: {
        Value base;
        Big magnitude = big_zero();
        uint32_t position = 0;
        memset(&base, 0, sizeof base);
        if (!eval_expr(c, expr->left, params, locals, depth, &base)) {
            return 0;
        }
        if (!decode_literal(c, expr, &magnitude) || !index_below(&magnitude, base.length) ||
            !charge(c, expr->start, expr->end, 1)) {
            c->failed = 1;
            value_clear(&base);
            return 0;
        }
        if (magnitude.nlimbs > 0) {
            position = magnitude.limbs[0];
        }
        if (position >= base.length || (base.pack == NULL && base.elems == NULL)) {
            c->failed = 1;
            value_clear(&base);
            return 0;
        }
        if (base.pack != NULL) {
            if (!scalar_from_word(c, out, base.type, base.pack->mod_index, pack_get(base.pack, position))) {
                value_clear(&base);
                c->failed = 1;
                return 0;
            }
            value_clear(&base);
            return 1;
        }
        if (!value_clone(c, out, &base.elems[position], expr->start, expr->end)) {
            value_clear(&base);
            return 0;
        }
        value_clear(&base);
        return 1;
    }
    case EX_SELECT: {
        Value base;
        Value index_value;
        uint32_t position = 0;
        memset(&base, 0, sizeof base);
        memset(&index_value, 0, sizeof index_value);
        if (!eval_expr(c, expr->left, params, locals, depth, &base) ||
            !eval_expr(c, expr->right, params, locals, depth, &index_value)) {
            value_clear(&base);
            value_clear(&index_value);
            return 0;
        }
        if (!index_position(c, &index_value, c->exprs[expr->right].start, c->exprs[expr->right].end, &position) ||
            !charge(c, expr->start, expr->end, 1)) {
            c->failed = 1;
            value_clear(&base);
            value_clear(&index_value);
            return 0;
        }
        if (position >= base.length || (base.pack == NULL && base.elems == NULL)) {
            c->failed = 1;
            value_clear(&base);
            value_clear(&index_value);
            return 0;
        }
        if (base.pack != NULL) {
            if (!scalar_from_word(c, out, base.type, base.pack->mod_index, pack_get(base.pack, position))) {
                value_clear(&base);
                value_clear(&index_value);
                c->failed = 1;
                return 0;
            }
            value_clear(&base);
            value_clear(&index_value);
            return 1;
        }
        if (!value_clone(c, out, &base.elems[position], expr->start, expr->end)) {
            value_clear(&base);
            value_clear(&index_value);
            return 0;
        }
        value_clear(&base);
        value_clear(&index_value);
        return 1;
    }
    case EX_UPDATE: {
        Value base;
        Value index_value;
        Value element;
        uint32_t position = 0;
        uint32_t length;
        uint64_t cost;
        memset(&base, 0, sizeof base);
        memset(&index_value, 0, sizeof index_value);
        memset(&element, 0, sizeof element);
        if (!eval_expr(c, expr->left, params, locals, depth, &base) ||
            !eval_expr(c, expr->right, params, locals, depth, &index_value) ||
            !eval_expr(c, expr->callee, params, locals, depth, &element)) {
            value_clear(&base);
            value_clear(&index_value);
            value_clear(&element);
            return 0;
        }
        if (base.length == 0 || (base.pack == NULL && base.elems == NULL) ||
            !index_position(c, &index_value, c->exprs[expr->right].start, c->exprs[expr->right].end, &position)) {
            c->failed = 1;
            value_clear(&base);
            value_clear(&index_value);
            value_clear(&element);
            return 0;
        }
        length = base.length;
        cost = ((uint64_t)length + 63u) / 64u;
        if (position >= length || !charge(c, expr->start, expr->end, cost == 0 ? 1 : cost)) {
            c->failed = 1;
            value_clear(&base);
            value_clear(&index_value);
            value_clear(&element);
            return 0;
        }
        if (base.pack != NULL) {
            uint64_t word = 0;
            if (!scalar_word(&element, &word)) {
                c->failed = 1;
                value_clear(&base);
                value_clear(&index_value);
                value_clear(&element);
                return 0;
            }
            if (!pack_make_unique(&base.pack)) {
                value_clear(&base);
                value_clear(&index_value);
                value_clear(&element);
                return array_oom(c, expr->start, expr->end);
            }
            pack_set(base.pack, position, word);
            transfer_array(out, &base);
            value_clear(&base);
            value_clear(&index_value);
            value_clear(&element);
            return 1;
        }
        value_clear(&base.elems[position]);
        value_move(&base.elems[position], &element);
        transfer_array(out, &base);
        value_clear(&base);
        value_clear(&index_value);
        value_clear(&element);
        return 1;
    }
    case EX_FILL: {
        Value element;
        Value *items = NULL;
        uint32_t count = expr->ty_len;
        uint32_t slot;
        uint64_t cost;
        TypeKind element_type;
        memset(&element, 0, sizeof element);
        if (expr->size_expr != UINT32_MAX) {
            uint32_t sized = 0;
            if (!size_length(c, expr->size_expr, 0, &sized)) {
                c->failed = 1;
                return 0;
            }
            count = sized;
        }
        if (count == 0 || !eval_expr(c, expr->left, params, locals, depth, &element)) {
            if (count == 0) {
                c->failed = 1;
            }
            value_clear(&element);
            return 0;
        }
        cost = ((uint64_t)count + 63u) / 64u;
        if (!charge(c, expr->start, expr->end, cost == 0 ? 1 : cost)) {
            value_clear(&element);
            return 0;
        }
        if (element.length == 0 && element.pack == NULL) {
            uint32_t stride = array_stride(c, element.type, element.mod_index);
            uint64_t word = 0;
            if (stride != 0 && scalar_word(&element, &word)) {
                Pack *packed = pack_new(element.type, count, stride, element.mod_index);
                if (packed == NULL) {
                    value_clear(&element);
                    return array_oom(c, expr->start, expr->end);
                }
                pack_fill(packed, word);
                element_type = element.type;
                value_clear(&element);
                out->type = element_type;
                out->length = count;
                out->mod_index = packed->mod_index;
                out->pack = packed;
                return 1;
            }
        }
        if (!alloc_array(c, &items, count, expr->start, expr->end)) {
            value_clear(&element);
            return 0;
        }
        for (slot = 0; slot < count; slot++) {
            if (!value_clone(c, &items[slot], &element, expr->start, expr->end)) {
                value_list_clear(items, count);
                value_clear(&element);
                return 0;
            }
        }
        element_type = element.type;
        value_clear(&element);
        out->type = element_type;
        out->length = count;
        out->elems = items;
        return 1;
    }
    case EX_LOOP: {
        const LoopDesc *loop = &c->loops[expr->arg0];
        Value acc;
        uint32_t step_index;
        uint32_t bound_a;
        uint32_t bound_b;
        memset(&acc, 0, sizeof acc);
        bound_a = loop->bound_a;
        bound_b = loop->bound_b;
        if (loop->a_sized || loop->b_sized) {
            int64_t a_size = (int64_t)loop->bound_a;
            int64_t b_size = (int64_t)loop->bound_b;
            if (loop->a_sized) {
                Sz bound = eval_size(c, loop->a_expr);
                if (bound.kind != 0 || bound.value < 0 || bound.value > (int64_t)MAX_LOOP_BOUND) {
                    c->failed = 1;
                    return 0;
                }
                a_size = bound.value;
            }
            if (loop->b_sized) {
                Sz bound = eval_size(c, loop->b_expr);
                if (bound.kind != 0 || bound.value < 0 || bound.value > (int64_t)MAX_LOOP_BOUND) {
                    c->failed = 1;
                    return 0;
                }
                b_size = bound.value;
            }
            if (a_size >= b_size) {
                c->failed = 1;
                return 0;
            }
            bound_a = (uint32_t)a_size;
            bound_b = (uint32_t)b_size;
        }
        if (!loop->bounds_ok || c->loop_k == NULL || c->loop_acc == NULL || loop->init_expr == UINT32_MAX ||
            loop->step_expr == UINT32_MAX) {
            c->failed = 1;
            return 0;
        }
        if (!loop->sole_ready) {
            mark_sole_accum(c, expr->arg0);
        }
        if (!eval_expr(c, loop->init_expr, params, locals, depth, &acc) ||
            !charge(c, expr->start, expr->end, 1)) {
            value_clear(&acc);
            return 0;
        }
        for (step_index = bound_a; step_index < bound_b; step_index++) {
            if (!charge(c, expr->start, expr->end, 1)) {
                value_clear(&acc);
                value_clear(&c->loop_acc[expr->arg0]);
                return 0;
            }
            c->loop_k[expr->arg0] = step_index;
            value_clear(&c->loop_acc[expr->arg0]);
            value_move(&c->loop_acc[expr->arg0], &acc);
            if (!eval_block(c, loop->bind0, loop->nbinds, loop->step_expr, params, locals, depth, &acc)) {
                value_clear(&c->loop_acc[expr->arg0]);
                return 0;
            }
        }
        value_move(out, &acc);
        value_clear(&c->loop_acc[expr->arg0]);
        return 1;
    }
    case EX_LOOP_INDEX: {
        if (c->loop_k == NULL || !charge(c, expr->start, expr->end, 1) ||
            !big_from_u64(&c->arena, c->loop_k[expr->arg0], &out->big)) {
            c->failed = 1;
            return 0;
        }
        out->type = TY_INT;
        out->length = 0;
        return 1;
    }
    case EX_TUPLE: {
        Value *items = NULL;
        uint16_t index;
        if (!alloc_array(c, &items, expr->argc, expr->start, expr->end)) {
            return 0;
        }
        for (index = 0; index < expr->argc; index++) {
            if (!eval_expr(c, c->args[expr->arg0 + index], params, locals, depth, &items[index])) {
                value_list_clear(items, expr->argc);
                return 0;
            }
        }
        if (!charge(c, expr->start, expr->end, expr->argc)) {
            value_list_clear(items, expr->argc);
            return 0;
        }
        out->type = TY_TUPLE;
        out->is_tuple = 1;
        out->length = expr->argc;
        out->elems = items;
        return 1;
    }
    case EX_PROJECT: {
        Value base;
        memset(&base, 0, sizeof base);
        if (!eval_expr(c, expr->left, params, locals, depth, &base) || !charge(c, expr->start, expr->end, 1)) {
            value_clear(&base);
            c->failed = 1;
            return 0;
        }
        if (!base.is_tuple || base.elems == NULL || expr->proj_pos >= base.length) {
            c->failed = 1;
            value_clear(&base);
            return 0;
        }
        if (!value_clone(c, out, &base.elems[expr->proj_pos], expr->start, expr->end)) {
            value_clear(&base);
            return 0;
        }
        value_clear(&base);
        return 1;
    }
    case EX_ACCUM: {
        uint32_t steps = expr->is_proj ? 2u : 1u;
        Value *acc;
        Value *slot;
        int move = 0;
        if (c->loop_acc == NULL || !charge(c, expr->start, expr->end, steps)) {
            c->failed = 1;
            return 0;
        }
        acc = &c->loop_acc[expr->arg0];
        if (expr->arg0 < c->nloops && c->loops[expr->arg0].sole_ready) {
            const LoopDesc *loop = &c->loops[expr->arg0];
            if (expr->is_proj) {
                move = expr->proj_pos < MAX_TUPLE && loop->sole_comp[expr->proj_pos];
            } else {
                move = loop->sole_whole;
            }
        }
        if (expr->is_proj) {
            if (!acc->is_tuple || acc->elems == NULL || expr->proj_pos >= acc->length) {
                c->failed = 1;
                return 0;
            }
            slot = &acc->elems[expr->proj_pos];
        } else {
            slot = acc;
        }
        if (move) {
            value_move(out, slot);
            return 1;
        }
        return value_clone(c, out, slot, expr->start, expr->end);
    }
    case EX_COND: {
        uint32_t arg0 = expr->arg0;
        uint16_t arms = expr->argc;
        uint32_t otherwise = expr->right;
        uint32_t else_bind0 = expr->else_bind0;
        uint16_t else_nbinds = expr->else_nbinds;
        uint32_t span_start = expr->start;
        uint32_t span_end = expr->end;
        uint16_t arm;
        for (arm = 0; arm < arms; arm++) {
            Value condition;
            uint32_t condition_expr = c->cond_arms[arg0 + arm].cond;
            uint32_t value_expr = c->cond_arms[arg0 + arm].value;
            uint32_t bind0 = c->cond_arms[arg0 + arm].bind0;
            uint16_t nbinds = c->cond_arms[arg0 + arm].nbinds;
            memset(&condition, 0, sizeof condition);
            if (!eval_expr(c, condition_expr, params, locals, depth, &condition)) {
                return 0;
            }
            if (condition.type != TY_BOOL || condition.length != 0 || !charge(c, span_start, span_end, 1)) {
                c->failed = 1;
                value_clear(&condition);
                return 0;
            }
            if (condition.word != 0) {
                value_clear(&condition);
                return eval_block(c, bind0, nbinds, value_expr, params, locals, depth, out);
            }
            value_clear(&condition);
        }
        return eval_block(c, else_bind0, else_nbinds, otherwise, params, locals, depth, out);
    }
    case EX_BYTES: {
        uint8_t *bytes = NULL;
        uint32_t count = 0;
        uint32_t err_start = 0;
        uint32_t err_end = 0;
        uint32_t point = 0;
        Pack *packed;
        uint32_t slot;
        if (decode_bytes(c, expr, NULL, &count, &err_start, &err_end, &point) != BYTES_OK || count == 0) {
            c->failed = 1;
            return 0;
        }
        bytes = malloc(count);
        if (bytes == NULL || decode_bytes(c, expr, bytes, &count, &err_start, &err_end, &point) != BYTES_OK ||
            !charge(c, expr->start, expr->end, 1)) {
            free(bytes);
            c->failed = 1;
            return 0;
        }
        packed = pack_new(TY_W8, count, 1, 0);
        if (packed == NULL) {
            free(bytes);
            return array_oom(c, expr->start, expr->end);
        }
        for (slot = 0; slot < count; slot++) {
            pack_set(packed, slot, bytes[slot]);
        }
        out->type = TY_W8;
        out->length = count;
        out->pack = packed;
        free(bytes);
        return 1;
    }
    case EX_SLICE:
    case EX_SLICE_UP: {
        Value base;
        Value replacement;
        Value *items = NULL;
        uint32_t start_at = 0;
        uint32_t end_at = 0;
        uint32_t length;
        uint32_t slot;
        uint64_t cost;
        uint32_t start_expr = expr->right;
        uint32_t end_expr = expr->kind == EX_SLICE ? expr->callee : expr->conv_site;
        int updating = expr->kind == EX_SLICE_UP;
        memset(&base, 0, sizeof base);
        memset(&replacement, 0, sizeof replacement);
        if (!eval_expr(c, expr->left, params, locals, depth, &base)) {
            return 0;
        }
        if (start_expr == UINT32_MAX) {
            if (!charge(c, expr->lit_start, expr->lit_end, 1)) {
                value_clear(&base);
                return 0;
            }
            start_at = 0;
        } else {
            Value bound;
            memset(&bound, 0, sizeof bound);
            if (!eval_expr(c, start_expr, params, locals, depth, &bound) ||
                !index_position(c, &bound, c->exprs[start_expr].start, c->exprs[start_expr].end, &start_at)) {
                value_clear(&bound);
                value_clear(&base);
                return 0;
            }
            value_clear(&bound);
        }
        if (end_expr == UINT32_MAX) {
            if (!charge(c, expr->lit_start, expr->lit_end, 1)) {
                value_clear(&base);
                return 0;
            }
            end_at = base.length;
        } else {
            Value bound;
            memset(&bound, 0, sizeof bound);
            if (!eval_expr(c, end_expr, params, locals, depth, &bound) ||
                !index_position(c, &bound, c->exprs[end_expr].start, c->exprs[end_expr].end, &end_at)) {
                value_clear(&bound);
                value_clear(&base);
                return 0;
            }
            value_clear(&bound);
        }
        if (updating && !eval_expr(c, expr->callee, params, locals, depth, &replacement)) {
            value_clear(&base);
            return 0;
        }
        if (base.length == 0 || (base.pack == NULL && base.elems == NULL) || end_at < start_at || end_at > base.length ||
            (updating && (replacement.length != end_at - start_at ||
                          (replacement.pack == NULL && replacement.elems == NULL)))) {
            c->failed = 1;
            value_clear(&base);
            value_clear(&replacement);
            return 0;
        }
        length = updating ? base.length : end_at - start_at;
        cost = ((uint64_t)(updating ? base.length : length) + 63u) / 64u;
        if (length == 0 || !charge(c, expr->start, expr->end, cost == 0 ? 1 : cost)) {
            value_clear(&base);
            value_clear(&replacement);
            return 0;
        }
        if (base.pack != NULL) {
            if (!updating) {
                Pack *sliced = pack_slice(base.pack, start_at, length);
                if (sliced == NULL) {
                    value_clear(&base);
                    value_clear(&replacement);
                    return array_oom(c, expr->start, expr->end);
                }
                out->type = base.type;
                out->length = length;
                out->mod_index = sliced->mod_index;
                out->pack = sliced;
            } else {
                uint32_t index;
                if (!pack_make_unique(&base.pack)) {
                    value_clear(&base);
                    value_clear(&replacement);
                    return array_oom(c, expr->start, expr->end);
                }
                if (replacement.pack != NULL && replacement.pack->stride == base.pack->stride) {
                    pack_write_from(base.pack, start_at, replacement.pack);
                } else {
                    uint32_t count = end_at - start_at;
                    for (index = 0; index < count; index++) {
                        uint64_t word = 0;
                        if (!elem_word(&replacement, index, &word)) {
                            c->failed = 1;
                            value_clear(&base);
                            value_clear(&replacement);
                            return 0;
                        }
                        pack_set(base.pack, start_at + index, word);
                    }
                }
                transfer_array(out, &base);
            }
            value_clear(&base);
            value_clear(&replacement);
            return 1;
        }
        if (!alloc_array(c, &items, length, expr->start, expr->end)) {
            value_clear(&base);
            value_clear(&replacement);
            return 0;
        }
        if (!updating) {
            for (slot = 0; slot < length; slot++) {
                if (!value_clone(c, &items[slot], &base.elems[start_at + slot], expr->start, expr->end)) {
                    value_list_clear(items, length);
                    value_clear(&base);
                    return 0;
                }
            }
            out->type = base.type;
        } else {
            for (slot = 0; slot < length; slot++) {
                const Value *source;
                if (slot >= start_at && slot < end_at) {
                    source = &replacement.elems[slot - start_at];
                } else {
                    source = &base.elems[slot];
                }
                if (!value_clone(c, &items[slot], source, expr->start, expr->end)) {
                    value_list_clear(items, length);
                    value_clear(&base);
                    value_clear(&replacement);
                    return 0;
                }
            }
            out->type = base.type;
        }
        value_clear(&base);
        value_clear(&replacement);
        out->length = length;
        out->elems = items;
        return 1;
    }
    default:
        c->failed = 1;
        return 0;
    }
}

typedef struct TextBuf {
    char *data;
    size_t length;
    size_t cap;
} TextBuf;

static int text_append(TextBuf *buf, const char *bytes, size_t count) {
    size_t need;
    size_t next;
    char *grown;
    if (count == 0) {
        return 1;
    }
    if (buf->length > SIZE_MAX - count - 1) {
        return 0;
    }
    need = buf->length + count + 1;
    if (need > buf->cap) {
        next = buf->cap == 0 ? 64 : buf->cap;
        while (next < need) {
            if (next > SIZE_MAX / 2) {
                next = need;
                break;
            }
            next *= 2;
        }
        grown = realloc(buf->data, next);
        if (grown == NULL) {
            return 0;
        }
        buf->data = grown;
        buf->cap = next;
    }
    memcpy(buf->data + buf->length, bytes, count);
    buf->length += count;
    buf->data[buf->length] = '\0';
    return 1;
}

/* Decimal spelling of an admitted Int. 2^16384-1 is 4933 digits, plus a sign. */
#define MAX_INT_DECIMAL 8192u

static int format_packed_word(TypeKind type, uint64_t word, TextBuf *buf) {
    if (type == TY_INT || type == TY_MOD) {
        uint32_t limbs[2];
        Big big = big_zero();
        char digits[96];
        if (word > 0xffffffffu) {
            limbs[0] = (uint32_t)word;
            limbs[1] = (uint32_t)(word >> 32);
            big.limbs = limbs;
            big.nlimbs = 2;
        } else if (word != 0) {
            limbs[0] = (uint32_t)word;
            big.limbs = limbs;
            big.nlimbs = 1;
        }
        if (!big_format(&big, digits, sizeof digits)) {
            return 0;
        }
        return text_append(buf, digits, strlen(digits));
    }
    {
        char hex[32];
        int width = type_width(type) / 4;
        int length = snprintf(hex, sizeof hex, "0x%0*llx", width, (unsigned long long)word);
        if (length < 0 || (size_t)length >= sizeof hex) {
            return 0;
        }
        return text_append(buf, hex, (size_t)length);
    }
}

static int format_value(const Value *value, TextBuf *buf) {
    uint32_t index;
    if (value->is_tuple) {
        if (value->elems == NULL || !text_append(buf, "(", 1)) {
            return 0;
        }
        for (index = 0; index < value->length; index++) {
            if (index > 0 && !text_append(buf, ", ", 2)) {
                return 0;
            }
            if (!format_value(&value->elems[index], buf)) {
                return 0;
            }
        }
        return text_append(buf, ")", 1);
    }
    if (value->length > 0 && value->pack != NULL) {
        if (!text_append(buf, "[", 1)) {
            return 0;
        }
        for (index = 0; index < value->length; index++) {
            if (index > 0 && !text_append(buf, ", ", 2)) {
                return 0;
            }
            if (!format_packed_word(value->pack->type, pack_get(value->pack, index), buf)) {
                return 0;
            }
        }
        return text_append(buf, "]", 1);
    }
    if (value->length > 0) {
        if (value->elems == NULL || !text_append(buf, "[", 1)) {
            return 0;
        }
        for (index = 0; index < value->length; index++) {
            if (index > 0 && !text_append(buf, ", ", 2)) {
                return 0;
            }
            if (!format_value(&value->elems[index], buf)) {
                return 0;
            }
        }
        return text_append(buf, "]", 1);
    }
    if (value->type == TY_BOOL) {
        const char *spelling = value->word != 0 ? "true" : "false";
        return text_append(buf, spelling, strlen(spelling));
    }
    if (value->type == TY_INT || value->type == TY_MOD) {
        char digits[MAX_INT_DECIMAL];
        if (!big_format(&value->big, digits, sizeof digits)) {
            return 0;
        }
        return text_append(buf, digits, strlen(digits));
    }
    {
        char hex[32];
        int width = type_width(value->type) / 4;
        int length = snprintf(hex, sizeof hex, "0x%0*llx", width, (unsigned long long)value->word);
        if (length < 0 || (size_t)length >= sizeof hex) {
            return 0;
        }
        return text_append(buf, hex, (size_t)length);
    }
}

static int compare_diag(const void *left_ptr, const void *right_ptr) {
    const Diag *left = left_ptr;
    const Diag *right = right_ptr;
    int order;
    if (left->start != right->start) {
        return left->start < right->start ? -1 : 1;
    }
    if (left->end != right->end) {
        return left->end < right->end ? -1 : 1;
    }
    order = strcmp(left->code, right->code);
    if (order != 0) {
        return order;
    }
    order = strcmp(left->message, right->message);
    if (order != 0) {
        return order;
    }
    order = strcmp(left->label, right->label);
    if (order != 0) {
        return order;
    }
    order = strcmp(left->note, right->note);
    if (order != 0) {
        return order;
    }
    return strcmp(left->note2, right->note2);
}

static void line_col(const char *text, size_t length, uint32_t offset, uint32_t *line, uint32_t *column) {
    size_t index = 0;
    uint32_t current_line = 1;
    uint32_t current_column = 1;
    if (offset > length) {
        offset = (uint32_t)length;
    }
    while (index < offset) {
        unsigned char byte = (unsigned char)text[index];
        if (byte == '\n') {
            current_line++;
            current_column = 1;
            index++;
        } else if (byte == '\r') {
            current_line++;
            current_column = 1;
            index++;
            if (index < length && text[index] == '\n' && index < offset) {
                index++;
            }
        } else {
            index += utf8_width(byte);
            current_column++;
        }
    }
    *line = current_line;
    *column = current_column;
}

static unsigned decimal_width_u(uint32_t value) {
    unsigned width = 1;
    while (value >= 10u) {
        value /= 10u;
        width++;
    }
    return width;
}

/* Rust `escape_default` for the characters a diagnostic excerpt escapes. */
static int escape_diag_char(uint32_t cp, char *tmp, size_t cap) {
    int n;
    if (cp == '\\') {
        n = snprintf(tmp, cap, "\\\\");
    } else if (cp == '\t') {
        n = snprintf(tmp, cap, "\\t");
    } else if (cp == '\r') {
        n = snprintf(tmp, cap, "\\r");
    } else if (cp == '\n') {
        n = snprintf(tmp, cap, "\\n");
    } else if (cp == '\'') {
        n = snprintf(tmp, cap, "\\'");
    } else if (cp == ' ' || (cp >= 0x21u && cp <= 0x7eu)) {
        n = snprintf(tmp, cap, "%c", (char)cp);
    } else {
        n = snprintf(tmp, cap, "\\u{%x}", cp);
    }
    return n < 0 ? 0 : n;
}

static uint32_t decode_cp(const char *text, size_t length, size_t index, size_t *next) {
    unsigned char lead;
    size_t width;
    uint32_t cp;
    if (index >= length) {
        *next = index;
        return 0;
    }
    lead = (unsigned char)text[index];
    width = utf8_width(lead);
    if (width > length - index) {
        width = 1;
    }
    if (width == 1) {
        *next = index + 1;
        return lead;
    }
    if (width == 2) {
        cp = ((uint32_t)(lead & 0x1fu) << 6) | ((unsigned char)text[index + 1] & 0x3fu);
    } else if (width == 3) {
        cp = ((uint32_t)(lead & 0x0fu) << 12) | (((unsigned char)text[index + 1] & 0x3fu) << 6) |
             ((unsigned char)text[index + 2] & 0x3fu);
    } else {
        cp = ((uint32_t)(lead & 0x07u) << 18) | (((unsigned char)text[index + 1] & 0x3fu) << 12) |
             (((unsigned char)text[index + 2] & 0x3fu) << 6) | ((unsigned char)text[index + 3] & 0x3fu);
    }
    *next = index + width;
    return cp;
}

static void fputs_sanitized(FILE *out, const char *text);

static void render_span_line(FILE *out, const Compiler *c, uint32_t span_start, uint32_t span_end, char marker,
                             const char *label, int always_label) {
    uint32_t line = 1;
    uint32_t column = 1;
    size_t line_start = 0;
    size_t line_end;
    size_t cursor;
    unsigned gutter;
    size_t index;
    size_t rel;
    size_t cps = 0;
    size_t cp_at[512];
    size_t cp_byte[512];
    size_t window_lo;
    size_t window_hi;
    size_t caret_at = 0;
    size_t caret_width = 1;
    int left_cut = 0;
    int right_cut = 0;
    char excerpt[768];
    size_t used = 0;
    size_t i;
    line_col(c->text, c->length, span_start, &line, &column);
    cursor = 0;
    {
        uint32_t seen = 1;
        while (cursor < c->length && seen < line) {
            if (c->text[cursor] == '\n') {
                seen++;
                cursor++;
                line_start = cursor;
            } else if (c->text[cursor] == '\r') {
                seen++;
                cursor++;
                if (cursor < c->length && c->text[cursor] == '\n') {
                    cursor++;
                }
                line_start = cursor;
            } else {
                cursor++;
            }
        }
    }
    line_end = line_start;
    while (line_end < c->length && c->text[line_end] != '\n' && c->text[line_end] != '\r') {
        line_end++;
    }
    index = line_start;
    while (index < line_end && cps + 1 < sizeof cp_at / sizeof cp_at[0]) {
        size_t next = index;
        cp_byte[cps] = index;
        cp_at[cps] = cps;
        (void)decode_cp(c->text, c->length, index, &next);
        if (next <= index) {
            next = index + 1;
        }
        cps++;
        index = next;
    }
    cp_byte[cps] = line_end;
    rel = 0;
    if (span_start > line_start) {
        size_t target = span_start;
        if (target > line_end) {
            target = line_end;
        }
        while (rel < cps && cp_byte[rel + 1] <= target) {
            rel++;
        }
    }
    window_lo = rel > 40 ? rel - 40 : 0;
    window_hi = rel + 80;
    if (window_hi > cps) {
        window_hi = cps;
    }
    left_cut = window_lo != 0;
    right_cut = window_hi != cps;
    excerpt[0] = '\0';
    if (left_cut) {
        memcpy(excerpt, "... ", 4);
        used = 4;
    }
    for (i = window_lo; i < window_hi; i++) {
        char tmp[16];
        uint32_t cp = decode_cp(c->text, c->length, cp_byte[i], &index);
        int n = escape_diag_char(cp, tmp, sizeof tmp);
        if (used + (size_t)n >= sizeof excerpt) {
            break;
        }
        memcpy(excerpt + used, tmp, (size_t)n);
        used += (size_t)n;
    }
    if (right_cut && used + 4 < sizeof excerpt) {
        memcpy(excerpt + used, " ...", 4);
        used += 4;
    }
    excerpt[used] = '\0';
    caret_at = left_cut ? 4u : 0u;
    for (i = window_lo; i < rel && i < window_hi; i++) {
        char tmp[16];
        uint32_t cp = decode_cp(c->text, c->length, cp_byte[i], &index);
        caret_at += (size_t)escape_diag_char(cp, tmp, sizeof tmp);
    }
    if (span_end > span_start) {
        size_t end_rel = rel;
        size_t end_byte = span_end < line_end ? span_end : line_end;
        size_t width = 0;
        while (end_rel < cps && cp_byte[end_rel] < end_byte && end_rel < window_hi) {
            char tmp[16];
            uint32_t cp = decode_cp(c->text, c->length, cp_byte[end_rel], &index);
            width += (size_t)escape_diag_char(cp, tmp, sizeof tmp);
            end_rel++;
        }
        if (width > 0) {
            caret_width = width;
        }
    }
    gutter = decimal_width_u(line);
    fprintf(out, "%*s |\n%u | %s\n%*s | ", (int)gutter, "", line, excerpt, (int)gutter, "");
    for (i = 0; i < caret_at; i++) {
        fputc(' ', out);
    }
    for (i = 0; i < caret_width; i++) {
        fputc(marker, out);
    }
    if (always_label || (label != NULL && label[0] != '\0')) {
        fputc(' ', out);
        if (label != NULL) {
            fputs_sanitized(out, label);
        }
    }
    fputc('\n', out);
}

/* Rust `push_sanitized_inline`: `\` and non-graphic characters use escape_default. */
static void fputs_sanitized(FILE *out, const char *text) {
    size_t index = 0;
    size_t length = strlen(text);
    while (index < length) {
        size_t next = index;
        uint32_t cp = decode_cp(text, length, index, &next);
        char tmp[16];
        int graphic = cp == ' ' || (cp >= 0x21u && cp <= 0x7eu && cp != '\\');
        if (graphic) {
            fputc((int)cp, out);
        } else {
            escape_diag_char(cp == '\\' ? '\\' : cp, tmp, sizeof tmp);
            fputs(tmp, out);
        }
        if (next <= index) {
            break;
        }
        index = next;
    }
}

static void render_diags(Compiler *c, FILE *out) {
    uint32_t index;
    if (c->ndiags > 1) {
        qsort(c->diags, c->ndiags, sizeof(Diag), compare_diag);
    }
    for (index = 0; index < c->ndiags; index++) {
        Diag *diag = &c->diags[index];
        uint32_t line = 1;
        uint32_t column = 1;
        if (index != 0) {
            fputc('\n', out);
        }
        line_col(c->text, c->length, diag->start, &line, &column);
        fprintf(out, "error[%s]: ", diag->code);
        fputs_sanitized(out, diag->message);
        fprintf(out, "\n --> %s:%u:%u\n", c->filename, line, column);
        render_span_line(out, c, diag->start, diag->end, '^', diag->label, 0);
        if (diag->has_sec) {
            uint32_t sec_line = 1;
            uint32_t sec_column = 1;
            line_col(c->text, c->length, diag->sec_start, &sec_line, &sec_column);
            fprintf(out, " ::: %s:%u:%u\n", c->filename, sec_line, sec_column);
            render_span_line(out, c, diag->sec_start, diag->sec_end, '-', diag->sec_label, 1);
        }
        if (diag->note[0] != '\0') {
            fputs("  = note: ", out);
            fputs_sanitized(out, diag->note);
            fputc('\n', out);
        }
        if (diag->has_note2 && diag->note2[0] != '\0') {
            fputs("  = note: ", out);
            fputs_sanitized(out, diag->note2);
            fputc('\n', out);
        }
        if (diag->has_note3 && diag->note3[0] != '\0') {
            fputs("  = note: ", out);
            fputs_sanitized(out, diag->note3);
            fputc('\n', out);
        }
    }
}

static void write_escaped(FILE *out, const char *text, uint32_t start, uint32_t end) {
    uint32_t index;
    for (index = start; index < end; index++) {
        unsigned char byte = (unsigned char)text[index];
        if (byte == '\\' || byte == '"') {
            fputc('\\', out);
            fputc(byte, out);
        } else if (byte >= 0x20 && byte < 0x7f) {
            fputc(byte, out);
        } else {
            fprintf(out, "\\u{%x}", byte);
        }
    }
}

static int run_lex_command(Compiler *c, FILE *out) {
    size_t index;
    for (index = 0; index < c->ntokens; index++) {
        Token token = c->tokens[index];
        fprintf(out, "%u..%u\t%s\t\"", token.start, token.end, TOKEN_NAMES[token.kind]);
        write_escaped(out, c->text, token.start, token.end);
        fputs("\"\n", out);
    }
    return c->ndiags == 0 ? 0 : 1;
}

static void release_loop_values(Compiler *c) {
    uint32_t index;
    if (c->loop_acc == NULL) {
        return;
    }
    for (index = 0; index < c->nloops; index++) {
        value_clear(&c->loop_acc[index]);
    }
}

static int offset_from_power(Compiler *c, const Big *modulus, uint32_t bit, int *below, uint64_t *offset) {
    Big one = big_zero();
    Big power = big_zero();
    Big diff = big_zero();
    int cmp;
    if (!big_from_u64(&c->arena, 1, &one) || !big_shl(&c->arena, &one, bit, &power)) {
        return 0;
    }
    cmp = big_cmp(&power, modulus);
    if (cmp == 0) {
        *below = 1;
        *offset = 0;
        return 1;
    }
    if (cmp > 0) {
        *below = 1;
        if (!big_sub(&c->arena, &power, modulus, &diff)) {
            return 0;
        }
    } else {
        *below = 0;
        if (!big_sub(&c->arena, modulus, &power, &diff)) {
            return 0;
        }
    }
    if (diff.negative || big_bits(&diff) > 64) {
        return 2;
    }
    *offset = 0;
    if (diff.nlimbs > 0) {
        *offset = diff.limbs[0];
    }
    if (diff.nlimbs > 1) {
        *offset |= (uint64_t)diff.limbs[1] << 32;
    }
    return 1;
}

static int format_modulus(Compiler *c, const Big *modulus, char *buffer, size_t cap) {
    uint32_t bits = big_bits(modulus);
    int below_ok = 0;
    int above_ok = 0;
    int below_flag = 0;
    int above_flag = 0;
    uint64_t below_off = 0;
    uint64_t above_off = 0;
    int use_below = 0;
    uint32_t bit = 0;
    uint64_t offset = 0;
    int nearer_below = 0;
    if (bits <= 64) {
        return big_format(modulus, buffer, cap);
    }
    below_ok = offset_from_power(c, modulus, bits, &below_flag, &below_off);
    if (bits > 0) {
        above_ok = offset_from_power(c, modulus, bits - 1u, &above_flag, &above_off);
    }
    if (below_ok == 0 || above_ok == 0) {
        return 0;
    }
    if (below_ok == 1 && above_ok == 1) {
        use_below = !(above_off < below_off);
    } else if (below_ok == 1) {
        use_below = 1;
    } else if (above_ok == 1) {
        use_below = 0;
    } else {
        uint32_t index;
        int length;
        if (modulus->nlimbs == 0) {
            return 0;
        }
        length = snprintf(buffer, cap, "0x%x", modulus->limbs[modulus->nlimbs - 1]);
        if (length < 0 || (size_t)length >= cap) {
            return 0;
        }
        for (index = modulus->nlimbs - 1u; index > 0; index--) {
            int next = snprintf(buffer + length, cap - (size_t)length, "%08x", modulus->limbs[index - 1u]);
            if (next < 0 || (size_t)length + (size_t)next >= cap) {
                return 0;
            }
            length += next;
        }
        return 1;
    }
    if (use_below) {
        bit = bits;
        offset = below_off;
        nearer_below = below_flag;
    } else {
        bit = bits - 1u;
        offset = above_off;
        nearer_below = above_flag;
    }
    if (offset == 0) {
        return snprintf(buffer, cap, "1 << %u", bit) > 0;
    }
    if (nearer_below) {
        return snprintf(buffer, cap, "(1 << %u) - %llu", bit, (unsigned long long)offset) > 0;
    }
    return snprintf(buffer, cap, "(1 << %u) + %llu", bit, (unsigned long long)offset) > 0;
}

static int format_type(Compiler *c, char *buffer, size_t cap, TypeKind type, uint32_t length, uint16_t mod_index);

static int format_tuple_type(Compiler *c, char *buffer, size_t cap, uint32_t tup0, uint16_t tup_n) {
    size_t used = 0;
    uint16_t index;
    if (cap < 4) {
        return 0;
    }
    buffer[used++] = '(';
    buffer[used] = '\0';
    for (index = 0; index < tup_n; index++) {
        char part[192];
        size_t part_len;
        const TupleElem *elem = &c->telems[tup0 + index];
        if (index > 0) {
            if (used + 2 >= cap) {
                return 0;
            }
            memcpy(buffer + used, ", ", 2);
            used += 2;
        }
        if (!format_type(c, part, sizeof part, elem->kind, elem->length, elem->mod_index)) {
            return 0;
        }
        part_len = strlen(part);
        if (used + part_len + 2 >= cap) {
            return 0;
        }
        memcpy(buffer + used, part, part_len);
        used += part_len;
        buffer[used] = '\0';
    }
    if (used + 2 >= cap) {
        return 0;
    }
    buffer[used++] = ')';
    buffer[used] = '\0';
    return 1;
}

static int format_type(Compiler *c, char *buffer, size_t cap, TypeKind type, uint32_t length, uint16_t mod_index) {
    char modulus_text[160];
    int written;
    if (type == TY_MOD) {
        const Big *modulus = modulus_at(c, mod_index);
        if (modulus == NULL || !format_modulus(c, modulus, modulus_text, sizeof modulus_text)) {
            return 0;
        }
        if (length == 0) {
            written = snprintf(buffer, cap, "Mod[%s]", modulus_text);
        } else {
            written = snprintf(buffer, cap, "Mod[%s]^%u", modulus_text, length);
        }
        return written > 0 && (size_t)written < cap;
    }
    write_type(buffer, cap, type, length);
    return 1;
}

static void stamp_exact_limit(Compiler *c, const Func *func) {
    Diag *diag;
    if (c->ndiags == 0) {
        return;
    }
    diag = &c->diags[c->ndiags - 1];
    if (diag->has_sec || diag->code == NULL || strcmp(diag->code, "ORC0301") != 0) {
        return;
    }
    if (strcmp(diag->message, "exact integer result exceeds the 16384-significant-bit limit") != 0) {
        return;
    }
    diag_add_secondary(c, func->name_start, func->name_end, "evaluation of this function");
}

static void report_step_limit(Compiler *c, const Func *func, uint64_t steps_before) {
    char note[160];
    Diag *diag;
    uint64_t limit = c->step_limit == 0 ? MAX_STEPS : c->step_limit;
    const char *label = steps_before == c->steps ? "evaluation stopped before this function"
                                                 : "evaluation stopped while evaluating this function";
    snprintf(note, sizeof note, "at most %llu evaluation steps are permitted", (unsigned long long)limit);
    add_diag(c, "ORC0301", func->name_start, func->name_end, "reference evaluation step limit exceeded", label, note, 2);
    if (c->ndiags == 0) {
        return;
    }
    diag = &c->diags[c->ndiags - 1];
    copy_text(diag->note2, sizeof diag->note2, "no partial value set is returned");
    diag->has_note2 = 1;
    if (limit < MAX_STEP_LIMIT) {
        copy_text(diag->note3, sizeof diag->note3,
                  "`orangec eval --steps N` sets the budget, up to 1073741824 steps");
        diag->has_note3 = 1;
    }
}

static void note_stat(TextBuf *stats, const Compiler *c, const Func *func, const char *instance, uint64_t used) {
    char line[384];
    const char *unit = used == 1 ? "step" : "steps";
    size_t module_len = (size_t)(c->module_end - c->module_start);
    size_t name_len = (size_t)(func->name_end - func->name_start);
    int wrote;
    if (instance != NULL && instance[0] != '\0') {
        wrote = snprintf(line, sizeof line, "%.*s::%.*s[%s]: %llu %s\n", (int)module_len, c->text + c->module_start,
                         (int)name_len, c->text + func->name_start, instance, (unsigned long long)used, unit);
    } else {
        wrote = snprintf(line, sizeof line, "%.*s::%.*s: %llu %s\n", (int)module_len, c->text + c->module_start,
                         (int)name_len, c->text + func->name_start, (unsigned long long)used, unit);
    }
    if (wrote > 0 && (size_t)wrote < sizeof line) {
        text_append(stats, line, (size_t)wrote);
    }
}

static int evaluate_source(Compiler *c, FILE *out) {
    TextBuf program = {0};
    TextBuf stats = {0};
    uint32_t index;
    if (!ensure_loops(c)) {
        release_loop_values(c);
        return 1;
    }
    for (index = 0; index < c->nfuncs && !c->failed; index++) {
        Func *func = &c->funcs[index];
        Value result;
        TextBuf value = {0};
        char type_text[768];
        int stop = 0;
        if (!func->typed || func->duplicate || func->nparams != 0 || !func->signature_ok) {
            continue;
        }
        if (func->nsizes > 0) {
            uint32_t ordinal;
            for (ordinal = 0; ordinal < func->ninst && !c->failed; ordinal++) {
                Instance *inst = &c->instances[func->inst0 + ordinal];
                char sizes_text[160];
                size_t used = 0;
                uint8_t slot;
                TypeKind saved_kind = func->result;
                uint32_t saved_len = func->result_len;
                uint16_t saved_mod = func->result_mod;
                uint32_t saved_tup0 = func->tup0;
                uint16_t saved_tup_n = func->tup_n;
                sizes_text[0] = '\0';
                c->cur_func = index;
                c->cur_inst = func->inst0 + ordinal;
                c->ncur = func->nsizes;
                memcpy(c->cur_sz, inst->sz, sizeof c->cur_sz);
                func->result = inst->result;
                func->result_len = inst->result_len;
                func->result_mod = inst->result_mod;
                func->tup0 = inst->tup0;
                func->tup_n = inst->tup_n;
                if (tp_func_has_types(func)) {
                    tp_write_values(c, func, inst, sizes_text, sizeof sizes_text);
                } else {
                    for (slot = 0; slot < func->nsizes; slot++) {
                        int wrote = snprintf(sizes_text + used, sizeof sizes_text - used, slot == 0 ? "%lld" : ", %lld",
                                             (long long)inst->sz[slot]);
                        if (wrote > 0 && (size_t)wrote < sizeof sizes_text - used) {
                            used += (size_t)wrote;
                        }
                    }
                }
                memset(&result, 0, sizeof result);
                {
                    uint64_t steps_before = c->steps;
                    if (!eval_function(c, index, NULL, 1, &result)) {
                        value_clear(&result);
                        if (c->step_hit) {
                            report_step_limit(c, func, steps_before);
                        } else {
                            stamp_exact_limit(c, func);
                        }
                        func->result = saved_kind;
                        func->result_len = saved_len;
                        func->result_mod = saved_mod;
                        func->tup0 = saved_tup0;
                        func->tup_n = saved_tup_n;
                        stop = 1;
                        break;
                    }
                    if (c->show_stats) {
                        note_stat(&stats, c, func, sizes_text, c->steps - steps_before);
                    }
                }
                if (func->result != TY_TUPLE && func->result_len == 0 && func->result != TY_INT &&
                    func->result != TY_BOOL && func->result != TY_MOD) {
                    result.type = func->result;
                    if (result.elems == NULL) {
                        result.length = 0;
                    }
                    result.word &= word_mask_of(type_width(func->result));
                }
                if (func->result == TY_MOD && result.length == 0) {
                    result.type = TY_MOD;
                    result.mod_index = func->result_mod;
                }
                if (func->result == TY_TUPLE
                        ? !format_tuple_type(c, type_text, sizeof type_text, func->tup0, func->tup_n)
                        : !format_type(c, type_text, sizeof type_text, func->result, func->result_len,
                                       func->result_mod)) {
                    c->failed = 1;
                    add_diag(c, "ORC0301", func->name_start, func->name_end, "evaluation could not format a type",
                             "resource limit reached", NULL, 2);
                    stop = 1;
                } else if (!format_value(&result, &value) ||
                           !text_append(&program, c->text + c->module_start,
                                        (size_t)(c->module_end - c->module_start)) ||
                           !text_append(&program, "::", 2) ||
                           !text_append(&program, c->text + func->name_start,
                                        (size_t)(func->name_end - func->name_start)) ||
                           !text_append(&program, "[", 1) ||
                           !text_append(&program, sizes_text, strlen(sizes_text)) || !text_append(&program, "]: ", 3) ||
                           !text_append(&program, type_text, strlen(type_text)) || !text_append(&program, " = ", 3) ||
                           !text_append(&program, value.data == NULL ? "" : value.data, value.length) ||
                           !text_append(&program, "\n", 1)) {
                    c->failed = 1;
                    add_diag(c, "ORC0301", func->name_start, func->name_end, "evaluation could not format a value",
                             "resource limit reached", NULL, 2);
                    stop = 1;
                }
                free(value.data);
                value.data = NULL;
                value.length = 0;
                value.cap = 0;
                value_clear(&result);
                func->result = saved_kind;
                func->result_len = saved_len;
                func->result_mod = saved_mod;
                func->tup0 = saved_tup0;
                func->tup_n = saved_tup_n;
                if (stop) {
                    break;
                }
            }
            if (stop) {
                break;
            }
            continue;
        }
        memset(&result, 0, sizeof result);
        c->cur_func = index;
        c->cur_inst = UINT32_MAX;
        c->ncur = 0;
        memset(c->cur_sz, 0, sizeof c->cur_sz);
        {
            uint64_t steps_before = c->steps;
            if (!eval_function(c, index, NULL, 1, &result)) {
                value_clear(&result);
                if (c->step_hit) {
                    report_step_limit(c, func, steps_before);
                } else {
                    stamp_exact_limit(c, func);
                }
                break;
            }
            if (c->show_stats) {
                note_stat(&stats, c, func, NULL, c->steps - steps_before);
            }
        }
        if (func->result != TY_TUPLE && func->result_len == 0 && func->result != TY_INT && func->result != TY_BOOL &&
            func->result != TY_MOD) {
            result.type = func->result;
            if (result.elems == NULL) {
                result.length = 0;
            }
            result.word &= word_mask_of(type_width(func->result));
        }
        if (func->result == TY_MOD && result.length == 0) {
            result.type = TY_MOD;
            result.mod_index = func->result_mod;
        }
        if (func->result == TY_TUPLE
                ? !format_tuple_type(c, type_text, sizeof type_text, func->tup0, func->tup_n)
                : !format_type(c, type_text, sizeof type_text, func->result, func->result_len, func->result_mod)) {
            c->failed = 1;
            add_diag(c, "ORC0301", func->name_start, func->name_end, "evaluation could not format a type",
                     "resource limit reached", NULL, 2);
            break;
        }
        if (!format_value(&result, &value) ||
            !text_append(&program, c->text + c->module_start, (size_t)(c->module_end - c->module_start)) ||
            !text_append(&program, "::", 2) ||
            !text_append(&program, c->text + func->name_start, (size_t)(func->name_end - func->name_start)) ||
            !text_append(&program, ": ", 2) ||
            !text_append(&program, type_text, strlen(type_text)) ||
            !text_append(&program, " = ", 3) ||
            !text_append(&program, value.data == NULL ? "" : value.data, value.length) ||
            !text_append(&program, "\n", 1)) {
            c->failed = 1;
            add_diag(c, "ORC0301", func->name_start, func->name_end, "evaluation could not format a value",
                     "resource limit reached", NULL, 2);
            stop = 1;
        }
        free(value.data);
        value_clear(&result);
        if (stop) {
            break;
        }
    }
    if (!c->failed && program.data != NULL) {
        fwrite(program.data, 1, program.length, out);
    }
    if (!c->failed && c->show_stats) {
        char total[96];
        uint64_t limit = c->step_limit == 0 ? MAX_STEPS : c->step_limit;
        int wrote = snprintf(total, sizeof total, "total: %llu of %llu steps\n", (unsigned long long)c->steps,
                             (unsigned long long)limit);
        if (stats.data != NULL) {
            fwrite(stats.data, 1, stats.length, stderr);
        }
        if (wrote > 0) {
            fputs(total, stderr);
        }
    }
    free(program.data);
    free(stats.data);
    release_loop_values(c);
    return c->failed ? 1 : 0;
}

static char *read_path(const char *path, size_t *length, char *error, size_t error_cap);

static Compiler *compiler_new(char *text, size_t length, const char *filename, int own_text, int own_filename) {
    Compiler *compiler = calloc(1, sizeof *compiler);
    if (compiler == NULL) {
        return NULL;
    }
    compiler->text = text;
    compiler->length = length;
    compiler->filename = filename;
    compiler->own_text = own_text;
    compiler->own_filename = own_filename;
    compiler->cur_func = UINT32_MAX;
    compiler->cur_inst = UINT32_MAX;
    compiler->step_limit = MAX_STEPS;
    if (!arena_init(&compiler->arena, ARENA_BYTES)) {
        free(compiler);
        return NULL;
    }
    return compiler;
}

static void compiler_free(Compiler *compiler) {
    if (compiler == NULL) {
        return;
    }
    release_loop_values(compiler);
    arena_dispose(&compiler->arena);
    free(compiler->tokens);
    free(compiler->exprs);
    free(compiler->args);
    free(compiler->funcs);
    free(compiler->params);
    free(compiler->locals);
    free(compiler->block_locals);
    free(compiler->edges);
    free(compiler->loops);
    free(compiler->cond_arms);
    free(compiler->loop_k);
    free(compiler->loop_acc);
    free(compiler->requested);
    free(compiler->types);
    free(compiler->sites);
    free(compiler->tp_sites);
    free(compiler->stamps);
    free(compiler->telems);
    free(compiler->moduli);
    free(compiler->finished);
    free(compiler->instances);
    free(compiler->iparams);
    if (compiler->own_text) {
        free(compiler->text);
    }
    if (compiler->own_filename) {
        free((char *)compiler->filename);
    }
    free(compiler);
}

static void program_free(Program *program) {
    int index;
    if (program == NULL) {
        return;
    }
    for (index = 0; index < program->nmods; index++) {
        compiler_free(program->mods[index]);
    }
    free(program);
}

static void clear_diags(Compiler *compiler) {
    compiler->ndiags = 0;
    compiler->lex_diags = 0;
    compiler->parse_diags = 0;
    compiler->sema_diags = 0;
    compiler->lex_limited = 0;
    compiler->parse_limited = 0;
    compiler->sema_limited = 0;
}

static int module_name_eq(const Compiler *mod, const char *text, uint32_t start, uint32_t end) {
    size_t length = (size_t)(end - start);
    return (size_t)(mod->module_end - mod->module_start) == length &&
           memcmp(mod->text + mod->module_start, text + start, length) == 0;
}

static int find_module_named(const Program *program, const char *text, uint32_t start, uint32_t end, uint16_t *index) {
    int cursor;
    for (cursor = 0; cursor < program->nmods; cursor++) {
        if (module_name_eq(program->mods[cursor], text, start, end)) {
            *index = (uint16_t)cursor;
            return 1;
        }
    }
    return 0;
}

static void report_namesakes(Program *program, uint16_t module) {
    Compiler *mod = program->mods[module];
    int other;
    char spelling[128];
    span_copy(spelling, sizeof spelling, mod->text, mod->module_start, mod->module_end);
    for (other = 0; other < program->nmods; other++) {
        Compiler *candidate;
        Compiler *later;
        char message[384];
        if (other == (int)module) {
            continue;
        }
        candidate = program->mods[other];
        if (!module_name_eq(candidate, mod->text, mod->module_start, mod->module_end)) {
            continue;
        }
        later = other > (int)module ? candidate : mod;
        snprintf(message, sizeof message, "duplicate module `%s`", spelling);
        add_diag(later, "ORC0231", later->module_start, later->module_end, message,
                 "this module repeats the name of a module of the program",
                 "a `use` names one module, so no other supplied module may share the name of a module of the program",
                 2);
        program->graph_error = 1;
    }
}

static void resolve_uses(Program *program, uint16_t module) {
    Compiler *mod = program->mods[module];
    uint16_t index;
    for (index = 0; index < mod->nuses; index++) {
        UseDecl *use = &mod->uses[index];
        uint16_t earlier;
        int repeated = 0;
        char spelling[128];
        char message[384];
        span_copy(spelling, sizeof spelling, mod->text, use->name_start, use->name_end);
        for (earlier = 0; earlier < index; earlier++) {
            if (same_span(mod, mod->uses[earlier].name_start, mod->uses[earlier].name_end, use->name_start,
                          use->name_end)) {
                repeated = 1;
                break;
            }
        }
        if (repeated) {
            snprintf(message, sizeof message, "module `%s` is used twice", spelling);
            add_diag(mod, "ORC0231", use->span_start, use->span_end, message,
                     "this declaration repeats an earlier `use`", "a module names each module it uses once", 2);
            diag_add_secondary(mod, mod->uses[earlier].span_start, mod->uses[earlier].span_end, "first used here");
            program->use_target[module][index] = UINT16_MAX;
            program->graph_error = 1;
            continue;
        }
        if (same_span(mod, mod->module_start, mod->module_end, use->name_start, use->name_end)) {
            snprintf(message, sizeof message, "module `%s` uses itself", spelling);
            add_diag(mod, "ORC0230", use->span_start, use->span_end, message, "this `use` names its own module",
                     "a module calls its own functions without a module name, as in `f(x)`", 2);
            program->use_target[module][index] = UINT16_MAX;
            program->graph_error = 1;
            continue;
        }
        {
            uint16_t target = UINT16_MAX;
            if (find_module_named(program, mod->text, use->name_start, use->name_end, &target)) {
                program->use_target[module][index] = target;
            } else {
                snprintf(message, sizeof message, "no module named `%s` in this program", spelling);
                add_diag(mod, "ORC0228", use->name_start, use->name_end, message, "unknown module",
                         "a `use` declaration names another module of the program; `orangec` reads the module `NAME` "
                         "from the file `NAME.or` beside the file that uses it",
                         2);
                program->use_target[module][index] = UINT16_MAX;
                program->graph_error = 1;
            }
        }
    }
}

static void append_route(char *route, size_t cap, size_t *used, const char *text) {
    size_t length = strlen(text);
    if (*used >= cap) {
        return;
    }
    if (length >= cap - *used) {
        length = cap - *used - 1;
    }
    if (length > 0) {
        memcpy(route + *used, text, length);
        *used += length;
    }
    route[*used] = '\0';
}

static void report_cycle(Program *program, const uint16_t *path_mod, int path_len, uint16_t node, uint16_t use_index,
                         uint16_t target) {
    Compiler *mod = program->mods[node];
    UseDecl *use = &mod->uses[use_index];
    char route[384];
    char message[640];
    char name[128];
    size_t used = 0;
    int start = 0;
    int cursor;
    route[0] = '\0';
    for (cursor = 0; cursor < path_len; cursor++) {
        if (path_mod[cursor] == target) {
            start = cursor;
            break;
        }
    }
    for (cursor = start; cursor < path_len; cursor++) {
        Compiler *step = program->mods[path_mod[cursor]];
        if (cursor - start >= 8) {
            append_route(route, sizeof route, &used, " -> ...");
            break;
        }
        if (cursor != start) {
            append_route(route, sizeof route, &used, " -> ");
        }
        span_copy(name, sizeof name, step->text, step->module_start, step->module_end);
        append_route(route, sizeof route, &used, "`");
        append_route(route, sizeof route, &used, name);
        append_route(route, sizeof route, &used, "`");
    }
    span_copy(name, sizeof name, program->mods[target]->text, program->mods[target]->module_start,
              program->mods[target]->module_end);
    snprintf(message, sizeof message, "module cycle %s -> `%s`", route, name);
    add_diag(mod, "ORC0230", use->span_start, use->span_end, message, "this `use` closes the cycle",
             "modules may not depend on each other in a cycle; move the functions they share into a module that both use",
             2);
    program->graph_error = 1;
}

static int link_program(Program *program) {
    uint8_t state[MAX_PROGRAM_SLOTS];
    uint16_t path_mod[MAX_MODULES];
    uint16_t path_use[MAX_MODULES];
    int path_len = 0;
    int entered = 0;
    int slot;
    int use_slot;
    memset(state, 0, sizeof state);
    for (slot = 0; slot < MAX_PROGRAM_SLOTS; slot++) {
        for (use_slot = 0; use_slot < MAX_USES; use_slot++) {
            program->use_target[slot][use_slot] = UINT16_MAX;
        }
    }
    if (program->nmods == 0) {
        return 0;
    }
    state[0] = 1;
    path_mod[0] = 0;
    path_use[0] = 0;
    path_len = 1;
    entered = 1;
    report_namesakes(program, 0);
    resolve_uses(program, 0);
    while (path_len > 0) {
        uint16_t node = path_mod[path_len - 1];
        uint16_t next = path_use[path_len - 1];
        Compiler *mod = program->mods[node];
        uint16_t target;
        if (next >= mod->nuses) {
            state[node] = 2;
            if (program->norder < MAX_MODULES) {
                program->order[program->norder++] = node;
            }
            path_len--;
            continue;
        }
        target = program->use_target[node][next];
        path_use[path_len - 1] = (uint16_t)(next + 1);
        if (target == UINT16_MAX || target >= program->nmods) {
            continue;
        }
        if (state[target] == 0) {
            if (entered >= MAX_MODULES) {
                Compiler *root = program->mods[0];
                for (slot = 0; slot < program->nmods; slot++) {
                    clear_diags(program->mods[slot]);
                }
                program->norder = 0;
                add_diag(root, "ORC0209", root->module_start, root->module_end,
                         "semantic analysis resource limit exceeded", "program reaches more than 64 modules",
                         "semantic analysis stopped without producing Core", 2);
                program->graph_error = 1;
                return 0;
            }
            entered++;
            state[target] = 1;
            path_mod[path_len] = target;
            path_use[path_len] = 0;
            path_len++;
            report_namesakes(program, target);
            resolve_uses(program, target);
        } else if (state[target] == 1) {
            report_cycle(program, path_mod, path_len, node, next, target);
        }
    }
    return program->graph_error ? 0 : 1;
}

static void print_use_note(FILE *err, const char *name, const char *user) {
    fprintf(err, "  = note: `use %s;` in module `%s` reads the module `%s` from this file\n", name, user, name);
}

static char *module_path(const char *root_path, const char *name) {
    const char *slash;
    size_t dir_len;
    size_t name_len = strlen(name);
    size_t total;
    char *path;
    int bare = 0;
    int root_dir = 0;
    if (strcmp(root_path, "-") == 0 || strchr(root_path, '/') == NULL) {
        bare = 1;
        dir_len = 0;
    } else {
        slash = strrchr(root_path, '/');
        if (slash == root_path) {
            root_dir = 1;
            dir_len = 1;
        } else {
            dir_len = (size_t)(slash - root_path);
        }
    }
    total = dir_len + name_len + 8;
    path = malloc(total);
    if (path == NULL) {
        return NULL;
    }
    if (bare) {
        memcpy(path, name, name_len);
        memcpy(path + name_len, ".or", 4);
    } else if (root_dir) {
        path[0] = '/';
        memcpy(path + 1, name, name_len);
        memcpy(path + 1 + name_len, ".or", 4);
    } else {
        memcpy(path, root_path, dir_len);
        path[dir_len] = '/';
        memcpy(path + dir_len + 1, name, name_len);
        memcpy(path + dir_len + 1 + name_len, ".or", 4);
    }
    return path;
}

static int name_requested(const Program *program, const char *name) {
    int index;
    for (index = 1; index < program->nmods; index++) {
        if (program->mods[index]->requested != NULL && strcmp(program->mods[index]->requested, name) == 0) {
            return 1;
        }
    }
    return 0;
}

static int load_one(Program *program, const char *path, const char *name, const char *user, FILE *err) {
    char error[1024];
    char *text;
    size_t length = 0;
    char *stored_path;
    char *stored_name;
    Compiler *mod;
    text = read_path(path, &length, error, sizeof error);
    if (text == NULL) {
        int missing = errno == ENOENT;
        fprintf(err, "error[ORC1001]: %s\n", error);
        if (missing) {
            fprintf(err, "  = note: file was not found\n");
        }
        print_use_note(err, name, user);
        return 0;
    }
    if (!utf8_ok((const unsigned char *)text, length)) {
        fprintf(err, "error[ORC1002]: source file `%s` is not valid UTF-8\n", path);
        print_use_note(err, name, user);
        free(text);
        return 0;
    }
    stored_path = malloc(strlen(path) + 1);
    stored_name = malloc(strlen(name) + 1);
    if (stored_path == NULL || stored_name == NULL) {
        free(stored_path);
        free(stored_name);
        free(text);
        fprintf(err, "error[ORC0008]: compiler could not reserve its work area\n");
        return 0;
    }
    memcpy(stored_path, path, strlen(path) + 1);
    memcpy(stored_name, name, strlen(name) + 1);
    mod = compiler_new(text, length, stored_path, 1, 1);
    if (mod == NULL) {
        free(stored_path);
        free(stored_name);
        free(text);
        fprintf(err, "error[ORC0008]: compiler could not reserve its work area\n");
        return 0;
    }
    mod->requested = stored_name;
    mod->step_limit = program->mods[0]->step_limit;
    mod->program = program;
    mod->self_index = (uint16_t)program->nmods;
    program->mods[program->nmods++] = mod;
    lex_source(mod);
    if (mod->lex_diags > 0 || mod->resource) {
        render_diags(mod, err);
        return 0;
    }
    parse_source(mod);
    if (mod->parse_diags > 0 || mod->resource) {
        render_diags(mod, err);
        print_use_note(err, name, user);
        return 0;
    }
    return 1;
}

static int load_used_modules(Program *program, const char *root_path, FILE *err) {
    Compiler *root = program->mods[0];
    int next = 0;
    for (;;) {
        Compiler *user;
        char user_name[256];
        uint16_t use_index;
        if (next == 0) {
            user = root;
        } else if (next < program->nmods) {
            user = program->mods[next];
            if (user->requested == NULL || !span_is(user, user->module_start, user->module_end, user->requested)) {
                next++;
                continue;
            }
        } else {
            return 1;
        }
        span_copy(user_name, sizeof user_name, user->text, user->module_start, user->module_end);
        for (use_index = 0; use_index < user->nuses; use_index++) {
            UseDecl *use = &user->uses[use_index];
            size_t name_len = (size_t)(use->name_end - use->name_start);
            char name[256];
            char *path;
            if (name_len == 0 || name_len >= sizeof name) {
                span_copy(name, sizeof name, user->text, use->name_start, use->name_end);
                fprintf(err, "error[ORC1001]: could not read source file `%s.or`\n", name);
                print_use_note(err, name, user_name);
                return 0;
            }
            memcpy(name, user->text + use->name_start, name_len);
            name[name_len] = '\0';
            if (((size_t)(root->module_end - root->module_start) == name_len &&
                 memcmp(root->text + root->module_start, name, name_len) == 0) ||
                name_requested(program, name)) {
                continue;
            }
            if (program->nmods - 1 >= MAX_MODULES) {
                return 1;
            }
            path = module_path(root_path, name);
            if (path == NULL) {
                fprintf(err, "error[ORC0008]: compiler could not reserve its work area\n");
                return 0;
            }
            if (!load_one(program, path, name, user_name, err)) {
                free(path);
                return 0;
            }
            free(path);
        }
        next++;
    }
}

static void render_program_diags(Program *program, FILE *err, int dependency_order) {
    int index;
    int started = 0;
    int count = dependency_order ? program->norder : program->nmods;
    for (index = 0; index < count; index++) {
        Compiler *mod = dependency_order ? program->mods[program->order[index]] : program->mods[index];
        if (mod->ndiags > 0) {
            if (started) {
                fputc('\n', err);
            }
            render_diags(mod, err);
            started = 1;
        }
    }
}

static int compile_text(char *text, size_t length, const char *filename, int command, uint64_t step_limit, int show_stats,
                        FILE *out, FILE *err) {
    Program *program = calloc(1, sizeof *program);
    Compiler *root;
    int status = 1;
    int index;
    if (program == NULL) {
        fprintf(err, "error[ORC0008]: compiler could not reserve its work area\n");
        return 1;
    }
    root = compiler_new(text, length, filename, 0, 0);
    if (root == NULL) {
        free(program);
        fprintf(err, "error[ORC0008]: compiler could not reserve its work area\n");
        return 1;
    }
    program->mods[0] = root;
    program->nmods = 1;
    root->step_limit = step_limit == 0 ? MAX_STEPS : step_limit;
    root->show_stats = show_stats;
    root->program = program;
    root->self_index = 0;
    lex_source(root);
    if (command == 2) {
        status = run_lex_command(root, out);
        render_diags(root, err);
        program_free(program);
        return status;
    }
    if (root->lex_diags > 0 || root->resource) {
        render_diags(root, err);
        program_free(program);
        return 1;
    }
    parse_source(root);
    if (root->parse_diags > 0 || root->resource) {
        render_diags(root, err);
        program_free(program);
        return 1;
    }
    if (root->nuses > 0 && !load_used_modules(program, filename, err)) {
        program_free(program);
        return 1;
    }
    if (!link_program(program)) {
        render_program_diags(program, err, 0);
        program_free(program);
        return 1;
    }
    status = 0;
    for (index = 0; index < program->norder; index++) {
        Compiler *mod = program->mods[program->order[index]];
        analyze(mod);
        if (mod->ndiags > 0 || mod->failed) {
            status = 1;
        }
    }
    if (status != 0) {
        render_program_diags(program, err, 1);
        program_free(program);
        return 1;
    }
    if (command == 1) {
        status = evaluate_source(root, out);
        if (status != 0) {
            int seen = 0;
            int mod_index;
            for (mod_index = 0; mod_index < program->nmods; mod_index++) {
                if (program->mods[mod_index] != NULL && program->mods[mod_index]->ndiags > 0) {
                    seen = 1;
                    break;
                }
            }
            /* A failing evaluation with nothing to print is a compiler bug.
               Say so, instead of exiting 1 with empty stdout and stderr. */
            if (!seen) {
                fputs("internal error: evaluation failed without a diagnostic\n", err);
            }
            render_program_diags(program, err, 0);
        }
    } else {
        status = 0;
    }
    program_free(program);
    return status;
}

static char *read_path(const char *path, size_t *length, char *error, size_t error_cap) {
    FILE *file;
    long size;
    char *buffer;
    size_t read_bytes;
    if (strcmp(path, "-") == 0) {
        size_t cap = 4096;
        size_t used = 0;
        buffer = malloc(cap);
        if (buffer == NULL) {
            snprintf(error, error_cap, "could not allocate standard input");
            return NULL;
        }
        for (;;) {
            size_t got;
            if (used == cap) {
                size_t next = cap * 2;
                char *grown;
                if (next > MAX_SOURCE_BYTES + 1) {
                    next = MAX_SOURCE_BYTES + 1;
                }
                if (next == cap) {
                    snprintf(error, error_cap, "source exceeds the %u-byte input limit", MAX_SOURCE_BYTES);
                    free(buffer);
                    return NULL;
                }
                grown = realloc(buffer, next);
                if (grown == NULL) {
                    free(buffer);
                    snprintf(error, error_cap, "could not allocate standard input");
                    return NULL;
                }
                buffer = grown;
                cap = next;
            }
            got = fread(buffer + used, 1, cap - used, stdin);
            used += got;
            if (got == 0) {
                break;
            }
        }
        if (used > MAX_SOURCE_BYTES) {
            snprintf(error, error_cap, "source exceeds the %u-byte input limit", MAX_SOURCE_BYTES);
            free(buffer);
            return NULL;
        }
        *length = used;
        return buffer;
    }
    file = fopen(path, "rb");
    if (file == NULL) {
        snprintf(error, error_cap, "could not read source file `%s`", path);
        return NULL;
    }
    if (fseek(file, 0, SEEK_END) != 0) {
        fclose(file);
        snprintf(error, error_cap, "could not read source file `%s`", path);
        return NULL;
    }
    size = ftell(file);
    if (size < 0) {
        fclose(file);
        snprintf(error, error_cap, "could not read source file `%s`", path);
        return NULL;
    }
    if ((unsigned long)size > MAX_SOURCE_BYTES) {
        fclose(file);
        snprintf(error, error_cap, "source file `%s` exceeds the %u-byte input limit", path, MAX_SOURCE_BYTES);
        return NULL;
    }
    if (fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        snprintf(error, error_cap, "could not read source file `%s`", path);
        return NULL;
    }
    buffer = malloc((size_t)size + 1);
    if (buffer == NULL) {
        fclose(file);
        snprintf(error, error_cap, "could not allocate source file `%s`", path);
        return NULL;
    }
    read_bytes = fread(buffer, 1, (size_t)size, file);
    fclose(file);
    if (read_bytes != (size_t)size) {
        free(buffer);
        snprintf(error, error_cap, "could not read source file `%s`", path);
        return NULL;
    }
    buffer[read_bytes] = '\0';
    *length = read_bytes;
    return buffer;
}

static void print_usage(FILE *out) {
    fputs(
        "Usage: orangec <check|eval|lex> <FILE>\n"
        "       orangec --self-test\n"
        "\n"
        "Standalone C frontend for the Orange 2026 expression, binding,\n"
        "conversion, array, loop, conditional, lookup, module, residue, block, tuple, byte, and size fragment.\n"
        "It does not use the Rust compiler.\n"
        "\n"
        "Commands:\n"
        "  check    Lex, parse, and check one program\n"
        "  eval     Check one program and reference-evaluate its root\n"
        "  lex      Print the token stream of one source\n"
        "\n"
        "  --self-test  Run exact-integer self-tests\n"
        "  -h, --help   Print this help\n"
        "  -V, --version  Print the implemented slice\n",
        out);
}

static int parse_step_budget(const char *text, uint64_t *out) {
    size_t index;
    uint64_t value = 0;
    if (text == NULL || text[0] == '\0' || text[0] == '0') {
        return 0;
    }
    for (index = 0; text[index] != '\0'; index++) {
        unsigned digit;
        if (text[index] < '0' || text[index] > '9') {
            return 0;
        }
        digit = (unsigned)(text[index] - '0');
        if (value > (MAX_STEP_LIMIT - digit) / 10u) {
            return 0;
        }
        value = value * 10u + digit;
    }
    if (value < 1 || value > MAX_STEP_LIMIT) {
        return 0;
    }
    *out = value;
    return 1;
}

static void reject_steps(const char *message) {
    fprintf(stderr, "orangec: %s\n", message);
    print_usage(stderr);
}

int orange_main(int argc, char **argv) {
    int command = -1;
    const char *path = NULL;
    char *text;
    size_t length = 0;
    char error[256];
    int index;
    int steps_seen = 0;
    int show_stats = 0;
    uint64_t step_limit = MAX_STEPS;
    for (index = 1; index < argc; index++) {
        if (strcmp(argv[index], "-h") == 0 || strcmp(argv[index], "--help") == 0) {
            print_usage(stdout);
            return 0;
        }
        if (strcmp(argv[index], "-V") == 0 || strcmp(argv[index], "--version") == 0) {
            fputs("orangec (standalone C) slice S3p\n", stdout);
            return 0;
        }
        if (strcmp(argv[index], "--self-test") == 0) {
            if (!pack_self_test()) {
                fputs("pack self-test failed\n", stderr);
                return 1;
            }
            if (!bigint_self_test()) {
                fputs("bigint self-test failed\n", stderr);
                return 1;
            }
            fputs("bigint self-test passed\n", stdout);
            return 0;
        }
        if (strcmp(argv[index], "--edition") == 0) {
            if (index + 1 >= argc || strcmp(argv[index + 1], "2026") != 0) {
                fputs("error[ORC0102]: the standalone compiler admits only edition 2026\n", stderr);
                return 2;
            }
            index++;
            continue;
        }
        if (strcmp(argv[index], "--stats") == 0) {
            show_stats = 1;
            continue;
        }
        if (strcmp(argv[index], "--steps") == 0 || strncmp(argv[index], "--steps=", 8) == 0) {
            const char *value;
            uint64_t parsed = 0;
            if (steps_seen) {
                reject_steps("option `--steps` may be specified at most once");
                return 2;
            }
            if (argv[index][7] == '=') {
                value = argv[index] + 8;
            } else if (index + 1 >= argc) {
                reject_steps("option `--steps` requires a value");
                return 2;
            } else {
                index++;
                value = argv[index];
            }
            if (!parse_step_budget(value, &parsed)) {
                reject_steps("option `--steps` takes a number of steps from 1 through 1073741824");
                return 2;
            }
            steps_seen = 1;
            step_limit = parsed;
            continue;
        }
        if (argv[index][0] == '-' && strcmp(argv[index], "-") != 0) {
            fputs("error: unknown option\n", stderr);
            print_usage(stderr);
            return 2;
        }
        if (command < 0) {
            if (strcmp(argv[index], "check") == 0) {
                command = 0;
            } else if (strcmp(argv[index], "eval") == 0) {
                command = 1;
            } else if (strcmp(argv[index], "lex") == 0) {
                command = 2;
            } else {
                fputs("error: expected check, eval, or lex\n", stderr);
                print_usage(stderr);
                return 2;
            }
            continue;
        }
        if (path != NULL) {
            fputs("error: the standalone compiler accepts one source file\n", stderr);
            return 2;
        }
        path = argv[index];
    }
    if (command >= 0 && command != 1 && steps_seen) {
        reject_steps("option `--steps` applies only to eval, test, and replay");
        return 2;
    }
    if (command >= 0 && command != 1 && show_stats) {
        reject_steps("option `--stats` applies only to eval, test, and replay");
        return 2;
    }
    if (command < 0 || path == NULL) {
        print_usage(stderr);
        return 2;
    }
    text = read_path(path, &length, error, sizeof error);
    if (text == NULL) {
        fprintf(stderr, "error[ORC1001]: %s\n", error);
        return 1;
    }
    if (!utf8_ok((const unsigned char *)text, length)) {
        fprintf(stderr, "error[ORC1002]: source file `%s` is not valid UTF-8\n", path);
        free(text);
        return 1;
    }
    {
        int status = compile_text(text, length, path, command, step_limit, show_stats, stdout, stderr);
        free(text);
        return status;
    }
}
#endif
