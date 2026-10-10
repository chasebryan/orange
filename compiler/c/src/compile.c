/* Standalone C frontend for one Orange 2026 program.
   This file owns lexing, parsing, checking, reference evaluation, and the
   CLI behind orange_main. Exact integer magnitudes live in bigint.c.
   src/main.c only forwards the process arguments.

   Slice boundary: S3a through S3m. The admitted source is edition 2026.
   A program is a root module plus every module it reaches through use.
   check and eval read module m of `use m;` from m.or beside the root.
   lex reads only the file it is given. A module names each used module
   once, before its functions, and calls that module's functions as
   m::f(...). The module graph is acyclic and is checked before any
   module. Each module is then checked on its own, after the modules it
   uses. A typed spec may have parameters, let bindings, and one result
   expression. The scalar types are Int, Bool, Word[8], Word[16],
   Word[32], Word[64], and Mod[m]. A module may name a type with `type`
   after its use declarations and before its functions. A bad alias
   target is reported once, at that declaration: ORC0204 for a Word
   width other than 8, 16, 32, or 64, and ORC0221 for a bad array length.
   A use of the alias does not report that target again. Mod[m] is the
   residue ring of a constant modulus from 2 through 2^521 - 1. The
   constant is built from integer literals with +, -, *, <<, and
   parentheses, evaluated once, and stored in the modulus table. +, -, *,
   and prefix - reduce to the least residue. / multiplies by an inverse
   and is 0 when there is none. A fixed-length array T^n holds n values
   of one scalar, with n a decimal
   integer from 1 through 256. A named array type may add an axis with
   `^LENGTH`, through 4. A fifth axis is ORC0203. The message names the
   type, as in "`Hyper` already has 4 array dimensions". The label is
   "arrays have at most 4 dimensions". The length span says "this length
   would add a fifth dimension". The note is "a row holds scalars, and
   each `^LENGTH` after a named array type adds a dimension of its
   rows". Expressions are literals, names, calls,
   parentheses, array literals, indices, exact integer arithmetic,
   Euclidean / and %, word ring arithmetic, bitwise operators, shifts,
   rotations, comparisons, !, &&, ||, and as conversions. A loop
   `for i in a..b with s: T = start { step }` folds step from the literal
   bound a up to b. An index is proved in range before evaluation. A word
   index ranges over its type, narrowed by bitwise operators, a literal
   shift, arithmetic, conversions, and conditionals where the result
   cannot wrap. An Int index is built from integer literals, loop indices,
   words and residues converted with `as Int`, arithmetic, and
   conditionals. A loop bound or index literal that does not fit the
   16,384-bit budget is
   ORC0205, the same code as any other oversized literal. A bound or index
   that decodes and is still out of range is ORC0225 or ORC0223. A second
   index of a scalar is ORC0224, reported once, and is not a syntax error.
   A rejected `as` target is ORC0204 for a bad word width or ORC0203 for
   any other unsupported type, and the operand is still checked. A loop
   step that fails after its opening brace is recovered without an extra
   ORC0104 on the function close. `x with
   [i] = v` replaces one element, and `[v; n]` repeats a value. A missing
   `=` after that index is ORC0101. Its note is "an update is written
   `x with [i] = value`, or `x with [i][j] = value` for an element of a
   row". `if c { a }
   else { b }` chooses one value; an else-if chain is one conditional, and
   only the chosen branch is evaluated. A loop's step and each branch of
   a conditional may begin with `let` bindings. A step's bindings run
   afresh at every step, a branch's only when that branch is chosen, and
   each name is in scope only inside its step or branch. Those bindings
   do not give the enclosing `if` a type of its own, so an untyped `if`
   reports ORC0220 only. A cross-module `Mod` call compares modulus
   values, not each file's private table index, and retags the value at
   the module boundary; a mismatch is ORC0214. A rejected result type
   (`Float`, a later `type` name, or `Mod[1]`) does not also check the
   body. A rejected `!`, `&&`, or `||` is ORC0215 only. A tuple type `(T, U)`
   holds 2 through 16 scalars or arrays. `(a, b)` builds one from left
   to right, `.k` selects an element, and a `let` or `with` pattern
   names each element. `let(x)` is a call, not a pattern. A pattern name
   that repeats the loop index is ORC0219; a pattern name used outside
   the loop is ORC0211. `p.01`, `p.0.1`, and `x[0].1` are each one
   ORC0101. A tuple inside a tuple, and an array of tuples, are rejected
   even through a `type` alias. Order on an array or a tuple is ORC0215.
   `==` and `!=` of a written-out array or tuple with no type of its own
   is ORC0227.
   A byte string "..." or hex"..." is the array Word[8]^n of its bytes,
   with n from 1 through 256. Characters are printable ASCII, from a
   space through `~`, or an escape; a non-printable or non-ASCII byte is
   ORC0235. An empty "" is ORC0221. A hex string is pairs of hex digits,
   and a space may separate bytes. A lexical error is ORC0009 at the
   first bad character. An unterminated hex string is ORC0003. hex "00"
   with a space before the quote is ORC0101. ++ joins two arrays of one
   element type, and a join longer than 256 bytes is ORC0222. A slice
   `x[a..b]` holds the b - a elements from index a, and
   `x with [a..b] = v` replaces that run. At least one bound is written.
   Bounds are integer literals and loop indices with +, -, and * by a
   constant, proved in range before evaluation. A runtime bound, such as
   the parameter in `data`, or a non-linear bound, such as `i * i` in
   `squared`, is ORC0226. A length that changes from step to step is
   ORC0236. A last step that leaves the array is ORC0223.
   A function may take at most 4 size parameters, `spec f[n in a..b]`,
   as in `mac[len in 1..256]`. n takes each value from a up to, but not
   including, b, with a < b and both bounds at most 65536. The function
   is checked once for each combination, at most 256 instances, and the
   first size changes slowest. An empty range, a bound past 65536, and
   a product past the cap (`many` has 361) are ORC0238, and the body is
   not checked. Instances of one function are checked from the first
   value upward. The first diagnostic ends that walk, and it names that
   instance, as in `last[1]` or `none[0]`. A sized length outside 1
   through 256 is ORC0221, and its note says 1 through 65536. A size is an
   Int constant in that instance, built from integer literals and the
   function's size parameters with +, -, *, /, %, and parentheses. / and
   % are Euclidean, the same rules as for Int, so `blocks[1]` is 3.
   Anything else in a size is ORC0237. A computed length or bound is
   parenthesized:
   `^n + 1`, `[0; 2 * n]`, and `0..n - 1` are ORC0101. A call writes one
   size per parameter, `f[2](x)` or `sha256::sha256[n](...)`, or writes
   none and fits the one instance whose array lengths match. An
   out-of-range size is ORC0238. The wrong number of sizes, including a
   size on a function that has none, is ORC0239. No matching instance is
   ORC0238, and more than one match is ORC0239. Instances may call one
   another. A cycle among them is ORC0217 and prints the chain, as in
   `swap[1] -> swap[2] -> swap[1]`.
   `for`, `in`, `with`, `if`, and `else` are names outside those
   positions. `true` and `false` are Bool values where no parameter or
   binding of that spelling is in scope. Empty spec and impl
   declarations parse and have no value.

   Evaluation owns each array in the Value that holds it and releases the
   element block when that value dies. The whole program shares one step
   budget. Printing grows a buffer to the spelling of the value. Failing
   to retain an array or its spelling reports ORC0301 and prints no value
   lines. A module-cycle message is kept inside its buffer.

   Token spellings match compiler/crates/orange-compiler/src/lexer.rs, so
   lex output can be compared with the Rust frontend. `let` and `as` stay
   identifiers there, and they stay identifiers here.

   Fail closed. Byte order, type parameters, tests, lengths above 256,
   and computed shift amounts are rejected rather than given a new
   meaning. The lexer still produces the Rust token
   names for those forms. The parser or the checker rejects them. This
   file does not implement S3n or any later slice.

   Layout:
     limits, token kinds, and the Rust token-name table
     syntax nodes, functions, diagnostics, Compiler, and Program
     diagnostic recording and table growth
     lexer
     parser
     checker
     reference evaluator
     diagnostic text, commands, and CLI */

#include "compile.h"

#include "bigint.h"

#include <ctype.h>
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Resource limits. Exhausting one of them yields a single resource
   diagnostic and no value lines. The numbers match the README. */
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
#define MAX_TUPLE 16
#define MAX_SIZES 4
#define MAX_INSTANCES 256

/* --- Tokens ---------------------------------------------------------------- */

/* Kind order is the index into TOKEN_NAMES. The strings are the Rust
   lexer's display names, including keywords and punctuation this slice
   does not accept as expressions. */
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

_Static_assert(sizeof TOKEN_NAMES / sizeof TOKEN_NAMES[0] == (size_t)TK_QUESTION + 1,
               "TOKEN_NAMES must stay aligned with TokenKind");

/* --- Syntax and semantics -------------------------------------------------- */

/* Admitted types. TY_NONE is an absent or rejected type, not a value.
   TY_BOOL is Bool. TY_MOD is a residue ring Mod[m]; the modulus lives in
   the modulus table. TY_TUPLE is a fixed tuple of 2 through 16 scalars
   or arrays. */
typedef enum TypeKind { TY_NONE = 0, TY_INT, TY_BOOL, TY_W8, TY_W16, TY_W32, TY_W64, TY_MOD, TY_TUPLE } TypeKind;

/* Flat expression node. Fields belong to the kind that uses them:
     EX_LIT    negative, lit_start, lit_end
     EX_NAME   name_start, name_end, then name_res, name_index, name_ty, name_len
     EX_CALL   name_* is the function. left and right are the module span of
               a qualified call, or left is UINT32_MAX for a local call.
               arg0, argc, and callee are filled by the checker. name_index
               is the callee's module.
     EX_UNARY  op, left, op_start, op_end
     EX_BINARY op, left, right, op_start, op_end
     EX_SHIFT  op, left, right (the amount), op_start, op_end
     EX_CONV   left, conv_ty, conv_ok, name_* holds the target type span
     EX_GROUP  left
     EX_ARRAY  arg0, argc (the elements, in the shared argument table)
     EX_INDEX  left (the array), lit_start, lit_end (a literal index)
     EX_SELECT left (the array), right (an index expression)
     EX_UPDATE left (the array), right (the index), callee (the new element)
     EX_FILL   left (the repeated element), lit_start, lit_end (the length)
     EX_LOOP   arg0 (the LoopDesc)
     EX_LOOP_INDEX and EX_ACCUM  arg0 (the enclosing LoopDesc)
     EX_COND   arg0, argc (the condition/value arms), right (the else value).
               Each arm and the else may open with block bindings.
     EX_TUPLE  arg0, argc (the elements, evaluated left to right)
     EX_PROJECT left (the tuple), proj_pos (the decimal position `.k`)
   ty is the scalar the checker expects. ty_len is 0 for a scalar and the
   array length otherwise. height is the syntax height used by the 256
   limit. Child indexes are UINT32_MAX when absent. */
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
    /* 0 is a scalar. 1 through 4 count array axes. A fifth axis is ORC0203. */
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
} InstParam;

typedef struct Diag {
    const char *code;
    char message[384];
    char label[192];
    char note[320];
    char note2[320];
    char sec_label[192];
    uint32_t start;
    uint32_t end;
    uint32_t sec_start;
    uint32_t sec_end;
    uint8_t has_sec;
    uint8_t has_note2;
} Diag;

/* A scalar has length 0 and a null element pointer. An array owns the
   block at elems and runs for `length` slots. value_clear releases it. */
typedef struct Value {
    TypeKind type;
    uint32_t length;
    /* Owned element block when length > 0. A copy duplicates the block;
       value_clear releases it. Scalar values leave this null. */
    struct Value *elems;
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

/* One `use name;` in a module. The name is the stem of the sibling file. */
typedef struct UseDecl {
    uint32_t span_start;
    uint32_t span_end;
    uint32_t name_start;
    uint32_t name_end;
} UseDecl;

/* One module. Tables grow with ensure_cap. The arena holds Int limbs for
   this module and is discarded with the Compiler. `program` is the graph
   this module belongs to. `requested` is the stem it was loaded as. */
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
} Compiler;

/* The root is mods[0]. order is the check order: a module after the
   modules it uses. use_target is the program index named by each use, or
   UINT16_MAX when unresolved. Index 0 is the root, so an unresolved use
   must not default to 0. */
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

/* --- Parser cursor --------------------------------------------------------- */

/* The cursor stops on TK_EOF. Spans are byte offsets into c->text. */

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

static void found_token_label(TokenKind kind, char *buf, size_t cap) {
    const char *name = "EOF";
    if ((unsigned)kind < sizeof TOKEN_NAMES / sizeof TOKEN_NAMES[0]) {
        name = TOKEN_NAMES[kind];
    }
    snprintf(buf, cap, "found %s", name);
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

/* --- Diagnostics ----------------------------------------------------------- */

/* Phase argument of add_diag. Call sites pass these values as literals.
   Each phase stops after MAX_ORDINARY_DIAGS and then emits one limit code. */
enum { PHASE_LEX = 0, PHASE_PARSE = 1, PHASE_SEMA = 2 };

/* Records one diagnostic. phase selects the counter and the overflow code:
   PHASE_LEX is ORC0007, PHASE_PARSE is ORC0105, PHASE_SEMA is ORC0208.
   A phase that is neither lex nor parse is counted as semantic analysis. */
static void add_diag(Compiler *c, const char *code, uint32_t start, uint32_t end, const char *message,
                     const char *label, const char *note, int phase) {
    uint32_t *count = phase == PHASE_LEX ? &c->lex_diags : phase == PHASE_PARSE ? &c->parse_diags : &c->sema_diags;
    int *limited = phase == PHASE_LEX ? &c->lex_limited : phase == PHASE_PARSE ? &c->parse_limited : &c->sema_limited;
    const char *limit_code = phase == PHASE_LEX ? "ORC0007" : phase == PHASE_PARSE ? "ORC0105" : "ORC0208";
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

static void diag_add_secondary(Compiler *c, uint32_t start, uint32_t end, const char *label) {
    Diag *diag;
    if (c->ndiags == 0) {
        return;
    }
    diag = &c->diags[c->ndiags - 1];
    if (diag->has_sec) {
        return;
    }
    diag->has_sec = 1;
    diag->sec_start = start;
    diag->sec_end = end;
    copy_text(diag->sec_label, sizeof diag->sec_label, label == NULL ? "" : label);
}

/* One resource diagnostic for the whole compilation. Later resource
   failures are ignored. The diagnostic is counted in the parse phase. */
static void resource_diag(Compiler *c, const char *code, uint32_t start, uint32_t end, const char *message) {
    if (c->resource) {
        return;
    }
    c->resource = 1;
    add_diag(c, code, start, end, message, "resource limit reached",
             "the source was not accepted", 1);
}

/* Grow a heap table to at least `need` elements, doubling until `max`.
   Returns 0 when `need` exceeds `max` or the allocation fails. */
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
    expr->size_expr = UINT32_MAX;
    expr->inst_id = UINT32_MAX;
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

/* --- Lexer ----------------------------------------------------------------- */

/* Well-formed UTF-8, rejecting overlong forms and surrogates. Identifiers
   themselves are ASCII; this check is the source-file gate. */
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

/* Reserved words from the Rust lexer. `let` and `as` are not keywords. */
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

/* Scan c->text into c->tokens, including a final TK_EOF.
   Block comments nest. A line comment ends at the line break.
   Integer well-formedness is checked here; the magnitude limit is not.
   A hex string is hex"..." with the quote immediately after hex. An
   unterminated hex string is ORC0003. A bad character, or a hex digit
   with no partner, is ORC0009 at that character. Ordinary "..." strings
   are tokens here; printable ASCII and escapes are checked later. An
   unrecognized byte is ORC0001 and is skipped. */
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
                /* First payload byte. The token span is the whole hex"..."
                   form, so this index is not stored on the token. */
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
                    uint32_t at = (uint32_t)offense;
                    unsigned char ch = at < c->length ? (unsigned char)c->text[at] : 0;
                    char message[80];
                    const char *label;
                    if (lone || (pending && !bad)) {
                        snprintf(message, sizeof message, "hex digit '%c' has no partner", ch);
                        label = "a byte is written as two hex digits";
                    } else if (ch == '\\') {
                        snprintf(message, sizeof message, "'\\' cannot appear in a hex string");
                        label = "a hex string has no escapes";
                    } else if (ch >= 0x21 && ch <= 0x7e) {
                        snprintf(message, sizeof message, "'%c' cannot appear in a hex string", ch);
                        label = "not a hex digit or a space";
                    } else {
                        snprintf(message, sizeof message, "U+%04X cannot appear in a hex string", ch);
                        label = "not a hex digit or a space";
                    }
                    add_diag(c, "ORC0009", at, at + 1, message, label,
                             "a hex string holds bytes written as pairs of hex digits, as in `hex\"00 1f a0\"`; spaces "
                             "may separate bytes but not split one",
                             0);
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

/* --- Parser ---------------------------------------------------------------- */

/* Recursive descent over the token cursor. A return of 0 means no node
   was produced: a resource limit, or an operand that could not be parsed.
   A return of 1 means the construct was consumed; a diagnostic may already
   have been recorded. parse_source itself always returns 1.
   Operator groups do not mix without parentheses. Shifts and `as` take
   one right-hand operand and do not chain. */

/* Nesting counts the parse stack. Height is stored on each node. */
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
           is_compare_op(kind) || kind == TK_AMPAMP || kind == TK_PIPEPIPE || kind == TK_SLASH || kind == TK_PERCENT ||
           kind == TK_PLUSPLUS;
}

/* A following operator, conversion, or `with [` must be parenthesized
   when it does not belong to the expression already started. */
static int trailing_joiner(const Compiler *c) {
    return is_binary_kind(peek_kind(c)) || is_as(c) || is_with_update(c);
}

/* Operator groups that must not be mixed without parentheses:
   1 arithmetic (+, -, *), 2 &, 3 |, 4 ^, 5 shifts and rotations,
   6 comparisons, 7 &&, 8 ||, 9 Euclidean / and %, 10 concatenation (++).
   In group 1, `*` folds inside `+` and `-`. Groups 2 through 4, 7, 8,
   and 10 chain only with the same operator. Groups 5, 6, and 9 take one
   right-hand operand and do not chain. */
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
    if (kind == TK_PLUSPLUS) {
        return 10;
    }
    return 0;
}

/* After an ungrouped operator, move to the next separator so one
   expression reports that fault once. */
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
   function-body skip still sees the function's own closing brace. A loop
   step that fails after `{` uses this so that close is not an extra ORC0104. */
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

/* Skip to the closing brace of a function that cannot be parsed.
   inside_body is 1 when the opening brace has already been consumed. */
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

/* Decimal length from 1 through MAX_ARRAY_LENGTH. A leading zero is rejected. */
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

static const char SIZE_PARAMETER_NOTE[] =
    "a sized function is written `spec f[n in 1..5](x: Word[8]^n) -> Type { ... }` and checked once for each n from 1 up to, but not including, 5";
static const char SIZED_CALL_NOTE[] =
    "a sized function is called with its sizes in brackets before its arguments, as in `sha256[2](m)`";
static const char COMPUTED_LENGTH_NOTE[] =
    "an array length computed from sizes is written in parentheses, as in `Word[8]^(2 * n)`";
static const char COMPUTED_FILL_NOTE[] =
    "a length computed from sizes is written in parentheses, as in `[0; (2 * n)]`";
static const char COMPUTED_BOUND_NOTE[] =
    "a bound computed from sizes is written in parentheses, as in `for i in 0..(n - 1) with s: Type = start { step }`";

static int size_operator(TokenKind kind) {
    return kind == TK_PLUS || kind == TK_MINUS || kind == TK_STAR || kind == TK_SLASH || kind == TK_PERCENT;
}

/* A size atom is an integer, a name other than `with`, or a parenthesized expression.
   An operator after the atom stays for the caller, which asks for parentheses. */
static int parse_size_atom(Compiler *c, const char *what, const char *note, uint32_t *out) {
    Token token = peek_token(c);
    if (token.kind == TK_INT) {
        advance_token(c);
        return make_lit(c, token.start, token.end, 0, token.start, token.end, out);
    }
    if (token.kind == TK_IDENT && !span_is(c, token.start, token.end, "with")) {
        advance_token(c);
        if (!new_expr(c, out)) {
            return 0;
        }
        c->exprs[*out].kind = EX_NAME;
        c->exprs[*out].start = token.start;
        c->exprs[*out].end = token.end;
        c->exprs[*out].name_start = token.start;
        c->exprs[*out].name_end = token.end;
        c->exprs[*out].height = 1;
        return note_height(c, *out);
    }
    if (token.kind == TK_LPAREN) {
        return parse_prefixed(c, out);
    }
    add_diag(c, "ORC0101", token.start, token.end, what, "expected a size", note, 1);
    return 0;
}

/* Brackets hold only tokens a size list can hold, and `(` follows `]`. */
static int starts_sized_call(const Compiler *c) {
    size_t pos;
    int depth = 0;
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
            kind == TK_SLASH || kind == TK_PERCENT || kind == TK_COMMA) {
        } else if (kind == TK_LPAREN) {
            depth++;
        } else if (kind == TK_RPAREN && depth > 0) {
            depth--;
        } else if (kind == TK_RBRACKET && depth == 0) {
            return pos + 1 < c->ntokens && c->tokens[pos + 1].kind == TK_LPAREN;
        } else {
            return 0;
        }
        pos++;
    }
}

static int parse_call_sizes(Compiler *c, uint32_t *size0, uint8_t *nsize, int *child_height) {
    *size0 = c->nargs;
    *nsize = 0;
    advance_token(c);
    if (peek_kind(c) == TK_RBRACKET) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected a size", "expected a size",
                 SIZED_CALL_NOTE, 1);
        return 0;
    }
    for (;;) {
        uint32_t size = 0;
        int size_height;
        if (!parse_expr(c, &size)) {
            return 0;
        }
        if (*nsize >= MAX_SIZES) {
            add_diag(c, "ORC0101", c->exprs[size].start, c->exprs[size].end, "a call gives at most 4 sizes",
                     "one size too many", SIZED_CALL_NOTE, 1);
            return 0;
        }
        if (!ensure_cap((void **)&c->args, &c->arg_cap, c->nargs + 1, sizeof(uint32_t), MAX_EXPRS)) {
            resource_diag(c, "ORC0106", c->exprs[size].start, c->exprs[size].end, "parser could not retain call sizes");
            return 0;
        }
        c->args[c->nargs++] = size;
        (*nsize)++;
        size_height = height_of(c, size);
        if (size_height > *child_height) {
            *child_height = size_height;
        }
        if (peek_kind(c) == TK_COMMA) {
            advance_token(c);
            continue;
        }
        if (peek_kind(c) == TK_RBRACKET) {
            advance_token(c);
            return 1;
        }
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `,` or `]` after the size",
                 "expected `,` or `]`", SIZED_CALL_NOTE, 1);
        return 0;
    }
}

/* Int, Word[8|16|32|64], Mod[m], a name, and, when allow_array is set, one T^n.
   A length after ^ is a decimal integer or a parenthesized size.
   `^n + 1` is ORC0101. A name that is not an admitted type still
   consumes the type syntax and returns with ok == 0. A bad alias target
   is reported once, at the declaration. A bad type written on a
   signature is reported when that signature is checked. A broken type
   returns 0. A second caret is rejected
   here. as_element rejects a tuple written inside another tuple. */
static int parse_type_body(Compiler *c, DeclaredType *type, int allow_array, int as_element) {
    Token name = peek_token(c);
    int admit_length = 0;
    memset(type, 0, sizeof *type);
    if (name.kind != TK_IDENT) {
        if (as_element) {
            char label[64];
            found_token_label(name.kind, label, sizeof label);
            add_diag(c, "ORC0101", name.start, name.end, "expected an element type", label,
                     "a tuple's elements are `Int`, `Bool`, words, residues, and arrays of them; a tuple holds no tuple",
                     1);
        } else {
            add_diag(c, "ORC0101", name.start, name.end, "expected a type name", "expected a type", NULL, 1);
        }
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
            char label[64];
            found_token_label(close.kind, label, sizeof label);
            add_diag(c, "ORC0101", close.start, close.end, "expected `]` after the modulus", label,
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
        if (length.kind == TK_INT || length.kind == TK_IDENT || length.kind == TK_LPAREN) {
            uint32_t size = UINT32_MAX;
            int sized = length.kind != TK_INT;
            if (sized) {
                if (!parse_size_atom(c, "expected a length after `^`", COMPUTED_LENGTH_NOTE, &size)) {
                    return 0;
                }
            } else {
                advance_token(c);
            }
            if (peek_kind(c) == TK_CARET) {
                char label[64];
                found_token_label(peek_kind(c), label, sizeof label);
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                         "expected the end of the type after its array length", label,
                         "name the row type with a `type` declaration, then write `Row^LENGTH`; repeated `^` dimensions "
                         "are not type syntax",
                         1);
                return 0;
            }
            if (size_operator(peek_kind(c))) {
                char label[64];
                found_token_label(peek_kind(c), label, sizeof label);
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                         "expected the end of the type after its array length", label, COMPUTED_LENGTH_NOTE, 1);
                return 0;
            }
            if (sized) {
                type->has_size_expr = 1;
                type->length_expr = size;
                type->length_start = c->exprs[size].start;
                type->length_end = c->exprs[size].end;
                type->end = c->exprs[size].end;
                type->length = 0;
            } else {
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
        } else {
            add_diag(c, "ORC0101", length.start, length.end, "expected a length after `^`", "expected a length",
                     "an array type is `T^n` with one decimal length, or a length written with sizes", 1);
            return 0;
        }
    }
    return 1;
}

static int push_site(Compiler *c, const DeclaredType *type, const char *role, uint32_t *site_out);

static const char TUPLE_TYPE_NOTE[] =
    "a tuple type is written `(T, U)` with two through 16 element types, each `Int`, `Bool`, a word, a residue, or an array of one";
static const char PATTERN_NOTE[] =
    "a tuple pattern names two through 16 values, each with its type, as in `let (sum: Word[64], carry: Word[64]) = add(x, y, c);`";
static const char POSITION_NOTE[] =
    "a tuple's element is selected by its position, counted from zero and written in decimal, as in `pair.0` or `pair.1`";

/* After an error inside a tuple type, skip through that type's closing `)`. */
static void recover_tuple_type(Compiler *c) {
    int depth = 1;
    while (peek_kind(c) != TK_EOF) {
        Token token = peek_token(c);
        TokenKind kind = token.kind;
        if (kind == TK_LBRACE || kind == TK_RBRACE || kind == TK_SEMI || kind == TK_ARROW || kind == TK_EQUAL) {
            return;
        }
        if (kind == TK_IDENT && (span_is(c, token.start, token.end, "spec") || span_is(c, token.start, token.end, "impl"))) {
            return;
        }
        advance_token(c);
        if (kind == TK_LPAREN) {
            depth++;
        } else if (kind == TK_RPAREN) {
            depth--;
            if (depth <= 0) {
                return;
            }
        }
    }
}

static int parse_tuple_type(Compiler *c, DeclaredType *type) {
    Token open = peek_token(c);
    uint32_t sites[MAX_TUPLE];
    uint32_t count = 0;
    Token close;
    memset(type, 0, sizeof *type);
    type->start = open.start;
    type->kind = TY_TUPLE;
    type->is_tuple = 1;
    advance_token(c);
    for (;;) {
        DeclaredType element;
        uint32_t site = UINT32_MAX;
        Token comma;
        if (count >= MAX_TUPLE) {
            resource_diag(c, "ORC0106", peek_token(c).start, peek_token(c).end, "a tuple type has more than 16 elements");
            recover_tuple_type(c);
            return 0;
        }
        if (!parse_type_body(c, &element, 1, 1)) {
            recover_tuple_type(c);
            return 0;
        }
        element.tuple_elem = 1;
        if (!push_site(c, &element, "element type", &site)) {
            recover_tuple_type(c);
            return 0;
        }
        c->sites[site].tuple_elem = 1;
        sites[count++] = site;
        if (peek_kind(c) == TK_COMMA) {
            comma = peek_token(c);
            advance_token(c);
            if (peek_kind(c) == TK_RPAREN) {
                if (count < 2) {
                    add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected an element type",
                             "expected an element type", TUPLE_TYPE_NOTE, 1);
                    advance_token(c);
                    return 0;
                }
                (void)comma;
                break;
            }
            continue;
        }
        break;
    }
    close = peek_token(c);
    if (count < 2) {
        char label[64];
        found_token_label(close.kind, label, sizeof label);
        add_diag(c, "ORC0101", close.start, close.end, "expected `,` and another element type", label, TUPLE_TYPE_NOTE,
                 1);
        if (close.kind == TK_RPAREN) {
            advance_token(c);
        } else {
            recover_tuple_type(c);
        }
        return 0;
    }
    if (close.kind != TK_RPAREN) {
        add_diag(c, "ORC0101", close.start, close.end, "expected `,` or `)` after the element type", "unclosed tuple type",
                 TUPLE_TYPE_NOTE, 1);
        recover_tuple_type(c);
        return 0;
    }
    type->end = close.end;
    type->elem0 = sites[0];
    type->elem_n = (uint16_t)count;
    type->ok = 1;
    advance_token(c);
    return 1;
}

static int parse_type(Compiler *c, DeclaredType *type, int allow_array) {
    if (peek_kind(c) == TK_LPAREN) {
        if (!parse_tuple_type(c, type)) {
            return 0;
        }
        if (allow_array && peek_kind(c) == TK_CARET) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected the end of the type after the tuple",
                     label,
                     "an array's elements are `Int`, `Bool`, words, or residues; arrays of tuples are not part of Orange 2026",
                     1);
            return 0;
        }
        return 1;
    }
    return parse_type_body(c, type, allow_array, 0);
}

/* Copy a parsed type into the parameter, binding, or result fields. */
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

/* Remember one parsed type until prepare_types resolves names and moduli. */
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
    site->is_tuple = type->is_tuple;
    site->tuple_elem = type->tuple_elem;
    site->elem0 = type->elem0;
    site->elem_n = type->elem_n;
    site->role = role;
    site->has_size_expr = type->has_size_expr;
    site->length_expr = type->length_expr;
    site->owner_func = c->parsing_func != NULL ? c->nfuncs : UINT32_MAX;
    site->wrote_axis = type->has_size_expr || (type->length > 0 && !type->length_bad);
    /* A ground type has at most one axis. A named type stays 0 until
       resolve_site copies the target's rank. */
    if (!type->named && !type->bare_mod) {
        site->rank = site->wrote_axis ? 1 : 0;
    }
    *site_out = c->nsites++;
    return 1;
}

/* Report a scalar type the parser stored with ok == 0. A Word form is
   ORC0204. Anything else is ORC0203. `type` is unused because the span
   decides the code. reject_declared uses ORC0221 when the length itself
   is bad. An alias's bad target is reported once, at the declaration. */
static void reject_type(Compiler *c, TypeKind type, int ok, uint32_t start, uint32_t end) {
    if (ok) {
        return;
    }
    if (end >= start + 4 && memcmp(c->text + start, "Word", 4) == 0) {
        if (end >= start + 5 && c->text[start + 4] == '[') {
            uint32_t width_start = start + 5;
            uint32_t width_end = end;
            if (width_end > width_start && c->text[width_end - 1] == ']') {
                width_end--;
            }
            add_diag(c, "ORC0204", width_start, width_end, "`Word` width must be exactly 8, 16, 32, or 64",
                     "unsupported word width", "word widths do not coerce, truncate, or wrap", 2);
        } else {
            add_diag(c, "ORC0204", start, start + 4, "`Word` requires an exact width of 8, 16, 32, or 64",
                     "missing word width", "write the width in decimal, as in `Word[32]`", 2);
        }
    } else {
        char message[160];
        char name[80];
        size_t length = end > start ? (size_t)(end - start) : 0;
        if (length >= sizeof name) {
            length = sizeof name - 1;
        }
        memcpy(name, c->text + start, length);
        name[length] = '\0';
        snprintf(message, sizeof message, "unsupported type `%s`", name);
        add_diag(c, "ORC0203", start, end, message, "unsupported type",
                 "types are resolved contextually and never inferred by spelling similarity", 2);
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

static int token_word(const Compiler *c, Token token, const char *word) {
    return token.kind == TK_IDENT && span_is(c, token.start, token.end, word);
}

static int is_shift_kind(TokenKind kind) {
    return kind == TK_LSHIFT || kind == TK_RSHIFT || kind == TK_ROL || kind == TK_ROR;
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
    {
        const char *note =
            "operators from different groups have no relative precedence in Orange; parenthesize the part that applies first";
        int previous_as = token_word(c, previous, "as");
        int current_as = token_word(c, token, "as");
        int previous_with = token_word(c, previous, "with");
        int current_with = token_word(c, token, "with");
        int previous_shift = is_shift_kind(previous.kind);
        int current_shift = is_shift_kind(token.kind);
        int previous_div = previous.kind == TK_SLASH || previous.kind == TK_PERCENT;
        int current_div = token.kind == TK_SLASH || token.kind == TK_PERCENT;
        int previous_cmp = is_compare_op(previous.kind);
        int current_cmp = is_compare_op(token.kind);
        if (previous_with || current_with) {
            note = "`with` updates exactly one array; parenthesize the update or the expression it updates";
        } else if (previous_as && token.kind == TK_CARET) {
            note = "`as` converts exactly one operand; parenthesize the conversion or the expression it converts. For "
                   "an array type, name a byte order first, as in `as big Word[32]^16`";
        } else if (previous_as || current_as) {
            note = "`as` converts exactly one operand; parenthesize the conversion or the expression it converts";
        } else if (previous_shift && current_shift) {
            note = "a shift or rotation takes exactly two operands; parenthesize one of them";
        } else if (previous_div && current_div) {
            note = "`/` and `%` take exactly two operands; parenthesize one of them";
        } else if (previous_cmp && current_cmp) {
            note = "a comparison takes exactly two operands; join two comparisons with `&&` or `||`";
        }
        add_diag(c, "ORC0108", token.start, token.end, message, "ungrouped operator", note, 1);
    }
    skip_expr_tail(c);
    return 1;
}

/* Build EX_INDEX for a literal index, or EX_SELECT for an index expression. */
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

static const char SLICE_BOUND_NOTE[] =
    "a slice is written `x[a..b]`, the elements of `x` from index a up to but not including index b, or `x[a..]` or "
    "`x[..b]` to run to the end or from the start";
static const char SLICE_UPDATE_NOTE[] =
    "`x with [a..b] = v` is the array `x` with its elements from index a up to b replaced by those of v";
static const char PARSER_SLICE_UPDATE_NOTE[] =
    "a slice update is written `x with [a..b] = values`, the array `x` with its elements from index a up to but not "
    "including index b replaced";

/* The current token is `..`. `start_expr` is UINT32_MAX when the start is omitted. */
static int parse_slice_range(Compiler *c, uint32_t start_expr, uint32_t *end_expr, uint32_t *range_start,
                             uint32_t *range_end, int *bound_height, const char *note) {
    Token dots = peek_token(c);
    uint32_t end = UINT32_MAX;
    int height = 0;
    advance_token(c);
    if (start_expr != UINT32_MAX) {
        height = height_of(c, start_expr);
    }
    if (peek_kind(c) != TK_RBRACKET) {
        if (!parse_expr(c, &end)) {
            return 0;
        }
        if (height_of(c, end) > height) {
            height = height_of(c, end);
        }
    }
    if (start_expr == UINT32_MAX && end == UINT32_MAX) {
        Token at = peek_token(c);
        char label[64];
        found_token_label(at.kind, label, sizeof label);
        add_diag(c, "ORC0101", at.start, at.end, "expected a bound of the slice after `..`", label, note, 1);
        return 0;
    }
    *range_start = start_expr != UINT32_MAX ? c->exprs[start_expr].start : dots.start;
    *range_end = end != UINT32_MAX ? c->exprs[end].end : dots.end;
    *end_expr = end;
    *bound_height = height;
    return 1;
}

static int finish_slice_expr(Compiler *c, uint32_t base, uint32_t start_expr, uint32_t end_expr, uint32_t range_start,
                             uint32_t range_end, uint32_t close_end, int bound_height, uint32_t *out) {
    int height = height_of(c, base);
    if (bound_height > height) {
        height = bound_height;
    }
    if (!new_expr(c, out)) {
        return 0;
    }
    c->exprs[*out].kind = EX_SLICE;
    c->exprs[*out].left = base;
    c->exprs[*out].right = start_expr;
    c->exprs[*out].callee = end_expr;
    c->exprs[*out].start = c->exprs[base].start;
    c->exprs[*out].end = close_end;
    c->exprs[*out].lit_start = range_start;
    c->exprs[*out].lit_end = range_end;
    c->exprs[*out].height = 1 + height;
    return note_height(c, *out);
}

/* One bracket suffix. A lone integer literal is EX_INDEX. Any other
   index expression is EX_SELECT. `a..b`, with either bound omitted but
   not both, is EX_SLICE. A following `[` is parsed by parse_index. */
static int parse_one_index(Compiler *c, uint32_t base, uint32_t *out) {
    Token open;
    Token index;
    Token close;
    uint32_t child = UINT32_MAX;
    uint32_t end_expr = UINT32_MAX;
    uint32_t range_start = 0;
    uint32_t range_end = 0;
    int bound_height = 0;
    open = peek_token(c);
    advance_token(c);
    if (peek_kind(c) == TK_RBRACKET) {
        char label[64];
        found_token_label(peek_kind(c), label, sizeof label);
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected an index after `[`", label,
                 "an array element is selected by an index, such as `x[0]` or `x[i + 1]`", 1);
        return 0;
    }
    index = peek_token(c);
    if (index.kind == TK_INT && c->at + 1 < c->ntokens && c->tokens[c->at + 1].kind == TK_RBRACKET) {
        advance_token(c);
        close = peek_token(c);
        advance_token(c);
        return finish_index_node(c, base, close.end, EX_INDEX, UINT32_MAX, index.start, index.end, out);
    }
    if (!enter_nest(c, open.start, open.end)) {
        return 0;
    }
    if (index.kind == TK_DOTDOT) {
        if (!parse_slice_range(c, UINT32_MAX, &end_expr, &range_start, &range_end, &bound_height, SLICE_BOUND_NOTE)) {
            leave_nest(c);
            return 0;
        }
        leave_nest(c);
        close = peek_token(c);
        if (close.kind != TK_RBRACKET) {
            char label[64];
            found_token_label(close.kind, label, sizeof label);
            add_diag(c, "ORC0101", close.start, close.end, "expected `]` after the slice", label, SLICE_BOUND_NOTE, 1);
            return 0;
        }
        advance_token(c);
        return finish_slice_expr(c, base, UINT32_MAX, end_expr, range_start, range_end, close.end, bound_height, out);
    }
    if (!parse_expr(c, &child)) {
        leave_nest(c);
        return 0;
    }
    if (peek_kind(c) == TK_DOTDOT) {
        if (!parse_slice_range(c, child, &end_expr, &range_start, &range_end, &bound_height, SLICE_BOUND_NOTE)) {
            leave_nest(c);
            return 0;
        }
        leave_nest(c);
        close = peek_token(c);
        if (close.kind != TK_RBRACKET) {
            char label[64];
            found_token_label(close.kind, label, sizeof label);
            add_diag(c, "ORC0101", close.start, close.end, "expected `]` after the slice", label, SLICE_BOUND_NOTE, 1);
            return 0;
        }
        advance_token(c);
        return finish_slice_expr(c, base, child, end_expr, range_start, range_end, close.end, bound_height, out);
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
   scalar is ORC0224. A slice is taken once: a following `[` or `.` is
   ORC0101. Arrays of arrays stay outside this slice. */
static int parse_index(Compiler *c, uint32_t base, uint32_t *out) {
    if (peek_kind(c) != TK_LBRACKET) {
        *out = base;
        return 1;
    }
    for (;;) {
        if (!parse_one_index(c, base, out)) {
            return 0;
        }
        if (c->exprs[*out].kind == EX_SLICE) {
            if (peek_kind(c) == TK_LBRACKET) {
                char label[64];
                found_token_label(peek_kind(c), label, sizeof label);
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                         "expected an operator or the end of the expression", label,
                         "a slice is taken once, from a name, a call, or a tuple's element; bind it with `let` to select from it",
                         1);
                return 0;
            }
            if (peek_kind(c) == TK_DOT) {
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                         "expected an operator or the end of the expression", "a slice has no `.k`",
                         "a slice is an array, not a tuple, so it has no `.k`", 1);
                return 0;
            }
            return 1;
        }
        if (peek_kind(c) != TK_LBRACKET) {
            return 1;
        }
        base = *out;
    }
}

/* A position is a decimal integer without a leading zero. `p.01` is one
   ORC0101 at the digits and is not read as position 1. */
static int canonical_position(const char *text, uint32_t start, uint32_t end, uint32_t *value) {
    uint64_t acc = 0;
    uint32_t index;
    int overflow = 0;
    if (end <= start) {
        return 0;
    }
    if (text[start] == '0' && end - start != 1) {
        return 0;
    }
    for (index = start; index < end; index++) {
        unsigned char digit = (unsigned char)text[index];
        if (digit < '0' || digit > '9') {
            return 0;
        }
        if (!overflow) {
            if (acc > (uint64_t)UINT32_MAX / 10u) {
                overflow = 1;
            } else {
                acc = acc * 10u + (uint64_t)(digit - '0');
                if (acc > (uint64_t)UINT32_MAX) {
                    overflow = 1;
                }
            }
        }
    }
    *value = overflow ? UINT32_MAX : (uint32_t)acc;
    return 1;
}

static int finish_project(Compiler *c, uint32_t base, uint32_t pos, uint32_t pos_start, uint32_t pos_end, uint32_t *out) {
    if (!new_expr(c, out)) {
        return 0;
    }
    c->exprs[*out].kind = EX_PROJECT;
    c->exprs[*out].left = base;
    c->exprs[*out].proj_pos = pos;
    c->exprs[*out].lit_start = pos_start;
    c->exprs[*out].lit_end = pos_end;
    c->exprs[*out].start = c->exprs[base].start;
    c->exprs[*out].end = pos_end;
    c->exprs[*out].height = 1 + height_of(c, base);
    return note_height(c, *out);
}

/* `.k` follows a name or a call, then at most one index. `p.0.1` and
   `x[0].1` are each one ORC0101 at the extra token and are not parsed further. */
static int parse_suffix(Compiler *c, uint32_t base, uint32_t *out) {
    if (peek_kind(c) == TK_DOT) {
        Token dot = peek_token(c);
        Token position;
        uint32_t pos = 0;
        advance_token(c);
        position = peek_token(c);
        if (position.kind != TK_INT) {
            char label[64];
            found_token_label(position.kind, label, sizeof label);
            add_diag(c, "ORC0101", position.start == dot.end ? dot.start : position.start, position.end,
                     "expected an element's position after `.`", label, POSITION_NOTE, 1);
            return 0;
        }
        if (!canonical_position(c->text, position.start, position.end, &pos)) {
            char label[64];
            found_token_label(position.kind, label, sizeof label);
            add_diag(c, "ORC0101", position.start, position.end, "expected an element's position in decimal", label,
                     POSITION_NOTE, 1);
            advance_token(c);
            return 0;
        }
        advance_token(c);
        if (!finish_project(c, base, pos, position.start, position.end, out)) {
            return 0;
        }
        if (peek_kind(c) == TK_LBRACKET) {
            if (!parse_index(c, *out, out)) {
                return 0;
            }
        }
        if (peek_kind(c) == TK_DOT) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                     "expected an operator or the end of the expression", label,
                     "a tuple's elements are not tuples, so an element is selected once", 1);
            return 0;
        }
        return 1;
    }
    if (peek_kind(c) == TK_LBRACKET) {
        if (!parse_index(c, base, out)) {
            return 0;
        }
        if (peek_kind(c) == TK_DOT) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                     "expected an operator or the end of the expression", label,
                     "an array's elements are not tuples, so an element has no `.k`", 1);
            return 0;
        }
        return 1;
    }
    *out = base;
    return 1;
}

/* `[e0, e1, ...]` or a fill `[element; length]`. The list is not empty
   and holds at most MAX_ARRAY_ELEMENTS expressions. Elements share the
   argument table. A fill stores the element and the length literal. */
static int parse_array(Compiler *c, Token open, uint32_t *out) {
    uint32_t local_elems[MAX_ARRAY_ELEMENTS];
    uint32_t count = 0;
    Token close;
    int height = 1;
    if (!enter_nest(c, open.start, open.end)) {
        return 0;
    }
    if (peek_kind(c) == TK_RBRACKET) {
        char label[64];
        found_token_label(peek_kind(c), label, sizeof label);
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected an array element", label,
                 "an array has at least one element; Orange 2026 has no empty arrays", 1);
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
        uint32_t size = UINT32_MAX;
        int sized = 0;
        advance_token(c);
        length = peek_token(c);
        if (length.kind == TK_IDENT || length.kind == TK_LPAREN ||
            (length.kind == TK_INT && size_operator(c->at + 1 < c->ntokens ? c->tokens[c->at + 1].kind : TK_EOF))) {
            sized = 1;
            if (!parse_size_atom(c, "expected a fill length", COMPUTED_FILL_NOTE, &size)) {
                leave_nest(c);
                return 0;
            }
        } else if (length.kind != TK_INT) {
            char label[64];
            found_token_label(length.kind, label, sizeof label);
            add_diag(c, "ORC0101", length.start, length.end, "expected an array length after `;`", label,
                     "`[e; n]` is the array of n copies of e, such as `[0; 64]`", 1);
            leave_nest(c);
            return 0;
        } else {
            advance_token(c);
        }
        close = peek_token(c);
        if (close.kind != TK_RBRACKET) {
            if (size_operator(close.kind)) {
                char label[64];
                found_token_label(close.kind, label, sizeof label);
                add_diag(c, "ORC0101", close.start, close.end, "expected `]` after the array length", label,
                         COMPUTED_FILL_NOTE, 1);
            } else {
                add_diag(c, "ORC0101", close.start, close.end, "expected `]`", "unclosed fill literal", NULL, 1);
            }
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
        c->exprs[*out].lit_start = sized ? c->exprs[size].start : length.start;
        c->exprs[*out].lit_end = sized ? c->exprs[size].end : length.end;
        c->exprs[*out].size_expr = sized ? size : UINT32_MAX;
        c->exprs[*out].height = 1 + height_of(c, element);
        if (sized && height_of(c, size) + 1 > c->exprs[*out].height) {
            c->exprs[*out].height = 1 + height_of(c, size);
        }
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

static const char STEP_BLOCK_NOTE[] =
    "a loop's step holds `let` bindings, if any, and then the expression that gives the accumulator's next value";
static const char BRANCH_BLOCK_NOTE[] =
    "each branch of a conditional holds `let` bindings, if any, and then its value";

static int starts_let_binding(const Compiler *c) {
    TokenKind next;
    if (peek_kind(c) != TK_IDENT || !ident_token_is(c, peek_token(c), "let") || c->at + 1 >= c->ntokens) {
        return 0;
    }
    next = c->tokens[c->at + 1].kind;
    if (next == TK_IDENT) {
        return 1;
    }
    /* `let(x)` calls a function named `let`. A tuple pattern is
       `let (name: Type, ...)` or the rejected form `let (name, ...)`. */
    if (next == TK_LPAREN && c->at + 3 < c->ntokens) {
        TokenKind name = c->tokens[c->at + 2].kind;
        TokenKind after = c->tokens[c->at + 3].kind;
        return name == TK_IDENT && (after == TK_COLON || after == TK_COMMA);
    }
    return 0;
}

static int parse_pattern_name(Compiler *c, Local *local) {
    Token name = peek_token(c);
    DeclaredType declared;
    if (name.kind != TK_IDENT) {
        add_diag(c, "ORC0101", name.start, name.end, "expected a name for the pattern's next value", "expected a name",
                 PATTERN_NOTE, 1);
        return 0;
    }
    memset(local, 0, sizeof *local);
    local->site = UINT32_MAX;
    local->value = UINT32_MAX;
    local->name_start = name.start;
    local->name_end = name.end;
    local->name_at = name.start;
    local->name_end_at = name.end;
    advance_token(c);
    if (peek_kind(c) != TK_COLON) {
        char label[64];
        found_token_label(peek_kind(c), label, sizeof label);
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `:` and the type of the name", label,
                 PATTERN_NOTE, 1);
        return 0;
    }
    advance_token(c);
    if (!parse_type(c, &declared, 1)) {
        return 0;
    }
    store_declared(&declared, &local->type, &local->length, &local->type_ok, &local->length_bad, &local->type_start,
                   &local->type_end, &local->length_start, &local->length_end);
    return push_site(c, &declared, "binding type", &local->site);
}

/* Parse `(name: Type, ...)`. `dest[0].pat_len` is the count; later names have `pat_i` > 0. */
static int parse_tuple_pattern(Compiler *c, Local *dest, uint16_t room, uint16_t *count) {
    uint16_t n = 0;
    *count = 0;
    if (peek_kind(c) != TK_LPAREN) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `(`", "expected a tuple pattern",
                 PATTERN_NOTE, 1);
        return 0;
    }
    advance_token(c);
    for (;;) {
        if (n >= MAX_TUPLE) {
            resource_diag(c, "ORC0106", peek_token(c).start, peek_token(c).end,
                          "a tuple pattern names more than 16 values");
            return 0;
        }
        if (n >= room) {
            resource_diag(c, "ORC0106", peek_token(c).start, peek_token(c).end, "function exceeds the 256-binding limit");
            return 0;
        }
        if (!parse_pattern_name(c, &dest[n])) {
            return 0;
        }
        dest[n].pat_i = n;
        n++;
        if (peek_kind(c) == TK_COMMA) {
            advance_token(c);
            if (peek_kind(c) == TK_RPAREN) {
                break;
            }
            continue;
        }
        break;
    }
    if (n < 2) {
        char label[64];
        found_token_label(peek_kind(c), label, sizeof label);
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `,` and another name in the pattern",
                 label, PATTERN_NOTE, 1);
        if (peek_kind(c) == TK_RPAREN) {
            advance_token(c);
        }
        return 0;
    }
    if (peek_kind(c) != TK_RPAREN) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `,` or `)` after the pattern's element",
                 "unclosed pattern", PATTERN_NOTE, 1);
        return 0;
    }
    advance_token(c);
    dest[0].pat_len = n;
    dest[0].pat_i = 0;
    *count = n;
    return 1;
}

/* `let` bindings at the start of a step or a branch. They are not body bindings.
   Nested blocks append their bindings while a value is parsed, so this block's
   bindings are held aside and appended together once the block is complete. */
static int parse_block_lets(Compiler *c, const char *note, uint32_t *bind0, uint16_t *nbinds, int *height) {
    Local *pending = NULL;
    *bind0 = 0;
    *nbinds = 0;
    *height = 0;
    while (starts_let_binding(c)) {
        Token let_token = peek_token(c);
        Token name;
        Local *local;
        uint32_t value = UINT32_MAX;
        DeclaredType declared;
        int child;
        if (*nbinds >= MAX_BINDINGS) {
            add_diag(c, "ORC0106", let_token.start, let_token.end, "a block declares more than 256 bindings",
                     "block binding limit", "a step or a branch declares at most 256 bindings", 1);
            free(pending);
            return 0;
        }
        if (pending == NULL) {
            pending = calloc(MAX_BINDINGS, sizeof(Local));
            if (pending == NULL) {
                resource_diag(c, "ORC0106", let_token.start, let_token.end, "parser could not retain bindings");
                return 0;
            }
        }
        advance_token(c);
        if (peek_kind(c) == TK_LPAREN) {
            uint16_t npat = 0;
            uint16_t pat;
            uint32_t pattern_value = UINT32_MAX;
            if (!parse_tuple_pattern(c, &pending[*nbinds], (uint16_t)(MAX_BINDINGS - *nbinds), &npat)) {
                free(pending);
                return 0;
            }
            for (pat = 0; pat < npat; pat++) {
                pending[*nbinds + pat].block = 1;
            }
            if (c->parsing_func != NULL) {
                c->parsing_func->has_blocks = 1;
            }
            if (peek_kind(c) != TK_EQUAL) {
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `=`",
                         "expected the binding's value", note, 1);
                free(pending);
                return 0;
            }
            advance_token(c);
            if (!parse_expr(c, &pattern_value)) {
                free(pending);
                return 0;
            }
            if (peek_kind(c) != TK_SEMI) {
                char label[64];
                found_token_label(peek_kind(c), label, sizeof label);
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `;` after the bound expression",
                         label, "each binding ends with `;`; the block's last item is its value", 1);
                free(pending);
                return 0;
            }
            advance_token(c);
            pending[*nbinds].value = pattern_value;
            child = height_of(c, pattern_value);
            if (child > *height) {
                *height = child;
            }
            *nbinds = (uint16_t)(*nbinds + npat);
            continue;
        }
        name = peek_token(c);
        if (name.kind != TK_IDENT) {
            add_diag(c, "ORC0101", name.start, name.end, "expected a binding name", "expected a name after `let`", NULL,
                     1);
            free(pending);
            return 0;
        }
        local = &pending[*nbinds];
        memset(local, 0, sizeof *local);
        local->site = UINT32_MAX;
        local->block = 1;
        local->name_start = name.start;
        local->name_end = name.end;
        local->name_at = name.start;
        local->name_end_at = name.end;
        advance_token(c);
        if (peek_kind(c) != TK_COLON) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `:` and the binding's type", label,
                     "every binding states its type, as in `let t: Word[32] = x + y;`", 1);
            free(pending);
            return 0;
        }
        advance_token(c);
        if (!parse_type(c, &declared, 1)) {
            free(pending);
            return 0;
        }
        /* A nested block may have moved `pending` only if we realloc it.
           `pending` itself is a fixed calloc, so `local` stays valid. Nested
           blocks append to `block_locals`, not to `pending`. */
        store_declared(&declared, &local->type, &local->length, &local->type_ok, &local->length_bad, &local->type_start,
                       &local->type_end, &local->length_start, &local->length_end);
        if (!push_site(c, &declared, "binding type", &local->site)) {
            free(pending);
            return 0;
        }
        if (peek_kind(c) != TK_EQUAL) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `=`",
                     "expected the binding's value", note, 1);
            free(pending);
            return 0;
        }
        advance_token(c);
        if (!parse_expr(c, &value)) {
            free(pending);
            return 0;
        }
        if (peek_kind(c) != TK_SEMI) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `;` after the bound expression",
                     label, "each binding ends with `;`; the block's last item is its value", 1);
            free(pending);
            return 0;
        }
        advance_token(c);
        local->value = value;
        (*nbinds)++;
        if (c->parsing_func != NULL) {
            c->parsing_func->has_blocks = 1;
        }
        child = height_of(c, value);
        if (declared.has_mod) {
            int mod_height = height_of(c, declared.mod_expr);
            if (mod_height > child) {
                child = mod_height;
            }
        }
        if (child > *height) {
            *height = child;
        }
    }
    if (*nbinds > 0 && peek_kind(c) == TK_RBRACE) {
        char label[64];
        found_token_label(peek_kind(c), label, sizeof label);
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected a value after the last binding", label,
                 note, 1);
        free(pending);
        return 0;
    }
    if (*nbinds > 0) {
        if (!ensure_cap((void **)&c->block_locals, &c->block_local_cap, c->nblock_locals + *nbinds, sizeof(Local),
                        MAX_EXPRS)) {
            resource_diag(c, "ORC0106", peek_token(c).start, peek_token(c).end, "parser could not retain bindings");
            free(pending);
            return 0;
        }
        *bind0 = c->nblock_locals;
        memcpy(&c->block_locals[c->nblock_locals], pending, (size_t)(*nbinds) * sizeof(Local));
        c->nblock_locals += *nbinds;
    }
    free(pending);
    return 1;
}

/* `for i in a..b with s: T = start { step }`. Bounds are integer
   literals. The index and accumulator are in scope only in the step.
   The step may open with `let` bindings that are also in scope only there. */
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
    int bind_height = 0;
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
    c->loops[id].a_expr = UINT32_MAX;
    c->loops[id].b_expr = UINT32_MAX;
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
    if (bound_a.kind == TK_IDENT || bound_a.kind == TK_LPAREN) {
        uint32_t size = UINT32_MAX;
        if (!parse_size_atom(c, "expected the loop's first bound", COMPUTED_BOUND_NOTE, &size)) {
            return 0;
        }
        c->loops[id].a_sized = 1;
        c->loops[id].a_expr = size;
        c->loops[id].a_start = c->exprs[size].start;
        c->loops[id].a_end = c->exprs[size].end;
    } else if (bound_a.kind != TK_INT) {
        add_diag(c, "ORC0101", bound_a.start, bound_a.end, "expected a loop bound", "a loop bound is an integer literal",
                 NULL, 1);
        return 0;
    } else {
        c->loops[id].a_start = bound_a.start;
        c->loops[id].a_end = bound_a.end;
        advance_token(c);
    }
    if (peek_kind(c) != TK_DOTDOT) {
        if (size_operator(peek_kind(c))) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `..` between the loop's bounds",
                     label, COMPUTED_BOUND_NOTE, 1);
        } else {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `..`", "a loop range is `a..b`",
                     NULL, 1);
        }
        return 0;
    }
    advance_token(c);
    bound_b = peek_token(c);
    if (bound_b.kind == TK_IDENT || bound_b.kind == TK_LPAREN) {
        uint32_t size = UINT32_MAX;
        if (!parse_size_atom(c, "expected the loop's second bound", COMPUTED_BOUND_NOTE, &size)) {
            return 0;
        }
        c->loops[id].b_sized = 1;
        c->loops[id].b_expr = size;
        c->loops[id].b_start = c->exprs[size].start;
        c->loops[id].b_end = c->exprs[size].end;
    } else if (bound_b.kind != TK_INT) {
        char label[64];
        found_token_label(bound_b.kind, label, sizeof label);
        add_diag(c, "ORC0101", bound_b.start, bound_b.end, "expected the loop's second bound", label,
                 "a loop is written `for i in 0..n with s: Type = start { step }`", 1);
        return 0;
    } else {
        c->loops[id].b_start = bound_b.start;
        c->loops[id].b_end = bound_b.end;
        advance_token(c);
    }
    if (!ident_token_is(c, peek_token(c), "with")) {
        if (size_operator(peek_kind(c))) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `with` and the loop's accumulator",
                     label, COMPUTED_BOUND_NOTE, 1);
        } else {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `with` and the loop's accumulator",
                     label, "a loop is written `for i in 0..n with s: Type = start { step }`", 1);
        }
        return 0;
    }
    advance_token(c);
    acc = peek_token(c);
    if (acc.kind == TK_LPAREN) {
        Local names[MAX_TUPLE];
        uint16_t npat = 0;
        uint16_t pat;
        DeclaredType tuple_type;
        if (!parse_tuple_pattern(c, names, MAX_TUPLE, &npat)) {
            return 0;
        }
        c->loops[id].nacc = (uint8_t)npat;
        c->loops[id].acc_start = names[0].name_start;
        c->loops[id].acc_end = names[0].name_end;
        for (pat = 0; pat < npat; pat++) {
            c->loops[id].an_start[pat] = names[pat].name_start;
            c->loops[id].an_end[pat] = names[pat].name_end;
            c->loops[id].an_site[pat] = names[pat].site;
        }
        memset(&tuple_type, 0, sizeof tuple_type);
        tuple_type.kind = TY_TUPLE;
        tuple_type.ok = 1;
        tuple_type.is_tuple = 1;
        tuple_type.start = names[0].name_start;
        tuple_type.end = names[npat - 1].type_end;
        tuple_type.elem0 = names[0].site;
        tuple_type.elem_n = npat;
        /* Element sites are contiguous only when nothing else was pushed between them.
           Record them explicitly; resolution walks `an_site`, and `elem0` is the first. */
        if (!push_site(c, &tuple_type, "accumulator type", &c->loops[id].site)) {
            return 0;
        }
        c->loops[id].acc_type = TY_TUPLE;
        c->loops[id].acc_ok = 1;
        c->loops[id].type_start = tuple_type.start;
        c->loops[id].type_end = tuple_type.end;
    } else {
        if (acc.kind != TK_IDENT) {
            add_diag(c, "ORC0101", acc.start, acc.end, "expected an accumulator name", "expected a name after `with`",
                     NULL, 1);
            return 0;
        }
        c->loops[id].acc_start = acc.start;
        c->loops[id].acc_end = acc.end;
        advance_token(c);
        if (peek_kind(c) != TK_COLON) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `:` and the accumulator's type",
                     label, "every accumulator states its type, as in `with s: Word[32]^16 = x`", 1);
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
    memset(&c->open_loops[c->nopen], 0, sizeof c->open_loops[c->nopen]);
    c->open_loops[c->nopen].id = id;
    c->open_loops[c->nopen].index_start = c->loops[id].index_start;
    c->open_loops[c->nopen].index_end = c->loops[id].index_end;
    c->open_loops[c->nopen].acc_start = c->loops[id].acc_start;
    c->open_loops[c->nopen].acc_end = c->loops[id].acc_end;
    c->open_loops[c->nopen].nacc = c->loops[id].nacc;
    if (c->loops[id].nacc > 0) {
        uint8_t acc_index;
        for (acc_index = 0; acc_index < c->loops[id].nacc; acc_index++) {
            c->open_loops[c->nopen].acc_at[acc_index] = c->loops[id].an_start[acc_index];
            c->open_loops[c->nopen].acc_to[acc_index] = c->loops[id].an_end[acc_index];
        }
    }
    c->nopen++;
    {
        uint32_t bind0 = 0;
        uint16_t nbinds = 0;
        if (!parse_block_lets(c, STEP_BLOCK_NOTE, &bind0, &nbinds, &bind_height)) {
            c->nopen--;
            skip_open_braces(c, 1);
            leave_nest(c);
            return 0;
        }
        c->loops[id].bind0 = bind0;
        c->loops[id].nbinds = nbinds;
    }
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
        add_diag(c, "ORC0101", close.start, close.end, "expected `}` after the loop's step", "a loop step ends at `}`",
                 STEP_BLOCK_NOTE, 1);
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
    if (bind_height > height) {
        height = bind_height;
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

/* `base with [index] = value`. The new element is stored in callee. */
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
    if (peek_kind(c) == TK_DOTDOT) {
        uint32_t end_expr = UINT32_MAX;
        uint32_t range_start = 0;
        uint32_t range_end = 0;
        int bound_height = 0;
        Token close;
        if (!parse_slice_range(c, UINT32_MAX, &end_expr, &range_start, &range_end, &bound_height,
                               PARSER_SLICE_UPDATE_NOTE)) {
            leave_nest(c);
            return 0;
        }
        close = peek_token(c);
        if (close.kind != TK_RBRACKET) {
            char label[64];
            found_token_label(close.kind, label, sizeof label);
            add_diag(c, "ORC0101", close.start, close.end, "expected `]` after the slice", label,
                     PARSER_SLICE_UPDATE_NOTE, 1);
            leave_nest(c);
            return 0;
        }
        advance_token(c);
        if (peek_kind(c) != TK_EQUAL) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `=` after the updated slice",
                     "a slice update assigns the run", SLICE_UPDATE_NOTE, 1);
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
        c->exprs[*out].kind = EX_SLICE_UP;
        c->exprs[*out].left = base;
        c->exprs[*out].right = UINT32_MAX;
        c->exprs[*out].conv_site = end_expr;
        c->exprs[*out].callee = value;
        c->exprs[*out].start = c->exprs[base].start;
        c->exprs[*out].end = c->exprs[value].end;
        c->exprs[*out].lit_start = range_start;
        c->exprs[*out].lit_end = range_end;
        c->exprs[*out].op_start = with_token.start;
        c->exprs[*out].op_end = with_token.end;
        height = height_of(c, base);
        if (bound_height > height) {
            height = bound_height;
        }
        if (height_of(c, value) > height) {
            height = height_of(c, value);
        }
        c->exprs[*out].height = 1 + height;
        return note_height(c, *out);
    }
    if (!parse_expr(c, &index)) {
        leave_nest(c);
        return 0;
    }
    if (peek_kind(c) == TK_DOTDOT) {
        uint32_t end_expr = UINT32_MAX;
        uint32_t range_start = 0;
        uint32_t range_end = 0;
        int bound_height = 0;
        Token close;
        if (!parse_slice_range(c, index, &end_expr, &range_start, &range_end, &bound_height, PARSER_SLICE_UPDATE_NOTE)) {
            leave_nest(c);
            return 0;
        }
        close = peek_token(c);
        if (close.kind != TK_RBRACKET) {
            char label[64];
            found_token_label(close.kind, label, sizeof label);
            add_diag(c, "ORC0101", close.start, close.end, "expected `]` after the slice", label,
                     PARSER_SLICE_UPDATE_NOTE, 1);
            leave_nest(c);
            return 0;
        }
        advance_token(c);
        if (peek_kind(c) != TK_EQUAL) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `=` after the updated slice",
                     "a slice update assigns the run", SLICE_UPDATE_NOTE, 1);
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
        c->exprs[*out].kind = EX_SLICE_UP;
        c->exprs[*out].left = base;
        c->exprs[*out].right = index;
        c->exprs[*out].conv_site = end_expr;
        c->exprs[*out].callee = value;
        c->exprs[*out].start = c->exprs[base].start;
        c->exprs[*out].end = c->exprs[value].end;
        c->exprs[*out].lit_start = range_start;
        c->exprs[*out].lit_end = range_end;
        c->exprs[*out].op_start = with_token.start;
        c->exprs[*out].op_end = with_token.end;
        height = height_of(c, base);
        if (bound_height > height) {
            height = bound_height;
        }
        if (height_of(c, value) > height) {
            height = height_of(c, value);
        }
        c->exprs[*out].height = 1 + height;
        return note_height(c, *out);
    }
    if (peek_kind(c) != TK_RBRACKET) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `]`", "unclosed update index", NULL, 1);
        leave_nest(c);
        return 0;
    }
    advance_token(c);
    if (peek_kind(c) != TK_EQUAL) {
        char label[64];
        found_token_label(peek_kind(c), label, sizeof label);
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `=` after the updated index", label,
                 "an update is written `x with [i] = value`, or `x with [i][j] = value` for an element of a row", 1);
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

/* `if c { a } else { b }`, including an else-if chain stored as one
   EX_COND. Both branches are parsed; only the chosen branch is evaluated. */
static int parse_conditional(Compiler *c, Token if_token, uint32_t *out) {
    CondArm *local = NULL;
    uint32_t count = 0;
    uint32_t cap = 0;
    uint32_t otherwise = UINT32_MAX;
    uint32_t else_bind0 = 0;
    uint16_t else_nbinds = 0;
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
        {
            uint32_t bind0 = 0;
            uint16_t nbinds = 0;
            int bind_height = 0;
            if (!parse_block_lets(c, BRANCH_BLOCK_NOTE, &bind0, &nbinds, &bind_height)) {
                skip_open_braces(c, 1);
                free(local);
                leave_nest(c);
                return 0;
            }
            if (bind_height > height) {
                height = bind_height;
            }
            if (!parse_expr(c, &value)) {
                skip_open_braces(c, 1);
                free(local);
                leave_nest(c);
                return 0;
            }
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
            local[count].bind0 = bind0;
            local[count].nbinds = nbinds;
        }
        if (peek_kind(c) != TK_RBRACE) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `}` after the value", label,
                     BRANCH_BLOCK_NOTE, 1);
            skip_open_braces(c, 1);
            free(local);
            leave_nest(c);
            return 0;
        }
        advance_token(c);
        if (!ident_token_is(c, peek_token(c), "else")) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                     "expected `else` and the value when the condition is false", label,
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
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `{` or `if` after `else`", label,
                     "a conditional is written `if condition { value } else { other value }`", 1);
            free(local);
            leave_nest(c);
            return 0;
        }
        advance_token(c);
        {
            int bind_height = 0;
            if (!parse_block_lets(c, BRANCH_BLOCK_NOTE, &else_bind0, &else_nbinds, &bind_height)) {
                skip_open_braces(c, 1);
                free(local);
                leave_nest(c);
                return 0;
            }
            if (bind_height > height) {
                height = bind_height;
            }
        }
        if (!parse_expr(c, &otherwise)) {
            skip_open_braces(c, 1);
            free(local);
            leave_nest(c);
            return 0;
        }
        if (peek_kind(c) != TK_RBRACE) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `}` after the value", label,
                     BRANCH_BLOCK_NOTE, 1);
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
    c->exprs[*out].else_bind0 = else_bind0;
    c->exprs[*out].else_nbinds = else_nbinds;
    c->exprs[*out].start = if_token.start;
    c->exprs[*out].end = close.end;
    c->exprs[*out].height = 1 + height;
    c->ncond_arms += count;
    return note_height(c, *out);
}

/* Operand: literal, name, call, parenthesized expression, array literal,
   fill, loop, conditional, byte string, hex string, unary minus, or
   bitwise complement. A name, call, or accumulator may then take one
   index or slice, and an array may take one `with [` update. A minus
   immediately before an integer is that literal's sign, matching the
   S3a body rule. `for` starts a loop only when the next token is an
   identifier. `if` starts a conditional only in that same position. */
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
            {
                uint32_t size0 = 0;
                uint8_t nsize = 0;
                int child_height = 0;
                if (peek_kind(c) == TK_LBRACKET) {
                    if (!enter_nest(c, module_name.start, module_name.end)) {
                        return 0;
                    }
                    if (!parse_call_sizes(c, &size0, &nsize, &child_height)) {
                        leave_nest(c);
                        return 0;
                    }
                    leave_nest(c);
                    if (peek_kind(c) != TK_LPAREN) {
                        char label[64];
                        found_token_label(peek_kind(c), label, sizeof label);
                        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `(` after the sizes",
                                 label, SIZED_CALL_NOTE, 1);
                        return 0;
                    }
                } else if (peek_kind(c) != TK_LPAREN) {
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
                c->exprs[*out].size0 = size0;
                c->exprs[*out].nsize = nsize;
                c->exprs[*out].height = 1 + child_height;
                for (uint16_t index = 0; index < argc; index++) {
                    int child = height_of(c, c->args[arg0 + index]);
                    if (1 + child > c->exprs[*out].height) {
                        c->exprs[*out].height = 1 + child;
                    }
                }
                if (!note_height(c, *out)) {
                    return 0;
                }
                return parse_suffix(c, *out, out);
            }
        }
        if (peek_kind(c) == TK_LBRACKET && starts_sized_call(c)) {
            uint32_t arg0 = 0;
            uint16_t argc = 0;
            uint32_t size0 = 0;
            uint8_t nsize = 0;
            int child_height = 0;
            Token close;
            if (!enter_nest(c, name.start, name.end)) {
                return 0;
            }
            if (!parse_call_sizes(c, &size0, &nsize, &child_height)) {
                leave_nest(c);
                return 0;
            }
            if (peek_kind(c) != TK_LPAREN) {
                char label[64];
                found_token_label(peek_kind(c), label, sizeof label);
                leave_nest(c);
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `(` after the sizes", label,
                         SIZED_CALL_NOTE, 1);
                return 0;
            }
            advance_token(c);
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
            c->exprs[*out].size0 = size0;
            c->exprs[*out].nsize = nsize;
            c->exprs[*out].height = 1 + child_height;
            for (uint16_t index = 0; index < argc; index++) {
                int child = height_of(c, c->args[arg0 + index]);
                if (1 + child > c->exprs[*out].height) {
                    c->exprs[*out].height = 1 + child;
                }
            }
            if (!note_height(c, *out)) {
                return 0;
            }
            return parse_suffix(c, *out, out);
        }
        if (peek_kind(c) != TK_LPAREN) {
            for (open_index = c->nopen - 1; open_index >= 0; open_index--) {
                OpenLoop *open = &c->open_loops[open_index];
                int is_index = same_span(c, open->index_start, open->index_end, name.start, name.end);
                int is_acc = 0;
                uint16_t acc_elem = 0;
                if (open->nacc == 0) {
                    is_acc = same_span(c, open->acc_start, open->acc_end, name.start, name.end);
                } else {
                    uint8_t acc_index;
                    for (acc_index = 0; acc_index < open->nacc; acc_index++) {
                        if (same_span(c, open->acc_at[acc_index], open->acc_to[acc_index], name.start, name.end)) {
                            is_acc = 1;
                            acc_elem = acc_index;
                            break;
                        }
                    }
                }
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
                if (!is_index && open->nacc > 0) {
                    c->exprs[*out].is_proj = 1;
                    c->exprs[*out].name_index = acc_elem;
                    c->exprs[*out].proj_pos = acc_elem;
                }
                return parse_suffix(c, *out, out);
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
            return parse_suffix(c, *out, out);
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
        return parse_suffix(c, *out, out);
    }
    if (token.kind == TK_STRING || token.kind == TK_HEX) {
        advance_token(c);
        if (!new_expr(c, out)) {
            return 0;
        }
        c->exprs[*out].kind = EX_BYTES;
        c->exprs[*out].op = token.kind;
        c->exprs[*out].start = token.start;
        c->exprs[*out].end = token.end;
        c->exprs[*out].height = 1;
        if (!note_height(c, *out)) {
            return 0;
        }
        if (peek_kind(c) == TK_LBRACKET || peek_kind(c) == TK_DOT) {
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                     "expected an operator or the end of the expression", label,
                     "a byte string is not indexed or sliced where it is written; bind it with `let` to select from it",
                     1);
            return 0;
        }
        return 1;
    }
    if (token.kind == TK_LBRACKET) {
        advance_token(c);
        return parse_array(c, token, out);
    }
    if (token.kind == TK_LPAREN) {
        uint32_t elems[MAX_TUPLE];
        uint32_t count = 0;
        Token close;
        int height = 1;
        uint32_t index;
        advance_token(c);
        if (!enter_nest(c, token.start, token.end)) {
            return 0;
        }
        if (!parse_expr(c, &elems[0])) {
            leave_nest(c);
            return 0;
        }
        count = 1;
        if (peek_kind(c) != TK_COMMA) {
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
            c->exprs[*out].left = elems[0];
            c->exprs[*out].start = token.start;
            c->exprs[*out].end = close.end;
            c->exprs[*out].height = 1 + height_of(c, elems[0]);
            return note_height(c, *out);
        }
        for (;;) {
            advance_token(c);
            if (peek_kind(c) == TK_RPAREN) {
                if (count < 2) {
                    char label[64];
                    found_token_label(peek_kind(c), label, sizeof label);
                    add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected another element after `,`",
                             label,
                             "a tuple is written `(a, b)` with two through 16 elements; `(a)` without a comma is a group",
                             1);
                    leave_nest(c);
                    return 0;
                }
                break;
            }
            if (count >= MAX_TUPLE) {
                resource_diag(c, "ORC0106", peek_token(c).start, peek_token(c).end, "a tuple has more than 16 elements");
                leave_nest(c);
                return 0;
            }
            if (!parse_expr(c, &elems[count])) {
                leave_nest(c);
                return 0;
            }
            count++;
            if (peek_kind(c) != TK_COMMA) {
                break;
            }
        }
        leave_nest(c);
        close = peek_token(c);
        if (close.kind != TK_RPAREN) {
            add_diag(c, "ORC0101", close.start, close.end, "expected `,` or `)` after the tuple's element",
                     "unclosed tuple",
                     "a tuple is written `(a, b)` with two through 16 elements; `(a)` without a comma is a group", 1);
            return 0;
        }
        advance_token(c);
        if (!ensure_cap((void **)&c->args, &c->arg_cap, c->nargs + count, sizeof(uint32_t), MAX_EXPRS)) {
            resource_diag(c, "ORC0106", token.start, close.end, "parser could not allocate tuple storage");
            return 0;
        }
        if (!new_expr(c, out)) {
            return 0;
        }
        c->exprs[*out].kind = EX_TUPLE;
        c->exprs[*out].start = token.start;
        c->exprs[*out].end = close.end;
        c->exprs[*out].arg0 = c->nargs;
        c->exprs[*out].argc = (uint16_t)count;
        memcpy(c->args + c->nargs, elems, (size_t)count * sizeof(uint32_t));
        c->nargs += count;
        for (index = 0; index < count; index++) {
            int child = height_of(c, elems[index]);
            if (1 + child > height) {
                height = 1 + child;
            }
        }
        c->exprs[*out].height = height;
        return note_height(c, *out);
    }
    add_diag(c, "ORC0101", token.start, token.end == token.start ? token.end + 0 : token.end,
             "expected an expression", "expected an operand", NULL, 1);
    return 0;
}

/* One expression. `as`, shifts, comparisons, and Euclidean `/` and `%`
   take one right-hand operand and do not chain. In group 1, products fold
   first and then `+` and `-` associate to the left. `&`, `|`, `^`, `&&`,
   `||`, and `++` associate to the left only with the same operator. A
   following operator from another group is rejected. */
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
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `:` after the parameter name", label,
                     "parameters are written `name: Type`", 1);
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
    if (peek_kind(c) == TK_LPAREN) {
        Local names[MAX_TUPLE];
        uint16_t npat = 0;
        uint16_t pat;
        if (!parse_tuple_pattern(c, names, MAX_TUPLE, &npat)) {
            return 0;
        }
        if ((uint32_t)func->nlocals + npat > MAX_BINDINGS) {
            resource_diag(c, "ORC0106", let_token.start, let_token.end, "function exceeds the 256-binding limit");
            return 0;
        }
        if (!ensure_cap((void **)&c->locals, &c->local_cap, c->nlocals + npat, sizeof(Local), MAX_EXPRS)) {
            resource_diag(c, "ORC0106", let_token.start, let_token.end, "parser could not retain bindings");
            return 0;
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
            char label[64];
            found_token_label(peek_kind(c), label, sizeof label);
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `;` after the bound expression",
                     label, "each binding ends with `;`; the body's last item is its result expression", 1);
            return 0;
        }
        advance_token(c);
        names[0].value = value;
        for (pat = 0; pat < npat; pat++) {
            names[pat].block = 0;
        }
        memcpy(&c->locals[c->nlocals], names, (size_t)npat * sizeof(Local));
        c->nlocals += npat;
        func->nlocals = (uint16_t)(func->nlocals + npat);
        return 1;
    }
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
        char label[64];
        found_token_label(peek_kind(c), label, sizeof label);
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `:` and the binding's type", label,
                 "every binding states its type, as in `let t: Word[32] = x + y;`", 1);
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
        char label[64];
        found_token_label(peek_kind(c), label, sizeof label);
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `;` after the bound expression", label,
                 "each binding ends with `;`; the body's last item is its result expression", 1);
        return 0;
    }
    advance_token(c);
    local->value = value;
    c->nlocals++;
    func->nlocals++;
    return 1;
}

/* Result type, optional let bindings, and the result expression.
   inside_params_done is unused; callers pass 1 after the parameter
   list has already been closed. */
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
    while (starts_let_binding(c)) {
        if (!parse_binding(c, func)) {
            skip_function_body(c, 1);
            return 1;
        }
    }
    if (peek_kind(c) == TK_RBRACE) {
        char label[64];
        found_token_label(peek_kind(c), label, sizeof label);
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                 "expected a result expression after the last binding", label,
                 "a typed `spec` body ends with the expression that gives its value", 1);
        advance_token(c);
        return 1;
    }
    if (!parse_expr(c, &func->body)) {
        skip_function_body(c, 1);
        return 1;
    }
    if (peek_kind(c) != TK_RBRACE) {
        Token extra = peek_token(c);
        if (extra.kind == TK_STRING && func->body != UINT32_MAX && c->exprs[func->body].kind == EX_NAME &&
            span_is(c, c->exprs[func->body].name_start, c->exprs[func->body].name_end, "hex")) {
            char label[64];
            found_token_label(extra.kind, label, sizeof label);
            add_diag(c, "ORC0101", extra.start, extra.end, "expected `}` after the body expression", label,
                     "a hex string's quote follows `hex` directly, with no space, as in `hex\"00 1f a0\"`", 1);
        } else {
            add_diag(c, "ORC0101", extra.start, extra.end, "expected `}`", "extra tokens after the result expression",
                     NULL, 1);
        }
        skip_function_body(c, 1);
        return 1;
    }
    advance_token(c);
    return 1;
}

static int parse_size_params(Compiler *c, Func *func) {
    advance_token(c);
    if (peek_kind(c) == TK_RBRACKET) {
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected a size parameter", "expected a name",
                 SIZE_PARAMETER_NOTE, 1);
        return 0;
    }
    for (;;) {
        Token name;
        Token first;
        Token second;
        if (peek_kind(c) != TK_IDENT) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected a size parameter",
                     "expected a name", SIZE_PARAMETER_NOTE, 1);
            return 0;
        }
        name = peek_token(c);
        advance_token(c);
        if (!span_is(c, peek_token(c).start, peek_token(c).end, "in")) {
            {
                char label[64];
                found_token_label(peek_kind(c), label, sizeof label);
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `in` after the size's name",
                         label, SIZE_PARAMETER_NOTE, 1);
            }
            return 0;
        }
        advance_token(c);
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
            {
                char label[64];
                found_token_label(peek_kind(c), label, sizeof label);
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected the size's second bound", label,
                         SIZE_PARAMETER_NOTE, 1);
            }
            return 0;
        }
        second = peek_token(c);
        advance_token(c);
        if (func->nsizes >= MAX_SIZES) {
            add_diag(c, "ORC0101", name.start, second.end, "a function has at most 4 size parameters",
                     "one size parameter too many", SIZE_PARAMETER_NOTE, 1);
            return 0;
        }
        func->sz_name0[func->nsizes] = name.start;
        func->sz_name1[func->nsizes] = name.end;
        func->sz_span0[func->nsizes] = name.start;
        func->sz_span1[func->nsizes] = second.end;
        func->sz_a0[func->nsizes] = first.start;
        func->sz_a1[func->nsizes] = first.end;
        func->sz_b0[func->nsizes] = second.start;
        func->sz_b1[func->nsizes] = second.end;
        func->nsizes++;
        if (peek_kind(c) == TK_COMMA) {
            advance_token(c);
            continue;
        }
        if (peek_kind(c) == TK_RBRACKET) {
            advance_token(c);
            return 1;
        }
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `,` or `]` after the size parameter",
                 "expected `,` or `]`", SIZE_PARAMETER_NOTE, 1);
        return 0;
    }
}

/* spec or impl. An impl and a parameterless spec with a brace body are
   empty in this slice. A spec with parameters, or a parameterless spec
   with an arrow, is typed. */
static int parse_function(Compiler *c) {
    Token kind = peek_token(c);
    Token name;
    Func *func;
    int is_impl = kind.kind == TK_IMPL;
    c->parsing_func = NULL;
    if (!ensure_cap((void **)&c->funcs, &c->func_cap, c->nfuncs + 1, sizeof(Func), MAX_EXPRS)) {
        resource_diag(c, "ORC0106", kind.start, kind.end, "parser could not retain functions");
        return 0;
    }
    func = &c->funcs[c->nfuncs];
    memset(func, 0, sizeof *func);
    c->parsing_func = func;
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
    if (peek_kind(c) == TK_LBRACKET) {
        if (is_impl) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "`impl` functions have no size parameters",
                     "expected `(`", SIZE_PARAMETER_NOTE, 1);
            skip_function_body(c, 0);
            c->nfuncs++;
            return 1;
        }
        if (!parse_size_params(c, func)) {
            skip_function_body(c, 0);
            c->nfuncs++;
            return 1;
        }
    }
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
            Token first = peek_token(c);
            uint32_t end = first.end;
            uint32_t at = c->at;
            while (at < c->ntokens && c->tokens[at].kind != TK_RPAREN && c->tokens[at].kind != TK_LBRACE &&
                   c->tokens[at].kind != TK_EOF) {
                end = c->tokens[at].end;
                at++;
            }
            add_diag(c, "ORC0101", first.start, end, "`impl` functions have an empty parameter list",
                     "parameters are allowed only on typed `spec` functions",
                     "keep the legacy `impl name() {}` form until implementation semantics are defined", 2);
            skip_function_body(c, 0);
            c->nfuncs++;
            return 1;
        }
        advance_token(c);
        if (peek_kind(c) != TK_LBRACE) {
            add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                     "typed bodies are allowed only on `spec` functions", "an `impl` function cannot have a typed body",
                     "keep the legacy `impl name() {}` form until implementation semantics are defined", 2);
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
            if (func->nsizes > 0) {
                add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end,
                         "a `spec` with size parameters needs a result type", "expected `->`", SIZE_PARAMETER_NOTE, 1);
                skip_function_body(c, 0);
                c->nfuncs++;
                return 1;
            }
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
        add_diag(c, "ORC0101", peek_token(c).start, peek_token(c).end, "expected `->` after the parameter list",
                 "a `spec` with parameters or sizes needs a result type and a body expression",
                 "write `spec name(x: Type) -> Type { expression }`", 2);
        skip_function_body(c, 0);
        return 1;
    }
    return parse_typed_tail(c, func, 1);
}

/* `use name;` before any function. The name is recorded; the sibling
   file is loaded later. More than 64 uses is a resource diagnostic. */
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

/* `type Name = T;` after `use` and before functions. The name is installed
   when prepare_types accepts it. More than 64 declarations is a resource
   diagnostic. */
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

/* edition 2026; module name { use declarations, then type declarations,
   then spec and impl functions }. A `use` after a `type` or a function,
   or a `type` after a function, is rejected. Always returns 1. Syntax
   errors and resource failures are recorded on the Compiler; they do not
   change this return value. */
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

/* --- Checker --------------------------------------------------------------- */

/* Names, local and qualified calls, operators, comparisons, Euclidean division, conditionals,
   conversions, arrays, indices, loops, updates, fills, and the call graph.
   check_expr returns 0 when analysis must stop
   (resource limit or the semantic diagnostic cap) and 1 when the node was
   visited, including when a type error was reported. find_leaf returns 1
   when a typed leaf was found, 0 when the leaf is an untyped literal, and
   -1 when the leaf is missing or already rejected. *length is 0 for a
   scalar. *silent suppresses a second diagnostic. base_type reads the
   type of a name, call, accumulator, or loop without walking through
   operators. A word index ranges over its type. An Int index is built
   from integer literals, loop indices, words converted with `as Int`,
   arithmetic, and conditionals, and is proved in range before evaluation.
   A literal that does not fit the bit budget is ORC0205, not a range
   error. A step or a branch may open with `let` bindings that stay in
   that block. A cross-module residue compares the modulus values.
   A rejected `!`, `&&`, or `||` is ORC0215 and does not typecheck its
   operands. A rejected result type does not typecheck the body.
   A bad alias target is reported once, at the declaration: ORC0204 for
   a Word width and ORC0221 for an array length. A use does not report
   that target again.
   A tuple is 2 through 16 scalars or arrays. `.k` selects one element.
   A pattern name that repeats the loop index is ORC0219, and a pattern
   name used outside the loop is ORC0211. Order on an array or a tuple
   is ORC0215. `==` and `!=` of a written-out array or tuple with no type
   of its own is ORC0227. A byte string is Word[8]^n. ++ joins arrays. A slice's
   bounds are an affine form of integer literals and loop indices, with
   one fixed positive length, proved inside the array before evaluation.
   A runtime or non-linear bound is ORC0226, a varying length is ORC0236,
   and an out-of-range step is ORC0223. A non-printable or non-ASCII byte
   is ORC0235, an empty string is ORC0221, and a join past 256 bytes is
   ORC0222. A sized function is instantiated for each value in range, at
   most 256 instances. Instances are checked from the first value, and
   the first diagnostic ends that walk and names that instance, as in
   `last[1]` or `none[0]`. A sized length outside 1 through 256 is
   ORC0221, and its note says 1 through 65536. Size `/` and `%` are
   Euclidean. A call resolves one instance: an out-of-range or unmatched
   size is ORC0238, and a wrong count or an ambiguous fit is ORC0239. A
   cycle among instances is ORC0217 and prints the chain, as in
   `swap[1] -> swap[2] -> swap[1]`. */

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
        snprintf(message, sizeof message, "an array length must be a decimal integer from 1 through 65536");
        add_diag(c, "ORC0221", length_start, length_end, message, "unsupported array length",
                 "write the length in decimal without leading zeros, as in `Word[32]^16`", 2);
        return;
    }
    reject_type(c, type, 0, start, end);
}

static int signature_is_usable(const Compiler *c, uint32_t func_index) {
    const Func *func = &c->funcs[func_index];
    uint16_t param;
    if (!func->typed) {
        return 0;
    }
    if (func->nsizes > 0) {
        return func->sizes_ok && func->ninst > 0;
    }
    if (!func->result_ok) {
        return 0;
    }
    for (param = 0; param < func->nparams; param++) {
        if (!c->params[func->param0 + param].type_ok) {
            return 0;
        }
    }
    return 1;
}

/* The unique typed spec with this name in `mod`. The name bytes come from
   `text`, which may be another module's source. An empty spec or an impl
   sets the out-flags and does not count as that function. */
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

/* A local call looks up the function in this module. */
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

/* A bare call stays in this module. `m::f` requires a `use m` whose
   target module declares f. A self-qualified name is not a use. */
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

static int size_slot_of(const Compiler *c, uint32_t func_index, uint32_t start, uint32_t end, uint8_t *slot);

/* Parameters, then bindings already closed by `;`, then a later binding
   of the same spelling (NAME_EARLY). Duplicates are skipped. */
static void resolve_name(Compiler *c, uint32_t func_index, uint32_t locals_in_scope, uint32_t start, uint32_t end,
                         NameRes *res, uint16_t *slot, TypeKind *type, uint32_t *length, int *type_ok,
                         uint32_t *abs_index) {
    const Func *func = &c->funcs[func_index];
    uint16_t index;
    int frame;
    *res = NAME_MISSING;
    *slot = 0;
    *type = TY_NONE;
    *length = 0;
    *type_ok = 0;
    if (abs_index != NULL) {
        *abs_index = 0;
    }
    {
        uint8_t size_index = 0;
        if (size_slot_of(c, func_index, start, end, &size_index)) {
            *res = NAME_SIZE;
            *slot = size_index;
            *type = TY_INT;
            *length = 0;
            *type_ok = 1;
            return;
        }
    }
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
            /* The live parameter record is whatever the last checked instance
               wrote. A call resolved while another instance is running needs
               that instance's length. */
            if (c->cur_func == func_index && c->cur_inst != UINT32_MAX && c->cur_inst < c->ninstances) {
                const Instance *inst = &c->instances[c->cur_inst];
                if (inst->func == func_index && inst->param0 + index < c->niparams) {
                    const InstParam *shape = &c->iparams[inst->param0 + index];
                    *type = shape->type;
                    *length = shape->length;
                    *type_ok = shape->type_ok;
                    *res = shape->type_ok ? NAME_PARAM : NAME_BAD;
                }
            }
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
                if (abs_index != NULL) {
                    *abs_index = func->local0 + local_index;
                }
                return;
            }
        }
    }
    for (frame = c->nframes - 1; frame >= 0; frame--) {
        const BlockFrame *block = &c->frames[frame];
        uint16_t bind;
        for (bind = 0; bind < block->visible; bind++) {
            const Local *local = &c->block_locals[block->bind0 + bind];
            if (local->duplicate) {
                continue;
            }
            if (!same_span(c, local->name_start, local->name_end, start, end)) {
                continue;
            }
            *res = local->type_ok ? NAME_BLOCK : NAME_BAD;
            *type = local->type;
            *length = local->length;
            *type_ok = local->type_ok;
            if (abs_index != NULL) {
                *abs_index = block->bind0 + bind;
            }
            return;
        }
    }
    for (frame = c->nframes - 1; frame >= 0; frame--) {
        const BlockFrame *block = &c->frames[frame];
        uint16_t bind;
        for (bind = block->visible; bind < block->nbinds; bind++) {
            const Local *local = &c->block_locals[block->bind0 + bind];
            if (!same_span(c, local->name_start, local->name_end, start, end)) {
                continue;
            }
            *res = NAME_BLOCK_EARLY;
            if (abs_index != NULL) {
                *abs_index = block->bind0 + bind;
            }
            return;
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
static int check_as_tuple(Compiler *c, uint32_t index, uint32_t tup0, uint16_t tup_n, uint32_t func_index,
                          uint32_t locals_in_scope);
static int find_leaf(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, TypeKind *type,
                     uint32_t *length, uint32_t *leaf, int *silent);

static int decode_literal(Compiler *c, const Expr *expr, Big *out) {
    return big_from_digits(&c->arena, c->text + expr->lit_start, (size_t)(expr->lit_end - expr->lit_start),
                           expr->negative, out);
}

static uint64_t word_maximum(TypeKind type);
static void spell_type(const Compiler *owner, char *buffer, size_t cap, TypeKind type, uint32_t length, uint16_t mod,
                       uint32_t tup0, uint16_t tup_n);

static void check_literal(Compiler *c, const Expr *expr, TypeKind expected, uint16_t expected_mod) {
    Big value = big_zero();
    if (!decode_literal(c, expr, &value)) {
        add_diag(c, "ORC0205", expr->lit_start, expr->lit_end, "integer magnitude exceeds the 16384-significant-bit limit",
                 "exact integer is too large for this semantic fragment",
                 "the literal is rejected rather than truncated or approximated", 2);
        return;
    }
    if (expected == TY_INT) {
        return;
    }
    if (expected == TY_MOD) {
        Big magnitude = value;
        const Big *modulus;
        char type_text[96];
        char message[160];
        magnitude.negative = 0;
        if (expected_mod == 0 || expected_mod >= c->nmoduli) {
            return;
        }
        modulus = &c->moduli[expected_mod];
        if (big_cmp(&magnitude, modulus) >= 0) {
            spell_type(c, type_text, sizeof type_text, TY_MOD, 0, expected_mod, 0, 0);
            snprintf(message, sizeof message, "literal is outside the range of `%s`", type_text);
            add_diag(c, "ORC0207", expr->lit_start, expr->lit_end, message,
                     "the literal's magnitude is not less than the modulus",
                     "a literal of `Mod[m]` has a magnitude n less than m, and `-n` stands for m - n; residues do not "
                     "reduce out-of-range literals",
                     2);
        }
        return;
    }
    if (expected == TY_BOOL) {
        add_diag(c, "ORC0214", expr->start, expr->end, "an integer literal cannot have type `Bool`", "expected `Bool`",
                 "the `Bool` values are written `true` and `false`", 2);
        return;
    }
    {
        char type_text[96];
        char maximum[32];
        char message[160];
        char label[128];
        spell_type(c, type_text, sizeof type_text, expected, 0, 0, 0, 0);
        snprintf(maximum, sizeof maximum, "%llu", (unsigned long long)word_maximum(expected));
        if (expr->negative) {
            snprintf(message, sizeof message, "`%s` literals cannot be negative", type_text);
            snprintf(label, sizeof label, "negative value is outside the range 0 through %s", maximum);
            add_diag(c, "ORC0206", expr->start, expr->end, message, label,
                     "fixed-width words do not wrap or coerce negative integers", 2);
            return;
        }
        if (big_bits(&value) > (uint32_t)type_width(expected)) {
            snprintf(message, sizeof message, "literal is outside the range of `%s`", type_text);
            snprintf(label, sizeof label, "expected a value from 0 through %s", maximum);
            add_diag(c, "ORC0207", expr->lit_start, expr->lit_end, message, label,
                     "fixed-width words do not truncate or wrap out-of-range integers", 2);
        }
    }
}

static const Big *modulus_at(const Compiler *c, uint16_t index);
static int intern_modulus(Compiler *c, const Big *value, uint16_t *out);

/* Each module keeps its own modulus table, so equal indexes can name
   different rings. Copy the foreign modulus value into this table and
   compare the values. A mismatch is ORC0214. */
static int adopt_modulus(Compiler *c, const Compiler *owner, uint16_t foreign, uint16_t *local) {
    const Big *value;
    if (owner == NULL || owner == c) {
        *local = foreign;
        return 1;
    }
    value = modulus_at(owner, foreign);
    if (value == NULL) {
        *local = 0;
        return 1;
    }
    return intern_modulus(c, value, local);
}

static int lookup_call(Compiler *c, Expr *expr, Compiler *target, uint32_t callee, int report, uint32_t caller_func,
                      uint32_t locals, uint32_t *inst_id);

static int base_type(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, TypeKind *type,
                     uint32_t *length, int *silent) {
    const Expr *expr = &c->exprs[index];
    *silent = 0;
    *type = TY_NONE;
    *length = 0;
    c->leaf_mod = 0;
    c->leaf_tup0 = 0;
    c->leaf_tup_n = 0;
    c->leaf_owner = c;
    if (expr->kind == EX_NAME) {
        NameRes res;
        uint16_t slot = 0;
        int type_ok = 0;
        uint32_t abs_index = 0;
        resolve_name(c, func_index, locals_in_scope, expr->name_start, expr->name_end, &res, &slot, type, length,
                     &type_ok, &abs_index);
        if (res == NAME_SIZE) {
            *type = TY_INT;
            *length = 0;
            return 1;
        }
        if (res == NAME_BAD) {
            *silent = 1;
            return -1;
        }
        if (res == NAME_PARAM || res == NAME_LOCAL || res == NAME_BLOCK) {
            if (*type == TY_MOD) {
                c->leaf_mod = res == NAME_PARAM ? c->params[c->funcs[func_index].param0 + slot].mod_index
                                : res == NAME_BLOCK ? c->block_locals[abs_index].mod_index
                                                    : c->locals[c->funcs[func_index].local0 + slot].mod_index;
            }
            if (*type == TY_TUPLE) {
                if (res == NAME_PARAM) {
                    const Param *param = &c->params[c->funcs[func_index].param0 + slot];
                    c->leaf_tup0 = param->tup0;
                    c->leaf_tup_n = param->tup_n;
                } else if (res == NAME_BLOCK) {
                    c->leaf_tup0 = c->block_locals[abs_index].tup0;
                    c->leaf_tup_n = c->block_locals[abs_index].tup_n;
                } else {
                    c->leaf_tup0 = c->locals[c->funcs[func_index].local0 + slot].tup0;
                    c->leaf_tup_n = c->locals[c->funcs[func_index].local0 + slot].tup_n;
                }
            }
            return 1;
        }
        return 0;
    }
    if (expr->kind == EX_PROJECT) {
        int state = base_type(c, expr->left, func_index, locals_in_scope, type, length, silent);
        uint32_t tup0 = c->leaf_tup0;
        uint16_t tup_n = c->leaf_tup_n;
        const Compiler *owner = c->leaf_owner == NULL ? c : c->leaf_owner;
        if (state != 1 || *type != TY_TUPLE || expr->proj_pos >= tup_n || owner->telems == NULL) {
            return state < 0 ? state : 0;
        }
        {
            const TupleElem *elem = &owner->telems[tup0 + expr->proj_pos];
            uint16_t local_mod = elem->mod_index;
            *type = elem->kind;
            *length = elem->length;
            c->leaf_tup0 = 0;
            c->leaf_tup_n = 0;
            c->leaf_owner = c;
            if (*type == TY_MOD && !adopt_modulus(c, owner, elem->mod_index, &local_mod)) {
                *silent = 1;
                return -1;
            }
            c->leaf_mod = *type == TY_MOD ? local_mod : 0;
        }
        return 1;
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
        if (callee.mod->funcs[callee.func].ninst > 0) {
            uint32_t id = UINT32_MAX;
            const Instance *inst;
            Expr *mutable_expr = &c->exprs[index];
            if (!lookup_call(c, mutable_expr, callee.mod, callee.func, 0, func_index, locals_in_scope, &id) ||
                id >= callee.mod->ninstances) {
                *silent = 1;
                return -1;
            }
            inst = &callee.mod->instances[id];
            if (!inst->result_ok) {
                *silent = 1;
                return -1;
            }
            *type = inst->result;
            *length = inst->result_len;
            if (*type == TY_MOD) {
                uint16_t local = 0;
                if (!adopt_modulus(c, callee.mod, inst->result_mod, &local)) {
                    *silent = 1;
                    return -1;
                }
                c->leaf_mod = local;
            }
            if (*type == TY_TUPLE) {
                c->leaf_owner = callee.mod;
                c->leaf_tup0 = inst->tup0;
                c->leaf_tup_n = inst->tup_n;
            }
            return 1;
        }
        *type = callee.mod->funcs[callee.func].result;
        *length = callee.mod->funcs[callee.func].result_len;
        if (*type == TY_MOD) {
            uint16_t local = 0;
            if (!adopt_modulus(c, callee.mod, callee.mod->funcs[callee.func].result_mod, &local)) {
                *silent = 1;
                return -1;
            }
            c->leaf_mod = local;
        }
        if (*type == TY_TUPLE) {
            c->leaf_owner = callee.mod;
            c->leaf_tup0 = callee.mod->funcs[callee.func].tup0;
            c->leaf_tup_n = callee.mod->funcs[callee.func].tup_n;
        }
        return 1;
    }
    if (expr->kind == EX_ACCUM && expr->is_proj) {
        const LoopDesc *loop = &c->loops[expr->arg0];
        const TupleElem *elem;
        if (!loop->acc_ok || expr->name_index >= loop->tup_n || c->telems == NULL) {
            *silent = 1;
            return -1;
        }
        elem = &c->telems[loop->tup0 + expr->name_index];
        *type = elem->kind;
        *length = elem->length;
        if (*type == TY_MOD) {
            c->leaf_mod = elem->mod_index;
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
        if (*type == TY_TUPLE) {
            c->leaf_tup0 = loop->tup0;
            c->leaf_tup_n = loop->tup_n;
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
    if (expr->kind == EX_INDEX || expr->kind == EX_SELECT || expr->kind == EX_PROJECT) {
        return find_leaf(c, index, func_index, locals_in_scope, type, length, &leaf, silent);
    }
    return base_type(c, index, func_index, locals_in_scope, type, length, silent);
}

static int leaf_names_bindings(const Compiler *c, uint32_t leaf, uint32_t bind0, uint16_t nbinds) {
    const Expr *expr;
    uint16_t bind;
    if (nbinds == 0 || leaf == UINT32_MAX || leaf >= c->nexprs) {
        return 0;
    }
    expr = &c->exprs[leaf];
    while (expr->kind == EX_INDEX || expr->kind == EX_SELECT || expr->kind == EX_GROUP || expr->kind == EX_PROJECT ||
           expr->kind == EX_SLICE || expr->kind == EX_SLICE_UP || expr->kind == EX_UPDATE) {
        if (expr->left == UINT32_MAX) {
            break;
        }
        expr = &c->exprs[expr->left];
    }
    if (expr->kind != EX_NAME && expr->kind != EX_LOOP_INDEX && expr->kind != EX_ACCUM) {
        return 0;
    }
    for (bind = 0; bind < nbinds; bind++) {
        const Local *local = &c->block_locals[bind0 + bind];
        if (same_span(c, local->name_start, local->name_end, expr->name_start, expr->name_end)) {
            return 1;
        }
    }
    return 0;
}

static int find_leaf(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, TypeKind *type,
                     uint32_t *length, uint32_t *leaf, int *silent);

enum { BYTES_OK = 0, BYTES_UNPRINTABLE = 1, BYTES_EMPTY = 2, BYTES_LONG = 3, BYTES_BAD = 4 };

static int hex_digit_value(unsigned char ch) {
    if (ch >= '0' && ch <= '9') {
        return ch - '0';
    }
    if (ch >= 'a' && ch <= 'f') {
        return ch - 'a' + 10;
    }
    if (ch >= 'A' && ch <= 'F') {
        return ch - 'A' + 10;
    }
    return -1;
}

static uint32_t utf8_codepoint(const char *text, uint32_t at, uint32_t end, uint32_t *next) {
    unsigned char lead;
    size_t width;
    uint32_t point = 0;
    uint32_t index;
    if (at >= end) {
        *next = at;
        return 0;
    }
    lead = (unsigned char)text[at];
    width = utf8_width(lead);
    if (at + width > end) {
        width = 1;
    }
    if (width == 1) {
        *next = at + 1;
        return lead;
    }
    if (width == 2) {
        point = (uint32_t)(lead & 0x1fu);
    } else if (width == 3) {
        point = (uint32_t)(lead & 0x0fu);
    } else {
        point = (uint32_t)(lead & 0x07u);
    }
    for (index = 1; index < width; index++) {
        unsigned char cont = (unsigned char)text[at + index];
        if ((cont & 0xc0u) != 0x80u) {
            *next = at + 1;
            return lead;
        }
        point = (point << 6) | (uint32_t)(cont & 0x3fu);
    }
    *next = at + (uint32_t)width;
    return point;
}

/* Decode a byte or hex string the lexer accepted. `out` may be NULL to count only. */
static int decode_bytes(const Compiler *c, const Expr *expr, uint8_t *out, uint32_t *count, uint32_t *err_start,
                        uint32_t *err_end, uint32_t *codepoint) {
    const char *text = c->text;
    uint32_t cursor;
    uint32_t limit;
    uint32_t n = 0;
    int hex = expr->op == TK_HEX;
    *count = 0;
    if (expr->end < expr->start + 2 || text[expr->end - 1] != '"') {
        return BYTES_BAD;
    }
    if (hex) {
        int pending = -1;
        if (expr->end < expr->start + 5) {
            return BYTES_BAD;
        }
        cursor = expr->start + 4;
        limit = expr->end - 1;
        for (; cursor < limit; cursor++) {
            unsigned char ch = (unsigned char)text[cursor];
            int digit;
            if (ch == ' ') {
                continue;
            }
            digit = hex_digit_value(ch);
            if (digit < 0) {
                return BYTES_BAD;
            }
            if (pending < 0) {
                pending = digit;
            } else {
                if (n >= MAX_ARRAY_LENGTH) {
                    return BYTES_LONG;
                }
                if (out != NULL) {
                    out[n] = (uint8_t)((pending << 4) | digit);
                }
                n++;
                pending = -1;
            }
        }
        if (pending >= 0) {
            return BYTES_BAD;
        }
    } else {
        cursor = expr->start + 1;
        limit = expr->end - 1;
        while (cursor < limit) {
            unsigned char ch = (unsigned char)text[cursor];
            if (ch == '\\') {
                unsigned char esc;
                if (cursor + 1 >= limit) {
                    return BYTES_BAD;
                }
                esc = (unsigned char)text[cursor + 1];
                cursor += 2;
                if (esc == 'n') {
                    ch = '\n';
                } else if (esc == 'r') {
                    ch = '\r';
                } else if (esc == 't') {
                    ch = '\t';
                } else if (esc == '0') {
                    ch = 0;
                } else if (esc == '"') {
                    ch = '"';
                } else if (esc == '\\') {
                    ch = '\\';
                } else if (esc == 'x') {
                    int hi;
                    int lo;
                    if (cursor + 1 >= limit) {
                        return BYTES_BAD;
                    }
                    hi = hex_digit_value((unsigned char)text[cursor]);
                    lo = hex_digit_value((unsigned char)text[cursor + 1]);
                    if (hi < 0 || lo < 0) {
                        return BYTES_BAD;
                    }
                    ch = (unsigned char)((hi << 4) | lo);
                    cursor += 2;
                } else {
                    return BYTES_BAD;
                }
                if (n >= MAX_ARRAY_LENGTH) {
                    return BYTES_LONG;
                }
                if (out != NULL) {
                    out[n] = ch;
                }
                n++;
                continue;
            }
            if (ch >= 0x20 && ch <= 0x7e) {
                if (n >= MAX_ARRAY_LENGTH) {
                    return BYTES_LONG;
                }
                if (out != NULL) {
                    out[n] = ch;
                }
                n++;
                cursor++;
                continue;
            }
            *err_start = cursor;
            *codepoint = utf8_codepoint(text, cursor, limit, err_end);
            return BYTES_UNPRINTABLE;
        }
    }
    if (n == 0) {
        return BYTES_EMPTY;
    }
    *count = n;
    return BYTES_OK;
}

#define MAX_AFFINE_TERMS 16

typedef struct AffineForm {
    int64_t constant;
    uint32_t loop_id[MAX_AFFINE_TERMS];
    int64_t coeff[MAX_AFFINE_TERMS];
    int nterms;
} AffineForm;

enum { AFF_OK = 0, AFF_NOT_STATIC = 1, AFF_PRODUCT = 2, AFF_OVERSIZED = 3 };

static int i64_add(int64_t left, int64_t right, int64_t *out) {
    if (right > 0 && left > INT64_MAX - right) {
        return 0;
    }
    if (right < 0 && left < INT64_MIN - right) {
        return 0;
    }
    *out = left + right;
    return 1;
}

static int i64_sub(int64_t left, int64_t right, int64_t *out) {
    if (right > 0 && left < INT64_MIN + right) {
        return 0;
    }
    if (right < 0 && left > INT64_MAX + right) {
        return 0;
    }
    *out = left - right;
    return 1;
}

static int i64_mul(int64_t left, int64_t right, int64_t *out) {
    if (left == 0 || right == 0) {
        *out = 0;
        return 1;
    }
    if ((left == -1 && right == INT64_MIN) || (right == -1 && left == INT64_MIN)) {
        return 0;
    }
    if (left > 0) {
        if (right > 0) {
            if (left > INT64_MAX / right) {
                return 0;
            }
        } else if (right < INT64_MIN / left) {
            return 0;
        }
    } else if (right > 0) {
        if (left < INT64_MIN / right) {
            return 0;
        }
    } else if (right < INT64_MAX / left) {
        return 0;
    }
    *out = left * right;
    return 1;
}

static int big_as_i64(const Big *value, int64_t *out) {
    uint64_t magnitude = 0;
    if (value->nlimbs > 2) {
        return 0;
    }
    if (value->nlimbs >= 1) {
        magnitude = value->limbs[0];
    }
    if (value->nlimbs >= 2) {
        magnitude |= (uint64_t)value->limbs[1] << 32;
    }
    if (!value->negative) {
        if (magnitude > (uint64_t)INT64_MAX) {
            return 0;
        }
        *out = (int64_t)magnitude;
        return 1;
    }
    if (magnitude > (uint64_t)INT64_MAX + 1ull) {
        return 0;
    }
    if (magnitude == (uint64_t)INT64_MAX + 1ull) {
        *out = INT64_MIN;
        return 1;
    }
    *out = -(int64_t)magnitude;
    return 1;
}

static int affine_add_term(AffineForm *form, uint32_t loop_id, int64_t coeff) {
    int index;
    if (coeff == 0) {
        return 1;
    }
    for (index = 0; index < form->nterms; index++) {
        if (form->loop_id[index] != loop_id) {
            continue;
        }
        if (!i64_add(form->coeff[index], coeff, &coeff)) {
            return 0;
        }
        if (coeff == 0) {
            form->nterms--;
            form->loop_id[index] = form->loop_id[form->nterms];
            form->coeff[index] = form->coeff[form->nterms];
        } else {
            form->coeff[index] = coeff;
        }
        return 1;
    }
    if (form->nterms >= MAX_AFFINE_TERMS) {
        return 0;
    }
    form->loop_id[form->nterms] = loop_id;
    form->coeff[form->nterms] = coeff;
    form->nterms++;
    return 1;
}

static int affine_combine(AffineForm *dest, const AffineForm *left, const AffineForm *right, int subtract) {
    int index;
    int64_t constant;
    *dest = *left;
    if (subtract) {
        if (!i64_sub(left->constant, right->constant, &constant)) {
            return 0;
        }
    } else if (!i64_add(left->constant, right->constant, &constant)) {
        return 0;
    }
    dest->constant = constant;
    for (index = 0; index < right->nterms; index++) {
        int64_t coeff = right->coeff[index];
        if (subtract) {
            if (coeff == INT64_MIN) {
                return 0;
            }
            coeff = -coeff;
        }
        if (!affine_add_term(dest, right->loop_id[index], coeff)) {
            return 0;
        }
    }
    return 1;
}

static int affine_scale(AffineForm *dest, const AffineForm *form, int64_t factor) {
    int index;
    dest->nterms = 0;
    if (!i64_mul(form->constant, factor, &dest->constant)) {
        return 0;
    }
    if (factor == 0) {
        return 1;
    }
    for (index = 0; index < form->nterms; index++) {
        int64_t coeff;
        if (!i64_mul(form->coeff[index], factor, &coeff) || !affine_add_term(dest, form->loop_id[index], coeff)) {
            return 0;
        }
    }
    return 1;
}

static int affine_range(const Compiler *c, const AffineForm *form, int64_t *low, int64_t *high) {
    int index;
    *low = form->constant;
    *high = form->constant;
    for (index = 0; index < form->nterms; index++) {
        const LoopDesc *loop;
        int64_t first;
        int64_t last;
        int64_t at_first;
        int64_t at_last;
        int64_t least;
        int64_t greatest;
        int64_t next_low;
        int64_t next_high;
        if (form->loop_id[index] >= c->nloops) {
            return 0;
        }
        loop = &c->loops[form->loop_id[index]];
        if (!loop->bounds_ok || loop->bound_b <= loop->bound_a || loop->bound_a > (uint32_t)INT64_MAX ||
            loop->bound_b - 1u > (uint32_t)INT64_MAX) {
            return 0;
        }
        first = (int64_t)loop->bound_a;
        last = (int64_t)(loop->bound_b - 1u);
        if (!i64_mul(form->coeff[index], first, &at_first) || !i64_mul(form->coeff[index], last, &at_last)) {
            return 0;
        }
        if (at_first > at_last) {
            least = at_last;
            greatest = at_first;
        } else {
            least = at_first;
            greatest = at_last;
        }
        if (!i64_add(*low, least, &next_low) || !i64_add(*high, greatest, &next_high)) {
            return 0;
        }
        *low = next_low;
        *high = next_high;
    }
    return 1;
}

/* Affine form of an Int bound. NOT_STATIC and PRODUCT set the offending span. */
static const char SIZE_NOTE[] =
    "a size is fixed in each instance of its function: it is built from integer literals and the function's size parameters with `+`, `-`, `*`, `/`, `%`, and parentheses";
static const char SIZE_RANGE_NOTE[] =
    "a size parameter `n in a..b` takes each value from a up to, but not including, b, with a < b <= 65536, and a function has at most 256 instances";

typedef struct Sz {
    int kind; /* 0 value, 1 not static, 2 too large */
    int64_t value;
    uint32_t start;
    uint32_t end;
} Sz;

static int size_slot_of(const Compiler *c, uint32_t func_index, uint32_t start, uint32_t end, uint8_t *slot) {
    const Func *func;
    uint8_t index;
    if (func_index >= c->nfuncs) {
        return 0;
    }
    func = &c->funcs[func_index];
    for (index = 0; index < func->nsizes; index++) {
        if (same_span(c, func->sz_name0[index], func->sz_name1[index], start, end)) {
            if (slot != NULL) {
                *slot = index;
            }
            return 1;
        }
    }
    return 0;
}

static void report_binding_dup(Compiler *c, uint32_t func_index, uint32_t local_limit, uint32_t diag_start,
                               uint32_t diag_end, uint32_t match_start, uint32_t match_end, uint32_t within_start,
                               uint32_t within_end) {
    char message[160];
    char spelling[64];
    const Func *func = &c->funcs[func_index];
    uint32_t earlier_start = within_start;
    uint32_t earlier_end = within_end;
    const char *label = within_end > within_start ? "the first name is here" : NULL;
    uint8_t slot = 0;
    uint16_t index;
    int overridden = 0;
    span_copy(spelling, sizeof spelling, c->text, match_start, match_end);
    snprintf(message, sizeof message, "duplicate binding `%s`", spelling);
    if (size_slot_of(c, func_index, match_start, match_end, &slot)) {
        earlier_start = func->sz_name0[slot];
        earlier_end = func->sz_name1[slot];
        label = "the size parameter is here";
        overridden = 1;
    }
    for (index = 0; index < func->nparams && !overridden; index++) {
        const Param *param = &c->params[func->param0 + index];
        if (!param->duplicate && same_span(c, param->name_start, param->name_end, match_start, match_end)) {
            earlier_start = param->name_start;
            earlier_end = param->name_end;
            label = "the parameter is here";
            overridden = 1;
        }
    }
    for (index = 0; index < local_limit && index < func->nlocals && !overridden; index++) {
        const Local *before = &c->locals[func->local0 + index];
        if (!before->duplicate && same_span(c, before->name_start, before->name_end, match_start, match_end)) {
            earlier_start = before->name_start;
            earlier_end = before->name_end;
            label = "the first binding is here";
            overridden = 1;
        }
    }
    add_diag(c, "ORC0219", diag_start, diag_end, message, "this binding repeats an earlier name",
             "each parameter and binding of a function has its own name; Orange has no shadowing", 2);
    if (label != NULL && earlier_end > earlier_start) {
        diag_add_secondary(c, earlier_start, earlier_end, label);
    }
}

/* Euclidean quotient and remainder. x / 0 is 0 and x % 0 is x. */
static int i64_euclid(int64_t left, int64_t right, int64_t *quot, int64_t *rem) {
    int64_t q;
    int64_t r;
    if (right == 0) {
        *quot = 0;
        *rem = left;
        return 1;
    }
    if (left == INT64_MIN && right == -1) {
        return 0;
    }
    q = left / right;
    r = left % right;
    if (r < 0) {
        if (right > 0) {
            q -= 1;
            r += right;
        } else {
            q += 1;
            r -= right;
        }
    }
    *quot = q;
    *rem = r;
    return 1;
}

static Sz sz_value(int64_t value) {
    Sz out;
    memset(&out, 0, sizeof out);
    out.value = value;
    return out;
}

static Sz sz_fault(int kind, uint32_t start, uint32_t end) {
    Sz out;
    memset(&out, 0, sizeof out);
    out.kind = kind;
    out.start = start;
    out.end = end;
    return out;
}

static Sz eval_size(Compiler *c, uint32_t index) {
    const Expr *expr;
    if (index == UINT32_MAX || index >= c->nexprs) {
        return sz_fault(1, 0, 0);
    }
    expr = &c->exprs[index];
    if (expr->kind == EX_GROUP) {
        return eval_size(c, expr->left);
    }
    if (expr->kind == EX_LIT) {
        Big magnitude = big_zero();
        int64_t value = 0;
        if (!decode_literal(c, expr, &magnitude) || !big_as_i64(&magnitude, &value)) {
            return sz_fault(2, expr->start, expr->end);
        }
        return sz_value(value);
    }
    if (expr->kind == EX_NAME) {
        uint8_t slot = 0;
        if (c->cur_func < c->nfuncs && size_slot_of(c, c->cur_func, expr->name_start, expr->name_end, &slot) &&
            slot < c->ncur) {
            return sz_value(c->cur_sz[slot]);
        }
        return sz_fault(1, expr->start, expr->end);
    }
    if (expr->kind == EX_UNARY && expr->op == TK_MINUS) {
        Sz inner = eval_size(c, expr->left);
        int64_t negated;
        if (inner.kind != 0) {
            return inner;
        }
        if (inner.value == INT64_MIN) {
            return sz_fault(2, expr->start, expr->end);
        }
        negated = -inner.value;
        return sz_value(negated);
    }
    if (expr->kind == EX_BINARY && size_operator(expr->op)) {
        Sz left = eval_size(c, expr->left);
        Sz right = eval_size(c, expr->right);
        int64_t value = 0;
        int64_t quot = 0;
        int64_t rem = 0;
        if (left.kind != 0 && right.kind != 0) {
            if (left.kind == 2 && right.kind == 1) {
                return right;
            }
            return left;
        }
        if (left.kind != 0) {
            return left;
        }
        if (right.kind != 0) {
            return right;
        }
        if (expr->op == TK_PLUS) {
            if (!i64_add(left.value, right.value, &value)) {
                return sz_fault(2, expr->start, expr->end);
            }
        } else if (expr->op == TK_MINUS) {
            if (right.value == INT64_MIN) {
                if (!i64_add(left.value, INT64_MAX, &value) || !i64_add(value, 1, &value)) {
                    return sz_fault(2, expr->start, expr->end);
                }
            } else if (!i64_add(left.value, -right.value, &value)) {
                return sz_fault(2, expr->start, expr->end);
            }
        } else if (expr->op == TK_STAR) {
            if (!i64_mul(left.value, right.value, &value)) {
                return sz_fault(2, expr->start, expr->end);
            }
        } else if (!i64_euclid(left.value, right.value, &quot, &rem)) {
            return sz_fault(2, expr->start, expr->end);
        } else {
            value = expr->op == TK_SLASH ? quot : rem;
        }
        return sz_value(value);
    }
    return sz_fault(1, expr->start, expr->end);
}

static void report_size_fault(Compiler *c, Sz fault) {
    if (fault.kind == 2) {
        add_diag(c, "ORC0205", fault.start, fault.end, "integer magnitude exceeds the 16384-significant-bit limit",
                 "this part of the size is too large", "the value is rejected rather than truncated or approximated", 2);
        return;
    }
    add_diag(c, "ORC0237", fault.start, fault.end, "a size may use only integer literals and size parameters",
             "this is neither", SIZE_NOTE, 2);
}

static int size_length(Compiler *c, uint32_t index, int report, uint32_t *length) {
    Sz value = eval_size(c, index);
    if (value.kind != 0) {
        if (report) {
            report_size_fault(c, value);
        }
        return 0;
    }
    if (value.value < 1 || value.value > (int64_t)MAX_ARRAY_LENGTH) {
        if (report) {
            char message[160];
            snprintf(message, sizeof message, "this array length is %lld, but an array has 1 through 65536 elements",
                     (long long)value.value);
            add_diag(c, "ORC0221", c->exprs[index].start, c->exprs[index].end, message,
                     "unsupported array length in this instance",
                     "a length written with sizes is computed in each instance of its function, and every instance's "
                     "lengths are from 1 through 65536",
                     2);
        }
        return 0;
    }
    *length = (uint32_t)value.value;
    return 1;
}

static int affine_form(Compiler *c, uint32_t index, AffineForm *form, uint32_t *bad_start, uint32_t *bad_end) {
    const Expr *expr = &c->exprs[index];
    form->constant = 0;
    form->nterms = 0;
    switch (expr->kind) {
    case EX_LIT: {
        Big magnitude = big_zero();
        int64_t value = 0;
        if (!decode_literal(c, expr, &magnitude) || !big_as_i64(&magnitude, &value)) {
            return AFF_OVERSIZED;
        }
        form->constant = value;
        return AFF_OK;
    }
    case EX_GROUP:
        return affine_form(c, expr->left, form, bad_start, bad_end);
    case EX_UNARY:
        if (expr->op != TK_MINUS) {
            *bad_start = expr->start;
            *bad_end = expr->end;
            return AFF_NOT_STATIC;
        }
        if (affine_form(c, expr->left, form, bad_start, bad_end) != AFF_OK) {
            return affine_form(c, expr->left, form, bad_start, bad_end);
        }
        if (form->constant == INT64_MIN) {
            return AFF_OVERSIZED;
        }
        form->constant = -form->constant;
        {
            int term;
            for (term = 0; term < form->nterms; term++) {
                if (form->coeff[term] == INT64_MIN) {
                    return AFF_OVERSIZED;
                }
                form->coeff[term] = -form->coeff[term];
            }
        }
        return AFF_OK;
    case EX_LOOP_INDEX:
        form->constant = 0;
        if (!affine_add_term(form, expr->arg0, 1)) {
            return AFF_OVERSIZED;
        }
        return AFF_OK;
    case EX_NAME: {
        uint8_t slot = 0;
        if (c->cur_func < c->nfuncs && size_slot_of(c, c->cur_func, expr->name_start, expr->name_end, &slot) &&
            slot < c->ncur) {
            form->constant = c->cur_sz[slot];
            return AFF_OK;
        }
        *bad_start = expr->start;
        *bad_end = expr->end;
        return AFF_NOT_STATIC;
    }
    case EX_BINARY:
        if (expr->op == TK_PLUS || expr->op == TK_MINUS || expr->op == TK_STAR) {
            AffineForm left_form;
            AffineForm right_form;
            uint32_t left_start = 0;
            uint32_t left_end = 0;
            uint32_t right_start = 0;
            uint32_t right_end = 0;
            int left_state = affine_form(c, expr->left, &left_form, &left_start, &left_end);
            int right_state = affine_form(c, expr->right, &right_form, &right_start, &right_end);
            int64_t low = 0;
            int64_t high = 0;
            if (left_state == AFF_OVERSIZED && right_state != AFF_OK) {
                *bad_start = right_start;
                *bad_end = right_end;
                return right_state;
            }
            if (left_state != AFF_OK) {
                *bad_start = left_start;
                *bad_end = left_end;
                return left_state;
            }
            if (right_state != AFF_OK) {
                *bad_start = right_start;
                *bad_end = right_end;
                return right_state;
            }
            if (expr->op == TK_STAR) {
                if (left_form.nterms == 0) {
                    if (!affine_scale(form, &right_form, left_form.constant)) {
                        return AFF_OVERSIZED;
                    }
                } else if (right_form.nterms == 0) {
                    if (!affine_scale(form, &left_form, right_form.constant)) {
                        return AFF_OVERSIZED;
                    }
                } else {
                    *bad_start = expr->op_start;
                    *bad_end = expr->op_end;
                    return AFF_PRODUCT;
                }
            } else if (!affine_combine(form, &left_form, &right_form, expr->op == TK_MINUS)) {
                return AFF_OVERSIZED;
            }
            if (!affine_range(c, form, &low, &high)) {
                return AFF_OVERSIZED;
            }
            (void)low;
            (void)high;
            return AFF_OK;
        }
        *bad_start = expr->start;
        *bad_end = expr->end;
        return AFF_NOT_STATIC;
    default:
        *bad_start = expr->start;
        *bad_end = expr->end;
        return AFF_NOT_STATIC;
    }
}

/* Fixed positive length of a slice, when both bounds are static and cancel. */
static int proved_slice_length(Compiler *c, uint32_t start_expr, uint32_t end_expr, uint32_t base_len, uint32_t *length) {
    AffineForm start_form;
    AffineForm end_form;
    AffineForm difference;
    uint32_t bad_start = 0;
    uint32_t bad_end = 0;
    if (end_expr == UINT32_MAX) {
        end_form.constant = (int64_t)base_len;
        end_form.nterms = 0;
    } else if (affine_form(c, end_expr, &end_form, &bad_start, &bad_end) != AFF_OK) {
        return 0;
    }
    if (start_expr == UINT32_MAX) {
        start_form.constant = 0;
        start_form.nterms = 0;
    } else if (affine_form(c, start_expr, &start_form, &bad_start, &bad_end) != AFF_OK) {
        return 0;
    }
    if (!affine_combine(&difference, &end_form, &start_form, 1) || difference.nterms != 0 || difference.constant < 1 ||
        difference.constant > (int64_t)MAX_ARRAY_LENGTH) {
        return 0;
    }
    *length = (uint32_t)difference.constant;
    return 1;
}

static void array_parts(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, int *have_len,
                        uint32_t *len, int *have_elem, TypeKind *elem);

static void array_parts(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, int *have_len,
                        uint32_t *len, int *have_elem, TypeKind *elem) {
    const Expr *expr = &c->exprs[index];
    *have_len = 0;
    *len = 0;
    *have_elem = 0;
    *elem = TY_NONE;
    switch (expr->kind) {
    case EX_GROUP:
        array_parts(c, expr->left, func_index, locals_in_scope, have_len, len, have_elem, elem);
        return;
    case EX_ARRAY:
        *have_len = 1;
        *len = expr->argc;
        return;
    case EX_FILL:
        if (expr->size_expr != UINT32_MAX) {
            uint32_t sized = 0;
            if (size_length(c, expr->size_expr, 0, &sized)) {
                *have_len = 1;
                *len = sized;
            }
        } else if (canonical_array_length(c->text, expr->lit_start, expr->lit_end, len)) {
            *have_len = 1;
        }
        return;
    case EX_BYTES: {
        uint32_t count = 0;
        uint32_t err_start = 0;
        uint32_t err_end = 0;
        uint32_t point = 0;
        if (decode_bytes(c, expr, NULL, &count, &err_start, &err_end, &point) == BYTES_OK) {
            *have_len = 1;
            *len = count;
            *have_elem = 1;
            *elem = TY_W8;
        }
        return;
    }
    case EX_BINARY:
        if (expr->op == TK_PLUSPLUS) {
            int left_len = 0;
            int right_len = 0;
            int left_elem = 0;
            int right_elem = 0;
            uint32_t left_n = 0;
            uint32_t right_n = 0;
            TypeKind left_ty = TY_NONE;
            TypeKind right_ty = TY_NONE;
            array_parts(c, expr->left, func_index, locals_in_scope, &left_len, &left_n, &left_elem, &left_ty);
            array_parts(c, expr->right, func_index, locals_in_scope, &right_len, &right_n, &right_elem, &right_ty);
            if (left_len && right_len && left_n <= UINT32_MAX - right_n) {
                *have_len = 1;
                *len = left_n + right_n;
            }
            if (left_elem || right_elem) {
                *have_elem = 1;
                *elem = left_elem ? left_ty : right_ty;
            }
            return;
        }
        break;
    case EX_UPDATE:
    case EX_SLICE_UP:
        array_parts(c, expr->left, func_index, locals_in_scope, have_len, len, have_elem, elem);
        return;
    case EX_SLICE: {
        TypeKind base_kind = TY_NONE;
        uint32_t base_len = 0;
        uint32_t leaf = 0;
        int silent = 0;
        int state = find_leaf(c, expr->left, func_index, locals_in_scope, &base_kind, &base_len, &leaf, &silent);
        uint32_t slice_len = 0;
        if (state == 1 && base_len > 0 &&
            proved_slice_length(c, expr->right, expr->callee, base_len, &slice_len)) {
            *have_len = 1;
            *len = slice_len;
            *have_elem = 1;
            *elem = base_kind;
        }
        return;
    }
    case EX_COND: {
        uint16_t arm;
        for (arm = 0; arm < expr->argc; arm++) {
            const CondArm *item = &c->cond_arms[expr->arg0 + arm];
            if (item->nbinds != 0) {
                continue;
            }
            array_parts(c, item->value, func_index, locals_in_scope, have_len, len, have_elem, elem);
            if (*have_len) {
                return;
            }
        }
        if (expr->else_nbinds == 0) {
            array_parts(c, expr->right, func_index, locals_in_scope, have_len, len, have_elem, elem);
        }
        return;
    }
    default:
        break;
    }
    {
        TypeKind ty = TY_NONE;
        uint32_t found = 0;
        uint32_t leaf = 0;
        int silent = 0;
        int state = find_leaf(c, index, func_index, locals_in_scope, &ty, &found, &leaf, &silent);
        if (state == 1 && found > 0) {
            *have_len = 1;
            *len = found;
            *have_elem = 1;
            *elem = ty;
        }
    }
}

static int lookup_call(Compiler *c, Expr *expr, Compiler *target, uint32_t callee, int report, uint32_t caller_func,
                      uint32_t locals, uint32_t *inst_id);

static int find_leaf(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, TypeKind *type,
                     uint32_t *length, uint32_t *leaf, int *silent) {
    const Expr *expr = &c->exprs[index];
    int left_state;
    *silent = 0;
    *leaf = index;
    *type = TY_NONE;
    *length = 0;
    c->leaf_mod = 0;
    c->leaf_tup0 = 0;
    c->leaf_tup_n = 0;
    c->leaf_owner = c;
    switch (expr->kind) {
    case EX_LIT:
        return 0;
    case EX_ARRAY:
    case EX_TUPLE:
        return 2;
    case EX_GROUP:
    case EX_UNARY:
        return find_leaf(c, expr->left, func_index, locals_in_scope, type, length, leaf, silent);
    case EX_SHIFT:
        return find_leaf(c, expr->left, func_index, locals_in_scope, type, length, leaf, silent);
    case EX_BINARY:
        if (expr->op == TK_PLUSPLUS) {
            int left_len = 0;
            int right_len = 0;
            int left_elem = 0;
            int right_elem = 0;
            uint32_t left_n = 0;
            uint32_t right_n = 0;
            TypeKind left_ty = TY_NONE;
            TypeKind right_ty = TY_NONE;
            array_parts(c, expr->left, func_index, locals_in_scope, &left_len, &left_n, &left_elem, &left_ty);
            array_parts(c, expr->right, func_index, locals_in_scope, &right_len, &right_n, &right_elem, &right_ty);
            if (left_len && right_len && (left_elem || right_elem) && left_n <= UINT32_MAX - right_n) {
                *type = left_elem ? left_ty : right_ty;
                *length = left_n + right_n;
                return 1;
            }
            return 0;
        }
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
            const CondArm *item = &c->cond_arms[expr->arg0 + arm];
            int state = find_leaf(c, item->value, func_index, locals_in_scope, type, length, leaf, silent);
            if (state != 0 && !leaf_names_bindings(c, *leaf, item->bind0, item->nbinds)) {
                return state;
            }
        }
        {
            int state = find_leaf(c, expr->right, func_index, locals_in_scope, type, length, leaf, silent);
            if (state != 0 && !leaf_names_bindings(c, *leaf, expr->else_bind0, expr->else_nbinds)) {
                return state;
            }
        }
        return 0;
    }
    case EX_NAME: {
        NameRes res;
        uint16_t slot;
        uint32_t abs_index = 0;
        int type_ok = 0;
        resolve_name(c, func_index, locals_in_scope, expr->name_start, expr->name_end, &res, &slot, type, length,
                     &type_ok, &abs_index);
        if (res == NAME_SIZE) {
            *type = TY_INT;
            *length = 0;
            return 1;
        }
        if (res == NAME_BAD) {
            *silent = 1;
            return -1;
        }
        if (res == NAME_PARAM || res == NAME_LOCAL || res == NAME_BLOCK) {
            if (*type == TY_MOD) {
                c->leaf_mod = res == NAME_PARAM ? c->params[c->funcs[func_index].param0 + slot].mod_index
                                : res == NAME_BLOCK ? c->block_locals[abs_index].mod_index
                                                    : c->locals[c->funcs[func_index].local0 + slot].mod_index;
            }
            if (*type == TY_TUPLE) {
                if (res == NAME_PARAM) {
                    const Param *param = &c->params[c->funcs[func_index].param0 + slot];
                    c->leaf_tup0 = param->tup0;
                    c->leaf_tup_n = param->tup_n;
                } else if (res == NAME_BLOCK) {
                    c->leaf_tup0 = c->block_locals[abs_index].tup0;
                    c->leaf_tup_n = c->block_locals[abs_index].tup_n;
                } else {
                    c->leaf_tup0 = c->locals[c->funcs[func_index].local0 + slot].tup0;
                    c->leaf_tup_n = c->locals[c->funcs[func_index].local0 + slot].tup_n;
                }
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
        if (callee.mod->funcs[callee.func].ninst > 0) {
            uint32_t id = UINT32_MAX;
            const Instance *inst;
            Expr *mutable_expr = &c->exprs[index];
            if (!lookup_call(c, mutable_expr, callee.mod, callee.func, 0, func_index, locals_in_scope, &id) ||
                id >= callee.mod->ninstances) {
                *silent = 1;
                return -1;
            }
            inst = &callee.mod->instances[id];
            if (!inst->result_ok) {
                *silent = 1;
                return -1;
            }
            *type = inst->result;
            *length = inst->result_len;
            if (*type == TY_MOD) {
                uint16_t local = 0;
                if (!adopt_modulus(c, callee.mod, inst->result_mod, &local)) {
                    *silent = 1;
                    return -1;
                }
                c->leaf_mod = local;
            }
            if (*type == TY_TUPLE) {
                c->leaf_owner = callee.mod;
                c->leaf_tup0 = inst->tup0;
                c->leaf_tup_n = inst->tup_n;
            }
            return 1;
        }
        *type = callee.mod->funcs[callee.func].result;
        *length = callee.mod->funcs[callee.func].result_len;
        if (*type == TY_MOD) {
            uint16_t local = 0;
            if (!adopt_modulus(c, callee.mod, callee.mod->funcs[callee.func].result_mod, &local)) {
                *silent = 1;
                return -1;
            }
            c->leaf_mod = local;
        }
        if (*type == TY_TUPLE) {
            c->leaf_owner = callee.mod;
            c->leaf_tup0 = callee.mod->funcs[callee.func].tup0;
            c->leaf_tup_n = callee.mod->funcs[callee.func].tup_n;
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
        if (expr->is_proj) {
            const TupleElem *elem;
            if (expr->name_index >= loop->tup_n || c->telems == NULL) {
                *silent = 1;
                return -1;
            }
            elem = &c->telems[loop->tup0 + expr->name_index];
            *type = elem->kind;
            *length = elem->length;
            if (*type == TY_MOD) {
                c->leaf_mod = elem->mod_index;
            }
            return 1;
        }
        *type = loop->acc_type;
        *length = loop->acc_len;
        if (*type == TY_MOD) {
            c->leaf_mod = loop->acc_mod;
        }
        if (*type == TY_TUPLE) {
            c->leaf_tup0 = loop->tup0;
            c->leaf_tup_n = loop->tup_n;
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
        if (*type == TY_TUPLE) {
            c->leaf_tup0 = loop->tup0;
            c->leaf_tup_n = loop->tup_n;
        }
        return 1;
    }
    case EX_PROJECT: {
        int state = base_type(c, index, func_index, locals_in_scope, type, length, silent);
        return state;
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
        /* An invalid target is still a leaf. Callers check this node so
           ORC0204 or ORC0203 is reported; silencing it dropped that
           diagnostic when the conversion was itself an operand. */
        if (!expr->conv_ok) {
            return -1;
        }
        *type = expr->conv_ty;
        if (*type == TY_MOD) {
            c->leaf_mod = expr->conv_mod;
        }
        return 1;
    case EX_BYTES: {
        uint32_t count = 0;
        uint32_t err_start = 0;
        uint32_t err_end = 0;
        uint32_t point = 0;
        if (decode_bytes(c, expr, NULL, &count, &err_start, &err_end, &point) == BYTES_OK) {
            *type = TY_W8;
            *length = count;
            return 1;
        }
        return -1;
    }
    case EX_SLICE: {
        TypeKind base_kind = TY_NONE;
        uint32_t base_len = 0;
        uint32_t base_leaf = 0;
        int base_silent = 0;
        int state = find_leaf(c, expr->left, func_index, locals_in_scope, &base_kind, &base_len, &base_leaf, &base_silent);
        uint32_t slice_len = 0;
        *silent = 0;
        *leaf = index;
        *type = TY_NONE;
        *length = 0;
        if (state == 1 && base_len > 0 && proved_slice_length(c, expr->right, expr->callee, base_len, &slice_len)) {
            *type = base_kind;
            *length = slice_len;
            return 1;
        }
        return 0;
    }
    case EX_SLICE_UP:
        return find_leaf(c, expr->left, func_index, locals_in_scope, type, length, leaf, silent);
    default:
        /* A conditional is not a typed leaf. Bindings inside a branch do
           not type the enclosing `if`, so converting that `if` reports
           ORC0220 only. */
        return 0;
    }
}

static void report_unknown_name(Compiler *c, const Expr *expr, uint32_t func_index, NameRes res) {
    char message[384];
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
    if (res == NAME_EARLY || res == NAME_BLOCK_EARLY) {
        uint32_t bind_start = 0;
        uint32_t bind_end = 0;
        snprintf(message, sizeof message, "`%s` is used before it is bound", ident);
        add_diag(c, "ORC0211", expr->start, expr->end, message, "not bound yet",
                 res == NAME_BLOCK_EARLY
                     ? "a binding is in scope after its own `;`, for the bindings that follow it and the value of its step or branch"
                     : "a binding is in scope after its own `;`, for the bindings that follow it and the result",
                 2);
        if (res == NAME_BLOCK_EARLY && expr->name_abs < c->nblock_locals) {
            const Local *bound = &c->block_locals[expr->name_abs];
            bind_start = bound->name_start;
            bind_end = bound->name_end;
        } else if (res == NAME_EARLY && expr->name_index < c->funcs[func_index].nlocals) {
            const Local *bound = &c->locals[c->funcs[func_index].local0 + expr->name_index];
            bind_start = bound->name_start;
            bind_end = bound->name_end;
        }
        if (bind_end > bind_start) {
            diag_add_secondary(c, bind_start, bind_end, "the binding is here");
        }
        return;
    }
    if (c->nfinished > 0) {
        uint32_t block;
        for (block = c->nfinished; block > 0; block--) {
            const FinishedBlock *done = &c->finished[block - 1];
            uint16_t bind;
            for (bind = 0; bind < done->nbinds; bind++) {
                const Local *local = &c->block_locals[done->bind0 + bind];
                if (!same_span(c, local->name_start, local->name_end, expr->name_start, expr->name_end)) {
                    continue;
                }
                snprintf(message, sizeof message, "`%s` is not in scope here", ident);
                add_diag(c, "ORC0211", expr->start, expr->end, message, "unknown name",
                         "a binding of a loop's step or a branch is in scope only within that step or branch", 2);
                diag_add_secondary(c, local->name_start, local->name_end, "a binding of this name is here");
                return;
            }
        }
    }
    if (c->funcs[func_index].nlocals > 0 || c->funcs[func_index].has_blocks) {
        snprintf(message, sizeof message, "`%s` is not a parameter or binding of `%s`", ident, func_name);
    } else {
        snprintf(message, sizeof message, "`%s` is not a parameter of `%s`", ident, func_name);
    }
    {
        const char *note = (c->funcs[func_index].nlocals > 0 || c->funcs[func_index].has_blocks)
                               ? "a bare name in a `spec` body refers to one of its parameters or bindings"
                               : "a bare name in a `spec` body refers to one of its parameters";
        char call_note[192];
        uint32_t cursor;
        for (cursor = 0; cursor < c->nfuncs; cursor++) {
            const Func *other = &c->funcs[cursor];
            if (!other->is_impl &&
                same_span(c, other->name_start, other->name_end, expr->name_start, expr->name_end)) {
                snprintf(call_note, sizeof call_note, "to call the function `%s`, write `%s()` with its arguments",
                         ident, ident);
                note = call_note;
                break;
            }
        }
        add_diag(c, "ORC0211", expr->start, expr->end, message, "unknown name", note, 2);
    }
}

static int record_edge(Compiler *c, uint32_t func_index, uint32_t callee, uint32_t callee_inst, uint32_t start,
                       uint32_t end) {
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
    c->edges[c->nedges].caller_inst = c->cur_inst;
    c->edges[c->nedges].callee_inst = callee_inst;
    c->nedges++;
    func->nedges++;
    return 1;
}

static int name_is_active_loop(const Compiler *c, uint32_t start, uint32_t end) {
    int index;
    for (index = 0; index < c->nactive; index++) {
        const LoopDesc *loop = &c->loops[c->active_loops[index]];
        if (same_span(c, loop->index_start, loop->index_end, start, end) ||
            (loop->nacc == 0 && same_span(c, loop->acc_start, loop->acc_end, start, end))) {
            return 1;
        }
        if (loop->nacc > 0) {
            uint8_t acc;
            for (acc = 0; acc < loop->nacc; acc++) {
                if (same_span(c, loop->an_start[acc], loop->an_end[acc], start, end)) {
                    return 1;
                }
            }
        }
    }
    return 0;
}

static void report_scope_duplicate(Compiler *c, uint32_t start, uint32_t end, uint32_t earlier_start,
                                   uint32_t earlier_end, const char *earlier_label) {
    char message[160];
    char spelling[64];
    span_copy(spelling, sizeof spelling, c->text, start, end);
    snprintf(message, sizeof message, "duplicate name `%s`", spelling);
    add_diag(c, "ORC0219", start, end, message, "this name repeats a name in scope",
             "each parameter, binding, loop index, and accumulator in scope has its own name; Orange has no shadowing",
             2);
    if (earlier_end > earlier_start) {
        diag_add_secondary(c, earlier_start, earlier_end, earlier_label);
    }
}

static int report_duplicate_loop_name(Compiler *c, uint32_t func_index, uint32_t locals_in_scope, uint32_t start,
                                      uint32_t end) {
    const Func *func = &c->funcs[func_index];
    uint16_t index;
    uint8_t size_slot = 0;
    uint32_t earlier_start = 0;
    uint32_t earlier_end = 0;
    const char *earlier_label = NULL;
    if (size_slot_of(c, func_index, start, end, &size_slot)) {
        earlier_start = func->sz_name0[size_slot];
        earlier_end = func->sz_name1[size_slot];
        earlier_label = "the size parameter is here";
    }
    for (index = 0; index < func->nparams && earlier_label == NULL; index++) {
        const Param *param = &c->params[func->param0 + index];
        if (!param->duplicate && same_span(c, param->name_start, param->name_end, start, end)) {
            earlier_start = param->name_start;
            earlier_end = param->name_end;
            earlier_label = "the parameter is here";
        }
    }
    if (locals_in_scope > func->nlocals) {
        locals_in_scope = func->nlocals;
    }
    for (index = 0; index < locals_in_scope && earlier_label == NULL; index++) {
        const Local *local = &c->locals[func->local0 + index];
        if (!local->duplicate && same_span(c, local->name_start, local->name_end, start, end)) {
            earlier_start = local->name_start;
            earlier_end = local->name_end;
            earlier_label = "the binding is here";
        }
    }
    for (index = 0; index < (uint16_t)c->nframes && earlier_label == NULL; index++) {
        const BlockFrame *block = &c->frames[index];
        uint16_t bind;
        for (bind = 0; bind < block->visible && earlier_label == NULL; bind++) {
            const Local *local = &c->block_locals[block->bind0 + bind];
            if (!local->duplicate && same_span(c, local->name_start, local->name_end, start, end)) {
                earlier_start = local->name_start;
                earlier_end = local->name_end;
                earlier_label = "the binding is here";
            }
        }
    }
    if (earlier_label == NULL) {
        int active;
        for (active = 0; active < c->nactive && earlier_label == NULL; active++) {
            const LoopDesc *loop = &c->loops[c->active_loops[active]];
            uint8_t acc;
            if (same_span(c, loop->index_start, loop->index_end, start, end)) {
                earlier_start = loop->index_start;
                earlier_end = loop->index_end;
                earlier_label = "the loop index is here";
            } else if (loop->nacc == 0 && same_span(c, loop->acc_start, loop->acc_end, start, end)) {
                earlier_start = loop->acc_start;
                earlier_end = loop->acc_end;
                earlier_label = "the accumulator is here";
            }
            for (acc = 0; loop->nacc > 0 && acc < loop->nacc && earlier_label == NULL; acc++) {
                if (same_span(c, loop->an_start[acc], loop->an_end[acc], start, end)) {
                    earlier_start = loop->an_start[acc];
                    earlier_end = loop->an_end[acc];
                    earlier_label = "the accumulator is here";
                }
            }
        }
    }
    if (earlier_label == NULL) {
        return 0;
    }
    report_scope_duplicate(c, start, end, earlier_start, earlier_end, earlier_label);
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

static const char IMPLICIT_NOTE[] = "Orange has no implicit conversions between types";
static const char ARRAY_OPERATOR_NOTE[] =
    "operators apply to `Int`, `Bool`, word, and residue values; apply them to elements, such as `x[0]`";
static const char TUPLE_OPERATOR_NOTE[] =
    "operators apply to `Int`, `Bool`, word, and residue values; apply them to elements, such as `p.0`";
static const char BOOL_OPERATOR_NOTE[] = "the operators on `Bool` are `!`, `&&`, `||`, `==`, and `!=`";
static const char SHIFT_AMOUNT_NOTE[] =
    "an amount written as one integer literal is from 0 through n - 1; any other amount is computed, an `Int` or a "
    "word, such as `x <<< r` or `x >> (i % 8)`";
static const char LOOP_RANGE_NOTE[] =
    "a loop `for i in a..b` runs once for each i from a up to b - 1, with a < b <= 65536";

static int format_type(Compiler *c, char *buffer, size_t cap, TypeKind type, uint32_t length, uint16_t mod_index);
static int format_tuple_type(Compiler *c, char *buffer, size_t cap, uint32_t tup0, uint16_t tup_n);

static void spell_type(const Compiler *owner, char *buffer, size_t cap, TypeKind type, uint32_t length, uint16_t mod,
                       uint32_t tup0, uint16_t tup_n) {
    int ok;
    Compiler *writable = (Compiler *)owner;
    if (writable != NULL && type == TY_TUPLE && tup_n > 0) {
        ok = format_tuple_type(writable, buffer, cap, tup0, tup_n);
    } else if (writable != NULL) {
        ok = format_type(writable, buffer, cap, type, length, type == TY_MOD ? mod : 0);
    } else {
        ok = 0;
    }
    if (!ok) {
        copy_text(buffer, cap, "?");
    }
}

static void spell_expected(Compiler *c, char *buffer, size_t cap, TypeKind type, uint32_t length) {
    spell_type(c, buffer, cap, type, length, c->expect_mod, c->expect_tup0, c->expect_tup_n);
}

static void add_expected(Compiler *c, uint32_t start, uint32_t end, const char *message, const char *expected_text,
                         const char *note) {
    char label[192];
    snprintf(label, sizeof label, "expected `%s`", expected_text);
    add_diag(c, "ORC0214", start, end, message, label, note, 2);
}

static const char *shift_name(TokenKind op) {
    switch (op) {
    case TK_LSHIFT: return "<<";
    case TK_RSHIFT: return ">>";
    case TK_ROL: return "<<<";
    case TK_ROR: return ">>>";
    default: return "operator";
    }
}

static void report_undefined_op(Compiler *c, uint32_t start, uint32_t end, const char *op, int prefix, TypeKind type,
                                uint32_t length, uint16_t mod, uint32_t tup0, uint16_t tup_n, const char *note) {
    char message[384];
    char type_text[96];
    char label[160];
    spell_type(c, type_text, sizeof type_text, type, length, mod, tup0, tup_n);
    if (prefix) {
        snprintf(message, sizeof message, "prefix `%s` is not defined for `%s`", op, type_text);
    } else {
        snprintf(message, sizeof message, "`%s` is not defined for `%s`", op, type_text);
    }
    snprintf(label, sizeof label, "`%s` is required here", type_text);
    add_diag(c, "ORC0215", start, end, message, label, note, 2);
}

static void report_name_mismatch(Compiler *c, uint32_t start, uint32_t end, TypeKind found, uint32_t found_len,
                                 uint16_t found_mod, uint32_t found_tup0, uint16_t found_tup_n, TypeKind expected,
                                 uint32_t expected_len) {
    char message[384];
    char spelling[64];
    char expected_text[96];
    char found_text[96];
    char note_buf[160];
    const char *note = IMPLICIT_NOTE;
    uint16_t index;
    span_copy(spelling, sizeof spelling, c->text, start, end);
    spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
    spell_type(c, found_text, sizeof found_text, found, found_len, found_mod, found_tup0, found_tup_n);
    snprintf(message, sizeof message, "`%s` has type `%s`, but `%s` is required here", spelling, found_text,
             expected_text);
    if (found_len != 0 && expected_len == 0 && found == expected &&
        !(found == TY_MOD && found_mod != c->expect_mod)) {
        snprintf(note_buf, sizeof note_buf, "select one element with an index, such as `%s[0]`", spelling);
        note = note_buf;
    } else if (found == TY_TUPLE && found_tup_n > 0 && expected != TY_TUPLE) {
        for (index = 0; index < found_tup_n; index++) {
            const TupleElem *elem = &c->telems[found_tup0 + index];
            if (elem->kind == expected && elem->length == expected_len &&
                (expected != TY_MOD || elem->mod_index == c->expect_mod)) {
                snprintf(note_buf, sizeof note_buf, "select one element by its position, such as `%s.0`", spelling);
                note = note_buf;
                break;
            }
        }
    }
    add_expected(c, start, end, message, expected_text, note);
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
        case TK_PLUSPLUS:
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
    case EX_NAME:
        if (c->cur_func < c->nfuncs && size_slot_of(c, c->cur_func, expr->name_start, expr->name_end, NULL)) {
            return 1;
        }
        *bad = index;
        return 0;
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
    if (expr->kind == EX_NAME) {
        uint8_t slot = 0;
        int64_t value;
        uint64_t magnitude;
        if (c->cur_func >= c->nfuncs || !size_slot_of(c, c->cur_func, expr->name_start, expr->name_end, &slot) ||
            slot >= c->ncur) {
            return 0;
        }
        value = c->cur_sz[slot];
        if (value >= 0) {
            return big_from_u64(&c->arena, (uint64_t)value, lo) && big_from_u64(&c->arena, (uint64_t)value, hi);
        }
        magnitude = value == INT64_MIN ? (uint64_t)INT64_MAX + 1u : (uint64_t)(-value);
        if (!big_from_u64(&c->arena, magnitude, lo) || !big_neg(lo, lo) || !big_from_u64(&c->arena, magnitude, hi) ||
            !big_neg(hi, hi)) {
            return 0;
        }
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
    char message[384];
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

static int block_name_taken(Compiler *c, uint32_t func_index, uint32_t locals_in_scope, uint32_t start, uint32_t end) {
    const Func *func = &c->funcs[func_index];
    uint16_t index;
    int frame;
    if (size_slot_of(c, func_index, start, end, NULL)) {
        return 1;
    }
    for (index = 0; index < func->nparams; index++) {
        const Param *param = &c->params[func->param0 + index];
        if (!param->duplicate && same_span(c, param->name_start, param->name_end, start, end)) {
            return 1;
        }
    }
    if (locals_in_scope > func->nlocals) {
        locals_in_scope = func->nlocals;
    }
    for (index = 0; index < locals_in_scope; index++) {
        const Local *local = &c->locals[func->local0 + index];
        if (!local->duplicate && same_span(c, local->name_start, local->name_end, start, end)) {
            return 1;
        }
    }
    for (frame = 0; frame < c->nframes; frame++) {
        uint16_t bind;
        for (bind = 0; bind < c->frames[frame].visible; bind++) {
            const Local *local = &c->block_locals[c->frames[frame].bind0 + bind];
            if (!local->duplicate && same_span(c, local->name_start, local->name_end, start, end)) {
                return 1;
            }
        }
    }
    return name_is_active_loop(c, start, end);
}

static int remember_block(Compiler *c, uint32_t bind0, uint16_t nbinds, uint32_t start, uint32_t end) {
    if (nbinds == 0) {
        return 1;
    }
    if (!ensure_cap((void **)&c->finished, &c->finished_cap, c->nfinished + 1, sizeof(FinishedBlock), MAX_EXPRS)) {
        resource_diag(c, "ORC0209", start, end, "semantic analysis could not retain block scopes");
        return 0;
    }
    c->finished[c->nfinished].bind0 = bind0;
    c->finished[c->nfinished].nbinds = nbinds;
    c->nfinished++;
    return 1;
}

/* Check one step or branch. Names are visible only after their own `;`
   and only inside this block. A rejected binding type does not also
   typecheck that binding's initializer. */
static int check_block(Compiler *c, uint32_t bind0, uint16_t nbinds, uint32_t value, TypeKind expected,
                       uint32_t expected_len, uint16_t expected_mod, uint32_t func_index, uint32_t locals_in_scope) {
    BlockFrame *frame;
    uint16_t bind;
    int ok;
    if (nbinds == 0) {
        if (value == UINT32_MAX) {
            return 1;
        }
        return check_at(c, value, expected, expected_len, expected_mod, func_index, locals_in_scope);
    }
    if (c->nframes >= MAX_OPEN_LOOPS) {
        resource_diag(c, "ORC0209", value == UINT32_MAX ? 0 : c->exprs[value].start, value == UINT32_MAX ? 0 : c->exprs[value].end,
                      "semantic analysis could not retain block scopes");
        return 0;
    }
    frame = &c->frames[c->nframes++];
    frame->bind0 = bind0;
    frame->nbinds = nbinds;
    frame->visible = 0;
    frame->slots = NULL;
    for (bind = 0; bind < nbinds; bind++) {
        Local *local = &c->block_locals[bind0 + bind];
        if (local->pat_i > 0) {
            continue;
        }
        if (local->pat_len > 0) {
            int bad_type = 0;
            uint16_t pat;
            for (pat = 0; pat < local->pat_len && bind + pat < nbinds; pat++) {
                Local *name = &c->block_locals[bind0 + bind + pat];
                uint16_t prev;
                int taken = block_name_taken(c, func_index, locals_in_scope, name->name_start, name->name_end);
                for (prev = 0; prev < pat && !taken; prev++) {
                    Local *before = &c->block_locals[bind0 + bind + prev];
                    if (!before->duplicate &&
                        same_span(c, before->name_start, before->name_end, name->name_start, name->name_end)) {
                        taken = 1;
                    }
                }
                if (taken) {
                    uint32_t within_start = 0;
                    uint32_t within_end = 0;
                    for (prev = 0; prev < pat; prev++) {
                        Local *before = &c->block_locals[bind0 + bind + prev];
                        if (!before->duplicate &&
                            same_span(c, before->name_start, before->name_end, name->name_start, name->name_end)) {
                            within_start = before->name_start;
                            within_end = before->name_end;
                            break;
                        }
                    }
                    name->duplicate = 1;
                    if (!report_duplicate_loop_name(c, func_index, locals_in_scope, name->name_start, name->name_end) &&
                        within_end > within_start) {
                        report_scope_duplicate(c, name->name_start, name->name_end, within_start, within_end,
                                               "the first name is here");
                    }
                }
                if (!name->type_ok) {
                    bad_type = 1;
                }
            }
            if (bad_type) {
                for (pat = 0; pat < local->pat_len && bind + pat < nbinds; pat++) {
                    Local *name = &c->block_locals[bind0 + bind + pat];
                    int unresolved = !name->type_ok;
                    name->type_ok = 0;
                    /* Only a name whose own type failed is diagnosed. A resolved
                       neighbor stays silent; marking it reported keeps a later
                       use of the whole pattern at NAME_BAD. */
                    if (unresolved && !name->type_reported) {
                        reject_declared(c, name->type, name->length_bad, name->type_start, name->type_end,
                                        name->length_start, name->length_end);
                    }
                    name->type_reported = 1;
                }
            } else if (!check_as_tuple(c, local->value, local->tup0, local->tup_n, func_index, locals_in_scope)) {
                c->nframes--;
                return 0;
            }
            frame->visible = (uint16_t)(bind + local->pat_len);
            bind = (uint16_t)(bind + local->pat_len - 1);
            continue;
        }
        if (block_name_taken(c, func_index, locals_in_scope, local->name_start, local->name_end)) {
            local->duplicate = 1;
            report_duplicate_loop_name(c, func_index, locals_in_scope, local->name_start, local->name_end);
        }
        if (!local->type_ok) {
            if (!local->type_reported) {
                reject_declared(c, local->type, local->length_bad, local->type_start, local->type_end,
                                local->length_start, local->length_end);
            }
        } else if (local->type == TY_TUPLE) {
            if (!check_as_tuple(c, local->value, local->tup0, local->tup_n, func_index, locals_in_scope)) {
                c->nframes--;
                return 0;
            }
        } else if (!check_at(c, local->value, local->type, local->length, local->mod_index, func_index,
                             locals_in_scope)) {
            c->nframes--;
            return 0;
        }
        frame->visible = (uint16_t)(bind + 1);
    }
    ok = value == UINT32_MAX || check_at(c, value, expected, expected_len, expected_mod, func_index, locals_in_scope);
    c->nframes--;
    if (!ok) {
        return 0;
    }
    return remember_block(c, bind0, nbinds, value == UINT32_MAX ? 0 : c->exprs[value].start,
                          value == UINT32_MAX ? 0 : c->exprs[value].end);
}

static int same_tuple(const Compiler *left_owner, uint32_t left0, uint16_t left_n, const Compiler *right_owner,
                     uint32_t right0, uint16_t right_n);
static int adopt_tuple_shape(Compiler *c, const Compiler *owner, uint32_t tup0, uint16_t tup_n, uint32_t *out0);

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
       ORC0225 on 70000. A bound written with sizes uses Euclidean arithmetic. */
    if (loop->a_sized || loop->b_sized) {
        int a_ok = 0;
        int b_ok = 0;
        int64_t a_size = 0;
        int64_t b_size = 0;
        bounds_decoded = 0;
        if (loop->a_sized) {
            Sz bound = eval_size(c, loop->a_expr);
            if (bound.kind != 0) {
                report_size_fault(c, bound);
            } else if (bound.value < 0) {
                add_diag(c, "ORC0225", loop->a_start, loop->a_end, "a loop bound must be at least 0", "loop bound",
                         "a loop runs over a nonempty range within 0 through 65536", 2);
            } else if (bound.value > (int64_t)MAX_LOOP_BOUND) {
                add_diag(c, "ORC0225", loop->a_start, loop->a_end, "a loop bound must be at most 65536", "loop bound",
                         "a loop runs over a nonempty range within 0 through 65536", 2);
            } else {
                a_size = bound.value;
                a_ok = 1;
            }
        } else {
            a_ok = decode_loop_bound(c, loop->a_start, loop->a_end, &a_above, &a_value);
            if (a_ok && !a_above) {
                a_size = (int64_t)a_value;
            } else if (a_ok && a_above) {
                add_diag(c, "ORC0225", loop->a_start, loop->a_end, "a loop bound must be at most 65536", "loop bound",
                         "a loop runs over a nonempty range within 0 through 65536", 2);
                a_ok = 0;
            }
        }
        if (a_ok && loop->b_sized) {
            Sz bound = eval_size(c, loop->b_expr);
            if (bound.kind != 0) {
                report_size_fault(c, bound);
            } else if (bound.value < 0) {
                add_diag(c, "ORC0225", loop->b_start, loop->b_end, "a loop bound must be at least 0", "loop bound",
                         "a loop runs over a nonempty range within 0 through 65536", 2);
            } else if (bound.value > (int64_t)MAX_LOOP_BOUND) {
                add_diag(c, "ORC0225", loop->b_start, loop->b_end, "a loop bound must be at most 65536", "loop bound",
                         "a loop runs over a nonempty range within 0 through 65536", 2);
            } else {
                b_size = bound.value;
                b_ok = 1;
            }
        } else if (a_ok) {
            b_ok = decode_loop_bound(c, loop->b_start, loop->b_end, &b_above, &b_value);
            if (b_ok && !b_above) {
                b_size = (int64_t)b_value;
            } else if (b_ok && b_above) {
                b_ok = 0;
                b_above = 1;
            }
        }
        if (a_ok && b_ok && a_size < b_size) {
            loop->bounds_ok = 1;
            loop->bound_a = (uint32_t)a_size;
            loop->bound_b = (uint32_t)b_size;
        } else if (a_ok && b_ok) {
            add_diag(c, "ORC0225", loop->b_start, loop->b_end,
                     "a loop range must be nonempty and within 0 through 65536", "loop bounds",
                     "write a..b with 0 <= a < b <= 65536", 2);
        } else if (a_ok && !loop->b_sized && b_above) {
            add_diag(c, "ORC0225", loop->b_start, loop->b_end,
                     "a loop range must be nonempty and within 0 through 65536", "loop bounds",
                     "write a..b with 0 <= a < b <= 65536", 2);
        }
    } else {
        bounds_decoded = decode_loop_bound(c, loop->a_start, loop->a_end, &a_above, &a_value) &&
                         decode_loop_bound(c, loop->b_start, loop->b_end, &b_above, &b_value);
    }
    if (bounds_decoded && a_above) {
        add_diag(c, "ORC0225", loop->a_start, loop->a_end, "a loop bound must be at most 65536", "loop bound too large",
                 LOOP_RANGE_NOTE, 2);
    } else if (bounds_decoded && b_above) {
        add_diag(c, "ORC0225", loop->b_start, loop->b_end, "a loop bound must be at most 65536", "loop bound too large",
                 LOOP_RANGE_NOTE, 2);
    } else if (bounds_decoded && a_value >= b_value) {
        char message[64];
        snprintf(message, sizeof message, "the loop range %u..%u is empty", a_value, b_value);
        add_diag(c, "ORC0225", loop->b_start, loop->b_end, message, "a loop runs at least once", LOOP_RANGE_NOTE, 2);
    } else if (bounds_decoded) {
        loop->bounds_ok = 1;
        loop->bound_a = a_value;
        loop->bound_b = b_value;
    }
    index_dup = report_duplicate_loop_name(c, func_index, locals_in_scope, loop->index_start, loop->index_end);
    if (loop->nacc > 0) {
        uint8_t acc;
        acc_dup = 0;
        for (acc = 0; acc < loop->nacc; acc++) {
            int taken = 0;
            uint8_t earlier;
            if (same_span(c, loop->index_start, loop->index_end, loop->an_start[acc], loop->an_end[acc])) {
                taken = 1;
                report_scope_duplicate(c, loop->an_start[acc], loop->an_end[acc], loop->index_start, loop->index_end,
                                       "the loop index is here");
            }
            for (earlier = 0; earlier < acc && !taken; earlier++) {
                if (same_span(c, loop->an_start[earlier], loop->an_end[earlier], loop->an_start[acc],
                              loop->an_end[acc])) {
                    taken = 1;
                    report_scope_duplicate(c, loop->an_start[acc], loop->an_end[acc], loop->an_start[earlier],
                                           loop->an_end[earlier], "the first name is here");
                }
            }
            if (!taken &&
                report_duplicate_loop_name(c, func_index, locals_in_scope, loop->an_start[acc], loop->an_end[acc])) {
                taken = 1;
            }
            if (taken) {
                acc_dup = 1;
            }
        }
    } else if (same_span(c, loop->index_start, loop->index_end, loop->acc_start, loop->acc_end)) {
        acc_dup = 1;
        report_scope_duplicate(c, loop->acc_start, loop->acc_end, loop->index_start, loop->index_end,
                               "the loop index is here");
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
    {
        int shape_bad = loop->acc_type == TY_TUPLE && expected == TY_TUPLE && c->expect_tup_n > 0 && loop->tup_n > 0 &&
                        !same_tuple(c, c->expect_tup0, c->expect_tup_n, c, loop->tup0, loop->tup_n);
        if (loop->acc_type != expected || loop->acc_len != expected_len || shape_bad ||
            (loop->acc_type == TY_MOD && expected == TY_MOD && loop->acc_mod != c->expect_mod)) {
            char message[384];
            char expected_text[96];
            char found_text[96];
            spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
            spell_type(c, found_text, sizeof found_text, loop->acc_type, loop->acc_len, loop->acc_mod, loop->tup0,
                       loop->tup_n);
            snprintf(message, sizeof message, "this loop has type `%s`, but `%s` is required here", found_text,
                     expected_text);
            add_expected(c, expr->start, expr->end, message, expected_text,
                         "a loop's value is its accumulator after the last step");
        }
    }
    {
        uint32_t saved_tup0 = c->expect_tup0;
        uint16_t saved_tup_n = c->expect_tup_n;
        int init_ok = 1;
        int step_ok = 1;
        if (loop->acc_type == TY_TUPLE) {
            c->expect_tup0 = loop->tup0;
            c->expect_tup_n = loop->tup_n;
        }
        if (loop->init_expr != UINT32_MAX) {
            init_ok = check_at(c, loop->init_expr, loop->acc_type, loop->acc_len, loop->acc_mod, func_index,
                               locals_in_scope);
        }
        if (init_ok && loop->bounds_ok && loop->step_expr != UINT32_MAX) {
            if (c->nactive >= MAX_OPEN_LOOPS) {
                resource_diag(c, "ORC0209", expr->start, expr->end, "semantic analysis could not retain loop scopes");
                c->expect_tup0 = saved_tup0;
                c->expect_tup_n = saved_tup_n;
                return 0;
            }
            c->active_loops[c->nactive++] = expr->arg0;
            step_ok = check_block(c, loop->bind0, loop->nbinds, loop->step_expr, loop->acc_type, loop->acc_len,
                                  loop->acc_mod, func_index, locals_in_scope);
            c->nactive--;
        }
        c->expect_tup0 = saved_tup0;
        c->expect_tup_n = saved_tup_n;
        return init_ok && step_ok;
    }
}

static int same_tuple(const Compiler *left_owner, uint32_t left0, uint16_t left_n, const Compiler *right_owner,
                     uint32_t right0, uint16_t right_n) {
    uint16_t index;
    if (left_n != right_n || left_owner == NULL || right_owner == NULL) {
        return 0;
    }
    for (index = 0; index < left_n; index++) {
        const TupleElem *left = &left_owner->telems[left0 + index];
        const TupleElem *right = &right_owner->telems[right0 + index];
        uint16_t mod = right->mod_index;
        if (left->kind != right->kind || left->length != right->length) {
            return 0;
        }
        if (left->kind == TY_MOD) {
            if (!adopt_modulus((Compiler *)left_owner, right_owner, right->mod_index, &mod)) {
                return 0;
            }
            if (mod != left->mod_index) {
                return 0;
            }
        }
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
    uint32_t saved_tup0 = c->expect_tup0;
    uint16_t saved_tup_n = c->expect_tup_n;
    int ok;
    c->expect_mod = expected == TY_MOD ? expected_mod : 0;
    if (expected != TY_TUPLE) {
        c->expect_tup0 = 0;
        c->expect_tup_n = 0;
    }
    ok = check_expr(c, index, expected, expected_len, func_index, locals_in_scope);
    c->expect_mod = saved;
    c->expect_tup0 = saved_tup0;
    c->expect_tup_n = saved_tup_n;
    return ok;
}

/* Check `index` as the tuple shape `(tup0, tup_n)` without leaving that shape
   as the required shape of whatever is checked next. */
static int check_as_tuple(Compiler *c, uint32_t index, uint32_t tup0, uint16_t tup_n, uint32_t func_index,
                          uint32_t locals_in_scope) {
    uint32_t saved0 = c->expect_tup0;
    uint16_t saved_n = c->expect_tup_n;
    int ok;
    c->expect_tup0 = tup0;
    c->expect_tup_n = tup_n;
    ok = check_at(c, index, TY_TUPLE, 0, 0, func_index, locals_in_scope);
    c->expect_tup0 = saved0;
    c->expect_tup_n = saved_n;
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
    case TK_PLUSPLUS: return "++";
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
        char message[384];
        char expected_text[96];
        spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
        snprintf(message, sizeof message, "a comparison gives `Bool`, but `%s` is required here", expected_text);
        add_expected(c, expr->start, expr->end, message, expected_text,
                     "a conditional `if c { a } else { b }` chooses a value by a `Bool`");
    }
    state = find_leaf(c, expr->left, func_index, locals_in_scope, &operand, &operand_len, &leaf, &silent);
    if (state == 0 || state == 2) {
        int left_state = state;
        TypeKind right_type = TY_NONE;
        uint32_t right_len = 0;
        uint32_t right_leaf = 0;
        int right_silent = 0;
        int right_state = find_leaf(c, expr->right, func_index, locals_in_scope, &right_type, &right_len, &right_leaf,
                                    &right_silent);
        if (right_state == 1) {
            state = 1;
            operand = right_type;
            operand_len = right_len;
            leaf = right_leaf;
            silent = right_silent;
        } else if (left_state == 2 || right_state == 2) {
            char message[384];
            if (order) {
                snprintf(message, sizeof message, "`%s` is not defined for arrays and tuples", op_spelling(expr->op));
                add_diag(c, "ORC0215", expr->op_start, expr->op_end, message,
                         "both operands are written out as arrays or tuples",
                         "arrays and tuples are compared whole with `==` and `!=`; they have no order, so compare "
                         "elements",
                         2);
            } else {
                snprintf(message, sizeof message, "the operands of `%s` have no type of their own",
                         op_spelling(expr->op));
                add_diag(c, "ORC0227", expr->start, expr->end, message,
                         "an array or tuple written out takes its type from where it is used",
                         "compare with a typed operand, such as a name, or give one side a type with a `let` binding",
                         2);
            }
            return 1;
        } else if (right_state < 0) {
            state = right_state;
            operand = right_type;
            operand_len = right_len;
            leaf = right_leaf;
            silent = right_silent;
        } else {
            state = 0;
        }
    }
    if (state == 0) {
        char message[160];
        snprintf(message, sizeof message, "the operands of `%s` have no type of their own", op_spelling(expr->op));
        add_diag(c, "ORC0227", expr->start, expr->end, message, "a literal takes its type from where it is used",
                 "compare with a typed operand, such as a name, or give the literal a type with a `let` binding", 2);
        return 1;
    }
    if (state < 0) {
        if (!silent) {
            return check_expr(c, leaf, operand == TY_NONE ? TY_INT : operand, 0, func_index, locals_in_scope);
        }
        return 1;
    }
    if (order && (operand == TY_MOD || operand_len != 0 || operand == TY_BOOL || operand == TY_TUPLE ||
                  !is_scalar_type(operand))) {
        char message[384];
        char found[96];
        char label[160];
        const char *note;
        spell_type(c, found, sizeof found, operand, operand_len, operand == TY_MOD ? c->leaf_mod : 0,
                   operand == TY_TUPLE ? c->leaf_tup0 : 0, operand == TY_TUPLE ? c->leaf_tup_n : 0);
        snprintf(message, sizeof message, "`%s` is not defined for `%s`", op_spelling(expr->op), found);
        snprintf(label, sizeof label, "the operands have type `%s`", found);
        if (operand == TY_MOD) {
            note = "residues are compared with `==` and `!=`; they have no order, so compare least residues, such as "
                   "`(x as Int) < (y as Int)`";
        } else if (operand == TY_BOOL) {
            note = "`Bool` values are compared with `==` and `!=`; they have no order";
        } else if (operand == TY_TUPLE) {
            note = "tuples are compared whole with `==` and `!=`; they have no order, so compare elements, such as "
                   "`p.0 < q.0`";
        } else {
            note = "arrays are compared whole with `==` and `!=`; they have no order, so compare elements, such as "
                   "`x[0] < y[0]`";
        }
        add_diag(c, "ORC0215", expr->op_start, expr->op_end, message, label, note, 2);
        return 1;
    }
    if (operand == TY_TUPLE && c->leaf_tup_n > 0) {
        uint32_t shape = c->leaf_tup0;
        const Compiler *owner = c->leaf_owner == NULL ? c : c->leaf_owner;
        if (!adopt_tuple_shape(c, owner, c->leaf_tup0, c->leaf_tup_n, &shape)) {
            return 0;
        }
        c->expect_tup0 = shape;
        c->expect_tup_n = c->leaf_tup_n;
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
        char message[384];
        char found[96];
        char label[128];
        spell_type(c, found, sizeof found, base_kind, 0, c->leaf_mod, c->leaf_tup0, c->leaf_tup_n);
        snprintf(message, sizeof message, "only an array can be indexed, but this has type `%s`", found);
        if (base_kind == TY_TUPLE) {
            snprintf(label, sizeof label, "`%s` is a tuple, not an array", found);
            add_diag(c, "ORC0224", c->exprs[base].start, c->exprs[base].end, message, label,
                     "a tuple's element is selected by its position, such as `p.0`", 2);
        } else {
            snprintf(label, sizeof label, "`%s` has no elements", found);
            add_diag(c, "ORC0224", c->exprs[base].start, c->exprs[base].end, message, label,
                     "an index selects one element of a value of type `T^n`", 2);
        }
    }
    return check_expr(c, base, base_kind, 0, func_index, locals_in_scope);
}

static const char STATIC_SLICE_NOTE[] =
    "a slice's position never depends on data: its bounds are built from integer literals and loop indices with "
    "`+`, `-`, and `*` by a constant";
static const char SLICE_LENGTH_NOTE[] =
    "a slice `x[a..b]` holds the b - a elements from index a, and b - a must be the same positive number at every "
    "step, as in `x[16 * i..16 * i + 16]`";
static const char CONCAT_NOTE[] = "`a ++ b` is the array of the elements of a followed by those of b, of one element type";

static void report_bound_form(Compiler *c, int status, uint32_t bad_start, uint32_t bad_end, uint32_t bound_start,
                              uint32_t bound_end) {
    if (status == AFF_PRODUCT) {
        add_diag(c, "ORC0226", bad_start, bad_end, "a slice's bound may multiply a loop index only by a constant",
                 "both operands of this `*` use a loop index", STATIC_SLICE_NOTE, 2);
        return;
    }
    if (status == AFF_NOT_STATIC) {
        add_diag(c, "ORC0226", bad_start, bad_end, "a slice's bounds may use only integer literals and loop indices",
                 "this is neither", STATIC_SLICE_NOTE, 2);
        return;
    }
    add_diag(c, "ORC0223", bound_start, bound_end,
             "a part of this bound exceeds the 16384-significant-bit limit of `Int`",
             "this bound has no representable range", STATIC_SLICE_NOTE, 2);
}

/* 1 when the slice length is proved in range. 0 when a diagnostic was reported. */
static int prove_slice(Compiler *c, uint32_t start_expr, uint32_t end_expr, uint32_t range_start, uint32_t range_end,
                       uint32_t base_len, TypeKind element, uint32_t *out_len) {
    AffineForm start_form;
    AffineForm end_form;
    AffineForm difference;
    uint32_t bad_start = 0;
    uint32_t bad_end = 0;
    int status;
    int64_t start_low = 0;
    int64_t start_high = 0;
    int64_t end_low = 0;
    int64_t end_high = 0;
    char array_text[64];
    memset(&start_form, 0, sizeof start_form);
    memset(&end_form, 0, sizeof end_form);
    if (start_expr == UINT32_MAX) {
        start_form.constant = 0;
    } else {
        status = affine_form(c, start_expr, &start_form, &bad_start, &bad_end);
        if (status != AFF_OK) {
            report_bound_form(c, status, bad_start, bad_end, c->exprs[start_expr].start, c->exprs[start_expr].end);
            return 0;
        }
    }
    if (end_expr == UINT32_MAX) {
        end_form.constant = (int64_t)base_len;
    } else {
        status = affine_form(c, end_expr, &end_form, &bad_start, &bad_end);
        if (status != AFF_OK) {
            report_bound_form(c, status, bad_start, bad_end, c->exprs[end_expr].start, c->exprs[end_expr].end);
            return 0;
        }
    }
    if (!affine_combine(&difference, &end_form, &start_form, 1)) {
        add_diag(c, "ORC0223", range_start, range_end,
                 "a part of this bound exceeds the 16384-significant-bit limit of `Int`",
                 "this bound has no representable range", STATIC_SLICE_NOTE, 2);
        return 0;
    }
    if (difference.nterms != 0 || difference.constant < 1) {
        char message[160];
        const char *label = "a slice holds at least one element";
        if (difference.nterms != 0) {
            snprintf(message, sizeof message, "the length of this slice changes from step to step");
            label = "its bounds must differ by the same number at every step";
        } else if (difference.constant == 0) {
            snprintf(message, sizeof message, "this slice is empty: its bounds are equal");
        } else if (difference.constant > INT64_MIN) {
            uint64_t apart = (uint64_t)(-difference.constant);
            snprintf(message, sizeof message, "this slice ends %llu %s before it starts", (unsigned long long)apart,
                     apart == 1 ? "element" : "elements");
        } else {
            snprintf(message, sizeof message, "this slice's bounds are too far apart");
        }
        add_diag(c, "ORC0236", range_start, range_end, message, label, SLICE_LENGTH_NOTE, 2);
        return 0;
    }
    if (!affine_range(c, &start_form, &start_low, &start_high) || !affine_range(c, &end_form, &end_low, &end_high)) {
        add_diag(c, "ORC0223", range_start, range_end,
                 "a part of this bound exceeds the 16384-significant-bit limit of `Int`",
                 "this bound has no representable range", STATIC_SLICE_NOTE, 2);
        return 0;
    }
    write_type(array_text, sizeof array_text, element, base_len);
    if (start_low < 0 || end_high > (int64_t)base_len) {
        char message[384];
        char label[96];
        int64_t last = end_high == INT64_MIN ? end_high : end_high - 1;
        uint32_t highest = base_len == 0 ? 0 : base_len - 1u;
        snprintf(message, sizeof message, "this slice reaches elements %lld through %lld, out of range for `%s`",
                 (long long)start_low, (long long)last, array_text);
        snprintf(label, sizeof label, "indices run from 0 through %u", highest);
        add_diag(c, "ORC0223", range_start, range_end, message, label,
                 "every element a slice can take, over every loop index in its bounds, must be an element of the array",
                 2);
        return 0;
    }
    if (difference.constant > (int64_t)MAX_ARRAY_LENGTH) {
        add_diag(c, "ORC0236", range_start, range_end, "this slice's bounds are too far apart",
                 "a slice holds at least one element", SLICE_LENGTH_NOTE, 2);
        return 0;
    }
    *out_len = (uint32_t)difference.constant;
    return 1;
}

static int check_owned(Compiler *c, uint32_t index, TypeKind kind, uint32_t length, uint16_t mod, uint32_t tup0,
                       uint16_t tup_n, const Compiler *owner, uint32_t func_index, uint32_t locals_in_scope) {
    if (kind == TY_TUPLE) {
        uint32_t shape = tup0;
        if (tup_n > 0 && !adopt_tuple_shape(c, owner == NULL ? c : owner, tup0, tup_n, &shape)) {
            return 0;
        }
        return check_as_tuple(c, index, shape, tup_n, func_index, locals_in_scope);
    }
    if (kind == TY_MOD) {
        return check_at(c, index, kind, length, mod, func_index, locals_in_scope);
    }
    return check_expr(c, index, kind, length, func_index, locals_in_scope);
}

static int check_bytes(Compiler *c, uint32_t index, TypeKind expected, uint32_t expected_len) {
    Expr *expr = &c->exprs[index];
    uint32_t count = 0;
    uint32_t err_start = 0;
    uint32_t err_end = 0;
    uint32_t point = 0;
    int status = decode_bytes(c, expr, NULL, &count, &err_start, &err_end, &point);
    char message[384];
    char expected_text[64];
    if (status == BYTES_UNPRINTABLE) {
        unsigned char encoded[4];
        int nbytes = 1;
        char hex[16];
        char label[80];
        int used = 0;
        int byte;
        snprintf(message, sizeof message, "U+%04X is not a printable ASCII character", point);
        if (point < 0x80u) {
            encoded[0] = (unsigned char)point;
        } else if (point < 0x800u) {
            encoded[0] = (unsigned char)(0xC0u | (point >> 6));
            encoded[1] = (unsigned char)(0x80u | (point & 0x3fu));
            nbytes = 2;
        } else if (point < 0x10000u) {
            encoded[0] = (unsigned char)(0xE0u | (point >> 12));
            encoded[1] = (unsigned char)(0x80u | ((point >> 6) & 0x3fu));
            encoded[2] = (unsigned char)(0x80u | (point & 0x3fu));
            nbytes = 3;
        } else {
            encoded[0] = (unsigned char)(0xF0u | (point >> 18));
            encoded[1] = (unsigned char)(0x80u | ((point >> 12) & 0x3fu));
            encoded[2] = (unsigned char)(0x80u | ((point >> 6) & 0x3fu));
            encoded[3] = (unsigned char)(0x80u | (point & 0x3fu));
            nbytes = 4;
        }
        for (byte = 0; byte < nbytes && used < (int)sizeof hex; byte++) {
            used += snprintf(hex + used, sizeof hex - (size_t)used, "%s%02x", byte == 0 ? "" : " ", encoded[byte]);
        }
        snprintf(label, sizeof label, point <= 0x7fu ? "its byte is written `hex\"%s\"`" : "its UTF-8 bytes are written `hex\"%s\"`",
                 hex);
        add_diag(c, "ORC0235", err_start, err_end, message, label,
                 "a byte string's characters are its bytes, so each is printable ASCII, from ` ` through `~`; write any other byte as an escape, or in a hex string joined with `++`",
                 2);
        return 1;
    }
    if (status != BYTES_OK) {
        if (status == BYTES_EMPTY) {
            add_diag(c, "ORC0221", expr->start, expr->end, "a byte string holds at least one byte", "this string is empty",
                     "a byte string is an array `Word[8]^n` of 1 through 65536 bytes; join longer runs with `++`", 2);
        } else {
            add_diag(c, "ORC0221", expr->start, expr->end, "a byte string holds at most 65536 bytes",
                     "this string holds more than 65536",
                     "a byte string is an array `Word[8]^n` of 1 through 65536 bytes; join longer runs with `++`", 2);
        }
        return 1;
    }
    if (expected == TY_W8 && expected_len == count) {
        return 1;
    }
    write_type(expected_text, sizeof expected_text, expected, expected_len);
    if (expected == TY_W8 && expected_len != 0) {
        char label[64];
        snprintf(message, sizeof message, "this byte string holds %u %s, but `%s` has %u", count,
                 count == 1 ? "byte" : "bytes", expected_text, expected_len);
        snprintf(label, sizeof label, "expected %u bytes", expected_len);
        add_diag(c, "ORC0222", expr->start, expr->end, message, label,
                 "a byte string is the array `Word[8]^n` of its n bytes", 2);
    } else {
        char label[96];
        snprintf(message, sizeof message, "this byte string has type `Word[8]^%u`, but `%s` is required here", count,
                 expected_text);
        snprintf(label, sizeof label, "expected `%s`", expected_text);
        add_diag(c, "ORC0214", expr->start, expr->end, message, label,
                 "a byte string is the array `Word[8]^n` of its n bytes", 2);
    }
    return 1;
}

static int known_non_array(Compiler *c, uint32_t index, uint32_t func_index, uint32_t locals_in_scope, TypeKind *type) {
    uint32_t length = 0;
    uint32_t leaf = 0;
    int silent = 0;
    int state = find_leaf(c, index, func_index, locals_in_scope, type, &length, &leaf, &silent);
    return state == 1 && length == 0;
}

static int check_concat(Compiler *c, uint32_t index, TypeKind expected, uint32_t expected_len, uint32_t func_index,
                        uint32_t locals_in_scope) {
    Expr *expr = &c->exprs[index];
    int have_left = 0;
    int have_right = 0;
    int left_elem = 0;
    int right_elem = 0;
    uint32_t left_len = 0;
    uint32_t right_len = 0;
    TypeKind left_ty = TY_NONE;
    TypeKind right_ty = TY_NONE;
    TypeKind scalar = TY_NONE;
    char expected_text[64];
    char message[384];
    write_type(expected_text, sizeof expected_text, expected, expected_len);
    if (expected_len == 0) {
        char label[96];
        snprintf(message, sizeof message, "`++` joins arrays, but `%s` is required here", expected_text);
        snprintf(label, sizeof label, "`%s` is not an array type", expected_text);
        add_diag(c, "ORC0214", expr->op_start, expr->op_end, message, label, CONCAT_NOTE, 2);
        return 1;
    }
    array_parts(c, expr->left, func_index, locals_in_scope, &have_left, &left_len, &left_elem, &left_ty);
    if (!have_left) {
        if (known_non_array(c, expr->left, func_index, locals_in_scope, &scalar)) {
            char found[64];
            write_type(found, sizeof found, scalar, 0);
            char label[96];
            snprintf(message, sizeof message, "only arrays can be joined, but this has type `%s`", found);
            snprintf(label, sizeof label, "`%s` is not an array", found);
            add_diag(c, "ORC0224", c->exprs[expr->left].start, c->exprs[expr->left].end, message, label, CONCAT_NOTE,
                     2);
            return 1;
        }
        {
            uint32_t before = c->ndiags;
            if (!check_expr(c, expr->left, expected, expected_len, func_index, locals_in_scope)) {
                return 0;
            }
            if (c->ndiags == before) {
                char label[64];
                snprintf(message, sizeof message,
                         "the left operand of `++` has %u elements, leaving none of the %u of `%s` for the right",
                         expected_len, expected_len, expected_text);
                snprintf(label, sizeof label, "expected %u elements in all", expected_len);
                add_diag(c, "ORC0222", expr->op_start, expr->op_end, message, label, CONCAT_NOTE, 2);
            }
        }
        return 1;
    }
    array_parts(c, expr->right, func_index, locals_in_scope, &have_right, &right_len, &right_elem, &right_ty);
    if (!have_right && known_non_array(c, expr->right, func_index, locals_in_scope, &scalar)) {
        char found[64];
        write_type(found, sizeof found, scalar, 0);
        char label[96];
        snprintf(message, sizeof message, "only arrays can be joined, but this has type `%s`", found);
        snprintf(label, sizeof label, "`%s` is not an array", found);
        add_diag(c, "ORC0224", c->exprs[expr->right].start, c->exprs[expr->right].end, message, label, CONCAT_NOTE, 2);
        return 1;
    }
    if (have_right) {
        unsigned long long total = (unsigned long long)left_len + right_len;
        if (total != expected_len) {
            char label[64];
            snprintf(message, sizeof message, "`++` joins %u and %u elements, %llu in all, but `%s` has %u", left_len,
                     right_len, total, expected_text, expected_len);
            snprintf(label, sizeof label, "expected %u elements in all", expected_len);
            add_diag(c, "ORC0222", expr->op_start, expr->op_end, message, label, CONCAT_NOTE, 2);
            return 1;
        }
    } else if (left_len >= expected_len) {
        char label[64];
        snprintf(message, sizeof message,
                 "the left operand of `++` has %u elements, leaving none of the %u of `%s` for the right", left_len,
                 expected_len, expected_text);
        snprintf(label, sizeof label, "expected %u elements in all", expected_len);
        add_diag(c, "ORC0222", expr->op_start, expr->op_end, message, label, CONCAT_NOTE, 2);
        return 1;
    }
    if (!check_expr(c, expr->left, expected, left_len, func_index, locals_in_scope)) {
        return 0;
    }
    return check_expr(c, expr->right, expected, expected_len - left_len, func_index, locals_in_scope);
}

static int check_written_bounds(Compiler *c, uint32_t start_expr, uint32_t end_expr, uint32_t func_index,
                                uint32_t locals_in_scope) {
    uint32_t before = c->ndiags;
    if (start_expr != UINT32_MAX && !check_expr(c, start_expr, TY_INT, 0, func_index, locals_in_scope)) {
        return 0;
    }
    if (end_expr != UINT32_MAX && !check_expr(c, end_expr, TY_INT, 0, func_index, locals_in_scope)) {
        return 0;
    }
    return c->ndiags == before;
}

static int check_slice(Compiler *c, uint32_t index, TypeKind expected, uint32_t expected_len, uint32_t func_index,
                       uint32_t locals_in_scope) {
    Expr *expr = &c->exprs[index];
    TypeKind base_kind = TY_NONE;
    uint32_t base_len = 0;
    uint32_t leaf = 0;
    int silent = 0;
    int state = find_leaf(c, expr->left, func_index, locals_in_scope, &base_kind, &base_len, &leaf, &silent);
    uint16_t base_mod = c->leaf_mod;
    uint32_t tup0 = c->leaf_tup0;
    uint16_t tup_n = c->leaf_tup_n;
    const Compiler *owner = c->leaf_owner == NULL ? c : c->leaf_owner;
    int element_ok;
    uint32_t slice_len = 0;
    int bounds_ok;
    if (state != 1) {
        if (silent) {
            return 1;
        }
        return check_expr(c, expr->left, expected, expected_len, func_index, locals_in_scope);
    }
    if (base_len == 0) {
        char message[384];
        char found[96];
        char label[128];
        spell_type(owner, found, sizeof found, base_kind, 0, base_mod, tup0, tup_n);
        snprintf(message, sizeof message, "only an array can be sliced, but this has type `%s`", found);
        if (base_kind == TY_TUPLE) {
            snprintf(label, sizeof label, "`%s` is a tuple, not an array", found);
        } else {
            snprintf(label, sizeof label, "`%s` has no elements", found);
        }
        add_diag(c, "ORC0224", c->exprs[expr->left].start, c->exprs[expr->left].end, message, label,
                 "a slice `x[a..b]` is taken from a value of type `T^n`", 2);
        return check_owned(c, expr->left, base_kind, 0, base_mod, tup0, tup_n, owner, func_index, locals_in_scope);
    }
    element_ok = expected_len != 0 && expected == base_kind;
    if (!element_ok) {
        char message[384];
        char expected_text[64];
        char element_text[64];
        write_type(expected_text, sizeof expected_text, expected, expected_len);
        write_type(element_text, sizeof element_text, base_kind, 0);
        if (expected_len != 0) {
            snprintf(message, sizeof message, "this slice is an array of `%s`, but `%s` is required here", element_text,
                     expected_text);
        } else {
            snprintf(message, sizeof message, "a slice is an array, but `%s` is required here", expected_text);
        }
        {
            char label[128];
            snprintf(label, sizeof label, "expected `%s`", expected_text);
            add_diag(c, "ORC0214", expr->start, expr->end, message, label,
                     expected_len != 0 ? "a slice is an array of the elements of the array it is taken from"
                                       : "one element is selected by an index, such as `x[0]`",
                     2);
        }
    }
    if (!check_owned(c, expr->left, base_kind, base_len, base_mod, tup0, tup_n, owner, func_index, locals_in_scope)) {
        return 0;
    }
    bounds_ok = check_written_bounds(c, expr->right, expr->callee, func_index, locals_in_scope);
    if (!bounds_ok) {
        return c->resource || c->sema_limited ? 0 : 1;
    }
    if (!prove_slice(c, expr->right, expr->callee, expr->lit_start, expr->lit_end, base_len, base_kind, &slice_len)) {
        return 1;
    }
    if (element_ok && slice_len != expected_len) {
        char message[384];
        char expected_text[64];
        write_type(expected_text, sizeof expected_text, expected, expected_len);
        char label[64];
        snprintf(message, sizeof message, "this slice has %u %s, but `%s` has %u", slice_len,
                 slice_len == 1 ? "element" : "elements", expected_text, expected_len);
        snprintf(label, sizeof label, "expected %u elements", expected_len);
        add_diag(c, "ORC0222", expr->start, expr->end, message, label, SLICE_LENGTH_NOTE, 2);
    }
    return 1;
}

static int check_slice_update(Compiler *c, uint32_t index, TypeKind expected, uint32_t expected_len, uint32_t func_index,
                              uint32_t locals_in_scope) {
    Expr *expr = &c->exprs[index];
    TypeKind base_kind = TY_NONE;
    uint32_t base_len = 0;
    uint32_t leaf = 0;
    int silent = 0;
    int state = find_leaf(c, expr->left, func_index, locals_in_scope, &base_kind, &base_len, &leaf, &silent);
    uint16_t base_mod = c->leaf_mod;
    uint32_t tup0 = c->leaf_tup0;
    uint16_t tup_n = c->leaf_tup_n;
    const Compiler *owner = c->leaf_owner == NULL ? c : c->leaf_owner;
    uint32_t slice_len = 0;
    char message[384];
    char expected_text[64];
    if (state == 1 && base_len == 0) {
        char found[96];
        char label[128];
        spell_type(owner, found, sizeof found, base_kind, 0, base_mod, tup0, tup_n);
        snprintf(message, sizeof message, "only an array can be updated, but this has type `%s`", found);
        if (base_kind == TY_TUPLE) {
            snprintf(label, sizeof label, "`%s` is a tuple, not an array", found);
        } else {
            snprintf(label, sizeof label, "`%s` has no elements", found);
        }
        add_diag(c, "ORC0224", c->exprs[expr->left].start, c->exprs[expr->left].end, message, label,
                 base_kind == TY_TUPLE ? "a tuple with elements replaced is written anew, such as `(v, p.1)`"
                                       : SLICE_UPDATE_NOTE,
                 2);
        return check_owned(c, expr->left, base_kind, 0, base_mod, tup0, tup_n, owner, func_index, locals_in_scope);
    }
    if (expected_len == 0) {
        write_type(expected_text, sizeof expected_text, expected, 0);
        char label[96];
        snprintf(message, sizeof message, "an update gives an array, but `%s` is required here", expected_text);
        snprintf(label, sizeof label, "expected `%s`", expected_text);
        add_diag(c, "ORC0214", expr->start, expr->end, message, label, SLICE_UPDATE_NOTE, 2);
        return 1;
    }
    if (!check_expr(c, expr->left, expected, expected_len, func_index, locals_in_scope)) {
        return 0;
    }
    if (!check_written_bounds(c, expr->right, expr->conv_site, func_index, locals_in_scope)) {
        return c->resource || c->sema_limited ? 0 : 1;
    }
    if (!prove_slice(c, expr->right, expr->conv_site, expr->lit_start, expr->lit_end, expected_len, expected,
                     &slice_len)) {
        return 1;
    }
    return check_expr(c, expr->callee, expected, slice_len, func_index, locals_in_scope);
}

static void copy_func_name(const Compiler *c, const Func *func, char *name, size_t cap) {
    span_copy(name, cap, c->text, func->name_start, func->name_end);
}

static int lookup_call(Compiler *c, Expr *expr, Compiler *target, uint32_t callee, int report, uint32_t caller_func,
                      uint32_t locals, uint32_t *inst_id) {
    Func *func = &target->funcs[callee];
    char name[64];
    uint8_t index;
    copy_func_name(target, func, name, sizeof name);
    /* A call that writes no sizes selects the instance whose array
       parameters match the argument lengths. A nonzero count that does
       not match is ORC0239, including sizes given to a function that
       has none. */
    if (expr->nsize != 0 && expr->nsize != func->nsizes) {
        if (report) {
            char message[384];
            if (func->nsizes == 0) {
                snprintf(message, sizeof message, "`%s` has no size parameters, but this call gives %u size%s", name,
                         expr->nsize, expr->nsize == 1 ? "" : "s");
            } else if (expr->nsize == 0) {
                snprintf(message, sizeof message, "`%s` takes %u size%s, but this call gives none", name, func->nsizes,
                         func->nsizes == 1 ? "" : "s");
            } else {
                snprintf(message, sizeof message, "`%s` takes %u size%s, but this call gives %u", name, func->nsizes,
                         func->nsizes == 1 ? "" : "s", expr->nsize);
            }
            add_diag(c, "ORC0239", expr->start, expr->end, message, "wrong number of sizes",
                     "a sized function is called with one value for each of its sizes, in brackets before its "
                     "arguments, as in `sha256[2](m)`; a function without sizes is called without brackets",
                     2);
        }
        return 0;
    }
    if (func->nsizes == 0) {
        if (func->ninst == 0) {
            return 0;
        }
        *inst_id = func->inst0;
        expr->inst_id = func->inst0;
        return 1;
    }
    if (expr->nsize > 0) {
        int64_t values[MAX_SIZES];
        for (index = 0; index < expr->nsize; index++) {
            uint32_t size_index = c->args[expr->size0 + index];
            Sz value = eval_size(c, size_index);
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
                    span_copy(size_name, sizeof size_name, target->text, func->sz_name0[index], func->sz_name1[index]);
                    snprintf(message, sizeof message, "`%s` is defined for `%s` in %lld..%lld", name, size_name,
                             (long long)func->sz_lo[index], (long long)func->sz_hi[index]);
                    {
                        char label[64];
                        snprintf(label, sizeof label, "this size is %lld", (long long)value.value);
                        add_diag(c, "ORC0238", c->exprs[size_index].start, c->exprs[size_index].end, message, label,
                                 SIZE_RANGE_NOTE, 2);
                    }
                }
                return 0;
            }
            values[index] = value.value;
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
    {
        uint32_t found[2];
        int nfound = 0;
        uint32_t cursor;
        uint32_t arg_len[MAX_PARAMS];
        int have_arg[MAX_PARAMS];
        uint16_t param;
        Instance *first = &target->instances[func->inst0];
        for (param = 0; param < func->nparams && param < MAX_PARAMS; param++) {
            InstParam *shape = &target->iparams[first->param0 + param];
            int have_len = 0;
            int have_elem = 0;
            uint32_t len = 0;
            TypeKind elem = TY_NONE;
            have_arg[param] = 0;
            arg_len[param] = 0;
            if (shape->length == 0 || param >= expr->argc) {
                continue;
            }
            array_parts(c, c->args[expr->arg0 + param], caller_func, locals, &have_len, &len, &have_elem, &elem);
            if (have_len) {
                have_arg[param] = 1;
                arg_len[param] = len;
            }
        }
        for (cursor = 0; cursor < func->ninst; cursor++) {
            Instance *inst = &target->instances[func->inst0 + cursor];
            int fits = 1;
            for (param = 0; param < func->nparams && param < MAX_PARAMS && fits; param++) {
                if (!have_arg[param]) {
                    continue;
                }
                if (target->iparams[inst->param0 + param].length != arg_len[param]) {
                    fits = 0;
                }
            }
            if (!fits) {
                continue;
            }
            if (nfound < 2) {
                found[nfound] = func->inst0 + cursor;
            }
            nfound++;
            if (nfound > 1) {
                break;
            }
        }
        if (nfound == 1) {
            *inst_id = found[0];
            expr->inst_id = found[0];
            return 1;
        }
        if (!report) {
            return 0;
        }
        if (nfound >= 2) {
            char message[512];
            char first_name[160];
            char second_name[160];
            Instance *a = &target->instances[found[0]];
            Instance *b = &target->instances[found[1]];
            char a_vals[64];
            char b_vals[64];
            size_t used = 0;
            uint8_t slot;
            a_vals[0] = '\0';
            b_vals[0] = '\0';
            for (slot = 0; slot < func->nsizes; slot++) {
                int wrote = snprintf(a_vals + used, sizeof a_vals - used, slot == 0 ? "%lld" : ", %lld",
                                     (long long)a->sz[slot]);
                if (wrote > 0 && (size_t)wrote < sizeof a_vals - used) {
                    used += (size_t)wrote;
                }
            }
            used = 0;
            for (slot = 0; slot < func->nsizes; slot++) {
                int wrote = snprintf(b_vals + used, sizeof b_vals - used, slot == 0 ? "%lld" : ", %lld",
                                     (long long)b->sz[slot]);
                if (wrote > 0 && (size_t)wrote < sizeof b_vals - used) {
                    used += (size_t)wrote;
                }
            }
            snprintf(first_name, sizeof first_name, "%s[%s]", name, a_vals);
            snprintf(second_name, sizeof second_name, "%s[%s]", name, b_vals);
            snprintf(message, sizeof message, "this call fits more than one instance of `%s`, among them `%s` and `%s`",
                     name, first_name, second_name);
            add_diag(c, "ORC0239", expr->start, expr->end, message, "write the sizes in brackets",
                     "a call that writes no sizes calls the one instance of its function whose array parameters have "
                     "the lengths of its arguments; any other call writes its sizes in brackets, as in `absorb[2](p)`",
                     2);
        } else {
            char message[384];
            char label[128];
            char domain[192];
            size_t used = 0;
            uint8_t slot;
            domain[0] = '\0';
            snprintf(message, sizeof message, "no instance of `%s` takes arguments of these lengths", name);
            if (func->nparams == 1 && have_arg[0]) {
                snprintf(label, sizeof label, "an array of length %u is given", arg_len[0]);
            } else {
                copy_text(label, sizeof label, "no instance fits these arguments");
            }
            for (slot = 0; slot < func->nsizes; slot++) {
                char piece[64];
                char size_name[32];
                int wrote;
                span_copy(size_name, sizeof size_name, target->text, func->sz_name0[slot], func->sz_name1[slot]);
                snprintf(piece, sizeof piece, "`%s` in %lld..%lld", size_name, (long long)func->sz_lo[slot],
                         (long long)func->sz_hi[slot]);
                wrote = snprintf(domain + used, sizeof domain - used, slot == 0 ? "%s" : ", %s", piece);
                if (wrote > 0 && (size_t)wrote < sizeof domain - used) {
                    used += (size_t)wrote;
                }
            }
            {
                char note[320];
                snprintf(note, sizeof note, "`%s` is defined for %s", name, domain);
                add_diag(c, "ORC0238", expr->start, expr->end, message, label, note, 2);
                if (c->ndiags > 0) {
                    Diag *diag = &c->diags[c->ndiags - 1];
                    copy_text(diag->note2, sizeof diag->note2,
                              "a call that writes no sizes calls the one instance of its function whose array "
                              "parameters have the lengths of its arguments; any other call writes its sizes in "
                              "brackets, as in `absorb[2](p)`");
                    diag->has_note2 = 1;
                }
            }
        }
        return 0;
    }
}

static int operand_passes_branch(const Compiler *c, uint32_t index) {
    const Expr *expr;
    uint16_t arm;
    if (index >= c->nexprs) {
        return 0;
    }
    expr = &c->exprs[index];
    if (expr->kind == EX_GROUP) {
        return operand_passes_branch(c, expr->left);
    }
    if (expr->kind != EX_COND) {
        return 0;
    }
    for (arm = 0; arm < expr->argc; arm++) {
        if (c->cond_arms[expr->arg0 + arm].nbinds > 0) {
            return 1;
        }
    }
    return expr->else_nbinds > 0;
}

/* Visit one expression at `expected` with array length `expected_len`.
   A scalar uses length 0. Returns 0 only to stop the walk. Loops, fills,
   updates, indices, slices, concatenation, byte strings, comparisons,
   division, conditionals, and conversions are checked here too. A
   chained index of a scalar is ORC0224 once. */
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
            char message[384];
            char expected_text[96];
            spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
            snprintf(message, sizeof message, "an integer literal cannot have type `%s`", expected_text);
            add_expected(c, expr->start, expr->end, message, expected_text,
                         "an array value is written `[e0, e1, ...]`, one element per index");
            return 1;
        }
        check_literal(c, expr, expected, c->expect_mod);
        return 1;
    case EX_ARRAY:
        if (expected_len == 0) {
            char message[384];
            char expected_text[96];
            spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
            snprintf(message, sizeof message, "an array literal cannot have type `%s`", expected_text);
            add_expected(c, expr->start, expr->end, message, expected_text,
                         "an array literal is written where an array type `T^n` is required");
            return 1;
        }
        if (expr->argc != expected_len) {
            char message[384];
            char expected_text[96];
            char label[96];
            spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
            snprintf(message, sizeof message, "this array has %u elements, but `%s` has %u", expr->argc, expected_text,
                     expected_len);
            snprintf(label, sizeof label, "expected %u %s", expected_len, expected_len == 1 ? "element" : "elements");
            add_diag(c, "ORC0222", expr->start, expr->end, message, label,
                     "an array literal lists every element of its type exactly once", 2);
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
                char message[384];
                char label[128];
                char type_text[64];
                char value_text[96];
                write_type(type_text, sizeof type_text, base_kind, base_len);
                if (!big_format(&magnitude, value_text, sizeof value_text)) {
                    copy_text(value_text, sizeof value_text, "?");
                }
                snprintf(message, sizeof message, "index `%s` is out of range for `%s`", value_text, type_text);
                snprintf(label, sizeof label, "indices run from 0 through %u", base_len == 0 ? 0 : base_len - 1u);
                add_diag(c, "ORC0223", expr->lit_start, expr->lit_end, message, label,
                         "a literal index must be less than the array's length", 2);
            }
        }
        {
            uint16_t element_mod = c->leaf_mod;
            if (base_kind != expected || expected_len != 0 ||
                (base_kind == TY_MOD && expected == TY_MOD && element_mod != c->expect_mod)) {
                char message[384];
                char expected_text[96];
                char found_text[96];
                spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
                spell_type(c, found_text, sizeof found_text, base_kind, 0, element_mod, 0, 0);
                snprintf(message, sizeof message, "this element has type `%s`, but `%s` is required here", found_text,
                         expected_text);
                add_expected(c, expr->start, expr->end, message, expected_text, IMPLICIT_NOTE);
            }
            return check_at(c, expr->left, base_kind, base_len, element_mod, func_index, locals_in_scope);
        }
    }
    case EX_TUPLE: {
        uint32_t tup0 = c->expect_tup0;
        uint16_t tup_n = c->expect_tup_n;
        uint16_t element;
        if (expected != TY_TUPLE || tup_n == 0) {
            char message[384];
            char expected_text[96];
            spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
            snprintf(message, sizeof message, "a tuple cannot have type `%s`", expected_text);
            add_expected(c, expr->start, expr->end, message, expected_text,
                         "a tuple is written where a tuple type `(T, U, ...)` is required");
            return 1;
        }
        if (expr->argc != tup_n) {
            char message[384];
            char expected_text[96];
            char label[96];
            spell_expected(c, expected_text, sizeof expected_text, TY_TUPLE, 0);
            snprintf(message, sizeof message, "this tuple has %u elements, but `%s` has %u", expr->argc, expected_text,
                     tup_n);
            snprintf(label, sizeof label, "expected %u elements", tup_n);
            add_diag(c, "ORC0214", expr->start, expr->end, message, label,
                     "a tuple lists every element of its type exactly once, in order", 2);
            return 1;
        }
        for (element = 0; element < expr->argc; element++) {
            TupleElem item = c->telems[tup0 + element];
            if (!check_at(c, c->args[expr->arg0 + element], item.kind, item.length, item.mod_index, func_index,
                          locals_in_scope)) {
                return 0;
            }
        }
        return 1;
    }
    case EX_PROJECT: {
        TypeKind base_kind = TY_NONE;
        uint32_t base_len = 0;
        int silent = 0;
        int state = base_type(c, expr->left, func_index, locals_in_scope, &base_kind, &base_len, &silent);
        uint32_t tup0 = c->leaf_tup0;
        uint16_t tup_n = c->leaf_tup_n;
        const Compiler *owner = c->leaf_owner == NULL ? c : c->leaf_owner;
        uint16_t base_mod = c->leaf_mod;
        if (state != 1) {
            if (silent) {
                return 1;
            }
            return check_expr(c, expr->left, TY_INT, 0, func_index, locals_in_scope);
        }
        if (base_kind != TY_TUPLE) {
            char message[384];
            char found[96];
            char label[128];
            const Expr *base = &c->exprs[expr->left];
            spell_type(c, found, sizeof found, base_kind, base_len, base_mod, tup0, tup_n);
            snprintf(message, sizeof message,
                     "only a tuple has elements selected by position, but this has type `%s`", found);
            snprintf(label, sizeof label, "`%s` is not a tuple", found);
            add_diag(c, "ORC0234", base->start, base->end, message, label,
                     base_len != 0 ? "an array's element is selected by an index, such as `x[0]`"
                                   : "`.k` selects element k of a value of a tuple type `(T, U, ...)`",
                     2);
            return check_at(c, expr->left, base_kind, base_len, base_mod, func_index, locals_in_scope);
        }
        if (expr->proj_pos >= tup_n || owner->telems == NULL) {
            char message[384];
            char found[96];
            char label[128];
            char written[32];
            spell_type(owner, found, sizeof found, TY_TUPLE, 0, 0, tup0, tup_n);
            span_copy(written, sizeof written, c->text, expr->lit_start, expr->lit_end);
            snprintf(message, sizeof message, "`%s` has no element %s", found, written);
            snprintf(label, sizeof label, "its elements are numbered 0 through %u", tup_n == 0 ? 0 : tup_n - 1u);
            add_diag(c, "ORC0223", expr->lit_start, expr->lit_end, message, label,
                     "a tuple's elements are counted from zero", 2);
        } else {
            TupleElem item = owner->telems[tup0 + expr->proj_pos];
            uint16_t mod = item.mod_index;
            if (owner != c && item.kind == TY_MOD && !adopt_modulus(c, owner, item.mod_index, &mod)) {
                return 0;
            }
            if (item.kind != expected || item.length != expected_len ||
                (item.kind == TY_MOD && expected == TY_MOD && mod != c->expect_mod)) {
                char message[384];
                char expected_text[96];
                char found_text[96];
                spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
                spell_type(owner, found_text, sizeof found_text, item.kind, item.length, mod, 0, 0);
                snprintf(message, sizeof message, "this element has type `%s`, but `%s` is required here", found_text,
                         expected_text);
                add_expected(c, expr->start, expr->end, message, expected_text, IMPLICIT_NOTE);
            }
        }
        {
            uint32_t shape = tup0;
            if (tup_n > 0 && !adopt_tuple_shape(c, owner, tup0, tup_n, &shape)) {
                return 0;
            }
            c->expect_tup0 = shape;
            c->expect_tup_n = tup_n;
            return check_at(c, expr->left, TY_TUPLE, 0, 0, func_index, locals_in_scope);
        }
    }
    case EX_NAME: {
        NameRes res;
        uint16_t slot = 0;
        uint32_t abs_index = 0;
        TypeKind type = TY_NONE;
        uint32_t length = 0;
        int type_ok = 0;
        resolve_name(c, func_index, locals_in_scope, expr->name_start, expr->name_end, &res, &slot, &type, &length,
                     &type_ok, &abs_index);
        expr->name_res = res;
        expr->name_index = slot;
        expr->name_abs = abs_index;
        expr->name_ty = type;
        expr->name_len = length;
        if (res == NAME_BAD) {
            return 1;
        }
        if (res == NAME_SIZE) {
            if (expected != TY_INT || expected_len != 0) {
                report_name_mismatch(c, expr->start, expr->end, TY_INT, 0, 0, 0, 0, expected, expected_len);
            }
            return 1;
        }
        if (res == NAME_BLOCK) {
            uint16_t found_mod = type == TY_MOD ? c->block_locals[abs_index].mod_index : 0;
            if (c->block_locals[abs_index].pat_len > 0 || c->block_locals[abs_index].pat_i > 0) {
                expr->is_proj = 1;
            }
            uint32_t nt0 = c->block_locals[abs_index].tup0;
            uint16_t ntn = c->block_locals[abs_index].tup_n;
            int shape = type == TY_TUPLE && expected == TY_TUPLE && c->expect_tup_n > 0 && ntn > 0 &&
                        !same_tuple(c, c->expect_tup0, c->expect_tup_n, c, nt0, ntn);
            if (type != expected || length != expected_len || shape ||
                (type == TY_MOD && expected == TY_MOD && found_mod != c->expect_mod)) {
                report_name_mismatch(c, expr->start, expr->end, type, length, found_mod, nt0, ntn, expected,
                                     expected_len);
            }
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
                    report_name_mismatch(c, expr->start, expr->end, TY_BOOL, 0, 0, 0, 0, expected, expected_len);
                }
                return 1;
            }
            report_unknown_name(c, expr, func_index, res);
            return 1;
        }
        {
            uint16_t found_mod = 0;
            uint32_t nt0 = 0;
            uint16_t ntn = 0;
            int shape;
            if (res == NAME_LOCAL) {
                const Local *bound = &c->locals[c->funcs[func_index].local0 + slot];
                if (bound->pat_len > 0 || bound->pat_i > 0) {
                    expr->is_proj = 1;
                }
            }
            if (type == TY_MOD) {
                found_mod = res == NAME_PARAM ? c->params[c->funcs[func_index].param0 + slot].mod_index
                                              : c->locals[c->funcs[func_index].local0 + slot].mod_index;
            }
            if (type == TY_TUPLE) {
                if (res == NAME_PARAM) {
                    nt0 = c->params[c->funcs[func_index].param0 + slot].tup0;
                    ntn = c->params[c->funcs[func_index].param0 + slot].tup_n;
                } else {
                    nt0 = c->locals[c->funcs[func_index].local0 + slot].tup0;
                    ntn = c->locals[c->funcs[func_index].local0 + slot].tup_n;
                }
            }
            shape = type == TY_TUPLE && expected == TY_TUPLE && c->expect_tup_n > 0 && ntn > 0 &&
                    !same_tuple(c, c->expect_tup0, c->expect_tup_n, c, nt0, ntn);
            if (type != expected || length != expected_len || shape ||
                (type == TY_MOD && expected == TY_MOD && found_mod != c->expect_mod)) {
                report_name_mismatch(c, expr->start, expr->end, type, length, found_mod, nt0, ntn, expected,
                                     expected_len);
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
            char message[384];
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
            char message[384];
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
                uint32_t declared;
                snprintf(message, sizeof message, "`spec` function `%s` has no typed body and cannot be called", ident);
                add_diag(c, "ORC0212", expr->name_start, expr->name_end, message, "no value to call", note_text, 2);
                for (declared = 0; declared < callee.mod->nfuncs; declared++) {
                    const Func *empty = &callee.mod->funcs[declared];
                    if (!empty->is_impl && !empty->typed &&
                        same_span(callee.mod, empty->name_start, empty->name_end, expr->name_start, expr->name_end)) {
                        diag_add_secondary(c, empty->name_start, empty->name_end, "declared without a result type here");
                        break;
                    }
                }
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
        }
        if (!signature_is_usable(target, callee.func)) {
            return 1;
        }
        if (expr->argc != target->funcs[callee.func].nparams) {
            char message[384];
            snprintf(message, sizeof message, "`%s` takes %u argument%s but %u %s supplied", ident,
                     target->funcs[callee.func].nparams, target->funcs[callee.func].nparams == 1 ? "" : "s", expr->argc,
                     expr->argc == 1 ? "was" : "were");
            add_diag(c, "ORC0213", expr->start, expr->end, message, "wrong number of arguments",
                     "every parameter receives exactly one argument", 2);
            return 1;
        }
        {
            Func *callee_func = &target->funcs[callee.func];
            Instance *inst = NULL;
            uint32_t inst_id = UINT32_MAX;
            TypeKind found_result;
            uint32_t found_len;
            uint16_t found_mod;
            uint32_t found_tup0;
            uint16_t found_tup_n;
            int mod_differs = 0;
            if (callee_func->ninst > 0) {
                if (!lookup_call(c, expr, target, callee.func, 1, func_index, locals_in_scope, &inst_id) ||
                    inst_id >= target->ninstances) {
                    return 1;
                }
                inst = &target->instances[inst_id];
                if (!inst->result_ok || !inst->signature_ok) {
                    return 1;
                }
                found_result = inst->result;
                found_len = inst->result_len;
                found_mod = inst->result_mod;
                found_tup0 = inst->tup0;
                found_tup_n = inst->tup_n;
            } else {
                found_result = callee_func->result;
                found_len = callee_func->result_len;
                found_mod = callee_func->result_mod;
                found_tup0 = callee_func->tup0;
                found_tup_n = callee_func->tup_n;
            }
            if (!callee.qualified && !record_edge(c, func_index, callee.func, inst_id, expr->start, expr->end)) {
                return 0;
            }
            if (found_result == TY_MOD && expected == TY_MOD) {
                uint16_t local = 0;
                /* result_mod is an index in the callee. expect_mod is an index
                   in the caller. Compare the modulus values. */
                if (!adopt_modulus(c, target, found_mod, &local)) {
                    return 0;
                }
                mod_differs = local != c->expect_mod;
            }
            if (found_result == TY_TUPLE && expected == TY_TUPLE && c->expect_tup_n > 0 && found_tup_n > 0) {
                mod_differs = !same_tuple(c, c->expect_tup0, c->expect_tup_n, target, found_tup0, found_tup_n);
            }
            if (found_result != expected || found_len != expected_len || mod_differs) {
                char message[384];
                char expected_text[96];
                char found_text[96];
                spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
                spell_type(target, found_text, sizeof found_text, found_result, found_len, found_mod, found_tup0,
                           found_tup_n);
                snprintf(message, sizeof message, "`%s` returns `%s`, but `%s` is required here", ident, found_text,
                         expected_text);
                add_expected(c, expr->start, expr->end, message, expected_text, IMPLICIT_NOTE);
            }
            for (uint16_t arg = 0; arg < expr->argc; arg++) {
                TypeKind arg_type;
                uint32_t arg_len;
                uint16_t arg_mod;
                int arg_ok;
                uint32_t arg_tup0;
                uint16_t arg_tup_n;
                if (inst != NULL) {
                    InstParam *shape = &target->iparams[inst->param0 + arg];
                    arg_type = shape->type;
                    arg_len = shape->length;
                    arg_mod = shape->mod_index;
                    arg_ok = shape->type_ok;
                    arg_tup0 = shape->tup0;
                    arg_tup_n = shape->tup_n;
                } else {
                    Param *param = &target->params[callee_func->param0 + arg];
                    arg_type = param->type;
                    arg_len = param->length;
                    arg_mod = param->mod_index;
                    arg_ok = param->type_ok;
                    arg_tup0 = param->tup0;
                    arg_tup_n = param->tup_n;
                }
                if (!arg_ok) {
                    continue;
                }
                if (arg_type == TY_MOD && !adopt_modulus(c, target, arg_mod, &arg_mod)) {
                    return 0;
                }
                if (arg_type == TY_TUPLE) {
                    uint32_t shape = arg_tup0;
                    if (!adopt_tuple_shape(c, target, arg_tup0, arg_tup_n, &shape)) {
                        return 0;
                    }
                    c->expect_tup0 = shape;
                    c->expect_tup_n = arg_tup_n;
                }
                if (!check_at(c, c->args[expr->arg0 + arg], arg_type, arg_len, arg_mod, func_index, locals_in_scope)) {
                    return 0;
                }
            }
        }
        return 1;
    }
    case EX_UNARY: {
        const char *uname = expr->op == TK_MINUS ? "-" : expr->op == TK_TILDE ? "~" : "!";
        if (expected_len != 0 || expected == TY_TUPLE) {
            report_undefined_op(c, expr->op_start, expr->op_end, uname, 1, expected, expected_len, c->expect_mod,
                                c->expect_tup0, c->expect_tup_n,
                                expected == TY_TUPLE ? TUPLE_OPERATOR_NOTE : ARRAY_OPERATOR_NOTE);
            return 1;
        }
        if (expr->op == TK_BANG) {
            /* Defined only for `Bool`. Rust reports ORC0215 and does not
               typecheck the operand, so a rejected `!` does not add the
               operand's own codes. */
            if (expected != TY_BOOL) {
                const char *note = type_width(expected) > 0
                                       ? "`!` negates a `Bool`; `~` is the bitwise complement of a word"
                                       : "`!` negates a `Bool`; `-` negates an `Int` or a residue";
                report_undefined_op(c, expr->op_start, expr->op_end, "!", 1, expected, 0, c->expect_mod, 0, 0, note);
                return 1;
            }
            return check_expr(c, expr->left, TY_BOOL, 0, func_index, locals_in_scope);
        }
        if (expr->op == TK_MINUS && expected == TY_MOD) {
            return check_expr(c, expr->left, TY_MOD, 0, func_index, locals_in_scope);
        }
        if (expr->op == TK_TILDE && expected == TY_MOD) {
            report_undefined_op(c, expr->op_start, expr->op_end, "~", 1, TY_MOD, 0, c->expect_mod, 0, 0,
                                "bitwise operators apply only to `Word[n]` values");
            return 1;
        }
        if (expr->op == TK_MINUS && expected != TY_INT) {
            if (expected == TY_BOOL) {
                report_undefined_op(c, expr->op_start, expr->op_end, "-", 1, TY_BOOL, 0, 0, 0, 0, BOOL_OPERATOR_NOTE);
            } else {
                char note[80];
                snprintf(note, sizeof note, "write `0 - x` for negation modulo 2^%d", type_width(expected));
                report_undefined_op(c, expr->op_start, expr->op_end, "-", 1, expected, 0, 0, 0, 0, note);
            }
            return 1;
        }
        if (expr->op == TK_TILDE && (expected == TY_INT || expected == TY_BOOL)) {
            report_undefined_op(c, expr->op_start, expr->op_end, "~", 1, expected, 0, 0, 0, 0,
                                expected == TY_BOOL ? BOOL_OPERATOR_NOTE
                                                    : "bitwise operators apply only to `Word[n]` values");
            return 1;
        }
        return check_expr(c, expr->left, expected, 0, func_index, locals_in_scope);
    }
    case EX_BINARY:
        if (expr->op == TK_PLUSPLUS) {
            return check_concat(c, index, expected, expected_len, func_index, locals_in_scope);
        }
        if (is_compare_op(expr->op)) {
            return check_compare(c, index, expected, expected_len, func_index, locals_in_scope);
        }
        if (expr->op == TK_AMPAMP || expr->op == TK_PIPEPIPE) {
            /* Defined only for `Bool`. Rust reports ORC0215 and does not
               typecheck either operand. An array expected type keeps its
               length in the message. */
            if (expected != TY_BOOL || expected_len != 0) {
                report_undefined_op(c, expr->op_start, expr->op_end, op_spelling(expr->op), 0, expected, expected_len,
                                    c->expect_mod, c->expect_tup0, c->expect_tup_n,
                                    "`&&` and `||` apply to `Bool` values; `&` and `|` are the bitwise operators on words");
                return 1;
            }
            if (!check_expr(c, expr->left, TY_BOOL, 0, func_index, locals_in_scope)) {
                return 0;
            }
            return check_expr(c, expr->right, TY_BOOL, 0, func_index, locals_in_scope);
        }
        if (expected_len != 0 || expected == TY_TUPLE) {
            report_undefined_op(c, expr->op_start, expr->op_end, op_spelling(expr->op), 0, expected, expected_len,
                                c->expect_mod, c->expect_tup0, c->expect_tup_n,
                                expected == TY_TUPLE ? TUPLE_OPERATOR_NOTE : ARRAY_OPERATOR_NOTE);
            return 1;
        }
        if (expected == TY_BOOL) {
            report_undefined_op(c, expr->op_start, expr->op_end, op_spelling(expr->op), 0, TY_BOOL, 0, 0, 0, 0,
                                BOOL_OPERATOR_NOTE);
            return 1;
        }
        if ((expr->op == TK_AMP || expr->op == TK_PIPE || expr->op == TK_CARET) && expected == TY_INT) {
            report_undefined_op(c, expr->op_start, expr->op_end, op_spelling(expr->op), 0, TY_INT, 0, 0, 0, 0,
                                "bitwise operators apply only to `Word[n]` values");
            return 1;
        }
        if (expected == TY_MOD) {
            if (expr->op != TK_PLUS && expr->op != TK_MINUS && expr->op != TK_STAR && expr->op != TK_SLASH) {
                const char *note = expr->op == TK_PERCENT
                                       ? "a residue is already reduced; `%` applies to `Int` and word values, such as "
                                         "`(x as Int) % 16`"
                                       : "bitwise operators apply only to `Word[n]` values";
                report_undefined_op(c, expr->op_start, expr->op_end, op_spelling(expr->op), 0, TY_MOD, 0, c->expect_mod,
                                    0, 0, note);
                return 1;
            }
            if (!check_expr(c, expr->left, expected, 0, func_index, locals_in_scope)) {
                return 0;
            }
            return check_expr(c, expr->right, expected, 0, func_index, locals_in_scope);
        }
        if (!is_number_type(expected) && expected != TY_NONE) {
            report_undefined_op(c, expr->op_start, expr->op_end, op_spelling(expr->op), 0, expected, expected_len,
                                c->expect_mod, c->expect_tup0, c->expect_tup_n, NULL);
            return 1;
        }
        if (!check_expr(c, expr->left, expected, 0, func_index, locals_in_scope)) {
            return 0;
        }
        return check_expr(c, expr->right, expected, 0, func_index, locals_in_scope);
    case EX_SHIFT:
        if (expected_len != 0 || expected == TY_TUPLE || expected == TY_BOOL || expected == TY_INT ||
            expected == TY_MOD || type_width(expected) == 0) {
            const char *note = expected == TY_BOOL ? BOOL_OPERATOR_NOTE
                               : expected == TY_TUPLE ? TUPLE_OPERATOR_NOTE
                               : expected_len != 0    ? ARRAY_OPERATOR_NOTE
                                                      : "shifts and rotations apply only to `Word[n]` values";
            report_undefined_op(c, expr->op_start, expr->op_end, shift_name(expr->op), 0, expected, expected_len,
                                c->expect_mod, c->expect_tup0, c->expect_tup_n, note);
            return 1;
        }
        if (!check_expr(c, expr->left, expected, 0, func_index, locals_in_scope)) {
            return 0;
        }
        {
            const Expr *amount = &c->exprs[expr->right];
            Big value = big_zero();
            int width = type_width(expected);
            int highest = width > 0 ? width - 1 : 0;
            char message[160];
            char label[96];
            char type_text[64];
            spell_type(c, type_text, sizeof type_text, expected, 0, 0, 0, 0);
            snprintf(message, sizeof message, "`%s` on `%s` needs an amount from 0 through %d", shift_name(expr->op),
                     type_text, highest);
            snprintf(label, sizeof label, "a literal amount is from 0 through %d", highest);
            if (amount->kind != EX_LIT || amount->negative) {
                add_diag(c, "ORC0216", amount->start, amount->end, message, label, SHIFT_AMOUNT_NOTE, 2);
                return 1;
            }
            if (!decode_literal(c, amount, &value)) {
                add_diag(c, "ORC0205", amount->lit_start, amount->lit_end,
                         "integer magnitude exceeds the 16384-significant-bit limit",
                         "exact integer is too large for this semantic fragment",
                         "the literal is rejected rather than truncated or approximated", 2);
                return 1;
            }
            if (big_bits(&value) > 31 || (value.nlimbs > 0 && value.limbs[0] >= (uint32_t)width) || width == 0) {
                add_diag(c, "ORC0216", amount->start, amount->end, message, label, SHIFT_AMOUNT_NOTE, 2);
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
           `expected`. reject_type reports ORC0204 for a bad word width and
           ORC0203 for any other unsupported type. The operand is still
           checked: both the target and the operand's own diagnostic are kept. */
        if (!expr->conv_ok) {
            int already = expr->conv_site < c->nsites && c->sites[expr->conv_site].reported;
            if (!already) {
                reject_type(c, expr->conv_ty, 0, expr->name_start, expr->name_end);
            }
        } else if (expr->conv_len != 0) {
            char message[384];
            char target[96];
            char note_buf[384];
            const char *note = "convert each element, such as `x[0] as Int`";
            spell_type(c, target, sizeof target, expr->conv_ty, expr->conv_len, expr->conv_mod, 0, 0);
            snprintf(message, sizeof message, "`as` does not convert to the array type `%s`", target);
            if (expr->conv_ty == TY_W8 || expr->conv_ty == TY_W16 || expr->conv_ty == TY_W32 ||
                expr->conv_ty == TY_W64) {
                snprintf(note_buf, sizeof note_buf,
                         "name a byte order to write words as `%s`, as in `as big %s`, or build the array from its "
                         "elements",
                         target, target);
                note = note_buf;
            }
            add_diag(c, "ORC0215", expr->name_start, expr->name_end, message,
                     "`as` gives one `Int`, word, or residue value", note, 2);
            return 1;
        } else if (expr->conv_ty == TY_TUPLE) {
            add_diag(c, "ORC0215", expr->name_start, expr->name_end, "`as` does not convert to a tuple type",
                     "`as` gives one `Int`, word, or residue value", "select one element, such as `p.0 as Int`", 2);
            return 1;
        } else if (expr->conv_ty != expected || expected_len != 0 ||
                   (expr->conv_ty == TY_MOD && expected == TY_MOD && expr->conv_mod != c->expect_mod)) {
            char message[384];
            char expected_text[96];
            char found_text[96];
            spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
            spell_type(c, found_text, sizeof found_text, expr->conv_ty, expr->conv_len, expr->conv_mod, 0, 0);
            snprintf(message, sizeof message, "this conversion gives `%s`, but `%s` is required here", found_text,
                     expected_text);
            add_expected(c, expr->name_start, expr->name_end, message, expected_text,
                         "`as` gives exactly the type written after it");
        }
        state = find_leaf(c, expr->left, func_index, locals_in_scope, &leaf_type, &leaf_len, &leaf, &silent);
        if (state == 2 || (state > 0 && (leaf_len != 0 || leaf_type == TY_TUPLE))) {
            char message[384];
            char found[96];
            int tuple = leaf_type == TY_TUPLE || c->exprs[expr->left].kind == EX_TUPLE;
            int words = leaf_len != 0 && type_width(leaf_type) > 0;
            const char *note = tuple ? "convert each element, such as `p.0 as Int`"
                               : words ? "name a byte order to read the words as one number or as words of another "
                                        "width, as in `x as big Int`, or convert each element, such as `x[0] as Int`"
                                       : "convert each element, such as `x[0] as Int`";
            if (state > 0 && leaf_type != TY_NONE) {
                spell_type(c, found, sizeof found, leaf_type, leaf_len, c->leaf_mod,
                           leaf_type == TY_TUPLE ? c->leaf_tup0 : 0, leaf_type == TY_TUPLE ? c->leaf_tup_n : 0);
                snprintf(message, sizeof message, "`as` is not defined for `%s`", found);
            } else {
                snprintf(message, sizeof message, "`as` is not defined for %s", tuple ? "a tuple" : "an array");
            }
            add_diag(c, "ORC0215", expr->op_start, expr->op_end, message,
                     "`as` converts one `Int`, word, or residue value", note, 2);
            return 1;
        }
        if (state == 0) {
            int branch = operand_passes_branch(c, expr->left);
            add_diag(c, "ORC0220", c->exprs[expr->left].start, c->exprs[expr->left].end,
                     "the operand of `as` has no type of its own",
                     branch ? "a branch's own bindings are not in scope outside it"
                            : "a literal takes its type from where it is used",
                     branch ? "bind the conditional's value with a typed `let` first, or convert within each branch"
                            : "write the literal where its type is required, or give it a type with a `let` binding",
                     2);
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
        uint32_t else_bind0 = expr->else_bind0;
        uint16_t else_nbinds = expr->else_nbinds;
        uint16_t arm;
        uint16_t expect_mod = c->expect_mod;
        for (arm = 0; arm < arms; arm++) {
            const CondArm *item = &c->cond_arms[arg0 + arm];
            uint32_t condition = item->cond;
            uint32_t value = item->value;
            uint32_t bind0 = item->bind0;
            uint16_t nbinds = item->nbinds;
            if (!check_expr(c, condition, TY_BOOL, 0, func_index, locals_in_scope) ||
                !check_block(c, bind0, nbinds, value, expected, expected_len, expect_mod, func_index, locals_in_scope)) {
                return 0;
            }
        }
        return check_block(c, else_bind0, else_nbinds, otherwise, expected, expected_len, expect_mod, func_index,
                           locals_in_scope);
    }
    case EX_LOOP:
        return check_loop(c, index, expected, expected_len, func_index, locals_in_scope);
    case EX_LOOP_INDEX:
        if (expected != TY_INT || expected_len != 0) {
            report_name_mismatch(c, expr->start, expr->end, TY_INT, 0, 0, 0, 0, expected, expected_len);
        }
        return 1;
    case EX_ACCUM: {
        const LoopDesc *loop = &c->loops[expr->arg0];
        if (!loop->acc_ok) {
            return 1;
        }
        if (expr->is_proj) {
            const TupleElem *elem;
            if (expr->name_index >= loop->tup_n || c->telems == NULL) {
                return 1;
            }
            elem = &c->telems[loop->tup0 + expr->name_index];
            if (elem->kind != expected || elem->length != expected_len ||
                (elem->kind == TY_MOD && expected == TY_MOD && elem->mod_index != c->expect_mod)) {
                report_name_mismatch(c, expr->start, expr->end, elem->kind, elem->length, elem->mod_index, 0, 0,
                                     expected, expected_len);
            }
            return 1;
        }
        if (loop->acc_type != expected || loop->acc_len != expected_len ||
            (loop->acc_type == TY_TUPLE && expected == TY_TUPLE && c->expect_tup_n > 0 && loop->tup_n > 0 &&
             !same_tuple(c, c->expect_tup0, c->expect_tup_n, c, loop->tup0, loop->tup_n)) ||
        (loop->acc_type == TY_MOD && expected == TY_MOD && loop->acc_mod != c->expect_mod)) {
            report_name_mismatch(c, expr->start, expr->end, loop->acc_type, loop->acc_len, loop->acc_mod, loop->tup0,
                                 loop->tup_n, expected, expected_len);
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
                char message[384];
                char expected_text[96];
                char found_text[96];
                spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
                spell_type(c, found_text, sizeof found_text, base_kind, 0, element_mod, 0, 0);
                snprintf(message, sizeof message, "this element has type `%s`, but `%s` is required here", found_text,
                         expected_text);
                add_expected(c, expr->start, expr->end, message, expected_text, IMPLICIT_NOTE);
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
            char message[384];
            char found[96];
            char label[128];
            const Compiler *owner = c->leaf_owner == NULL ? c : c->leaf_owner;
            spell_type(owner, found, sizeof found, leaf_type, 0, c->leaf_mod, c->leaf_tup0, c->leaf_tup_n);
            snprintf(message, sizeof message, "only an array can be updated, but this has type `%s`", found);
            if (leaf_type == TY_TUPLE) {
                snprintf(label, sizeof label, "`%s` is a tuple, not an array", found);
            } else {
                snprintf(label, sizeof label, "`%s` has no elements", found);
            }
            add_diag(c, "ORC0224", c->exprs[expr->left].start, c->exprs[expr->left].end, message, label,
                     leaf_type == TY_TUPLE ? "a tuple with one element replaced is written anew, such as `(v, p.1)`"
                                           : "`x with [i] = v` is the array `x` with one element replaced",
                     2);
            if (!silent) {
                return check_expr(c, expr->left, leaf_type, 0, func_index, locals_in_scope);
            }
            return 1;
        }
        if (expected_len == 0) {
            char message[384];
            char expected_text[96];
            spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
            snprintf(message, sizeof message, "an update gives an array, but `%s` is required here", expected_text);
            add_expected(c, expr->start, expr->end, message, expected_text,
                         "`x with [i] = v` is the array `x` with one element replaced");
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
            char message[384];
            char expected_text[96];
            spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
            snprintf(message, sizeof message, "an array literal cannot have type `%s`", expected_text);
            add_expected(c, expr->start, expr->end, message, expected_text,
                         "an array literal is written where an array type `T^n` is required");
            return 1;
        }
        if (expr->size_expr != UINT32_MAX) {
            admitted = size_length(c, expr->size_expr, 1, &length);
            if (!admitted) {
                return check_expr(c, expr->left, expected, 0, func_index, locals_in_scope);
            }
        } else {
            admitted = canonical_array_length(c->text, expr->lit_start, expr->lit_end, &length);
        }
        if (!admitted) {
            add_diag(c, "ORC0221", expr->lit_start, expr->lit_end,
                     "an array length must be a decimal integer from 1 through 65536", "unsupported array length",
                     "write the length in decimal without leading zeros, as in `Word[32]^16`", 2);
        } else if (length != expected_len) {
            char message[384];
            char expected_text[96];
            char label[64];
            spell_expected(c, expected_text, sizeof expected_text, expected, expected_len);
            snprintf(message, sizeof message, "this array has %u %s, but `%s` has %u", length,
                     length == 1 ? "element" : "elements", expected_text, expected_len);
            snprintf(label, sizeof label, "expected %u %s", expected_len, expected_len == 1 ? "element" : "elements");
            add_diag(c, "ORC0222", expr->start, expr->end, message, label, "`[e; n]` is the array of n copies of e", 2);
        }
        return check_expr(c, expr->left, expected, 0, func_index, locals_in_scope);
    }
    case EX_BYTES:
        return check_bytes(c, index, expected, expected_len);
    case EX_SLICE:
        return check_slice(c, index, expected, expected_len, func_index, locals_in_scope);
    case EX_SLICE_UP:
        return check_slice_update(c, index, expected, expected_len, func_index, locals_in_scope);
    default:
        return 1;
    }
}

/* 1 when `text[start, end)` is exactly 8, 16, 32, or 64. */
static int admitted_word_width(const Compiler *c, uint32_t start, uint32_t end) {
    static const char *widths[] = {"8", "16", "32", "64"};
    uint32_t index;
    size_t length = (size_t)(end - start);
    for (index = 0; index < 4; index++) {
        if (strlen(widths[index]) == length && memcmp(c->text + start, widths[index], length) == 0) {
            return 1;
        }
    }
    return 0;
}

/* 1 when `site` spells `Word` with a width other than 8, 16, 32, or 64.
   `*missing` selects the bare-`Word` diagnostic. The span stops at `]`, so
   `Word[1]^5` and `Word[1]^0` underline the `1`. */
static int bad_word_width(const Compiler *c, const TypeSite *site, uint32_t *width_start, uint32_t *width_end,
                          int *missing) {
    uint32_t start = site->start;
    uint32_t end = site->end;
    *missing = 0;
    if (end < start + 4 || memcmp(c->text + start, "Word", 4) != 0) {
        return 0;
    }
    if (end < start + 5 || c->text[start + 4] != '[') {
        *missing = 1;
        *width_start = start;
        *width_end = start + 4;
        return 1;
    }
    *width_start = start + 5;
    *width_end = *width_start;
    while (*width_end < end && c->text[*width_end] != ']') {
        *width_end += 1;
    }
    return !admitted_word_width(c, *width_start, *width_end);
}

/* `resolve_site` leaves a bad width or length unreported, and a use of that
   alias is then marked reported. Diagnose the declaration itself: one error
   for the width, the length, or a size name. A use, a further alias, and a
   tuple element do not add another error. No size parameter is in scope here. */
static void report_alias_target(Compiler *c, TypeSite *site) {
    uint16_t index;
    uint32_t width_start = 0;
    uint32_t width_end = 0;
    int missing = 0;
    if (c->resource) {
        return;
    }
    if (site->is_tuple) {
        int failed = !site->ok;
        for (index = 0; index < site->elem_n; index++) {
            if (site->elem0 + index < c->nsites) {
                TypeSite *elem = &c->sites[site->elem0 + index];
                report_alias_target(c, elem);
                if (!elem->ok) {
                    failed = 1;
                }
            }
        }
        if (failed) {
            site->ok = 0;
            site->reported = 1;
        }
        return;
    }
    if (site->reported) {
        return;
    }
    if (!site->ok) {
        if (bad_word_width(c, site, &width_start, &width_end, &missing)) {
            if (missing) {
                add_diag(c, "ORC0204", width_start, width_end, "`Word` requires an exact width of 8, 16, 32, or 64",
                         "missing word width", "write the width in decimal, as in `Word[32]`", 2);
            } else {
                add_diag(c, "ORC0204", width_start, width_end, "`Word` width must be exactly 8, 16, 32, or 64",
                         "unsupported word width", "word widths do not coerce, truncate, or wrap", 2);
            }
            site->reported = 1;
            return;
        }
        if (site->length_bad) {
            reject_declared(c, site->kind, 1, site->start, site->end, site->length_start, site->length_end);
            site->reported = 1;
            return;
        }
        reject_type(c, site->kind, 0, site->start, site->end);
        site->reported = 1;
        return;
    }
    if (site->has_size_expr) {
        uint32_t length = 0;
        if (!size_length(c, site->length_expr, 1, &length)) {
            site->ok = 0;
            site->reported = 1;
            return;
        }
        site->length = length;
        /* Rank was taken from the resolved target. A length expression
           does not lower it. */
    }
}

/* Gate 0 allows 524288 bytes in one text file. The rest of this
   translation unit is included below; only this file defines
   ORANGE_COMPILE_REST, so the static helpers above stay visible. */
#define ORANGE_COMPILE_REST
#include "compile.h"
