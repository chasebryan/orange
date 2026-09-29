"""D-006 v0.3 statement renderer (research-only).

The shared packet writes every statement, observation and negative case as a
candidate-neutral term (see shared-inputs/semantics.md, section 1). This
module renders those terms into Rocq or Lean 4 source from a candidate's
adapter, which maps every shared symbol to exactly one declaration, and
writes the files a candidate is checked with:

* ``Check`` files hold, per case, one parity declaration per shared theorem
  (the candidate's theorem must check against the rendered statement), the
  M-01 instance checks, one proof per observation by the candidate's
  declared computation method, and a trust audit of every declaration.
* one file per negative case holds exactly the rendered term, definition,
  obligation or axiom use that must be refused or reported.

Rendering is the same for both candidates up to each language's concrete
syntax; the only per-candidate knobs are the declaration names, the literal
templates, the preamble and the proof methods named in the adapter.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from typing import Any

LANGUAGES = ("rocq", "lean4")
IDENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_']*(\.[A-Za-z_][A-Za-z0-9_']*)*$")
CASE_PREFIX = {"DS-01": "D1", "DS-02": "D2", "DS-03": "D3", "DS-04": "D4"}


class RenderError(Exception):
    pass


def ident(item: str) -> str:
    """A shared id as a source identifier: D1-TH01 -> D1_TH01."""

    return item.replace("-", "_")


@dataclass
class Language:
    name: str
    adapter: dict[str, Any]
    symbols: dict[str, str] = field(init=False)

    def __post_init__(self) -> None:
        if self.name not in LANGUAGES:
            raise RenderError(f"unknown language {self.name}")
        self.symbols = dict(self.adapter["symbols"])
        for key, value in self.symbols.items():
            if not IDENT.match(value):
                raise RenderError(f"{key} maps to {value!r}, which is not a declaration name")
        self.literals = self.adapter["literals"]
        self.list_literal = bool(self.adapter.get("list_literal", False))

    # -- terms -------------------------------------------------------------

    def symbol(self, item: str) -> str:
        if item not in self.symbols:
            raise RenderError(f"no mapping for shared symbol {item}")
        return "@" + self.symbols[item]

    def nat(self, digits: str) -> str:
        if not re.fullmatch(r"0|[1-9][0-9]*", digits):
            raise RenderError(f"bad natural literal {digits!r}")
        return self.literals["nat"].replace("{}", digits)

    def string(self, text: str) -> str:
        if any(ord(ch) > 0x7E or (ord(ch) < 0x20 and ch != "\n") for ch in text):
            raise RenderError("shared strings are printable ASCII and line feeds")
        if self.name == "rocq":
            body = text.replace('"', '""')
        else:
            body = text.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")
        return self.literals["str"].replace("{}", body)

    def items(self, kind: list[Any], values: list[str]) -> str:
        kind_text = self.argument(kind)
        if not values:
            return f"({self.symbol('C-08')} {kind_text})"
        if self.list_literal:
            sep = "; " if self.name == "rocq" else ", "
            return f"(([{sep.join(values)}]) : {self.symbol('T-05')} {kind_text})"
        text = f"({self.symbol('C-08')} {kind_text})"
        for value in reversed(values):
            text = f"({self.symbol('C-09')} {kind_text} {value} {text})"
        return text

    def argument(self, node: Any) -> str:
        # Rocq parses `@f` as an explicit application, so a bare symbol in
        # argument position is parenthesized; Lean's `@f` is an atom.
        text = self.term(node)
        if self.name == "rocq" and text.startswith("@"):
            return f"({text})"
        return text

    def binders(self, binders: list[list[Any]]) -> str:
        return " ".join(f"({name} : {self.term(kind)})" for name, kind in binders)

    def nary(self, parts: list[Any], op: str) -> str:
        if len(parts) < 2:
            raise RenderError("a connective takes two or more parts")
        text = self.term(parts[-1])
        for part in reversed(parts[:-1]):
            text = f"({self.term(part)} {op} {text})"
        return text

    def scoped(self, text: str) -> str:
        scope = self.adapter.get("nat_scope")
        return f"({text})%{scope}" if self.name == "rocq" and scope else f"({text})"

    def term(self, node: Any) -> str:
        if not isinstance(node, list) or not node or not isinstance(node[0], str):
            raise RenderError(f"malformed term {node!r}")
        form, rest = node[0], node[1:]
        rocq = self.name == "rocq"
        if form == "sym":
            return self.symbol(rest[0])
        if form == "var":
            if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_']*", rest[0]):
                raise RenderError(f"bad variable {rest[0]!r}")
            return rest[0]
        if form == "nat":
            return self.nat(rest[0])
        if form == "str":
            return self.string(rest[0])
        if form == "bytes":
            data = bytes.fromhex(rest[0])
            word8 = ["app", ["sym", "T-03"], ["nat", "8"]]
            values = [self.term(["app", ["sym", "F-01"], ["nat", "8"], ["nat", str(b)]]) for b in data]
            return self.items(word8, values)
        if form == "list":
            return self.items(rest[0], [self.term(item) for item in rest[1]])
        if form == "app":
            head, args = self.term(rest[0]), [self.argument(part) for part in rest[1:]]
            return "(" + " ".join([head, *args]) + ")"
        if form in ("forall", "exists"):
            word = ("forall" if form == "forall" else "exists") if rocq else ("∀" if form == "forall" else "∃")
            return f"({word} {self.binders(rest[0])}, {self.term(rest[1])})"
        if form == "imp":
            return f"({self.term(rest[0])} {'->' if rocq else '→'} {self.term(rest[1])})"
        if form == "and":
            return self.nary(rest, "/\\" if rocq else "∧")
        if form == "or":
            return self.nary(rest, "\\/" if rocq else "∨")
        if form == "not":
            return f"({'~' if rocq else '¬'} {self.term(rest[0])})"
        if form == "iff":
            return f"({self.term(rest[0])} {'<->' if rocq else '↔'} {self.term(rest[1])})"
        if form == "eq":
            return f"({self.term(rest[0])} = {self.term(rest[1])})"
        if form == "ne":
            return f"({self.term(rest[0])} {'<>' if rocq else '≠'} {self.term(rest[1])})"
        if form in ("lt", "le", "nat_add"):
            op = {"lt": "<", "le": "<=" if rocq else "≤", "nat_add": "+"}[form]
            return self.scoped(f"{self.term(rest[0])} {op} {self.term(rest[1])}")
        if form == "hole":
            return "_"
        if form == "sort":
            if rest[0] not in ("Prop", "Type"):
                raise RenderError(f"bad sort {rest[0]!r}")
            return rest[0]
        raise RenderError(f"unknown term form {form!r}")

    # -- declarations ------------------------------------------------------

    def comment(self, text: str) -> str:
        return f"(* {text} *)" if self.name == "rocq" else f"-- {text}"

    def method(self, case: str, kind: str) -> str:
        methods = self.adapter["methods"][kind]
        if isinstance(methods, str):
            return methods
        return methods.get(case, methods.get("default"))

    def theorem(self, name: str, statement: str, proof_method: str) -> list[str]:
        if self.name == "rocq":
            return [f"Example {name} : {statement}.", f"Proof. {proof_method}. Qed."]
        return [f"theorem {name} : {statement} := by", f"  {proof_method}"]

    def definition(self, name: str, statement: str, value: str) -> list[str]:
        if self.name == "rocq":
            return [f"Definition {name} : {statement} :=", f"  {value}."]
        return [f"theorem {name} : {statement} :=", f"  {value}"]

    def audit(self, name: str) -> list[str]:
        if self.name == "rocq":
            return [f'Redirect "audit-{name}" Print Assumptions {name}.']
        return [f"#print axioms {name}"]


@dataclass
class Item:
    """One checked item: its shared id, its kind and its source lines."""

    ident: str
    kind: str
    phases: dict[str, tuple[int, int]]
    expected: list[str] | None = None
    declaration: str | None = None


@dataclass
class Source:
    lang: Language
    lines: list[str] = field(default_factory=list)
    items: list[Item] = field(default_factory=list)

    def add(self, block: list[str]) -> tuple[int, int]:
        start = len(self.lines) + 1
        for line in block:
            self.lines.extend(line.split("\n"))
        return start, len(self.lines)

    def text(self) -> str:
        return "\n".join(self.lines) + "\n"


def preamble(lang: Language, case: str) -> list[str]:
    header = lang.comment(f"Generated by the D-006 v0.3 renderer for {case}; do not edit.")
    body = lang.adapter["preamble"]
    if isinstance(body, dict):
        body = body.get(case, body.get("default", ""))
    return [header, body, ""]


def merge_adapter(base: dict[str, Any], fragment: dict[str, Any], where: str = "") -> dict[str, Any]:
    """Merge one adapter fragment (adapter.d/*.json) into the adapter.

    Maps merge key by key, lists append (classifier rules from a fragment go
    before the base rules, so a case's specific rules win over catch-alls),
    and a scalar may only repeat its existing value.
    """

    for key, value in fragment.items():
        path = f"{where}.{key}" if where else key
        if key not in base:
            base[key] = value
        elif isinstance(base[key], dict) and isinstance(value, dict):
            merge_adapter(base[key], value, path)
        elif isinstance(base[key], list) and isinstance(value, list):
            base[key] = value + base[key] if key == "classifier" else base[key] + value
        elif base[key] != value:
            raise RenderError(f"adapter fragment conflicts at {path}")
    return base


def check_file(lang: Language, case: str, theorems: list[dict[str, Any]], observations: list[dict[str, Any]], instances: list[dict[str, Any]] | None = None) -> Source:
    """Parity, instance checks, observations and audits for one case."""

    src = Source(lang)
    src.add(preamble(lang, case))
    mapping = lang.adapter["theorems"]
    for th in theorems:
        if th["id"] not in mapping:
            raise RenderError(f"no mapping for theorem {th['id']}")
        name = "parity_" + ident(th["id"])
        src.add([lang.comment(th["id"])])
        span = src.add(lang.definition(name, lang.term(th["statement"]), mapping[th["id"]]))
        src.add(lang.audit(name) + [""])
        src.items.append(Item(th["id"], "parity", {"statement": span}, declaration=name))
    for inst in instances or []:
        name = "instance_" + ident(inst["id"])
        src.add([lang.comment(inst["id"] + " is M-01 instantiated")])
        setup = inst.get("setup")
        if setup:
            src.add([setup])
        stmt = f"({lang.symbol(inst['id'])} = {inst['function']})"
        span = src.add(lang.theorem(name, stmt, lang.method(case, "instance")))
        src.add(lang.audit(name) + [""])
        src.items.append(Item(inst["id"], "instance", {"proof": span}, declaration=name))
    for ob in observations:
        name = "obs_" + ident(ob["id"])
        stmt = f"({lang.term(ob['lhs'])} = {lang.term(ob['rhs'])})"
        src.add([lang.comment(ob["id"])])
        first = len(src.lines) + 1
        block = lang.theorem(name, stmt, lang.method(case, "observation"))
        src.add(block)
        span = (first, len(src.lines))
        src.add(lang.audit(name) + [""])
        src.items.append(Item(ob["id"], "observation", {"proof": span}, declaration=name))
    return src


def negative_file(lang: Language, case: str, negative: dict[str, Any]) -> Source:
    """The one file that must be refused (or audited) for a rendered negative case."""

    src = Source(lang)
    src.add(preamble(lang, case))
    src.add([lang.comment(negative["id"] + ": " + negative["name"])])
    form = negative["form"]
    name = "neg_" + ident(negative["id"])
    expected = list(negative["expected"])
    rocq = lang.name == "rocq"
    if form == "term":
        text = lang.term(negative["term"])
        block = [f"Definition {name} := {text}."] if rocq else [f"noncomputable def {name} := {text}"]
        span = src.add(block)
        src.items.append(Item(negative["id"], form, {"statement": span}, expected))
    elif form == "recursive_definition":
        d = negative["definition"]
        params = " ".join(f"({p} : {lang.term(t)})" for p, t in d["parameters"])
        result, body = lang.term(d["result"]), lang.term(d["body"])
        block = [f"Fixpoint {d['name']} {params} : {result} := {body}."] if rocq else [f"def {d['name']} {params} : {result} := {body}"]
        span = src.add(block)
        src.items.append(Item(negative["id"], form, {"definition": span}, expected))
    elif form in ("obligation", "exhaustive"):
        if form == "obligation":
            stmt = f"({lang.term(negative['lhs'])} = {lang.term(negative['rhs'])})"
            proof = lang.method(case, "observation")
        else:
            stmt = lang.term(negative["statement"])
            proof = lang.method(case, "exhaustive")
        head, *tail = lang.theorem(name, stmt, proof)
        s1 = src.add([head])
        s2 = src.add(tail)
        src.items.append(Item(negative["id"], form, {"statement": s1, "proof": s2}, expected, name))
    elif form == "axiom_use":
        axiom = name + "_false"
        stmt = lang.term(negative["statement"])
        if rocq:
            src.add([f"Axiom {axiom} : False."])
            span = src.add([f"Example {name} : {stmt}.", f"Proof. destruct {axiom}. Qed."])
        else:
            src.add([f"axiom {axiom} : False"])
            span = src.add([f"theorem {name} : {stmt} := {axiom}.elim"])
        audit = src.add(lang.audit(name))
        src.items.append(Item(negative["id"], form, {"proof": span, "audit": audit}, expected, name))
    else:
        raise RenderError(f"{negative['id']}: form {form} is not rendered to a file")
    return src


def load_adapter(raw: bytes) -> dict[str, Any]:
    adapter = json.loads(raw)
    if adapter.get("schema_version") != "d006-v0.3-adapter-1":
        raise RenderError("adapter schema_version must be d006-v0.3-adapter-1")
    if adapter.get("language") not in LANGUAGES:
        raise RenderError("adapter language must be rocq or lean4")
    return adapter
