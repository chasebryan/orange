#ifndef ORANGE_H
#define ORANGE_H

/* Shared declarations for the standalone C frontend.
   compile.c and typeparams.c are separate translation units. */

#include "bigint.h"

#include <stddef.h>
#include <stdint.h>

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
/* The most steps `eval --steps` admits: 1,024 times the default budget. */
#define MAX_STEP_LIMIT 1073741824ull
#define MAX_CALL_DEPTH 256
#define MAX_ARRAY_LENGTH 65536u
#define MAX_ARRAY_ELEMENTS 65536u
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
#define MAX_TUPLE 16
#define MAX_SIZES 4
#define MAX_INSTANCES 256

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


typedef enum TypeKind { TY_NONE = 0, TY_INT, TY_BOOL, TY_W8, TY_W16, TY_W32, TY_W64, TY_MOD, TY_TUPLE } TypeKind;

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
    EX_COND,
    EX_TUPLE,
    EX_PROJECT,
    EX_BYTES,
    EX_SLICE,
    EX_SLICE_UP
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
    /* A tuple type. `elem0`/`elem_n` index the element type sites. */
    int is_tuple;
    int tuple_elem;
    uint32_t elem0;
    uint16_t elem_n;
    /* A length written with sizes: an integer token stays a decoded length. */
    int has_size_expr;
    uint32_t length_expr;
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
    int is_tuple;
    int tuple_elem;
    /* Element type sites, filled while parsing. */
    uint32_t elem0;
    uint16_t elem_n;
    /* Resolved element types in `Compiler.telems`. */
    uint32_t tup0;
    uint16_t tup_n;
    int has_size_expr;
    uint32_t length_expr;
    /* Function that owns this site, or UINT32_MAX for a `type` declaration. */
    uint32_t owner_func;
    /* 0xFF when this use is not a type parameter. Otherwise the bracket slot. */
    uint8_t param_slot;
    uint8_t param_axis;
    uint32_t param_len;
} TypeSite;

typedef struct TypeDecl {
    uint32_t name_start;
    uint32_t name_end;
    uint32_t site;
    int installed;
} TypeDecl;

typedef enum NameRes {
    NAME_NONE = 0,
    NAME_PARAM,
    NAME_LOCAL,
    NAME_MISSING,
    NAME_EARLY,
    NAME_BAD,
    NAME_BOOL,
    NAME_BLOCK,
    NAME_BLOCK_EARLY,
    NAME_SIZE
} NameRes;

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
    uint32_t argc;
    uint32_t arg0;
    TokenKind op;
    uint32_t left;
    uint32_t right;
    uint32_t op_start;
    uint32_t op_end;
    TypeKind ty;
    uint32_t ty_len;
    uint16_t ty_mod;
    /* Tuple shape this expression was checked as, when `ty` is a tuple.
       Indices into `Compiler.telems`. A nested generic call reads them back
       so the caller's concrete instance chooses the callee. */
    uint32_t ty_tup0;
    uint16_t ty_tup_n;
    TypeKind conv_ty;
    int conv_ok;
    uint32_t conv_site;
    uint32_t conv_len;
    uint16_t conv_mod;
    /* 0: plain `as`. 1: `as big`. 2: `as little`. The order word's span is `lit_start`/`lit_end`. */
    uint8_t conv_order;
    NameRes name_res;
    uint16_t name_index;
    TypeKind name_ty;
    uint32_t name_len;
    /* Absolute local index when the name is a block binding. */
    uint32_t name_abs;
    /* Bindings of a conditional's final else branch. */
    uint32_t else_bind0;
    uint16_t else_nbinds;
    /* `.k` position, or the element of a projected accumulator. */
    uint32_t proj_pos;
    uint8_t is_proj;
    /* Call sizes, stored in `Compiler.args` at `size0`. */
    uint8_t nsize;
    uint32_t size0;
    /* Fill or other length written as a size expression. UINT32_MAX if none. */
    uint32_t size_expr;
    /* Instance this call names. UINT32_MAX until checking resolves it. */
    uint32_t inst_id;
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
    uint32_t tup0;
    uint16_t tup_n;
    /* 2 when this parameter is a matrix. `inner` is the row length. */
    int rank;
    uint32_t inner;
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
    /* 1 when the binding belongs to a loop step or a conditional branch. */
    int block;
    /* A tuple pattern. `pat_len` is set on the first name; later names have `pat_i` > 0.
       Every name's `type` is its element type. The shared value lives on the first name. */
    uint16_t pat_i;
    uint16_t pat_len;
    uint32_t tup0;
    uint16_t tup_n;
    /* 2 when this binding is a matrix. `inner` is the row length. */
    int rank;
    uint32_t inner;
} Local;

typedef struct Edge {
    uint32_t callee;
    uint32_t start;
    uint32_t end;
    /* Instance indices. UINT32_MAX when the edge is only a function edge. */
    uint32_t caller_inst;
    uint32_t callee_inst;
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
    int a_sized;
    int b_sized;
    uint32_t a_expr;
    uint32_t b_expr;
    uint32_t bind0;
    uint16_t nbinds;
    /* 0 is one accumulator. 2..16 is a tuple pattern. */
    uint8_t nacc;
    uint32_t an_start[MAX_TUPLE];
    uint32_t an_end[MAX_TUPLE];
    uint32_t an_site[MAX_TUPLE];
    uint32_t tup0;
    uint16_t tup_n;
    int acc_rank;
    uint32_t acc_inner;
    /* 1 after the step is scanned. A component with one use is moved out of
       the accumulator, so a later update of that array is the only owner. */
    uint8_t sole_ready;
    uint8_t sole_whole;
    uint8_t sole_comp[MAX_TUPLE];
} LoopDesc;

typedef struct OpenLoop {
    uint32_t id;
    uint32_t index_start;
    uint32_t index_end;
    uint32_t acc_start;
    uint32_t acc_end;
    uint8_t nacc;
    uint32_t acc_at[MAX_TUPLE];
    uint32_t acc_to[MAX_TUPLE];
} OpenLoop;

typedef struct CondArm {
    uint32_t cond;
    uint32_t value;
    uint32_t bind0;
    uint16_t nbinds;
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
    uint32_t tup0;
    uint16_t tup_n;
    int result_rank;
    uint32_t result_inner;
    uint32_t body;
    uint32_t edge0;
    uint32_t nedges;
    int has_blocks;
    uint8_t nsizes;
    int sizes_ok;
    uint32_t sz_name0[MAX_SIZES];
    uint32_t sz_name1[MAX_SIZES];
    uint32_t sz_span0[MAX_SIZES];
    uint32_t sz_span1[MAX_SIZES];
    uint32_t sz_a0[MAX_SIZES];
    uint32_t sz_a1[MAX_SIZES];
    uint32_t sz_b0[MAX_SIZES];
    uint32_t sz_b1[MAX_SIZES];
    int64_t sz_lo[MAX_SIZES];
    int64_t sz_hi[MAX_SIZES];
    /* 0 is a size `n in a..b`. 1 is a type `K in {T, U}`. */
    uint8_t sz_kind[MAX_SIZES];
    uint32_t sz_list0[MAX_SIZES];
    uint16_t sz_nlist[MAX_SIZES];
    uint32_t inst0;
    uint16_t ninst;
} Func;

/* One concrete signature of a function. A function without sizes has one. */
typedef struct Instance {
    uint32_t func;
    int64_t sz[MAX_SIZES];
    TypeKind result;
    uint32_t result_len;
    uint16_t result_mod;
    int result_ok;
    uint32_t tup0;
    uint16_t tup_n;
    int result_rank;
    uint32_t result_inner;
    uint32_t param0;
    int signature_ok;
} Instance;

typedef struct InstParam {
    TypeKind type;
    uint32_t length;
    uint16_t mod_index;
    int type_ok;
    uint32_t tup0;
    uint16_t tup_n;
    int rank;
    uint32_t inner;
} InstParam;

typedef struct Diag {
    const char *code;
    char message[384];
    char label[192];
    char note[320];
    char note2[320];
    char note3[320];
    char sec_label[192];
    uint32_t start;
    uint32_t end;
    uint32_t sec_start;
    uint32_t sec_end;
    uint8_t has_sec;
    uint8_t has_note2;
    uint8_t has_note3;
} Diag;

typedef struct Pack Pack;

typedef struct Value {
    TypeKind type;
    uint32_t length;
    /* Owned element block when length > 0 and `pack` is null. A copy
       duplicates the block; value_clear releases it. Words and residues
       whose modulus fits in 64 bits use `pack` instead, and that block is
       shared until an update needs a private copy. Scalar values leave both
       null. */
    struct Value *elems;
    Pack *pack;
    uint64_t word;
    Big big;
    uint16_t mod_index;
    uint8_t is_tuple;
} Value;

/* Bindings of the step or branch currently being checked or evaluated. */
typedef struct BlockFrame {
    uint32_t bind0;
    uint16_t nbinds;
    uint16_t visible;
    Value *slots;
} BlockFrame;

typedef struct FinishedBlock {
    uint32_t bind0;
    uint16_t nbinds;
} FinishedBlock;

typedef struct Program Program;

/* One resolved element of a tuple type. A shape is a contiguous slice. */
typedef struct TupleElem {
    TypeKind kind;
    uint32_t length;
    uint16_t mod_index;
    int ok;
} TupleElem;

typedef struct UseDecl {
    uint32_t span_start;
    uint32_t span_end;
    uint32_t name_start;
    uint32_t name_end;
} UseDecl;


typedef struct TpStamp {
    uint32_t expr;
    uint32_t inst;
    TypeKind ty;
    uint32_t ty_len;
    uint16_t ty_mod;
    /* Tuple shape for this instance when `ty` is a tuple. Each element is a
       scalar or an array (`TupleElem.length` is the array length). A nested
       call is chosen by the whole shape, so `(K, Bool)`, `(Int, K)`, and
       `(K^2, K)` follow the caller's K on the parts that mention it. */
    uint32_t tup0;
    uint16_t tup_n;
    TypeKind conv_ty;
    uint32_t conv_len;
    uint16_t conv_mod;
    int conv_ok;
} TpStamp;

typedef struct Sz {
    int kind; /* 0 value, 1 not static, 2 too large */
    int64_t value;
    uint32_t start;
    uint32_t end;
} Sz;

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
    /* Bindings of loop steps and conditional branches. Separate from body
       locals so a binding value that itself contains a block cannot reuse
       the body slot still being parsed. */
    Local *block_locals;
    uint32_t nblock_locals;
    size_t block_local_cap;
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
    BlockFrame frames[MAX_OPEN_LOOPS];
    int nframes;
    FinishedBlock *finished;
    uint32_t nfinished;
    size_t finished_cap;
    Func *parsing_func;
    uint32_t *loop_k;
    Value *loop_acc;
    CondArm *cond_arms;
    uint32_t ncond_arms;
    size_t cond_arm_cap;
    uint64_t steps;
    uint64_t step_limit;
    int step_hit;
    int show_stats;
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
    TupleElem *telems;
    uint32_t ntelems;
    size_t telem_cap;
    /* Tuple shape required where a tuple is being checked. Indices into `telems`. */
    uint32_t expect_tup0;
    uint16_t expect_tup_n;
    /* Rank of the type required here. 2 is a matrix; `expect_inner` is its row length. */
    int expect_rank;
    uint32_t expect_inner;
    /* Shape of the typed leaf most recently found. Owned by `leaf_owner`. */
    uint32_t leaf_tup0;
    uint16_t leaf_tup_n;
    const struct Compiler *leaf_owner;
    Instance *instances;
    uint32_t ninstances;
    size_t instance_cap;
    InstParam *iparams;
    uint32_t niparams;
    size_t iparam_cap;
    /* Sizes of the instance being checked or evaluated. */
    int64_t cur_sz[MAX_SIZES];
    uint8_t ncur;
    uint32_t cur_func;
    uint32_t cur_inst;
    /* Listed types of type parameters, as site indexes. */
    uint32_t *tp_sites;
    uint32_t ntp_sites;
    size_t tp_site_cap;
    /* Type expected by the place of the call being resolved. */
    int fit_set;
    /* When set, a silent type-parameter lookup also reports. */
    int fit_report;
    TypeKind fit_kind;
    uint32_t fit_len;
    uint16_t fit_mod;
    uint32_t fit_tup0;
    uint16_t fit_tup_n;
    /* Per-instance literal and conversion types captured while checking. */
    struct TpStamp *stamps;
    uint32_t nstamps;
    size_t stamp_cap;
    /* While set, listed types resolve. The ordinary pass leaves them for admit. */
    int admit_listed;
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


#endif
