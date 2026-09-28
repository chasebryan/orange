// The code editor: a native <textarea> for typing, selection, undo, and
// input methods, drawn over a highlighted copy of the same text. The two
// layers share one font and one grid, so what you see is exactly what you
// edit, and nothing about typing depends on Tabula's code.

import { h, showTooltip, hideTooltip } from "./ui.js";
import { tokenize, classify, lineStarts, lineOf } from "./orange.js";

const PLAIN_ABOVE = 400_000; // characters; beyond this, skip colouring
const PAD_TOP = 14;
const PAD_LEFT = 16;
const INDENT = "  ";

const ESCAPES = { "&": "&amp;", "<": "&lt;", ">": "&gt;" };
const escapeHtml = (text) => text.replace(/[&<>]/g, (c) => ESCAPES[c]);

export class Editor {
  constructor(host, options = {}) {
    this.options = options;
    this.diagnostics = [];
    this.related = [];
    this.inlays = new Map();
    this.noteLines = new Map();
    this.flashRange = null;
    this.lineCount = 0;
    this.frame = 0;

    this.gutterInner = h("div", { class: "gutter-inner" });
    this.gutter = h("div", { class: "gutter", "aria-hidden": "true" }, this.gutterInner);
    this.currentLine = h("div", { class: "current-line" });
    this.pre = h("pre", { class: "highlight", "aria-hidden": "true" });
    this.layer = h("div", { class: "code-layer" }, this.currentLine, this.pre);
    this.input = h("textarea", {
      class: "input",
      spellcheck: "false",
      autocapitalize: "off",
      autocomplete: "off",
      autocorrect: "off",
      wrap: "off",
      "aria-label": options.label || "Orange source",
    });
    this.code = h("div", { class: "code" }, this.layer, this.input);
    this.root = h("div", { class: "editor" }, this.gutter, this.code);
    host.append(this.root);

    this.input.addEventListener("input", () => {
      this.flashRange = null;
      this.schedule();
      if (this.options.onChange) this.options.onChange(this.input.value);
    });
    this.input.addEventListener("scroll", () => this.syncScroll());
    this.input.addEventListener("keydown", (event) => this.onKey(event));
    for (const name of ["select", "keyup", "mouseup", "focus"]) {
      this.input.addEventListener(name, () => this.updateCursor());
    }
    this.input.addEventListener("mousemove", (event) => this.onHover(event));
    this.input.addEventListener("mouseleave", () => hideTooltip());
    this.gutter.addEventListener("click", (event) => this.onGutterClick(event));
    this.gutter.addEventListener("wheel", (event) => {
      this.input.scrollTop += event.deltaY;
      event.preventDefault();
    }, { passive: false });
  }

  // ---------- Content ----------

  getValue() { return this.input.value; }

  setValue(text) {
    this.input.value = text;
    this.input.setSelectionRange(0, 0);
    this.input.scrollTop = 0;
    this.input.scrollLeft = 0;
    this.render();
  }

  setDiagnostics(diagnostics, related = []) {
    this.diagnostics = diagnostics;
    this.related = related;
    this.schedule();
  }

  // inlays: Map(lineIndex -> DOM-free {label, value}); rendered at line end.
  setInlays(inlays) {
    this.inlays = inlays;
    this.schedule();
  }

  // noteLines: Map(lineIndex -> [note names])
  setNoteLines(noteLines) {
    this.noteLines = noteLines;
    this.renderGutterMarks();
  }

  schedule() {
    if (this.frame) return;
    this.frame = requestAnimationFrame(() => {
      this.frame = 0;
      this.render();
    });
  }

  // ---------- Painting ----------

  render() {
    const text = this.input.value;
    this.starts = lineStarts(text);
    const tokens = text.length > PLAIN_ABOVE ? [{ kind: "plain", start: 0, end: text.length }] : classify(text, tokenize(text));
    this.tokens = tokens;
    const marks = new Uint8Array(text.length + 1);
    const paint = (from, to, bit) => {
      const end = Math.min(Math.max(to, from + 1), text.length);
      for (let i = Math.max(0, from); i < end; i++) marks[i] |= bit;
    };
    for (const d of this.diagnostics) paint(d.from, d.to, 1);
    for (const r of this.related) paint(r.from, r.to, 2);
    if (this.flashRange) paint(this.flashRange.from, this.flashRange.to, 4);

    let html = "";
    let line = 0;
    const inlay = (index) => {
      const item = this.inlays.get(index);
      return item ? `<span class="inlay">${escapeHtml(item.label)} <b>${escapeHtml(item.value)}</b></span>` : "";
    };
    for (const token of tokens) {
      const cls = tokenClass(token, text);
      let pos = token.start;
      while (pos < token.end) {
        const bits = marks[pos];
        let end = pos;
        while (end < token.end && marks[end] === bits && text[end] !== "\n") end++;
        if (end === pos) {
          // A newline: finish the line with its inlay.
          html += `${inlay(line)}\n`;
          line++;
          pos++;
          continue;
        }
        const piece = text.slice(pos, end);
        const markCls = bits ? `${bits & 1 ? " d-error" : ""}${bits & 2 ? " d-related" : ""}${bits & 4 ? " d-flash" : ""}` : "";
        const classes = `${cls}${markCls}`.trim();
        const body = token.kind === "number" && !bits ? numberHtml(piece) : escapeHtml(piece);
        html += classes ? `<span class="${classes}">${body}</span>` : body;
        pos = end;
      }
    }
    html += inlay(line);
    if (text.endsWith("\n") || text.length === 0) html += " ";
    this.pre.innerHTML = html;
    this.renderGutter();
    this.updateCursor();
    this.syncScroll();
  }

  renderGutter() {
    const count = this.starts.length;
    if (count !== this.lineCount) {
      const fragment = document.createDocumentFragment();
      for (let i = this.lineCount; i < count; i++) {
        fragment.append(h("div", { class: "gutter-line", dataset: { line: String(i) }, text: String(i + 1) }));
      }
      while (this.gutterInner.childElementCount > count) this.gutterInner.lastChild.remove();
      this.gutterInner.append(fragment);
      this.lineCount = count;
    }
    this.renderGutterMarks();
  }

  renderGutterMarks() {
    for (const node of this.gutterInner.querySelectorAll(".mark, .note-mark")) node.remove();
    if (!this.starts) return;
    const rows = this.gutterInner.children;
    const errorLines = new Set(this.diagnostics.map((d) => lineOf(this.starts, d.from)));
    for (const index of errorLines) {
      if (rows[index]) rows[index].append(h("span", { class: "mark error" }));
    }
    for (const [index, names] of this.noteLines) {
      if (rows[index]) rows[index].append(h("span", { class: "note-mark", title: `Cited in ${names.join(", ")}` }));
    }
  }

  syncScroll() {
    const { scrollLeft, scrollTop } = this.input;
    this.layer.style.transform = `translate(${-scrollLeft}px, ${-scrollTop}px)`;
    this.gutterInner.style.transform = `translateY(${-scrollTop}px)`;
  }

  // Measured once the editor is visible; a hidden editor measures zero, so
  // nothing is cached until a real width comes back.
  metrics() {
    if (!this.charWidth) {
      const probe = h("span", { text: "M".repeat(80) });
      this.pre.append(probe);
      const width = probe.getBoundingClientRect().width / 80;
      const lineHeight = parseFloat(getComputedStyle(this.pre).lineHeight) || 21;
      probe.remove();
      if (width > 0) {
        this.charWidth = width;
        this.lineHeight = lineHeight;
      }
      return { charWidth: width || 8, lineHeight };
    }
    return { charWidth: this.charWidth, lineHeight: this.lineHeight };
  }

  // ---------- Cursor ----------

  cursor() {
    const offset = this.input.selectionStart;
    const starts = this.starts || lineStarts(this.input.value);
    const line = lineOf(starts, offset);
    const column = [...this.input.value.slice(starts[line], offset)].length + 1;
    return { offset, line: line + 1, column, selection: this.input.selectionEnd - this.input.selectionStart };
  }

  updateCursor() {
    if (!this.starts) return;
    const { lineHeight } = this.metrics();
    const position = this.cursor();
    this.currentLine.style.top = `${PAD_TOP + (position.line - 1) * lineHeight}px`;
    const rows = this.gutterInner.children;
    if (this.currentRow) this.currentRow.classList.remove("current");
    this.currentRow = rows[position.line - 1];
    if (this.currentRow) this.currentRow.classList.add("current");
    if (this.options.onCursor) this.options.onCursor(position);
  }

  // Selects [from, to), scrolls it into view, and briefly highlights it.
  reveal(from, to = from, { flash = true, focus = true } = {}) {
    if (!this.starts) this.render();
    const { lineHeight, charWidth } = this.metrics();
    if (focus) this.input.focus({ preventScroll: true });
    this.input.setSelectionRange(from, to);
    const line = lineOf(this.starts, from);
    const column = from - this.starts[line];
    const top = PAD_TOP + line * lineHeight;
    const view = this.input.clientHeight;
    if (top < this.input.scrollTop + lineHeight || top > this.input.scrollTop + view - 3 * lineHeight) {
      this.input.scrollTop = Math.max(0, top - view / 3);
    }
    const left = PAD_LEFT + column * charWidth;
    if (left < this.input.scrollLeft || left > this.input.scrollLeft + this.input.clientWidth - 40) {
      this.input.scrollLeft = Math.max(0, left - 80);
    }
    if (flash) {
      this.flashRange = { from, to: Math.max(to, from + 1) };
      clearTimeout(this.flashTimer);
      this.flashTimer = setTimeout(() => { this.flashRange = null; this.schedule(); }, 1200);
    }
    this.render();
  }

  focus() { this.input.focus({ preventScroll: true }); }

  // ---------- Editing ----------

  // Replaces [from, to) with `text` as one undoable edit.
  replace(from, to, text, selectFrom = null, selectTo = null) {
    this.input.focus({ preventScroll: true });
    this.input.setSelectionRange(from, to);
    let done = false;
    try { done = document.execCommand("insertText", false, text); } catch { done = false; }
    if (!done) {
      this.input.setRangeText(text, from, to, "end");
      this.input.dispatchEvent(new Event("input"));
    }
    if (selectFrom !== null) this.input.setSelectionRange(selectFrom, selectTo ?? selectFrom);
    this.updateCursor();
  }

  // Inserts `text` at `offset`, re-indenting its lines to match that line.
  insertBlock(offset, text) {
    const value = this.input.value;
    const lineStart = value.lastIndexOf("\n", offset - 1) + 1;
    const indent = value.slice(lineStart, offset).match(/^[ \t]*/)[0];
    const body = text.split("\n").map((line, index) => (index === 0 || line === "" ? line : indent + line)).join("\n");
    this.replace(offset, offset, body, offset + body.length);
    this.reveal(offset, offset + body.length, { flash: true });
  }

  onKey(event) {
    const input = this.input;
    const value = input.value;
    const start = input.selectionStart;
    const end = input.selectionEnd;
    if ((event.ctrlKey || event.metaKey) && !event.altKey && event.key.toLowerCase() === "s") {
      event.preventDefault();
      if (this.options.onSave) this.options.onSave();
      return;
    }
    if (event.key === "Tab" && !event.ctrlKey && !event.metaKey && !event.altKey) {
      event.preventDefault();
      const lineStart = value.lastIndexOf("\n", start - 1) + 1;
      const multiLine = value.slice(start, end).includes("\n");
      if (!event.shiftKey && !multiLine) {
        this.replace(start, end, INDENT);
        return;
      }
      const blockEnd = value.indexOf("\n", end - (end > start && value[end - 1] === "\n" ? 1 : 0));
      const stop = blockEnd === -1 ? value.length : blockEnd;
      const lines = value.slice(lineStart, stop).split("\n");
      const changed = lines.map((line) => (event.shiftKey ? line.replace(/^( {1,2}|\t)/, "") : line.length ? INDENT + line : line)).join("\n");
      this.replace(lineStart, stop, changed, lineStart, lineStart + changed.length);
      return;
    }
    if (event.key === "Enter" && !event.ctrlKey && !event.metaKey && !event.altKey && !event.isComposing) {
      event.preventDefault();
      const lineStart = value.lastIndexOf("\n", start - 1) + 1;
      const indent = value.slice(lineStart, start).match(/^[ \t]*/)[0];
      const before = value.slice(lineStart, start).trimEnd();
      const opens = before.endsWith("{") || before.endsWith("(") || before.endsWith("[");
      const closes = /^[ \t]*[}\])]/.test(value.slice(end));
      if (opens && closes) {
        const insert = `\n${indent}${INDENT}\n${indent}`;
        const caret = start + 1 + indent.length + INDENT.length;
        this.replace(start, end, insert, caret);
      } else {
        const insert = `\n${indent}${opens ? INDENT : ""}`;
        this.replace(start, end, insert, start + insert.length);
      }
      return;
    }
    if ((event.key === "}" || event.key === ")" || event.key === "]") && start === end) {
      const lineStart = value.lastIndexOf("\n", start - 1) + 1;
      const head = value.slice(lineStart, start);
      if (/^[ \t]+$/.test(head) && head.length >= INDENT.length) {
        event.preventDefault();
        const trimmed = head.slice(0, head.length - INDENT.length);
        this.replace(lineStart, start, trimmed + event.key, lineStart + trimmed.length + 1);
      }
    }
  }

  onGutterClick(event) {
    const row = event.target.closest(".gutter-line");
    if (!row || !this.starts) return;
    const index = Number(row.dataset.line);
    if (event.target.classList.contains("note-mark") && this.options.onNoteMark) {
      this.options.onNoteMark(index, this.noteLines.get(index) || []);
      return;
    }
    const from = this.starts[index];
    const next = index + 1 < this.starts.length ? this.starts[index + 1] : this.input.value.length;
    this.input.focus({ preventScroll: true });
    this.input.setSelectionRange(from, next);
    this.updateCursor();
  }

  onHover(event) {
    if (!this.diagnostics.length || !this.starts) return;
    const { charWidth, lineHeight } = this.metrics();
    const rect = this.input.getBoundingClientRect();
    const x = event.clientX - rect.left + this.input.scrollLeft - PAD_LEFT;
    const y = event.clientY - rect.top + this.input.scrollTop - PAD_TOP;
    const line = Math.floor(y / lineHeight);
    if (line < 0 || line >= this.starts.length) { hideTooltip(); return; }
    const offset = this.starts[line] + Math.max(0, Math.floor(x / charWidth));
    const hit = this.diagnostics.find((d) => offset >= d.from && offset < Math.max(d.to, d.from + 1));
    if (!hit) { hideTooltip(); return; }
    const notes = (hit.notes || []).map((note) => h("small", { text: note }));
    showTooltip(event.clientX, event.clientY,
      h("div", {}, h("code", { text: hit.code }), " ", hit.message),
      hit.label ? h("small", { text: hit.label }) : null,
      notes);
  }
}

function tokenClass(token, text) {
  switch (token.kind) {
    case "ws": case "plain": return "";
    case "stratum": return `t-stratum t-${text.slice(token.start, token.end)}`;
    default: return `t-${token.kind}`;
  }
}

// Dims a number's radix prefix and digit separators: 0x_ff_ff.
function numberHtml(text) {
  const match = /^(0[bBxX])?(.*)$/.exec(text);
  const prefix = match[1] ? `<span class="radix">${match[1]}</span>` : "";
  return prefix + escapeHtml(match[2]).replace(/_/g, '<span class="sep">_</span>');
}
