// A small CommonMark-style renderer for The Orange Book, the manuals, and
// notes. It builds DOM nodes directly (never HTML strings), so document text
// can only ever become text: raw HTML is shown literally and only http(s),
// mailto, relative, and tabula: links are made clickable.
//
// Supported: ATX headings with GitHub-compatible ids, paragraphs, fenced
// code, pipe tables with alignment, ordered and unordered (nested) lists,
// task items, blockquotes and GitHub alerts, thematic breaks, YAML front
// matter, inline code, emphasis with CommonMark flanking rules, strikethrough,
// links, images, autolinks, backslash escapes, and common entities.

const FENCE = /^( {0,3})(`{3,}|~{3,})[ \t]*([^\s`]*)[^`]*$/;
const HEADING = /^ {0,3}(#{1,6})(?:[ \t]+(.*?))?(?:[ \t]+#+)?[ \t]*$/;
const RULE = /^ {0,3}([-*_])(?:[ \t]*\1){2,}[ \t]*$/;
const QUOTE = /^ {0,3}> ?(.*)$/;
const ITEM = /^( {0,3})([-*+]|\d{1,9}[.)])(?:([ \t]+)(.*)|[ \t]*$)/;
const TABLE_RULE = /^ {0,3}\|?[ \t]*:?-+:?[ \t]*(?:\|[ \t]*:?-+:?[ \t]*)*\|?[ \t]*$/;
const ALERTS = { NOTE: "Note", TIP: "Tip", IMPORTANT: "Important", WARNING: "Warning", CAUTION: "Caution" };
const ENTITIES = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'", nbsp: " ", mdash: "—", ndash: "–", hellip: "…", copy: "©", times: "×", minus: "−", rarr: "→", larr: "←" };

// ---------- Blocks ----------

export function parse(source) {
  const lines = source.replace(/\r\n?/g, "\n").replace(/\t/g, "    ").split("\n");
  let start = 0;
  const blocks = [];
  if (lines[0] === "---") {
    const end = lines.indexOf("---", 1);
    if (end > 0) {
      blocks.push({ type: "front", text: lines.slice(1, end).join("\n") });
      start = end + 1;
    }
  }
  blocks.push(...parseBlocks(lines.slice(start)));
  return blocks;
}

function startsBlock(line, inParagraph) {
  if (FENCE.test(line) || HEADING.test(line) || RULE.test(line) || QUOTE.test(line)) return true;
  const item = ITEM.exec(line);
  if (item && item[4] !== undefined && item[4].trim() !== "") {
    return !inParagraph || !/^\d/.test(item[2]) || /^1[.)]$/.test(item[2]);
  }
  return false;
}

function parseBlocks(lines) {
  const blocks = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) { i++; continue; }
    let match;
    if ((match = FENCE.exec(line))) {
      const [, indent, marker, lang] = match;
      const body = [];
      i++;
      while (i < lines.length) {
        const close = new RegExp(`^ {0,3}${marker[0] === "`" ? "`" : "~"}{${marker.length},}[ \\t]*$`);
        if (close.test(lines[i])) { i++; break; }
        body.push(lines[i].startsWith(indent) ? lines[i].slice(indent.length) : lines[i].trimStart());
        i++;
      }
      blocks.push({ type: "code", lang: lang.toLowerCase(), text: body.join("\n") });
    } else if ((match = HEADING.exec(line))) {
      blocks.push({ type: "heading", level: match[1].length, text: match[2] || "" });
      i++;
    } else if (RULE.test(line)) {
      blocks.push({ type: "rule" });
      i++;
    } else if (line.trimStart().startsWith("<!--")) {
      while (i < lines.length && !lines[i].includes("-->")) i++;
      i++;
    } else if (QUOTE.test(line)) {
      const inner = [];
      while (i < lines.length && lines[i].trim()) {
        const quoted = QUOTE.exec(lines[i]);
        if (quoted) inner.push(quoted[1]);
        else if (inner.length && !startsBlock(lines[i], true)) inner.push(lines[i]);
        else break;
        i++;
      }
      const alert = /^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*$/i.exec(inner[0] || "");
      if (alert) inner.shift();
      blocks.push({ type: "quote", alert: alert ? alert[1].toUpperCase() : null, children: parseBlocks(inner) });
    } else if (ITEM.test(line) && (ITEM.exec(line)[4] || "").trim() !== "") {
      const result = parseList(lines, i);
      blocks.push(result.block);
      i = result.next;
    } else if (line.includes("|") && i + 1 < lines.length && TABLE_RULE.test(lines[i + 1]) && lines[i + 1].includes("-")) {
      const header = splitRow(line);
      const aligns = splitRow(lines[i + 1]).map((cell) => (cell.endsWith(":") ? (cell.startsWith(":") ? "center" : "right") : cell.startsWith(":") ? "left" : null));
      const rows = [];
      i += 2;
      while (i < lines.length && lines[i].trim() && lines[i].includes("|")) {
        rows.push(splitRow(lines[i]));
        i++;
      }
      blocks.push({ type: "table", header, aligns, rows });
    } else {
      const text = [line.trim()];
      i++;
      while (i < lines.length && lines[i].trim() && !startsBlock(lines[i], true)) {
        if (lines[i].includes("|") && i + 1 < lines.length && TABLE_RULE.test(lines[i + 1])) break;
        text.push(lines[i].trim());
        i++;
      }
      blocks.push({ type: "paragraph", text: text.join("\n") });
    }
  }
  return blocks;
}

function parseList(lines, i) {
  const first = ITEM.exec(lines[i]);
  const ordered = /^\d/.test(first[2]);
  const delimiter = first[2].slice(-1);
  const block = { type: "list", ordered, start: ordered ? parseInt(first[2], 10) : 1, items: [] };
  while (i < lines.length) {
    const match = ITEM.exec(lines[i]);
    if (!match || /^\d/.test(match[2]) !== ordered || match[2].slice(-1) !== delimiter) break;
    const contentIndent = match[1].length + match[2].length + Math.min((match[3] || " ").length, 4);
    const body = [match[4] || ""];
    i++;
    let sawBlank = false;
    while (i < lines.length) {
      const next = lines[i];
      if (!next.trim()) { sawBlank = true; body.push(""); i++; continue; }
      const indent = next.length - next.trimStart().length;
      if (indent >= contentIndent) { body.push(next.slice(contentIndent)); sawBlank = false; i++; continue; }
      if (sawBlank || startsBlock(next, true) || ITEM.test(next)) break;
      body.push(next.trim());
      i++;
    }
    while (body.length && body[body.length - 1] === "") body.pop();
    const task = /^\[([ xX])\][ \t]+/.exec(body[0]);
    if (task) body[0] = body[0].slice(task[0].length);
    block.items.push({ task: task ? task[1] !== " " : null, children: parseBlocks(body), loose: body.includes("") });
    if (sawBlank && !(i < lines.length && ITEM.test(lines[i]))) break;
  }
  return { block, next: i };
}

function splitRow(line) {
  let text = line.trim();
  if (text.startsWith("|")) text = text.slice(1);
  if (text.endsWith("|") && !text.endsWith("\\|")) text = text.slice(0, -1);
  const cells = [];
  let cell = "";
  let inCode = false;
  for (let k = 0; k < text.length; k++) {
    const c = text[k];
    if (c === "\\" && text[k + 1] === "|") { cell += "|"; k++; continue; }
    if (c === "`") inCode = !inCode;
    if (c === "|" && !inCode) { cells.push(cell.trim()); cell = ""; continue; }
    cell += c;
  }
  cells.push(cell.trim());
  return cells;
}

// ---------- Rendering ----------

// Renders Markdown into a new <article class="prose">. `context` supplies:
//   link(href) -> {kind: "external"|"doc"|"anchor"|"cite"|"none", ...}
//   onLink(target, event), image(src, img), code(lang, text) -> Element|null
export function render(source, context = {}) {
  const article = document.createElement("article");
  article.className = "prose";
  const slugs = new Map();
  const headings = [];
  const state = { context, slugs, headings };
  for (const block of parse(source)) article.append(renderBlock(block, state));
  return { element: article, headings };
}

function el(tag, className, ...children) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  for (const child of children) if (child !== null && child !== undefined) node.append(child);
  return node;
}

function renderBlock(block, state) {
  switch (block.type) {
    case "heading": {
      const node = el(`h${block.level}`);
      node.append(inline(block.text, state));
      const id = slugFor(node.textContent, state.slugs);
      node.id = id;
      state.headings.push({ level: block.level, text: node.textContent, id });
      return node;
    }
    case "paragraph": return el("p", null, inline(block.text, state));
    case "rule": return el("hr");
    case "front": return el("pre", "front-matter", el("code", null, block.text));
    case "code": {
      const custom = state.context.code ? state.context.code(block.lang, block.text) : null;
      if (custom) return custom;
      const pre = el("pre", null, el("code", null, block.text));
      if (block.lang) pre.dataset.lang = block.lang;
      return pre;
    }
    case "quote": {
      const node = el("blockquote", block.alert ? `callout callout-${block.alert.toLowerCase()}` : null);
      if (block.alert) node.append(el("p", "callout-title", ALERTS[block.alert]));
      for (const child of block.children) node.append(renderBlock(child, state));
      return node;
    }
    case "list": {
      const node = el(block.ordered ? "ol" : "ul");
      if (block.ordered && block.start !== 1) node.start = block.start;
      for (const item of block.items) {
        const li = el("li");
        if (item.task !== null) {
          const box = document.createElement("input");
          box.type = "checkbox";
          box.checked = item.task;
          box.disabled = true;
          box.className = "task";
          li.append(box);
        }
        for (const child of item.children) {
          if (child.type === "paragraph" && !item.loose) li.append(inline(child.text, state));
          else li.append(renderBlock(child, state));
        }
        node.append(li);
      }
      return node;
    }
    case "table": {
      const table = el("table");
      const head = el("tr");
      block.header.forEach((cell, index) => {
        const th = el("th", null, inline(cell, state));
        if (block.aligns[index]) th.style.textAlign = block.aligns[index];
        head.append(th);
      });
      table.append(el("thead", null, head));
      const body = el("tbody");
      for (const row of block.rows) {
        const tr = el("tr");
        block.header.forEach((_, index) => {
          const td = el("td", null, inline(row[index] ?? "", state));
          if (block.aligns[index]) td.style.textAlign = block.aligns[index];
          tr.append(td);
        });
        body.append(tr);
      }
      table.append(body);
      return el("div", "table-wrap", table);
    }
    default: return document.createTextNode("");
  }
}

// GitHub's heading ids: lower case, drop punctuation except `-` and `_`,
// spaces become `-`, and repeats get `-1`, `-2`, ….
export function slugFor(text, seen = new Map()) {
  const base = text.toLowerCase().replace(/[^\p{L}\p{M}\p{N}\p{Pc} -]/gu, "").replace(/ /g, "-");
  const count = seen.get(base) || 0;
  seen.set(base, count + 1);
  return count ? `${base}-${count}` : base;
}

// ---------- Inline ----------

const PUNCT = /[\p{P}\p{S}]/u;
const SPACE = /\s/u;

export function inline(text, state) {
  const items = scan(text, state);
  processEmphasis(items);
  const fragment = document.createDocumentFragment();
  for (const item of items) fragment.append(item.node || document.createTextNode(item.text));
  return fragment;
}

function scan(text, state) {
  const items = [];
  let buffer = "";
  const flush = () => { if (buffer) { items.push({ text: buffer }); buffer = ""; } };
  let k = 0;
  while (k < text.length) {
    const c = text[k];
    if (c === "\\" && k + 1 < text.length) {
      const next = text[k + 1];
      if (next === "\n") { flush(); items.push({ node: document.createElement("br") }); k += 2; continue; }
      if (PUNCT.test(next)) { buffer += next; k += 2; continue; }
    }
    if (c === "`") {
      let run = 1;
      while (text[k + run] === "`") run++;
      const fence = "`".repeat(run);
      let close = text.indexOf(fence, k + run);
      while (close !== -1 && text[close + run] === "`") close = text.indexOf(fence, close + run + 1);
      if (close !== -1) {
        flush();
        let code = text.slice(k + run, close).replace(/\n/g, " ");
        if (/^ .* $/.test(code) && code.trim()) code = code.slice(1, -1);
        items.push({ node: el("code", null, code) });
        k = close + run;
        continue;
      }
      buffer += fence;
      k += run;
      continue;
    }
    if (c === "!" && text[k + 1] === "[") {
      const link = parseLink(text, k + 1);
      if (link) {
        flush();
        items.push({ node: renderImage(link, state) });
        k = link.end;
        continue;
      }
    }
    if (c === "[") {
      const link = parseLink(text, k);
      if (link) {
        flush();
        items.push({ node: renderLink(link, state) });
        k = link.end;
        continue;
      }
    }
    if (c === "<") {
      const auto = /^<((?:https?:\/\/|mailto:)[^\s<>]+)>/.exec(text.slice(k));
      if (auto) {
        flush();
        items.push({ node: renderLink({ label: auto[1], href: auto[1], plain: true }, state) });
        k += auto[0].length;
        continue;
      }
      if (text.startsWith("<!--", k)) {
        const end = text.indexOf("-->", k + 4);
        if (end !== -1) { k = end + 3; continue; }
      }
    }
    if (c === "&") {
      const entity = /^&(?:#(\d{1,7})|#[xX]([0-9a-fA-F]{1,6})|([a-zA-Z]{2,8}));/.exec(text.slice(k));
      if (entity) {
        const code = entity[1] ? parseInt(entity[1], 10) : entity[2] ? parseInt(entity[2], 16) : null;
        const value = code !== null ? (code > 0 && code <= 0x10ffff ? String.fromCodePoint(code) : "�") : ENTITIES[entity[3]];
        if (value !== undefined) { buffer += value; k += entity[0].length; continue; }
      }
    }
    if (c === "*" || c === "_" || c === "~") {
      let run = 1;
      while (text[k + run] === c) run++;
      if (c === "~" && run !== 2) { buffer += c.repeat(run); k += run; continue; }
      const before = k > 0 ? text[k - 1] : " ";
      const after = k + run < text.length ? text[k + run] : " ";
      const left = !SPACE.test(after) && (!PUNCT.test(after) || SPACE.test(before) || PUNCT.test(before));
      const right = !SPACE.test(before) && (!PUNCT.test(before) || SPACE.test(after) || PUNCT.test(after));
      const canOpen = c === "_" ? left && (!right || PUNCT.test(before)) : left;
      const canClose = c === "_" ? right && (!left || PUNCT.test(after)) : right;
      flush();
      items.push({ delim: c, count: run, original: run, canOpen, canClose, text: c.repeat(run) });
      k += run;
      continue;
    }
    if (c === "\n") {
      if (buffer.endsWith("  ")) { buffer = buffer.replace(/ +$/, ""); flush(); items.push({ node: document.createElement("br") }); }
      else buffer = buffer.replace(/ +$/, "") + " ";
      k++;
      continue;
    }
    buffer += c;
    k++;
  }
  flush();
  return items;
}

// Parses `[label](destination "title")` starting at the `[` at `k`.
function parseLink(text, k) {
  let depth = 0;
  let j = k;
  for (; j < text.length; j++) {
    const c = text[j];
    if (c === "\\") { j++; continue; }
    if (c === "`") {
      const close = text.indexOf("`", j + 1);
      if (close !== -1) { j = close; continue; }
    }
    if (c === "[") depth++;
    else if (c === "]" && --depth === 0) break;
  }
  if (j >= text.length || text[j + 1] !== "(") return null;
  let m = j + 2;
  while (text[m] === " " || text[m] === "\n") m++;
  let href = "";
  if (text[m] === "<") {
    const close = text.indexOf(">", m);
    if (close === -1) return null;
    href = text.slice(m + 1, close);
    m = close + 1;
  } else {
    let parens = 0;
    for (; m < text.length; m++) {
      const c = text[m];
      if (c === "\\" && m + 1 < text.length) { href += text[m + 1]; m++; continue; }
      if (c === " " || c === "\n") break;
      if (c === "(") parens++;
      if (c === ")") { if (parens === 0) break; parens--; }
      href += c;
    }
  }
  while (text[m] === " " || text[m] === "\n") m++;
  let title = null;
  if (text[m] === '"' || text[m] === "'") {
    const close = text.indexOf(text[m], m + 1);
    if (close === -1) return null;
    title = text.slice(m + 1, close);
    m = close + 1;
    while (text[m] === " " || text[m] === "\n") m++;
  }
  if (text[m] !== ")") return null;
  return { label: text.slice(k + 1, j), href, title, end: m + 1 };
}

function renderLink(link, state) {
  const target = state.context.link ? state.context.link(link.href) : { kind: /^(https?:|mailto:)/i.test(link.href) ? "external" : "none" };
  const label = link.plain ? document.createTextNode(link.label) : inline(link.label, { ...state, inLink: true });
  if (state.inLink || target.kind === "none") {
    const span = el("span", null, label);
    return span;
  }
  const a = el("a", target.kind === "cite" ? "cite" : null, label);
  if (target.kind === "external") {
    a.href = link.href;
    a.target = "_blank";
    a.rel = "noopener noreferrer";
  } else {
    a.href = "#";
    a.addEventListener("click", (event) => {
      event.preventDefault();
      if (state.context.onLink) state.context.onLink(target, event);
    });
  }
  a.title = link.title || target.title || link.href;
  return a;
}

function renderImage(link, state) {
  const img = document.createElement("img");
  img.alt = link.label.replace(/[*_`]/g, "");
  if (link.title) img.title = link.title;
  img.loading = "lazy";
  if (state.context.image) state.context.image(link.href, img);
  return img;
}

// CommonMark's delimiter algorithm, reduced to what prose needs: `*`/`_`
// emphasis and strong, and `~~` strikethrough.
function processEmphasis(items) {
  for (let c = 0; c < items.length; c++) {
    const closer = items[c];
    if (!closer.delim || !closer.canClose || closer.count === 0) continue;
    let o = c - 1;
    for (; o >= 0; o--) {
      const opener = items[o];
      if (!opener.delim || opener.delim !== closer.delim || !opener.canOpen || opener.count === 0) continue;
      if (closer.delim !== "~" && (opener.canClose || closer.canOpen) && (opener.original + closer.original) % 3 === 0 && !(opener.original % 3 === 0 && closer.original % 3 === 0)) continue;
      break;
    }
    if (o < 0) continue;
    const opener = items[o];
    const use = closer.delim === "~" ? 2 : opener.count >= 2 && closer.count >= 2 ? 2 : 1;
    const tag = closer.delim === "~" ? "del" : use === 2 ? "strong" : "em";
    const node = document.createElement(tag);
    for (const inner of items.slice(o + 1, c)) node.append(inner.node || document.createTextNode(inner.text));
    opener.count -= use;
    closer.count -= use;
    opener.text = opener.delim.repeat(opener.count);
    closer.text = closer.delim.repeat(closer.count);
    items.splice(o + 1, c - o - 1, { node });
    c = o + 1;
    if (opener.count === 0) { items.splice(o, 1); c--; }
    if (closer.count === 0) { items.splice(c + 1, 1); }
    c--;
  }
}
