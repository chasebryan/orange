// Orange 2026 lexical structure, for colouring and outlining in the editor.
// The compiler stays the authority: this tokenizer only paints the text while
// you type, and orangec's own diagnostics decide what is valid.

export const STRATA = ["spec", "impl", "game", "proof", "claim"];
export const KEYWORDS = new Set(["edition", "module", ...STRATA]);

// Longest match, in the same order as orangec: three bytes, then two, then one.
const THREE = ["<<<", ">>>"];
const TWO = new Set(["..", "++", "::", "&&", "||", "==", "!=", "<=", ">=", "<<", ">>", "->", "=>"]);
const ONE = new Set([..."(){}[],:;.+-*/%&|^~!=<>?"]);
const BRACKETS = new Set([..."(){}[]"]);
const IDENT_START = /[A-Za-z_]/;
const IDENT_PART = /[A-Za-z0-9_]/;
const INTEGER = /^(?:0[bB][01](?:_?[01])*|0[xX][0-9a-fA-F](?:_?[0-9a-fA-F])*|[0-9](?:_?[0-9])*)$/;

// Returns tokens {kind, start, end} covering every character of `text`.
// Kinds: ws, comment, string, number, keyword, stratum, ident, punct, op, error.
export function tokenize(text) {
  const tokens = [];
  const n = text.length;
  let i = 0;
  const push = (kind, start, end) => tokens.push({ kind, start, end });
  while (i < n) {
    const c = text[i];
    const start = i;
    if (c === " " || c === "\t" || c === "\n" || c === "\r") {
      while (i < n && " \t\n\r".includes(text[i])) i++;
      push("ws", start, i);
    } else if (c === "/" && text[i + 1] === "/") {
      while (i < n && text[i] !== "\n" && text[i] !== "\r") i++;
      push("comment", start, i);
    } else if (c === "/" && text[i + 1] === "*") {
      let depth = 1;
      i += 2;
      while (i < n && depth > 0) {
        if (text[i] === "/" && text[i + 1] === "*") { depth++; i += 2; }
        else if (text[i] === "*" && text[i + 1] === "/") { depth--; i += 2; }
        else i++;
      }
      push(depth > 0 ? "error" : "comment", start, i);
    } else if (c === '"') {
      // Escapes: \" \\ \n \r \t \0 and \xNN; anything else is an error.
      i++;
      let closed = false;
      let valid = true;
      while (i < n && text[i] !== "\n" && text[i] !== "\r") {
        if (text[i] === "\\") {
          const next = text[i + 1];
          if (next !== undefined && '"\\nrt0'.includes(next)) { i += 2; continue; }
          if (next === "x" && /^[0-9A-Fa-f]{2}$/.test(text.slice(i + 2, i + 4))) { i += 4; continue; }
          valid = false;
          i += next === undefined || next === "\n" || next === "\r" ? 1 : 2;
          continue;
        }
        if (text[i] === '"') { i++; closed = true; break; }
        i++;
      }
      push(closed && valid ? "string" : "error", start, i);
    } else if (c >= "0" && c <= "9") {
      while (i < n && IDENT_PART.test(text[i])) i++;
      push(INTEGER.test(text.slice(start, i)) ? "number" : "error", start, i);
    } else if (IDENT_START.test(c)) {
      while (i < n && IDENT_PART.test(text[i])) i++;
      const word = text.slice(start, i);
      // `hex` written directly before `"` is one hex string, as in orangec.
      if (word === "hex" && text[i] === '"') {
        const hex = lexHexString(text, i);
        push(hex.valid ? "string" : "error", start, hex.end);
        i = hex.end;
        continue;
      }
      push(STRATA.includes(word) ? "stratum" : KEYWORDS.has(word) ? "keyword" : "ident", start, i);
    } else if (THREE.some((spelling) => text.startsWith(spelling, i))) {
      i += 3;
      push("op", start, i);
    } else if (TWO.has(text.slice(i, i + 2))) {
      i += 2;
      push("op", start, i);
    } else if (ONE.has(c)) {
      i++;
      push(BRACKETS.has(c) || c === ";" || c === "," ? "punct" : "op", start, i);
    } else {
      const scalar = text.codePointAt(i);
      i += scalar > 0xffff ? 2 : 1;
      push("error", start, i);
    }
  }
  return tokens;
}

// The bytes of a hex string that begins at its opening quote.
// A hex string is pairs of digits, and spaces only between those pairs.
function lexHexString(text, quote) {
  let i = quote + 1;
  let pending = false;
  let valid = true;
  let closed = false;
  while (i < text.length && text[i] !== "\n" && text[i] !== "\r") {
    const c = text[i];
    if (c === '"') {
      i++;
      closed = true;
      break;
    }
    if (valid) {
      if (/[0-9A-Fa-f]/.test(c)) pending = !pending;
      else if (c === " " && !pending) { /* a space may separate bytes */ }
      else valid = false;
    }
    const scalar = text.codePointAt(i);
    i += scalar > 0xffff ? 2 : 1;
  }
  if (pending) valid = false;
  return { end: i, valid: closed && valid };
}

// Refines identifier kinds by position: declaration names, module names,
// and types (after `->`, or any capitalised identifier).
export function classify(text, tokens) {
  const code = tokens.filter((t) => t.kind !== "ws" && t.kind !== "comment");
  let inType = false;
  for (let k = 0; k < code.length; k++) {
    const t = code[k];
    const prev = code[k - 1];
    const word = text.slice(t.start, t.end);
    if (t.kind === "op" && word === "->") inType = true;
    else if (t.kind === "punct" && (word === "{" || word === ";")) inType = false;
    if (t.kind !== "ident") continue;
    const prevWord = prev ? text.slice(prev.start, prev.end) : "";
    if (prev && prev.kind === "stratum") t.kind = "decl";
    else if (prev && prevWord === "module") t.kind = "module";
    else if (inType || /^[A-Z]/.test(word)) t.kind = "type";
  }
  return tokens;
}

// Outlines a source: its edition, module, and declarations grouped by stratum.
export function outline(text, tokens) {
  const code = tokens.filter((t) => t.kind !== "ws" && t.kind !== "comment");
  const word = (t) => (t ? text.slice(t.start, t.end) : "");
  // Walks a matched pair, counting only that pair, so a parameter list
  // `(x: Mod[(1 << 255) - 19])` stops at its own close.
  const skipGroup = (index, open, close) => {
    if (word(code[index]) !== open) return index;
    let depth = 0;
    for (let j = index; j < code.length; j++) {
      const spelling = word(code[j]);
      if (spelling === open) depth++;
      else if (spelling === close && --depth === 0) return j + 1;
    }
    return code.length;
  };
  const result = { edition: null, module: null, moduleStart: null, decls: [] };
  for (let k = 0; k < code.length; k++) {
    const t = code[k];
    if (t.kind === "keyword" && word(t) === "edition" && code[k + 1]) {
      result.edition = word(code[k + 1]);
    } else if (t.kind === "keyword" && word(t) === "module" && code[k + 1]) {
      result.module = word(code[k + 1]);
      result.moduleStart = code[k + 1].start;
    } else if (t.kind === "stratum" && code[k + 1] && /^[A-Za-z_]/.test(word(code[k + 1]))) {
      const nameToken = code[k + 1];
      const decl = { stratum: word(t), name: word(nameToken), start: t.start, nameStart: nameToken.start, nameEnd: nameToken.end, end: nameToken.end, type: null, body: null };
      let j = k + 2;
      // Size and type parameters, `f[n in 1..5]` and `pow[K in {F, P}]`,
      // come before the value parameters. Their brackets can hold braces.
      if (word(code[j]) === "[") j = skipGroup(j, "[", "]");
      if (word(code[j]) === "(") j = skipGroup(j, "(", ")");
      if (word(code[j]) === "->") {
        const typeStart = code[j + 1];
        let m = j + 1;
        while (m < code.length && word(code[m]) !== "{" && word(code[m]) !== "}") m++;
        if (typeStart && m > j + 1) decl.type = text.slice(typeStart.start, code[m - 1].end).replace(/\s+/g, "");
        j = m;
      }
      if (word(code[j]) === "{") {
        let depth = 0;
        const open = j;
        for (; j < code.length; j++) {
          const w = word(code[j]);
          if (w === "{") depth++;
          else if (w === "}" && --depth === 0) break;
        }
        if (j < code.length) {
          decl.end = code[j].end;
          if (j > open + 1) decl.body = text.slice(code[open + 1].start, code[j - 1].end).trim();
        }
      }
      result.decls.push(decl);
    }
  }
  return result;
}

// Offsets of the first character of each line. Lines end at LF, CRLF, or CR,
// as in orangec.
export function lineStarts(text) {
  const starts = [0];
  for (let i = 0; i < text.length; i++) {
    const c = text.charCodeAt(i);
    if (c === 10) starts.push(i + 1);
    else if (c === 13) {
      if (text.charCodeAt(i + 1) === 10) i++;
      starts.push(i + 1);
    }
  }
  return starts;
}

// Converts a 1-based line and 1-based column counted in Unicode scalar values
// (orangec's convention) to a UTF-16 offset in `text`.
export function offsetAt(text, starts, line, column) {
  const lineIndex = Math.min(Math.max(line - 1, 0), starts.length - 1);
  let offset = starts[lineIndex];
  const limit = lineIndex + 1 < starts.length ? starts[lineIndex + 1] : text.length;
  for (let col = 1; col < column && offset < limit; col++) {
    const code = text.charCodeAt(offset);
    offset += code >= 0xd800 && code <= 0xdbff ? 2 : 1;
  }
  return Math.min(offset, text.length);
}

// Converts a UTF-16 offset to a 0-based line index (binary search).
export function lineOf(starts, offset) {
  let lo = 0;
  let hi = starts.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (starts[mid] <= offset) lo = mid;
    else hi = mid - 1;
  }
  return lo;
}

// Advances from `offset` by `count` Unicode scalar values.
export function advanceScalars(text, offset, count) {
  let end = offset;
  for (let k = 0; k < count && end < text.length; k++) {
    const code = text.charCodeAt(end);
    if (code === 10 || code === 13) break;
    end += code >= 0xd800 && code <= 0xdbff ? 2 : 1;
  }
  return end;
}

// A file name turned into a valid Orange module name.
export function moduleNameFor(fileName) {
  const base = fileName.replace(/\.or$/i, "").replace(/[^A-Za-z0-9_]/g, "_");
  const name = /^[A-Za-z_]/.test(base) ? base : `m_${base}`;
  return KEYWORDS.has(name) ? `${name}_` : name || "main";
}

// Source text for a new Orange file.
export function template(fileName) {
  return `edition 2026;\n\nmodule ${moduleNameFor(fileName)} {\n  spec answer() -> Int { 42 }\n}\n`;
}
