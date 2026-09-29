"""D-006 v0.3 shared inputs and their foundation-neutral reference (research-only).

The D-006 suite compares Rocq and Lean 4 on seven cases. Both candidates
receive one frozen, foundation-neutral packet of statements, fixtures and
expected observations. This module is the executable reading of that packet:
a plain-Python reference for every shared computation (the DS-01 Core
fragment, the DS-02 constant-time language, the DS-03 canonical record
format, and the DS-04 bit-blast and LRAT rules), and the generator that
writes the packet's JSON files from it.

The reference is an oracle for expected observations only. It is not a
candidate, proves nothing, and is not trusted by either candidate's proofs;
a candidate that agrees with it has computed the same observations, which is
what DS-01, DS-03 and DS-04 compare.

``generate`` writes the shared inputs (the golden LRAT certificate is copied
from the path given, since producing it needs the pinned producers), and
``check`` rebuilds every generated file in memory and fails on any byte
difference. The standard library suffices; nothing here touches the network.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from typing import Any, Callable

ROOT = Path(__file__).resolve().parents[1]
LAB = "research/decisions/D-006/d006-v0.3/"
SHARED = LAB + "shared-inputs/"
SCHEMA = "d006-v0.3-shared-input-1"
SUITE_VERSION = "d006-v0.3"


def canonical(value: Any) -> bytes:
    """RFC 8785 canonical JSON for the integer-only subset the repository uses."""

    def text(item: Any) -> str:
        if item is None:
            return "null"
        if item is True:
            return "true"
        if item is False:
            return "false"
        if isinstance(item, int):
            if not -(2**53) + 1 <= item <= 2**53 - 1:
                raise ValueError("integer outside the interoperable range")
            return str(item)
        if isinstance(item, str):
            return json.dumps(item, ensure_ascii=False, separators=(",", ":"))
        if isinstance(item, list):
            return "[" + ",".join(text(entry) for entry in item) + "]"
        if isinstance(item, dict):
            keys = sorted(item, key=lambda key: key.encode("utf-16-be"))
            return "{" + ",".join(f"{text(key)}:{text(item[key])}" for key in keys) + "}"
        raise TypeError(f"unsupported JSON value {type(item).__name__}")

    return text(value).encode("utf-8")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


# ---------------------------------------------------------------------------
# Neutral term syntax. Every shared statement, observation and negative case
# is a term in this syntax; each candidate's adapter renders it into its own
# language from a mapping of symbol ids to its declarations.
# ---------------------------------------------------------------------------


def sym(ident: str) -> list[Any]:
    return ["sym", ident]


def var(name: str) -> list[Any]:
    return ["var", name]


def nat(value: int) -> list[Any]:
    return ["nat", str(value)]


def app(function: list[Any], *arguments: list[Any]) -> list[Any]:
    return ["app", function, *arguments]


def forall(binders: list[tuple[str, list[Any]]], body: list[Any]) -> list[Any]:
    return ["forall", [[name, kind] for name, kind in binders], body]


def exists(binders: list[tuple[str, list[Any]]], body: list[Any]) -> list[Any]:
    return ["exists", [[name, kind] for name, kind in binders], body]


def imp(left: list[Any], right: list[Any]) -> list[Any]:
    return ["imp", left, right]


def eq(left: list[Any], right: list[Any]) -> list[Any]:
    return ["eq", left, right]


def ne(left: list[Any], right: list[Any]) -> list[Any]:
    return ["ne", left, right]


def lt(left: list[Any], right: list[Any]) -> list[Any]:
    return ["lt", left, right]


def le(left: list[Any], right: list[Any]) -> list[Any]:
    return ["le", left, right]


def conj(*parts: list[Any]) -> list[Any]:
    return ["and", *parts]


def disj(*parts: list[Any]) -> list[Any]:
    return ["or", *parts]


def iff(left: list[Any], right: list[Any]) -> list[Any]:
    return ["iff", left, right]


def lnot(part: list[Any]) -> list[Any]:
    return ["not", part]


def lst(kind: list[Any], items: list[list[Any]]) -> list[Any]:
    return ["list", kind, items]


def string(value: str) -> list[Any]:
    return ["str", value]


HOLE = ["hole"]
PROP = ["sort", "Prop"]
TYPE = ["sort", "Type"]

# DS-01 Core fragment symbols.
BOOL, NAT, DECERR, STRING = sym("T-01"), sym("T-02"), sym("T-07"), sym("T-08")
TRUE, FALSE = sym("C-06"), sym("C-07")


def WORD(width: list[Any]) -> list[Any]:
    return app(sym("T-03"), width)


def SEQ(kind: list[Any], length: list[Any]) -> list[Any]:
    return app(sym("T-04"), kind, length)


def LIST(kind: list[Any]) -> list[Any]:
    return app(sym("T-05"), kind)


def RESULT(value: list[Any], error: list[Any]) -> list[Any]:
    return app(sym("T-06"), value, error)


def word(width: int, value: int) -> list[Any]:
    return app(sym("F-01"), nat(width), nat(value))


def OK(value_kind: list[Any], error_kind: list[Any], value: list[Any]) -> list[Any]:
    return app(sym("C-04"), value_kind, error_kind, value)


def ERR(value_kind: list[Any], error_kind: list[Any], error: list[Any]) -> list[Any]:
    return app(sym("C-05"), value_kind, error_kind, error)


def boolean(value: bool) -> list[Any]:
    return TRUE if value else FALSE


def arrow(*kinds: list[Any]) -> list[Any]:
    result = kinds[-1]
    for kind in reversed(kinds[:-1]):
        result = imp(kind, result)
    return result


def bytes_term(values: list[int]) -> list[Any]:
    """A List (Word 8) literal, written as lowercase hex."""

    return ["bytes", bytes(values).hex()]


W, A, E, N = var("w"), var("A"), var("E"), var("n")

CORE_SIGNATURE: list[dict[str, Any]] = [
    {"id": "T-01", "name": "Bool", "kind": "type", "type": TYPE, "meaning": "The two truth values; C-06 is true and C-07 is false."},
    {"id": "T-02", "name": "Nat", "kind": "type", "type": TYPE, "meaning": "Mathematical natural numbers, unbounded."},
    {"id": "T-03", "name": "Word", "kind": "type", "type": arrow(NAT, TYPE), "meaning": "Word w: exact bit vectors of width w, one value for each natural below 2^w."},
    {"id": "T-04", "name": "Seq", "kind": "type", "type": arrow(TYPE, NAT, TYPE), "meaning": "Seq A n: sequences of exactly n elements of A (length-indexed)."},
    {"id": "T-05", "name": "List", "kind": "type", "type": arrow(TYPE, TYPE), "meaning": "Finite lists; list literals render through C-08 and C-09."},
    {"id": "T-06", "name": "Result", "kind": "type", "type": arrow(TYPE, TYPE, TYPE), "meaning": "Result A E: success with an A (C-04) or typed failure with an E (C-05)."},
    {"id": "T-07", "name": "DecodeError", "kind": "type", "type": TYPE, "meaning": "The F-14 decoder's failures: exactly C-01, C-02 and C-03."},
    {"id": "T-08", "name": "String", "kind": "type", "type": TYPE, "meaning": "Finite text; every shared string literal is ASCII."},
    {"id": "C-01", "name": "too_many", "kind": "constructor", "type": DECERR, "meaning": "The count byte exceeds 16."},
    {"id": "C-02", "name": "truncated", "kind": "constructor", "type": DECERR, "meaning": "The input ends before the counted words."},
    {"id": "C-03", "name": "trailing", "kind": "constructor", "type": DECERR, "meaning": "Bytes follow the counted words."},
    {"id": "C-04", "name": "ok", "kind": "constructor", "type": forall([("A", TYPE), ("E", TYPE)], arrow(A, RESULT(A, E))), "meaning": "Success; both type arguments are explicit in shared terms."},
    {"id": "C-05", "name": "err", "kind": "constructor", "type": forall([("A", TYPE), ("E", TYPE)], arrow(E, RESULT(A, E))), "meaning": "Typed failure; both type arguments are explicit in shared terms."},
    {"id": "C-06", "name": "true", "kind": "constructor", "type": BOOL, "meaning": "Truth."},
    {"id": "C-07", "name": "false", "kind": "constructor", "type": BOOL, "meaning": "Falsity."},
    {"id": "C-08", "name": "nil", "kind": "constructor", "type": forall([("A", TYPE)], LIST(A)), "meaning": "The empty list."},
    {"id": "C-09", "name": "cons", "kind": "constructor", "type": forall([("A", TYPE)], arrow(A, LIST(A), LIST(A))), "meaning": "A list with a first element."},
    {"id": "F-01", "name": "word_of_nat", "kind": "function", "type": forall([("w", NAT)], arrow(NAT, WORD(W))), "meaning": "word_of_nat w n is n mod 2^w."},
    {"id": "F-02", "name": "nat_of_word", "kind": "function", "type": forall([("w", NAT)], arrow(WORD(W), NAT)), "meaning": "The natural a word denotes."},
    {"id": "F-03", "name": "word_add", "kind": "function", "type": forall([("w", NAT)], arrow(WORD(W), WORD(W), WORD(W))), "meaning": "Addition mod 2^w."},
    {"id": "F-04", "name": "word_xor", "kind": "function", "type": forall([("w", NAT)], arrow(WORD(W), WORD(W), WORD(W))), "meaning": "Bitwise exclusive or."},
    {"id": "F-05", "name": "word_and", "kind": "function", "type": forall([("w", NAT)], arrow(WORD(W), WORD(W), WORD(W))), "meaning": "Bitwise and."},
    {"id": "F-06", "name": "word_not", "kind": "function", "type": forall([("w", NAT)], arrow(WORD(W), WORD(W))), "meaning": "Bitwise complement: (2^w - 1) - x."},
    {"id": "F-07", "name": "word_rotl", "kind": "function", "type": forall([("w", NAT)], arrow(WORD(W), NAT, WORD(W))), "meaning": "Left rotation by r mod w; the identity when w is 0."},
    {"id": "F-08", "name": "word_shl", "kind": "function", "type": forall([("w", NAT)], arrow(WORD(W), NAT, WORD(W))), "meaning": "Left shift: (x * 2^r) mod 2^w."},
    {"id": "F-09", "name": "be32_of_bytes", "kind": "function", "type": arrow(SEQ(WORD(nat(8)), nat(4)), WORD(nat(32))), "meaning": "Big-endian: the first byte is most significant."},
    {"id": "F-10", "name": "bytes_of_be32", "kind": "function", "type": arrow(WORD(nat(32)), SEQ(WORD(nat(8)), nat(4))), "meaning": "Inverse of F-09."},
    {"id": "F-11", "name": "le32_of_bytes", "kind": "function", "type": arrow(SEQ(WORD(nat(8)), nat(4)), WORD(nat(32))), "meaning": "Little-endian: the first byte is least significant."},
    {"id": "F-12", "name": "bytes_of_le32", "kind": "function", "type": arrow(WORD(nat(32)), SEQ(WORD(nat(8)), nat(4))), "meaning": "Inverse of F-11."},
    {"id": "F-13", "name": "seq_map2", "kind": "function", "type": forall([("A", TYPE), ("n", NAT)], arrow(arrow(A, A, A), SEQ(A, N), SEQ(A, N), SEQ(A, N))), "meaning": "Pointwise combination of two sequences of the same length."},
    {"id": "F-14", "name": "decode_words", "kind": "function", "type": arrow(LIST(WORD(nat(8))), RESULT(LIST(WORD(nat(32))), DECERR)), "meaning": "The canonical decoder: a count byte n at most 16, then exactly 4n bytes read as n big-endian words."},
    {"id": "F-15", "name": "encode_words", "kind": "function", "type": arrow(LIST(WORD(nat(32))), LIST(WORD(nat(8)))), "meaning": "The count (mod 256) as one byte, then each word big-endian."},
    {"id": "F-16", "name": "list_length", "kind": "function", "type": forall([("A", TYPE)], arrow(LIST(A), NAT)), "meaning": "Number of elements."},
    {"id": "F-17", "name": "seq4", "kind": "function", "type": forall([("A", TYPE)], arrow(A, A, A, A, SEQ(A, nat(4)))), "meaning": "The four-element sequence of its arguments in order."},
    {"id": "F-18", "name": "chacha_qr", "kind": "function", "type": arrow(SEQ(WORD(nat(32)), nat(4)), SEQ(WORD(nat(32)), nat(4))), "meaning": "M-01 instantiated at width 32 with rotations 16, 12, 8, 7 (the ChaCha quarter round)."},
    {"id": "F-19", "name": "toy_qr", "kind": "function", "type": arrow(SEQ(WORD(nat(8)), nat(4)), SEQ(WORD(nat(8)), nat(4))), "meaning": "M-01 instantiated at width 8 with rotations 4, 3, 2, 1."},
    {"id": "F-20", "name": "seq_to_list", "kind": "function", "type": forall([("A", TYPE), ("n", NAT)], arrow(SEQ(A, N), LIST(A))), "meaning": "The elements in order."},
    {"id": "F-21", "name": "nat_pow", "kind": "function", "type": arrow(NAT, NAT, NAT), "meaning": "Exponentiation of naturals."},
    {"id": "F-22", "name": "list_append", "kind": "function", "type": forall([("A", TYPE)], arrow(LIST(A), LIST(A), LIST(A))), "meaning": "Concatenation."},
]

CORE_MODULES = [
    {
        "id": "M-01",
        "name": "QuarterRound",
        "parameters": ["w : Nat", "r1 : Nat", "r2 : Nat", "r3 : Nat", "r4 : Nat"],
        "provides": "qr : Seq (Word w) 4 -> Seq (Word w) 4",
        "definition": [
            "given (a, b, c, d):",
            "a := a + b; d := d xor a; d := rotl d r1;",
            "c := c + d; b := b xor c; b := rotl b r2;",
            "a := a + b; d := d xor a; d := rotl d r3;",
            "c := c + d; b := b xor c; b := rotl b r4;",
            "result (a, b, c, d)",
        ],
        "rule": "One parameterized construct, defined once and instantiated twice (F-18, F-19) with the candidate's module or parameter mechanism; the instances are not written out separately.",
    }
]


# --- DS-01 reference semantics --------------------------------------------


def w_of_nat(width: int, value: int) -> int:
    return value % (1 << width)


def w_add(width: int, left: int, right: int) -> int:
    return (left + right) % (1 << width)


def w_not(width: int, value: int) -> int:
    return (1 << width) - 1 - value


def w_rotl(width: int, value: int, amount: int) -> int:
    if width == 0:
        return value
    shift = amount % width
    return ((value << shift) | (value >> (width - shift))) % (1 << width)


def w_shl(width: int, value: int, amount: int) -> int:
    return (value << amount) % (1 << width)


def be32_of_bytes(data: list[int]) -> int:
    return (data[0] << 24) | (data[1] << 16) | (data[2] << 8) | data[3]


def bytes_of_be32(value: int) -> list[int]:
    return [(value >> 24) & 255, (value >> 16) & 255, (value >> 8) & 255, value & 255]


def le32_of_bytes(data: list[int]) -> int:
    return be32_of_bytes(list(reversed(data)))


def bytes_of_le32(value: int) -> list[int]:
    return list(reversed(bytes_of_be32(value)))


def quarter_round(width: int, rotations: tuple[int, int, int, int], state: list[int]) -> list[int]:
    a, b, c, d = state
    r1, r2, r3, r4 = rotations
    a = w_add(width, a, b); d = w_rotl(width, d ^ a, r1)  # noqa: E702
    c = w_add(width, c, d); b = w_rotl(width, b ^ c, r2)  # noqa: E702
    a = w_add(width, a, b); d = w_rotl(width, d ^ a, r3)  # noqa: E702
    c = w_add(width, c, d); b = w_rotl(width, b ^ c, r4)  # noqa: E702
    return [a, b, c, d]


def decode_words(data: list[int]) -> tuple[str, Any]:
    if not data:
        return ("err", "C-02")
    count = data[0]
    if count > 16:
        return ("err", "C-01")
    if len(data) - 1 < 4 * count:
        return ("err", "C-02")
    if len(data) - 1 > 4 * count:
        return ("err", "C-03")
    return ("ok", [be32_of_bytes(data[1 + 4 * i : 5 + 4 * i]) for i in range(count)])


def encode_words(words: list[int]) -> list[int]:
    return [len(words) % 256] + [byte for value in words for byte in bytes_of_be32(value)]


def core_observations() -> list[dict[str, Any]]:
    W8, W32 = WORD(nat(8)), WORD(nat(32))
    words32 = LIST(W32)

    def seq4(kind: list[Any], values: list[int], width: int) -> list[Any]:
        return app(sym("F-17"), kind, *[word(width, value) for value in values])

    def as_list(kind: list[Any], length: int, term: list[Any]) -> list[Any]:
        return app(sym("F-20"), kind, nat(length), term)

    def words_term(width: int, values: list[int]) -> list[Any]:
        return lst(WORD(nat(width)), [word(width, value) for value in values])

    def decoded(data: list[int]) -> list[Any]:
        state, value = decode_words(data)
        if state == "ok":
            return OK(words32, DECERR, words_term(32, value))
        return ERR(words32, DECERR, sym(value))

    chacha_in = [0x11111111, 0x01020304, 0x9B8D6F43, 0x01234567]
    chacha_out = quarter_round(32, (16, 12, 8, 7), chacha_in)
    assert chacha_out == [0xEA2A92F4, 0xCB1CF8CE, 0x4581472E, 0x5881C4BB]
    toy_in = [1, 2, 3, 4]
    rows: list[tuple[str, list[Any], list[Any], str]] = [
        ("D1-O01", app(sym("F-07"), nat(32), word(32, 0x12345678), nat(8)), word(32, w_rotl(32, 0x12345678, 8)), "rotation"),
        ("D1-O02", app(sym("F-03"), nat(32), word(32, 0xFFFFFFFF), word(32, 1)), word(32, w_add(32, 0xFFFFFFFF, 1)), "wrapping addition"),
        ("D1-O03", app(sym("F-04"), nat(8), word(8, 0xA5), word(8, 0x5A)), word(8, 0xA5 ^ 0x5A), "exclusive or"),
        ("D1-O04", app(sym("F-06"), nat(16), word(16, 0)), word(16, w_not(16, 0)), "complement"),
        ("D1-O05", app(sym("F-08"), nat(8), word(8, 0x81), nat(1)), word(8, w_shl(8, 0x81, 1)), "shift drops the high bit"),
        ("D1-O06", as_list(W8, 4, app(sym("F-10"), word(32, 0x01020304))), words_term(8, bytes_of_be32(0x01020304)), "big-endian bytes"),
        ("D1-O07", as_list(W8, 4, app(sym("F-12"), word(32, 0x01020304))), words_term(8, bytes_of_le32(0x01020304)), "little-endian bytes"),
        ("D1-O08", app(sym("F-09"), seq4(W8, [0xDE, 0xAD, 0xBE, 0xEF], 8)), word(32, be32_of_bytes([0xDE, 0xAD, 0xBE, 0xEF])), "big-endian word"),
        ("D1-O09", app(sym("F-11"), seq4(W8, [0xDE, 0xAD, 0xBE, 0xEF], 8)), word(32, le32_of_bytes([0xDE, 0xAD, 0xBE, 0xEF])), "little-endian word"),
        ("D1-O10", as_list(W32, 4, app(sym("F-18"), seq4(W32, chacha_in, 32))), words_term(32, chacha_out), "RFC 8439 section 2.1.1 quarter-round vector"),
        ("D1-O11", as_list(W8, 4, app(sym("F-19"), seq4(W8, toy_in, 8))), words_term(8, quarter_round(8, (4, 3, 2, 1), toy_in)), "second instance of the parameterized module"),
        ("D1-O12", app(sym("F-14"), bytes_term([2, 0, 0, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF])), decoded([2, 0, 0, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF]), "decoder success"),
        ("D1-O13", app(sym("F-14"), bytes_term([])), decoded([]), "empty input is truncated"),
        ("D1-O14", app(sym("F-14"), bytes_term([1, 0, 0, 0])), decoded([1, 0, 0, 0]), "missing byte is truncated"),
        ("D1-O15", app(sym("F-14"), bytes_term([0, 7])), decoded([0, 7]), "extra byte is trailing"),
        ("D1-O16", app(sym("F-14"), bytes_term([17])), decoded([17]), "count above 16"),
        ("D1-O17", app(sym("F-02"), nat(64), app(sym("F-01"), nat(64), nat(2**64 + 5))), nat(5), "reduction mod 2^64 of a literal above 2^64"),
        ("D1-O18", as_list(W8, 4, app(sym("F-13"), W8, nat(4), app(sym("F-04"), nat(8)), seq4(W8, [1, 2, 3, 4], 8), seq4(W8, [0xFF] * 4, 8))), words_term(8, [value ^ 0xFF for value in [1, 2, 3, 4]]), "pointwise map over length-indexed sequences"),
        ("D1-O19", app(sym("F-15"), words_term(32, [0xDEADBEEF])), words_term(8, encode_words([0xDEADBEEF])), "encoder"),
        ("D1-O20", app(sym("F-07"), nat(32), word(32, 0x80000001), nat(33)), word(32, w_rotl(32, 0x80000001, 33)), "rotation amount taken mod the width"),
    ]
    return [{"id": ident, "lhs": lhs, "rhs": rhs, "note": note} for ident, lhs, rhs, note in rows]


def core_theorems() -> list[dict[str, Any]]:
    w, x, y, n, r, s = var("w"), var("x"), var("y"), var("n"), var("r"), var("s")
    W32 = WORD(nat(32))
    words32 = LIST(W32)
    rows = [
        ("D1-TH01", "word values are bounded", forall([("w", NAT), ("x", WORD(w))], lt(app(sym("F-02"), w, x), app(sym("F-21"), nat(2), w)))),
        ("D1-TH02", "small naturals are words", forall([("w", NAT), ("n", NAT)], imp(lt(n, app(sym("F-21"), nat(2), w)), eq(app(sym("F-02"), w, app(sym("F-01"), w, n)), n)))),
        ("D1-TH03", "words are their values", forall([("w", NAT), ("x", WORD(w)), ("y", WORD(w))], imp(eq(app(sym("F-02"), w, x), app(sym("F-02"), w, y)), eq(x, y)))),
        ("D1-TH04", "addition commutes", forall([("w", NAT), ("x", WORD(w)), ("y", WORD(w))], eq(app(sym("F-03"), w, x, y), app(sym("F-03"), w, y, x)))),
        ("D1-TH05", "exclusive or cancels", forall([("w", NAT), ("x", WORD(w))], eq(app(sym("F-04"), w, x, x), app(sym("F-01"), w, nat(0))))),
        ("D1-TH06", "rotations compose", forall([("w", NAT), ("x", WORD(w)), ("r", NAT), ("s", NAT)], eq(app(sym("F-07"), w, app(sym("F-07"), w, x, r), s), app(sym("F-07"), w, x, ["nat_add", r, s])))),
        ("D1-TH07", "big-endian word round trip", forall([("x", W32)], eq(app(sym("F-09"), app(sym("F-10"), x)), x))),
        ("D1-TH08", "big-endian bytes round trip", forall([("s", SEQ(WORD(nat(8)), nat(4)))], eq(app(sym("F-10"), app(sym("F-09"), s)), s))),
        ("D1-TH09", "little-endian word round trip", forall([("x", W32)], eq(app(sym("F-11"), app(sym("F-12"), x)), x))),
        ("D1-TH10", "decoder inverts the encoder", forall([("l", words32)], imp(le(app(sym("F-16"), W32, var("l")), nat(16)), eq(app(sym("F-14"), app(sym("F-15"), var("l"))), OK(words32, DECERR, var("l")))))),
        ("D1-TH11", "decoding is canonical", forall([("b", LIST(WORD(nat(8)))), ("l", words32)], imp(eq(app(sym("F-14"), var("b")), OK(words32, DECERR, var("l"))), eq(app(sym("F-15"), var("l")), var("b"))))),
        ("D1-TH12", "sequences have their index length", forall([("A", TYPE), ("n", NAT), ("s", SEQ(A, n))], eq(app(sym("F-16"), A, app(sym("F-20"), A, n, s)), n))),
    ]
    return [{"id": ident, "name": name, "statement": statement} for ident, name, statement in rows]


def core_negatives() -> list[dict[str, Any]]:
    W32 = WORD(nat(32))
    return [
        {"id": "D1-N01", "name": "ill-typed width", "form": "term", "term": app(sym("F-03"), nat(32), word(32, 1), word(16, 1)), "expected": ["type_failure"]},
        {"id": "D1-N02", "name": "non-total recursion", "form": "recursive_definition", "definition": {"name": "d1_n02_loop", "parameters": [["n", NAT]], "result": NAT, "body": app(var("d1_n02_loop"), ["nat_add", var("n"), nat(1)])}, "expected": ["non_total"]},
        {"id": "D1-N03", "name": "bytes used as a word", "form": "term", "term": app(sym("F-03"), nat(32), app(sym("F-10"), word(32, 5)), word(32, 1)), "expected": ["type_failure"]},
        {"id": "D1-N04", "name": "endian confusion", "form": "obligation", "lhs": app(sym("F-09"), app(sym("F-17"), WORD(nat(8)), word(8, 0xDE), word(8, 0xAD), word(8, 0xBE), word(8, 0xEF))), "rhs": app(sym("F-11"), app(sym("F-17"), WORD(nat(8)), word(8, 0xDE), word(8, 0xAD), word(8, 0xBE), word(8, 0xEF))), "expected": ["disproved_obligation"]},
        {"id": "D1-N05", "name": "invalid decoding claimed", "form": "obligation", "lhs": app(sym("F-14"), bytes_term([1, 0, 0, 0])), "rhs": OK(LIST(W32), DECERR, lst(W32, [word(32, 0)])), "expected": ["disproved_obligation"]},
        {"id": "D1-N06", "name": "ambiguous width", "form": "term", "term": app(sym("F-02"), HOLE, app(sym("F-01"), HOLE, nat(5))), "expected": ["type_failure"]},
        {"id": "D1-N07", "name": "exhaustive evaluation over 2^32 words", "form": "exhaustive", "statement": forall([("x", W32)], eq(app(sym("F-04"), nat(32), var("x"), var("x")), word(32, 0))), "ceiling": "negative", "expected": ["timeout", "resource_exhaustion"]},
        {"id": "D1-N08", "name": "undeclared axiom", "form": "axiom_use", "statement": eq(app(sym("F-03"), nat(32), word(32, 1), word(32, 1)), word(32, 3)), "expected": ["undeclared_trust"]},
        {"id": "D1-N09", "name": "oversized literal power", "form": "obligation", "lhs": app(sym("F-02"), nat(32), app(sym("F-01"), nat(32), app(sym("F-21"), nat(2), app(sym("F-21"), nat(2), nat(40))))), "rhs": nat(0), "ceiling": "negative", "expected": ["timeout", "resource_exhaustion"]},
        {"id": "D1-N10", "name": "decoder accepts trailing bytes", "form": "candidate_patch", "target": "F-14", "change": "The decoder returns the counted words and ignores bytes after them instead of failing with C-03.", "affected": ["D1-TH11"], "expected": ["proof_failure", "disproved_obligation"]},
    ]


# --- DS-02: Sieve, a small constant-time language --------------------------

PUBLIC, SECRET = "S-C01", "S-C02"
TY_WORD, TY_BOOL = "S-C03", "S-C04"

SIEVE_SIGNATURE: list[dict[str, Any]] = [
    {"id": "S-T01", "name": "Label", "kind": "type", "constructors": ["S-C01 public", "S-C02 secret"]},
    {"id": "S-T02", "name": "Ty", "kind": "type", "constructors": ["S-C03 ty_word", "S-C04 ty_bool"]},
    {"id": "S-T03", "name": "Val", "kind": "type", "constructors": ["S-C05 vword (Word 8)", "S-C06 vbool Bool"]},
    {"id": "S-T04", "name": "Expr", "kind": "type", "constructors": ["S-C07 lit Val", "S-C08 var Nat", "S-C09 add Expr Expr", "S-C10 xor Expr Expr", "S-C11 band Expr Expr", "S-C12 eq Expr Expr", "S-C13 get Nat Expr"]},
    {"id": "S-T05", "name": "Cmd", "kind": "type", "constructors": ["S-C14 skip", "S-C15 assign Nat Expr", "S-C16 set Nat Expr Expr", "S-C17 seq Cmd Cmd", "S-C18 cond Expr Cmd Cmd", "S-C19 loop Nat Cmd", "S-C20 declassify Nat Expr"]},
    {"id": "S-T06", "name": "Obs", "kind": "type", "constructors": ["S-C21 read Nat Nat", "S-C22 write Nat Nat", "S-C23 branch Bool", "S-C24 release Val", "S-C25 oob Nat Nat"]},
    {"id": "S-T07", "name": "VarDecl", "kind": "type", "constructors": ["S-C26 vdecl Ty Label"]},
    {"id": "S-T08", "name": "ArrDecl", "kind": "type", "constructors": ["S-C27 adecl Nat Label"]},
    {"id": "S-T09", "name": "Env", "kind": "type", "constructors": ["S-C28 env (List VarDecl) (List ArrDecl)"]},
    {"id": "S-T10", "name": "State", "kind": "type", "constructors": ["S-C29 state (List Val) (List (List (Word 8)))"]},
    {"id": "S-T11", "name": "StepResult", "kind": "type", "constructors": ["S-C30 done", "S-C31 next Cmd State (List Obs)", "S-C32 fail (List Obs)", "S-C33 stuck"]},
    {"id": "S-T12", "name": "Outcome", "kind": "type", "constructors": ["S-C34 halted State", "S-C35 failed", "S-C36 out_of_fuel Cmd State", "S-C37 wedged"]},
    {"id": "S-F01", "name": "well_typed", "kind": "function", "type": "Env -> Cmd -> Bool"},
    {"id": "S-F02", "name": "wf", "kind": "predicate", "type": "Env -> State -> Prop"},
    {"id": "S-F03", "name": "low_eq", "kind": "predicate", "type": "Env -> State -> State -> Prop"},
    {"id": "S-F04", "name": "step", "kind": "function", "type": "State -> Cmd -> StepResult"},
    {"id": "S-F05", "name": "run_trace", "kind": "function", "type": "Nat -> State -> Cmd -> List Obs"},
    {"id": "S-F06", "name": "run_outcome", "kind": "function", "type": "Nat -> State -> Cmd -> Outcome"},
    {"id": "S-F07", "name": "outcome_low_eq", "kind": "predicate", "type": "Env -> Outcome -> Outcome -> Prop"},
]


def c(ident: str, *fields: Any) -> tuple[Any, ...]:
    return (ident, *fields)


def vword(value: int) -> tuple[Any, ...]:
    return ("S-C05", value)


def vbool(value: bool) -> tuple[Any, ...]:
    return ("S-C06", value)


def lit_w(value: int) -> tuple[Any, ...]:
    return ("S-C07", vword(value))


def ev(name: int) -> tuple[Any, ...]:
    return ("S-C08", name)


SKIP = ("S-C14",)


def seq(*commands: tuple[Any, ...]) -> tuple[Any, ...]:
    result = commands[-1]
    for command in reversed(commands[:-1]):
        result = ("S-C17", command, result)
    return result


def join(left: str, right: str) -> str:
    return SECRET if SECRET in (left, right) else PUBLIC


def flows(source: str, target: str) -> bool:
    return source == PUBLIC or target == SECRET


def val_ty(value: tuple[Any, ...]) -> str:
    return TY_WORD if value[0] == "S-C05" else TY_BOOL


def type_expr(env: tuple[Any, ...], expr: tuple[Any, ...]) -> tuple[str, str] | None:
    variables, arrays = env
    kind = expr[0]
    if kind == "S-C07":
        return (val_ty(expr[1]), PUBLIC)
    if kind == "S-C08":
        return tuple(variables[expr[1]]) if expr[1] < len(variables) else None  # type: ignore[return-value]
    if kind in ("S-C09", "S-C10", "S-C11", "S-C12"):
        left, right = type_expr(env, expr[1]), type_expr(env, expr[2])
        if left is None or right is None or left[0] != TY_WORD or right[0] != TY_WORD:
            return None
        return (TY_BOOL if kind == "S-C12" else TY_WORD, join(left[1], right[1]))
    if kind == "S-C13":
        index = type_expr(env, expr[2])
        if expr[1] >= len(arrays) or index != (TY_WORD, PUBLIC):
            return None
        return (TY_WORD, arrays[expr[1]][1])
    raise ValueError(expr)


def well_typed(env: tuple[Any, ...], command: tuple[Any, ...]) -> bool:
    variables, arrays = env
    kind = command[0]
    if kind == "S-C14":
        return True
    if kind in ("S-C15", "S-C20"):
        target, found = command[1], type_expr(env, command[2])
        if target >= len(variables) or found is None or found[0] != variables[target][0]:
            return False
        if kind == "S-C15":
            return flows(found[1], variables[target][1])
        return variables[target][1] == PUBLIC
    if kind == "S-C16":
        index, value = type_expr(env, command[2]), type_expr(env, command[3])
        return (
            command[1] < len(arrays)
            and index == (TY_WORD, PUBLIC)
            and value is not None
            and value[0] == TY_WORD
            and flows(value[1], arrays[command[1]][1])
        )
    if kind == "S-C17":
        return well_typed(env, command[1]) and well_typed(env, command[2])
    if kind == "S-C18":
        return type_expr(env, command[1]) == (TY_BOOL, PUBLIC) and well_typed(env, command[2]) and well_typed(env, command[3])
    if kind == "S-C19":
        return well_typed(env, command[2])
    raise ValueError(command)


def sieve_eval(state: tuple[Any, ...], expr: tuple[Any, ...]) -> tuple[Any, ...]:
    variables, arrays = state
    kind = expr[0]
    if kind == "S-C07":
        return ("ok", expr[1], [])
    if kind == "S-C08":
        return ("ok", variables[expr[1]], []) if expr[1] < len(variables) else ("stuck",)
    if kind in ("S-C09", "S-C10", "S-C11", "S-C12"):
        left = sieve_eval(state, expr[1])
        if left[0] != "ok":
            return left
        right = sieve_eval(state, expr[2])
        if right[0] == "fail":
            return ("fail", left[2] + right[1])
        if right[0] == "stuck":
            return right
        if left[1][0] != "S-C05" or right[1][0] != "S-C05":
            return ("stuck",)
        x, y = left[1][1], right[1][1]
        result = {"S-C09": lambda: vword((x + y) % 256), "S-C10": lambda: vword(x ^ y), "S-C11": lambda: vword(x & y), "S-C12": lambda: vbool(x == y)}[kind]()
        return ("ok", result, left[2] + right[2])
    if kind == "S-C13":
        index = sieve_eval(state, expr[2])
        if index[0] != "ok":
            return index
        if index[1][0] != "S-C05" or expr[1] >= len(arrays):
            return ("stuck",)
        position, cells = index[1][1], arrays[expr[1]]
        if position < len(cells):
            return ("ok", vword(cells[position]), index[2] + [("S-C21", expr[1], position)])
        return ("fail", index[2] + [("S-C25", expr[1], position)])
    raise ValueError(expr)


def assign_var(state: tuple[Any, ...], target: int, value: tuple[Any, ...]) -> tuple[Any, ...]:
    variables = list(state[0])
    variables[target] = value
    return (variables, state[1])


def sieve_step(state: tuple[Any, ...], command: tuple[Any, ...]) -> tuple[Any, ...]:
    kind = command[0]
    if kind == "S-C14":
        return ("S-C30",)
    if kind in ("S-C15", "S-C20"):
        result = sieve_eval(state, command[2])
        if result[0] != "ok":
            return ("S-C32", result[1]) if result[0] == "fail" else ("S-C33",)
        if command[1] >= len(state[0]):
            return ("S-C33",)
        trace = result[2] + ([("S-C24", result[1])] if kind == "S-C20" else [])
        return ("S-C31", SKIP, assign_var(state, command[1], result[1]), trace)
    if kind == "S-C16":
        index = sieve_eval(state, command[2])
        if index[0] != "ok":
            return ("S-C32", index[1]) if index[0] == "fail" else ("S-C33",)
        if index[1][0] != "S-C05":
            return ("S-C33",)
        value = sieve_eval(state, command[3])
        if value[0] == "fail":
            return ("S-C32", index[2] + value[1])
        if value[0] == "stuck" or value[1][0] != "S-C05" or command[1] >= len(state[1]):
            return ("S-C33",)
        array, position = command[1], index[1][1]
        trace = index[2] + value[2]
        if position >= len(state[1][array]):
            return ("S-C32", trace + [("S-C25", array, position)])
        arrays = [list(cells) for cells in state[1]]
        arrays[array][position] = value[1][1]
        return ("S-C31", SKIP, (list(state[0]), arrays), trace + [("S-C22", array, position)])
    if kind == "S-C17":
        if command[1] == SKIP:
            return ("S-C31", command[2], state, [])
        inner = sieve_step(state, command[1])
        if inner[0] == "S-C31":
            return ("S-C31", ("S-C17", inner[1], command[2]), inner[2], inner[3])
        return inner if inner[0] in ("S-C32", "S-C33") else ("S-C33",)
    if kind == "S-C18":
        result = sieve_eval(state, command[1])
        if result[0] != "ok":
            return ("S-C32", result[1]) if result[0] == "fail" else ("S-C33",)
        if result[1][0] != "S-C06":
            return ("S-C33",)
        taken = command[2] if result[1][1] else command[3]
        return ("S-C31", taken, state, result[2] + [("S-C23", result[1][1])])
    if kind == "S-C19":
        if command[1] == 0:
            return ("S-C31", SKIP, state, [])
        return ("S-C31", ("S-C17", command[2], ("S-C19", command[1] - 1, command[2])), state, [])
    raise ValueError(command)


def sieve_run(fuel: int, state: tuple[Any, ...], command: tuple[Any, ...]) -> tuple[tuple[Any, ...], list[Any]]:
    trace: list[Any] = []
    while True:
        if fuel == 0:
            return (("S-C36", command, state), trace)
        result = sieve_step(state, command)
        if result[0] == "S-C30":
            return (("S-C34", state), trace)
        if result[0] == "S-C32":
            return (("S-C35",), trace + result[1])
        if result[0] == "S-C33":
            return (("S-C37",), trace)
        command, state, fuel = result[1], result[2], fuel - 1
        trace = trace + result[3]


def sieve_value_term(value: tuple[Any, ...]) -> list[Any]:
    if value[0] == "S-C05":
        return app(sym("S-C05"), word(8, value[1]))
    return app(sym("S-C06"), boolean(value[1]))


def sieve_term(node: Any) -> list[Any]:
    """Render a Sieve expression, command, observation or outcome as a shared term."""

    kind = node[0]
    if kind in ("S-C05", "S-C06"):
        return sieve_value_term(node)
    if kind == "S-C07":
        return app(sym(kind), sieve_value_term(node[1]))
    if kind in ("S-C08",):
        return app(sym(kind), nat(node[1]))
    if kind in ("S-C09", "S-C10", "S-C11", "S-C12", "S-C17"):
        return app(sym(kind), sieve_term(node[1]), sieve_term(node[2]))
    if kind in ("S-C13", "S-C15", "S-C20"):
        return app(sym(kind), nat(node[1]), sieve_term(node[2]))
    if kind == "S-C16":
        return app(sym(kind), nat(node[1]), sieve_term(node[2]), sieve_term(node[3]))
    if kind == "S-C18":
        return app(sym(kind), sieve_term(node[1]), sieve_term(node[2]), sieve_term(node[3]))
    if kind == "S-C19":
        return app(sym(kind), nat(node[1]), sieve_term(node[2]))
    if kind in ("S-C14", "S-C30", "S-C33", "S-C35", "S-C37"):
        return sym(kind)
    if kind in ("S-C21", "S-C22", "S-C25"):
        return app(sym(kind), nat(node[1]), nat(node[2]))
    if kind == "S-C23":
        return app(sym(kind), boolean(node[1]))
    if kind == "S-C24":
        return app(sym(kind), sieve_value_term(node[1]))
    if kind == "S-C31":
        return app(sym(kind), sieve_term(node[1]), state_term(node[2]), obs_list(node[3]))
    if kind == "S-C32":
        return app(sym(kind), obs_list(node[1]))
    if kind == "S-C34":
        return app(sym(kind), state_term(node[1]))
    if kind == "S-C36":
        return app(sym(kind), sieve_term(node[1]), state_term(node[2]))
    raise ValueError(node)


def obs_list(trace: list[Any]) -> list[Any]:
    return lst(sym("S-T06"), [sieve_term(item) for item in trace])


def state_term(state: tuple[Any, ...]) -> list[Any]:
    variables, arrays = state
    return app(
        sym("S-C29"),
        lst(sym("S-T03"), [sieve_value_term(value) for value in variables]),
        lst(LIST(WORD(nat(8))), [bytes_term(list(cells)) for cells in arrays]),
    )


def env_term(env: tuple[Any, ...]) -> list[Any]:
    variables, arrays = env
    return app(
        sym("S-C28"),
        lst(sym("S-T07"), [app(sym("S-C26"), sym(ty), sym(label)) for ty, label in variables]),
        lst(sym("S-T08"), [app(sym("S-C27"), nat(size), sym(label)) for size, label in arrays]),
    )


# Gamma+: key (secret word), nonce (public word), acc (secret word), ok (public bool);
# table (4 public words), state (4 secret words).
GAMMA = ([(TY_WORD, SECRET), (TY_WORD, PUBLIC), (TY_WORD, SECRET), (TY_BOOL, PUBLIC)], [(4, PUBLIC), (4, SECRET)])
KEY, NONCE, ACC, OK_FLAG = 0, 1, 2, 3
LOW_INDEX = ("S-C11", ev(NONCE), lit_w(3))
P_PLUS = seq(
    ("S-C15", ACC, ("S-C10", ev(KEY), ev(NONCE))),
    ("S-C16", 1, LOW_INDEX, ("S-C09", ev(ACC), ("S-C13", 0, LOW_INDEX))),
    ("S-C19", 2, ("S-C15", ACC, ("S-C09", ev(ACC), ev(KEY)))),
    ("S-C18", ("S-C12", ev(NONCE), lit_w(7)), ("S-C15", ACC, ("S-C10", ev(ACC), ev(NONCE))), SKIP),
    ("S-C20", OK_FLAG, ("S-C12", ev(ACC), ev(KEY))),
)
P_MINUS = ("S-C18", ("S-C12", ev(KEY), lit_w(0)), ("S-C15", ACC, lit_w(1)), ("S-C15", ACC, lit_w(2)))
SIGMA_A = ([vword(0x00), vword(7), vword(0), vbool(False)], [[0x10, 0x20, 0x30, 0x40], [0, 0, 0, 0]])
SIGMA_B = ([vword(0x01), vword(7), vword(0), vbool(False)], [[0x10, 0x20, 0x30, 0x40], [0, 0, 0, 0]])
RUN_FUEL = 32


def sieve_statements() -> list[dict[str, Any]]:
    g, command, s, s1, s2 = var("g"), var("c"), var("s"), var("s1"), var("s2")
    ENV, CMD, STATE, OBS, VAL = sym("S-T09"), sym("S-T05"), sym("S-T10"), sym("S-T06"), sym("S-T03")
    OBSL = LIST(OBS)

    def typed(env: list[Any], command: list[Any]) -> list[Any]:
        return eq(app(sym("S-F01"), env, command), TRUE)

    def step(state: list[Any], command: list[Any]) -> list[Any]:
        return app(sym("S-F04"), state, command)

    def release_tail(prefix: list[Any], value: list[Any]) -> list[Any]:
        return app(sym("F-22"), OBS, prefix, lst(OBS, [app(sym("S-C24"), value)]))

    def ni_step_body(env: list[Any], command: list[Any], left: list[Any], right: list[Any]) -> list[Any]:
        return disj(
            conj(eq(step(left, command), sym("S-C30")), eq(step(right, command), sym("S-C30"))),
            exists([("t", OBSL)], conj(eq(step(left, command), app(sym("S-C32"), var("t"))), eq(step(right, command), app(sym("S-C32"), var("t"))))),
            exists(
                [("c'", CMD), ("s1'", STATE), ("s2'", STATE), ("t", OBSL)],
                conj(
                    eq(step(left, command), app(sym("S-C31"), var("c'"), var("s1'"), var("t"))),
                    eq(step(right, command), app(sym("S-C31"), var("c'"), var("s2'"), var("t"))),
                    app(sym("S-F03"), env, var("s1'"), var("s2'")),
                ),
            ),
            exists(
                [("c'", CMD), ("s1'", STATE), ("s2'", STATE), ("p", OBSL), ("v1", VAL), ("v2", VAL)],
                conj(
                    eq(step(left, command), app(sym("S-C31"), var("c'"), var("s1'"), release_tail(var("p"), var("v1")))),
                    eq(step(right, command), app(sym("S-C31"), var("c'"), var("s2'"), release_tail(var("p"), var("v2")))),
                    ne(var("v1"), var("v2")),
                ),
            ),
        )

    def ni_run_body(env: list[Any], command: list[Any], left: list[Any], right: list[Any]) -> list[Any]:
        def trace(state: list[Any]) -> list[Any]:
            return app(sym("S-F05"), var("n"), state, command)

        def outcome(state: list[Any]) -> list[Any]:
            return app(sym("S-F06"), var("n"), state, command)

        def split(prefix: list[Any], value: list[Any], rest: list[Any]) -> list[Any]:
            return app(sym("F-22"), OBS, prefix, app(sym("C-09"), OBS, app(sym("S-C24"), value), rest))

        return disj(
            conj(eq(trace(left), trace(right)), app(sym("S-F07"), env, outcome(left), outcome(right))),
            exists(
                [("p", OBSL), ("v1", VAL), ("v2", VAL), ("r1", OBSL), ("r2", OBSL)],
                conj(
                    eq(trace(left), split(var("p"), var("v1"), var("r1"))),
                    eq(trace(right), split(var("p"), var("v2"), var("r2"))),
                    ne(var("v1"), var("v2")),
                ),
            ),
        )

    def two_states(env: list[Any], body: list[Any]) -> list[Any]:
        return imp(app(sym("S-F02"), env, s1), imp(app(sym("S-F02"), env, s2), imp(app(sym("S-F03"), env, s1, s2), body)))

    gamma, p_plus, p_minus = env_term(GAMMA), sieve_term(P_PLUS), sieve_term(P_MINUS)
    sigma_a, sigma_b = state_term(SIGMA_A), state_term(SIGMA_B)
    minus_trace = [app(sym("S-F05"), nat(8), sigma, p_minus) for sigma in (sigma_a, sigma_b)]
    rows = [
        ("D2-TH01", "progress", forall([("g", ENV), ("c", CMD), ("s", STATE)], imp(typed(g, command), imp(app(sym("S-F02"), g, s), ne(step(s, command), sym("S-C33")))))),
        (
            "D2-TH02",
            "preservation",
            forall(
                [("g", ENV), ("c", CMD), ("s", STATE), ("c'", CMD), ("s'", STATE), ("t", OBSL)],
                imp(typed(g, command), imp(app(sym("S-F02"), g, s), imp(eq(step(s, command), app(sym("S-C31"), var("c'"), var("s'"), var("t"))), conj(typed(g, var("c'")), app(sym("S-F02"), g, var("s'")))))),
            ),
        ),
        ("D2-TH03", "lock-step noninterference", forall([("g", ENV), ("c", CMD), ("s1", STATE), ("s2", STATE)], imp(typed(g, command), two_states(g, ni_step_body(g, command, s1, s2))))),
        ("D2-TH04", "two-run noninterference up to release", forall([("n", NAT), ("g", ENV), ("c", CMD), ("s1", STATE), ("s2", STATE)], imp(typed(g, command), two_states(g, ni_run_body(g, command, s1, s2))))),
        ("D2-TH05", "the positive program is a checked witness", forall([("n", NAT), ("s1", STATE), ("s2", STATE)], two_states(gamma, ni_run_body(gamma, p_plus, s1, s2)))),
        (
            "D2-TH06",
            "the negative program leaks through control flow",
            conj(
                app(sym("S-F02"), gamma, sigma_a),
                app(sym("S-F02"), gamma, sigma_b),
                app(sym("S-F03"), gamma, sigma_a, sigma_b),
                exists(
                    [("p", OBSL), ("r1", OBSL), ("r2", OBSL)],
                    conj(
                        eq(minus_trace[0], app(sym("F-22"), OBS, var("p"), app(sym("C-09"), OBS, app(sym("S-C23"), TRUE), var("r1")))),
                        eq(minus_trace[1], app(sym("F-22"), OBS, var("p"), app(sym("C-09"), OBS, app(sym("S-C23"), FALSE), var("r2")))),
                    ),
                ),
            ),
        ),
    ]
    return [{"id": ident, "name": name, "statement": statement} for ident, name, statement in rows]


TYPING_FIXTURES: list[tuple[str, tuple[Any, ...], str]] = [
    ("T01", ("S-C15", NONCE, ev(KEY)), "secret into a public variable"),
    ("T02", ("S-C16", 0, lit_w(1), ev(KEY)), "secret into a public array"),
    ("T03", ("S-C16", 1, ev(KEY), lit_w(1)), "secret index"),
    ("T04", ("S-C15", ACC, ("S-C13", 0, ev(KEY))), "secret index in a read"),
    ("T05", ("S-C20", ACC, ev(KEY)), "declassify into a secret variable"),
    ("T06", ("S-C15", OK_FLAG, ev(NONCE)), "word into a bool variable"),
    ("T07", ("S-C15", 9, lit_w(0)), "undeclared variable"),
    ("T08", ("S-C16", 5, lit_w(0), lit_w(0)), "undeclared array"),
    ("T09", ("S-C19", 3, ("S-C15", NONCE, ("S-C09", ev(NONCE), lit_w(1)))), "bounded loop over public state"),
    ("T10", ("S-C18", ev(OK_FLAG), SKIP, SKIP), "public branch"),
    ("T11", ("S-C15", ACC, ("S-C13", 1, ev(NONCE))), "public index into a secret array"),
    ("T12", ("S-C18", ("S-C12", ("S-C13", 1, lit_w(0)), lit_w(0)), SKIP, SKIP), "secret array read in a branch condition"),
]

STEP_FIXTURES: list[tuple[str, tuple[Any, ...], str]] = [
    ("S01", ("S-C16", 0, lit_w(9), lit_w(1)), "write out of bounds fails explicitly"),
    ("S02", ("S-C15", ACC, ("S-C09", ("S-C07", vbool(True)), lit_w(1))), "ill-typed arithmetic is stuck"),
    ("S03", seq(SKIP, SKIP), "sequencing after skip is silent"),
    ("S04", ("S-C15", ACC, ("S-C13", 0, lit_w(5))), "read out of bounds fails explicitly"),
    ("S05", ("S-C19", 0, SKIP), "an exhausted loop is skip"),
    ("S06", ("S-C18", ("S-C12", ev(NONCE), lit_w(7)), SKIP, ("S-C15", ACC, lit_w(1))), "a public branch reports its outcome"),
]


def sieve_observations() -> list[dict[str, Any]]:
    gamma = env_term(GAMMA)
    rows: list[dict[str, Any]] = [
        {"id": "D2-O01", "lhs": app(sym("S-F01"), gamma, sieve_term(P_PLUS)), "rhs": boolean(well_typed(GAMMA, P_PLUS)), "note": "the positive program is well typed"},
        {"id": "D2-O02", "lhs": app(sym("S-F01"), gamma, sieve_term(P_MINUS)), "rhs": boolean(well_typed(GAMMA, P_MINUS)), "note": "the negative program is rejected"},
    ]
    assert well_typed(GAMMA, P_PLUS) and not well_typed(GAMMA, P_MINUS)
    for index, (label, command, note) in enumerate(TYPING_FIXTURES, start=3):
        rows.append({"id": f"D2-O{index:02d}", "fixture": label, "lhs": app(sym("S-F01"), gamma, sieve_term(command)), "rhs": boolean(well_typed(GAMMA, command)), "note": note})
    index = 3 + len(TYPING_FIXTURES)
    for label, command, note in STEP_FIXTURES:
        rows.append({"id": f"D2-O{index:02d}", "fixture": label, "lhs": app(sym("S-F04"), state_term(SIGMA_A), sieve_term(command)), "rhs": sieve_term(sieve_step(SIGMA_A, command)), "note": note})
        index += 1
    for sigma_label, sigma in (("sigma_a", SIGMA_A), ("sigma_b", SIGMA_B)):
        outcome, trace = sieve_run(RUN_FUEL, sigma, P_PLUS)
        rows.append({"id": f"D2-O{index:02d}", "lhs": app(sym("S-F05"), nat(RUN_FUEL), state_term(sigma), sieve_term(P_PLUS)), "rhs": obs_list(trace), "note": f"positive program trace from {sigma_label}"})
        rows.append({"id": f"D2-O{index + 1:02d}", "lhs": app(sym("S-F06"), nat(RUN_FUEL), state_term(sigma), sieve_term(P_PLUS)), "rhs": sieve_term(outcome), "note": f"positive program outcome from {sigma_label}"})
        index += 2
    for sigma_label, sigma in (("sigma_a", SIGMA_A), ("sigma_b", SIGMA_B)):
        outcome, trace = sieve_run(8, sigma, P_MINUS)
        rows.append({"id": f"D2-O{index:02d}", "lhs": app(sym("S-F05"), nat(8), state_term(sigma), sieve_term(P_MINUS)), "rhs": obs_list(trace), "note": f"negative program trace from {sigma_label}"})
        index += 1
    outcome, trace = sieve_run(0, SIGMA_A, P_PLUS)
    rows.append({"id": f"D2-O{index:02d}", "lhs": app(sym("S-F06"), nat(0), state_term(SIGMA_A), sieve_term(P_PLUS)), "rhs": sieve_term(outcome), "note": "no fuel"})
    return rows


def sieve_negatives() -> list[dict[str, Any]]:
    mislabeled = (GAMMA[0][:ACC] + [(TY_WORD, PUBLIC)] + GAMMA[0][ACC + 1 :], GAMMA[1])
    exposed = seq(*_plus_parts()[:3], ("S-C18", ("S-C12", ev(KEY), lit_w(7)), ("S-C15", ACC, ("S-C10", ev(ACC), ev(NONCE))), SKIP), _plus_parts()[4])
    assert well_typed(GAMMA, P_PLUS) and not well_typed(mislabeled, P_PLUS) and not well_typed(GAMMA, exposed)
    return [
        {"id": "D2-M01", "name": "remove the public-condition premise", "form": "candidate_patch", "target": "S-F01", "change": "The branch rule no longer requires its condition to be public.", "affected": ["D2-TH03", "D2-TH04"], "expected": ["proof_failure"]},
        {"id": "D2-M02", "name": "alter one transition", "form": "candidate_patch", "target": "S-F04", "change": "Assignment stores its value into variable x + 1 instead of x.", "affected": ["D2-TH02", "D2-TH03"], "expected": ["proof_failure"]},
        {"id": "D2-M03", "name": "mislabel one secret", "form": "obligation", "lhs": app(sym("S-F01"), env_term(mislabeled), sieve_term(P_PLUS)), "rhs": TRUE, "expected": ["disproved_obligation"]},
        {"id": "D2-M04", "name": "expose one secret-dependent branch", "form": "obligation", "lhs": app(sym("S-F01"), env_term(GAMMA), sieve_term(exposed)), "rhs": TRUE, "expected": ["disproved_obligation"]},
        {"id": "D2-M05", "name": "corrupt the compiled proof object", "form": "artifact_truncation", "target": "D2-TH03", "change": "The compiled artifact holding D2-TH03 is cut to half its length before an independent kernel check or load.", "expected": ["parse_failure", "proof_failure"], "crash_is": "failure"},
        {"id": "D2-M06", "name": "substitute one proof", "form": "candidate_patch", "target": "D2-TH03", "change": "The proof of D2-TH03 is replaced by the proof of D2-TH02.", "affected": ["D2-TH03"], "expected": ["proof_failure", "type_failure"]},
    ]


def _plus_parts() -> list[tuple[Any, ...]]:
    parts, node = [], P_PLUS
    while node[0] == "S-C17":
        parts.append(node[1])
        node = node[2]
    return parts + [node]


# --- DS-03: canonical records ----------------------------------------------

MAGIC = b"OCR"
MAX_INPUT = 4096
MAX_RECORDS = 64
MAX_NAME = 200
MAX_REFS = 64
RECORD_ERRORS = [
    ("R-C01", "oversized"),
    ("R-C02", "bad_magic"),
    ("R-C03", "unknown_version"),
    ("R-C04", "truncated"),
    ("R-C05", "malformed_number"),
    ("R-C06", "noncanonical_number"),
    ("R-C07", "unknown_field"),
    ("R-C08", "invalid_name"),
    ("R-C09", "invalid_utf8"),
    ("R-C10", "duplicate_name"),
    ("R-C11", "noncanonical_order"),
    ("R-C12", "reference_escape"),
    ("R-C13", "cyclic_reference"),
    ("R-C14", "reference_kind"),
    ("R-C15", "invalid_level"),
    ("R-C16", "trailing_data"),
]
ERROR_ID = {name: ident for ident, name in RECORD_ERRORS}


class RecordError(Exception):
    def __init__(self, code: str, path: str) -> None:
        super().__init__(code, path)
        self.code, self.path = code, path


def utf8_valid(data: bytes) -> bool:
    index = 0
    while index < len(data):
        lead = data[index]
        if lead < 0x80:
            index += 1
            continue
        if 0xC2 <= lead <= 0xDF:
            ranges = [(0x80, 0xBF)]
        elif lead == 0xE0:
            ranges = [(0xA0, 0xBF), (0x80, 0xBF)]
        elif 0xE1 <= lead <= 0xEC or 0xEE <= lead <= 0xEF:
            ranges = [(0x80, 0xBF), (0x80, 0xBF)]
        elif lead == 0xED:
            ranges = [(0x80, 0x9F), (0x80, 0xBF)]
        elif lead == 0xF0:
            ranges = [(0x90, 0xBF), (0x80, 0xBF), (0x80, 0xBF)]
        elif 0xF1 <= lead <= 0xF3:
            ranges = [(0x80, 0xBF)] * 3
        elif lead == 0xF4:
            ranges = [(0x80, 0x8F), (0x80, 0xBF), (0x80, 0xBF)]
        else:
            return False
        for offset, (low, high) in enumerate(ranges, start=1):
            if index + offset >= len(data) or not low <= data[index + offset] <= high:
                return False
        index += 1 + len(ranges)
    return True


def uvar_encode(value: int) -> bytes:
    if value < 0x80:
        return bytes([value])
    if value < 0x4000:
        return bytes([0x80 | (value % 128), value // 128])
    raise ValueError("uvar out of range")


def decode_records(data: bytes) -> list[tuple[Any, ...]]:
    if len(data) > MAX_INPUT:
        raise RecordError("oversized", "input")
    position = 0

    def take(count: int, path: str) -> bytes:
        nonlocal position
        if position + count > len(data):
            raise RecordError("truncated", path)
        chunk = data[position : position + count]
        position += count
        return chunk

    def uvar(path: str) -> int:
        first = take(1, path)[0]
        if first < 0x80:
            return first
        second = take(1, path)[0]
        if second >= 0x80:
            raise RecordError("malformed_number", path)
        if second == 0:
            raise RecordError("noncanonical_number", path)
        return (first - 0x80) + 128 * second

    for expected in MAGIC:
        if position >= len(data):
            raise RecordError("truncated", "header")
        if data[position] != expected:
            raise RecordError("bad_magic", "header")
        position += 1
    if take(1, "header")[0] != 1:
        raise RecordError("unknown_version", "header")
    count = uvar("count")
    if count > MAX_RECORDS:
        raise RecordError("oversized", "count")
    records: list[tuple[Any, ...]] = []
    previous: bytes | None = None
    for index in range(count):
        base = f"records/{index}"
        tag = take(1, base + "/tag")[0]
        if tag not in (1, 2, 3):
            raise RecordError("unknown_field", base + "/tag")
        length = uvar(base + "/name")
        if length == 0 or length > MAX_NAME:
            raise RecordError("invalid_name", base + "/name")
        name = take(length, base + "/name")
        if not utf8_valid(name):
            raise RecordError("invalid_utf8", base + "/name")
        if previous is not None and name == previous:
            raise RecordError("duplicate_name", base + "/name")
        if previous is not None and name < previous:
            raise RecordError("noncanonical_order", base + "/name")
        previous = name
        if tag == 1:
            records.append(("def", name, take(32, base + "/digest")))
            continue
        if tag == 2:
            fingerprint = take(32, base + "/fingerprint")
            total = uvar(base + "/refs")
            if total > MAX_REFS:
                raise RecordError("oversized", base + "/refs")
            references: list[int] = []
            for slot in range(total):
                path = f"{base}/refs/{slot}"
                reference = uvar(path)
                if reference >= count:
                    raise RecordError("reference_escape", path)
                if reference >= index:
                    raise RecordError("cyclic_reference", path)
                if references and reference <= references[-1]:
                    raise RecordError("noncanonical_order", path)
                references.append(reference)
            records.append(("thm", name, fingerprint, references))
            continue
        reference = uvar(base + "/ref")
        if reference >= count:
            raise RecordError("reference_escape", base + "/ref")
        if reference >= index:
            raise RecordError("cyclic_reference", base + "/ref")
        if records[reference][0] != "thm":
            raise RecordError("reference_kind", base + "/ref")
        level = take(1, base + "/level")[0]
        if level > 3:
            raise RecordError("invalid_level", base + "/level")
        records.append(("claim", name, reference, level))
    if position != len(data):
        raise RecordError("trailing_data", "trailing")
    return records


def encode_records(records: list[tuple[Any, ...]]) -> bytes:
    out = bytearray(MAGIC + b"\x01" + uvar_encode(len(records)))
    for record in records:
        kind, name = record[0], record[1]
        out += bytes([{"def": 1, "thm": 2, "claim": 3}[kind]]) + uvar_encode(len(name)) + name
        if kind == "def":
            out += record[2]
        elif kind == "thm":
            out += record[2] + uvar_encode(len(record[3])) + b"".join(uvar_encode(reference) for reference in record[3])
        else:
            out += uvar_encode(record[2]) + bytes([record[3]])
    return bytes(out)


def records_valid(records: list[tuple[Any, ...]]) -> bool:
    try:
        return decode_records(encode_records(records)) == records
    except (RecordError, ValueError):
        return False


def digest_of(label: str) -> bytes:
    return hashlib.sha256(label.encode("ascii")).digest()


def record_fixtures() -> list[tuple[str, bytes, str]]:
    base = [
        ("def", b"alpha", digest_of("alpha")),
        ("thm", b"beta", digest_of("beta"), [0]),
        ("claim", b"gamma", 1, 2),
    ]
    good = encode_records(base)
    long_name = ("é" * 75).encode("utf-8")
    unicode_names = [
        ("def", "aé".encode("utf-8"), digest_of("a")),
        ("def", "b∀".encode("utf-8"), digest_of("b")),
        ("thm", "c\U0001d53d".encode("utf-8"), digest_of("c"), [0, 1]),
    ]

    def with_name(name: bytes) -> bytes:
        return encode_records([("def", name, digest_of("x"))])

    def patch(data: bytes, offset: int, replacement: bytes, remove: int = 1) -> bytes:
        return data[:offset] + replacement + data[offset + remove :]

    count_at = len(MAGIC) + 1
    rows = [
        ("R-F01", good, "three records: a definition, a theorem citing it, and a claim on the theorem"),
        ("R-F02", MAGIC + b"\x01\x00", "the empty record set"),
        ("R-F03", encode_records([("def", long_name, digest_of("long"))]), "a 150-byte name whose length takes a two-byte number"),
        ("R-F04", encode_records(unicode_names), "names with two-, three- and four-byte UTF-8 sequences"),
        ("R-F05", encode_records([("def", b"a", digest_of("a")), ("def", b"a", digest_of("b"))]), "duplicate name"),
        ("R-F06", encode_records([("def", b"b", digest_of("b")), ("def", b"a", digest_of("a"))]), "names out of order"),
        ("R-F07", with_name(b"\xc0\x80"), "overlong encoding of U+0000"),
        ("R-F08", with_name(b"\xed\xa0\x80"), "encoded surrogate"),
        ("R-F09", with_name(b"\xf4\x90\x80\x80"), "code point above U+10FFFF"),
        ("R-F10", with_name(b"a\x80"), "lone continuation byte"),
        ("R-F11", with_name(b"a\xe2\x88"), "truncated three-byte sequence"),
        ("R-F12", MAGIC + b"\x01\x81\x80\x01", "number longer than two bytes"),
        ("R-F13", MAGIC + b"\x01\x80\x00", "zero in the high byte of a two-byte number"),
        ("R-F14", patch(good, count_at, b"\x83\x00"), "record count 3 written as a two-byte number"),
        ("R-F15", patch(good, count_at + 1, b"\x07"), "unknown record tag"),
        ("R-F16", encode_records([("thm", b"t", digest_of("t"), [])])[:-1] + b"\x01\x05", "reference outside the record set"),
        ("R-F17", encode_records([("def", b"a", digest_of("a")), ("thm", b"b", digest_of("b"), [])])[:-1] + b"\x01\x01", "theorem citing itself"),
        ("R-F18", MAGIC + b"\x01\x02" + b"\x02" + uvar_encode(1) + b"a" + digest_of("a") + b"\x01\x01" + b"\x01" + uvar_encode(1) + b"b" + digest_of("b"), "forward reference"),
        ("R-F19", encode_records([("def", b"a", digest_of("a"))]) + b"\x00" * (MAX_INPUT + 1 - len(encode_records([("def", b"a", digest_of("a"))]))), "input above 4096 bytes"),
        ("R-F20", MAGIC + b"\x01" + uvar_encode(65), "record count above 64"),
        ("R-F21", b"OCX\x01\x00", "wrong magic"),
        ("R-F22", MAGIC + b"\x02\x00", "unknown version"),
        ("R-F23", good[:40], "input cut inside a digest"),
        ("R-F24", good + b"\x00", "byte after the last record"),
        ("R-F25", encode_records([("def", b"a", digest_of("a")), ("claim", b"b", 0, 1)]), "claim citing a definition"),
        ("R-F26", good[:-1] + b"\x04", "claim level 4"),
        ("R-F27", MAGIC + b"\x01\x01\x01\x00", "empty name"),
        ("R-F28", encode_records([("def", b"a", digest_of("a")), ("def", b"b", digest_of("b")), ("thm", b"c", digest_of("c"), [0, 1])])[:-2] + b"\x01\x00", "references out of order"),
        ("R-F29", MAGIC + b"\x01\x01\x01" + b"\x85\x00" + b"alpha", "name length 5 written as a two-byte number"),
        ("R-F30", b"OC", "input ends inside the magic"),
    ]
    return rows


def record_value_term(records: list[tuple[Any, ...]]) -> list[Any]:
    items = []
    for record in records:
        if record[0] == "def":
            items.append(app(sym("R-C17"), bytes_term(list(record[1])), bytes_term(list(record[2]))))
        elif record[0] == "thm":
            items.append(app(sym("R-C18"), bytes_term(list(record[1])), bytes_term(list(record[2])), lst(NAT, [nat(reference) for reference in record[3]])))
        else:
            items.append(app(sym("R-C19"), bytes_term(list(record[1])), nat(record[2]), nat(record[3])))
    return lst(sym("R-T01"), items)


RECORD_SIGNATURE: list[dict[str, Any]] = [
    {"id": "R-T01", "name": "Record", "kind": "type", "constructors": ["R-C17 rdef (name : List (Word 8)) (digest : List (Word 8))", "R-C18 rthm (name : List (Word 8)) (fingerprint : List (Word 8)) (refs : List Nat)", "R-C19 rclaim (name : List (Word 8)) (ref : Nat) (level : Nat)"]},
    {"id": "R-T02", "name": "ErrorCode", "kind": "type", "constructors": [f"{ident} {name}" for ident, name in RECORD_ERRORS]},
    {"id": "R-T03", "name": "Failure", "kind": "type", "constructors": ["R-C20 failure (code : ErrorCode) (path : String)"]},
    {"id": "R-F01", "name": "decode_records", "kind": "function", "type": "List (Word 8) -> Result (List Record) Failure"},
    {"id": "R-F02", "name": "encode_records", "kind": "function", "type": "List Record -> List (Word 8)"},
    {"id": "R-F03", "name": "records_valid", "kind": "function", "type": "List Record -> Bool"},
    {"id": "R-F04", "name": "utf8_valid", "kind": "function", "type": "List (Word 8) -> Bool"},
]


def record_observations() -> list[dict[str, Any]]:
    REC, FAIL = LIST(sym("R-T01")), sym("R-T03")
    rows = []
    for ident, data, note in record_fixtures():
        try:
            value = decode_records(data)
            expected: list[Any] = OK(REC, FAIL, record_value_term(value))
            verdict = "accept " + " ".join(render_record(record) for record in value)
            verdict = verdict.rstrip()
        except RecordError as error:
            expected = ERR(REC, FAIL, app(sym("R-C20"), sym(ERROR_ID[error.code]), string(error.path)))
            verdict = f"reject {error.code} {error.path}"
        rows.append({"id": ident.replace("R-F", "D3-O"), "fixture": ident, "lhs": app(sym("R-F01"), bytes_term(list(data))), "rhs": expected, "standalone": verdict, "note": note})
    values = [
        ("valid record set", decode_records(record_fixtures()[0][1])),
        ("the empty set", []),
        ("duplicate names", [("def", b"a", digest_of("a")), ("def", b"a", digest_of("b"))]),
        ("a 31-byte digest", [("def", b"a", digest_of("a")[:31])]),
        ("a claim citing a definition", [("def", b"a", digest_of("a")), ("claim", b"b", 0, 1)]),
        ("a theorem citing itself", [("thm", b"a", digest_of("a"), [0])]),
        ("a claim level of 4", [("def", b"a", digest_of("a")), ("thm", b"b", digest_of("b"), [0]), ("claim", b"c", 1, 4)]),
    ]
    for offset, (note, value) in enumerate(values):
        rows.append({"id": f"D3-O{31 + offset}", "lhs": app(sym("R-F03"), record_value_term(value)), "rhs": boolean(records_valid(value)), "note": note})
    names = [
        ("ASCII", b"alpha"),
        ("empty", b""),
        ("four-byte sequence", "\U0001d53d".encode("utf-8")),
        ("overlong", b"\xc0\x80"),
        ("surrogate", b"\xed\xa0\x80"),
        ("above U+10FFFF", b"\xf4\x90\x80\x80"),
        ("lone continuation", b"a\x80"),
    ]
    for offset, (note, name) in enumerate(names):
        rows.append({"id": f"D3-O{31 + len(values) + offset}", "lhs": app(sym("R-F04"), bytes_term(list(name))), "rhs": boolean(utf8_valid(name)), "note": note})
    return rows


def render_record(record: tuple[Any, ...]) -> str:
    if record[0] == "def":
        return f"def:{record[1].hex()}:{record[2].hex()}"
    if record[0] == "thm":
        return f"thm:{record[1].hex()}:{record[2].hex()}:{','.join(str(reference) for reference in record[3])}"
    return f"claim:{record[1].hex()}:{record[2]}:{record[3]}"


def record_statements() -> list[dict[str, Any]]:
    REC, FAIL, BYTES = LIST(sym("R-T01")), sym("R-T03"), LIST(WORD(nat(8)))
    rs, bs = var("l"), var("b")

    def decodes(data: list[Any], value: list[Any]) -> list[Any]:
        return eq(app(sym("R-F01"), data), OK(REC, FAIL, value))

    rows = [
        ("D3-TH01", "encoding round-trips", forall([("l", REC)], imp(eq(app(sym("R-F03"), rs), TRUE), decodes(app(sym("R-F02"), rs), rs)))),
        ("D3-TH02", "accepted bytes are canonical", forall([("b", BYTES), ("l", REC)], imp(decodes(bs, rs), eq(app(sym("R-F02"), rs), bs)))),
        ("D3-TH03", "accepted values are valid", forall([("b", BYTES), ("l", REC)], imp(decodes(bs, rs), eq(app(sym("R-F03"), rs), TRUE)))),
        ("D3-TH04", "accepted inputs are bounded", forall([("b", BYTES), ("l", REC)], imp(decodes(bs, rs), le(app(sym("F-16"), WORD(nat(8)), bs), nat(MAX_INPUT))))),
        ("D3-TH05", "one value has one encoding", forall([("b1", BYTES), ("b2", BYTES), ("l", REC)], imp(decodes(var("b1"), rs), imp(decodes(var("b2"), rs), eq(var("b1"), var("b2")))))),
    ]
    return [{"id": ident, "name": name, "statement": statement} for ident, name, statement in rows]


# --- DS-04: canonical bit-blast, CNF and LRAT -------------------------------

WIDTH = 32


def obligation_terms() -> dict[str, dict[str, Any]]:
    x, y = ("x",), ("y",)
    return {
        "B-C01": {"lhs": ("add", x, y), "rhs": ("add", ("xor", x, y), ("shl", ("and", x, y), 1)), "holds": True, "name": "carry-save decomposition of modular addition"},
        "B-C02": {"lhs": ("add", x, y), "rhs": ("add", ("xor", x, y), ("and", x, y)), "holds": False, "name": "the same identity without the carry shift"},
    }


def bv_term(node: tuple[Any, ...]) -> list[Any]:
    if node[0] in ("x", "y"):
        return var(node[0])
    function = {"add": "F-03", "xor": "F-04", "and": "F-05", "shl": "F-08"}[node[0]]
    if node[0] == "shl":
        return app(sym(function), nat(WIDTH), bv_term(node[1]), nat(node[2]))
    return app(sym(function), nat(WIDTH), bv_term(node[1]), bv_term(node[2]))


def bv_eval(node: tuple[Any, ...], x: int, y: int) -> int:
    if node[0] == "x":
        return x
    if node[0] == "y":
        return y
    if node[0] == "shl":
        return w_shl(WIDTH, bv_eval(node[1], x, y), node[2])
    left, right = bv_eval(node[1], x, y), bv_eval(node[2], x, y)
    return {"add": w_add(WIDTH, left, right), "xor": left ^ right, "and": left & right}[node[0]]


def blast(obligation: str) -> tuple[int, list[list[int]]]:
    """The canonical bit-blast of an obligation, as the shared rules fix it."""

    terms = obligation_terms()[obligation]
    clauses: list[list[int]] = [[-1]]
    counter = [1 + 2 * WIDTH]
    inputs = {"x": [2 + i for i in range(WIDTH)], "y": [2 + WIDTH + i for i in range(WIDTH)]}

    def gate(kind: str, a: int, b: int) -> int:
        counter[0] += 1
        g = counter[0]
        if kind == "and":
            clauses.extend([[-g, a], [-g, b], [g, -a, -b]])
        elif kind == "xor":
            clauses.extend([[-g, a, b], [-g, -a, -b], [g, -a, b], [g, a, -b]])
        else:
            clauses.extend([[g, -a], [g, -b], [-g, a, b]])
        return g

    def bits(node: tuple[Any, ...]) -> list[int]:
        if node[0] in inputs:
            return inputs[node[0]]
        if node[0] == "shl":
            source = bits(node[1])
            return [1 if i < node[2] else source[i - node[2]] for i in range(WIDTH)]
        left = bits(node[1])
        right = bits(node[2])
        if node[0] in ("xor", "and"):
            return [gate(node[0], left[i], right[i]) for i in range(WIDTH)]
        carry, total = 1, []
        for i in range(WIDTH):
            t = gate("xor", left[i], right[i])
            total.append(gate("xor", t, carry))
            u = gate("and", left[i], right[i])
            v = gate("and", carry, t)
            carry = gate("or", u, v)
        return total

    lhs = bits(terms["lhs"])
    rhs = bits(terms["rhs"])
    differences = [gate("xor", lhs[i], rhs[i]) for i in range(WIDTH)]
    output = differences[0]
    for difference in differences[1:]:
        output = gate("or", output, difference)
    clauses.append([output])
    return counter[0], clauses


def cnf_text(obligation: str) -> str:
    variables, clauses = blast(obligation)
    return f"p cnf {variables} {len(clauses)}\n" + "".join(" ".join(str(literal) for literal in clause) + " 0\n" for clause in clauses)


MAX_CERTIFICATE = 4 * 1024 * 1024


def _decimal(token: str, allow_negative: bool) -> int | None:
    body = token[1:] if allow_negative and token.startswith("-") else token
    if not body or not body.isdigit() or not body.isascii() or (body[0] == "0" and body != "0"):
        return None
    value = int(body)
    if token.startswith("-"):
        return -value if value else None
    return value


def lrat_verdict(obligation: str, cnf: str, certificate: str) -> tuple[Any, ...]:
    """Check an LRAT certificate against an obligation's canonical CNF."""

    if cnf != cnf_text(obligation):
        return ("reject", "cnf_mismatch", 0)
    if len(certificate.encode("utf-8")) > MAX_CERTIFICATE:
        return ("reject", "oversized", 0)
    variables, clauses = blast(obligation)
    active: dict[int, list[int]] = {index + 1: clause for index, clause in enumerate(clauses)}
    last_id = len(clauses)
    if certificate and not certificate.endswith("\n"):
        lines = certificate.split("\n")
    else:
        lines = certificate.split("\n")[:-1] if certificate else []
    finished = False
    for number, line in enumerate(lines, start=1):
        if finished:
            return ("reject", "trailing", number)
        if number == len(lines) and certificate and not certificate.endswith("\n"):
            return ("reject", "parse", number)
        tokens = line.split(" ")
        if len(tokens) < 2 or any(token == "" for token in tokens):
            return ("reject", "parse", number)
        ident = _decimal(tokens[0], False)
        if ident is None or ident == 0:
            return ("reject", "parse", number)
        if tokens[1] == "d":
            if tokens[-1] != "0" or len(tokens) < 3:
                return ("reject", "parse", number)
            targets = [_decimal(token, False) for token in tokens[2:-1]]
            if any(target is None or target == 0 for target in targets):
                return ("reject", "parse", number)
            for target in targets:
                if target not in active:
                    return ("reject", "unknown_deletion", number)
                del active[target]
            continue
        values = [_decimal(token, True) for token in tokens[1:]]
        if any(value is None for value in values):
            return ("reject", "parse", number)
        try:
            split = values.index(0)
        except ValueError:
            return ("reject", "parse", number)
        lemma, hints = values[:split], values[split + 1 :]
        if not hints or hints[-1] != 0 or 0 in hints[:-1]:
            return ("reject", "parse", number)
        hints = hints[:-1]
        if ident <= last_id:
            return ("reject", "id_order", number)
        if any(abs(literal) > variables for literal in lemma):
            return ("reject", "var_range", number)
        if len(set(lemma)) != len(lemma) or any(-literal in lemma for literal in lemma):
            return ("reject", "lemma_form", number)
        if any(hint < 0 for hint in hints):
            return ("reject", "rat_unsupported", number)
        assignment = {-literal for literal in lemma}
        conflict = False
        for position, hint in enumerate(hints):
            clause = active.get(hint)
            if clause is None:
                return ("reject", "unknown_hint", number)
            if any(literal in assignment for literal in clause):
                return ("reject", "rup", number)
            open_literals = [literal for literal in clause if -literal not in assignment]
            if not open_literals:
                if position != len(hints) - 1:
                    return ("reject", "rup", number)
                conflict = True
                break
            if len(open_literals) != 1:
                return ("reject", "rup", number)
            assignment.add(open_literals[0])
        if not conflict:
            return ("reject", "rup", number)
        active[ident] = lemma
        last_id = ident
        if not lemma:
            finished = True
    if not finished:
        return ("reject", "no_empty_clause", 0)
    return ("accept",)


VARIANTS = [
    ("V-00", "unchanged"),
    ("V-01", "drop the final line"),
    ("V-02", "cut at half the byte length"),
    ("V-03", "negate the first lemma literal of the first addition"),
    ("V-04", "drop the first hint of the first addition"),
    ("V-05", "swap the first two additions"),
    ("V-06", "pair with the other obligation's CNF"),
    ("V-07", "claim the other obligation"),
    ("V-08", "pad past 4 MiB with deletion lines"),
    ("V-09", "empty certificate"),
    ("V-10", "make the first addition's last hint a clause that was never added"),
    ("V-11", "append a RAT (negative) hint to the first addition"),
]


def certificate_variants(certificate: str, obligation: str = "B-C01") -> list[dict[str, Any]]:
    """The shared deterministic mutations of a certificate, with their reference verdicts."""

    variants = []
    for ident, change in VARIANTS:
        claimed, cnf, text = mutate_certificate(certificate, ident, obligation)
        data = text.encode("ascii")
        variants.append(
            {
                "id": ident,
                "change": change,
                "claimed": claimed,
                "cnf_sha256": sha256(cnf.encode("ascii")),
                "certificate_sha256": sha256(data),
                "certificate_bytes": len(data),
                "verdict": list(lrat_verdict(claimed, cnf, text)),
            }
        )
    return variants


def mutate_certificate(certificate: str, variant: str, obligation: str = "B-C01") -> tuple[str, str, str]:
    """Rebuild one variant's (claimed obligation, CNF, certificate) from its id."""

    lines = certificate.split("\n")[:-1]
    additions = [index for index, line in enumerate(lines) if line.split(" ")[1] != "d"]
    first, second = additions[0], additions[1]
    other = "B-C02" if obligation == "B-C01" else "B-C01"

    def join(rows: list[str]) -> str:
        return "".join(row + "\n" for row in rows)

    def edited(change: Callable[[list[str]], list[str]]) -> str:
        rows = list(lines)
        rows[first] = " ".join(change(rows[first].split(" ")))
        return join(rows)

    own = cnf_text(obligation)
    if variant == "V-00":
        return obligation, own, certificate
    if variant == "V-01":
        return obligation, own, join(lines[:-1])
    if variant == "V-02":
        return obligation, own, certificate[: len(certificate) // 2]
    if variant == "V-03":
        return obligation, own, edited(lambda t: [t[0], t[1][1:] if t[1].startswith("-") else "-" + t[1], *t[2:]])
    if variant == "V-04":
        return obligation, own, edited(lambda t: t[: t.index("0", 1) + 1] + t[t.index("0", 1) + 2 :])
    if variant == "V-05":
        rows = list(lines)
        rows[first], rows[second] = rows[second], rows[first]
        return obligation, own, join(rows)
    if variant == "V-06":
        return obligation, cnf_text(other), certificate
    if variant == "V-07":
        return other, own, certificate
    if variant == "V-08":
        return obligation, own, certificate + "1 d 0\n" * (MAX_CERTIFICATE // 6 + 1)
    if variant == "V-09":
        return obligation, own, ""
    if variant == "V-10":
        return obligation, own, edited(lambda t: [*t[:-2], str(int(t[0]) + 1000000), t[-1]])
    if variant == "V-11":
        return obligation, own, edited(lambda t: [*t[:-1], "-1", "0"])
    raise ValueError(variant)


# ---------------------------------------------------------------------------
# The shared-input documents.
# ---------------------------------------------------------------------------

TAXONOMY = [
    ("parse_failure", "Input bytes or source text cannot be read as the expected syntax or artifact format."),
    ("type_failure", "A term, statement or declaration is ill typed, including an unresolved or ambiguous inference."),
    ("non_total", "A definition is refused because its totality or termination cannot be established."),
    ("proof_failure", "A proof, script or term does not establish its stated theorem."),
    ("disproved_obligation", "A closed decidable obligation evaluates to false, or a checked counterexample exists."),
    ("unknown", "Automation neither proves nor refutes an obligation within its limits."),
    ("timeout", "A wall-clock ceiling ends the step before it completes."),
    ("resource_exhaustion", "A memory, recursion, output or size ceiling ends the step, or the tool reports exhausting one."),
    ("unsupported_feature", "The candidate cannot express or run a required construct in the tested envelope."),
    ("untrusted_solver_step", "A solver result is used without a checked certificate."),
    ("failed_certificate", "A certificate is rejected by its checker."),
    ("unmet_target_assumption", "A host or target row the case requires is missing or unsupported."),
    ("undeclared_trust", "An axiom, plugin, native evaluator or extension outside the declared trust inventory is used."),
]


def ds01() -> dict[str, Any]:
    return {
        "case": "DS-01",
        "schema_version": SCHEMA,
        "title": "Define and check the proposed Core fragment",
        "signature": CORE_SIGNATURE,
        "modules": CORE_MODULES,
        "theorems": core_theorems(),
        "observations": core_observations(),
        "negatives": core_negatives(),
        "rules": [
            "Every T-, C- and F- symbol and the M-01 construct is mapped to exactly one candidate declaration; statements and observations are rendered from that mapping and must check as rendered.",
            "Observations are proved by the candidate's declared computation method, whose evaluator is listed in its trust inventory.",
            "Terms use explicit type and width arguments everywhere a symbol's type lists them; the hole term marks an argument left for inference.",
            "The operator add on naturals inside statements is plain addition of naturals.",
        ],
    }


def ds02() -> dict[str, Any]:
    return {
        "case": "DS-02",
        "schema_version": SCHEMA,
        "title": "Mechanize progress, preservation and leakage",
        "language": "Sieve",
        "signature": SIEVE_SIGNATURE,
        "fixtures": {
            "gamma_plus": env_term(GAMMA),
            "p_plus": sieve_term(P_PLUS),
            "p_minus": sieve_term(P_MINUS),
            "sigma_a": state_term(SIGMA_A),
            "sigma_b": state_term(SIGMA_B),
            "variables": ["0 key: secret word", "1 nonce: public word", "2 acc: secret word", "3 ok: public bool"],
            "arrays": ["0 table: 4 public words", "1 state: 4 secret words"],
        },
        "theorems": sieve_statements(),
        "observations": sieve_observations(),
        "mutations": sieve_negatives(),
        "leakage_model": {
            "observations": "Each step emits the addresses it reads and writes, the outcome of every branch, every out-of-bounds address, and every released value, in evaluation order.",
            "public_equivalence": "low_eq: two states agree on every declared public variable and every declared public array.",
            "declassification": "declassify x e is the only release: it emits release v and assigns v to a public variable x. D2-TH04 allows two runs to differ first at a release and nowhere earlier.",
            "nonclaims": "The model is suite-only. It does not model timing, caches, speculation or the hardware, and it neither ratifies nor drafts D-012.",
        },
    }


def ds03() -> dict[str, Any]:
    return {
        "case": "DS-03",
        "schema_version": SCHEMA,
        "title": "Validate canonical serialization",
        "format": "OCR1",
        "limits": {"input_bytes": MAX_INPUT, "records": MAX_RECORDS, "name_bytes": MAX_NAME, "references": MAX_REFS},
        "signature": RECORD_SIGNATURE,
        "theorems": record_statements(),
        "observations": record_observations(),
        "rules": [
            "The candidate's own heap, parser state or serialization never stands in for these bytes.",
            "Standalone verdicts are one line: accept followed by each record rendered def:NAME:DIGEST, thm:NAME:FINGERPRINT:REFS or claim:NAME:REF:LEVEL in hex and decimal, or reject CODE PATH.",
        ],
    }


def ds04(golden: str) -> dict[str, Any]:
    obligations = obligation_terms()
    W32 = WORD(nat(WIDTH))
    rows = []
    for ident, terms in obligations.items():
        statement = forall([("x", W32), ("y", W32)], eq(bv_term(terms["lhs"]), bv_term(terms["rhs"])))
        text = cnf_text(ident)
        variables, clauses = blast(ident)
        rows.append({"id": ident, "name": terms["name"], "statement": statement, "holds": terms["holds"], "cnf_sha256": sha256(text.encode("ascii")), "cnf_bytes": len(text), "cnf_variables": variables, "cnf_clauses": len(clauses)})
    counterexample = {"x": 1, "y": 1}
    assert bv_eval(obligations["B-C02"]["lhs"], 1, 1) != bv_eval(obligations["B-C02"]["rhs"], 1, 1)
    OB = sym("B-T01")
    theorems = [
        {"id": "D4-TH01", "name": "the obligation holds, through a checked certificate", "statement": rows[0]["statement"]},
        {"id": "D4-TH02", "name": "meaning of B-C01", "statement": iff(app(sym("B-F03"), sym("B-C01")), rows[0]["statement"])},
        {"id": "D4-TH03", "name": "meaning of B-C02", "statement": iff(app(sym("B-F03"), sym("B-C02")), rows[1]["statement"])},
        {"id": "D4-TH04", "name": "an accepted certificate proves its obligation", "statement": forall([("o", OB), ("f", STRING), ("p", STRING)], imp(eq(app(sym("B-F02"), var("o"), var("f"), var("p")), sym("B-C03")), app(sym("B-F03"), var("o"))))},
        {"id": "D4-TH05", "name": "the satisfiable variant is refuted", "statement": lnot(app(sym("B-F03"), sym("B-C02")))},
    ]
    return {
        "case": "DS-04",
        "schema_version": SCHEMA,
        "title": "Replay an LRAT-backed bit-vector proof",
        "width": WIDTH,
        "signature": [
            {"id": "B-T01", "name": "Obligation", "kind": "type", "constructors": ["B-C01 carry_save", "B-C02 carry_save_unshifted"]},
            {"id": "B-T02", "name": "Verdict", "kind": "type", "constructors": ["B-C03 accept", "B-C04 reject (code : String) (line : Nat)"]},
            {"id": "B-F01", "name": "cnf_text", "kind": "function", "type": "Obligation -> String"},
            {"id": "B-F02", "name": "lrat_verdict", "kind": "function", "type": "Obligation -> String -> String -> Verdict"},
            {"id": "B-F03", "name": "holds", "kind": "predicate", "type": "Obligation -> Prop"},
        ],
        "obligations": rows,
        "theorems": theorems,
        "counterexample": counterexample,
        "golden": {
            "cnf_path": "ds04-carry-save.cnf",
            "cnf_sha256": sha256(cnf_text("B-C01").encode("ascii")),
            "certificate_path": "ds04-carry-save-golden.lrat",
            "certificate_sha256": sha256(golden.encode("ascii")),
            "producer": "drat-trim -L from a CaDiCaL 3.0.1 DRAT proof; see protocol/toolchains.json",
            "variants": certificate_variants(golden),
        },
        "solver": {
            "tool": "cadical",
            "argv": ["cadical", "--lrat", "--no-binary", "--seed=0", "-q", "{cnf}", "{certificate}"],
            "exit_codes": {"10": "satisfiable", "20": "unsatisfiable", "0": "unknown"},
            "unknown_argv": ["cadical", "-c", "0", "--lrat", "--no-binary", "--seed=0", "-q", "{cnf}", "{certificate}"],
            "trust": "untrusted: its search, heuristics and output are outside every logical trusted base.",
        },
        "run_time_cases": [
            {"id": "D4-R01", "name": "fresh certificate", "rule": "The pinned solver runs on the candidate-emitted CNF; the fresh certificate must be accepted in the prover and by the standalone checker, and every V-01 to V-11 mutation of it rejected exactly as the reference rejects it."},
            {"id": "D4-R02", "name": "satisfiable input", "rule": "The solver runs on B-C02's CNF and reports satisfiable; the claim stays non-successful as disproved_obligation, and the reported x and y are checked in the prover to falsify the identity."},
            {"id": "D4-R03", "name": "unknown", "rule": "The solver runs with a conflict limit of 0 and reports unknown; the claim stays non-successful as unknown."},
            {"id": "D4-R04", "name": "timeout", "rule": "The solver runs under a wall ceiling of 0 seconds; the claim stays non-successful as timeout."},
            {"id": "D4-R05", "name": "missing output", "rule": "The certificate file is absent after a solver run; the claim stays non-successful as failed_certificate."},
        ],
        "rules": [
            "Variable 1 is constant false and clause 1 is its negative unit. Bit i (least significant first) of x is variable 2 + i and of y is variable 34 + i.",
            "Gates are numbered from 66 in creation order and emit their Tseitin clauses as they are created: AND g=a.b gives (-g a) (-g b) (g -a -b); XOR gives (-g a b) (-g -a -b) (g -a b) (g a -b); OR gives (g -a) (g -b) (-g a b).",
            "The left side is blasted before the right side, and each operation's left operand before its right. xor and and make one gate per bit; shl by k makes no gates and shifts in literal 1; add is a ripple-carry adder whose carry starts as literal 1 and makes, per bit, t=XOR(a,b), s=XOR(t,c), u=AND(a,b), v=AND(c,t), c=OR(u,v) in that order.",
            "The miter makes d_i=XOR(lhs_i, rhs_i) for every bit, then o=d_0 and o=OR(o, d_i) for i from 1, and ends with the unit clause (o).",
            "The CNF text is p cnf V C then one line per clause, literals in decimal separated by single spaces, each line ending in space, 0 and a line feed.",
            "An LRAT certificate is ASCII lines ending in line feeds. An addition is ID LITERALS 0 HINTS 0 with IDs strictly increasing and above the clause count; a deletion is ID d IDS 0 and its leading ID is otherwise ignored. Numbers are canonical decimal.",
            "Each addition is checked by reverse unit propagation from the negation of its literals: every hint but the last must be unit and assigns its open literal, the last must be falsified, and a satisfied hint fails. Negative hints are refused as rat_unsupported.",
            "The certificate is accepted when an addition derives the empty clause on its final line. Rejections carry the first failing line number, or 0 for cnf_mismatch, oversized and no_empty_clause; certificates above 4 MiB are refused before parsing.",
        ],
    }


def ds05() -> dict[str, Any]:
    return {
        "case": "DS-05",
        "schema_version": SCHEMA,
        "title": "Extract and distribute the authoritative checker case",
        "interface": [
            {"argv": ["CHECKER", "records", "PATH"], "output": "one DS-03 standalone verdict line", "exit": 0},
            {"argv": ["CHECKER", "cnf", "OBLIGATION"], "output": "the obligation's canonical CNF text", "exit": 0},
            {"argv": ["CHECKER", "lrat", "OBLIGATION", "CNF_PATH", "CERTIFICATE_PATH"], "output": "accept, or reject CODE LINE", "exit": 0},
            {"argv": ["CHECKER"], "output": "usage on standard error", "exit": 2},
        ],
        "corpus": ["every DS-03 fixture", "the golden certificate and its V-00 to V-11 variants", "the run's fresh certificate and its V-00 to V-11 variants"],
        "agreement": "Each standalone verdict equals the reference verdict and the candidate's in-prover verdict for the same input, on every corpus item and every supported host row.",
        "hosts": "protocol/suite-overlay.json host_matrix",
        "distribution": [
            "The checker runs with no prover, IDE, registry, network, tactic, plugin or solver at check time.",
            "A dynamically linked checker lists every shared object it loads; its closure is part of the bootstrap manifest.",
            "The source-to-binary mapping records the extraction or compilation command, its inputs and the output digest.",
        ],
        "negatives": [
            {"id": "D5-N01", "name": "unsupported host", "expected": ["unmet_target_assumption"]},
            {"id": "D5-N02", "name": "missing runtime", "rule": "The checker's declared runtime closure is withheld.", "expected": ["unmet_target_assumption", "parse_failure"]},
            {"id": "D5-N03", "name": "altered checker byte", "rule": "One byte of the checker changes; the bootstrap manifest digest check refuses to run it.", "expected": ["failed_certificate"]},
            {"id": "D5-N04", "name": "incompatible format", "rule": "A DS-03 input with version byte 2 (fixture R-F22).", "expected_verdict": "reject unknown_version header"},
            {"id": "D5-N05", "name": "malformed proof", "rule": "The V-02 variant of each certificate.", "expected_verdict": "the reference verdict for V-02"},
            {"id": "D5-N06", "name": "absent dependency", "rule": "The build runs with one declared dependency removed.", "expected": ["parse_failure", "unsupported_feature"]},
            {"id": "D5-N07", "name": "bootstrap mismatch", "rule": "The toolchain archive digest differs from the manifest.", "expected": ["failed_certificate"]},
        ],
    }


def ds06() -> dict[str, Any]:
    return {
        "case": "DS-06",
        "schema_version": SCHEMA,
        "title": "Measure clean bootstrap, replay, diagnostics and dependency surface",
        "profiles": {
            "cold_bootstrap": {"runs": 5, "rule": "From the content-addressed toolchain archive and the candidate sources into an empty workspace and cache: unpack, check every proof, build the standalone checker, and write the output manifest."},
            "deterministic_replay": {"runs": 3, "modes": ["serial", "declared_parallel"], "rule": "Clean builds whose deterministic outputs (observation records, verdicts, and the output manifest of declared deterministic artifacts) must match byte for byte."},
            "timed_replay": {"warmup": 1, "runs": 30, "rule": "Per case, the independent kernel re-check of the compiled proofs and the standalone checker over the case's corpus, paired and interleaved between candidates."},
        },
        "statistics": ["every raw observation", "median", "median absolute deviation", "95th percentile", "bootstrap 95% confidence interval of the median (10000 resamples, seed fixed by the epoch)"],
        "faults": [
            {"id": "D6-F01", "name": "empty cache", "expected": "builds from nothing"},
            {"id": "D6-F02", "name": "poisoned cache", "rule": "One compiled dependency in the cache is replaced by a stale build of a different source before the build.", "expected": "rebuilds or refuses; never reuses the stale artifact as success"},
            {"id": "D6-F03", "name": "unavailable dependency", "rule": "The standard library the candidate imports is removed from the toolchain.", "expected": "bounded, attributable failure"},
            {"id": "D6-F04", "name": "read-only home", "expected": "succeeds or fails attributably without writing outside the workspace"},
            {"id": "D6-F05", "name": "path change", "rule": "The workspace moves to another absolute path.", "expected": "identical deterministic outputs"},
            {"id": "D6-F06", "name": "locale and time variation", "rule": "LC_ALL, TZ and SOURCE_DATE_EPOCH change.", "expected": "identical deterministic outputs"},
            {"id": "D6-F07", "name": "one-core limit", "expected": "completes within ceilings or fails attributably"},
            {"id": "D6-F08", "name": "timeout", "rule": "The wall ceiling is 1 second.", "expected": "timeout, never success"},
            {"id": "D6-F09", "name": "forced process failure", "rule": "The build is killed after its first compiled artifact appears, then rerun.", "expected": "the rerun does not reuse partial output as success"},
        ],
        "workspaces": {"count": 2, "rule": "Two separately provisioned workspaces with distinct checkouts, toolchain unpacks, caches and output roots recreate the deterministic manifests. Matching results are reproducibility level 2, same-owner evidence, never independent reproduction."},
    }


def ds07() -> dict[str, Any]:
    return {
        "case": "DS-07",
        "schema_version": SCHEMA,
        "title": "Exercise solo auditability and maintenance",
        "performer": "the project owner; a contributor rehearsal is recorded separately and never counts toward M-16",
        "tasks": [
            {"id": "D7-T01", "kind": "audit", "rule": "From a clean workspace and the published packet only: locate D2-TH04, list every assumption and trusted component it rests on, name one failing mutation case and its diagnostic, and rebuild the standalone checker and match its digest."},
            {"id": "D7-T02", "kind": "maintenance", "rule": "Add the operator sub (wrapping subtraction) to Sieve expressions with the same typing rule as add, repair every DS-02 proof, and add one observation for it."},
        ],
        "seeded_faults": [
            {"id": "D7-S01", "kind": "hidden assumption", "rule": "An assumption enters one proof through a route the candidate's own tools can surface."},
            {"id": "D7-S02", "kind": "stale artifact", "rule": "A compiled artifact from an older source is placed where the build may reuse it."},
            {"id": "D7-S03", "kind": "ambiguous diagnostic", "rule": "One negative case's diagnostic is made to name the wrong shared category."},
            {"id": "D7-S04", "kind": "dependency substitution", "rule": "One library file in the toolchain is replaced by a same-named file with different content."},
        ],
        "sealing": "The per-candidate fault seeds are written after both candidates exist and before either task starts. Only their SHA-256 commitments are published until the owner finishes both tasks; the sealed files are then revealed and checked against the commitments.",
        "order": "Candidate order is counterbalanced: audit Rocq then Lean 4, maintenance Lean 4 then Rocq.",
        "controls": {"time_ceiling_minutes": 120, "permitted": ["the published packet", "each candidate's own documentation and tools", "a terminal and editor"], "recorded": ["every command", "every consulted source", "every tool suggestion", "every hint", "prior familiarity", "role overlap"]},
    }


PROTOCOL_SCHEMA = "d006-v0.3-protocol-1"
BASE = {
    "suite": {"path": "docs/PROOF_FOUNDATION_DECISION_SUITE.md", "suite_version": "d006-v0.2-draft", "sha256": "6b1aa32784dd31d40bdaca4c6f3b62b8721a909ab3415051aa5a8e7994f0254b"},
    "packet": {"path": "research/decisions/D-006/d006-v0.2-draft-packet.json", "canonical_sha256": "b56ad768c4584bdd00da4d4e85af642757b877dd5dc5ae438560ba4a486d9d21"},
    "index": {"path": "research/decisions/D-006/d006-v0.2-case-input-index.json", "sha256": "1aec6a731bef0620c8500120ec8385d584f99a528b4a03c014e8516c55cc8136"},
}


def toolchains() -> dict[str, Any]:
    return {
        "schema_version": PROTOCOL_SCHEMA,
        "suite_version": SUITE_VERSION,
        "capture": {
            "date": "2026-09-29",
            "host": "x86_64-linux-gnu, Ubuntu 24.04, 4 vCPU, 15 GiB",
            "network": "Capture used the network. The opam registry and the Lean release and package servers were unreachable from the capture host, so Rocq was built from its tagged sources and Lean 4 came from its GitHub release archive. Measured steps deny the network.",
            "binding": "The epoch binds each tool's installed-tree digest as captured at run time; the values here identify the acquisitions.",
        },
        "tools": [
            {
                "id": "TC-01",
                "candidate": "C-02",
                "name": "Lean 4",
                "version": "4.34.1",
                "commit": "5045d0056413266e57c625dcd7c365b10e377c52",
                "acquisition": {"kind": "release_archive", "url": "https://github.com/leanprover/lean4/releases/download/v4.34.1/lean-4.34.1-linux.tar.zst", "sha256": "47bf4bbd78f70c2e9670598ab7124d92b6efb7330ff33e5fbb4030f6fd72e4e4", "bytes": 580432872},
                "provides": ["lean", "lake", "leanchecker", "leanc with its bundled clang and runtime", "the Init, Std and Lean libraries", "a bundled cadical 2.1.2 the suite does not use"],
                "terms": "Apache-2.0; bundled components under the archive's LICENSES directory",
            },
            {
                "id": "TC-02",
                "candidate": "C-01",
                "name": "Rocq Prover",
                "version": "9.2.0",
                "acquisition": {"kind": "source_build", "repository": "https://github.com/rocq-prover/rocq", "tag": "V9.2.0", "commit": "adfbf1855c348766beb4b790dcc8ebc02f908f63"},
                "build": [["./configure", "-prefix", "PREFIX"], ["make", "dunestrap"], ["dune", "build", "-p", "rocq-runtime,coq-core,rocq-core"], ["dune", "install", "--prefix", "PREFIX", "rocq-runtime", "coq-core", "rocq-core"]],
                "build_inputs": ["TC-06", "TC-07"],
                "configuration": {"bytecode_vm": True, "native_compiler": False},
                "provides": ["rocq (compile, top-level and checker front ends)", "rocqchk", "the Corelib and Ltac2", "extraction to OCaml"],
                "terms": "LGPL-2.1-only",
            },
            {
                "id": "TC-03",
                "candidate": "C-01",
                "name": "Rocq Stdlib",
                "version": "9.2.0",
                "acquisition": {"kind": "source_build", "repository": "https://github.com/rocq-prover/stdlib", "tag": "V9.2.0", "commit": "8dd155bc10529814202f8f4c643e5ae6c2c88fa6"},
                "build": [["make", "-j4"], ["make", "install"]],
                "build_inputs": ["TC-02"],
                "terms": "LGPL-2.1-only",
            },
            {
                "id": "TC-04",
                "candidate": "shared",
                "name": "CaDiCaL",
                "version": "3.0.1",
                "role": "the pinned untrusted DS-04 solver; outside every logical trusted base",
                "acquisition": {"kind": "source_build", "repository": "https://github.com/arminbiere/cadical", "tag": "rel-3.0.1", "commit": "c60730422e758ef1cebe7aeddf2dda31c996bf04"},
                "build": [["./configure"], ["make"]],
                "binary_sha256": "18ec528630e11254fa61b5b7b4c40312f127d1fd1ce14ad1cc224289ab71551f",
                "terms": "MIT",
            },
            {
                "id": "TC-05",
                "candidate": "shared",
                "name": "drat-trim",
                "role": "produced the golden DS-04 certificate at capture; never run during an epoch",
                "acquisition": {"kind": "source_build", "repository": "https://github.com/marijnheule/drat-trim", "commit": "2e3b2dc0ecf938addbd779d42877b6ed69d9a985"},
                "build": [["make"]],
                "binary_sha256": "92f0aa9575ed519d66a99b8b1b3dde6ece4618ae4c202a3a4b200265dda0aa7a",
                "terms": "MIT",
            },
            {"id": "TC-06", "candidate": "C-01", "name": "OCaml", "version": "4.14.1", "acquisition": {"kind": "distribution_package", "packages": {"ocaml": "4.14.1-1ubuntu1", "ocaml-base": "4.14.1-1ubuntu1", "ocaml-findlib": "1.9.6-1build4", "libzarith-ocaml-dev": "1.13-2build4"}}, "role": "builds Rocq and compiles Rocq's extracted checker", "terms": "LGPL-2.1 with the OCaml linking exception; Zarith LGPL-2.0 with exception"},
            {"id": "TC-07", "candidate": "C-01", "name": "dune", "version": "3.14.0", "acquisition": {"kind": "distribution_package", "packages": {"ocaml-dune": "3.14.0-1"}}, "role": "builds Rocq", "terms": "MIT"},
            {"id": "TC-08", "candidate": "shared", "name": "qemu-user-static", "version": "8.2.2", "acquisition": {"kind": "distribution_package", "packages": {"qemu-user-static": "1:8.2.2+ds-0ubuntu1.18"}}, "role": "runs the H-02 AArch64 row by emulation", "terms": "GPL-2.0"},
            {"id": "TC-09", "candidate": "shared", "name": "GCC for AArch64 Linux", "version": "13.3.0", "acquisition": {"kind": "distribution_package", "packages": {"gcc-aarch64-linux-gnu": "4:13.2.0-7ubuntu1"}}, "role": "links H-02 standalone checkers", "terms": "GPL-3.0 with the GCC runtime library exception"},
        ],
        "golden_certificate": {
            "argv": [["cadical", "--no-binary", "--seed=0", "-q", "ds04-carry-save.cnf", "carry.drat"], ["drat-trim", "ds04-carry-save.cnf", "carry.drat", "-L", "ds04-carry-save-golden.lrat"]],
            "tools": ["TC-04", "TC-05"],
            "drat_sha256": "9f701a8d15015632ad5f7e1d45bde74353abf091b58916f959fa8b95bc17ed8e",
            "note": "Both runs of the pair produced identical bytes. The golden certificate comes from a different producer (drat-trim) than the fresh run-time certificate (CaDiCaL's own LRAT), so both checkers meet two certificate styles.",
        },
        "dispositions": {
            "d018": "No D-018 owner admission exists for any tool here. The tools are used only inside the research laboratory, outside the product lineage, under a recorded contributor disposition; owner admission remains an open prerequisite for selection.",
            "product": "No tool is added to the compiler, its build or its dependency inventory.",
        },
    }


def overlay(manifest_sha256: str) -> dict[str, Any]:
    ceilings = {"wall_seconds": 1800, "memory_bytes": 8 * 1024**3, "temp_bytes": 8 * 1024**3, "output_bytes": 64 * 1024**2, "pids": 4096}
    return {
        "schema_version": PROTOCOL_SCHEMA,
        "suite_version": SUITE_VERSION,
        "status": "prerequisites_draft",
        "status_meaning": "Shared inputs, contracts and toolchains are written. No candidate is built, no epoch is frozen and no execution evidence exists (0/14).",
        "base": BASE,
        "shared_inputs": {"manifest": "shared-inputs/manifest.json", "input_manifest_sha256": manifest_sha256},
        "candidates": [
            {"id": "C-01", "name": "Rocq", "toolchain": ["TC-02", "TC-03", "TC-06", "TC-07"]},
            {"id": "C-02", "name": "Lean 4", "toolchain": ["TC-01"]},
        ],
        "closes": [
            {"gap": "foundation-neutral shared inputs and all seven executable case inventories absent", "by": "shared-inputs/: statements, fixtures, expected observations and negative cases for DS-01 to DS-07, with the reference in tools/d006_shared.py"},
            {"gap": "input-manifest digest unfrozen", "by": "shared-inputs/manifest.json; an epoch freezes it by binding this digest"},
            {"gap": "execution resource, timeout, host, environment, cache, and network contract unassigned", "by": "execution and host_matrix below"},
            {"gap": "candidate tool versions, dependency graphs, acquisitions, and D-018 admissions absent", "by": "protocol/toolchains.json for versions, build inputs and acquisitions; D-018 owner admissions stay open"},
            {"gap": "physical execution order, correction window, and materiality bands unassigned", "by": "execution_order, correction_window and materiality_bands below"},
        ],
        "remaining": [
            "D-004 acceptance absent",
            "D-005 acceptance absent",
            "D-018 owner admission of every candidate tool absent",
            "candidate adapters, runner, observer, and isolation backend absent (next tranche)",
            "versioned D-006 result and same-owner-replay schema absent (next tranche, with the runner that writes it)",
            "owner protocol review absent",
        ],
        "amendments": [
            {"id": "AM-01", "rule": "The Gate 0 evaluation-host matrix is host_matrix below. H-01 and H-02 are required; H-02 is an emulated row, and its timings are never compared with H-01's or reported as performance."},
            {"id": "AM-02", "rule": "The contributor builds both candidates and runs DS-01 to DS-06 on the owner's standing direction. DS-07 and R-01 to R-09 stay owner-performed; a contributor rehearsal is recorded separately, labeled as such, and never satisfies M-16 or an owner scope."},
            {"id": "AM-03", "rule": "Candidate tools run in the laboratory under a recorded contributor disposition. That disposition is not a D-018 admission, and selection stays blocked until the owner records admissions."},
            {"id": "AM-04", "rule": "An epoch binds the shared-input manifest, this overlay, the toolchain record, the runner and the observer by digest. Candidate artifacts are bound per run, so a correction inside the window adds a run to the same epoch while any shared change opens a new epoch for both candidates."},
        ],
        "host_matrix": [
            {"id": "H-01", "triple": "x86_64-unknown-linux-gnu", "kind": "native", "required": True, "cases": ["DS-01", "DS-02", "DS-03", "DS-04", "DS-05", "DS-06"]},
            {"id": "H-02", "triple": "aarch64-unknown-linux-gnu", "kind": "emulated by TC-08 on H-01", "required": True, "cases": ["DS-05"]},
            {"id": "H-03", "triple": "aarch64-apple-darwin and x86_64-apple-darwin", "kind": "not evaluated: no such host in the laboratory", "required": False, "cases": []},
            {"id": "H-04", "triple": "x86_64-pc-windows-msvc", "kind": "not evaluated: no such host in the laboratory", "required": False, "cases": []},
        ],
        "host_matrix_note": "This matrix is the tested envelope for this suite only and does not accept D-011. A different envelope reruns DS-05.",
        "execution": {
            "ceilings": {"measured_step": ceilings, "negative_case": {"wall_seconds": 120, "memory_bytes": 4 * 1024**3, "temp_bytes": 1024**3, "output_bytes": 1024**2, "pids": 1024}, "timed_replay_step": dict(ceilings, wall_seconds=600)},
            "timeouts": "Each step runs in its own cgroup. At the wall ceiling the whole cgroup is killed and the step is timeout. A memory kill, a temp or output overrun, or the pid limit is resource_exhaustion. Neither is ever success.",
            "cpu": {"serial": "one CPU and the candidate's one-job build setting", "declared_parallel": "four CPUs and the candidate's declared parallel build setting"},
            "network": "Every measured step runs in a new network namespace with no interfaces.",
            "filesystem": "The toolchain tree and the candidate sources are read-only. Writes go only to the step's output root and temporary directory.",
            "environment": {"allowlist": {"HOME": "WORKSPACE/home", "LANG": "C", "LC_ALL": "C", "PATH": "TOOLCHAIN/bin:/usr/bin:/bin", "SOURCE_DATE_EPOCH": "0", "TMPDIR": "WORKSPACE/tmp", "TZ": "UTC"}, "candidate_variables": "A candidate adapter may add variables its toolchain needs; each is listed, and its value derives from the workspace or toolchain paths, never from the host environment."},
            "cache": "Cold runs start from an empty candidate cache and output root. A warm profile is declared as warm and measured separately.",
            "observer": "The runner measures wall time with a monotonic clock, CPU time and peak memory from the step's cgroup, and temp and output bytes after the step. Its identity is its raw SHA-256.",
        },
        "execution_order": {
            "implementation": "Candidate construction alternates which candidate goes first: DS-01 Rocq, DS-02 Lean 4, DS-03 Rocq, DS-04 Lean 4, DS-05 Rocq. Every statement mapping is complete before any timed run or comparative summary is viewed.",
            "cold_bootstrap": "Runs 1 to 5 alternate: odd runs C-01 then C-02, even runs C-02 then C-01.",
            "deterministic_replay": "Runs 1 to 3 in each mode alternate the same way; serial before declared_parallel.",
            "timed_replay": "After one unmeasured warmup per candidate, pairs 1 to 30 alternate the same way within each case; cases run DS-01 to DS-06.",
            "workspaces": "Workspace W1 completes before W2 is provisioned.",
        },
        "correction_window": {
            "rounds_per_candidate": 1,
            "rule": "After the epoch's first complete run of both candidates, each candidate may receive one round of corrections to its own artifacts for defects that run exposed. The failed run stays archived and is reported beside the corrected run. The window closes before any comparative summary of the corrected run is viewed. A change to a shared input, the runner, the observer, the toolchains or this overlay opens a new epoch that reruns both candidates.",
        },
        "materiality_bands": [
            {"metrics": ["M-07", "M-08", "M-09"], "measure": "ratio of C-01's median to C-02's, with a paired bootstrap 95% confidence interval", "rocq_better": "the interval lies below 1 and the ratio is below 0.9", "lean_better": "the interval lies above 1 and the ratio is above 1/0.9", "practically_equivalent": "the interval lies within [0.9, 1/0.9]", "inconclusive": "otherwise"},
            {"metrics": ["M-10", "M-12"], "measure": "exact bytes (deterministic)", "rocq_better": "C-01 at most 0.9 times C-02", "lean_better": "C-02 at most 0.9 times C-01", "practically_equivalent": "within 10%", "inconclusive": "a value is missing"},
            {"metrics": ["M-18"], "measure": "elapsed owner time and files changed for the same seeded change (one same-owner observation each)", "rocq_better": "C-01 at most 0.75 times C-02 on elapsed time", "lean_better": "C-02 at most 0.75 times C-01 on elapsed time", "practically_equivalent": "within 25%", "inconclusive": "a value is missing or a task failed"},
            {"metrics": ["M-11"], "measure": "reported by role only", "label": "none: M-11 plans audit effort and carries no better or worse label"},
        ],
        "nonclaims": [
            "no epoch frozen and no candidate built, executed or measured",
            "no D-004, D-005 or D-011 acceptance inferred",
            "no D-018 admission recorded",
            "no proof foundation selected, preferred or recommended",
            "no independent review; M-17 stays unavailable",
            "no proof-bearing product work authorized",
        ],
    }


GENERATED = {
    "taxonomy.json": lambda golden: {"schema_version": SCHEMA, "categories": [{"id": name, "meaning": meaning} for name, meaning in TAXONOMY]},
    "ds01-core-fragment.json": lambda golden: ds01(),
    "ds02-sieve.json": lambda golden: ds02(),
    "ds03-canonical-records.json": lambda golden: ds03(),
    "ds04-lrat-obligation.json": ds04,
    "ds05-standalone-checker.json": lambda golden: ds05(),
    "ds06-measurement.json": lambda golden: ds06(),
    "ds07-owner-tasks.json": lambda golden: ds07(),
}
TEXT_FILES = {"ds04-carry-save.cnf": lambda golden: cnf_text("B-C01"), "ds04-carry-save-unshifted.cnf": lambda golden: cnf_text("B-C02")}
GOLDEN = "ds04-carry-save-golden.lrat"
SEMANTICS = "semantics.md"
MANIFEST = "manifest.json"


def build(golden: str, semantics: bytes) -> dict[str, bytes]:
    """Every generated laboratory file, keyed by its path under the laboratory root."""

    files: dict[str, bytes] = {}
    for name, make in GENERATED.items():
        files[name] = canonical(make(golden)) + b"\n"
    for name, make in TEXT_FILES.items():
        files[name] = make(golden).encode("ascii")
    files[GOLDEN] = golden.encode("ascii")
    files[SEMANTICS] = semantics
    entries = [{"name": name, "mode": "100644", "bytes": len(data), "sha256": sha256(data)} for name, data in sorted(files.items())]
    cases = ["DS-01", "DS-02", "DS-03", "DS-04", "DS-05", "DS-06", "DS-07"]
    digest = sha256(canonical({"cases": cases, "files": entries}))
    files[MANIFEST] = canonical({"schema_version": SCHEMA, "suite_version": SUITE_VERSION, "cases": cases, "files": entries, "input_manifest_sha256": digest}) + b"\n"
    laboratory = {"shared-inputs/" + name: data for name, data in files.items()}
    laboratory["protocol/toolchains.json"] = canonical(toolchains()) + b"\n"
    laboratory["protocol/suite-overlay.json"] = canonical(overlay(digest)) + b"\n"
    return laboratory


def main(arguments: list[str]) -> int:
    root = ROOT / LAB
    if len(arguments) == 2 and arguments[0] == "generate":
        golden = Path(arguments[1]).read_text(encoding="ascii")
        if lrat_verdict("B-C01", cnf_text("B-C01"), golden) != ("accept",):
            print("the golden certificate does not check", file=sys.stderr)
            return 1
        for name, data in build(golden, (root / "shared-inputs" / SEMANTICS).read_bytes()).items():
            (root / name).parent.mkdir(parents=True, exist_ok=True)
            (root / name).write_bytes(data)
        return 0
    if arguments == ["check"]:
        golden = (root / "shared-inputs" / GOLDEN).read_text(encoding="ascii")
        expected = build(golden, (root / "shared-inputs" / SEMANTICS).read_bytes())
        present = {path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file()}
        problems = sorted(set(expected) ^ present)
        problems += [name for name, data in expected.items() if name in present and (root / name).read_bytes() != data]
        for name in problems:
            print(f"laboratory file differs from the reference: {name}", file=sys.stderr)
        return 1 if problems else 0
    print("usage: d006_shared.py generate GOLDEN_LRAT | check", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
