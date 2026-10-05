#include "compile.h"

#include "bigint.h"

#include <ctype.h>
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_SOURCE_BYTES (16u * 1024u * 1024u)
#define MAX_TOKENS 262144u
#define MAX_EXPRS 262144u
#define MAX_ORDINARY_DIAGS 100u
#define MAX_NESTING 64
#define MAX_HEIGHT 256
#define MAX_PARAMS 64
#define MAX_ARGS 256
#define MAX_BINDINGS 256
#define MAX_STEPS 1048576u
#define MAX_CALL_DEPTH 256
#define MAX_ARRAY_LENGTH 256u
#define MAX_ARRAY_ELEMENTS 256u
#define MAX_LOOP_BOUND 65536u
#define MAX_OPEN_LOOPS 64
#define ARENA_BYTES (16u * 1024u * 1024u)
#define MAX_MODULES 64
#define MAX_PROGRAM_SLOTS 65
#define MAX_USES 64
#define MAX_TYPE_DECLS 64
#define MAX_TYPE_SITES 4096
#define MAX_MODULI 256
#define MAX_MODULUS_BITS 521

typedef enum TokenKind {
    TK_EOF,
    TK_IDENT,
    TK_INT,
    TK_STRING,
    TK_HEX,
    TK_EDITION,
    TK_MODULE,
    TK_SPEC,
    TK_IMPL,
    TK_GAME,
    TK_PROOF,
    TK_CLAIM,
    TK_LPAREN,
    TK_RPAREN,
    TK_LBRACE,
    TK_RBRACE,
    TK_LBRACKET,
    TK_RBRACKET,
    TK_COMMA,
    TK_COLON,
    TK_SEMI,
    TK_DOT,
    TK_DOTDOT,
    TK_COLONCOLON,
    TK_PLUS,
    TK_PLUSPLUS,
    TK_MINUS,
    TK_STAR,
    TK_SLASH,
    TK_PERCENT,
    TK_AMP,
    TK_AMPAMP,
    TK_PIPE,
    TK_PIPEPIPE,
    TK_CARET,
    TK_TILDE,
    TK_BANG,
    TK_EQUAL,
    TK_LESS,
    TK_GREATER,
    TK_EQEQ,
    TK_BANGEQ,
    TK_LESSEQ,
    TK_GREATEREQ,
    TK_LSHIFT,
    TK_RSHIFT,
    TK_ROL,
    TK_ROR,
    TK_ARROW,
    TK_FATARROW,
    TK_QUESTION
} TokenKind;

static const char *TOKEN_NAMES[] = {
    "EOF",
    "IDENTIFIER",
    "INTEGER",
    "STRING",
    "HEX_STRING",
    "KW_EDITION",
    "KW_MODULE",
    "KW_SPEC",
    "KW_IMPL",
    "KW_GAME",
    "KW_PROOF",
    "KW_CLAIM",
    "LEFT_PAREN",
    "RIGHT_PAREN",
    "LEFT_BRACE",
    "RIGHT_BRACE",
    "LEFT_BRACKET",
    "RIGHT_BRACKET",
    "COMMA",
    "COLON",
    "SEMICOLON",
    "DOT",
    "DOT_DOT",
    "DOUBLE_COLON",
    "PLUS",
    "PLUS_PLUS",
    "MINUS",
    "STAR",
    "SLASH",
    "PERCENT",
    "AMPERSAND",
    "AMP_AMP",
    "PIPE",
    "PIPE_PIPE",
    "CARET",
    "TILDE",
    "BANG",
    "EQUAL",
    "LESS",
    "GREATER",
    "EQUAL_EQUAL",
    "BANG_EQUAL",
    "LESS_EQUAL",
    "GREATER_EQUAL",
    "LESS_LESS",
    "GREATER_GREATER",
    "LESS_LESS_LESS",
    "GREATER_GREATER_GREATER",
    "ARROW",
    "FAT_ARROW",
    "QUESTION",
};

typedef enum TypeKind { TY_NONE = 0, TY_INT, TY_BOOL, TY_W8, TY_W16, TY_W32, TY_W64, TY_MOD } TypeKind;

typedef enum ExprKind {
    EX_NONE = 0,
    EX_LIT,
    EX_NAME,
    EX_CALL,
    EX_UNARY,
    EX_BINARY,
    EX_SHIFT,
    EX_CONV,
    EX_GROUP,
    EX_ARRAY,
    EX_INDEX,
    EX_SELECT,
    EX_UPDATE,
    EX_FILL,
    EX_LOOP,
    EX_LOOP_INDEX,
    EX_ACCUM,
    EX_COND
} ExprKind;

typedef struct DeclaredType {
    TypeKind kind;
    uint32_t length;
    int ok;
    int length_bad;
    uint32_t start;
    uint32_t end;
    uint32_t length_start;
    uint32_t length_end;
    /* Identifier that names this type, before a width or a modulus. */
    uint32_t ident_start;
    uint32_t ident_end;
    uint32_t mod_expr;
    int has_mod;
    int bare_mod;
    int named;
} DeclaredType;

/* One written type. Moduli are filled before names are resolved. */
typedef struct TypeSite {
    TypeKind kind;
    uint32_t length;
    int ok;
    int length_bad;
    uint32_t start;
    uint32_t end;
    uint32_t length_start;
    uint32_t length_end;
    uint32_t ident_start;
    uint32_t ident_end;
    uint32_t mod_expr;
    int has_mod;
    int bare_mod;
    int named;
    int modulus_done;
    uint16_t mod_index;
    int reported;
    int resolved;
    /* The spelling itself carries `^n`. Resolution turns that into rank. */
    int wrote_axis;
    /* 0 scalar, 1 one array axis, 2 a matrix. Rank 2 is a name, not a value. */
    int rank;
    uint32_t inner_len;
    const char *role;
} TypeSite;

typedef struct TypeDecl {
    uint32_t name_start;
    uint32_t name_end;
    uint32_t site;
    int installed;
} TypeDecl;

typedef enum NameRes { NAME_NONE = 0, NAME_PARAM, NAME_LOCAL, NAME_MISSING, NAME_EARLY, NAME_BAD, NAME_BOOL } NameRes;

typedef struct Token {
    TokenKind kind;
    uint32_t start;
    uint32_t end;
} Token;

typedef struct Expr {
    ExprKind kind;
    uint32_t start;
    uint32_t end;
    int height;
    int negative;
    uint32_t lit_start;
    uint32_t lit_end;
    uint32_t name_start;
    uint32_t name_end;
    uint32_t callee;
    uint16_t argc;
    uint32_t arg0;
    TokenKind op;
    uint32_t left;
    uint32_t right;
    uint32_t op_start;
    uint32_t op_end;
    TypeKind ty;
    uint32_t ty_len;
    uint16_t ty_mod;
    TypeKind conv_ty;
    int conv_ok;
    uint32_t conv_site;
    uint32_t conv_len;
    uint16_t conv_mod;
    NameRes name_res;
    uint16_t name_index;
    TypeKind name_ty;
    uint32_t name_len;
} Expr;

typedef struct Param {
    uint32_t name_start;
    uint32_t name_end;
    uint32_t type_start;
    uint32_t type_end;
    TypeKind type;
    uint32_t length;
    int type_ok;
    int length_bad;
    uint32_t length_start;
    uint32_t length_end;
    int duplicate;
    uint32_t site;
    uint16_t mod_index;
    int type_reported;
} Param;

typedef struct Local {
    uint32_t name_start;
    uint32_t name_end;
    uint32_t name_at;
    uint32_t name_end_at;
    uint32_t type_start;
    uint32_t type_end;
    TypeKind type;
    uint32_t length;
    int type_ok;
    int length_bad;
    uint32_t length_start;
    uint32_t length_end;
    int duplicate;
    uint32_t value;
    uint32_t site;
    uint16_t mod_index;
    int type_reported;
} Local;

typedef struct Edge {
    uint32_t callee;
    uint32_t start;
    uint32_t end;
} Edge;

typedef struct LoopDesc {
    uint32_t index_start;
    uint32_t index_end;
    uint32_t acc_start;
    uint32_t acc_end;
    uint32_t a_start;
    uint32_t a_end;
    uint32_t b_start;
    uint32_t b_end;
    TypeKind acc_type;
    uint32_t acc_len;
    int acc_ok;
    int acc_length_bad;
    uint32_t type_start;
    uint32_t type_end;
    uint32_t length_start;
    uint32_t length_end;
    uint32_t init_expr;
    uint32_t step_expr;
    uint32_t site;
    uint16_t acc_mod;
    int acc_reported;
    int bounds_ok;
    uint32_t bound_a;
    uint32_t bound_b;
} LoopDesc;

typedef struct OpenLoop {
    uint32_t id;
    uint32_t index_start;
    uint32_t index_end;
    uint32_t acc_start;
    uint32_t acc_end;
} OpenLoop;

typedef struct CondArm {
    uint32_t cond;
    uint32_t value;
} CondArm;

typedef struct Func {
    int is_impl;
    int typed;
    int duplicate;
    int signature_ok;
    uint32_t name_start;
    uint32_t name_end;
    uint32_t param0;
    uint16_t nparams;
    uint32_t local0;
    uint16_t nlocals;
    TypeKind result;
    uint32_t result_len;
    int result_ok;
    int result_length_bad;
    uint32_t result_start;
    uint32_t result_end;
    uint32_t result_length_start;
    uint32_t result_length_end;
    uint32_t result_site;
    uint16_t result_mod;
    int result_reported;
    uint32_t body;
    uint32_t edge0;
    uint32_t nedges;
} Func;

typedef struct Diag {
    const char *code;
    char message[192];
    char label[128];
    char note[320];
    uint32_t start;
    uint32_t end;
} Diag;

typedef struct Value {
    TypeKind type;
    uint32_t length;
    /* Owned element block when length > 0. A copy duplicates the block;
       value_clear releases it. Scalar values leave this null. */
    struct Value *elems;
    uint64_t word;
    Big big;
    uint16_t mod_index;
} Value;

typedef struct Program Program;

typedef struct UseDecl {
    uint32_t span_start;
    uint32_t span_end;
    uint32_t name_start;
    uint32_t name_end;
} UseDecl;

typedef struct Compiler {
    char *text;
    size_t length;
    const char *filename;
    Token *tokens;
    size_t ntokens;
    size_t token_cap;
    size_t at;
    Expr *exprs;
    uint32_t nexprs;
    size_t expr_cap;
    uint32_t *args;
    uint32_t nargs;
    size_t arg_cap;
    Func *funcs;
    uint32_t nfuncs;
    size_t func_cap;
    Param *params;
    uint32_t nparams;
    size_t param_cap;
    Local *locals;
    uint32_t nlocals;
    size_t local_cap;
    Edge *edges;
    uint32_t nedges;
    size_t edge_cap;
    Diag diags[MAX_ORDINARY_DIAGS + 4];
    uint32_t ndiags;
    uint32_t lex_diags;
    uint32_t parse_diags;
    uint32_t sema_diags;
    int lex_limited;
    int parse_limited;
    int sema_limited;
    int resource;
    int nesting;
    uint32_t module_start;
    uint32_t module_end;
    Arena arena;
    LoopDesc *loops;
    uint32_t nloops;
    size_t loop_cap;
    OpenLoop open_loops[MAX_OPEN_LOOPS];
    int nopen;
    uint32_t active_loops[MAX_OPEN_LOOPS];
    int nactive;
    uint32_t *loop_k;
    Value *loop_acc;
    CondArm *cond_arms;
    uint32_t ncond_arms;
    size_t cond_arm_cap;
    uint64_t steps;
    int failed;
    Program *program;
    uint16_t self_index;
    UseDecl uses[MAX_USES];
    uint16_t nuses;
    /* Stem this file was loaded as. Null on the root. Owned by the compiler. */
    char *requested;
    int own_text;
    int own_filename;
    TypeDecl *types;
    uint32_t ntypes;
    size_t type_cap;
    TypeSite *sites;
    uint32_t nsites;
    size_t site_cap;
    Big *moduli;
    uint16_t nmoduli;
    uint16_t leaf_mod;
    /* Modulus required by the expression currently being checked. */
    uint16_t expect_mod;
} Compiler;

struct Program {
    Compiler *mods[MAX_PROGRAM_SLOTS];
    int nmods;
    uint16_t order[MAX_MODULES];
    int norder;
    /* Program index named by each use, or UINT16_MAX when unresolved.
       Index 0 is the root, so an unresolved use must not default to 0. */
    uint16_t use_target[MAX_PROGRAM_SLOTS][MAX_USES];
    int graph_error;
};

static TokenKind peek_kind(const Compiler *c) {
    return c->tokens[c->at].kind;
}

static Token peek_token(const Compiler *c) {
    return c->tokens[c->at];
}

static void advance_token(Compiler *c) {
    if (c->tokens[c->at].kind != TK_EOF) {
        c->at++;
    }
}

static int same_span(const Compiler *c, uint32_t a0, uint32_t a1, uint32_t b0, uint32_t b1) {
    size_t length = (size_t)(a1 - a0);
    if ((size_t)(b1 - b0) != length) {
        return 0;
    }
    return memcmp(c->text + a0, c->text + b0, length) == 0;
}

static int span_is(const Compiler *c, uint32_t start, uint32_t end, const char *word) {
    size_t length = strlen(word);
    return (size_t)(end - start) == length && memcmp(c->text + start, word, length) == 0;
}

static void span_copy(char *dest, size_t cap, const char *text, uint32_t start, uint32_t end) {
    size_t length = end >= start ? (size_t)(end - start) : 0;
    if (length >= cap) {
        length = cap - 1;
    }
    if (length > 0) {
        memcpy(dest, text + start, length);
    }
    dest[length] = '\0';
}

static void copy_text(char *dest, size_t cap, const char *src) {
    size_t length = strlen(src);
    if (length >= cap) {
        length = cap - 1;
    }
    memcpy(dest, src, length);
    dest[length] = '\0';
}

static void add_diag(Compiler *c, const char *code, uint32_t start, uint32_t end, const char *message,
                     const char *label, const char *note, int phase) {
    uint32_t *count = phase == 0 ? &c->lex_diags : phase == 1 ? &c->parse_diags : &c->sema_diags;
    int *limited = phase == 0 ? &c->lex_limited : phase == 1 ? &c->parse_limited : &c->sema_limited;
    const char *limit_code = phase == 0 ? "ORC0007" : phase == 1 ? "ORC0105" : "ORC0208";
    Diag *diag;
    if (*count >= MAX_ORDINARY_DIAGS) {
        if (!*limited && c->ndiags < MAX_ORDINARY_DIAGS + 4) {
            *limited = 1;
            diag = &c->diags[c->ndiags++];
            memset(diag, 0, sizeof *diag);
            diag->code = limit_code;
            diag->start = start;
            diag->end = end;
            snprintf(diag->message, sizeof diag->message, "stopped reporting after %u errors", MAX_ORDINARY_DIAGS);
            copy_text(diag->label, sizeof diag->label, "further errors are suppressed");
            copy_text(diag->note, sizeof diag->note, "fix the reported errors before checking this source again");
        }
        return;
    }
    if (c->ndiags >= MAX_ORDINARY_DIAGS + 4) {
        return;
    }
    (*count)++;
    diag = &c->diags[c->ndiags++];
    memset(diag, 0, sizeof *diag);
    diag->code = code;
    diag->start = start;
    diag->end = end == start ? end + 0 : end;
    copy_text(diag->message, sizeof diag->message, message);
    copy_text(diag->label, sizeof diag->label, label == NULL ? "" : label);
    copy_text(diag->note, sizeof diag->note, note == NULL ? "" : note);
}

static void resource_diag(Compiler *c, const char *code, uint32_t start, uint32_t end, const char *message) {
    if (c->resource) {
        return;
    }
    c->resource = 1;
    add_diag(c, code, start, end, message, "resource limit reached",
             "the source was not accepted", 1);
}

static int ensure_cap(void **ptr, size_t *cap, size_t need, size_t elem, size_t max) {
    size_t next;
    void *grown;
    if (need > max) {
        return 0;
    }
    if (need <= *cap) {
        return 1;
    }
    next = *cap == 0 ? 64 : *cap;
    while (next < need) {
        if (next > max / 2) {
            next = max;
            break;
        }
        next *= 2;
    }
    grown = realloc(*ptr, next * elem);
    if (grown == NULL) {
        return 0;
    }
    *ptr = grown;
    *cap = next;
    return 1;
}

static int new_expr(Compiler *c, uint32_t *out) {
    Expr *expr;
    if (c->nexprs >= MAX_EXPRS) {
        resource_diag(c, "ORC0106", 0, 0, "source exceeds the syntax-node limit");
        return 0;
    }
    if (!ensure_cap((void **)&c->exprs, &c->expr_cap, c->nexprs + 1, sizeof(Expr), MAX_EXPRS)) {
        resource_diag(c, "ORC0106", 0, 0, "parser could not retain the syntax tree");
        return 0;
    }
    expr = &c->exprs[c->nexprs];
    memset(expr, 0, sizeof *expr);
    expr->callee = UINT32_MAX;
    expr->left = UINT32_MAX;
    expr->right = UINT32_MAX;
    expr->conv_site = UINT32_MAX;
    *out = c->nexprs++;
    return 1;
}

static int push_token(Compiler *c, TokenKind kind, uint32_t start, uint32_t end) {
    Token *token;
    if (kind != TK_EOF && c->ntokens >= MAX_TOKENS) {
        if (!c->resource) {
            c->resource = 1;
            add_diag(c, "ORC0006", start, end, "source exceeds the 262144-token lexical limit",
                     "token limit reached", "split the source into smaller files", 0);
        }
        return 0;
    }
    if (!ensure_cap((void **)&c->tokens, &c->token_cap, c->ntokens + 1, sizeof(Token), MAX_TOKENS + 1)) {
        resource_diag(c, "ORC0008", start, end, "lexer could not reserve the token stream");
        return 0;
    }
    token = &c->tokens[c->ntokens++];
    token->kind = kind;
    token->start = start;
    token->end = end;
    return 1;
}

static int utf8_ok(const unsigned char *text, size_t length) {
    size_t index = 0;
    while (index < length) {
        unsigned char byte = text[index];
        size_t width = 1;
        uint32_t codepoint;
        if (byte < 0x80) {
            index++;
            continue;
        }
        if ((byte & 0xe0) == 0xc0) {
            width = 2;
            codepoint = byte & 0x1f;
        } else if ((byte & 0xf0) == 0xe0) {
            width = 3;
            codepoint = byte & 0x0f;
        } else if ((byte & 0xf8) == 0xf0) {
            width = 4;
            codepoint = byte & 0x07;
        } else {
            return 0;
        }
        if (index + width > length) {
            return 0;
        }
        for (size_t extra = 1; extra < width; extra++) {
            unsigned char cont = text[index + extra];
            if ((cont & 0xc0) != 0x80) {
                return 0;
            }
            codepoint = (codepoint << 6) | (cont & 0x3f);
        }
        if ((width == 2 && codepoint < 0x80) || (width == 3 && codepoint < 0x800) ||
            (width == 4 && codepoint < 0x10000) || codepoint > 0x10ffff ||
            (codepoint >= 0xd800 && codepoint <= 0xdfff)) {
            return 0;
        }
        index += width;
    }
    return 1;
}

static size_t utf8_width(unsigned char byte) {
    if (byte < 0x80) {
        return 1;
    }
    if ((byte & 0xe0) == 0xc0) {
        return 2;
    }
    if ((byte & 0xf0) == 0xe0) {
        return 3;
    }
    if ((byte & 0xf8) == 0xf0) {
        return 4;
    }
    return 1;
}

static int is_ident_start(unsigned char byte) {
    return (byte >= 'A' && byte <= 'Z') || (byte >= 'a' && byte <= 'z') || byte == '_';
}

static int is_ident_continue(unsigned char byte) {
    return is_ident_start(byte) || (byte >= '0' && byte <= '9');
}

static int is_space(unsigned char byte) {
    return byte == ' ' || byte == '\t' || byte == '\n' || byte == '\r';
}

static int digit_ok(unsigned char byte, int base) {
    int value = -1;
    if (byte >= '0' && byte <= '9') {
        value = byte - '0';
    } else if (byte >= 'a' && byte <= 'z') {
        value = byte - 'a' + 10;
    } else if (byte >= 'A' && byte <= 'Z') {
        value = byte - 'A' + 10;
    }
    return value >= 0 && value < base;
}

static int integer_well_formed(const char *text, uint32_t start, uint32_t end) {
    uint32_t index = start;
    int base = 10;
    int previous_sep = 1;
    int saw = 0;
    if (end - start >= 2 && text[start] == '0' && (text[start + 1] == 'x' || text[start + 1] == 'X')) {
        base = 16;
        index += 2;
    } else if (end - start >= 2 && text[start] == '0' && (text[start + 1] == 'b' || text[start + 1] == 'B')) {
        base = 2;
        index += 2;
    }
    if (index >= end) {
        return 0;
    }
    for (; index < end; index++) {
        unsigned char byte = (unsigned char)text[index];
        if (byte == '_') {
            if (previous_sep) {
                return 0;
            }
            previous_sep = 1;
        } else if (digit_ok(byte, base)) {
            previous_sep = 0;
            saw = 1;
        } else {
            return 0;
        }
    }
    return saw && !previous_sep;
}

static TokenKind keyword_kind(const char *text, uint32_t start, uint32_t end) {
    if (end - start == 7 && memcmp(text + start, "edition", 7) == 0) {
        return TK_EDITION;
    }
    if (end - start == 6 && memcmp(text + start, "module", 6) == 0) {
        return TK_MODULE;
    }
    if (end - start == 4 && memcmp(text + start, "spec", 4) == 0) {
        return TK_SPEC;
    }
    if (end - start == 4 && memcmp(text + start, "impl", 4) == 0) {
        return TK_IMPL;
    }
    if (end - start == 4 && memcmp(text + start, "game", 4) == 0) {
        return TK_GAME;
    }
    if (end - start == 5 && memcmp(text + start, "proof", 5) == 0) {
        return TK_PROOF;
    }
    if (end - start == 5 && memcmp(text + start, "claim", 5) == 0) {
        return TK_CLAIM;
    }
    return TK_IDENT;
}

static int starts_with(const Compiler *c, size_t cursor, const char *word) {
    size_t length = strlen(word);
    return cursor + length <= c->length && memcmp(c->text + cursor, word, length) == 0;
}

static void lex_source(Compiler *c) {
    size_t cursor = 0;
    while (cursor < c->length && !c->resource) {
        unsigned char byte = (unsigned char)c->text[cursor];
        if (is_space(byte)) {
            cursor++;
            continue;
        }
        if (starts_with(c, cursor, "//")) {
            cursor += 2;
            while (cursor < c->length && c->text[cursor] != '\n' && c->text[cursor] != '\r') {
                cursor++;
            }
            continue;
        }
        if (starts_with(c, cursor, "/*")) {
            size_t start = cursor;
            int depth = 1;
            cursor += 2;
            while (cursor < c->length && depth > 0) {
                if (starts_with(c, cursor, "/*")) {
                    depth++;
                    cursor += 2;
                } else if (starts_with(c, cursor, "*/")) {
                    depth--;
                    cursor += 2;
                } else {
                    cursor += utf8_width((unsigned char)c->text[cursor]);
                }
            }
            if (depth != 0) {
                uint32_t end = (uint32_t)(start + 2 <= c->length ? start + 2 : c->length);
                add_diag(c, "ORC0002", (uint32_t)start, end, "unterminated block comment",
                         "this comment is never closed",
                         "block comments may nest, and every opening `/*` needs a closing `*/`", 0);
            }
            continue;
        }
        if (is_ident_start(byte)) {
            size_t start = cursor;
            cursor++;
            while (cursor < c->length && is_ident_continue((unsigned char)c->text[cursor])) {
                cursor++;
            }
            if (cursor - start == 3 && memcmp(c->text + start, "hex", 3) == 0 && cursor < c->length &&
                c->text[cursor] == '"') {
                size_t body = cursor + 1;
                int terminated = 0;
                int pending = 0;
                size_t offense = 0;
                int lone = 0;
                int bad = 0;
                cursor++;
                while (cursor < c->length && c->text[cursor] != '\n' && c->text[cursor] != '\r') {
                    unsigned char current = (unsigned char)c->text[cursor];
                    if (current == '"') {
                        terminated = 1;
                        cursor++;
                        break;
                    }
                    if (!bad) {
                        if (isxdigit(current)) {
                            pending = !pending;
                            if (!pending) {
                                offense = 0;
                            } else {
                                offense = cursor;
                            }
                        } else if (current == ' ') {
                            if (pending) {
                                bad = 1;
                                lone = 1;
                            }
                        } else {
                            bad = 1;
                            lone = 0;
                            offense = cursor;
                        }
                    }
                    cursor++;
                }
                if (!terminated) {
                    uint32_t end = (uint32_t)(start + 4 <= c->length ? start + 4 : c->length);
                    add_diag(c, "ORC0003", (uint32_t)start, end, "unterminated hex string",
                             "this hex string is never closed",
                             "pre-alpha Orange strings cannot cross a line boundary", 0);
                } else if (bad || pending) {
                    uint32_t at = (uint32_t)(pending && !bad ? offense : offense);
                    add_diag(c, "ORC0009", at, at + 1,
                             lone || pending ? "hex digit has no partner" : "malformed hex string",
                             "a byte is written as two hex digits",
                             "a hex string holds bytes written as pairs of hex digits", 0);
                }
                (void)body;
                if (!push_token(c, TK_HEX, (uint32_t)start, (uint32_t)cursor)) {
                    break;
                }
                continue;
            }
            if (!push_token(c, keyword_kind(c->text, (uint32_t)start, (uint32_t)cursor), (uint32_t)start,
                            (uint32_t)cursor)) {
                break;
            }
            continue;
        }
        if (byte >= '0' && byte <= '9') {
            size_t start = cursor;
            cursor++;
            while (cursor < c->length) {
                unsigned char current = (unsigned char)c->text[cursor];
                if (isalnum(current) || current == '_') {
                    cursor++;
                } else {
                    break;
                }
            }
            if (!integer_well_formed(c->text, (uint32_t)start, (uint32_t)cursor)) {
                add_diag(c, "ORC0005", (uint32_t)start, (uint32_t)cursor, "malformed integer literal",
                         "invalid digits or separators",
                         "underscores may appear only once between two valid digits", 0);
            }
            if (!push_token(c, TK_INT, (uint32_t)start, (uint32_t)cursor)) {
                break;
            }
            continue;
        }
        if (byte == '"') {
            size_t start = cursor;
            int terminated = 0;
            cursor++;
            while (cursor < c->length && c->text[cursor] != '\n' && c->text[cursor] != '\r') {
                if (c->text[cursor] == '"') {
                    terminated = 1;
                    cursor++;
                    break;
                }
                if (c->text[cursor] == '\\') {
                    size_t escape = cursor;
                    cursor++;
                    if (cursor >= c->length || c->text[cursor] == '\n' || c->text[cursor] == '\r') {
                        add_diag(c, "ORC0004", (uint32_t)escape, (uint32_t)cursor, "invalid string escape",
                                 "unsupported escape",
                                 "supported escapes are \\\", \\\\, \\n, \\r, \\t, \\0, and \\xNN", 0);
                        break;
                    }
                    if (c->text[cursor] == 'x') {
                        cursor++;
                        if (cursor + 1 >= c->length || !isxdigit((unsigned char)c->text[cursor]) ||
                            !isxdigit((unsigned char)c->text[cursor + 1])) {
                            add_diag(c, "ORC0004", (uint32_t)escape, (uint32_t)cursor, "invalid string escape",
                                     "unsupported escape",
                                     "supported escapes are \\\", \\\\, \\n, \\r, \\t, \\0, and \\xNN", 0);
                        } else {
                            cursor += 2;
                        }
                    } else if (strchr("\"\\nrt0", c->text[cursor]) != NULL) {
                        cursor++;
                    } else {
                        cursor++;
                        add_diag(c, "ORC0004", (uint32_t)escape, (uint32_t)cursor, "invalid string escape",
                                 "unsupported escape",
                                 "supported escapes are \\\", \\\\, \\n, \\r, \\t, \\0, and \\xNN", 0);
                    }
                    continue;
                }
                cursor += utf8_width((unsigned char)c->text[cursor]);
            }
            if (!terminated) {
                add_diag(c, "ORC0003", (uint32_t)start, (uint32_t)start + 1, "unterminated string literal",
                         "this string is never closed",
                         "pre-alpha Orange strings cannot cross a line boundary", 0);
            }
            if (!push_token(c, TK_STRING, (uint32_t)start, (uint32_t)cursor)) {
                break;
            }
            continue;
        }
        {
            TokenKind kind = TK_EOF;
            size_t start = cursor;
            size_t length = 1;
            if (starts_with(c, cursor, "<<<")) {
                kind = TK_ROL;
                length = 3;
            } else if (starts_with(c, cursor, ">>>")) {
                kind = TK_ROR;
                length = 3;
            } else if (starts_with(c, cursor, "..")) {
                kind = TK_DOTDOT;
                length = 2;
            } else if (starts_with(c, cursor, "++")) {
                kind = TK_PLUSPLUS;
                length = 2;
            } else if (starts_with(c, cursor, "::")) {
                kind = TK_COLONCOLON;
                length = 2;
            } else if (starts_with(c, cursor, "&&")) {
                kind = TK_AMPAMP;
                length = 2;
            } else if (starts_with(c, cursor, "||")) {
                kind = TK_PIPEPIPE;
                length = 2;
            } else if (starts_with(c, cursor, "==")) {
                kind = TK_EQEQ;
                length = 2;
            } else if (starts_with(c, cursor, "!=")) {
                kind = TK_BANGEQ;
                length = 2;
            } else if (starts_with(c, cursor, "<=")) {
                kind = TK_LESSEQ;
                length = 2;
            } else if (starts_with(c, cursor, ">=")) {
                kind = TK_GREATEREQ;
                length = 2;
            } else if (starts_with(c, cursor, "<<")) {
                kind = TK_LSHIFT;
                length = 2;
            } else if (starts_with(c, cursor, ">>")) {
                kind = TK_RSHIFT;
                length = 2;
            } else if (starts_with(c, cursor, "->")) {
                kind = TK_ARROW;
                length = 2;
            } else if (starts_with(c, cursor, "=>")) {
                kind = TK_FATARROW;
                length = 2;
            } else {
                switch (byte) {
                case '(': kind = TK_LPAREN; break;
                case ')': kind = TK_RPAREN; break;
                case '{': kind = TK_LBRACE; break;
                case '}': kind = TK_RBRACE; break;
                case '[': kind = TK_LBRACKET; break;
                case ']': kind = TK_RBRACKET; break;
                case ',': kind = TK_COMMA; break;
                case ':': kind = TK_COLON; break;
                case ';': kind = TK_SEMI; break;
                case '.': kind = TK_DOT; break;
                case '+': kind = TK_PLUS; break;
                case '-': kind = TK_MINUS; break;
                case '*': kind = TK_STAR; break;
                case '/': kind = TK_SLASH; break;
                case '%': kind = TK_PERCENT; break;
                case '&': kind = TK_AMP; break;
                case '|': kind = TK_PIPE; break;
                case '^': kind = TK_CARET; break;
                case '~': kind = TK_TILDE; break;
                case '!': kind = TK_BANG; break;
                case '=': kind = TK_EQUAL; break;
                case '<': kind = TK_LESS; break;
                case '>': kind = TK_GREATER; break;
                case '?': kind = TK_QUESTION; break;
                default: kind = TK_EOF; break;
                }
            }
            if (kind == TK_EOF) {
                size_t width = utf8_width(byte);
                char message[64];
                snprintf(message, sizeof message, "unexpected character '%c'", byte < 0x80 && byte >= 0x20 ? byte : '?');
                add_diag(c, "ORC0001", (uint32_t)cursor, (uint32_t)(cursor + width), message,
                         "character is not part of Orange 2026", "identifiers are ASCII in this pre-alpha edition", 0);
                cursor += width;
                continue;
            }
            cursor = start + length;
            if (!push_token(c, kind, (uint32_t)start, (uint32_t)cursor)) {
                break;
            }
        }
    }
    if (!c->resource) {
        push_token(c, TK_EOF, (uint32_t)c->length, (uint32_t)c->length);
    } else if (c->ntokens == 0 || c->tokens[c->ntokens - 1].kind != TK_EOF) {
        push_token(c, TK_EOF, (uint32_t)c->length, (uint32_t)c->length);
    }
}

static int enter_nest(Compiler *c, uint32_t start, uint32_t end) {
    if (c->nesting >= MAX_NESTING) {
        resource_diag(c, "ORC0106", start, end,
                      "expression exceeds the nesting limit of 64 for groups, calls, arrays, indices, loops, conditionals, updates, moduli, and prefix operators");
        return 0;
    }
    c->nesting++;
    return 1;
}

static void leave_nest(Compiler *c) {
    if (c->nesting > 0) {
        c->nesting--;
    }
}

static int height_of(const Compiler *c, uint32_t index) {
    if (index == UINT32_MAX) {
        return 1;
    }
    return c->exprs[index].height;
}

static int note_height(Compiler *c, uint32_t index) {
    int height = c->exprs[index].height;
    if (height > MAX_HEIGHT) {
        resource_diag(c, "ORC0106", c->exprs[index].start, c->exprs[index].end,
                      "expression exceeds the height limit of 256");
        return 0;
    }
    return 1;
}

static int make_lit(Compiler *c, uint32_t start, uint32_t end, int negative, uint32_t lit_start, uint32_t lit_end,
                    uint32_t *out) {
    if (!new_expr(c, out)) {
        return 0;
    }
    c->exprs[*out].kind = EX_LIT;
    c->exprs[*out].start = start;
    c->exprs[*out].end = end;
    c->exprs[*out].height = 1;
    c->exprs[*out].negative = negative;
    c->exprs[*out].lit_start = lit_start;
    c->exprs[*out].lit_end = lit_end;
    return 1;
}

static int ident_token_is(const Compiler *c, Token token, const char *word) {
    return token.kind == TK_IDENT && span_is(c, token.start, token.end, word);
}

static int is_as(const Compiler *c) {
    return ident_token_is(c, peek_token(c), "as");
}

static int is_with_update(const Compiler *c) {
    if (!ident_token_is(c, peek_token(c), "with")) {
        return 0;
    }
    return c->at + 1 < c->ntokens && c->tokens[c->at + 1].kind == TK_LBRACKET;
}

static int is_compare_op(TokenKind kind) {
    return kind == TK_EQEQ || kind == TK_BANGEQ || kind == TK_LESS || kind == TK_GREATER || kind == TK_LESSEQ ||
           kind == TK_GREATEREQ;
}

static int is_binary_kind(TokenKind kind) {
    return kind == TK_PLUS || kind == TK_MINUS || kind == TK_STAR || kind == TK_AMP || kind == TK_PIPE ||
           kind == TK_CARET || kind == TK_LSHIFT || kind == TK_RSHIFT || kind == TK_ROL || kind == TK_ROR ||
           is_compare_op(kind) || kind == TK_AMPAMP || kind == TK_PIPEPIPE || kind == TK_SLASH || kind == TK_PERCENT;
}

static int trailing_joiner(const Compiler *c) {
    return is_binary_kind(peek_kind(c)) || is_as(c) || is_with_update(c);
}

static int group_of(TokenKind kind) {
    if (kind == TK_PLUS || kind == TK_MINUS || kind == TK_STAR) {
        return 1;
    }
    if (kind == TK_AMP) {
        return 2;
    }
    if (kind == TK_PIPE) {
        return 3;
    }
    if (kind == TK_CARET) {
        return 4;
    }
    if (kind == TK_LSHIFT || kind == TK_RSHIFT || kind == TK_ROL || kind == TK_ROR) {
        return 5;
    }
    if (is_compare_op(kind)) {
        return 6;
    }
    if (kind == TK_AMPAMP) {
        return 7;
    }
    if (kind == TK_PIPEPIPE) {
        return 8;
    }
    if (kind == TK_SLASH || kind == TK_PERCENT) {
        return 9;
    }
    return 0;
}

static void skip_expr_tail(Compiler *c) {
    int paren = 0;
    int bracket = 0;
    int brace = 0;
    while (peek_kind(c) != TK_EOF) {
        TokenKind kind = peek_kind(c);
        if (paren == 0 && bracket == 0 && brace == 0 &&
            (kind == TK_SEMI || kind == TK_RBRACE || kind == TK_COMMA || kind == TK_RPAREN)) {
            return;
        }
        advance_token(c);
        if (kind == TK_LPAREN) {
            paren++;
        } else if (kind == TK_RPAREN && paren > 0) {
            paren--;
        } else if (kind == TK_LBRACKET) {
            bracket++;
        } else if (kind == TK_RBRACKET && bracket > 0) {
            bracket--;
        } else if (kind == TK_LBRACE) {
            brace++;
        } else if (kind == TK_RBRACE && brace > 0) {
            brace--;
        }
    }
}

/* Consume `depth` braces that this parse has already opened, so the caller's
   function-body skip still sees the function's own closing brace. */
static void skip_open_braces(Compiler *c, int depth) {
    while (peek_kind(c) != TK_EOF && depth > 0) {
        TokenKind kind = peek_kind(c);
        advance_token(c);
        if (kind == TK_LBRACE) {
            depth++;
        } else if (kind == TK_RBRACE) {
            depth--;
        }
    }
}

static void skip_function_body(Compiler *c, int inside_body) {
    int brace = inside_body ? 1 : 0;
    int seen = inside_body;
    while (peek_kind(c) != TK_EOF) {
        TokenKind kind = peek_kind(c);
        advance_token(c);
        if (kind == TK_LBRACE) {
            seen = 1;
            brace++;
        } else if (kind == TK_RBRACE && seen) {
            brace--;
            if (brace == 0) {
                return;
            }
        }
    }
}

static int parse_prefixed(Compiler *c, uint32_t *out);
static int parse_expr(Compiler *c, uint32_t *out);

static int canonical_array_length(const char *text, uint32_t start, uint32_t end, uint32_t *value) {
    uint64_t acc = 0;
    uint32_t index;
    if (end <= start || text[start] == '0') {
        return 0;
    }
    for (index = start; index < end; index++) {
        unsigned char digit = (unsigned char)text[index];
        if (digit < '0' || digit > '9') {
            return 0;
        }
        if (acc > MAX_ARRAY_LENGTH / 10u) {
            return 0;
        }
        acc = acc * 10u + (uint64_t)(digit - '0');
        if (acc > MAX_ARRAY_LENGTH) {
            return 0;
        }
    }
    if (acc < 1u) {
        return 0;
    }
    *value = (uint32_t)acc;
    return 1;
}

static int parse_type(Compiler *c, DeclaredType *type, int allow_array) {
    Token name = peek_token(c);
    int admit_length = 0;
    memset(type, 0, sizeof *type);
    if (name.kind != TK_IDENT) {
        add_diag(c, "ORC0101", name.start, name.end, "expected a type name", "expected a type", NULL, 1);
        type->start = name.start;
        type->end = name.end;
        return 0;
    }
    type->start = name.start;
    type->ident_start = name.start;
    type->ident_end = name.end;
    advance_token(c);
    type->end = name.end;
    if (span_is(c, name.start, name.end, "Mod") && peek_kind(c) == TK_LBRACKET) {
        Token open = peek_token(c);
        Token close;
        uint32_t modulus = UINT32_MAX;
        advance_token(c);
        if (!enter_nest(c, open.start, open.end)) {
            return 0;
        }
        if (!parse_expr(c, &modulus)) {
            leave_nest(c);
            return 0;
        }
        leave_nest(c);
        close = peek_token(c);
        if (close.kind != TK_RBRACKET) {
            add_diag(c, "ORC0101", close.start, close.end, "expected `]` after the modulus", "found a different token",
                     "a modulus type is written `Mod[MODULUS]`, as in `Mod[(1 << 255) - 19]`", 1);
            type->end = close.end;
            return 0;
        }
        type->end = close.end;
        advance_token(c);
        type->kind = TY_MOD;
        type->has_mod = 1;
        type->mod_expr = modulus;
        type->ok = 1;
        admit_length = 1;
    } else if (peek_kind(c) == TK_LBRACKET) {
        Token width;
        advance_token(c);
        width = peek_token(c);
        if (width.kind != TK_INT) {
            add_diag(c, "ORC0101", width.start, width.end, "expected a word width", "expected an integer width", NULL, 1);
            type->end = width.end;
            return 0;
        }
        advance_token(c);
        if (peek_kind(c) != TK_RBRACKET) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `]`", "unclosed type width", NULL, 1);
            type->end = peek_token(c).end;
            return 0;
        }
        type->end = peek_token(c).end;
        advance_token(c);
        if (span_is(c, name.start, name.end, "Word")) {
            if (span_is(c, width.start, width.end, "8")) {
                type->kind = TY_W8;
                type->ok = 1;
            } else if (span_is(c, width.start, width.end, "16")) {
                type->kind = TY_W16;
                type->ok = 1;
            } else if (span_is(c, width.start, width.end, "32")) {
                type->kind = TY_W32;
                type->ok = 1;
            } else if (span_is(c, width.start, width.end, "64")) {
                type->kind = TY_W64;
                type->ok = 1;
            }
        }
        admit_length = type->ok;
    } else if (span_is(c, name.start, name.end, "Int")) {
        type->kind = TY_INT;
        type->ok = 1;
        admit_length = 1;
    } else if (span_is(c, name.start, name.end, "Bool")) {
        type->kind = TY_BOOL;
        type->ok = 1;
        admit_length = 1;
    } else if (span_is(c, name.start, name.end, "Mod")) {
        type->kind = TY_MOD;
        type->bare_mod = 1;
    } else if (!span_is(c, name.start, name.end, "Word")) {
        type->named = 1;
        admit_length = 1;
    }
    if (allow_array && peek_kind(c) == TK_CARET) {
        Token length;
        advance_token(c);
        length = peek_token(c);
        if (length.kind != TK_INT) {
            add_diag(c, "ORC0101", length.start, length.end, "expected an array length", "expected a length",
                     "an array type is `T^n` with one decimal length", 1);
            return 0;
        }
        advance_token(c);
        if (peek_kind(c) == TK_CARET) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                     "expected the end of the type after its array length", "repeated array length",
                     "arrays of arrays are not part of this slice", 1);
            return 0;
        }
        type->length_start = length.start;
        type->length_end = length.end;
        type->end = length.end;
        if (admit_length || type->ok) {
            uint32_t value = 0;
            if (!canonical_array_length(c->text, length.start, length.end, &value)) {
                type->ok = 0;
                type->length_bad = 1;
            } else {
                type->length = value;
            }
        }
    }
    return 1;
}

static void store_declared(DeclaredType *type, TypeKind *kind, uint32_t *length, int *ok, int *length_bad,
                           uint32_t *start, uint32_t *end, uint32_t *length_start, uint32_t *length_end) {
    *kind = type->kind;
    *length = type->length;
    *ok = type->ok;
    *length_bad = type->length_bad;
    *start = type->start;
    *end = type->end;
    *length_start = type->length_start;
    *length_end = type->length_end;
}

static int push_site(Compiler *c, const DeclaredType *type, const char *role, uint32_t *site_out) {
    TypeSite *site;
    if (c->nsites >= MAX_TYPE_SITES) {
        resource_diag(c, "ORC0106", type->start, type->end, "source exceeds the type-site limit");
        return 0;
    }
    if (!ensure_cap((void **)&c->sites, &c->site_cap, c->nsites + 1, sizeof(TypeSite), MAX_TYPE_SITES)) {
        resource_diag(c, "ORC0106", type->start, type->end, "parser could not retain a type");
        return 0;
    }
    site = &c->sites[c->nsites];
    memset(site, 0, sizeof *site);
    site->kind = type->kind;
    site->length = type->length;
    site->ok = type->ok;
    site->length_bad = type->length_bad;
    site->start = type->start;
    site->end = type->end;
    site->length_start = type->length_start;
    site->length_end = type->length_end;
    site->ident_start = type->ident_start;
    site->ident_end = type->ident_end;
    site->mod_expr = type->mod_expr;
    site->has_mod = type->has_mod;
    site->bare_mod = type->bare_mod;
    site->named = type->named;
    site->role = role;
    site->wrote_axis = type->length > 0 && !type->length_bad;
    if (!type->named && !type->bare_mod) {
        site->rank = site->wrote_axis ? 1 : 0;
    }
    *site_out = c->nsites++;
    return 1;
}

static void reject_type(Compiler *c, TypeKind type, int ok, uint32_t start, uint32_t end) {
    if (ok) {
        return;
    }
    if (end >= start + 4 && memcmp(c->text + start, "Word", 4) == 0) {
        add_diag(c, "ORC0204", start, end, "word width must be 8, 16, 32, or 64",
                 "this width is not admitted", "write Word[8], Word[16], Word[32], or Word[64]", 2);
    } else {
        add_diag(c, "ORC0203", start, end, "type is outside the admitted fragment",
                 "unsupported type", "admitted types are Int and Word[8], Word[16], Word[32], and Word[64]", 2);
    }
    (void)type;
}

static int finish_binary(Compiler *c, uint32_t left, uint32_t right, TokenKind op, uint32_t op_start, uint32_t op_end,
                         uint32_t *out) {
    int height = 1 + (height_of(c, left) > height_of(c, right) ? height_of(c, left) : height_of(c, right));
    if (!new_expr(c, out)) {
        return 0;
    }
    c->exprs[*out].kind = EX_BINARY;
    c->exprs[*out].start = c->exprs[left].start;
    c->exprs[*out].end = c->exprs[right].end;
    c->exprs[*out].height = height;
    c->exprs[*out].op = op;
    c->exprs[*out].left = left;
    c->exprs[*out].right = right;
    c->exprs[*out].op_start = op_start;
    c->exprs[*out].op_end = op_end;
    return note_height(c, *out);
}

static int continue_product(Compiler *c, uint32_t left, uint32_t *out) {
    while (peek_kind(c) == TK_STAR && !c->resource) {
        Token op = peek_token(c);
        uint32_t right;
        advance_token(c);
        if (!parse_prefixed(c, &right)) {
            return 0;
        }
        if (!finish_binary(c, left, right, TK_STAR, op.start, op.end, &left)) {
            return 0;
        }
    }
    *out = left;
    return 1;
}

static int ungrouped(Compiler *c, Token token, Token previous) {
    char message[160];
    char previous_text[32];
    char current_text[32];
    size_t previous_len = previous.end - previous.start;
    size_t current_len = token.end - token.start;
    if (previous_len >= sizeof previous_text) {
        previous_len = sizeof previous_text - 1;
    }
    if (current_len >= sizeof current_text) {
        current_len = sizeof current_text - 1;
    }
    memcpy(previous_text, c->text + previous.start, previous_len);
    previous_text[previous_len] = '\0';
    memcpy(current_text, c->text + token.start, current_len);
    current_text[current_len] = '\0';
    snprintf(message, sizeof message, "`%s` follows `%s` without grouping parentheses", current_text, previous_text);
    add_diag(c, "ORC0108", token.start, token.end, message, "ungrouped operator",
             "operators from different groups have no relative precedence in Orange; parenthesize the part that applies first",
             1);
    skip_expr_tail(c);
    return 1;
}

static int finish_index_node(Compiler *c, uint32_t base, uint32_t end, ExprKind kind, uint32_t child, uint32_t lit_start,
                             uint32_t lit_end, uint32_t *out) {
    int height = height_of(c, base);
    if (kind == EX_SELECT) {
        int index_height = height_of(c, child);
        if (index_height > height) {
            height = index_height;
        }
    }
    if (!new_expr(c, out)) {
        return 0;
    }
    c->exprs[*out].kind = kind;
    c->exprs[*out].left = base;
    c->exprs[*out].right = child;
    c->exprs[*out].start = c->exprs[base].start;
    c->exprs[*out].end = end;
    c->exprs[*out].lit_start = lit_start;
    c->exprs[*out].lit_end = lit_end;
    c->exprs[*out].height = 1 + height;
    return note_height(c, *out);
}

static int parse_one_index(Compiler *c, uint32_t base, uint32_t *out) {
    Token index;
    Token close;
    uint32_t child = UINT32_MAX;
    advance_token(c);
    if (peek_kind(c) == TK_RBRACKET) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected an index after `[`", "empty index",
                 "an index is an integer literal or an expression", 1);
        return 0;
    }
    index = peek_token(c);
    if (index.kind == TK_INT && c->at + 1 < c->ntokens && c->tokens[c->at + 1].kind == TK_RBRACKET) {
        advance_token(c);
        close = peek_token(c);
        advance_token(c);
        return finish_index_node(c, base, close.end, EX_INDEX, UINT32_MAX, index.start, index.end, out);
    }
    if (!enter_nest(c, index.start, index.end)) {
        return 0;
    }
    if (!parse_expr(c, &child)) {
        leave_nest(c);
        return 0;
    }
    leave_nest(c);
    close = peek_token(c);
    if (close.kind != TK_RBRACKET) {
        add_diag(c, "ORC0101", close.start, close.end, "expected `]`", "unclosed index", NULL, 1);
        return 0;
    }
    advance_token(c);
    return finish_index_node(c, base, close.end, EX_SELECT, child, 0, 0, out);
}

/* A second `[` is another index, not a syntax error. Indexing the resulting
   scalar is ORC0224. Arrays of arrays stay outside this slice. */
static int parse_index(Compiler *c, uint32_t base, uint32_t *out) {
    if (peek_kind(c) != TK_LBRACKET) {
        *out = base;
        return 1;
    }
    for (;;) {
        if (!parse_one_index(c, base, out)) {
            return 0;
        }
        if (peek_kind(c) != TK_LBRACKET) {
            return 1;
        }
        base = *out;
    }
}

static int parse_array(Compiler *c, Token open, uint32_t *out) {
    uint32_t local_elems[MAX_ARRAY_ELEMENTS];
    uint32_t count = 0;
    Token close;
    int height = 1;
    if (!enter_nest(c, open.start, open.end)) {
        return 0;
    }
    if (peek_kind(c) == TK_RBRACKET) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected an array element",
                 "an array has at least one element", "Orange 2026 has no empty arrays in this slice", 1);
        leave_nest(c);
        return 0;
    }
    if (!parse_expr(c, &local_elems[0])) {
        leave_nest(c);
        return 0;
    }
    count = 1;
    if (peek_kind(c) == TK_SEMI) {
        Token length;
        uint32_t element = local_elems[0];
        advance_token(c);
        length = peek_token(c);
        if (length.kind != TK_INT) {
            add_diag(c, "ORC0101", length.start, length.end, "expected a fill length", "expected an integer length",
                     "a fill literal is `[element; length]`", 1);
            leave_nest(c);
            return 0;
        }
        advance_token(c);
        close = peek_token(c);
        if (close.kind != TK_RBRACKET) {
            add_diag(c, "ORC0101", close.start, close.end, "expected `]`", "unclosed fill literal", NULL, 1);
            leave_nest(c);
            return 0;
        }
        advance_token(c);
        leave_nest(c);
        if (!new_expr(c, out)) {
            return 0;
        }
        c->exprs[*out].kind = EX_FILL;
        c->exprs[*out].left = element;
        c->exprs[*out].start = open.start;
        c->exprs[*out].end = close.end;
        c->exprs[*out].lit_start = length.start;
        c->exprs[*out].lit_end = length.end;
        c->exprs[*out].height = 1 + height_of(c, element);
        return note_height(c, *out);
    }
    while (peek_kind(c) == TK_COMMA) {
        advance_token(c);
        if (peek_kind(c) == TK_RBRACKET) {
            break;
        }
        if (count >= MAX_ARRAY_ELEMENTS) {
            resource_diag(c, "ORC0106", peek_token(c).start, peek_token(c).end,
                          "array literal exceeds the 256-element limit");
            leave_nest(c);
            return 0;
        }
        if (!parse_expr(c, &local_elems[count])) {
            leave_nest(c);
            return 0;
        }
        count++;
    }
    leave_nest(c);
    close = peek_token(c);
    if (close.kind != TK_RBRACKET) {
        add_diag(c, "ORC0101", close.start, close.end, "expected `]`", "unclosed array literal", NULL, 1);
        return 0;
    }
    advance_token(c);
    if (!ensure_cap((void **)&c->args, &c->arg_cap, c->nargs + count, sizeof(uint32_t), MAX_EXPRS)) {
        resource_diag(c, "ORC0106", open.start, close.end, "parser could not retain array elements");
        return 0;
    }
    if (!new_expr(c, out)) {
        return 0;
    }
    c->exprs[*out].kind = EX_ARRAY;
    c->exprs[*out].start = open.start;
    c->exprs[*out].end = close.end;
    c->exprs[*out].arg0 = c->nargs;
    c->exprs[*out].argc = (uint16_t)count;
    memcpy(c->args + c->nargs, local_elems, (size_t)count * sizeof(uint32_t));
    c->nargs += count;
    for (uint32_t index = 0; index < count; index++) {
        int child = height_of(c, local_elems[index]);
        if (1 + child > height) {
            height = 1 + child;
        }
    }
    c->exprs[*out].height = height;
    return note_height(c, *out);
}

static int parse_loop(Compiler *c, Token for_token, uint32_t *out) {
    Token index;
    Token bound_a;
    Token bound_b;
    Token acc;
    DeclaredType declared;
    uint32_t init = UINT32_MAX;
    uint32_t step = UINT32_MAX;
    Token close;
    uint32_t id;
    int height;
    if (c->nloops >= MAX_EXPRS || c->nopen >= MAX_OPEN_LOOPS) {
        resource_diag(c, "ORC0106", for_token.start, for_token.end, "source exceeds the loop limit");
        return 0;
    }
    if (!ensure_cap((void **)&c->loops, &c->loop_cap, c->nloops + 1, sizeof(LoopDesc), MAX_EXPRS)) {
        resource_diag(c, "ORC0106", for_token.start, for_token.end, "parser could not retain a loop");
        return 0;
    }
    id = c->nloops++;
    memset(&c->loops[id], 0, sizeof c->loops[id]);
    c->loops[id].init_expr = UINT32_MAX;
    c->loops[id].step_expr = UINT32_MAX;
    c->loops[id].site = UINT32_MAX;
    advance_token(c);
    index = peek_token(c);
    if (index.kind != TK_IDENT) {
        add_diag(c, "ORC0101", index.start, index.end, "expected a loop index", "expected `for name in ...`", NULL, 1);
        return 0;
    }
    c->loops[id].index_start = index.start;
    c->loops[id].index_end = index.end;
    advance_token(c);
    if (!ident_token_is(c, peek_token(c), "in")) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `in`", "expected `for name in a..b`",
                 NULL, 1);
        return 0;
    }
    advance_token(c);
    bound_a = peek_token(c);
    if (bound_a.kind != TK_INT) {
        add_diag(c, "ORC0101", bound_a.start, bound_a.end, "expected a loop bound", "a loop bound is an integer literal",
                 NULL, 1);
        return 0;
    }
    c->loops[id].a_start = bound_a.start;
    c->loops[id].a_end = bound_a.end;
    advance_token(c);
    if (peek_kind(c) != TK_DOTDOT) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `..`", "a loop range is `a..b`", NULL,
                 1);
        return 0;
    }
    advance_token(c);
    bound_b = peek_token(c);
    if (bound_b.kind != TK_INT) {
        add_diag(c, "ORC0101", bound_b.start, bound_b.end, "expected a loop bound", "a loop bound is an integer literal",
                 NULL, 1);
        return 0;
    }
    c->loops[id].b_start = bound_b.start;
    c->loops[id].b_end = bound_b.end;
    advance_token(c);
    if (!ident_token_is(c, peek_token(c), "with")) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `with`",
                 "a loop names its accumulator", "write `for i in a..b with s: T = start { step }`", 1);
        return 0;
    }
    advance_token(c);
    acc = peek_token(c);
    if (acc.kind != TK_IDENT) {
        add_diag(c, "ORC0101", acc.start, acc.end, "expected an accumulator name", "expected a name after `with`", NULL,
                 1);
        return 0;
    }
    c->loops[id].acc_start = acc.start;
    c->loops[id].acc_end = acc.end;
    advance_token(c);
    if (peek_kind(c) != TK_COLON) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `:`", "an accumulator states its type",
                 NULL, 1);
        return 0;
    }
    advance_token(c);
    if (!parse_type(c, &declared, 1)) {
        return 0;
    }
    store_declared(&declared, &c->loops[id].acc_type, &c->loops[id].acc_len, &c->loops[id].acc_ok,
                   &c->loops[id].acc_length_bad, &c->loops[id].type_start, &c->loops[id].type_end,
                   &c->loops[id].length_start, &c->loops[id].length_end);
    if (!push_site(c, &declared, "accumulator type", &c->loops[id].site)) {
        return 0;
    }
    if (peek_kind(c) != TK_EQUAL) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `=`", "expected the accumulator's start",
                 NULL, 1);
        return 0;
    }
    advance_token(c);
    if (!enter_nest(c, for_token.start, for_token.end)) {
        return 0;
    }
    if (!parse_expr(c, &init)) {
        leave_nest(c);
        return 0;
    }
    c->loops[id].init_expr = init;
    if (peek_kind(c) != TK_LBRACE) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `{`", "expected the loop step", NULL,
                 1);
        leave_nest(c);
        return 0;
    }
    advance_token(c);
    if (c->nopen >= MAX_OPEN_LOOPS) {
        resource_diag(c, "ORC0106", for_token.start, for_token.end, "expression exceeds the nesting limit of 64");
        skip_open_braces(c, 1);
        leave_nest(c);
        return 0;
    }
    c->open_loops[c->nopen].id = id;
    c->open_loops[c->nopen].index_start = c->loops[id].index_start;
    c->open_loops[c->nopen].index_end = c->loops[id].index_end;
    c->open_loops[c->nopen].acc_start = c->loops[id].acc_start;
    c->open_loops[c->nopen].acc_end = c->loops[id].acc_end;
    c->nopen++;
    if (!parse_expr(c, &step)) {
        c->nopen--;
        /* The step's `{` is already open. Consume through its `}` so the
           function-body skip does not treat that brace as the function's. */
        skip_open_braces(c, 1);
        leave_nest(c);
        return 0;
    }
    c->nopen--;
    c->loops[id].step_expr = step;
    close = peek_token(c);
    if (close.kind != TK_RBRACE) {
        add_diag(c, "ORC0101", close.start, close.end, "expected `}`", "a loop step ends at `}`", NULL, 1);
        skip_open_braces(c, 1);
        leave_nest(c);
        return 0;
    }
    advance_token(c);
    leave_nest(c);
    if (!new_expr(c, out)) {
        return 0;
    }
    c->exprs[*out].kind = EX_LOOP;
    c->exprs[*out].arg0 = id;
    c->exprs[*out].start = for_token.start;
    c->exprs[*out].end = close.end;
    height = height_of(c, init);
    if (height_of(c, step) > height) {
        height = height_of(c, step);
    }
    if (c->loops[id].site != UINT32_MAX && c->sites[c->loops[id].site].has_mod) {
        int mod_height = height_of(c, c->sites[c->loops[id].site].mod_expr);
        if (mod_height > height) {
            height = mod_height;
        }
    }
    c->exprs[*out].height = 1 + height;
    return note_height(c, *out);
}

static int parse_update(Compiler *c, uint32_t base, uint32_t *out) {
    Token with_token = peek_token(c);
    uint32_t index = UINT32_MAX;
    uint32_t value = UINT32_MAX;
    int height;
    advance_token(c);
    if (peek_kind(c) != TK_LBRACKET) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `[`", "an update is `with [index] = value`",
                 NULL, 1);
        return 0;
    }
    advance_token(c);
    if (!enter_nest(c, with_token.start, with_token.end)) {
        return 0;
    }
    if (!parse_expr(c, &index)) {
        leave_nest(c);
        return 0;
    }
    if (peek_kind(c) != TK_RBRACKET) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `]`", "unclosed update index", NULL, 1);
        leave_nest(c);
        return 0;
    }
    advance_token(c);
    if (peek_kind(c) != TK_EQUAL) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `=`", "an update assigns one element",
                 NULL, 1);
        leave_nest(c);
        return 0;
    }
    advance_token(c);
    if (!parse_expr(c, &value)) {
        leave_nest(c);
        return 0;
    }
    leave_nest(c);
    if (!new_expr(c, out)) {
        return 0;
    }
    c->exprs[*out].kind = EX_UPDATE;
    c->exprs[*out].left = base;
    c->exprs[*out].right = index;
    c->exprs[*out].callee = value;
    c->exprs[*out].start = c->exprs[base].start;
    c->exprs[*out].end = c->exprs[value].end;
    c->exprs[*out].op_start = with_token.start;
    c->exprs[*out].op_end = with_token.end;
    height = height_of(c, base);
    if (height_of(c, index) > height) {
        height = height_of(c, index);
    }
    if (height_of(c, value) > height) {
        height = height_of(c, value);
    }
    c->exprs[*out].height = 1 + height;
    return note_height(c, *out);
}

static int parse_args(Compiler *c, uint32_t *arg0, uint16_t *argc) {
    uint32_t local_args[MAX_ARGS];
    uint32_t count = 0;
    /* Nested calls append their own arguments while this list is parsed, so
       this call's arguments are copied only after every nested call is done. */
    if (peek_kind(c) == TK_RPAREN) {
        *arg0 = c->nargs;
        *argc = 0;
        return 1;
    }
    for (;;) {
        if (count >= MAX_ARGS) {
            resource_diag(c, "ORC0106", peek_token(c).start, peek_token(c).end,
                          "call exceeds the 256-argument limit");
            return 0;
        }
        if (!parse_expr(c, &local_args[count])) {
            return 0;
        }
        count++;
        if (peek_kind(c) == TK_COMMA) {
            advance_token(c);
            if (peek_kind(c) == TK_RPAREN) {
                break;
            }
            continue;
        }
        break;
    }
    if (!ensure_cap((void **)&c->args, &c->arg_cap, c->nargs + count, sizeof(uint32_t), MAX_EXPRS)) {
        resource_diag(c, "ORC0106", 0, 0, "parser could not retain call arguments");
        return 0;
    }
    *arg0 = c->nargs;
    memcpy(c->args + c->nargs, local_args, (size_t)count * sizeof(uint32_t));
    c->nargs += count;
    *argc = (uint16_t)count;
    return 1;
}

static int token_at_is_word(const Compiler *c, uint32_t index, const char *word) {
    Token token;
    if (index >= c->ntokens) {
        return 0;
    }
    token = c->tokens[index];
    return token.kind == TK_IDENT && span_is(c, token.start, token.end, word);
}

static int else_follows(const Compiler *c, uint32_t start) {
    int depth = 0;
    uint32_t position = start;
    for (;;) {
        TokenKind kind;
        if (position >= c->ntokens) {
            return 0;
        }
        kind = c->tokens[position].kind;
        if (kind == TK_EOF) {
            return 0;
        }
        if (kind == TK_LPAREN || kind == TK_LBRACKET || kind == TK_LBRACE) {
            depth++;
        } else if (kind == TK_RPAREN || kind == TK_RBRACKET || kind == TK_RBRACE) {
            if (depth == 0) {
                return 0;
            }
            depth--;
            if (depth == 0 && kind == TK_RBRACE && token_at_is_word(c, position + 1u, "else")) {
                return 1;
            }
        } else if ((kind == TK_SEMI || kind == TK_COMMA) && depth == 0) {
            return 0;
        }
        position++;
    }
}

static int starts_conditional(const Compiler *c) {
    uint32_t next = c->at + 1u;
    TokenKind kind;
    int continues;
    if (next >= c->ntokens) {
        return 0;
    }
    kind = c->tokens[next].kind;
    if (kind == TK_IDENT) {
        continues = token_at_is_word(c, next, "as") ||
                    (token_at_is_word(c, next, "with") && next + 1u < c->ntokens &&
                     c->tokens[next + 1u].kind == TK_LBRACKET);
        return !continues || else_follows(c, next);
    }
    if (kind == TK_INT || kind == TK_BANG || kind == TK_TILDE) {
        return 1;
    }
    if (kind == TK_LPAREN || kind == TK_MINUS || kind == TK_LBRACKET) {
        return else_follows(c, next);
    }
    return 0;
}

static int parse_conditional(Compiler *c, Token if_token, uint32_t *out) {
    CondArm *local = NULL;
    uint32_t count = 0;
    uint32_t cap = 0;
    uint32_t otherwise = UINT32_MAX;
    int height = 0;
    Token close = if_token;
    if (!enter_nest(c, if_token.start, if_token.end)) {
        return 0;
    }
    for (;;) {
        uint32_t condition = UINT32_MAX;
        uint32_t value = UINT32_MAX;
        CondArm *grown;
        advance_token(c);
        if (!parse_expr(c, &condition)) {
            free(local);
            leave_nest(c);
            return 0;
        }
        if (peek_kind(c) != TK_LBRACE) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `{` after the condition",
                     "expected a conditional value", "a conditional is `if c { a } else { b }`", 1);
            free(local);
            leave_nest(c);
            return 0;
        }
        advance_token(c);
        if (!parse_expr(c, &value)) {
            skip_open_braces(c, 1);
            free(local);
            leave_nest(c);
            return 0;
        }
        if (peek_kind(c) != TK_RBRACE) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `}` after the value",
                     "a conditional value is one expression", NULL, 1);
            skip_open_braces(c, 1);
            free(local);
            leave_nest(c);
            return 0;
        }
        advance_token(c);
        if (!ident_token_is(c, peek_token(c), "else")) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                     "expected `else` and the value when the condition is false", "missing else",
                     "every `if` has an `else`, so that a conditional always has a value", 1);
            free(local);
            leave_nest(c);
            return 0;
        }
        advance_token(c);
        if (count == cap) {
            uint32_t next = cap == 0 ? 4u : cap * 2u;
            grown = realloc(local, (size_t)next * sizeof(CondArm));
            if (grown == NULL) {
                resource_diag(c, "ORC0106", if_token.start, if_token.end, "parser could not retain a conditional");
                free(local);
                leave_nest(c);
                return 0;
            }
            local = grown;
            cap = next;
        }
        local[count].cond = condition;
        local[count].value = value;
        count++;
        if (height_of(c, condition) > height) {
            height = height_of(c, condition);
        }
        if (height_of(c, value) > height) {
            height = height_of(c, value);
        }
        if (ident_token_is(c, peek_token(c), "if")) {
            continue;
        }
        if (peek_kind(c) != TK_LBRACE) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `{` or `if` after `else`",
                     "expected the other value", "after `else`, `{` begins the last value and `if` begins another arm",
                     1);
            free(local);
            leave_nest(c);
            return 0;
        }
        advance_token(c);
        if (!parse_expr(c, &otherwise)) {
            skip_open_braces(c, 1);
            free(local);
            leave_nest(c);
            return 0;
        }
        if (peek_kind(c) != TK_RBRACE) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `}` after the value",
                     "a conditional value is one expression", NULL, 1);
            skip_open_braces(c, 1);
            free(local);
            leave_nest(c);
            return 0;
        }
        close = peek_token(c);
        advance_token(c);
        break;
    }
    leave_nest(c);
    if (height_of(c, otherwise) > height) {
        height = height_of(c, otherwise);
    }
    if (c->ncond_arms > MAX_EXPRS - count ||
        !ensure_cap((void **)&c->cond_arms, &c->cond_arm_cap, c->ncond_arms + count, sizeof(CondArm), MAX_EXPRS)) {
        resource_diag(c, "ORC0106", if_token.start, close.end, "parser could not retain a conditional");
        free(local);
        return 0;
    }
    if (!new_expr(c, out)) {
        free(local);
        return 0;
    }
    memcpy(c->cond_arms + c->ncond_arms, local, (size_t)count * sizeof(CondArm));
    free(local);
    c->exprs[*out].kind = EX_COND;
    c->exprs[*out].arg0 = c->ncond_arms;
    c->exprs[*out].argc = (uint16_t)count;
    c->exprs[*out].right = otherwise;
    c->exprs[*out].start = if_token.start;
    c->exprs[*out].end = close.end;
    c->exprs[*out].height = 1 + height;
    c->ncond_arms += count;
    return note_height(c, *out);
}

static int parse_prefixed(Compiler *c, uint32_t *out) {
    Token token = peek_token(c);
    if (c->resource) {
        return 0;
    }
    if (token.kind == TK_MINUS) {
        Token next;
        advance_token(c);
        next = peek_token(c);
        if (next.kind == TK_INT) {
            advance_token(c);
            return make_lit(c, token.start, next.end, 1, next.start, next.end, out);
        }
        if (!enter_nest(c, token.start, token.end)) {
            return 0;
        }
        if (!parse_prefixed(c, out)) {
            leave_nest(c);
            return 0;
        }
        leave_nest(c);
        {
            uint32_t operand = *out;
            int height = 1 + height_of(c, operand);
            if (!new_expr(c, out)) {
                return 0;
            }
            c->exprs[*out].kind = EX_UNARY;
            c->exprs[*out].op = TK_MINUS;
            c->exprs[*out].left = operand;
            c->exprs[*out].start = token.start;
            c->exprs[*out].end = c->exprs[operand].end;
            c->exprs[*out].op_start = token.start;
            c->exprs[*out].op_end = token.end;
            c->exprs[*out].height = height;
            return note_height(c, *out);
        }
    }
    if (token.kind == TK_BANG) {
        uint32_t operand;
        advance_token(c);
        if (!enter_nest(c, token.start, token.end)) {
            return 0;
        }
        if (!parse_prefixed(c, &operand)) {
            leave_nest(c);
            return 0;
        }
        leave_nest(c);
        if (!new_expr(c, out)) {
            return 0;
        }
        c->exprs[*out].kind = EX_UNARY;
        c->exprs[*out].op = TK_BANG;
        c->exprs[*out].left = operand;
        c->exprs[*out].start = token.start;
        c->exprs[*out].end = c->exprs[operand].end;
        c->exprs[*out].op_start = token.start;
        c->exprs[*out].op_end = token.end;
        c->exprs[*out].height = 1 + height_of(c, operand);
        return note_height(c, *out);
    }
    if (token.kind == TK_TILDE) {
        uint32_t operand;
        advance_token(c);
        if (!enter_nest(c, token.start, token.end)) {
            return 0;
        }
        if (!parse_prefixed(c, &operand)) {
            leave_nest(c);
            return 0;
        }
        leave_nest(c);
        if (!new_expr(c, out)) {
            return 0;
        }
        c->exprs[*out].kind = EX_UNARY;
        c->exprs[*out].op = TK_TILDE;
        c->exprs[*out].left = operand;
        c->exprs[*out].start = token.start;
        c->exprs[*out].end = c->exprs[operand].end;
        c->exprs[*out].op_start = token.start;
        c->exprs[*out].op_end = token.end;
        c->exprs[*out].height = 1 + height_of(c, operand);
        return note_height(c, *out);
    }
    if (token.kind == TK_INT) {
        advance_token(c);
        return make_lit(c, token.start, token.end, 0, token.start, token.end, out);
    }
    if (token.kind == TK_IDENT) {
        Token name = token;
        int open_index;
        if (span_is(c, token.start, token.end, "for") && c->at + 1 < c->ntokens &&
            c->tokens[c->at + 1].kind == TK_IDENT) {
            return parse_loop(c, token, out);
        }
        if (span_is(c, token.start, token.end, "if") && starts_conditional(c)) {
            return parse_conditional(c, token, out);
        }
        advance_token(c);
        if (peek_kind(c) == TK_COLONCOLON) {
            Token module_name = name;
            Token func_name;
            uint32_t arg0 = 0;
            uint16_t argc = 0;
            Token close;
            advance_token(c);
            func_name = peek_token(c);
            if (func_name.kind != TK_IDENT) {
                add_diag(c, "ORC0101", func_name.start, func_name.end, "expected a function name",
                         "expected an identifier", NULL, 1);
                return 0;
            }
            advance_token(c);
            if (peek_kind(c) != TK_LPAREN) {
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                         "expected `(` after the qualified function name", "expected `(`",
                         "a name qualified by its module is always called, as in `sha256::initial()`", 1);
                return 0;
            }
            advance_token(c);
            if (!enter_nest(c, module_name.start, module_name.end)) {
                return 0;
            }
            if (!parse_args(c, &arg0, &argc)) {
                leave_nest(c);
                return 0;
            }
            leave_nest(c);
            close = peek_token(c);
            if (close.kind != TK_RPAREN) {
                add_diag(c, "ORC0101", close.start, close.end, "expected `)`", "unclosed argument list", NULL, 1);
                return 0;
            }
            advance_token(c);
            if (!new_expr(c, out)) {
                return 0;
            }
            c->exprs[*out].kind = EX_CALL;
            c->exprs[*out].start = module_name.start;
            c->exprs[*out].end = close.end;
            c->exprs[*out].left = module_name.start;
            c->exprs[*out].right = module_name.end;
            c->exprs[*out].name_start = func_name.start;
            c->exprs[*out].name_end = func_name.end;
            c->exprs[*out].arg0 = arg0;
            c->exprs[*out].argc = argc;
            c->exprs[*out].height = 1;
            for (uint16_t index = 0; index < argc; index++) {
                int child = height_of(c, c->args[arg0 + index]);
                if (1 + child > c->exprs[*out].height) {
                    c->exprs[*out].height = 1 + child;
                }
            }
            if (!note_height(c, *out)) {
                return 0;
            }
            return parse_index(c, *out, out);
        }
        if (peek_kind(c) != TK_LPAREN) {
            for (open_index = c->nopen - 1; open_index >= 0; open_index--) {
                OpenLoop *open = &c->open_loops[open_index];
                int is_index = same_span(c, open->index_start, open->index_end, name.start, name.end);
                int is_acc = same_span(c, open->acc_start, open->acc_end, name.start, name.end);
                if (!is_index && !is_acc) {
                    continue;
                }
                if (!new_expr(c, out)) {
                    return 0;
                }
                c->exprs[*out].kind = is_index ? EX_LOOP_INDEX : EX_ACCUM;
                c->exprs[*out].arg0 = open->id;
                c->exprs[*out].start = name.start;
                c->exprs[*out].end = name.end;
                c->exprs[*out].name_start = name.start;
                c->exprs[*out].name_end = name.end;
                c->exprs[*out].height = 1;
                return parse_index(c, *out, out);
            }
        }
        if (peek_kind(c) == TK_LPAREN) {
            uint32_t arg0 = 0;
            uint16_t argc = 0;
            Token close;
            advance_token(c);
            if (!enter_nest(c, name.start, name.end)) {
                return 0;
            }
            if (!parse_args(c, &arg0, &argc)) {
                leave_nest(c);
                return 0;
            }
            leave_nest(c);
            close = peek_token(c);
            if (close.kind != TK_RPAREN) {
                add_diag(c, "ORC0101", close.start, close.end, "expected `)`", "unclosed argument list", NULL, 1);
                return 0;
            }
            advance_token(c);
            if (!new_expr(c, out)) {
                return 0;
            }
            c->exprs[*out].kind = EX_CALL;
            c->exprs[*out].start = name.start;
            c->exprs[*out].end = close.end;
            c->exprs[*out].name_start = name.start;
            c->exprs[*out].name_end = name.end;
            c->exprs[*out].arg0 = arg0;
            c->exprs[*out].argc = argc;
            c->exprs[*out].height = 1;
            for (uint16_t index = 0; index < argc; index++) {
                int child = height_of(c, c->args[arg0 + index]);
                if (1 + child > c->exprs[*out].height) {
                    c->exprs[*out].height = 1 + child;
                }
            }
            if (!note_height(c, *out)) {
                return 0;
            }
            return parse_index(c, *out, out);
        }
        if (!new_expr(c, out)) {
            return 0;
        }
        c->exprs[*out].kind = EX_NAME;
        c->exprs[*out].start = name.start;
        c->exprs[*out].end = name.end;
        c->exprs[*out].name_start = name.start;
        c->exprs[*out].name_end = name.end;
        c->exprs[*out].height = 1;
        return parse_index(c, *out, out);
    }
    if (token.kind == TK_LBRACKET) {
        advance_token(c);
        return parse_array(c, token, out);
    }
    if (token.kind == TK_LPAREN) {
        uint32_t inner;
        Token close;
        advance_token(c);
        if (!enter_nest(c, token.start, token.end)) {
            return 0;
        }
        if (!parse_expr(c, &inner)) {
            leave_nest(c);
            return 0;
        }
        leave_nest(c);
        close = peek_token(c);
        if (close.kind != TK_RPAREN) {
            add_diag(c, "ORC0101", close.start, close.end, "expected `)`", "unclosed group", NULL, 1);
            return 0;
        }
        advance_token(c);
        if (!new_expr(c, out)) {
            return 0;
        }
        c->exprs[*out].kind = EX_GROUP;
        c->exprs[*out].left = inner;
        c->exprs[*out].start = token.start;
        c->exprs[*out].end = close.end;
        c->exprs[*out].height = 1 + height_of(c, inner);
        return note_height(c, *out);
    }
    add_diag(c, "ORC0101", token.start, token.end == token.start ? token.end + 0 : token.end,
             "expected an expression", "expected an operand", NULL, 1);
    return 0;
}

static int parse_expr(Compiler *c, uint32_t *out) {
    uint32_t left;
    Token first_op;
    int group;
    if (!parse_prefixed(c, &left)) {
        return 0;
    }
    if (is_as(c)) {
        Token as_token = peek_token(c);
        DeclaredType type;
        advance_token(c);
        if (!parse_type(c, &type, 0)) {
            return 0;
        }
        if (trailing_joiner(c)) {
            ungrouped(c, peek_token(c), as_token);
        }
        if (!new_expr(c, out)) {
            return 0;
        }
        c->exprs[*out].kind = EX_CONV;
        c->exprs[*out].left = left;
        c->exprs[*out].start = c->exprs[left].start;
        c->exprs[*out].end = type.end;
        c->exprs[*out].op_start = as_token.start;
        c->exprs[*out].op_end = as_token.end;
        c->exprs[*out].conv_ty = type.kind;
        c->exprs[*out].conv_ok = type.ok;
        c->exprs[*out].name_start = type.start;
        c->exprs[*out].name_end = type.end;
        c->exprs[*out].ty_len = type.length;
        if (!push_site(c, &type, "conversion", &c->exprs[*out].conv_site)) {
            return 0;
        }
        {
            int base = height_of(c, left);
            if (type.has_mod) {
                int mod_height = height_of(c, type.mod_expr);
                if (mod_height > base) {
                    base = mod_height;
                }
            }
            c->exprs[*out].height = 1 + base;
        }
        return note_height(c, *out);
    }
    if (is_with_update(c)) {
        return parse_update(c, left, out);
    }
    if (!is_binary_kind(peek_kind(c))) {
        *out = left;
        return 1;
    }
    first_op = peek_token(c);
    group = group_of(first_op.kind);
    if (group == 5) {
        Token op = first_op;
        uint32_t amount;
        advance_token(c);
        if (!parse_prefixed(c, &amount)) {
            return 0;
        }
        if (trailing_joiner(c)) {
            ungrouped(c, peek_token(c), op);
        }
        if (!new_expr(c, out)) {
            return 0;
        }
        c->exprs[*out].kind = EX_SHIFT;
        c->exprs[*out].left = left;
        c->exprs[*out].right = amount;
        c->exprs[*out].op = op.kind;
        c->exprs[*out].start = c->exprs[left].start;
        c->exprs[*out].end = c->exprs[amount].end;
        c->exprs[*out].op_start = op.start;
        c->exprs[*out].op_end = op.end;
        c->exprs[*out].height =
            1 + (height_of(c, left) > height_of(c, amount) ? height_of(c, left) : height_of(c, amount));
        return note_height(c, *out);
    }
    if (group == 6 || group == 9) {
        Token op = first_op;
        uint32_t right;
        advance_token(c);
        if (!parse_prefixed(c, &right)) {
            return 0;
        }
        if (trailing_joiner(c)) {
            ungrouped(c, peek_token(c), op);
        }
        return finish_binary(c, left, right, op.kind, op.start, op.end, out);
    }
    if (group == 1) {
        if (!continue_product(c, left, &left)) {
            return 0;
        }
        while ((peek_kind(c) == TK_PLUS || peek_kind(c) == TK_MINUS) && !c->resource) {
            Token op = peek_token(c);
            uint32_t right;
            advance_token(c);
            if (!parse_prefixed(c, &right) || !continue_product(c, right, &right) ||
                !finish_binary(c, left, right, op.kind, op.start, op.end, &left)) {
                return 0;
            }
        }
        if (trailing_joiner(c)) {
            ungrouped(c, peek_token(c), first_op);
        }
        *out = left;
        return 1;
    }
    while (peek_kind(c) == first_op.kind && !c->resource) {
        Token op = peek_token(c);
        uint32_t right;
        advance_token(c);
        if (!parse_prefixed(c, &right) || !finish_binary(c, left, right, op.kind, op.start, op.end, &left)) {
            return 0;
        }
    }
    if (trailing_joiner(c)) {
        ungrouped(c, peek_token(c), first_op);
    }
    *out = left;
    return 1;
}

static int parse_params(Compiler *c, Func *func) {
    func->param0 = c->nparams;
    func->nparams = 0;
    if (peek_kind(c) == TK_RPAREN) {
        return 1;
    }
    for (;;) {
        Token name;
        Param *param;
        if (func->nparams >= MAX_PARAMS) {
            resource_diag(c, "ORC0106", peek_token(c).start, peek_token(c).end,
                          "function exceeds the 64-parameter limit");
            return 0;
        }
        name = peek_token(c);
        if (name.kind != TK_IDENT) {
            add_diag(c, "ORC0101", name.start, name.end, "expected a parameter name", "expected a parameter", NULL, 1);
            return 0;
        }
        if (!ensure_cap((void **)&c->params, &c->param_cap, c->nparams + 1, sizeof(Param), MAX_EXPRS)) {
            resource_diag(c, "ORC0106", name.start, name.end, "parser could not retain parameters");
            return 0;
        }
        param = &c->params[c->nparams];
        memset(param, 0, sizeof *param);
        param->site = UINT32_MAX;
        param->name_start = name.start;
        param->name_end = name.end;
        advance_token(c);
        if (peek_kind(c) != TK_COLON) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `:` after the parameter name",
                     "expected a parameter type", NULL, 1);
            return 0;
        }
        advance_token(c);
        {
            DeclaredType declared;
            if (!parse_type(c, &declared, 1)) {
                return 0;
            }
            store_declared(&declared, &param->type, &param->length, &param->type_ok, &param->length_bad,
                           &param->type_start, &param->type_end, &param->length_start, &param->length_end);
            if (!push_site(c, &declared, "parameter type", &param->site)) {
                return 0;
            }
        }
        c->nparams++;
        func->nparams++;
        if (peek_kind(c) == TK_COMMA) {
            advance_token(c);
            if (peek_kind(c) == TK_RPAREN) {
                break;
            }
            continue;
        }
        break;
    }
    return 1;
}

static int parse_binding(Compiler *c, Func *func) {
    Token let_token = peek_token(c);
    Token name;
    Local *local;
    uint32_t value = UINT32_MAX;
    if (func->nlocals >= MAX_BINDINGS) {
        resource_diag(c, "ORC0106", let_token.start, let_token.end, "function exceeds the 256-binding limit");
        return 0;
    }
    advance_token(c);
    name = peek_token(c);
    if (name.kind != TK_IDENT) {
        add_diag(c, "ORC0101", name.start, name.end, "expected a binding name", "expected a name after `let`", NULL, 1);
        return 0;
    }
    if (!ensure_cap((void **)&c->locals, &c->local_cap, c->nlocals + 1, sizeof(Local), MAX_EXPRS)) {
        resource_diag(c, "ORC0106", name.start, name.end, "parser could not retain bindings");
        return 0;
    }
    local = &c->locals[c->nlocals];
    memset(local, 0, sizeof *local);
    local->site = UINT32_MAX;
    local->name_start = name.start;
    local->name_end = name.end;
    local->name_at = name.start;
    local->name_end_at = name.end;
    advance_token(c);
    if (peek_kind(c) != TK_COLON) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `:` after the binding name",
                 "a binding states its type", NULL, 1);
        return 0;
    }
    advance_token(c);
    {
        DeclaredType declared;
        if (!parse_type(c, &declared, 1)) {
            return 0;
        }
        store_declared(&declared, &local->type, &local->length, &local->type_ok, &local->length_bad, &local->type_start,
                       &local->type_end, &local->length_start, &local->length_end);
        if (!push_site(c, &declared, "binding type", &local->site)) {
            return 0;
        }
    }
    if (peek_kind(c) != TK_EQUAL) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `=`", "expected the binding's value",
                 NULL, 1);
        return 0;
    }
    advance_token(c);
    if (!parse_expr(c, &value)) {
        return 0;
    }
    if (peek_kind(c) != TK_SEMI) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `;`", "a binding ends with `;`", NULL,
                 1);
        return 0;
    }
    advance_token(c);
    local->value = value;
    c->nlocals++;
    func->nlocals++;
    return 1;
}

static int parse_typed_tail(Compiler *c, Func *func, int inside_params_done) {
    uint32_t result_start = 0;
    uint32_t result_end = 0;
    (void)inside_params_done;
    if (peek_kind(c) != TK_ARROW) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `->`",
                 "a function with parameters needs a result type", NULL, 1);
        skip_function_body(c, 0);
        return 1;
    }
    advance_token(c);
    {
        DeclaredType declared;
        if (!parse_type(c, &declared, 1)) {
            skip_function_body(c, 0);
            return 1;
        }
        store_declared(&declared, &func->result, &func->result_len, &func->result_ok, &func->result_length_bad,
                       &result_start, &result_end, &func->result_length_start, &func->result_length_end);
        if (!push_site(c, &declared, "result type", &func->result_site)) {
            skip_function_body(c, 0);
            return 1;
        }
    }
    func->typed = 1;
    func->result_start = result_start;
    func->result_end = result_end;
    if (peek_kind(c) != TK_LBRACE) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `{`", "expected a function body", NULL,
                 1);
        skip_function_body(c, 0);
        return 1;
    }
    advance_token(c);
    func->local0 = c->nlocals;
    while (peek_kind(c) == TK_IDENT && ident_token_is(c, peek_token(c), "let") && c->at + 1 < c->ntokens &&
           c->tokens[c->at + 1].kind == TK_IDENT) {
        if (!parse_binding(c, func)) {
            skip_function_body(c, 1);
            return 1;
        }
    }
    if (peek_kind(c) == TK_RBRACE) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected an expression",
                 "a typed body ends with its result", NULL, 1);
        advance_token(c);
        return 1;
    }
    if (!parse_expr(c, &func->body)) {
        skip_function_body(c, 1);
        return 1;
    }
    if (peek_kind(c) != TK_RBRACE) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `}`",
                 "extra tokens after the result expression", NULL, 1);
        skip_function_body(c, 1);
        return 1;
    }
    advance_token(c);
    return 1;
}

static int parse_function(Compiler *c) {
    Token kind = peek_token(c);
    Token name;
    Func *func;
    int is_impl = kind.kind == TK_IMPL;
    if (!ensure_cap((void **)&c->funcs, &c->func_cap, c->nfuncs + 1, sizeof(Func), MAX_EXPRS)) {
        resource_diag(c, "ORC0106", kind.start, kind.end, "parser could not retain functions");
        return 0;
    }
    func = &c->funcs[c->nfuncs];
    memset(func, 0, sizeof *func);
    func->is_impl = is_impl;
    func->body = UINT32_MAX;
    func->result = TY_NONE;
    func->result_site = UINT32_MAX;
    advance_token(c);
    name = peek_token(c);
    if (name.kind != TK_IDENT) {
        add_diag(c, "ORC0101", name.start, name.end, "expected a function name", "expected an identifier", NULL, 1);
        skip_function_body(c, 0);
        c->nfuncs++;
        return 1;
    }
    func->name_start = name.start;
    func->name_end = name.end;
    advance_token(c);
    if (peek_kind(c) != TK_LPAREN) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `(`", "expected a parameter list", NULL,
                 1);
        skip_function_body(c, 0);
        c->nfuncs++;
        return 1;
    }
    advance_token(c);
    if (is_impl) {
        if (peek_kind(c) != TK_RPAREN) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                     "an `impl` function has no parameters in this slice", "expected `)`", NULL, 1);
            skip_function_body(c, 0);
            c->nfuncs++;
            return 1;
        }
        advance_token(c);
        if (peek_kind(c) != TK_LBRACE) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `{`", "expected an empty body",
                     NULL, 1);
            skip_function_body(c, 0);
            c->nfuncs++;
            return 1;
        }
        advance_token(c);
        if (peek_kind(c) != TK_RBRACE) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `}`",
                     "an `impl` body is empty in this slice", NULL, 1);
            skip_function_body(c, 1);
            c->nfuncs++;
            return 1;
        }
        advance_token(c);
        c->nfuncs++;
        return 1;
    }
    if (peek_kind(c) == TK_RPAREN) {
        advance_token(c);
        if (peek_kind(c) == TK_LBRACE) {
            advance_token(c);
            if (peek_kind(c) != TK_RBRACE) {
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `}`", "expected an empty body",
                         NULL, 1);
                skip_function_body(c, 1);
                c->nfuncs++;
                return 1;
            }
            advance_token(c);
            c->nfuncs++;
            return 1;
        }
        c->nfuncs++;
        return parse_typed_tail(c, func, 1);
    }
    if (!parse_params(c, func)) {
        skip_function_body(c, 0);
        c->nfuncs++;
        return 1;
    }
    if (peek_kind(c) != TK_RPAREN) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `)`", "unclosed parameter list", NULL,
                 1);
        skip_function_body(c, 0);
        c->nfuncs++;
        return 1;
    }
    advance_token(c);
    c->nfuncs++;
    if (peek_kind(c) == TK_LBRACE) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                 "a `spec` with parameters needs a result type", "expected `->`", NULL, 1);
        skip_function_body(c, 0);
        return 1;
    }
    return parse_typed_tail(c, func, 1);
}

static int parse_use(Compiler *c) {
    Token use_token = peek_token(c);
    Token name;
    Token semi;
    advance_token(c);
    name = peek_token(c);
    if (name.kind != TK_IDENT) {
        add_diag(c, "ORC0101", name.start, name.end, "expected a module name", "expected an identifier", NULL, 1);
        return 0;
    }
    advance_token(c);
    semi = peek_token(c);
    if (semi.kind != TK_SEMI) {
        add_diag(c, "ORC0101", semi.start, semi.end, "expected `;`", "a `use` declaration ends with `;`", NULL, 1);
        return 0;
    }
    advance_token(c);
    if (c->nuses >= MAX_USES) {
        resource_diag(c, "ORC0106", use_token.start, semi.end, "module has more than 64 `use` declarations");
        return 1;
    }
    c->uses[c->nuses].span_start = use_token.start;
    c->uses[c->nuses].span_end = semi.end;
    c->uses[c->nuses].name_start = name.start;
    c->uses[c->nuses].name_end = name.end;
    c->nuses++;
    return 1;
}

static int parse_type_decl(Compiler *c) {
    Token name;
    DeclaredType declared;
    Token semi;
    TypeDecl *decl;
    uint32_t site = UINT32_MAX;
    advance_token(c);
    name = peek_token(c);
    if (name.kind != TK_IDENT) {
        add_diag(c, "ORC0101", name.start, name.end, "expected a type name", "expected an identifier", NULL, 1);
        return 0;
    }
    advance_token(c);
    if (peek_kind(c) != TK_EQUAL) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `=`", "a `type` declaration names a type",
                 NULL, 1);
        return 0;
    }
    advance_token(c);
    if (!parse_type(c, &declared, 1)) {
        return 0;
    }
    semi = peek_token(c);
    if (semi.kind != TK_SEMI) {
        add_diag(c, "ORC0101", semi.start, semi.end, "expected `;`", "a `type` declaration ends with `;`", NULL, 1);
        return 0;
    }
    advance_token(c);
    if (c->ntypes >= MAX_TYPE_DECLS) {
        resource_diag(c, "ORC0106", name.start, semi.end, "module has more than 64 `type` declarations");
        return 1;
    }
    if (!push_site(c, &declared, "declared type", &site)) {
        return 0;
    }
    if (!ensure_cap((void **)&c->types, &c->type_cap, c->ntypes + 1, sizeof(TypeDecl), MAX_TYPE_DECLS)) {
        resource_diag(c, "ORC0106", name.start, semi.end, "parser could not retain type declarations");
        return 0;
    }
    decl = &c->types[c->ntypes++];
    memset(decl, 0, sizeof *decl);
    decl->name_start = name.start;
    decl->name_end = name.end;
    decl->site = site;
    return 1;
}

static int parse_source(Compiler *c) {
    Token token = peek_token(c);
    Token year;
    Token name;
    if (token.kind != TK_EDITION) {
        add_diag(c, "ORC0101", token.start, token.end, "expected `edition 2026;`", "a source starts with its edition",
                 NULL, 1);
        return 1;
    }
    advance_token(c);
    year = peek_token(c);
    if (year.kind != TK_INT || !span_is(c, year.start, year.end, "2026")) {
        add_diag(c, "ORC0102", token.start, year.end, "edition must be exactly `edition 2026;`",
                 "this edition is not admitted", "Orange 2026 is the only edition of this slice", 1);
        return 1;
    }
    advance_token(c);
    if (peek_kind(c) != TK_SEMI) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `;` after the edition",
                 "expected `edition 2026;`", NULL, 1);
        return 1;
    }
    advance_token(c);
    if (peek_kind(c) != TK_MODULE) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `module`", "expected one module", NULL,
                 1);
        return 1;
    }
    advance_token(c);
    name = peek_token(c);
    if (name.kind != TK_IDENT) {
        add_diag(c, "ORC0101", name.start, name.end, "expected a module name", "expected an identifier", NULL, 1);
        return 1;
    }
    c->module_start = name.start;
    c->module_end = name.end;
    advance_token(c);
    if (peek_kind(c) != TK_LBRACE) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `{`", "expected a module body", NULL, 1);
        return 1;
    }
    advance_token(c);
    {
        int saw_function = 0;
        int saw_type = 0;
        while (peek_kind(c) != TK_RBRACE && peek_kind(c) != TK_EOF) {
            if (peek_kind(c) == TK_IDENT && ident_token_is(c, peek_token(c), "use")) {
                if (saw_function) {
                    add_diag(c, "ORC0103", peek_token(c).start, peek_token(c).end,
                             "expected a `spec` or `impl` function declaration",
                             "a `use` declaration cannot follow a function",
                             "`use` declarations come first in a module, before its functions", 1);
                    return 1;
                }
                if (saw_type) {
                    add_diag(c, "ORC0103", peek_token(c).start, peek_token(c).end,
                             "expected a `type` declaration or a function",
                             "a `use` declaration cannot follow a `type` declaration",
                             "a module's `use` declarations come first, then its `type` declarations, then its functions",
                             1);
                    if (!parse_use(c) || c->resource) {
                        return 1;
                    }
                    continue;
                }
                if (!parse_use(c) || c->resource) {
                    return 1;
                }
                continue;
            }
            if (peek_kind(c) == TK_IDENT && ident_token_is(c, peek_token(c), "type")) {
                if (saw_function) {
                    add_diag(c, "ORC0103", peek_token(c).start, peek_token(c).end,
                             "expected a `spec` or `impl` function declaration",
                             "a `type` declaration cannot follow a function",
                             "`type` declarations come before a module's functions", 1);
                    if (!parse_type_decl(c) || c->resource) {
                        return 1;
                    }
                    continue;
                }
                saw_type = 1;
                if (!parse_type_decl(c) || c->resource) {
                    return 1;
                }
                continue;
            }
            if (peek_kind(c) == TK_SPEC || peek_kind(c) == TK_IMPL) {
                saw_function = 1;
                if (!parse_function(c) || c->resource) {
                    return 1;
                }
                continue;
            }
            add_diag(c, "ORC0103", peek_token(c).start, peek_token(c).end,
                     saw_type ? "expected a `type` declaration or a function" : "expected a function declaration",
                     saw_type ? "a module member must be a `type` declaration or a function"
                              : "a module member must be `spec` or `impl`",
                     NULL, 1);
            return 1;
        }
    }
    if (peek_kind(c) != TK_RBRACE) {
        add_diag(c, "ORC0103", peek_token(c).start, peek_token(c).end, "expected a function declaration",
                 "a module member must be `spec` or `impl`", NULL, 1);
        return 1;
    }
    advance_token(c);
    if (peek_kind(c) != TK_EOF) {
        add_diag(c, "ORC0104", peek_token(c).start, peek_token(c).end, "syntax follows the module",
                 "one source holds one module", NULL, 1);
    }
    return 1;
}

static const char *type_spelling(TypeKind type) {
    switch (type) {
    case TY_INT: return "Int";
    case TY_BOOL: return "Bool";
    case TY_W8: return "Word[8]";
    case TY_W16: return "Word[16]";
    case TY_W32: return "Word[32]";
    case TY_W64: return "Word[64]";
    default: return "?";
    }
}

static int type_width(TypeKind type) {
    switch (type) {
    case TY_W8: return 8;
    case TY_W16: return 16;
    case TY_W32: return 32;
    case TY_W64: return 64;
    default: return 0;
    }
}

static void write_type(char *buffer, size_t cap, TypeKind type, uint32_t length) {
    if (length == 0) {
        snprintf(buffer, cap, "%s", type_spelling(type));
    } else {
        snprintf(buffer, cap, "%s^%u", type_spelling(type), length);
    }
}

static void reject_declared(Compiler *c, TypeKind type, int length_bad, uint32_t start, uint32_t end,
                            uint32_t length_start, uint32_t length_end) {
    if (length_bad) {
        char message[128];
        snprintf(message, sizeof message, "array length must be a decimal integer from 1 through %u", MAX_ARRAY_LENGTH);
        add_diag(c, "ORC0221", length_start, length_end, message, "unsupported array length",
                 "write the length in decimal without a prefix, separator, or leading zero", 2);
        return;
    }
    reject_type(c, type, 0, start, end);
}

static int signature_is_usable(const Compiler *c, uint32_t func_index) {
    const Func *func = &c->funcs[func_index];
    uint16_t param;
    if (!func->typed || !func->result_ok) {
        return 0;
    }
    for (param = 0; param < func->nparams; param++) {
        if (!c->params[func->param0 + param].type_ok) {
            return 0;
        }
    }
    return 1;
}

static int find_function_in(const Compiler *mod, const char *text, uint32_t start, uint32_t end, uint32_t *index,
                           int *empty_spec, int *impl) {
    uint32_t cursor;
    size_t length = (size_t)(end - start);
    *empty_spec = 0;
    *impl = 0;
    *index = UINT32_MAX;
    for (cursor = 0; cursor < mod->nfuncs; cursor++) {
        const Func *func = &mod->funcs[cursor];
        if (func->duplicate || (size_t)(func->name_end - func->name_start) != length ||
            memcmp(mod->text + func->name_start, text + start, length) != 0) {
            continue;
        }
        if (func->is_impl) {
            *impl = 1;
            continue;
        }
        if (!func->typed) {
            *empty_spec = 1;
            continue;
        }
        *index = cursor;
        return 1;
    }
    return 0;
}

static int find_function(const Compiler *c, uint32_t start, uint32_t end, uint32_t *index, int *empty_spec, int *impl) {
    return find_function_in(c, c->text, start, end, index, empty_spec, impl);
}

static int module_declares_spec(const Compiler *mod, const char *text, uint32_t start, uint32_t end) {
    uint32_t cursor;
    size_t length = (size_t)(end - start);
    for (cursor = 0; cursor < mod->nfuncs; cursor++) {
        const Func *func = &mod->funcs[cursor];
        if (func->is_impl || func->duplicate) {
            continue;
        }
        if ((size_t)(func->name_end - func->name_start) == length &&
            memcmp(mod->text + func->name_start, text + start, length) == 0) {
            return 1;
        }
    }
    return 0;
}

typedef struct Callee {
    Compiler *mod;
    uint16_t mod_index;
    uint32_t func;
    int found;
    int empty_spec;
    int is_impl;
    int not_used;
    int is_self;
    int qualified;
} Callee;

static void resolve_callee(Compiler *c, const Expr *expr, Callee *out) {
    memset(out, 0, sizeof *out);
    out->func = UINT32_MAX;
    out->mod = c;
    out->mod_index = c->self_index;
    out->qualified = expr->left != UINT32_MAX;
    if (!out->qualified) {
        out->found = find_function(c, expr->name_start, expr->name_end, &out->func, &out->empty_spec, &out->is_impl);
        return;
    }
    if (same_span(c, c->module_start, c->module_end, expr->left, expr->right)) {
        out->not_used = 1;
        out->is_self = 1;
        return;
    }
    {
        uint16_t use_index;
        int matched = 0;
        uint16_t target = UINT16_MAX;
        for (use_index = 0; use_index < c->nuses; use_index++) {
            if (!same_span(c, c->uses[use_index].name_start, c->uses[use_index].name_end, expr->left, expr->right)) {
                continue;
            }
            matched = 1;
            if (c->program != NULL) {
                target = c->program->use_target[c->self_index][use_index];
            }
            break;
        }
        if (!matched || c->program == NULL || target == UINT16_MAX || target >= c->program->nmods) {
            out->not_used = 1;
            return;
        }
        out->mod = c->program->mods[target];
        out->mod_index = target;
        out->found = find_function_in(out->mod, c->text, expr->name_start, expr->name_end, &out->func, &out->empty_spec,
                                      &out->is_impl);
    }
}

static void resolve_name(Compiler *c, uint32_t func_index, uint32_t locals_in_scope, uint32_t start, uint32_t end,
                         NameRes *res, uint16_t *slot, TypeKind *type, uint32_t *length, int *type_ok) {
    const Func *func = &c->funcs[func_index];
    uint16_t index;
    *res = NAME_MISSING;
    *slot = 0;
    *type = TY_NONE;
    *length = 0;
    *type_ok = 0;
    for (index = 0; index < func->nparams; index++) {
        const Param *param = &c->params[func->param0 + index];
        if (param->duplicate) {
            continue;
        }
        if (same_span(c, param->name_start, param->name_end, start, end)) {
            *res = param->type_ok ? NAME_PARAM : NAME_BAD;
            *slot = index;
            *type = param->type;
            *length = param->length;
            *type_ok = param->type_ok;
            return;
        }
    }
    /* nlocals is uint16_t and at most MAX_BINDINGS, so the visible count
       fits the slot. Compare equal-width counters so a wide count cannot
       wrap the loop. */
    {
        uint32_t visible = locals_in_scope;
        if (visible > func->nlocals) {
            visible = func->nlocals;
        }
        for (uint32_t local_index = 0; local_index < visible; local_index++) {
            const Local *local = &c->locals[func->local0 + local_index];
            if (local->duplicate) {
                continue;
            }
            if (same_span(c, local->name_start, local->name_end, start, end)) {
                *res = local->type_ok ? NAME_LOCAL : NAME_BAD;
                *slot = (uint16_t)local_index;
                *type = local->type;
                *length = local->length;
                *type_ok = local->type_ok;
                return;
            }
        }
    }
    for (index = 0; index < func->nlocals; index++) {
        const Local *local = &c->locals[func->local0 + index];
        if (same_span(c, local->name_start, local->name_end, start, end)) {
            *res = NAME_EARLY;
            *slot = index;
            return;
        }
    }
}

static int check_expr(Compiler *c, uint32_t index, TypeKind expected, uint32_t expected_len, uint32_t func_index,
                      uint32_t locals_in_scope);
static int check_at(Compiler *c, uint32_t index, TypeKind expected, uint32_t expected_len, uint16_t expected_mod,
                    uint32_t func_index, uint32_t locals_in_scope);
static int find_leaf(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, TypeKind *type,
                     uint32_t *length, uint32_t *leaf, int *silent);

static int decode_literal(Compiler *c, const Expr *expr, Big *out) {
    return big_from_digits(&c->arena, c->text + expr->lit_start, (size_t)(expr->lit_end - expr->lit_start),
                           expr->negative, out);
}

static void check_literal(Compiler *c, const Expr *expr, TypeKind expected, uint16_t expected_mod) {
    Big value = big_zero();
    if (!decode_literal(c, expr, &value)) {
        add_diag(c, "ORC0205", expr->start, expr->end, "integer magnitude exceeds 16384 significant bits",
                 "literal is too large", "Int is unbounded, but one literal must fit the representation budget", 2);
        return;
    }
    if (expected == TY_INT) {
        return;
    }
    if (expected == TY_MOD) {
        Big magnitude = value;
        const Big *modulus;
        magnitude.negative = 0;
        if (expected_mod == 0 || expected_mod >= c->nmoduli) {
            return;
        }
        modulus = &c->moduli[expected_mod];
        if (big_cmp(&magnitude, modulus) >= 0) {
            add_diag(c, "ORC0207", expr->lit_start, expr->lit_end, "literal is outside the range of its modulus",
                     "the literal's magnitude is not less than the modulus",
                     "a literal of `Mod[m]` has a magnitude n less than m, and `-n` stands for m - n; residues do not "
                     "reduce out-of-range literals",
                     2);
        }
        return;
    }
    if (expected == TY_BOOL) {
        add_diag(c, "ORC0214", expr->start, expr->end, "an integer literal cannot have type `Bool`", "type mismatch",
                 "the `Bool` values are written `true` and `false`", 2);
        return;
    }
    if (expr->negative) {
        add_diag(c, "ORC0206", expr->start, expr->end, "a word literal cannot be negative",
                 "write the residue in range instead", "word literals are canonical residues, never a sign", 2);
        return;
    }
    if (big_bits(&value) > (uint32_t)type_width(expected)) {
        add_diag(c, "ORC0207", expr->start, expr->end, "word literal is outside its type",
                 "this value does not fit the word", "a word literal must lie in 0 through 2^n - 1", 2);
    }
}

static int base_type(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, TypeKind *type,
                     uint32_t *length, int *silent) {
    const Expr *expr = &c->exprs[index];
    *silent = 0;
    *type = TY_NONE;
    *length = 0;
    c->leaf_mod = 0;
    if (expr->kind == EX_NAME) {
        NameRes res;
        uint16_t slot = 0;
        int type_ok = 0;
        resolve_name(c, func_index, locals_in_scope, expr->name_start, expr->name_end, &res, &slot, type, length,
                     &type_ok);
        if (res == NAME_BAD) {
            *silent = 1;
            return -1;
        }
        if (res == NAME_PARAM || res == NAME_LOCAL) {
            if (*type == TY_MOD) {
                c->leaf_mod = res == NAME_PARAM ? c->params[c->funcs[func_index].param0 + slot].mod_index
                                                : c->locals[c->funcs[func_index].local0 + slot].mod_index;
            }
            return 1;
        }
        return 0;
    }
    if (expr->kind == EX_CALL) {
        Callee callee;
        resolve_callee(c, expr, &callee);
        if (callee.not_used || !callee.found) {
            return 0;
        }
        if (!signature_is_usable(callee.mod, callee.func)) {
            *silent = 1;
            return -1;
        }
        *type = callee.mod->funcs[callee.func].result;
        *length = callee.mod->funcs[callee.func].result_len;
        if (*type == TY_MOD) {
            c->leaf_mod = callee.mod->funcs[callee.func].result_mod;
        }
        return 1;
    }
    if (expr->kind == EX_ACCUM || expr->kind == EX_LOOP) {
        const LoopDesc *loop = &c->loops[expr->arg0];
        if (!loop->acc_ok) {
            *silent = 1;
            return -1;
        }
        *type = loop->acc_type;
        *length = loop->acc_len;
        if (*type == TY_MOD) {
            c->leaf_mod = loop->acc_mod;
        }
        return 1;
    }
    return 0;
}

static int index_below(const Big *value, uint32_t length) {
    if (value->negative || value->nlimbs > 1) {
        return 0;
    }
    if (value->nlimbs == 0) {
        return 1;
    }
    return value->limbs[0] < length;
}

/* Type of the value an index selects from. A chain sees through earlier
   indices, so `t[0][1]` knows that `t[0]` is one element. */
static int index_subject(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, TypeKind *type,
                         uint32_t *length, int *silent) {
    const Expr *expr = &c->exprs[index];
    uint32_t leaf = index;
    if (expr->kind == EX_INDEX || expr->kind == EX_SELECT) {
        return find_leaf(c, index, func_index, locals_in_scope, type, length, &leaf, silent);
    }
    return base_type(c, index, func_index, locals_in_scope, type, length, silent);
}

static int find_leaf(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, TypeKind *type,
                     uint32_t *length, uint32_t *leaf, int *silent) {
    const Expr *expr = &c->exprs[index];
    int left_state;
    *silent = 0;
    *leaf = index;
    *type = TY_NONE;
    *length = 0;
    c->leaf_mod = 0;
    switch (expr->kind) {
    case EX_LIT:
        return 0;
    case EX_ARRAY:
        return 2;
    case EX_GROUP:
    case EX_UNARY:
        return find_leaf(c, expr->left, func_index, locals_in_scope, type, length, leaf, silent);
    case EX_SHIFT:
        return find_leaf(c, expr->left, func_index, locals_in_scope, type, length, leaf, silent);
    case EX_BINARY:
        if (is_compare_op(expr->op)) {
            *type = TY_BOOL;
            *length = 0;
            return 1;
        }
        left_state = find_leaf(c, expr->left, func_index, locals_in_scope, type, length, leaf, silent);
        if (left_state != 0) {
            return left_state;
        }
        return find_leaf(c, expr->right, func_index, locals_in_scope, type, length, leaf, silent);
    case EX_COND: {
        uint16_t arm;
        for (arm = 0; arm < expr->argc; arm++) {
            int state = find_leaf(c, c->cond_arms[expr->arg0 + arm].value, func_index, locals_in_scope, type, length,
                                  leaf, silent);
            if (state != 0) {
                return state;
            }
        }
        return find_leaf(c, expr->right, func_index, locals_in_scope, type, length, leaf, silent);
    }
    case EX_NAME: {
        NameRes res;
        uint16_t slot;
        int type_ok = 0;
        resolve_name(c, func_index, locals_in_scope, expr->name_start, expr->name_end, &res, &slot, type, length,
                     &type_ok);
        if (res == NAME_BAD) {
            *silent = 1;
            return -1;
        }
        if (res == NAME_PARAM || res == NAME_LOCAL) {
            if (*type == TY_MOD) {
                c->leaf_mod = res == NAME_PARAM ? c->params[c->funcs[func_index].param0 + slot].mod_index
                                                : c->locals[c->funcs[func_index].local0 + slot].mod_index;
            }
            return 1;
        }
        if (span_is(c, expr->name_start, expr->name_end, "true") ||
            span_is(c, expr->name_start, expr->name_end, "false")) {
            *type = TY_BOOL;
            *length = 0;
            return 1;
        }
        return -1;
    }
    case EX_CALL: {
        Callee callee;
        resolve_callee(c, expr, &callee);
        if (callee.not_used || !callee.found) {
            return -1;
        }
        if (!signature_is_usable(callee.mod, callee.func)) {
            *silent = 1;
            return -1;
        }
        *type = callee.mod->funcs[callee.func].result;
        *length = callee.mod->funcs[callee.func].result_len;
        if (*type == TY_MOD) {
            c->leaf_mod = callee.mod->funcs[callee.func].result_mod;
        }
        return 1;
    }
    case EX_LOOP_INDEX:
        *type = TY_INT;
        *length = 0;
        return 1;
    case EX_ACCUM: {
        const LoopDesc *loop = &c->loops[expr->arg0];
        if (!loop->acc_ok) {
            *silent = 1;
            return -1;
        }
        *type = loop->acc_type;
        *length = loop->acc_len;
        if (*type == TY_MOD) {
            c->leaf_mod = loop->acc_mod;
        }
        return 1;
    }
    case EX_LOOP: {
        const LoopDesc *loop = &c->loops[expr->arg0];
        if (!loop->acc_ok) {
            *silent = 1;
            return -1;
        }
        *type = loop->acc_type;
        *length = loop->acc_len;
        if (*type == TY_MOD) {
            c->leaf_mod = loop->acc_mod;
        }
        return 1;
    }
    case EX_FILL:
    case EX_UPDATE:
        return 2;
    case EX_INDEX:
    case EX_SELECT: {
        int state = index_subject(c, expr->left, func_index, locals_in_scope, type, length, silent);
        if (state != 1) {
            if (state < 0) {
                return -1;
            }
            *silent = 0;
            return -1;
        }
        /* An index of a scalar still has that scalar's type. The checker
           reports ORC0224 once, on the first index that left the array. */
        if (*length == 0) {
            return 1;
        }
        *length = 0;
        return 1;
    }
    case EX_CONV:
        /* An invalid target is still a leaf. Callers check this node so the
           width or unsupported type is reported; silencing it dropped that
           diagnostic when the conversion was itself an operand. */
        if (!expr->conv_ok) {
            return -1;
        }
        *type = expr->conv_ty;
        if (*type == TY_MOD) {
            c->leaf_mod = expr->conv_mod;
        }
        return 1;
    default:
        return 0;
    }
}

static void report_unknown_name(Compiler *c, const Expr *expr, uint32_t func_index, NameRes res) {
    char message[192];
    const char *func_name_start = c->text + c->funcs[func_index].name_start;
    size_t func_len = c->funcs[func_index].name_end - c->funcs[func_index].name_start;
    char func_name[64];
    char ident[64];
    size_t ident_len = expr->name_end - expr->name_start;
    if (func_len >= sizeof func_name) {
        func_len = sizeof func_name - 1;
    }
    if (ident_len >= sizeof ident) {
        ident_len = sizeof ident - 1;
    }
    memcpy(func_name, func_name_start, func_len);
    func_name[func_len] = '\0';
    memcpy(ident, c->text + expr->name_start, ident_len);
    ident[ident_len] = '\0';
    if (res == NAME_EARLY) {
        snprintf(message, sizeof message, "`%s` is used before it is bound", ident);
        add_diag(c, "ORC0211", expr->start, expr->end, message, "binding is not in scope yet",
                 "a binding is in scope after its `;`", 2);
        return;
    }
    if (c->funcs[func_index].nlocals > 0) {
        snprintf(message, sizeof message, "`%s` is not a parameter or binding of `%s`", ident, func_name);
    } else {
        snprintf(message, sizeof message, "`%s` is not a parameter of `%s`", ident, func_name);
    }
    add_diag(c, "ORC0211", expr->start, expr->end, message, "unknown name",
             "a bare name refers to a parameter or a binding in scope", 2);
}

static int record_edge(Compiler *c, uint32_t func_index, uint32_t callee, uint32_t start, uint32_t end) {
    Func *func = &c->funcs[func_index];
    if (func->nedges == 0) {
        func->edge0 = c->nedges;
    }
    if (!ensure_cap((void **)&c->edges, &c->edge_cap, c->nedges + 1, sizeof(Edge), MAX_EXPRS)) {
        resource_diag(c, "ORC0209", start, end, "semantic analysis could not retain the call graph");
        return 0;
    }
    c->edges[c->nedges].callee = callee;
    c->edges[c->nedges].start = start;
    c->edges[c->nedges].end = end;
    c->nedges++;
    func->nedges++;
    return 1;
}

static int name_is_active_loop(const Compiler *c, uint32_t start, uint32_t end) {
    int index;
    for (index = 0; index < c->nactive; index++) {
        const LoopDesc *loop = &c->loops[c->active_loops[index]];
        if (same_span(c, loop->index_start, loop->index_end, start, end) ||
            same_span(c, loop->acc_start, loop->acc_end, start, end)) {
            return 1;
        }
    }
    return 0;
}

static int report_duplicate_loop_name(Compiler *c, uint32_t func_index, uint32_t locals_in_scope, uint32_t start,
                                      uint32_t end) {
    const Func *func = &c->funcs[func_index];
    uint16_t index;
    int duplicate = name_is_active_loop(c, start, end);
    for (index = 0; index < func->nparams && !duplicate; index++) {
        const Param *param = &c->params[func->param0 + index];
        if (!param->duplicate && same_span(c, param->name_start, param->name_end, start, end)) {
            duplicate = 1;
        }
    }
    if (locals_in_scope > func->nlocals) {
        locals_in_scope = func->nlocals;
    }
    for (index = 0; index < locals_in_scope && !duplicate; index++) {
        const Local *local = &c->locals[func->local0 + index];
        if (!local->duplicate && same_span(c, local->name_start, local->name_end, start, end)) {
            duplicate = 1;
        }
    }
    if (!duplicate) {
        return 0;
    }
    add_diag(c, "ORC0219", start, end, "duplicate name", "this name is already in scope",
             "a loop index and accumulator are new names", 2);
    return 1;
}

/* Returns whether the bound fits the integer literal budget. A failure is
   ORC0205, the same code as any other oversized literal. A bound that fits
   but is outside 0..65536 sets *above; the caller reports ORC0225. */
static int decode_loop_bound(Compiler *c, uint32_t start, uint32_t end, int *above, uint32_t *value) {
    Big magnitude = big_zero();
    *above = 0;
    *value = 0;
    if (!big_from_digits(&c->arena, c->text + start, (size_t)(end - start), 0, &magnitude)) {
        add_diag(c, "ORC0205", start, end, "integer magnitude exceeds 16384 significant bits",
                 "literal is too large", "Int is unbounded, but one literal must fit the representation budget", 2);
        return 0;
    }
    if (magnitude.nlimbs > 1 || (magnitude.nlimbs == 1 && magnitude.limbs[0] > MAX_LOOP_BOUND)) {
        *above = 1;
        return 1;
    }
    if (magnitude.nlimbs == 1) {
        *value = magnitude.limbs[0];
    }
    return 1;
}

static uint64_t word_maximum(TypeKind type) {
    int width = type_width(type);
    if (width >= 64) {
        return UINT64_MAX;
    }
    if (width <= 0) {
        return 0;
    }
    return (UINT64_C(1) << width) - 1u;
}

/* Least value of the form 2^k - 1 that is at least `value`. */
static uint64_t all_ones_u64(uint64_t value) {
    if (value == 0) {
        return 0;
    }
    return UINT64_MAX >> __builtin_clzll(value);
}

static int big_as_u64(const Big *value, uint64_t *out) {
    if (value->negative) {
        return 0;
    }
    if (value->nlimbs == 0) {
        *out = 0;
        return 1;
    }
    if (value->nlimbs == 1) {
        *out = value->limbs[0];
        return 1;
    }
    if (value->nlimbs == 2) {
        *out = (uint64_t)value->limbs[0] | ((uint64_t)value->limbs[1] << 32);
        return 1;
    }
    return 0;
}

static int literal_shift_amount(Compiler *c, const Expr *amount, uint32_t *out) {
    Big magnitude = big_zero();
    uint64_t value = 0;
    if (amount->kind != EX_LIT || amount->negative || !decode_literal(c, amount, &magnitude) ||
        !big_as_u64(&magnitude, &value) || value > UINT32_MAX) {
        return 0;
    }
    *out = (uint32_t)value;
    return 1;
}

/* Least and greatest values of a word expression whose type ends at `maximum`.
   An operator narrows that only where the result cannot wrap. Anything else
   ranges over the whole type. Bounds stay within `maximum`. */
static void word_range(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, uint64_t maximum,
                       uint64_t *lo, uint64_t *hi) {
    const Expr *expr = &c->exprs[index];
    *lo = 0;
    *hi = maximum;
    switch (expr->kind) {
    case EX_LIT: {
        Big magnitude = big_zero();
        uint64_t value = 0;
        if (expr->negative || !decode_literal(c, expr, &magnitude) || !big_as_u64(&magnitude, &value) ||
            value > maximum) {
            return;
        }
        *lo = value;
        *hi = value;
        return;
    }
    case EX_GROUP:
        word_range(c, expr->left, func_index, locals_in_scope, maximum, lo, hi);
        return;
    case EX_UNARY:
        if (expr->op == TK_TILDE) {
            uint64_t inner_lo = 0;
            uint64_t inner_hi = 0;
            word_range(c, expr->left, func_index, locals_in_scope, maximum, &inner_lo, &inner_hi);
            *lo = maximum - inner_hi;
            *hi = maximum - inner_lo;
        }
        return;
    case EX_SHIFT: {
        uint64_t low = 0;
        uint64_t high = 0;
        uint32_t amount = 0;
        word_range(c, expr->left, func_index, locals_in_scope, maximum, &low, &high);
        if (!literal_shift_amount(c, &c->exprs[expr->right], &amount)) {
            return;
        }
        if (expr->op == TK_RSHIFT) {
            if (amount >= 64) {
                *lo = 0;
                *hi = 0;
            } else {
                *lo = low >> amount;
                *hi = high >> amount;
            }
            return;
        }
        if (expr->op == TK_LSHIFT) {
            uint64_t greatest;
            if (amount >= 128) {
                return;
            }
            if (amount >= 64) {
                if (high == 0) {
                    *lo = 0;
                    *hi = 0;
                }
                return;
            }
            greatest = high << amount;
            if ((greatest >> amount) == high && greatest <= maximum) {
                *lo = low << amount;
                *hi = greatest;
            }
            return;
        }
        return;
    }
    case EX_BINARY: {
        uint64_t left_lo = 0;
        uint64_t left_hi = 0;
        uint64_t right_lo = 0;
        uint64_t right_hi = 0;
        uint64_t peak;
        word_range(c, expr->left, func_index, locals_in_scope, maximum, &left_lo, &left_hi);
        word_range(c, expr->right, func_index, locals_in_scope, maximum, &right_lo, &right_hi);
        switch (expr->op) {
        case TK_AMP:
            *lo = 0;
            *hi = left_hi < right_hi ? left_hi : right_hi;
            return;
        case TK_PIPE:
            peak = left_hi > right_hi ? left_hi : right_hi;
            *lo = left_lo > right_lo ? left_lo : right_lo;
            *hi = all_ones_u64(peak);
            return;
        case TK_CARET:
            peak = left_hi > right_hi ? left_hi : right_hi;
            *lo = 0;
            *hi = all_ones_u64(peak);
            return;
        case TK_PLUS:
            if (left_lo > UINT64_MAX - right_lo || left_hi > UINT64_MAX - right_hi || left_hi + right_hi > maximum) {
                return;
            }
            *lo = left_lo + right_lo;
            *hi = left_hi + right_hi;
            return;
        case TK_MINUS:
            if (left_lo < right_hi) {
                return;
            }
            *lo = left_lo - right_hi;
            *hi = left_hi - right_lo;
            return;
        case TK_STAR:
            if ((right_hi != 0 && left_hi > UINT64_MAX / right_hi) || left_hi * right_hi > maximum) {
                return;
            }
            *lo = left_lo * right_lo;
            *hi = left_hi * right_hi;
            return;
        case TK_SLASH:
            if (right_lo == 0 || right_hi == 0) {
                *lo = 0;
                *hi = left_hi;
            } else {
                *lo = left_lo / right_hi;
                *hi = left_hi / right_lo;
            }
            return;
        case TK_PERCENT:
            if (right_lo > 0 && left_hi < right_lo) {
                *lo = left_lo;
                *hi = left_hi;
            } else if (right_lo > 0) {
                uint64_t cap = right_hi - 1u;
                *lo = 0;
                *hi = left_hi < cap ? left_hi : cap;
            } else {
                *lo = 0;
                *hi = left_hi;
            }
            return;
        default:
            return;
        }
    }
    case EX_COND: {
        uint16_t arm;
        uint64_t least = 0;
        uint64_t greatest = 0;
        int have = 0;
        for (arm = 0; arm < expr->argc; arm++) {
            uint64_t arm_lo = 0;
            uint64_t arm_hi = 0;
            word_range(c, c->cond_arms[expr->arg0 + arm].value, func_index, locals_in_scope, maximum, &arm_lo, &arm_hi);
            if (!have || arm_lo < least) {
                least = arm_lo;
            }
            if (!have || arm_hi > greatest) {
                greatest = arm_hi;
            }
            have = 1;
        }
        {
            uint64_t arm_lo = 0;
            uint64_t arm_hi = 0;
            word_range(c, expr->right, func_index, locals_in_scope, maximum, &arm_lo, &arm_hi);
            if (!have || arm_lo < least) {
                least = arm_lo;
            }
            if (!have || arm_hi > greatest) {
                greatest = arm_hi;
            }
            have = 1;
        }
        if (have) {
            *lo = least;
            *hi = greatest;
        }
        return;
    }
    case EX_CONV: {
        TypeKind leaf_type = TY_NONE;
        uint32_t leaf_len = 0;
        uint32_t leaf = index;
        int silent = 0;
        int state = find_leaf(c, expr->left, func_index, locals_in_scope, &leaf_type, &leaf_len, &leaf, &silent);
        uint64_t from_max;
        uint64_t inner_lo = 0;
        uint64_t inner_hi = 0;
        if (state == 1 && leaf_len == 0 && type_width(leaf_type) != 0) {
            from_max = word_maximum(leaf_type);
            word_range(c, expr->left, func_index, locals_in_scope, from_max, &inner_lo, &inner_hi);
            if (inner_hi <= maximum) {
                *lo = inner_lo;
                *hi = inner_hi;
            }
        }
        return;
    }
    default:
        return;
    }
}

static int static_index(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, uint32_t *bad) {
    const Expr *expr = &c->exprs[index];
    switch (expr->kind) {
    case EX_LIT:
    case EX_LOOP_INDEX:
        return 1;
    case EX_GROUP:
        return static_index(c, expr->left, func_index, locals_in_scope, bad);
    case EX_UNARY:
        if (expr->op != TK_MINUS) {
            *bad = index;
            return 0;
        }
        return static_index(c, expr->left, func_index, locals_in_scope, bad);
    case EX_BINARY:
        if (expr->op != TK_PLUS && expr->op != TK_MINUS && expr->op != TK_STAR && expr->op != TK_SLASH &&
            expr->op != TK_PERCENT) {
            *bad = index;
            return 0;
        }
        if (!static_index(c, expr->left, func_index, locals_in_scope, bad)) {
            return 0;
        }
        return static_index(c, expr->right, func_index, locals_in_scope, bad);
    case EX_COND: {
        uint16_t arm;
        for (arm = 0; arm < expr->argc; arm++) {
            if (!static_index(c, c->cond_arms[expr->arg0 + arm].value, func_index, locals_in_scope, bad)) {
                return 0;
            }
        }
        return static_index(c, expr->right, func_index, locals_in_scope, bad);
    }
    case EX_CONV: {
        TypeKind leaf_type = TY_NONE;
        uint32_t leaf_len = 0;
        uint32_t leaf = index;
        int silent = 0;
        int state;
        if (!expr->conv_ok || expr->conv_ty != TY_INT) {
            *bad = index;
            return 0;
        }
        state = find_leaf(c, expr->left, func_index, locals_in_scope, &leaf_type, &leaf_len, &leaf, &silent);
        if (state == 1 && leaf_len == 0 && type_width(leaf_type) != 0) {
            return 1;
        }
        if (state == 1 && leaf_len == 0 && leaf_type == TY_MOD) {
            return 1;
        }
        if (state == 1 && leaf_type == TY_INT && leaf_len == 0) {
            return static_index(c, expr->left, func_index, locals_in_scope, bad);
        }
        *bad = index;
        return 0;
    }
    default:
        *bad = index;
        return 0;
    }
}

static int range_of(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, Big *lo, Big *hi);

static int range_combine(Compiler *c, TokenKind op, const Big *left_lo, const Big *left_hi, const Big *right_lo,
                         const Big *right_hi, Big *lo, Big *hi) {
    if (op == TK_PLUS) {
        return big_add(&c->arena, left_lo, right_lo, lo) && big_add(&c->arena, left_hi, right_hi, hi);
    }
    if (op == TK_MINUS) {
        return big_sub(&c->arena, left_lo, right_hi, lo) && big_sub(&c->arena, left_hi, right_lo, hi);
    }
    if (op == TK_STAR) {
        const Big *lefts[2] = {left_lo, left_hi};
        const Big *rights[2] = {right_lo, right_hi};
        Big product = big_zero();
        int first = 1;
        int i;
        int j;
        for (i = 0; i < 2; i++) {
            for (j = 0; j < 2; j++) {
                if (!big_mul(&c->arena, lefts[i], rights[j], &product)) {
                    return 0;
                }
                if (first || big_cmp(&product, lo) < 0) {
                    *lo = product;
                }
                if (first || big_cmp(&product, hi) > 0) {
                    *hi = product;
                }
                first = 0;
            }
        }
        return 1;
    }
    return 0;
}

static int euclid_quot(Compiler *c, const Big *value, const Big *divisor, Big *quot) {
    Big rem = big_zero();
    if (divisor->nlimbs == 0) {
        return big_from_u64(&c->arena, 0, quot);
    }
    return big_div_euclid(&c->arena, value, divisor, quot, &rem);
}

static int positive_divisor_range(Compiler *c, TokenKind op, const Big *low, const Big *high, const Big *divisor_low,
                                  const Big *divisor_high, Big *out_lo, Big *out_hi) {
    if (op == TK_SLASH) {
        Big corners[4];
        const Big *xs[2] = {low, high};
        const Big *ds[2] = {divisor_low, divisor_high};
        int i;
        int j;
        int first = 1;
        for (i = 0; i < 2; i++) {
            for (j = 0; j < 2; j++) {
                if (!euclid_quot(c, xs[i], ds[j], &corners[i * 2 + j])) {
                    return 0;
                }
                if (first || big_cmp(&corners[i * 2 + j], out_lo) < 0) {
                    *out_lo = corners[i * 2 + j];
                }
                if (first || big_cmp(&corners[i * 2 + j], out_hi) > 0) {
                    *out_hi = corners[i * 2 + j];
                }
                first = 0;
            }
        }
        return 1;
    }
    if (big_cmp(divisor_low, divisor_high) == 0) {
        Big low_q = big_zero();
        Big high_q = big_zero();
        Big low_r = big_zero();
        Big high_r = big_zero();
        if (!big_div_euclid(&c->arena, low, divisor_low, &low_q, &low_r) ||
            !big_div_euclid(&c->arena, high, divisor_low, &high_q, &high_r)) {
            return 0;
        }
        if (big_cmp(&low_q, &high_q) == 0) {
            *out_lo = low_r;
            *out_hi = high_r;
            return 1;
        }
    }
    {
        Big one = big_zero();
        Big largest = big_zero();
        if (!big_from_u64(&c->arena, 1, &one) || !big_sub(&c->arena, divisor_high, &one, &largest) ||
            !big_from_u64(&c->arena, 0, out_lo)) {
            return 0;
        }
        if (!(low->negative && low->nlimbs != 0) && big_cmp(high, &largest) < 0) {
            *out_hi = *high;
        } else {
            *out_hi = largest;
        }
    }
    return 1;
}

static int divide_ranges(Compiler *c, TokenKind op, const Big *left_lo, const Big *left_hi, const Big *divisor_lo,
                         const Big *divisor_hi, Big *lo, Big *hi) {
    Big one = big_zero();
    Big minus_one = big_zero();
    Big zero = big_zero();
    int have = 0;
    Big part_lo = big_zero();
    Big part_hi = big_zero();
    if (!big_from_u64(&c->arena, 1, &one) || !big_neg(&one, &minus_one) || !big_from_u64(&c->arena, 0, &zero)) {
        return 0;
    }
    if (big_cmp(divisor_hi, &one) >= 0) {
        Big lowest = big_zero();
        if (big_cmp(divisor_lo, &one) < 0) {
            lowest = one;
        } else {
            lowest = *divisor_lo;
        }
        if (!positive_divisor_range(c, op, left_lo, left_hi, &lowest, divisor_hi, &part_lo, &part_hi)) {
            return 0;
        }
        *lo = part_lo;
        *hi = part_hi;
        have = 1;
    }
    if (big_cmp(divisor_lo, &zero) <= 0 && big_cmp(divisor_hi, &zero) >= 0) {
        Big zero_lo = big_zero();
        Big zero_hi = big_zero();
        if (op == TK_SLASH) {
            zero_lo = zero;
            zero_hi = zero;
        } else {
            zero_lo = *left_lo;
            zero_hi = *left_hi;
        }
        if (!have || big_cmp(&zero_lo, lo) < 0) {
            *lo = zero_lo;
        }
        if (!have || big_cmp(&zero_hi, hi) > 0) {
            *hi = zero_hi;
        }
        have = 1;
    }
    if (big_cmp(divisor_lo, &minus_one) <= 0) {
        Big nearest = big_zero();
        Big farthest = big_zero();
        Big neg_hi = big_zero();
        Big neg_lo = big_zero();
        if (big_cmp(divisor_hi, &minus_one) > 0) {
            nearest = one;
        } else if (!big_neg(divisor_hi, &nearest)) {
            return 0;
        }
        if (!big_neg(divisor_lo, &farthest) ||
            !positive_divisor_range(c, op, left_lo, left_hi, &nearest, &farthest, &neg_lo, &neg_hi)) {
            return 0;
        }
        if (op == TK_SLASH) {
            Big flipped_lo = big_zero();
            Big flipped_hi = big_zero();
            if (!big_neg(&neg_hi, &flipped_lo) || !big_neg(&neg_lo, &flipped_hi)) {
                return 0;
            }
            neg_lo = flipped_lo;
            neg_hi = flipped_hi;
        }
        if (!have || big_cmp(&neg_lo, lo) < 0) {
            *lo = neg_lo;
        }
        if (!have || big_cmp(&neg_hi, hi) > 0) {
            *hi = neg_hi;
        }
        have = 1;
    }
    return have;
}

static int range_of(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, Big *lo, Big *hi) {
    const Expr *expr = &c->exprs[index];
    if (expr->kind == EX_LIT) {
        if (!decode_literal(c, expr, lo)) {
            return 0;
        }
        *hi = *lo;
        return 1;
    }
    if (expr->kind == EX_GROUP) {
        return range_of(c, expr->left, func_index, locals_in_scope, lo, hi);
    }
    if (expr->kind == EX_LOOP_INDEX) {
        const LoopDesc *loop = &c->loops[expr->arg0];
        if (!loop->bounds_ok || loop->bound_b == 0) {
            return 0;
        }
        return big_from_u64(&c->arena, loop->bound_a, lo) && big_from_u64(&c->arena, loop->bound_b - 1u, hi);
    }
    if (expr->kind == EX_UNARY && expr->op == TK_MINUS) {
        Big inner_lo = big_zero();
        Big inner_hi = big_zero();
        if (!range_of(c, expr->left, func_index, locals_in_scope, &inner_lo, &inner_hi)) {
            return 0;
        }
        return big_neg(&inner_hi, lo) && big_neg(&inner_lo, hi);
    }
    if (expr->kind == EX_BINARY && (expr->op == TK_PLUS || expr->op == TK_MINUS || expr->op == TK_STAR ||
                                    expr->op == TK_SLASH || expr->op == TK_PERCENT)) {
        Big left_lo = big_zero();
        Big left_hi = big_zero();
        Big right_lo = big_zero();
        Big right_hi = big_zero();
        if (!range_of(c, expr->left, func_index, locals_in_scope, &left_lo, &left_hi) ||
            !range_of(c, expr->right, func_index, locals_in_scope, &right_lo, &right_hi)) {
            return 0;
        }
        if (expr->op == TK_SLASH || expr->op == TK_PERCENT) {
            return divide_ranges(c, expr->op, &left_lo, &left_hi, &right_lo, &right_hi, lo, hi);
        }
        return range_combine(c, expr->op, &left_lo, &left_hi, &right_lo, &right_hi, lo, hi);
    }
    if (expr->kind == EX_COND) {
        uint16_t arm;
        int have = 0;
        for (arm = 0; arm < expr->argc; arm++) {
            Big arm_lo = big_zero();
            Big arm_hi = big_zero();
            if (!range_of(c, c->cond_arms[expr->arg0 + arm].value, func_index, locals_in_scope, &arm_lo, &arm_hi)) {
                return 0;
            }
            if (!have || big_cmp(&arm_lo, lo) < 0) {
                *lo = arm_lo;
            }
            if (!have || big_cmp(&arm_hi, hi) > 0) {
                *hi = arm_hi;
            }
            have = 1;
        }
        {
            Big arm_lo = big_zero();
            Big arm_hi = big_zero();
            if (!range_of(c, expr->right, func_index, locals_in_scope, &arm_lo, &arm_hi)) {
                return 0;
            }
            if (!have || big_cmp(&arm_lo, lo) < 0) {
                *lo = arm_lo;
            }
            if (!have || big_cmp(&arm_hi, hi) > 0) {
                *hi = arm_hi;
            }
        }
        return 1;
    }
    if (expr->kind == EX_CONV && expr->conv_ok && expr->conv_ty == TY_INT) {
        TypeKind leaf_type = TY_NONE;
        uint32_t leaf_len = 0;
        uint32_t leaf = index;
        int silent = 0;
        int state = find_leaf(c, expr->left, func_index, locals_in_scope, &leaf_type, &leaf_len, &leaf, &silent);
        uint64_t wlo = 0;
        uint64_t whi = 0;
        if (state == 1 && leaf_len == 0 && type_width(leaf_type) != 0) {
            word_range(c, expr->left, func_index, locals_in_scope, word_maximum(leaf_type), &wlo, &whi);
            return big_from_u64(&c->arena, wlo, lo) && big_from_u64(&c->arena, whi, hi);
        }
        if (state == 1 && leaf_len == 0 && leaf_type == TY_MOD && c->leaf_mod != 0 && c->leaf_mod < c->nmoduli) {
            Big one = big_zero();
            if (!big_from_u64(&c->arena, 0, lo) || !big_from_u64(&c->arena, 1, &one)) {
                return 0;
            }
            return big_sub(&c->arena, &c->moduli[c->leaf_mod], &one, hi);
        }
        return range_of(c, expr->left, func_index, locals_in_scope, lo, hi);
    }
    return 0;
}

static int big_below_u32(const Big *value, uint32_t limit) {
    if (value->negative && value->nlimbs != 0) {
        return 0;
    }
    if (value->nlimbs == 0) {
        return 1;
    }
    if (value->nlimbs > 1) {
        return 0;
    }
    return value->limbs[0] < limit;
}

static void report_index_range(Compiler *c, uint32_t start, uint32_t end, const Big *lo, const Big *hi, int have_range,
                               TypeKind element, uint32_t length) {
    char message[192];
    char label[128];
    char low_text[96];
    char high_text[96];
    char type_text[64];
    uint32_t highest = length == 0 ? 0 : length - 1u;
    int written = -1;
    write_type(type_text, sizeof type_text, element, length);
    snprintf(label, sizeof label, "indices run from 0 through %u", highest);
    if (have_range && lo != NULL && hi != NULL && big_format(lo, low_text, sizeof low_text) &&
        big_format(hi, high_text, sizeof high_text)) {
        if (big_cmp(lo, hi) == 0) {
            written = snprintf(message, sizeof message, "index %s is out of range for `%s`", low_text, type_text);
        } else {
            written = snprintf(message, sizeof message,
                               "this index runs from %s through %s, out of range for `%s`", low_text, high_text,
                               type_text);
        }
    }
    if (written < 0 || (size_t)written >= sizeof message) {
        snprintf(message, sizeof message, "this index is out of range for `%s`", type_text);
    }
    add_diag(c, "ORC0223", start, end, message, label,
             "every value an index can take, over every loop index and word in it, must select an element", 2);
}

static int check_index_expr(Compiler *c, uint32_t index_expr, TypeKind element, uint32_t length, uint32_t func_index,
                            uint32_t locals_in_scope) {
    TypeKind leaf_type = TY_NONE;
    uint32_t leaf_len = 0;
    uint32_t leaf = index_expr;
    int silent = 0;
    int state = find_leaf(c, index_expr, func_index, locals_in_scope, &leaf_type, &leaf_len, &leaf, &silent);
    uint32_t diags_before = c->ndiags;
    const Expr *expr = &c->exprs[index_expr];
    if (state == 1 && leaf_len == 0 && type_width(leaf_type) != 0) {
        uint64_t wlo = 0;
        uint64_t whi = 0;
        Big lo = big_zero();
        Big hi = big_zero();
        if (!check_expr(c, index_expr, leaf_type, 0, func_index, locals_in_scope)) {
            return 0;
        }
        /* A literal that does not fit the bit budget is already ORC0205. */
        if (c->ndiags != diags_before) {
            return 1;
        }
        word_range(c, index_expr, func_index, locals_in_scope, word_maximum(leaf_type), &wlo, &whi);
        if (whi >= length) {
            if (!big_from_u64(&c->arena, wlo, &lo) || !big_from_u64(&c->arena, whi, &hi)) {
                return 0;
            }
            report_index_range(c, expr->start, expr->end, &lo, &hi, 1, element, length);
        }
        return 1;
    }
    if (!check_expr(c, index_expr, TY_INT, 0, func_index, locals_in_scope)) {
        return 0;
    }
    /* A literal that does not fit the bit budget is already ORC0205. A second
       ORC0223 would treat that failed decode as an ordinary out-of-range index. */
    if (c->ndiags != diags_before) {
        return 1;
    }
    {
        uint32_t bad = index_expr;
        Big lo = big_zero();
        Big hi = big_zero();
        if (!static_index(c, index_expr, func_index, locals_in_scope, &bad)) {
            add_diag(c, "ORC0226", c->exprs[bad].start, c->exprs[bad].end,
                     "an `Int` index may use only integer literals, loop indices, and words converted with `as Int`",
                     "this `Int` has no bound",
                     "every index is proved in range when the program is checked: a word index ranges over its type, "
                     "and an `Int` index is built from integer literals, loop indices, and words converted with "
                     "`as Int`, using `+`, `-`, `*`, `/`, `%`, and conditionals",
                     2);
            return 1;
        }
        if (!range_of(c, index_expr, func_index, locals_in_scope, &lo, &hi)) {
            report_index_range(c, expr->start, expr->end, NULL, NULL, 0, element, length);
            return 1;
        }
        if ((lo.negative && lo.nlimbs != 0) || !big_below_u32(&hi, length)) {
            report_index_range(c, expr->start, expr->end, &lo, &hi, 1, element, length);
        }
    }
    return 1;
}

static int check_loop(Compiler *c, uint32_t index, TypeKind expected, uint32_t expected_len, uint32_t func_index,
                      uint32_t locals_in_scope) {
    Expr *expr = &c->exprs[index];
    LoopDesc *loop = &c->loops[expr->arg0];
    int a_above = 0;
    int b_above = 0;
    uint32_t a_value = 0;
    uint32_t b_value = 0;
    int index_dup;
    int acc_dup;
    int bounds_decoded;
    /* Magnitude is checked before the range, and the lower bound first, so
       70000..(a literal over 16384 bits) is ORC0205 on the upper bound rather than
       ORC0225 on 70000. */
    bounds_decoded = decode_loop_bound(c, loop->a_start, loop->a_end, &a_above, &a_value) &&
                     decode_loop_bound(c, loop->b_start, loop->b_end, &b_above, &b_value);
    if (bounds_decoded && a_above) {
        add_diag(c, "ORC0225", loop->a_start, loop->a_end, "a loop bound must be at most 65536", "loop bound",
                 "a loop runs over a nonempty range within 0 through 65536", 2);
    } else if (bounds_decoded && (b_above || a_value >= b_value)) {
        add_diag(c, "ORC0225", loop->b_start, loop->b_end, "a loop range must be nonempty and within 0 through 65536",
                 "loop bounds", "write a..b with 0 <= a < b <= 65536", 2);
    } else if (bounds_decoded) {
        loop->bounds_ok = 1;
        loop->bound_a = a_value;
        loop->bound_b = b_value;
    }
    index_dup = report_duplicate_loop_name(c, func_index, locals_in_scope, loop->index_start, loop->index_end);
    if (same_span(c, loop->index_start, loop->index_end, loop->acc_start, loop->acc_end)) {
        acc_dup = 1;
        add_diag(c, "ORC0219", loop->acc_start, loop->acc_end, "duplicate name", "this name is already in scope",
                 "the accumulator must differ from the loop index", 2);
    } else {
        acc_dup = report_duplicate_loop_name(c, func_index, locals_in_scope, loop->acc_start, loop->acc_end);
    }
    (void)index_dup;
    (void)acc_dup;
    if (!loop->acc_ok) {
        if (!loop->acc_reported) {
            reject_declared(c, loop->acc_type, loop->acc_length_bad, loop->type_start, loop->type_end,
                            loop->length_start, loop->length_end);
        }
        return 1;
    }
    if (loop->acc_type != expected || loop->acc_len != expected_len ||
        (loop->acc_type == TY_MOD && expected == TY_MOD && loop->acc_mod != c->expect_mod)) {
        char message[192];
        char expected_text[64];
        char found_text[64];
        write_type(expected_text, sizeof expected_text, expected, expected_len);
        write_type(found_text, sizeof found_text, loop->acc_type, loop->acc_len);
        snprintf(message, sizeof message, "expected %s, found %s", expected_text, found_text);
        add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                 "a loop has its accumulator's type", 2);
    }
    if (loop->init_expr != UINT32_MAX &&
        !check_at(c, loop->init_expr, loop->acc_type, loop->acc_len, loop->acc_mod, func_index, locals_in_scope)) {
        return 0;
    }
    if (loop->bounds_ok && loop->step_expr != UINT32_MAX) {
        if (c->nactive >= MAX_OPEN_LOOPS) {
            resource_diag(c, "ORC0209", expr->start, expr->end, "semantic analysis could not retain loop scopes");
            return 0;
        }
        c->active_loops[c->nactive++] = expr->arg0;
        if (!check_at(c, loop->step_expr, loop->acc_type, loop->acc_len, loop->acc_mod, func_index, locals_in_scope)) {
            c->nactive--;
            return 0;
        }
        c->nactive--;
    }
    return 1;
}

static int is_number_type(TypeKind type) {
    return type == TY_INT || type == TY_W8 || type == TY_W16 || type == TY_W32 || type == TY_W64;
}

static int is_scalar_type(TypeKind type) {
    return is_number_type(type) || type == TY_BOOL || type == TY_MOD;
}

static int check_at(Compiler *c, uint32_t index, TypeKind expected, uint32_t expected_len, uint16_t expected_mod,
                    uint32_t func_index, uint32_t locals_in_scope) {
    uint16_t saved = c->expect_mod;
    int ok;
    c->expect_mod = expected == TY_MOD ? expected_mod : 0;
    ok = check_expr(c, index, expected, expected_len, func_index, locals_in_scope);
    c->expect_mod = saved;
    return ok;
}

static const char *op_spelling(TokenKind op) {
    switch (op) {
    case TK_PLUS: return "+";
    case TK_MINUS: return "-";
    case TK_STAR: return "*";
    case TK_SLASH: return "/";
    case TK_PERCENT: return "%";
    case TK_AMP: return "&";
    case TK_PIPE: return "|";
    case TK_CARET: return "^";
    case TK_EQEQ: return "==";
    case TK_BANGEQ: return "!=";
    case TK_LESS: return "<";
    case TK_GREATER: return ">";
    case TK_LESSEQ: return "<=";
    case TK_GREATEREQ: return ">=";
    case TK_AMPAMP: return "&&";
    case TK_PIPEPIPE: return "||";
    default: return "operator";
    }
}

static int bool_word(const Compiler *c, uint32_t start, uint32_t end, int *value) {
    if (span_is(c, start, end, "true")) {
        *value = 1;
        return 1;
    }
    if (span_is(c, start, end, "false")) {
        *value = 0;
        return 1;
    }
    return 0;
}

static int check_compare(Compiler *c, uint32_t index, TypeKind expected, uint32_t expected_len, uint32_t func_index,
                         uint32_t locals_in_scope) {
    Expr *expr = &c->exprs[index];
    TypeKind operand = TY_NONE;
    uint32_t operand_len = 0;
    uint32_t leaf = index;
    int silent = 0;
    int state;
    int order = expr->op != TK_EQEQ && expr->op != TK_BANGEQ;
    if (expected != TY_BOOL || expected_len != 0) {
        char message[192];
        char expected_text[64];
        write_type(expected_text, sizeof expected_text, expected, expected_len);
        snprintf(message, sizeof message, "a comparison gives `Bool`, but %s is required here", expected_text);
        add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                 "a conditional `if c { a } else { b }` chooses a value by a `Bool`", 2);
    }
    state = find_leaf(c, expr->left, func_index, locals_in_scope, &operand, &operand_len, &leaf, &silent);
    if (state == 0) {
        state = find_leaf(c, expr->right, func_index, locals_in_scope, &operand, &operand_len, &leaf, &silent);
    }
    if (state == 0) {
        add_diag(c, "ORC0227", expr->op_start, expr->op_end, "the operands of a comparison have no type of their own",
                 "untyped comparison", "compare with a typed operand, such as a name, or give the literal a type with a `let` binding",
                 2);
        return 1;
    }
    if (state < 0) {
        if (!silent) {
            return check_expr(c, leaf, operand == TY_NONE ? TY_INT : operand, 0, func_index, locals_in_scope);
        }
        return 1;
    }
    if (order && operand == TY_MOD && operand_len == 0) {
        char message[160];
        snprintf(message, sizeof message, "`%s` is not defined for a residue", op_spelling(expr->op));
        add_diag(c, "ORC0215", expr->op_start, expr->op_end, message, "the operands have a residue type",
                 "residues are compared with `==` and `!=`; they have no order, so compare least residues, such as "
                 "`(x as Int) < (y as Int)`",
                 2);
        return 1;
    }
    if (operand_len != 0 || !is_scalar_type(operand) || (order && operand == TY_BOOL)) {
        char message[160];
        char found[64];
        write_type(found, sizeof found, operand, operand_len);
        snprintf(message, sizeof message, "`%s` is not defined for `%s`", op_spelling(expr->op), found);
        if (operand == TY_BOOL) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, message, "operator not defined",
                     "`Bool` values are compared with `==` and `!=`; they have no order", 2);
        } else {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, message, "operator not defined",
                     "compare elements, such as `x[0] == y[0]`", 2);
        }
    }
    if (operand == TY_MOD) {
        uint16_t mod = c->leaf_mod;
        if (!check_at(c, expr->left, operand, operand_len, mod, func_index, locals_in_scope)) {
            return 0;
        }
        return check_at(c, expr->right, operand, operand_len, mod, func_index, locals_in_scope);
    }
    if (!check_expr(c, expr->left, operand, operand_len, func_index, locals_in_scope)) {
        return 0;
    }
    return check_expr(c, expr->right, operand, operand_len, func_index, locals_in_scope);
}

/* True when `base` is itself an index of a scalar, so a longer chain such as
   `t[0][1][2]` reports ORC0224 once, on `t[0]`. */
static int scalar_index_base(Compiler *c, uint32_t base, uint32_t func_index, uint32_t locals_in_scope) {
    const Expr *expr = &c->exprs[base];
    TypeKind type = TY_NONE;
    uint32_t length = 0;
    int silent = 0;
    int state;
    if (expr->kind != EX_INDEX && expr->kind != EX_SELECT) {
        return 0;
    }
    state = index_subject(c, expr->left, func_index, locals_in_scope, &type, &length, &silent);
    return state == 1 && length == 0;
}

static int finish_scalar_index(Compiler *c, uint32_t base, TypeKind base_kind, uint32_t func_index,
                               uint32_t locals_in_scope) {
    if (!scalar_index_base(c, base, func_index, locals_in_scope)) {
        char message[192];
        snprintf(message, sizeof message, "only an array can be indexed, but this has type %s",
                 type_spelling(base_kind));
        add_diag(c, "ORC0224", c->exprs[base].start, c->exprs[base].end, message, "not an array",
                 "an index selects one element of a value of type T^n", 2);
    }
    return check_expr(c, base, base_kind, 0, func_index, locals_in_scope);
}

static int check_expr(Compiler *c, uint32_t index, TypeKind expected, uint32_t expected_len, uint32_t func_index,
                      uint32_t locals_in_scope) {
    Expr *expr;
    if (c->resource || c->sema_limited) {
        return 0;
    }
    expr = &c->exprs[index];
    expr->ty = expected;
    expr->ty_len = expected_len;
    if (expected == TY_MOD) {
        expr->ty_mod = c->expect_mod;
    }
    switch (expr->kind) {
    case EX_GROUP:
        return check_expr(c, expr->left, expected, expected_len, func_index, locals_in_scope);
    case EX_LIT:
        if (expected_len != 0) {
            char message[192];
            char expected_text[64];
            write_type(expected_text, sizeof expected_text, expected, expected_len);
            snprintf(message, sizeof message, "an integer literal cannot have type %s", expected_text);
            add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                     "an array value is written as an array literal", 2);
            return 1;
        }
        check_literal(c, expr, expected, c->expect_mod);
        return 1;
    case EX_ARRAY:
        if (expected_len == 0) {
            char message[192];
            snprintf(message, sizeof message, "an array literal cannot have type %s", type_spelling(expected));
            add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                     "an array literal is written where an array type is required", 2);
            return 1;
        }
        if (expr->argc != expected_len) {
            char message[192];
            char expected_text[64];
            write_type(expected_text, sizeof expected_text, expected, expected_len);
            snprintf(message, sizeof message, "this array has %u elements, but %s has %u", expr->argc, expected_text,
                     expected_len);
            add_diag(c, "ORC0222", expr->start, expr->end, message, "array length mismatch",
                     "an array literal lists every element of its type", 2);
        }
        for (uint16_t element = 0; element < expr->argc; element++) {
            if (!check_expr(c, c->args[expr->arg0 + element], expected, 0, func_index, locals_in_scope)) {
                return 0;
            }
        }
        return 1;
    case EX_INDEX: {
        TypeKind base_kind = TY_NONE;
        uint32_t base_len = 0;
        int silent = 0;
        int state = index_subject(c, expr->left, func_index, locals_in_scope, &base_kind, &base_len, &silent);
        if (state != 1) {
            if (silent) {
                return 1;
            }
            return check_expr(c, expr->left, TY_INT, 0, func_index, locals_in_scope);
        }
        if (base_len == 0) {
            return finish_scalar_index(c, expr->left, base_kind, func_index, locals_in_scope);
        }
        {
            Big magnitude = big_zero();
            if (!decode_literal(c, expr, &magnitude)) {
                add_diag(c, "ORC0205", expr->lit_start, expr->lit_end,
                         "integer magnitude exceeds 16384 significant bits", "index is too large",
                         "an index literal must fit the representation budget", 2);
            } else if (!index_below(&magnitude, base_len)) {
                add_diag(c, "ORC0223", expr->lit_start, expr->lit_end, "index is not below the array length",
                         "index out of range", "a literal index must be less than the array's length", 2);
            }
        }
        {
            uint16_t element_mod = c->leaf_mod;
            if (base_kind != expected || expected_len != 0 ||
                (base_kind == TY_MOD && expected == TY_MOD && element_mod != c->expect_mod)) {
                char message[192];
                char expected_text[64];
                char found_text[64];
                write_type(expected_text, sizeof expected_text, expected, expected_len);
                write_type(found_text, sizeof found_text, base_kind, 0);
                snprintf(message, sizeof message, "this element has type %s, but %s is required here", found_text,
                         expected_text);
                add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                         "Orange does not convert between types implicitly", 2);
            }
            return check_at(c, expr->left, base_kind, base_len, element_mod, func_index, locals_in_scope);
        }
    }
    case EX_NAME: {
        NameRes res;
        uint16_t slot = 0;
        TypeKind type = TY_NONE;
        uint32_t length = 0;
        int type_ok = 0;
        resolve_name(c, func_index, locals_in_scope, expr->name_start, expr->name_end, &res, &slot, &type, &length,
                     &type_ok);
        expr->name_res = res;
        expr->name_index = slot;
        expr->name_ty = type;
        expr->name_len = length;
        if (res == NAME_BAD) {
            return 1;
        }
        if (res != NAME_PARAM && res != NAME_LOCAL) {
            int bool_value = 0;
            if (res != NAME_BAD && bool_word(c, expr->name_start, expr->name_end, &bool_value)) {
                expr->name_res = NAME_BOOL;
                expr->name_index = (uint16_t)bool_value;
                expr->name_ty = TY_BOOL;
                expr->name_len = 0;
                if (expected != TY_BOOL || expected_len != 0) {
                    char message[192];
                    char expected_text[64];
                    write_type(expected_text, sizeof expected_text, expected, expected_len);
                    snprintf(message, sizeof message, "expected %s, found Bool", expected_text);
                    add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                             "Orange does not convert between types implicitly", 2);
                }
                return 1;
            }
            report_unknown_name(c, expr, func_index, res);
            return 1;
        }
        {
            uint16_t found_mod = 0;
            if (type == TY_MOD) {
                found_mod = res == NAME_PARAM ? c->params[c->funcs[func_index].param0 + slot].mod_index
                                              : c->locals[c->funcs[func_index].local0 + slot].mod_index;
            }
            if (type != expected || length != expected_len ||
                (type == TY_MOD && expected == TY_MOD && found_mod != c->expect_mod)) {
                char message[192];
                char expected_text[64];
                char found_text[64];
                write_type(expected_text, sizeof expected_text, expected, expected_len);
                write_type(found_text, sizeof found_text, type, length);
                snprintf(message, sizeof message, "expected %s, found %s", expected_text, found_text);
                add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                         "Orange does not convert between types implicitly", 2);
            }
        }
        return 1;
    }
    case EX_CALL: {
        Callee callee;
        char ident[64];
        char module_name[64];
        Compiler *target;
        resolve_callee(c, expr, &callee);
        span_copy(ident, sizeof ident, c->text, expr->name_start, expr->name_end);
        if (callee.qualified) {
            span_copy(module_name, sizeof module_name, c->text, expr->left, expr->right);
        } else {
            module_name[0] = '\0';
        }
        if (callee.not_used) {
            char message[192];
            if (callee.is_self) {
                snprintf(message, sizeof message, "`%s` is the calling module", module_name);
                add_diag(c, "ORC0229", expr->left, expr->right, message, "a module does not qualify calls to itself",
                         "call a function of the same module without a module name, as in `f(x)`", 2);
            } else {
                char own[64];
                char note[192];
                span_copy(own, sizeof own, c->text, c->module_start, c->module_end);
                snprintf(message, sizeof message, "module `%s` is not used by `%s`", module_name, own);
                snprintf(note, sizeof note, "declare `use %s;` at the head of the module to call its functions",
                         module_name);
                add_diag(c, "ORC0229", expr->left, expr->right, message, "no `use` declaration names this module", note,
                         2);
            }
            return 1;
        }
        if (!callee.found) {
            char message[192];
            char note[320];
            const char *note_text;
            if (callee.is_impl) {
                note_text = "`impl` functions have no semantics yet and cannot be called";
            } else if (callee.qualified) {
                note_text = "a qualified call names a typed `spec` of the used module";
            } else if (c->nuses == 0) {
                note_text = "calls name a typed `spec` declared in the same module";
            } else if (!callee.empty_spec) {
                uint16_t use_index;
                int imported = 0;
                note[0] = '\0';
                for (use_index = 0; use_index < c->nuses; use_index++) {
                    uint16_t target_index;
                    Compiler *used;
                    char used_name[64];
                    if (c->program == NULL) {
                        break;
                    }
                    target_index = c->program->use_target[c->self_index][use_index];
                    if (target_index == UINT16_MAX || target_index >= c->program->nmods) {
                        continue;
                    }
                    used = c->program->mods[target_index];
                    if (!module_declares_spec(used, c->text, expr->name_start, expr->name_end)) {
                        continue;
                    }
                    span_copy(used_name, sizeof used_name, used->text, used->module_start, used->module_end);
                    snprintf(note, sizeof note, "the used module `%s` declares `%s`; call it as `%s::%s(...)`",
                             used_name, ident, used_name, ident);
                    imported = 1;
                    break;
                }
                note_text = imported ? note
                                     : "calls name a typed `spec` declared in the same module, or one of a used "
                                       "module as `NAME::f(...)`";
            } else {
                note_text = "calls name a typed `spec` declared in the same module, or one of a used "
                            "module as `NAME::f(...)`";
            }
            if (callee.empty_spec) {
                snprintf(message, sizeof message, "`spec` function `%s` has no typed body and cannot be called", ident);
                add_diag(c, "ORC0212", expr->name_start, expr->name_end, message, "no value to call", note_text, 2);
            } else if (callee.qualified) {
                snprintf(message, sizeof message, "no typed `spec` function named `%s` in module `%s`", ident,
                         module_name);
                add_diag(c, "ORC0212", expr->name_start, expr->name_end, message, "unknown function", note_text, 2);
            } else {
                snprintf(message, sizeof message, "no typed `spec` function named `%s` in this module", ident);
                add_diag(c, "ORC0212", expr->name_start, expr->name_end, message, "unknown function", note_text, 2);
            }
            return 1;
        }
        target = callee.mod;
        expr->callee = callee.func;
        if (callee.qualified) {
            expr->name_index = callee.mod_index;
        } else if (!record_edge(c, func_index, callee.func, expr->start, expr->end)) {
            return 0;
        }
        if (!signature_is_usable(target, callee.func)) {
            return 1;
        }
        if (expr->argc != target->funcs[callee.func].nparams) {
            char message[192];
            snprintf(message, sizeof message, "`%s` takes %u argument%s but %u %s supplied", ident,
                     target->funcs[callee.func].nparams, target->funcs[callee.func].nparams == 1 ? "" : "s", expr->argc,
                     expr->argc == 1 ? "was" : "were");
            add_diag(c, "ORC0213", expr->start, expr->end, message, "wrong number of arguments",
                     "every parameter receives exactly one argument", 2);
            return 1;
        }
        if (target->funcs[callee.func].result != expected || target->funcs[callee.func].result_len != expected_len ||
            (target->funcs[callee.func].result == TY_MOD && expected == TY_MOD &&
             target->funcs[callee.func].result_mod != c->expect_mod)) {
            char message[192];
            char expected_text[64];
            char found_text[64];
            write_type(expected_text, sizeof expected_text, expected, expected_len);
            write_type(found_text, sizeof found_text, target->funcs[callee.func].result,
                       target->funcs[callee.func].result_len);
            snprintf(message, sizeof message, "expected %s, found %s", expected_text, found_text);
            add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                     "Orange does not convert between types implicitly", 2);
        }
        for (uint16_t arg = 0; arg < expr->argc; arg++) {
            Param *param = &target->params[target->funcs[callee.func].param0 + arg];
            if (!param->type_ok) {
                continue;
            }
            if (!check_at(c, c->args[expr->arg0 + arg], param->type, param->length, param->mod_index, func_index,
                          locals_in_scope)) {
                return 0;
            }
        }
        return 1;
    }
    case EX_UNARY:
        if (expected_len != 0) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, "this operator is not defined for an array",
                     "operators apply to elements", "index the array and apply the operator to one element", 2);
            return 1;
        }
        if (expr->op == TK_BANG) {
            if (expected != TY_BOOL) {
                char message[128];
                snprintf(message, sizeof message, "prefix `!` is not defined for `%s`", type_spelling(expected));
                add_diag(c, "ORC0215", expr->op_start, expr->op_end, message, "operator not defined",
                         "`!` negates a `Bool`; `~` is the bitwise complement of a word", 2);
            }
            return check_expr(c, expr->left, expected == TY_BOOL ? TY_BOOL : expected, 0, func_index, locals_in_scope);
        }
        if (expr->op == TK_MINUS && expected == TY_MOD) {
            return check_expr(c, expr->left, TY_MOD, 0, func_index, locals_in_scope);
        }
        if (expr->op == TK_TILDE && expected == TY_MOD) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, "prefix `~` is not defined for a residue",
                     "bitwise complement needs a word", "bitwise operators apply only to `Word[n]` values", 2);
            return 1;
        }
        if (expr->op == TK_MINUS && expected != TY_INT) {
            if (expected == TY_BOOL) {
                add_diag(c, "ORC0215", expr->op_start, expr->op_end, "prefix `-` is not defined for `Bool`",
                         "operator not defined", "`Bool` has `!`, `&&`, `||`, `==`, and `!=`", 2);
            } else {
                add_diag(c, "ORC0215", expr->op_start, expr->op_end, "negation is not defined for this type",
                         "write `0 - a` for a word", "prefix `-` is exact integer negation", 2);
            }
            return 1;
        }
        if (expr->op == TK_TILDE && (expected == TY_INT || expected == TY_BOOL)) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end,
                     expected == TY_BOOL ? "prefix `~` is not defined for `Bool`" : "complement is not defined for `Int`",
                     "bitwise complement needs a word",
                     expected == TY_BOOL ? "`Bool` has `!`, `&&`, `||`, `==`, and `!=`" : NULL, 2);
            return 1;
        }
        return check_expr(c, expr->left, expected, 0, func_index, locals_in_scope);
    case EX_BINARY:
        if (is_compare_op(expr->op)) {
            return check_compare(c, index, expected, expected_len, func_index, locals_in_scope);
        }
        if (expr->op == TK_AMPAMP || expr->op == TK_PIPEPIPE) {
            if (expected != TY_BOOL || expected_len != 0) {
                char message[128];
                snprintf(message, sizeof message, "`%s` is not defined for `%s`", op_spelling(expr->op),
                         type_spelling(expected));
                add_diag(c, "ORC0215", expr->op_start, expr->op_end, message, "operator not defined",
                         "`&&` and `||` apply to `Bool` values; `&` and `|` are the bitwise operators on words", 2);
            }
            if (!check_expr(c, expr->left, expected_len == 0 && expected == TY_BOOL ? TY_BOOL : expected, expected_len,
                            func_index, locals_in_scope)) {
                return 0;
            }
            return check_expr(c, expr->right, expected_len == 0 && expected == TY_BOOL ? TY_BOOL : expected,
                              expected_len, func_index, locals_in_scope);
        }
        if (expected_len != 0) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, "this operator is not defined for an array",
                     "operators apply to elements", "index the array and apply the operator to one element", 2);
            return 1;
        }
        if (expected == TY_BOOL) {
            char message[128];
            snprintf(message, sizeof message, "`%s` is not defined for `Bool`", op_spelling(expr->op));
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, message, "operator not defined",
                     "`Bool` has `!`, `&&`, `||`, `==`, and `!=`", 2);
            return 1;
        }
        if ((expr->op == TK_AMP || expr->op == TK_PIPE || expr->op == TK_CARET) && expected == TY_INT) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, "bitwise operators are not defined for `Int`",
                     "this operator needs a word", NULL, 2);
            return 1;
        }
        if (expected == TY_MOD) {
            if (expr->op != TK_PLUS && expr->op != TK_MINUS && expr->op != TK_STAR && expr->op != TK_SLASH) {
                const char *note = expr->op == TK_PERCENT
                                       ? "a residue is already reduced; `%` applies to `Int` and word values, such as "
                                         "`(x as Int) % 16`"
                                       : "bitwise operators apply only to `Word[n]` values";
                char message[128];
                snprintf(message, sizeof message, "`%s` is not defined for a residue", op_spelling(expr->op));
                add_diag(c, "ORC0215", expr->op_start, expr->op_end, message, "operator not defined", note, 2);
                return 1;
            }
            if (!check_expr(c, expr->left, expected, 0, func_index, locals_in_scope)) {
                return 0;
            }
            return check_expr(c, expr->right, expected, 0, func_index, locals_in_scope);
        }
        if (!is_number_type(expected) && expected != TY_NONE) {
            char message[128];
            snprintf(message, sizeof message, "`%s` is not defined for `%s`", op_spelling(expr->op),
                     type_spelling(expected));
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, message, "operator not defined", NULL, 2);
            return 1;
        }
        if (!check_expr(c, expr->left, expected, 0, func_index, locals_in_scope)) {
            return 0;
        }
        return check_expr(c, expr->right, expected, 0, func_index, locals_in_scope);
    case EX_SHIFT:
        if (expected_len != 0) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, "this operator is not defined for an array",
                     "operators apply to elements", "index the array and apply the operator to one element", 2);
            return 1;
        }
        if (expected == TY_BOOL) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, "shifts and rotations are not defined for `Bool`",
                     "operator not defined", "`Bool` has `!`, `&&`, `||`, `==`, and `!=`", 2);
            return 1;
        }
        if (expected == TY_INT) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, "shifts and rotations are not defined for `Int`",
                     "this operator needs a word", NULL, 2);
            return 1;
        }
        if (expected == TY_MOD) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, "shifts and rotations are not defined for a residue",
                     "this operator needs a word", "shifts and rotations apply only to `Word[n]` values", 2);
            return 1;
        }
        if (!check_expr(c, expr->left, expected, 0, func_index, locals_in_scope)) {
            return 0;
        }
        {
            const Expr *amount = &c->exprs[expr->right];
            Big value = big_zero();
            int width = type_width(expected);
            if (amount->kind != EX_LIT || amount->negative) {
                add_diag(c, "ORC0216", amount->start, amount->end,
                         "shift amount must be an unsigned literal below the width", "invalid shift amount",
                         "a literal amount is an integer from 0 through n - 1", 2);
                return 1;
            }
            if (!decode_literal(c, amount, &value)) {
                add_diag(c, "ORC0205", amount->start, amount->end, "integer magnitude exceeds 16384 significant bits",
                         "literal is too large", NULL, 2);
                return 1;
            }
            if (big_bits(&value) > 31 || (value.nlimbs > 0 && value.limbs[0] >= (uint32_t)width) ||
                (width == 0)) {
                add_diag(c, "ORC0216", amount->start, amount->end,
                         "shift amount must be an unsigned literal below the width", "invalid shift amount",
                         "a literal amount is an integer from 0 through n - 1", 2);
            }
        }
        return 1;
    case EX_CONV: {
        TypeKind leaf_type = TY_NONE;
        uint32_t leaf_len = 0;
        uint32_t leaf = index;
        int silent = 0;
        int state;
        /* A rejected target is not a type, so it is not also a mismatch with
           `expected`. The operand is still checked: Rust reports both the
           target and whatever the operand itself has wrong. */
        if (!expr->conv_ok) {
            int already = expr->conv_site < c->nsites && c->sites[expr->conv_site].reported;
            if (!already) {
                reject_type(c, expr->conv_ty, 0, expr->name_start, expr->name_end);
            }
        } else if (expr->conv_len != 0) {
            add_diag(c, "ORC0215", expr->name_start, expr->name_end, "`as` does not convert to an array type",
                     "`as` gives one `Int`, word, or residue value",
                     "convert each element, such as `x[0] as Int`", 2);
            return 1;
        } else if (expr->conv_ty != expected || expected_len != 0 ||
                   (expr->conv_ty == TY_MOD && expected == TY_MOD && expr->conv_mod != c->expect_mod)) {
            char message[192];
            char expected_text[64];
            char found_text[64];
            write_type(expected_text, sizeof expected_text, expected, expected_len);
            write_type(found_text, sizeof found_text, expr->conv_ty, 0);
            snprintf(message, sizeof message, "expected %s, found %s", expected_text, found_text);
            add_diag(c, "ORC0214", expr->name_start, expr->name_end, message, "conversion has a different type",
                     "the target of `as` is the type of the conversion", 2);
        }
        state = find_leaf(c, expr->left, func_index, locals_in_scope, &leaf_type, &leaf_len, &leaf, &silent);
        if (state == 2 || (state > 0 && leaf_len != 0)) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, "`as` is not defined for an array",
                     "convert one element", "a conversion applies to one Int or word value", 2);
            return 1;
        }
        if (state == 0) {
            add_diag(c, "ORC0220", c->exprs[expr->left].start, c->exprs[expr->left].end,
                     "conversion operand has no type of its own", "no typed leaf",
                     "a literal takes its type from context; name it or convert a typed value", 2);
            return 1;
        }
        if (state < 0) {
            if (!silent) {
                return check_expr(c, leaf, leaf_type == TY_NONE ? TY_INT : leaf_type, 0, func_index, locals_in_scope);
            }
            return 1;
        }
        if (leaf_type == TY_BOOL || expr->conv_ty == TY_BOOL) {
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, "`as` does not convert to or from `Bool`",
                     "`as` converts one `Int`, word, or residue value",
                     "choose a number with a conditional, such as `if b { 1 } else { 0 }`, or compare a number, such as `x != 0`",
                     2);
            return 1;
        }
        if (leaf_type == TY_MOD) {
            return check_at(c, expr->left, leaf_type, 0, c->leaf_mod, func_index, locals_in_scope);
        }
        return check_expr(c, expr->left, leaf_type, 0, func_index, locals_in_scope);
    }
    case EX_COND: {
        uint32_t arg0 = expr->arg0;
        uint16_t arms = expr->argc;
        uint32_t otherwise = expr->right;
        uint16_t arm;
        for (arm = 0; arm < arms; arm++) {
            uint32_t condition = c->cond_arms[arg0 + arm].cond;
            uint32_t value = c->cond_arms[arg0 + arm].value;
            if (!check_expr(c, condition, TY_BOOL, 0, func_index, locals_in_scope) ||
                !check_expr(c, value, expected, expected_len, func_index, locals_in_scope)) {
                return 0;
            }
        }
        return check_expr(c, otherwise, expected, expected_len, func_index, locals_in_scope);
    }
    case EX_LOOP:
        return check_loop(c, index, expected, expected_len, func_index, locals_in_scope);
    case EX_LOOP_INDEX:
        if (expected != TY_INT || expected_len != 0) {
            char message[192];
            char expected_text[64];
            write_type(expected_text, sizeof expected_text, expected, expected_len);
            snprintf(message, sizeof message, "expected %s, found Int", expected_text);
            add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                     "a loop index has type Int", 2);
        }
        return 1;
    case EX_ACCUM: {
        const LoopDesc *loop = &c->loops[expr->arg0];
        if (!loop->acc_ok) {
            return 1;
        }
        if (loop->acc_type != expected || loop->acc_len != expected_len ||
        (loop->acc_type == TY_MOD && expected == TY_MOD && loop->acc_mod != c->expect_mod)) {
            char message[192];
            char expected_text[64];
            char found_text[64];
            write_type(expected_text, sizeof expected_text, expected, expected_len);
            write_type(found_text, sizeof found_text, loop->acc_type, loop->acc_len);
            snprintf(message, sizeof message, "expected %s, found %s", expected_text, found_text);
            add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                     "an accumulator has its declared type", 2);
        }
        return 1;
    }
    case EX_SELECT: {
        TypeKind base_kind = TY_NONE;
        uint32_t base_len = 0;
        int silent = 0;
        int state = index_subject(c, expr->left, func_index, locals_in_scope, &base_kind, &base_len, &silent);
        if (state != 1) {
            if (silent) {
                return 1;
            }
            return check_expr(c, expr->left, TY_INT, 0, func_index, locals_in_scope);
        }
        if (base_len == 0) {
            return finish_scalar_index(c, expr->left, base_kind, func_index, locals_in_scope);
        }
        {
            uint16_t element_mod = c->leaf_mod;
            if (base_kind != expected || expected_len != 0 ||
                (base_kind == TY_MOD && expected == TY_MOD && element_mod != c->expect_mod)) {
                char message[192];
                char expected_text[64];
                char found_text[64];
                write_type(expected_text, sizeof expected_text, expected, expected_len);
                write_type(found_text, sizeof found_text, base_kind, 0);
                snprintf(message, sizeof message, "this element has type %s, but %s is required here", found_text,
                         expected_text);
                add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                         "Orange does not convert between types implicitly", 2);
            }
            if (!check_at(c, expr->left, base_kind, base_len, element_mod, func_index, locals_in_scope)) {
                return 0;
            }
        }
        return check_index_expr(c, expr->right, base_kind, base_len, func_index, locals_in_scope);
    }
    case EX_UPDATE: {
        TypeKind leaf_type = TY_NONE;
        uint32_t leaf_len = 0;
        uint32_t leaf = index;
        int silent = 0;
        int state = find_leaf(c, expr->left, func_index, locals_in_scope, &leaf_type, &leaf_len, &leaf, &silent);
        if (state == 1 && leaf_len == 0) {
            char message[192];
            snprintf(message, sizeof message, "only an array can be updated, but this has type %s",
                     type_spelling(leaf_type));
            add_diag(c, "ORC0224", c->exprs[expr->left].start, c->exprs[expr->left].end, message, "not an array",
                     "an update replaces one element of an array", 2);
            if (!silent) {
                return check_expr(c, expr->left, leaf_type, 0, func_index, locals_in_scope);
            }
            return 1;
        }
        if (expected_len == 0) {
            char message[192];
            snprintf(message, sizeof message, "an update cannot have type %s", type_spelling(expected));
            add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                     "an update has the type of the array it updates", 2);
            return 1;
        }
        if (!check_expr(c, expr->left, expected, expected_len, func_index, locals_in_scope) ||
            !check_index_expr(c, expr->right, expected, expected_len, func_index, locals_in_scope)) {
            return 0;
        }
        return check_expr(c, expr->callee, expected, 0, func_index, locals_in_scope);
    }
    case EX_FILL: {
        uint32_t length = 0;
        int admitted;
        if (expected_len == 0) {
            char message[192];
            snprintf(message, sizeof message, "a fill literal cannot have type %s", type_spelling(expected));
            add_diag(c, "ORC0214", expr->start, expr->end, message, "type mismatch",
                     "a fill literal is written where an array type is required", 2);
            return 1;
        }
        admitted = canonical_array_length(c->text, expr->lit_start, expr->lit_end, &length);
        if (!admitted) {
            char message[128];
            snprintf(message, sizeof message, "array length must be a decimal integer from 1 through %u",
                     MAX_ARRAY_LENGTH);
            add_diag(c, "ORC0221", expr->lit_start, expr->lit_end, message, "unsupported fill length",
                     "write the length in decimal without a prefix, separator, or leading zero", 2);
        } else if (length != expected_len) {
            char message[192];
            char expected_text[64];
            write_type(expected_text, sizeof expected_text, expected, expected_len);
            snprintf(message, sizeof message, "this fill has length %u, but %s has %u", length, expected_text,
                     expected_len);
            add_diag(c, "ORC0222", expr->start, expr->end, message, "array length mismatch",
                     "a fill literal states its type's length", 2);
        }
        return check_expr(c, expr->left, expected, 0, func_index, locals_in_scope);
    }
    default:
        return 1;
    }
}

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

static void bind_modulus(Compiler *c, TypeSite *site) {
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
    uint16_t arg;
    if (index == UINT32_MAX || c->resource) {
        return;
    }
    expr = &c->exprs[index];
    switch (expr->kind) {
    case EX_GROUP:
    case EX_UNARY:
    case EX_FILL:
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
    case EX_CALL:
    case EX_ARRAY:
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
        walk_moduli(c, c->loops[expr->arg0].step_expr);
        break;
    case EX_COND:
        for (arg = 0; arg < expr->argc; arg++) {
            walk_moduli(c, c->cond_arms[expr->arg0 + arg].cond);
            walk_moduli(c, c->cond_arms[expr->arg0 + arg].value);
        }
        walk_moduli(c, expr->right);
        break;
    default:
        break;
    }
}

static int builtin_type_name(const Compiler *c, uint32_t start, uint32_t end) {
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

static void copy_ident(char *dest, size_t cap, const Compiler *c, uint32_t start, uint32_t end) {
    span_copy(dest, cap, c->text, start, end);
}

static void resolve_site(Compiler *c, TypeSite *site, int from_decl, uint32_t earlier_limit) {
    uint32_t found = 0;
    TypeSite *target;
    char name[64];
    if (site->resolved || c->resource) {
        return;
    }
    site->resolved = 1;
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
        }
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
    if (target->rank >= 2 && site->wrote_axis) {
        char message[160];
        copy_ident(name, sizeof name, c, site->ident_start, site->ident_end);
        snprintf(message, sizeof message, "`%s` already has two array dimensions", name);
        add_diag(c, "ORC0203", site->start, site->end, message, "arrays have at most two dimensions",
                 "a row holds scalars; a matrix holds rows of the same type", 2);
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

static void prepare_types(Compiler *c) {
    uint32_t index;
    uint16_t param;
    uint16_t local;
    if (c->resource || c->parse_diags > 0) {
        return;
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
        } else {
            decl->installed = 1;
        }
        resolve_site(c, &c->sites[decl->site], 1, index);
    }
    for (index = 0; index < c->nsites; index++) {
        resolve_site(c, &c->sites[index], 0, c->ntypes);
    }
    for (index = 0; index < c->nfuncs; index++) {
        Func *func = &c->funcs[index];
        for (param = 0; param < func->nparams; param++) {
            Param *item = &c->params[func->param0 + param];
            publish_site(c, item->site, &item->type, &item->length, &item->type_ok, &item->mod_index, &item->type_reported);
        }
        publish_site(c, func->result_site, &func->result, &func->result_len, &func->result_ok, &func->result_mod,
                     &func->result_reported);
        for (local = 0; local < func->nlocals; local++) {
            Local *item = &c->locals[func->local0 + local];
            publish_site(c, item->site, &item->type, &item->length, &item->type_ok, &item->mod_index, &item->type_reported);
        }
    }
    for (index = 0; index < c->nloops; index++) {
        LoopDesc *loop = &c->loops[index];
        publish_site(c, loop->site, &loop->acc_type, &loop->acc_len, &loop->acc_ok, &loop->acc_mod, &loop->acc_reported);
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

static void analyze(Compiler *c) {
    uint32_t index;
    prepare_types(c);
    for (index = 0; index < c->nfuncs; index++) {
        Func *func = &c->funcs[index];
        uint16_t param_index;
        uint16_t local_index;
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
                add_diag(c, "ORC0201", func->name_start, func->name_end, message, "this name is already declared",
                         "spec and impl names are separate, and each kind is unique", 2);
                func->duplicate = 1;
                break;
            }
        }
        if (!func->typed) {
            continue;
        }
        func->signature_ok = func->result_ok;
        for (param_index = 0; param_index < func->nparams; param_index++) {
            Param *param = &c->params[func->param0 + param_index];
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
                    param->duplicate = 1;
                    break;
                }
            }
            if (!param->type_ok && !param->type_reported) {
                reject_declared(c, param->type, param->length_bad, param->type_start, param->type_end,
                                param->length_start, param->length_end);
                func->signature_ok = 0;
            }
        }
        if (!func->result_ok && !func->result_reported) {
            reject_declared(c, func->result, func->result_length_bad, func->result_start, func->result_end,
                            func->result_length_start, func->result_length_end);
            func->signature_ok = 0;
            continue;
        }
        for (local_index = 0; local_index < func->nlocals; local_index++) {
            Local *local = &c->locals[func->local0 + local_index];
            int hidden = 0;
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
            if (hidden) {
                char message[128];
                char name[64];
                size_t length = local->name_end - local->name_start;
                if (length >= sizeof name) {
                    length = sizeof name - 1;
                }
                memcpy(name, c->text + local->name_start, length);
                name[length] = '\0';
                snprintf(message, sizeof message, "duplicate binding `%s`", name);
                add_diag(c, "ORC0219", local->name_at, local->name_end_at, message, "this name is already in scope",
                         "parameters and bindings share one set of names", 2);
                local->duplicate = 1;
            }
            if (!local->type_ok && !local->type_reported) {
                reject_declared(c, local->type, local->length_bad, local->type_start, local->type_end,
                                local->length_start, local->length_end);
                continue;
            }
            check_at(c, local->value, local->type, local->length, local->mod_index, index, local_index);
        }
        if (func->body != UINT32_MAX) {
            check_at(c, func->body, func->result, func->result_len, func->result_mod, index, func->nlocals);
        }
    }
    {
        uint8_t *color = calloc(c->nfuncs ? c->nfuncs : 1, 1);
        uint32_t *stack = calloc(c->nfuncs ? c->nfuncs : 1, sizeof(uint32_t));
        if (color == NULL || stack == NULL) {
            resource_diag(c, "ORC0209", 0, 0, "semantic analysis could not retain the call graph");
            free(color);
            free(stack);
            return;
        }
        for (index = 0; index < c->nfuncs; index++) {
            uint32_t top = 0;
            uint32_t *edge_at = calloc(c->nfuncs ? c->nfuncs : 1, sizeof(uint32_t));
            if (edge_at == NULL) {
                free(color);
                free(stack);
                resource_diag(c, "ORC0209", 0, 0, "semantic analysis could not retain the call graph");
                return;
            }
            if (!c->funcs[index].typed || color[index] != 0) {
                free(edge_at);
                continue;
            }
            stack[top] = index;
            color[index] = 1;
            edge_at[index] = 0;
            while (top != UINT32_MAX) {
                uint32_t current = stack[top];
                Func *func = &c->funcs[current];
                if (edge_at[current] < func->nedges) {
                    Edge edge = c->edges[func->edge0 + edge_at[current]];
                    edge_at[current]++;
                    if (color[edge.callee] == 1) {
                        add_diag(c, "ORC0217", edge.start, edge.end, "call cycle", "this call closes a cycle",
                                 "specifications are acyclic, so every accepted program terminates", 2);
                    } else if (color[edge.callee] == 0) {
                        color[edge.callee] = 1;
                        edge_at[edge.callee] = 0;
                        stack[++top] = edge.callee;
                    }
                } else {
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
    uint32_t length;
    uint32_t index;
    if (value == NULL) {
        return;
    }
    elems = value->elems;
    length = value->length;
    memset(value, 0, sizeof *value);
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
    if (src->length == 0) {
        *dst = *src;
        dst->elems = NULL;
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

static int charge(Compiler *c, uint32_t start, uint32_t end, uint64_t cost) {
    if (c->failed) {
        return 0;
    }
    if (cost > MAX_STEPS || c->steps > MAX_STEPS - cost) {
        c->failed = 1;
        add_diag(c, "ORC0301", start, end, "evaluation exceeded the step budget", "step limit reached",
                 "one source shares 1048576 reference-evaluation steps", 2);
        return 0;
    }
    c->steps += cost;
    return 1;
}

static int eval_expr(Compiler *c, uint32_t index, Value *params, Value *locals, int depth, Value *out);

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
        ok = eval_expr(c, c->locals[func->local0 + index].value, params, locals, depth, &locals[index]);
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
        if (!charge(c, expr->start, expr->end, 1)) {
            return 0;
        }
        if (expr->name_res == NAME_BOOL) {
            out->type = TY_BOOL;
            out->word = expr->name_index;
            out->length = 0;
            out->big = big_zero();
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
            /* One step budget and one failure flag for the whole program.
               Copy them across the call so a nested module spends the same
               counter, then copy the result back. */
            target->steps = c->steps;
            target->failed = c->failed;
            ok = eval_function(target, expr->callee, arguments, depth + 1, out);
            c->steps = target->steps;
            if (target->failed) {
                c->failed = 1;
            }
        } else {
            if (expr->callee >= c->nfuncs) {
                c->failed = 1;
                value_list_clear(arguments, count);
                return 0;
            }
            ok = eval_function(c, expr->callee, arguments, depth + 1, out);
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
                    add_diag(c, "ORC0301", expr->op_start, expr->op_end,
                             "integer result exceeds 16384 significant bits", "magnitude limit reached", NULL, 2);
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
                add_diag(c, "ORC0301", expr->op_start, expr->op_end, "integer result exceeds 16384 significant bits",
                         "magnitude limit reached", NULL, 2);
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
                add_diag(c, "ORC0301", expr->op_start, expr->op_end, "integer result exceeds 16384 significant bits",
                         "magnitude limit reached", NULL, 2);
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
        Value *items = NULL;
        uint32_t element;
        if (!charge(c, expr->start, expr->end, expr->argc == 0 ? 1 : expr->argc)) {
            return 0;
        }
        if (expr->argc == 0) {
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
        if (position >= base.length || base.elems == NULL) {
            c->failed = 1;
            value_clear(&base);
            return 0;
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
        if (position >= base.length || base.elems == NULL) {
            c->failed = 1;
            value_clear(&base);
            value_clear(&index_value);
            return 0;
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
        Value *items = NULL;
        uint32_t position = 0;
        uint32_t slot;
        uint32_t length;
        uint64_t cost;
        TypeKind element_type;
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
        if (base.length == 0 || base.elems == NULL ||
            !index_position(c, &index_value, c->exprs[expr->right].start, c->exprs[expr->right].end, &position)) {
            c->failed = 1;
            value_clear(&base);
            value_clear(&index_value);
            value_clear(&element);
            return 0;
        }
        length = base.length;
        element_type = base.type;
        cost = ((uint64_t)length + 63u) / 64u;
        if (position >= length || !charge(c, expr->start, expr->end, cost == 0 ? 1 : cost)) {
            c->failed = 1;
            value_clear(&base);
            value_clear(&index_value);
            value_clear(&element);
            return 0;
        }
        if (!alloc_array(c, &items, length, expr->start, expr->end)) {
            value_clear(&base);
            value_clear(&index_value);
            value_clear(&element);
            return 0;
        }
        for (slot = 0; slot < length; slot++) {
            const Value *source = slot == position ? &element : &base.elems[slot];
            if (!value_clone(c, &items[slot], source, expr->start, expr->end)) {
                value_list_clear(items, length);
                value_clear(&base);
                value_clear(&index_value);
                value_clear(&element);
                return 0;
            }
        }
        value_clear(&base);
        value_clear(&index_value);
        value_clear(&element);
        out->type = element_type;
        out->length = length;
        out->elems = items;
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
        memset(&acc, 0, sizeof acc);
        if (!loop->bounds_ok || c->loop_k == NULL || c->loop_acc == NULL || loop->init_expr == UINT32_MAX ||
            loop->step_expr == UINT32_MAX) {
            c->failed = 1;
            return 0;
        }
        if (!eval_expr(c, loop->init_expr, params, locals, depth, &acc) ||
            !charge(c, expr->start, expr->end, 1)) {
            value_clear(&acc);
            return 0;
        }
        for (step_index = loop->bound_a; step_index < loop->bound_b; step_index++) {
            if (!charge(c, expr->start, expr->end, 1)) {
                value_clear(&acc);
                value_clear(&c->loop_acc[expr->arg0]);
                return 0;
            }
            c->loop_k[expr->arg0] = step_index;
            value_clear(&c->loop_acc[expr->arg0]);
            value_move(&c->loop_acc[expr->arg0], &acc);
            if (!eval_expr(c, loop->step_expr, params, locals, depth, &acc)) {
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
    case EX_ACCUM: {
        if (c->loop_acc == NULL || !charge(c, expr->start, expr->end, 1)) {
            c->failed = 1;
            return 0;
        }
        return value_clone(c, out, &c->loop_acc[expr->arg0], expr->start, expr->end);
    }
    case EX_COND: {
        uint32_t arg0 = expr->arg0;
        uint16_t arms = expr->argc;
        uint32_t otherwise = expr->right;
        uint32_t span_start = expr->start;
        uint32_t span_end = expr->end;
        uint16_t arm;
        for (arm = 0; arm < arms; arm++) {
            Value condition;
            uint32_t condition_expr = c->cond_arms[arg0 + arm].cond;
            uint32_t value_expr = c->cond_arms[arg0 + arm].value;
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
                return eval_expr(c, value_expr, params, locals, depth, out);
            }
            value_clear(&condition);
        }
        return eval_expr(c, otherwise, params, locals, depth, out);
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

static int format_value(const Value *value, TextBuf *buf) {
    uint32_t index;
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
    if (left->start != right->start) {
        return left->start < right->start ? -1 : 1;
    }
    return strcmp(left->code, right->code);
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

static void render_diags(Compiler *c, FILE *out) {
    uint32_t index;
    if (c->ndiags > 1) {
        qsort(c->diags, c->ndiags, sizeof(Diag), compare_diag);
    }
    for (index = 0; index < c->ndiags; index++) {
        Diag *diag = &c->diags[index];
        uint32_t line = 1;
        uint32_t column = 1;
        size_t line_start = 0;
        size_t cursor = 0;
        uint32_t seen = 1;
        line_col(c->text, c->length, diag->start, &line, &column);
        while (cursor < c->length && seen < line) {
            if (c->text[cursor] == '\n') {
                seen++;
                line_start = cursor + 1;
            } else if (c->text[cursor] == '\r') {
                seen++;
                cursor++;
                line_start = cursor < c->length && c->text[cursor] == '\n' ? cursor + 1 : cursor;
                if (cursor < c->length && c->text[cursor - 1] == '\r' && c->text[cursor] == '\n') {
                    cursor++;
                }
                continue;
            }
            cursor++;
        }
        fprintf(out, "error[%s]: %s\n --> %s:%u:%u\n", diag->code, diag->message, c->filename, line, column);
        if (diag->label[0] != '\0') {
            fprintf(out, "  = %s\n", diag->label);
        }
        if (diag->note[0] != '\0') {
            fprintf(out, "  = note: %s\n", diag->note);
        }
        (void)line_start;
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

static int evaluate_source(Compiler *c, FILE *out) {
    TextBuf program = {0};
    uint32_t index;
    if (!ensure_loops(c)) {
        release_loop_values(c);
        return 1;
    }
    for (index = 0; index < c->nfuncs && !c->failed; index++) {
        Func *func = &c->funcs[index];
        Value result;
        TextBuf value = {0};
        char type_text[192];
        int stop = 0;
        if (!func->typed || func->duplicate || func->nparams != 0 || !func->signature_ok) {
            continue;
        }
        memset(&result, 0, sizeof result);
        if (!eval_function(c, index, NULL, 1, &result)) {
            value_clear(&result);
            break;
        }
        if (func->result_len == 0 && func->result != TY_INT && func->result != TY_BOOL && func->result != TY_MOD) {
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
        if (!format_type(c, type_text, sizeof type_text, func->result, func->result_len, func->result_mod)) {
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
    free(program.data);
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
    free(compiler->edges);
    free(compiler->loops);
    free(compiler->cond_arms);
    free(compiler->loop_k);
    free(compiler->loop_acc);
    free(compiler->requested);
    free(compiler->types);
    free(compiler->sites);
    free(compiler->moduli);
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
        char message[192];
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
        char message[192];
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
        fprintf(err, "error[ORC1001]: %s\n", error);
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
    int count = dependency_order ? program->norder : program->nmods;
    for (index = 0; index < count; index++) {
        Compiler *mod = dependency_order ? program->mods[program->order[index]] : program->mods[index];
        if (mod->ndiags > 0) {
            render_diags(mod, err);
        }
    }
}

static int compile_text(char *text, size_t length, const char *filename, int command, FILE *out, FILE *err) {
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
        "conversion, array, loop, conditional, lookup, module, and residue fragment.\n"
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

int orange_main(int argc, char **argv) {
    int command = -1;
    const char *path = NULL;
    char *text;
    size_t length = 0;
    char error[256];
    int index;
    for (index = 1; index < argc; index++) {
        if (strcmp(argv[index], "-h") == 0 || strcmp(argv[index], "--help") == 0) {
            print_usage(stdout);
            return 0;
        }
        if (strcmp(argv[index], "-V") == 0 || strcmp(argv[index], "--version") == 0) {
            fputs("orangec (standalone C) slice S3i\n", stdout);
            return 0;
        }
        if (strcmp(argv[index], "--self-test") == 0) {
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
        int status = compile_text(text, length, path, command, stdout, stderr);
        free(text);
        return status;
    }
}
