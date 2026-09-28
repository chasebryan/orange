// Tabula's application shell. It ties together the workspace explorer, the
// Orange editor with orangec's live results, the Library (The Orange Book and
// the manuals, read from the Orange checkout itself), and the notebook.
//
// Everything is a visible button or a click. The only keyboard shortcut is
// the familiar Ctrl+S (⌘S on a Mac) to save.

import { api, loadAsset } from "./api.js";
import { h, icon, button, iconButton, clear, dialog, ask, confirmDialog, toast, relativeTime, plural } from "./ui.js";
import { Editor } from "./editor.js";
import { STRATA, tokenize, classify, outline, lineStarts, lineOf, offsetAt, advanceScalars, template } from "./orange.js";
import { parseValue, wordWidth, formatValue, bitLength, wordBits } from "./values.js";
import { render as renderMarkdown } from "./markdown.js";

// ---------- Small helpers ----------

const $ = (id) => document.getElementById(id);
const baseName = (path) => path.slice(path.lastIndexOf("/") + 1);
const dirName = (path) => (path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : "");
const joinPath = (folder, name) => (folder ? `${folder}/${name}` : name);

const store = {
  get(key, fallback) {
    try {
      const raw = localStorage.getItem(`tabula.${key}`);
      return raw === null ? fallback : JSON.parse(raw);
    } catch {
      return fallback;
    }
  },
  set(key, value) {
    try { localStorage.setItem(`tabula.${key}`, JSON.stringify(value)); } catch { /* storage unavailable */ }
  },
};

// The ids stay "ink" and "paper" so earlier choices still load. boot.js
// repeats the ids so the first paint is already in the right theme.
const THEMES = [
  { id: "ink", name: "Dark", note: "Tabula's own dark theme, with Orange accents." },
  { id: "paper", name: "Light", note: "Warm paper tones for daylight." },
  { id: "tokyo", name: "Tokyo", note: "A deep blue night, after the Tokyo Night colours." },
  { id: "corporate", name: "Corporate", note: "Black, greys, white, and navy blue, with red kept for errors." },
];

async function copyText(text, what = "Copied") {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    const scratch = h("textarea", { class: "clipboard-scratch" });
    scratch.value = text;
    document.body.append(scratch);
    scratch.select();
    try { document.execCommand("copy"); } catch { /* nothing more to try */ }
    scratch.remove();
  }
  toast(what);
}

async function sha256Hex(text) {
  if (!globalThis.crypto || !crypto.subtle) return null;
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function utf8Length(text) {
  return new TextEncoder().encode(text).length;
}

// Paints Orange source as coloured spans (text only, never HTML).
function highlight(text) {
  const fragment = document.createDocumentFragment();
  for (const token of classify(text, tokenize(text))) {
    const piece = text.slice(token.start, token.end);
    if (token.kind === "ws") { fragment.append(piece); continue; }
    const cls = token.kind === "stratum" ? `t-stratum t-${piece}` : `t-${token.kind}`;
    fragment.append(h("span", { class: cls, text: piece }));
  }
  return fragment;
}

// ---------- State ----------

const STRATUM_INFO = {
  spec: { title: "spec", hint: "the mathematical definition", live: true },
  impl: { title: "impl", hint: "the implementation", live: true },
  game: { title: "game", hint: "reserved in Orange 2026", live: false },
  proof: { title: "proof", hint: "reserved in Orange 2026", live: false },
  claim: { title: "claim", hint: "reserved in Orange 2026", live: false },
};

const PHASES = [
  { id: "lexical", title: "Lexical", about: "characters and tokens" },
  { id: "syntax", title: "Syntax", about: "the shape of the file" },
  { id: "semantic", title: "Semantic", about: "names, types, literals" },
  { id: "evaluation", title: "Evaluation", about: "exact values of typed specs" },
];

const S = {
  session: null,
  tree: null,
  treeError: null,
  expanded: new Set(),
  selectedFolder: "",
  tabs: [],
  active: null,
  opening: new Map(),
  side: store.get("side", "files"),
  panelTab: store.get("panelTab", "problems"),
  radix: store.get("radix", 10),
  live: store.get("live", true),
  catalog: null,
  catalogError: null,
  collapsedSections: new Set(store.get("collapsedSections", ["governance", "project"])),
  libraryQuery: "",
  searchResults: null,
  notes: [],
  note: null,
  notePreview: false,
};

let tabSequence = 0;

const els = {
  app: $("app"),
  toolbar: $("toolbar"),
  titlebarEnd: $("titlebar-end"),
  workspaceName: $("workspace-name"),
  rail: $("rail"),
  sidebar: $("sidebar"),
  main: $("main"),
  tabs: $("tabs"),
  views: $("views"),
  panel: $("panel"),
  notebook: $("notebook"),
  statusbar: $("statusbar"),
};

const workspaceKey = (name) => `ws:${S.session ? S.session.workspaceRoot : ""}:${name}`;

// ---------- Start ----------

const systemLight = matchMedia("(prefers-color-scheme: light)");
applyTheme(store.get("theme", null));
// Until a theme is picked, Tabula follows the system between Dark and Light.
systemLight.addEventListener("change", () => { if (store.get("theme", null) === null) applyTheme(null); });
start();

async function start() {
  if (!api.hasToken()) {
    showDisconnected("This page was opened without its session link.");
    return;
  }
  try {
    S.session = await api.session();
  } catch (error) {
    showDisconnected(error.message);
    return;
  }
  document.title = `${S.session.workspace} · Tabula`;
  els.workspaceName.textContent = S.session.workspace;
  els.workspaceName.title = S.session.workspaceRoot;
  S.expanded = new Set(store.get(workspaceKey("expanded"), []));

  buildToolbar();
  buildTitlebarEnd();
  buildRail();
  buildNotebook();
  buildSplitters();
  restoreLayout();

  window.addEventListener("beforeunload", (event) => {
    if (S.tabs.some((tab) => tab.dirty) || (S.note && S.note.text !== S.note.savedText)) {
      event.preventDefault();
      event.returnValue = "";
    }
  });
  window.addEventListener("keydown", (event) => {
    if (event.defaultPrevented) return;
    if ((event.ctrlKey || event.metaKey) && !event.altKey && event.key.toLowerCase() === "s") {
      event.preventDefault();
      if (document.activeElement && document.activeElement.classList.contains("note-text")) saveNote();
      else saveActive();
    }
  });

  await Promise.all([refreshTree(), loadCatalog(), refreshNotes()]);
  await restoreTabs();
  renderSidebar();
  renderPanel();
  renderStatus();
  updateToolbar();
  if (!S.session.orangec) {
    toast("orangec was not found, so checks are off. Start Tabula with --orangec <path> to turn them on.", true);
  }
}

function showDisconnected(reason) {
  els.app.classList.add("disconnected");
  els.app.replaceChildren(h("div", { class: "welcome" },
    h("div", { class: "welcome-inner" },
      h("img", { class: "welcome-mark", src: "/assets/tabula.svg", alt: "" }),
      h("h1", { text: "Tabula is not connected" }),
      h("p", { class: "lede", text: `${reason} Start Tabula from your terminal and open the link it prints; the link carries this session's key.` }),
      h("pre", { class: "raw-text inline-command", text: "tabula path/to/your/workspace" }),
    )));
}

// ---------- Theme and layout ----------

function applyTheme(theme, remember = false) {
  const known = THEMES.some((entry) => entry.id === theme) ? theme : systemLight.matches ? "paper" : "ink";
  document.documentElement.dataset.theme = known;
  if (remember) store.set("theme", known);
}

// A small picture of Tabula in one theme: title bar, left edge, side
// panel, and a few lines of Orange in that theme's colours.
function themePreview(theme) {
  const t = (kind, text) => h("span", { class: kind, text });
  return h("div", { class: "theme-preview", dataset: { theme }, "aria-hidden": "true" },
    h("div", { class: "pv-bar pv-chrome" }, h("i", { class: "pv-mark" }), h("i", { class: "pv-title" })),
    h("div", { class: "pv-body" },
      h("div", { class: "pv-rail pv-chrome" }, h("i", { class: "pv-on" }), h("i"), h("i")),
      h("div", { class: "pv-side" }, h("i", { class: "pv-on" }), h("i"), h("i"), h("i")),
      h("div", { class: "pv-code" },
        h("div", {}, t("t-keyword", "module"), " ", t("t-module", "round"), " ", t("t-punct", "{")),
        h("div", {}, "  ", t("t-comment", "// the key mask")),
        h("div", {}, "  ", t("t-spec", "spec"), " ", t("t-decl", "mask"), t("t-punct", "()"), " ", t("t-op", "->"), " ", t("t-type", "Word"), t("t-punct", "["), t("t-number", "8"), t("t-punct", "]")),
        h("div", {}, "    ", t("t-number", "0x5a")),
        h("div", {}, "  ", t("t-impl", "impl"), " ", t("t-decl", "mask_fast"), t("t-punct", "() {}")),
      ),
    ),
    h("div", { class: "pv-status pv-chrome" }, h("i", { class: "pv-ok" })),
  );
}

function showThemes() {
  const cards = THEMES.map((theme) => h("button", {
    class: "theme-choice",
    type: "button",
    dataset: { choice: theme.id },
    autofocus: theme.id === document.documentElement.dataset.theme,
    onclick: () => { applyTheme(theme.id, true); mark(); },
  },
  themePreview(theme.id),
  h("span", { class: "theme-name", text: theme.name }),
  h("span", { class: "theme-note", text: theme.note })));
  const mark = () => {
    for (const card of cards) {
      const on = card.dataset.choice === document.documentElement.dataset.theme;
      card.classList.toggle("selected", on);
      card.setAttribute("aria-pressed", on ? "true" : "false");
    }
  };
  mark();
  dialog({
    title: "Theme",
    content: h("div", {},
      h("p", { text: "Click a theme to use it. Tabula remembers the choice in this browser." }),
      h("div", { class: "theme-grid" }, cards)),
    actions: [{ label: "Done", value: true, kind: "primary" }],
    wide: true,
  });
}

function restoreLayout() {
  const layout = store.get("layout", {});
  if (layout.sidebar) els.app.style.setProperty("--sidebar-w", `${layout.sidebar}px`);
  if (layout.notebook) els.app.style.setProperty("--notebook-w", `${layout.notebook}px`);
  if (layout.panel) els.main.style.setProperty("--panel-h", `${layout.panel}px`);
  els.app.classList.toggle("sidebar-hidden", !!layout.sidebarHidden);
  els.app.classList.toggle("notebook-hidden", layout.notebookHidden !== false);
  els.main.classList.toggle("panel-hidden", !!layout.panelHidden);
}

function saveLayout(changes) {
  store.set("layout", { ...store.get("layout", {}), ...changes });
}

function buildSplitters() {
  const drag = (node, orientation, onMove, onReset) => {
    node.addEventListener("pointerdown", (event) => {
      if (event.button !== 0) return;
      event.preventDefault();
      node.setPointerCapture(event.pointerId);
      node.classList.add("dragging");
      document.body.classList.add(`dragging-${orientation}`);
      const move = (moveEvent) => onMove(moveEvent);
      const end = () => {
        node.classList.remove("dragging");
        document.body.classList.remove(`dragging-${orientation}`);
        node.removeEventListener("pointermove", move);
        node.removeEventListener("pointerup", end);
        node.removeEventListener("pointercancel", end);
      };
      node.addEventListener("pointermove", move);
      node.addEventListener("pointerup", end);
      node.addEventListener("pointercancel", end);
    });
    node.addEventListener("dblclick", onReset);
  };
  drag($("split-sidebar"), "v", (event) => {
    const width = Math.round(Math.min(Math.max(event.clientX - 52, 180), 560));
    els.app.style.setProperty("--sidebar-w", `${width}px`);
    saveLayout({ sidebar: width });
  }, () => { els.app.style.removeProperty("--sidebar-w"); saveLayout({ sidebar: null }); });
  drag($("split-notebook"), "v", (event) => {
    const width = Math.round(Math.min(Math.max(window.innerWidth - event.clientX - 2, 240), 720));
    els.app.style.setProperty("--notebook-w", `${width}px`);
    saveLayout({ notebook: width });
  }, () => { els.app.style.removeProperty("--notebook-w"); saveLayout({ notebook: null }); });
  drag($("split-panel"), "h", (event) => {
    const rect = els.main.getBoundingClientRect();
    const height = Math.round(Math.min(Math.max(rect.bottom - event.clientY, 96), rect.height - 140));
    els.main.style.setProperty("--panel-h", `${height}px`);
    saveLayout({ panel: height });
  }, () => { els.main.style.removeProperty("--panel-h"); saveLayout({ panel: null }); });
}

function toggleSidebar(show = els.app.classList.contains("sidebar-hidden")) {
  els.app.classList.toggle("sidebar-hidden", !show);
  saveLayout({ sidebarHidden: !show });
  updateRail();
}

function toggleNotebook(show = els.app.classList.contains("notebook-hidden")) {
  els.app.classList.toggle("notebook-hidden", !show);
  saveLayout({ notebookHidden: !show });
  updateRail();
  if (show && !S.note && S.notes.length) openNote(S.notes[0].name);
}

function togglePanel(show = els.main.classList.contains("panel-hidden")) {
  els.main.classList.toggle("panel-hidden", !show);
  saveLayout({ panelHidden: !show });
  renderPanel();
}

// ---------- Title bar, toolbar, rail ----------

const toolbarButtons = {};

function buildToolbar() {
  toolbarButtons.newFile = button("New file", "plus", () => newFile(), { title: "Create a new Orange file" });
  toolbarButtons.save = button("Save", "save", () => saveActive(), { title: "Save this file (Ctrl+S)" });
  toolbarButtons.check = button("Check", "check", () => runActive("check"), { title: "Check this file with orangec" });
  toolbarButtons.evaluate = button("Evaluate", "play", () => runActive("eval"), { title: "Evaluate the typed specs with orangec" });
  toolbarButtons.cite = button("Cite in note", "cite", () => citeActive(), { title: "Put a link to this place into the open note" });
  els.toolbar.append(
    toolbarButtons.newFile, toolbarButtons.save,
    h("span", { class: "sep" }),
    toolbarButtons.check, toolbarButtons.evaluate,
    h("span", { class: "sep" }),
    toolbarButtons.cite,
  );
}

function updateToolbar() {
  const tab = S.active;
  const file = tab && tab.kind === "file";
  toolbarButtons.save.disabled = !(file && tab.dirty);
  toolbarButtons.check.disabled = !(file && S.session.orangec);
  toolbarButtons.evaluate.disabled = !(file && S.session.orangec);
  toolbarButtons.cite.disabled = !(tab && (tab.kind === "file" || tab.kind === "doc"));
  if (!S.session.orangec) {
    toolbarButtons.check.title = "orangec was not found; start Tabula with --orangec <path>";
    toolbarButtons.evaluate.title = toolbarButtons.check.title;
  }
}

function buildTitlebarEnd() {
  els.titlebarEnd.append(
    iconButton("sidebar", "Show or hide the side panel", () => toggleSidebar()),
    iconButton("panel", "Show or hide the results panel", () => togglePanel()),
  );
}

const RAIL = [
  { id: "files", icon: "files", title: "Files" },
  { id: "strata", icon: "strata", title: "Strata: the outline of this file" },
  { id: "library", icon: "book", title: "Library: The Orange Book and the manuals" },
];

function buildRail() {
  for (const item of RAIL) {
    els.rail.append(h("button", {
      class: "rail-btn",
      type: "button",
      title: item.title,
      "aria-label": item.title,
      dataset: { side: item.id },
      onclick: () => {
        if (S.side === item.id && !els.app.classList.contains("sidebar-hidden")) {
          toggleSidebar(false);
          return;
        }
        S.side = item.id;
        store.set("side", item.id);
        toggleSidebar(true);
        renderSidebar();
      },
    }, icon(item.icon)));
  }
  els.rail.append(h("button", {
    class: "rail-btn",
    type: "button",
    id: "notebook-toggle",
    title: "Notebook",
    "aria-label": "Notebook",
    onclick: () => toggleNotebook(),
  }, icon("notebook")));
  els.rail.append(h("div", { class: "rail-spacer" }));
  els.rail.append(h("button", { class: "rail-btn", type: "button", title: "Theme", "aria-label": "Theme", onclick: () => showThemes() }, icon("palette")));
  els.rail.append(h("button", { class: "rail-btn", type: "button", title: "About Tabula", "aria-label": "About Tabula", onclick: () => showAbout() }, icon("info")));
  updateRail();
}

function updateRail() {
  const hidden = els.app.classList.contains("sidebar-hidden");
  for (const node of els.rail.querySelectorAll("[data-side]")) {
    node.classList.toggle("active", !hidden && node.dataset.side === S.side);
  }
  const notebook = $("notebook-toggle");
  if (notebook) notebook.classList.toggle("active", !els.app.classList.contains("notebook-hidden"));
}

function sideHead(title, actions = []) {
  return h("div", { class: "side-head" },
    h("span", { class: "side-title", text: title }),
    h("div", { class: "side-actions" }, actions));
}

function renderSidebar() {
  updateRail();
  if (S.side === "strata") renderStrata();
  else if (S.side === "library") renderLibrary();
  else renderFiles();
}

// ---------- Files ----------

async function refreshTree() {
  try {
    S.tree = await api.tree();
    S.treeError = null;
  } catch (error) {
    S.treeError = error.message;
  }
  if (S.side === "files") renderFiles();
}

function renderFiles() {
  const body = h("div", { class: "side-body", role: "tree", "aria-label": "Workspace files" });
  els.sidebar.replaceChildren(
    sideHead(S.session.workspace, [
      iconButton("plus", "New Orange file", () => newFile()),
      iconButton("folder-plus", "New folder", () => newFolder()),
      iconButton("refresh", "Refresh the file list", () => refreshTree()),
    ]),
    body,
  );
  if (S.treeError) {
    body.append(h("div", { class: "side-note", text: `Could not list the workspace: ${S.treeError}` }));
    return;
  }
  if (!S.tree) return;
  if (!S.tree.children.length) {
    body.append(
      h("div", { class: "side-note", text: "This workspace has no Orange files yet. Tabula lists .or files and the folders that hold them." }),
      h("div", { class: "side-note" }, button("Create the first file", "plus", () => newFile(), { kind: "outline", small: true })),
    );
    return;
  }
  appendTreeRows(body, S.tree.children, 0);
  if (S.tree.truncated) body.append(h("div", { class: "side-note", text: "This workspace is very large; only part of it is listed." }));
}

function appendTreeRows(body, nodes, depth) {
  for (const node of nodes) {
    const row = h("div", { class: "tree-row", role: "treeitem", tabindex: "0", title: node.path });
    row.style.paddingLeft = `${10 + depth * 14}px`;
    if (node.kind === "folder") {
      const open = S.expanded.has(node.path);
      row.classList.toggle("collapsed", !open);
      row.setAttribute("aria-expanded", String(open));
      row.append(
        icon("chevron", "chev"),
        icon("folder"),
        h("span", { class: "label", text: node.name }),
        h("span", { class: "row-actions" },
          iconButton("plus", `New file in ${node.name}`, (event) => { event.stopPropagation(); newFile(node.path); }),
          iconButton("rename", `Rename ${node.name}`, (event) => { event.stopPropagation(); renameEntry(node); }),
          iconButton("trash", `Delete ${node.name}`, (event) => { event.stopPropagation(); deleteEntry(node); }),
        ),
      );
      row.addEventListener("click", () => {
        if (S.expanded.has(node.path)) S.expanded.delete(node.path);
        else S.expanded.add(node.path);
        S.selectedFolder = node.path;
        store.set(workspaceKey("expanded"), [...S.expanded]);
        renderFiles();
      });
      body.append(row);
      if (open) {
        if (node.children.length) appendTreeRows(body, node.children, depth + 1);
        else {
          const empty = h("div", { class: "side-note", text: "Empty folder" });
          empty.style.paddingLeft = `${34 + depth * 14}px`;
          body.append(empty);
        }
      }
    } else {
      const tab = S.tabs.find((candidate) => candidate.key === `file:${node.path}`);
      row.classList.toggle("active", !!(S.active && S.active === tab));
      row.append(
        h("span", { class: "chev-space" }),
        icon("orange", "file-glyph"),
        h("span", { class: "label", text: node.name }),
        tab && tab.dirty ? h("span", { class: "dirty-dot", title: "Unsaved changes" }) : "",
        h("span", { class: "row-actions" },
          iconButton("rename", `Rename ${node.name}`, (event) => { event.stopPropagation(); renameEntry(node); }),
          iconButton("trash", `Delete ${node.name}`, (event) => { event.stopPropagation(); deleteEntry(node); }),
        ),
      );
      row.addEventListener("click", () => {
        S.selectedFolder = dirName(node.path);
        openFile(node.path);
      });
      body.append(row);
    }
    row.addEventListener("keydown", (event) => {
      if (event.key === "Enter" && event.target === row) row.click();
    });
  }
}

function findNode(path, nodes = S.tree ? S.tree.children : []) {
  for (const node of nodes) {
    if (node.path === path) return node;
    if (node.kind === "folder") {
      const found = findNode(path, node.children);
      if (found) return found;
    }
  }
  return null;
}

function expandTo(path) {
  let folder = dirName(path);
  while (folder) {
    S.expanded.add(folder);
    folder = dirName(folder);
  }
  store.set(workspaceKey("expanded"), [...S.expanded]);
}

function checkName(name, { orange }) {
  if (!name) return "Enter a name.";
  if (name.includes("/") || name.includes("\\")) return "Use a single name without slashes.";
  if (name.startsWith(".")) return "Names cannot start with a dot.";
  if (name === "target" || name === "node_modules") return "Tabula skips folders with that name.";
  if (orange && /\.or$/i.test(name) && name.length === 3) return "Enter a name before .or.";
  return null;
}

function allFolders(nodes = S.tree ? S.tree.children : [], out = []) {
  for (const node of nodes) {
    if (node.kind !== "folder") continue;
    out.push(node.path);
    allFolders(node.children, out);
  }
  return out;
}

// Asks for a name and the folder to put it in.
function askPlace({ title, label, value = "", folder = "", confirm, orange }) {
  const input = h("input", { class: "field", type: "text", value, spellcheck: "false", autocomplete: "off", "aria-label": label });
  const folders = allFolders();
  const select = h("select", { class: "field", "aria-label": "Folder" },
    h("option", { value: "", text: `${S.session.workspace} (top level)` }),
    folders.map((path) => h("option", { value: path, text: path })));
  select.value = folders.includes(folder) ? folder : "";
  const content = h("div", {}, h("label", { text: label }), input, folders.length ? h("label", { text: "Folder" }) : null, folders.length ? select : null);
  return dialog({
    title,
    content,
    validate: (result) => checkName(result.name, { orange }),
    actions: [
      { label: "Cancel", value: null },
      { label: confirm, value: () => ({ name: input.value.trim(), folder: select.value }), kind: "primary" },
    ],
  });
}

function uniqueFileName(folder, stem) {
  const taken = new Set(((folder ? (findNode(folder) || { children: [] }).children : S.tree ? S.tree.children : [])).map((node) => node.name));
  if (!taken.has(`${stem}.or`)) return `${stem}.or`;
  for (let k = 2; ; k++) if (!taken.has(`${stem}_${k}.or`)) return `${stem}_${k}.or`;
}

async function newFile(where = S.selectedFolder, text = null) {
  const place = await askPlace({
    title: text === null ? "New Orange file" : "Copy the example into a new file",
    label: "File name",
    value: uniqueFileName(where, text === null ? "untitled" : "example"),
    folder: where,
    confirm: "Create",
    orange: true,
  });
  if (!place) return null;
  const file = /\.or$/i.test(place.name) ? place.name : `${place.name}.or`;
  const path = joinPath(place.folder, file);
  try {
    await api.create(path, text === null ? template(file) : text);
  } catch (error) {
    toast(`Could not create ${file}: ${error.message}`, true);
    return null;
  }
  expandTo(path);
  await refreshTree();
  return openFile(path);
}

async function newFolder(parent = S.selectedFolder) {
  const place = await askPlace({ title: "New folder", label: "Folder name", folder: parent, confirm: "Create", orange: false });
  if (!place) return;
  const name = place.name;
  const path = joinPath(place.folder, name);
  try {
    await api.createFolder(path);
  } catch (error) {
    toast(`Could not create ${name}: ${error.message}`, true);
    return;
  }
  S.expanded.add(path);
  expandTo(path);
  S.selectedFolder = path;
  await refreshTree();
}

async function renameEntry(node) {
  const folder = node.kind === "folder";
  const name = await ask({
    title: `Rename ${node.name}`,
    label: folder ? "New folder name" : "New file name",
    value: node.name,
    confirm: "Rename",
    validate: (value) => checkName(value, { orange: !folder }),
  });
  if (!name || name === node.name) return;
  const finalName = folder || /\.or$/i.test(name) ? name : `${name}.or`;
  const to = joinPath(dirName(node.path), finalName);
  try {
    await api.rename(node.path, to);
  } catch (error) {
    toast(`Could not rename ${node.name}: ${error.message}`, true);
    return;
  }
  for (const tab of S.tabs) {
    if (tab.kind !== "file") continue;
    if (tab.path === node.path || tab.path.startsWith(`${node.path}/`)) {
      tab.path = to + tab.path.slice(node.path.length);
      tab.key = `file:${tab.path}`;
      tab.title = baseName(tab.path);
    }
  }
  if (folder && S.expanded.has(node.path)) {
    S.expanded.delete(node.path);
    S.expanded.add(to);
  }
  expandTo(to);
  renderTabs();
  persistTabs();
  await refreshTree();
  toast(`Renamed to ${finalName}`);
}

async function deleteEntry(node) {
  const affected = S.tabs.filter((tab) => tab.kind === "file" && (tab.path === node.path || tab.path.startsWith(`${node.path}/`)));
  const unsaved = affected.some((tab) => tab.dirty);
  const yes = await confirmDialog({
    title: `Delete ${node.name}?`,
    message: `Tabula moves ${node.kind === "folder" ? "the folder and everything in it" : "the file"} to .tabula/trash in this workspace, so it can be recovered from there.${unsaved ? " Unsaved changes in open tabs will be lost." : ""}`,
    confirm: "Move to trash",
    danger: true,
  });
  if (!yes) return;
  try {
    await api.remove(node.path);
  } catch (error) {
    toast(`Could not delete ${node.name}: ${error.message}`, true);
    return;
  }
  for (const tab of affected) {
    tab.dirty = false;
    await closeTab(tab);
  }
  await refreshTree();
  toast(`Moved ${node.name} to .tabula/trash`);
}

// ---------- Tabs ----------

function registerTab(tab) {
  S.tabs.push(tab);
  els.views.append(tab.view);
  return tab;
}

function activate(tab) {
  if (!tab) return;
  S.active = tab;
  els.main.classList.toggle("reading", tab.kind !== "file");
  for (const candidate of S.tabs) candidate.view.classList.toggle("active", candidate === tab);
  renderTabs();
  updateToolbar();
  renderSidebar();
  renderPanel();
  renderStatus();
  if (tab.kind === "file") {
    tab.editor.render();
    requestAnimationFrame(() => tab.editor.focus());
  }
  persistTabs();
}

function renderTabs() {
  clear(els.tabs);
  for (const tab of S.tabs) {
    const isActive = tab === S.active;
    const node = h("div", {
      class: `tab kind-${tab.kind}${isActive ? " active" : ""}${tab.dirty ? " dirty" : ""}`,
      role: "tab",
      tabindex: "0",
      "aria-selected": String(isActive),
      title: tab.path || tab.title,
      onclick: () => activate(tab),
      onauxclick: (event) => { if (event.button === 1) { event.preventDefault(); closeTab(tab); } },
    },
      icon(tab.kind === "doc" ? "book" : tab.kind === "file" ? "orange" : "bookmark", "tab-icon"),
      h("span", { class: "tab-label", text: tab.title }),
      h("span", {
        class: "tab-close",
        role: "button",
        title: tab.dirty ? "Close (unsaved changes)" : "Close",
        "aria-label": `Close ${tab.title}`,
        onclick: (event) => { event.stopPropagation(); closeTab(tab); },
      }, icon("close")),
    );
    els.tabs.append(node);
    if (isActive) requestAnimationFrame(() => node.scrollIntoView({ block: "nearest", inline: "nearest" }));
  }
  if (S.side === "files") renderFiles();
}

async function closeTab(tab) {
  if (tab.kind === "file" && tab.dirty) {
    const choice = await dialog({
      title: `Save changes to ${tab.title}?`,
      content: "Your changes are lost if you close without saving.",
      actions: [
        { label: "Cancel", value: null },
        { label: "Close without saving", value: "discard", kind: "danger" },
        { label: "Save", value: "save", kind: "primary" },
      ],
    });
    if (!choice) return false;
    if (choice === "save" && !(await saveTab(tab))) return false;
  }
  const index = S.tabs.indexOf(tab);
  if (index === -1) return true;
  S.tabs.splice(index, 1);
  clearTimeout(tab.runTimer);
  clearTimeout(tab.outlineTimer);
  for (const url of tab.objectUrls || []) URL.revokeObjectURL(url);
  tab.view.remove();
  if (S.active === tab) {
    S.active = null;
    const next = S.tabs[Math.min(index, S.tabs.length - 1)];
    if (next) activate(next);
    else openWelcome();
  } else {
    renderTabs();
  }
  persistTabs();
  return true;
}

function persistTabs() {
  if (!S.session) return;
  const open = S.tabs.filter((tab) => tab.kind !== "welcome").map((tab) => ({ kind: tab.kind, path: tab.path }));
  store.set(workspaceKey("tabs"), { open, active: S.active ? S.active.key : null });
}

async function restoreTabs() {
  const saved = store.get(workspaceKey("tabs"), null);
  if (!saved || !saved.open.length) {
    openWelcome();
    return;
  }
  for (const entry of saved.open.slice(0, 20)) {
    if (entry.kind === "file") await openFile(entry.path, { quiet: true, activate: false });
    else if (entry.kind === "doc" && S.session.library) await openDoc(entry.path, { quiet: true, activate: false });
  }
  const active = S.tabs.find((tab) => tab.key === saved.active) || S.tabs[0];
  if (active) activate(active);
  else openWelcome();
}

// ---------- Welcome ----------

function openWelcome() {
  const existing = S.tabs.find((tab) => tab.kind === "welcome");
  if (existing) { activate(existing); return; }
  const view = h("div", { class: "view" });
  const tab = registerTab({ id: ++tabSequence, kind: "welcome", key: "welcome", title: "Welcome", view });
  renderWelcome(tab);
  activate(tab);
}

function renderWelcome(tab) {
  const session = S.session;
  const recent = store.get(workspaceKey("recent"), []).filter((path) => findNode(path)).slice(0, 6);
  const card = (iconName, title, text, onclick) => h("button", { class: "welcome-card", type: "button", onclick }, icon(iconName), h("b", { text: title }), h("span", { text }));
  const bookDoc = S.catalog && S.catalog.sections[0] && S.catalog.sections[0].documents[0];
  tab.view.replaceChildren(h("div", { class: "welcome" },
    h("div", { class: "welcome-inner" },
      h("img", { class: "welcome-mark", src: "/assets/tabula.svg", alt: "" }),
      h("h1", { text: "Tabula" }),
      h("p", { class: "lede", text: "A quiet table for writing Orange. Keep the specification and the implementation in one file, let orangec check them as you type, and keep The Orange Book, the manuals, and your notes beside the code." }),
      h("div", { class: "welcome-actions" },
        card("plus", "New Orange file", "Start from a small module with one typed spec.", () => newFile()),
        card("book", "Read The Orange Book", session.library ? "The book and manuals, read from this checkout." : "Needs an Orange checkout; see About.", () => {
          if (bookDoc) openDoc(bookDoc.path);
          else showAbout();
        }),
        card("notebook", "Open the notebook", "Markdown notes that can cite code and pages.", () => toggleNotebook(true)),
      ),
      recent.length ? h("div", { class: "welcome-recent" },
        h("h2", { text: "Recent files" }),
        recent.map((path) => h("div", { class: "tree-row", role: "button", tabindex: "0", onclick: () => openFile(path), title: path },
          icon("orange", "file-glyph"), h("span", { class: "label", text: path }))),
      ) : null,
      h("h2", { text: "This session" }),
      h("dl", { class: "welcome-meta" },
        h("dt", { text: "Workspace" }), h("dd", { text: session.workspaceRoot }),
        h("dt", { text: "Compiler" }), h("dd", { text: session.orangec ? `${session.orangec.version} · ${session.orangec.path}` : "not found (start Tabula with --orangec <path>)" }),
        h("dt", { text: "Library" }), h("dd", { text: session.library ? session.library.root : "not found (start Tabula with --library <checkout>)" }),
        h("dt", { text: "Notes" }), h("dd", { text: `${session.workspaceRoot}/.tabula/notes` }),
      ),
    )));
}

// ---------- Orange files ----------

function rememberRecent(path) {
  const recent = store.get(workspaceKey("recent"), []).filter((item) => item !== path);
  recent.unshift(path);
  store.set(workspaceKey("recent"), recent.slice(0, 12));
}

async function openFile(path, { reveal = null, quiet = false, activate: focus = true } = {}) {
  const key = `file:${path}`;
  let tab = S.tabs.find((candidate) => candidate.key === key);
  if (!tab) {
    if (!S.opening.has(key)) {
      S.opening.set(key, api.read(path).then((data) => createFileTab(path, data.text)).finally(() => S.opening.delete(key)));
    }
    try {
      tab = await S.opening.get(key) || S.tabs.find((candidate) => candidate.key === key);
    } catch (error) {
      if (!quiet) toast(`Could not open ${path}: ${error.message}`, true);
      return null;
    }
  }
  if (!tab) return null;
  rememberRecent(path);
  if (focus) activate(tab);
  if (reveal) revealAnchor(tab, reveal);
  return tab;
}

function createFileTab(path, text) {
  const existing = S.tabs.find((candidate) => candidate.key === `file:${path}`);
  if (existing) return existing;
  const view = h("div", { class: "view" });
  const eol = text.includes("\r\n") ? "\r\n" : "\n";
  const tab = registerTab({
    id: ++tabSequence,
    kind: "file",
    key: `file:${path}`,
    path,
    title: baseName(path),
    view,
    eol,
    savedText: null,
    dirty: false,
    diagnostics: [],
    result: null,
    values: null,
    valuesSource: null,
    tokens: null,
    running: false,
    seq: 0,
    outline: null,
    hostError: null,
    cursor: { line: 1, column: 1, selection: 0 },
  });
  tab.editor = new Editor(view, {
    label: `Orange source ${path}`,
    onChange: () => onEdit(tab),
    onCursor: (position) => {
      tab.cursor = position;
      if (tab === S.active) renderCursor();
    },
    onSave: () => saveTab(tab),
    onNoteMark: (line, names) => {
      if (!names.length) return;
      toggleNotebook(true);
      openNote(names[0]);
    },
  });
  tab.editor.setValue(text);
  tab.savedText = tab.editor.getValue();
  refreshOutline(tab);
  if (S.session.orangec && S.live) scheduleRun(tab, 30);
  return tab;
}

function onEdit(tab) {
  const dirty = tab.editor.getValue() !== tab.savedText;
  if (dirty !== tab.dirty) {
    tab.dirty = dirty;
    renderTabs();
    updateToolbar();
  }
  clearTimeout(tab.outlineTimer);
  tab.outlineTimer = setTimeout(() => refreshOutline(tab), tab.editor.getValue().length > 200_000 ? 800 : 140);
  if (S.live && S.session.orangec) scheduleRun(tab, 420);
  else if (tab === S.active) renderStatus();
}

function refreshOutline(tab) {
  const text = tab.editor.getValue();
  tab.outline = outline(text, classify(text, tokenize(text)));
  tab.starts = lineStarts(text);
  updateInlays(tab);
  applyNoteMarks(tab);
  if (tab === S.active && S.side === "strata") renderStrata();
}

async function saveTab(tab) {
  if (!tab || tab.kind !== "file") return true;
  const text = tab.editor.getValue();
  try {
    await api.save(tab.path, diskTextOf(tab, text));
  } catch (error) {
    toast(`Could not save ${tab.title}: ${error.message}`, true);
    return false;
  }
  tab.savedText = text;
  tab.dirty = tab.editor.getValue() !== text;
  if (tab.result && tab.result.source === text) tab.result.saved = true;
  renderTabs();
  updateToolbar();
  renderStatus();
  if (tab === S.active && S.panelTab === "evidence") renderPanel();
  toast(`Saved ${tab.title}`);
  return true;
}

function saveActive() {
  if (S.active && S.active.kind === "file") saveTab(S.active);
}

// ---------- Running orangec ----------

function scheduleRun(tab, delay) {
  clearTimeout(tab.runTimer);
  tab.runTimer = setTimeout(() => runTab(tab, "eval", { quiet: true }), delay);
}

function runActive(action) {
  const tab = S.active;
  if (!tab || tab.kind !== "file") return;
  S.panelTab = action === "eval" ? "values" : "problems";
  store.set("panelTab", S.panelTab);
  togglePanel(true);
  runTab(tab, action);
}

async function runTab(tab, action, { quiet = false } = {}) {
  if (!S.session.orangec) {
    if (!quiet) toast("orangec was not found. Start Tabula with --orangec <path>.", true);
    return;
  }
  clearTimeout(tab.runTimer);
  const source = tab.editor.getValue();
  const seq = ++tab.seq;
  tab.running = true;
  if (tab === S.active) renderStatus();
  let response;
  try {
    response = action === "check" ? await api.check(diskTextOf(tab, source)) : await api.evaluate(diskTextOf(tab, source));
  } catch (error) {
    if (seq !== tab.seq) return;
    tab.running = false;
    tab.hostError = error.message;
    if (tab === S.active) { renderStatus(); renderPanel(); }
    if (!quiet) toast(error.message, true);
    return;
  }
  if (seq !== tab.seq || !S.tabs.includes(tab)) return;
  tab.running = false;
  applyResult(tab, action, source, response);
}

function diskTextOf(tab, text) {
  return tab.eol === "\r\n" ? text.replace(/\n/g, "\r\n") : text;
}

function mapDiagnostic(diagnostic, source, starts) {
  if (diagnostic.line === null || diagnostic.line === undefined) return { ...diagnostic, from: null, to: null, related: [] };
  const from = offsetAt(source, starts, diagnostic.line, diagnostic.column || 1);
  const to = advanceScalars(source, from, Math.max(diagnostic.width || 1, 1));
  const related = (diagnostic.related || []).map((item) => {
    const start = offsetAt(source, starts, item.line, item.column || 1);
    return { ...item, from: start, to: advanceScalars(source, start, Math.max(item.width || 1, 1)) };
  });
  return { ...diagnostic, from, to, related };
}

function applyResult(tab, action, source, response) {
  tab.hostError = null;
  const starts = lineStarts(source);
  tab.diagnostics = response.diagnostics.map((diagnostic) => mapDiagnostic(diagnostic, source, starts));
  const located = tab.diagnostics.filter((diagnostic) => diagnostic.from !== null);
  tab.editor.setDiagnostics(located, located.flatMap((diagnostic) => diagnostic.related));
  if (action === "eval" && response.success) {
    tab.values = response.values;
    tab.valuesSource = source;
  }
  const result = { action, source, response, at: new Date(), sha256: null, saved: source === tab.savedText, path: tab.path, bytes: utf8Length(diskTextOf(tab, source)) };
  tab.result = result;
  sha256Hex(diskTextOf(tab, source)).then((hex) => {
    result.sha256 = hex || "unavailable in this browser";
    if (tab === S.active && S.panelTab === "evidence") renderPanel();
  }).catch(() => { result.sha256 = "unavailable in this browser"; });
  updateInlays(tab);
  if (tab === S.active) {
    renderPanel();
    renderStatus();
    if (S.side === "strata") renderStrata();
  }
}

function currentValues(tab) {
  if (!tab.values) return new Map();
  const map = new Map();
  for (const value of tab.values) map.set(`${value.module}::${value.name}`, value);
  return map;
}

function showValue(value, radix = S.radix) {
  const number = parseValue(value.type, value.value);
  if (number === null) return value.value;
  return formatValue(number, radix, wordWidth(value.type));
}

function updateInlays(tab) {
  const inlays = new Map();
  if (tab.values && tab.outline && tab.outline.module) {
    const values = currentValues(tab);
    const stale = tab.valuesSource !== tab.editor.getValue();
    const starts = tab.starts || lineStarts(tab.editor.getValue());
    for (const decl of tab.outline.decls) {
      if (decl.stratum !== "spec") continue;
      const value = values.get(`${tab.outline.module}::${decl.name}`);
      if (!value) continue;
      inlays.set(lineOf(starts, decl.end), { label: stale ? "was" : "=", value: showValue(value) });
    }
  }
  tab.editor.setInlays(inlays);
}

// ---------- Citations and anchors ----------

function declAt(tab, offset) {
  if (!tab.outline) return null;
  return tab.outline.decls.find((decl) => offset >= decl.start && offset <= decl.end) || null;
}

function revealAnchor(tab, anchor) {
  if (!anchor || tab.kind !== "file") return;
  const text = tab.editor.getValue();
  const starts = lineStarts(text);
  const line = /^L(\d+)$/.exec(anchor);
  if (line) {
    const index = Math.min(Math.max(Number(line[1]) - 1, 0), starts.length - 1);
    const end = index + 1 < starts.length ? starts[index + 1] - 1 : text.length;
    tab.editor.reveal(starts[index], Math.max(end, starts[index]));
    return;
  }
  const [stratum, name] = anchor.includes(".") ? anchor.split(".", 2) : [null, anchor];
  const decl = (tab.outline ? tab.outline.decls : []).find((item) => item.name === name && (!stratum || item.stratum === stratum));
  if (decl) tab.editor.reveal(decl.nameStart, decl.nameEnd);
  else toast(`${anchor} is no longer in ${tab.title}`, true);
}

function citationForFile(tab) {
  const editor = tab.editor;
  const start = editor.input.selectionStart;
  const end = editor.input.selectionEnd;
  const decl = declAt(tab, start);
  const selected = editor.getValue().slice(start, end);
  let link;
  if (decl && (start === end || end <= decl.end)) {
    link = `[${decl.stratum} ${decl.name}](tabula:${tab.path}#${decl.stratum}.${decl.name})`;
  } else {
    const line = lineOf(tab.starts || lineStarts(editor.getValue()), start) + 1;
    link = `[${tab.title} line ${line}](tabula:${tab.path}#L${line})`;
  }
  if (selected.trim() && selected.length <= 4000) {
    return `${link}\n\n\`\`\`orange\n${selected.replace(/\n+$/, "")}\n\`\`\`\n`;
  }
  return link;
}

function citationForDoc(tab) {
  const heading = currentHeading(tab);
  const anchor = heading ? `#${heading.id}` : "";
  const label = heading && heading.text !== tab.title ? `${tab.title} › ${heading.text}` : tab.title;
  return `[${label}](library:${tab.path}${anchor})`;
}

async function citeActive() {
  const tab = S.active;
  if (!tab) return;
  const text = tab.kind === "file" ? citationForFile(tab) : tab.kind === "doc" ? citationForDoc(tab) : null;
  if (text) await insertIntoNote(text);
}

function applyNoteMarks(tab) {
  if (tab.kind !== "file") return;
  const marks = new Map();
  const starts = tab.starts || lineStarts(tab.editor.getValue());
  for (const note of S.notes) {
    for (const citation of note.citations) {
      if (citation.path !== tab.path) continue;
      let line = null;
      const byLine = /^L(\d+)$/.exec(citation.anchor);
      if (byLine) line = Number(byLine[1]) - 1;
      else if (citation.anchor && tab.outline) {
        const [stratum, name] = citation.anchor.includes(".") ? citation.anchor.split(".", 2) : [null, citation.anchor];
        const decl = tab.outline.decls.find((item) => item.name === name && (!stratum || item.stratum === stratum));
        if (decl) line = lineOf(starts, decl.start);
      }
      if (line === null || line < 0 || line >= starts.length) continue;
      const names = marks.get(line) || [];
      if (!names.includes(note.name)) names.push(note.name);
      marks.set(line, names);
    }
  }
  tab.editor.setNoteLines(marks);
}

// ---------- Strata ----------

function renderStrata() {
  const tab = S.active && S.active.kind === "file" ? S.active : null;
  const body = h("div", { class: "side-body" });
  els.sidebar.replaceChildren(sideHead("Strata"), body);
  if (!tab) {
    body.append(h("div", { class: "side-note", text: "Open an Orange file to see its strata: the specification, the implementation, and the layers Orange reserves for games, proofs, and claims." }));
    return;
  }
  const info = tab.outline || { module: null, edition: null, decls: [] };
  body.append(info.module
    ? h("div", { class: "module-line" }, "module ", h("b", { text: info.module }), info.edition ? ` · edition ${info.edition}` : "")
    : h("div", { class: "module-line", text: "No module yet. Every Orange file declares one." }));
  const values = currentValues(tab);
  const stale = tab.valuesSource !== tab.editor.getValue();
  for (const stratum of STRATA) {
    const meta = STRATUM_INFO[stratum];
    const decls = info.decls.filter((decl) => decl.stratum === stratum);
    const section = h("div", { class: `stratum-${stratum}${meta.live ? "" : " stratum-reserved"}` },
      h("div", { class: "stratum-head" },
        h("span", { class: "swatch" }),
        h("span", { text: meta.title }),
        h("span", { class: "hint", text: meta.hint }),
        h("span", { class: "count", text: String(decls.length) })));
    for (const decl of decls) {
      const value = stratum === "spec" && info.module ? values.get(`${info.module}::${decl.name}`) : null;
      const row = h("div", { class: "tree-row decl-row", role: "button", tabindex: "0", title: `Go to ${decl.stratum} ${decl.name}`, onclick: () => tab.editor.reveal(decl.nameStart, decl.nameEnd) },
        h("span", { class: "label" }, decl.name, decl.type ? h("span", { class: "decl-type", text: ` → ${decl.type}` }) : null),
        value ? h("span", { class: "decl-value", title: stale ? "Value before your latest edit" : "Value from orangec eval", text: showValue(value) }) : null,
        h("span", { class: "row-actions" },
          iconButton("cite", `Cite ${decl.name} in the open note`, (event) => {
            event.stopPropagation();
            insertIntoNote(`[${decl.stratum} ${decl.name}](tabula:${tab.path}#${decl.stratum}.${decl.name})`);
          })));
      row.style.paddingLeft = "28px";
      section.append(row);
    }
    if (!decls.length && meta.live) {
      section.append(h("div", { class: "side-note", text: stratum === "spec" ? "No specs yet. A spec states what must be true." : "No implementations yet." }));
    }
    body.append(section);
  }
  body.append(h("div", { class: "side-section", text: "Insert" }));
  body.append(h("div", { class: "insert-grid" },
    insertButton(tab, "spec", "-> Int", (name) => `spec ${name}() -> Int { 0 }`),
    insertButton(tab, "spec", "-> Word[8]", (name) => `spec ${name}() -> Word[8] { 0x00 }`),
    insertButton(tab, "spec", "{}", (name) => `spec ${name}() {}`),
    insertButton(tab, "impl", "{}", (name) => `impl ${name}() {}`),
  ));
  body.append(h("div", { class: "side-note", text: "Orange 2026 accepts empty spec and impl functions and typed literal specs. The words game, proof, and claim are reserved for later layers; orangec rejects them today." }));
}

function insertButton(tab, stratum, suffix, make) {
  const node = h("button", { class: "btn small", type: "button", title: `Insert a ${stratum} function`, onclick: () => insertDecl(tab, stratum, make) },
    h("span", { class: `kw t-${stratum}`, text: stratum }),
    h("span", { class: "kw", text: suffix }));
  return node;
}

// Finds the offsets of the module's `{` and its matching `}`.
function moduleBraces(text) {
  const tokens = tokenize(text).filter((token) => token.kind !== "ws" && token.kind !== "comment");
  const at = tokens.findIndex((token) => token.kind === "keyword" && text.slice(token.start, token.end) === "module");
  if (at === -1) return null;
  let k = at + 1;
  while (k < tokens.length && text.slice(tokens[k].start, tokens[k].end) !== "{") k++;
  if (k >= tokens.length) return null;
  const open = tokens[k].start;
  let depth = 0;
  for (; k < tokens.length; k++) {
    const word = text.slice(tokens[k].start, tokens[k].end);
    if (word === "{") depth++;
    else if (word === "}" && --depth === 0) return { open, close: tokens[k].start };
  }
  return { open, close: null };
}

function insertDecl(tab, stratum, make) {
  const text = tab.editor.getValue();
  const taken = new Set((tab.outline ? tab.outline.decls : []).filter((decl) => decl.stratum === stratum).map((decl) => decl.name));
  const stem = stratum === "impl" ? "implementation" : "value";
  let name = stem;
  for (let k = 2; taken.has(name); k++) name = `${stem}_${k}`;
  const decl = make(name);
  const braces = moduleBraces(text);
  let at;
  let insertion;
  if (!braces) {
    at = text.length;
    const prefix = text && !text.endsWith("\n") ? "\n" : "";
    const header = /\bedition\b/.test(text) ? "" : "edition 2026;\n\n";
    insertion = `${prefix}${header}module main {\n  ${decl}\n}\n`;
  } else if (braces.close === null) {
    at = text.length;
    insertion = `\n  ${decl}\n}\n`;
  } else {
    const cursorDecl = declAt(tab, tab.editor.input.selectionStart);
    if (cursorDecl && cursorDecl.end < braces.close) {
      const lineEnd = text.indexOf("\n", cursorDecl.end);
      at = lineEnd === -1 ? text.length : lineEnd;
      const lineStart = text.lastIndexOf("\n", cursorDecl.start - 1) + 1;
      const indent = text.slice(lineStart).match(/^[ \t]*/)[0];
      insertion = `\n${indent}${decl}`;
    } else {
      const lineStart = text.lastIndexOf("\n", braces.close - 1) + 1;
      if (/^[ \t]*$/.test(text.slice(lineStart, braces.close)) && lineStart > braces.open) {
        at = lineStart;
        insertion = `  ${decl}\n`;
      } else {
        at = braces.close;
        insertion = `\n  ${decl}\n`;
      }
    }
  }
  tab.editor.replace(at, at, insertion);
  const nameStart = at + insertion.indexOf(`${stratum} ${name}`) + stratum.length + 1;
  tab.editor.reveal(nameStart, nameStart + name.length, { flash: true });
  refreshOutline(tab);
}

// ---------- Library ----------

async function loadCatalog() {
  if (!S.session.library) return;
  try {
    S.catalog = await api.library();
    S.catalogError = null;
  } catch (error) {
    S.catalogError = error.message;
  }
  const welcome = S.tabs.find((tab) => tab.kind === "welcome");
  if (welcome) renderWelcome(welcome);
}

function catalogTitle(path) {
  if (!S.catalog) return null;
  for (const section of S.catalog.sections) {
    const doc = section.documents.find((item) => item.path === path);
    if (doc) return doc.title;
  }
  return null;
}

let searchTimer = 0;
let searchSequence = 0;

function renderLibrary() {
  const head = sideHead("Library", [iconButton("refresh", "Reload the Library", async () => { await loadCatalog(); renderLibrary(); })]);
  if (!S.session.library) {
    els.sidebar.replaceChildren(head, h("div", { class: "side-body" },
      h("div", { class: "side-note", text: "No Orange checkout was found, so The Orange Book and the manuals are not available. Start Tabula inside an Orange checkout, or pass --library <checkout>." })));
    return;
  }
  const input = h("input", {
    class: "field",
    type: "search",
    placeholder: "Search the book and manuals",
    value: S.libraryQuery,
    spellcheck: "false",
    "aria-label": "Search the Library",
  });
  const results = h("div", { class: "side-body" });
  input.addEventListener("input", () => {
    S.libraryQuery = input.value;
    clearTimeout(searchTimer);
    searchTimer = setTimeout(() => runSearch(results), 220);
  });
  els.sidebar.replaceChildren(head, h("div", { class: "lib-search" }, input), results);
  renderLibraryBody(results);
}

async function runSearch(container) {
  const query = S.libraryQuery.trim();
  const sequence = ++searchSequence;
  if (!query) {
    S.searchResults = null;
    renderLibraryBody(container);
    return;
  }
  try {
    const found = await api.search(query);
    if (sequence !== searchSequence) return;
    S.searchResults = found;
  } catch (error) {
    if (sequence !== searchSequence) return;
    S.searchResults = { query, results: [], error: error.message };
  }
  renderLibraryBody(container);
}

function renderLibraryBody(container) {
  clear(container);
  if (S.libraryQuery.trim() && S.searchResults) {
    const { results, truncated, error } = S.searchResults;
    if (error) container.append(h("div", { class: "side-note", text: error }));
    else if (!results.length) container.append(h("div", { class: "side-note", text: `Nothing matches “${S.searchResults.query}”.` }));
    for (const hit of results) {
      container.append(h("button", {
        class: "search-hit",
        type: "button",
        onclick: () => openDoc(hit.path, { heading: hit.heading, find: S.searchResults.query }),
      },
        h("small", { text: hit.heading ? `${hit.title} › ${hit.heading}` : hit.title }),
        h("span", {}, markQuery(hit.snippet, S.searchResults.query))));
    }
    if (truncated) container.append(h("div", { class: "side-note", text: "Showing the first matches; refine the search to see more." }));
    return;
  }
  if (S.catalogError) container.append(h("div", { class: "side-note", text: S.catalogError }));
  const doc = S.active && S.active.kind === "doc" ? S.active : null;
  if (doc && doc.headings && doc.headings.length > 1) {
    container.append(h("div", { class: "side-section", text: "On this page" }));
    const toc = h("div", { class: "lib-toc" });
    for (const heading of doc.headings.filter((item) => item.level <= 3)) {
      toc.append(h("div", {
        class: `tree-row depth-${heading.level}`,
        role: "button",
        tabindex: "0",
        dataset: { id: heading.id },
        title: heading.text,
        onclick: () => scrollToHeading(doc, heading.id),
      }, h("span", { class: "label", text: heading.text })));
    }
    container.append(toc);
    markCurrentHeading(doc);
  }
  if (!S.catalog) return;
  for (const section of S.catalog.sections) {
    const collapsed = S.collapsedSections.has(section.id);
    const header = h("div", { class: `tree-row lib-section${collapsed ? " collapsed" : ""}`, role: "button", tabindex: "0" },
      icon("chevron", "chev"), h("span", { class: "label", text: section.title }), h("span", { class: "badge", text: String(section.documents.length) }));
    header.addEventListener("click", () => {
      if (collapsed) S.collapsedSections.delete(section.id);
      else S.collapsedSections.add(section.id);
      store.set("collapsedSections", [...S.collapsedSections]);
      renderLibraryBody(container);
    });
    container.append(header);
    if (collapsed) continue;
    for (const item of section.documents) {
      const row = h("div", {
        class: `tree-row lib-doc${section.id === "book" ? " book" : ""}${doc && doc.path === item.path ? " active" : ""}`,
        role: "button",
        tabindex: "0",
        title: item.path,
        onclick: () => openDoc(item.path),
      }, icon(section.id === "book" ? "book" : "file"), h("span", { class: "label", text: item.title }));
      row.style.paddingLeft = "26px";
      container.append(row);
    }
  }
}

function markQuery(text, query) {
  const fragment = document.createDocumentFragment();
  const lower = text.toLowerCase();
  const needle = query.toLowerCase();
  let at = 0;
  for (let found = lower.indexOf(needle); needle && found !== -1; found = lower.indexOf(needle, at)) {
    fragment.append(text.slice(at, found), h("mark", { text: text.slice(found, found + needle.length) }));
    at = found + needle.length;
  }
  fragment.append(text.slice(at));
  return fragment;
}

function resolvePath(base, relative) {
  const parts = relative.startsWith("/") ? [] : base.split("/").filter(Boolean);
  for (const segment of relative.split("/")) {
    if (!segment || segment === ".") continue;
    if (segment === "..") {
      if (!parts.length) return null;
      parts.pop();
    } else {
      parts.push(segment);
    }
  }
  return parts.join("/");
}

function splitAnchor(target) {
  const hash = target.indexOf("#");
  return hash === -1 ? [target, ""] : [target.slice(0, hash), target.slice(hash + 1)];
}

function safeDecode(text) {
  try { return decodeURIComponent(text); } catch { return text; }
}

// Decides what a link in a document or note points at. `basePath` is the
// Library path of the document (null for notes).
function classifyLink(href, basePath) {
  if (/^(https?:|mailto:)/i.test(href)) return { kind: "external" };
  if (href.startsWith("tabula:")) {
    const [path, anchor] = splitAnchor(href.slice(7));
    return path ? { kind: "cite", path, anchor, title: `Open ${path}${anchor ? ` at ${anchor}` : ""}` } : { kind: "none" };
  }
  if (href.startsWith("library:")) {
    const [path, anchor] = splitAnchor(href.slice(8));
    return path ? { kind: "doc", path: safeDecode(path), anchor: safeDecode(anchor), title: path } : { kind: "none" };
  }
  if (href.startsWith("#")) return basePath === null ? { kind: "none" } : { kind: "anchor", id: safeDecode(href.slice(1)) };
  if (/^[a-z][a-z0-9+.-]*:/i.test(href) || href.startsWith("//")) return { kind: "none" };
  if (basePath === null) {
    const [path, anchor] = splitAnchor(href);
    return /\.or$/i.test(path) ? { kind: "cite", path: safeDecode(path), anchor } : { kind: "none" };
  }
  const [raw, anchor] = splitAnchor(href);
  let path = resolvePath(dirName(basePath), safeDecode(raw));
  if (path === null) return { kind: "none" };
  if (!path || raw.endsWith("/")) path = joinPath(path, "README.md");
  else if (!/\.[A-Za-z0-9]+$/.test(baseName(path))) path = joinPath(path, "README.md");
  if (/\.(png|jpe?g|gif|webp|svg)$/i.test(path)) return { kind: "none" };
  return { kind: "doc", path, anchor: safeDecode(anchor), title: path };
}

function followLink(target, fromTab) {
  if (target.kind === "anchor" && fromTab) scrollToHeading(fromTab, target.id);
  else if (target.kind === "doc") openDoc(target.path, { anchor: target.anchor });
  else if (target.kind === "cite") openFile(target.path, { reveal: target.anchor || null });
}

function docContext(tab) {
  return {
    link: (href) => classifyLink(href, tab ? tab.path : null),
    onLink: (target) => followLink(target, tab),
    image: (src, img) => {
      if (!tab || /^[a-z][a-z0-9+.-]*:/i.test(src) || src.startsWith("//")) return;
      const path = resolvePath(dirName(tab.path), safeDecode(splitAnchor(src)[0]));
      if (!path) return;
      loadAsset(path).then((url) => {
        (tab.objectUrls || (tab.objectUrls = [])).push(url);
        img.src = url;
      }).catch(() => img.classList.add("missing"));
    },
    code: (lang, text) => codeBlock(lang, text),
  };
}

function codeBlock(lang, text) {
  const orange = lang === "orange" || lang === "or";
  const code = h("code");
  if (orange) code.append(highlight(text));
  else code.textContent = text;
  const pre = h("pre", { class: "has-tools" }, code);
  if (lang) pre.dataset.lang = lang;
  pre.append(h("div", { class: "code-tools" },
    lang ? h("span", { class: "code-lang", text: lang }) : null,
    orange ? button("Open as a new file", "external", () => newFile(S.selectedFolder, text.endsWith("\n") ? text : `${text}\n`), { small: true }) : null,
    button("Copy", "copy", () => copyText(text), { small: true })));
  return pre;
}

async function openDoc(path, { anchor = "", heading = "", find = "", quiet = false, activate: focus = true } = {}) {
  const key = `doc:${path}`;
  let tab = S.tabs.find((candidate) => candidate.key === key);
  if (!tab) {
    if (!S.opening.has(key)) {
      S.opening.set(key, api.doc(path).then((data) => createDocTab(path, data.text)).finally(() => S.opening.delete(key)));
    }
    try {
      tab = await S.opening.get(key) || S.tabs.find((candidate) => candidate.key === key);
    } catch (error) {
      if (!quiet) toast(`Could not open ${path}: ${error.message}`, true);
      return null;
    }
  }
  if (!tab) return null;
  if (focus) activate(tab);
  if (find) highlightFind(tab, find);
  const target = anchor || (heading ? tab.headings.find((item) => item.text === heading)?.id : "");
  if (find) {
    requestAnimationFrame(() => {
      const hits = [...tab.article.querySelectorAll(".find-hit")];
      const section = target ? tab.article.querySelector(`[id="${CSS.escape(target)}"]`) : null;
      const hit = hits.find((node) => !section || section.compareDocumentPosition(node) & Node.DOCUMENT_POSITION_FOLLOWING) || hits[0];
      if (hit) hit.scrollIntoView({ block: "center" });
      else if (target) scrollToHeading(tab, target);
    });
  } else if (target) {
    requestAnimationFrame(() => scrollToHeading(tab, target));
  }
  return tab;
}

function createDocTab(path, text) {
  const existing = S.tabs.find((candidate) => candidate.key === `doc:${path}`);
  if (existing) return existing;
  const view = h("div", { class: "view" });
  const scroller = h("div", { class: "doc-scroll" });
  const tab = registerTab({ id: ++tabSequence, kind: "doc", key: `doc:${path}`, path, title: catalogTitle(path) || baseName(path), view, scroller, headings: [], objectUrls: [] });
  const bar = h("div", { class: "doc-bar" },
    icon("book"),
    h("span", { class: "crumb", text: path }),
    h("span", { class: "spacer" }),
    button("Cite in note", "cite", () => insertIntoNote(citationForDoc(tab)), { small: true, title: "Put a link to this section into the open note" }),
    button("Top", "up", () => { scroller.scrollTop = 0; }, { small: true, title: "Back to the top" }));
  scroller.append(bar);
  if (/\.md$/i.test(path)) {
    const { element, headings } = renderMarkdown(text, docContext(tab));
    tab.article = element;
    tab.headings = headings;
    if (!catalogTitle(path)) {
      const first = headings.find((item) => item.level === 1);
      if (first) tab.title = first.text;
    }
    scroller.append(element);
  } else {
    const pre = h("pre", { class: "raw-text" });
    if (/\.or$/i.test(path)) {
      pre.append(highlight(text));
      bar.insertBefore(button("Open as a new file", "external", () => newFile(S.selectedFolder, text), { small: true }), bar.lastChild);
    } else {
      pre.textContent = text;
    }
    tab.article = pre;
    scroller.append(pre);
  }
  let pending = false;
  scroller.addEventListener("scroll", () => {
    if (pending) return;
    pending = true;
    requestAnimationFrame(() => { pending = false; markCurrentHeading(tab); });
  });
  view.append(scroller);
  return tab;
}

function scrollToHeading(tab, id) {
  if (!tab.article) return;
  const target = tab.article.querySelector(`[id="${CSS.escape(id)}"]`);
  if (!target) return;
  const top = target.getBoundingClientRect().top - tab.scroller.getBoundingClientRect().top + tab.scroller.scrollTop - 48;
  tab.scroller.scrollTo({ top: Math.max(0, top) });
  target.classList.add("flash-target");
  setTimeout(() => target.classList.remove("flash-target"), 1400);
}

function currentHeading(tab) {
  if (!tab.article || !tab.headings.length) return null;
  const limit = tab.scroller.getBoundingClientRect().top + 90;
  let current = null;
  for (const heading of tab.headings) {
    const node = tab.article.querySelector(`[id="${CSS.escape(heading.id)}"]`);
    if (!node) continue;
    if (node.getBoundingClientRect().top <= limit) current = heading;
    else break;
  }
  return current || tab.headings[0];
}

function markCurrentHeading(tab) {
  if (S.side !== "library" || S.active !== tab) return;
  const current = currentHeading(tab);
  for (const row of els.sidebar.querySelectorAll(".lib-toc .tree-row")) {
    row.classList.toggle("active", !!current && row.dataset.id === current.id);
  }
}

function highlightFind(tab, query) {
  if (!tab.article) return;
  for (const mark of tab.article.querySelectorAll(".find-hit")) mark.replaceWith(...mark.childNodes);
  tab.article.normalize();
  const needle = query.toLowerCase();
  if (!needle) return;
  const walker = document.createTreeWalker(tab.article, NodeFilter.SHOW_TEXT);
  const nodes = [];
  while (walker.nextNode()) if (walker.currentNode.nodeValue.toLowerCase().includes(needle)) nodes.push(walker.currentNode);
  let count = 0;
  for (const node of nodes) {
    let current = node;
    for (;;) {
      const index = current.nodeValue.toLowerCase().indexOf(needle);
      if (index === -1 || count >= 300) break;
      const match = current.splitText(index);
      current = match.splitText(needle.length);
      const mark = h("span", { class: "find-hit" });
      match.replaceWith(mark);
      mark.append(match);
      count++;
    }
  }
}

// ---------- Notebook ----------

const nb = {};

function buildNotebook() {
  nb.list = h("div", { class: "note-list", role: "list" });
  nb.title = h("input", { class: "note-title", type: "text", spellcheck: "false", placeholder: "Note name", "aria-label": "Note name", title: "Rename this note" });
  nb.text = h("textarea", { class: "note-text", spellcheck: "true", placeholder: "Write in Markdown. Use Cite in note to link code and pages here.", "aria-label": "Note text" });
  nb.preview = h("div", { class: "note-preview" });
  nb.status = h("div", { class: "note-status" });
  nb.previewButton = iconButton("eye", "Preview", () => setNotePreview(!S.notePreview));
  const tool = (label, title, onclick) => h("button", { class: "icon-btn", type: "button", title, "aria-label": title, onclick, text: label });
  nb.tools = h("div", { class: "note-tools" },
    tool("H", "Heading", () => prefixLines("## ")),
    tool("B", "Bold", () => wrapSelection("**", "**", "bold text")),
    tool("I", "Italic", () => wrapSelection("*", "*", "italic text")),
    iconButton("code", "Inline code", () => wrapSelection("`", "`", "code")),
    iconButton("list", "Bulleted list", () => prefixLines("- ")),
    iconButton("sigma", "Orange code block", () => wrapSelection("\n```orange\n", "\n```\n", "spec value() -> Int { 0 }")),
    iconButton("cite", "Cite the place open in the editor or reader", () => citeActive()),
    nb.previewButton,
    iconButton("trash", "Delete this note", () => deleteNote()));
  nb.editor = h("div", { class: "note-editor" },
    h("div", { class: "note-title-row" }, nb.title),
    nb.tools,
    nb.text,
    nb.preview,
    nb.status);
  els.notebook.replaceChildren(
    sideHead("Notebook", [
      iconButton("plus", "New note", () => newNote()),
      iconButton("close", "Hide the notebook", () => toggleNotebook(false)),
    ]),
    nb.list,
    nb.editor,
  );
  nb.text.addEventListener("input", () => {
    if (!S.note) return;
    S.note.text = nb.text.value;
    clearTimeout(S.note.timer);
    S.note.timer = setTimeout(() => saveNote(), 700);
    renderNoteStatus();
  });
  nb.text.addEventListener("blur", () => { if (S.note && S.note.text !== S.note.savedText) saveNote(); });
  nb.title.addEventListener("change", () => renameNote(nb.title.value.trim()));
  nb.title.addEventListener("keydown", (event) => { if (event.key === "Enter") nb.title.blur(); });
  showNoteEditor();
}

async function refreshNotes() {
  try {
    const data = await api.notes();
    S.notes = data.notes;
  } catch (error) {
    S.notes = [];
    toast(`Could not read the notebook: ${error.message}`, true);
  }
  renderNoteList();
  for (const tab of S.tabs) applyNoteMarks(tab);
  if (!S.note && S.notes.length && !els.app.classList.contains("notebook-hidden")) openNote(S.notes[0].name);
}

function renderNoteList() {
  clear(nb.list);
  if (!S.notes.length) {
    nb.list.append(h("div", { class: "side-note", text: "No notes yet. Notes are Markdown files kept in .tabula/notes beside your code." }),
      h("div", { class: "side-note" }, button("New note", "plus", () => newNote(), { kind: "outline", small: true })));
    return;
  }
  for (const note of S.notes) {
    const cites = note.citations.length;
    nb.list.append(h("div", {
      class: `tree-row note-row${S.note && S.note.name === note.name ? " active" : ""}`,
      role: "listitem",
      tabindex: "0",
      title: `${note.name}.md`,
      onclick: () => openNote(note.name),
    },
      h("span", { class: "label", text: note.title }),
      h("small", { text: `${relativeTime(note.modified)}${cites ? ` · ${plural(cites, "citation")}` : ""}` })));
  }
}

function showNoteEditor() {
  const has = !!S.note;
  nb.editor.hidden = !has;
  if (!has) return;
  nb.title.value = S.note.name;
  nb.text.hidden = S.notePreview;
  nb.preview.hidden = !S.notePreview;
  nb.previewButton.replaceChildren(icon(S.notePreview ? "pencil" : "eye"));
  nb.previewButton.title = S.notePreview ? "Edit" : "Preview";
  nb.previewButton.classList.toggle("active", S.notePreview);
  for (const node of nb.tools.querySelectorAll("button")) {
    if (node !== nb.previewButton && node.title !== "Delete this note") node.disabled = S.notePreview;
  }
  if (S.notePreview) renderNotePreview();
  renderNoteStatus();
}

function renderNotePreview() {
  const { element } = renderMarkdown(S.note.text || "*This note is empty.*", docContext(null));
  nb.preview.replaceChildren(element);
}

function renderNoteStatus() {
  if (!S.note) return;
  const words = (S.note.text.match(/\S+/g) || []).length;
  const state = S.note.error ? `Not saved: ${S.note.error}` : S.note.saving ? "Saving…" : S.note.text !== S.note.savedText ? "Unsaved" : "Saved";
  nb.status.replaceChildren(h("span", { text: state }), h("span", { text: plural(words, "word") }));
}

function setNotePreview(on) {
  S.notePreview = on;
  showNoteEditor();
}

async function openNote(name) {
  if (S.note && S.note.name === name) return;
  if (S.note && S.note.text !== S.note.savedText) await saveNote();
  let data;
  try {
    data = await api.note(name);
  } catch (error) {
    toast(`Could not open the note: ${error.message}`, true);
    return;
  }
  S.note = { name, text: data.text, savedText: data.text, saving: false, timer: 0, error: null };
  nb.text.value = data.text;
  renderNoteList();
  showNoteEditor();
}

async function saveNote() {
  const note = S.note;
  if (!note || note.text === note.savedText) return true;
  clearTimeout(note.timer);
  const text = note.text;
  note.saving = true;
  renderNoteStatus();
  try {
    await api.saveNote(note.name, text);
    note.savedText = text;
    note.error = null;
  } catch (error) {
    note.error = error.message;
  }
  note.saving = false;
  if (S.note === note) renderNoteStatus();
  if (!note.error) refreshNotes();
  return !note.error;
}

async function newNote(text = "") {
  if (S.note && S.note.text !== S.note.savedText) await saveNote();
  let created;
  try {
    created = await api.createNote("", text);
  } catch (error) {
    toast(`Could not create a note: ${error.message}`, true);
    return null;
  }
  toggleNotebook(true);
  await refreshNotes();
  await openNote(created.result);
  setNotePreview(false);
  nb.text.focus();
  return created.result;
}

async function renameNote(to) {
  const note = S.note;
  if (!note || !to || to === note.name) { if (note) nb.title.value = note.name; return; }
  if (!/^[A-Za-z0-9][A-Za-z0-9 ._-]*$/.test(to) || /[ .]$/.test(to) || to.includes("..") || to.length > 80) {
    toast("Note names use letters, digits, spaces, dots, dashes, and underscores, and cannot end with a space or dot.", true);
    nb.title.value = note.name;
    return;
  }
  if (!(await saveNote())) return;
  try {
    await api.renameNote(note.name, to);
  } catch (error) {
    toast(`Could not rename the note: ${error.message}`, true);
    nb.title.value = note.name;
    return;
  }
  note.name = to;
  await refreshNotes();
}

async function deleteNote() {
  const note = S.note;
  if (!note) return;
  const yes = await confirmDialog({
    title: `Delete the note ${note.name}?`,
    message: "Tabula moves it to .tabula/trash in this workspace, so it can be recovered from there.",
    confirm: "Move to trash",
    danger: true,
  });
  if (!yes) return;
  clearTimeout(note.timer);
  try {
    await api.removeNote(note.name);
  } catch (error) {
    toast(`Could not delete the note: ${error.message}`, true);
    return;
  }
  S.note = null;
  nb.text.value = "";
  showNoteEditor();
  await refreshNotes();
}

// Inserts `text` into the open note at its cursor, opening or creating a
// note first when needed.
async function insertIntoNote(text) {
  toggleNotebook(true);
  if (!S.note) {
    if (S.notes.length) await openNote(S.notes[0].name);
    else await newNote();
  }
  if (!S.note) return;
  setNotePreview(false);
  const area = nb.text;
  const value = area.value;
  let start = area.selectionStart;
  let end = area.selectionEnd;
  if (document.activeElement !== area && start === 0 && end === 0 && value) {
    start = value.length;
    end = value.length;
  }
  const before = value.slice(0, start);
  const lead = !before || before.endsWith("\n\n") ? "" : before.endsWith("\n") ? "\n" : "\n\n";
  replaceInNote(start, end, `${lead}${text}\n`);
  toast("Cited in the note");
}

function replaceInNote(start, end, text, selectFrom = null, selectTo = null) {
  const area = nb.text;
  area.focus();
  area.setSelectionRange(start, end);
  let done = false;
  try { done = document.execCommand("insertText", false, text); } catch { done = false; }
  if (!done) {
    area.setRangeText(text, start, end, "end");
    area.dispatchEvent(new Event("input"));
  }
  if (selectFrom !== null) area.setSelectionRange(selectFrom, selectTo ?? selectFrom);
}

function wrapSelection(before, after, placeholder) {
  if (!S.note) return;
  const area = nb.text;
  const start = area.selectionStart;
  const end = area.selectionEnd;
  const selected = area.value.slice(start, end) || placeholder;
  replaceInNote(start, end, `${before}${selected}${after}`, start + before.length, start + before.length + selected.length);
}

function prefixLines(prefix) {
  if (!S.note) return;
  const area = nb.text;
  const value = area.value;
  const start = value.lastIndexOf("\n", area.selectionStart - 1) + 1;
  let end = value.indexOf("\n", area.selectionEnd);
  if (end === -1) end = value.length;
  const lines = value.slice(start, end).split("\n");
  const all = lines.every((line) => line.startsWith(prefix));
  const changed = lines.map((line) => (all ? line.slice(prefix.length) : line.replace(/^(#{1,6} |- )?/, prefix))).join("\n");
  replaceInNote(start, end, changed, start, start + changed.length);
}

// ---------- Results panel ----------

const PANEL_TABS = [
  { id: "problems", title: "Problems" },
  { id: "values", title: "Values" },
  { id: "evidence", title: "Evidence" },
  { id: "tokens", title: "Tokens" },
];

function renderPanel() {
  const tab = S.active && S.active.kind === "file" ? S.active : null;
  const hidden = els.main.classList.contains("panel-hidden");
  const tabsRow = h("div", { class: "panel-tabs", role: "tablist" });
  for (const item of PANEL_TABS) {
    let pill = null;
    if (tab && item.id === "problems" && tab.result) pill = h("span", { class: `count-pill${tab.diagnostics.length ? " bad" : ""}`, text: String(tab.diagnostics.length) });
    if (tab && item.id === "values" && tab.values) pill = h("span", { class: "count-pill", text: String(tab.values.length) });
    tabsRow.append(h("button", {
      class: `panel-tab${S.panelTab === item.id && !hidden ? " active" : ""}`,
      type: "button",
      role: "tab",
      onclick: () => {
        S.panelTab = item.id;
        store.set("panelTab", item.id);
        if (hidden) togglePanel(true);
        else renderPanel();
      },
    }, item.title, pill));
  }
  tabsRow.append(h("span", { class: "panel-spacer" }),
    iconButton(hidden ? "up" : "chevron", hidden ? "Show the results panel" : "Hide the results panel", () => togglePanel()));
  const body = h("div", { class: "panel-body", role: "tabpanel" });
  if (!tab) body.append(h("div", { class: "panel-empty", text: "Open an Orange file to see what orangec reports about it." }));
  else if (S.panelTab === "values") renderValues(body, tab);
  else if (S.panelTab === "evidence") renderEvidence(body, tab);
  else if (S.panelTab === "tokens") renderTokens(body, tab);
  else renderProblems(body, tab);
  els.panel.replaceChildren(tabsRow, body);
}

function noCompilerNote(body) {
  body.append(h("div", { class: "panel-empty" },
    h("b", { text: "orangec was not found." }),
    " Build it with make build-compiler in the Orange checkout, or start Tabula with --orangec <path>."));
}

function renderProblems(body, tab) {
  if (!S.session.orangec) { noCompilerNote(body); return; }
  if (tab.hostError) {
    body.append(h("div", { class: "panel-empty" }, h("b", { text: "orangec could not run. " }), tab.hostError));
    return;
  }
  if (!tab.result) {
    body.append(h("div", { class: "panel-empty", text: tab.running ? "Checking…" : "Not checked yet. Press Check to run orangec." }));
    return;
  }
  const response = tab.result.response;
  if (!tab.diagnostics.length) {
    if (response.success) {
      body.append(h("div", { class: "panel-empty" }, h("b", { text: "No problems." }), ` ${response.version} accepted this text${tab.result.action === "eval" ? " and evaluated it" : ""}.`));
    } else {
      body.append(h("div", { class: "panel-empty" }, h("b", { text: response.timedOut ? "orangec took too long and was stopped." : "orangec failed without a diagnostic." })));
      if (response.stderr) body.append(h("pre", { class: "raw-text", text: response.stderr }));
    }
    return;
  }
  for (const diagnostic of tab.diagnostics) {
    const where = diagnostic.line ? `Ln ${diagnostic.line}, Col ${diagnostic.column || 1}` : "whole file";
    const row = h("div", {
      class: "problem",
      role: "button",
      tabindex: "0",
      title: diagnostic.from !== null ? "Show in the editor" : diagnostic.raw,
      onclick: () => { if (diagnostic.from !== null) tab.editor.reveal(diagnostic.from, diagnostic.to); },
    },
      h("span", { class: "sev" }),
      h("span", { class: "pcode", text: diagnostic.code }),
      h("div", { class: "msg" },
        diagnostic.message,
        diagnostic.label ? h("small", { text: diagnostic.label }) : null,
        diagnostic.notes.map((note) => h("small", { text: `note: ${note}` })),
        diagnostic.related.map((item) => h("small", { class: "related", text: `Ln ${item.line}: ${item.label}` }))),
      h("span", { class: "where" }, where,
        button("Look up", "search", (event) => {
          event.stopPropagation();
          lookUp(diagnostic.code);
        }, { small: true, kind: "explain", title: `Search the Library for ${diagnostic.code}` })));
    body.append(row);
  }
}

function lookUp(query) {
  S.side = "library";
  store.set("side", "library");
  S.libraryQuery = query;
  toggleSidebar(true);
  renderLibrary();
  const results = els.sidebar.querySelector(".side-body");
  if (results) runSearch(results);
}

function renderValues(body, tab) {
  if (!S.session.orangec) { noCompilerNote(body); return; }
  if (!tab.values) {
    const failing = tab.result && !tab.result.response.success;
    body.append(h("div", { class: "panel-empty" },
      failing ? "Fix the problems first; orangec evaluates only a file it accepts. " : "Evaluate to see the exact value of each typed spec. ",
      failing ? null : button("Evaluate", "play", () => runActive("eval"), { small: true, kind: "outline" })));
    return;
  }
  if (!tab.values.length) {
    body.append(h("div", { class: "panel-empty", text: "This file has no typed specs to evaluate. Add one such as spec answer() -> Int { 42 }." }));
    return;
  }
  const radix = h("span", { class: "radix", role: "group", "aria-label": "Number base" },
    [[10, "dec"], [16, "hex"], [2, "bin"]].map(([base, label]) => h("button", {
      type: "button",
      class: S.radix === base ? "active" : "",
      title: { 10: "Decimal", 16: "Hexadecimal", 2: "Binary" }[base],
      text: label,
      onclick: () => {
        S.radix = base;
        store.set("radix", base);
        for (const other of S.tabs) if (other.kind === "file") updateInlays(other);
        renderPanel();
        if (S.side === "strata") renderStrata();
      },
    })));
  const stale = tab.valuesSource !== tab.editor.getValue();
  const table = h("table", { class: "values" },
    h("thead", {}, h("tr", {},
      h("th", { text: "Spec" }), h("th", { text: "Type" }), h("th", {}, "Value ", radix), h("th", { text: stale ? "Before your latest edit" : "Size" }))));
  const tbody = h("tbody");
  const starts = tab.starts || lineStarts(tab.editor.getValue());
  for (const value of tab.values) {
    const decl = tab.outline ? tab.outline.decls.find((item) => item.stratum === "spec" && item.name === value.name) : null;
    const number = parseValue(value.type, value.value);
    const width = wordWidth(value.type);
    let meta = null;
    if (number !== null && width) {
      const bits = h("span", { class: "bits", title: `${width} bits, most significant first` });
      wordBits(number, width).forEach((on, index) => bits.append(h("i", { class: `${on ? "on" : ""}${index && index % 4 === 0 ? " gap" : ""}`.trim() })));
      meta = bits;
    } else if (number !== null) {
      meta = `${plural(bitLength(number), "bit")}${number < 0n ? ", negative" : ""}`;
    }
    const shown = showValue(value);
    tbody.append(h("tr", {},
      h("td", { class: "name", title: decl ? `Go to line ${lineOf(starts, decl.start) + 1}` : "", onclick: () => { if (decl) tab.editor.reveal(decl.nameStart, decl.nameEnd); } },
        h("span", { text: `${value.module}::` }), value.name),
      h("td", { class: "ty", text: value.type }),
      h("td", { class: "val", title: "Copy this value", onclick: () => copyText(shown, `Copied ${shown}`) }, shown),
      h("td", { class: "meta" }, meta)));
  }
  table.append(tbody);
  body.append(table);
}

function phaseStates(result) {
  const response = result.response;
  const counts = { lexical: 0, syntax: 0, semantic: 0, evaluation: 0, host: 0, unknown: 0 };
  for (const diagnostic of response.diagnostics) counts[diagnostic.phase] = (counts[diagnostic.phase] || 0) + 1;
  if (counts.host || response.timedOut) {
    const detail = response.timedOut ? "orangec was stopped" : "orangec could not read the text";
    return PHASES.map((phase) => ({ ...phase, state: "skip", detail }));
  }
  const firstFailure = PHASES.findIndex((phase) => counts[phase.id] > 0);
  return PHASES.map((phase, index) => {
    const count = counts[phase.id];
    if (count) return { ...phase, state: "fail", detail: plural(count, "error") };
    if (firstFailure !== -1 && index > firstFailure) return { ...phase, state: "skip", detail: "not reached" };
    if (phase.id === "evaluation") {
      if (result.action !== "eval") return { ...phase, state: "skip", detail: "not run; press Evaluate" };
      return { ...phase, state: "pass", detail: plural((response.values || []).length, "value") };
    }
    if (!response.success && firstFailure === -1) return { ...phase, state: "skip", detail: "unknown" };
    return { ...phase, state: "pass", detail: "no findings" };
  });
}

function evidenceRecord(tab) {
  const result = tab.result;
  const response = result.response;
  const lines = [
    `Orange evidence record (Tabula ${S.session.tabula})`,
    `file:     ${result.path}${result.saved ? "" : " (unsaved text)"}`,
    `sha256:   ${result.sha256 || "pending"}`,
    `bytes:    ${result.bytes}`,
    `compiler: ${response.version}`,
    `command:  orangec ${response.action} -`,
    `result:   ${response.success ? "accepted" : `rejected, exit ${response.status ?? "none"}`}${response.diagnostics.length ? `, ${plural(response.diagnostics.length, "diagnostic")}` : ""}`,
    `phases:   ${phaseStates(result).map((phase) => `${phase.id} ${phase.state === "pass" ? "passed" : phase.state === "fail" ? "failed" : phase.detail}`).join("; ")}`,
  ];
  for (const diagnostic of response.diagnostics) {
    lines.push(`  ${diagnostic.code} ${diagnostic.line ? `${diagnostic.line}:${diagnostic.column || 1}` : "-"} ${diagnostic.message}`);
  }
  if (response.action === "eval" && response.success) {
    lines.push("values:");
    for (const value of response.values) lines.push(`  ${value.module}::${value.name}: ${value.type} = ${value.value}`);
  }
  lines.push(`recorded: ${result.at.toISOString()}`);
  return lines.join("\n");
}

function renderEvidence(body, tab) {
  if (!S.session.orangec) { noCompilerNote(body); return; }
  if (!tab.result) {
    body.append(h("div", { class: "panel-empty", text: "No run yet. Check or Evaluate the file to record what orangec found." }));
    return;
  }
  const result = tab.result;
  const response = result.response;
  const current = result.source === tab.editor.getValue();
  const wrap = h("div", { class: "assurance" });
  wrap.append(h("dl", { class: "assurance-grid" },
    h("dt", { text: "File" }), h("dd", { text: `${result.path}${result.saved ? "" : " (unsaved text)"}${current ? "" : " · edited since this run"}` }),
    h("dt", { text: "SHA-256" }), h("dd", { text: result.sha256 || "computing…" }),
    h("dt", { text: "Size" }), h("dd", { text: `${result.bytes} bytes` }),
    h("dt", { text: "Compiler" }), h("dd", { text: response.version }),
    h("dt", { text: "Run" }), h("dd", { text: `orangec ${response.action} · exit ${response.status ?? "none"} · ${response.millis} ms · ${result.at.toLocaleTimeString()}` })));
  wrap.append(h("div", { class: "phases" }, phaseStates(result).map((phase) => h("div", { class: `phase ${phase.state}`, title: phase.about },
    h("b", { text: phase.title }), h("span", { text: phase.detail })))));
  wrap.append(h("div", { class: "evidence-actions" },
    button("Copy record", "copy", () => copyText(evidenceRecord(tab), "Copied the evidence record"), { small: true, kind: "outline" }),
    button("Add to note", "notebook", () => insertIntoNote(`Evidence for [${tab.title}](tabula:${tab.path}):\n\n\`\`\`text\n${evidenceRecord(tab)}\n\`\`\``), { small: true, kind: "outline" })));
  wrap.append(h("p", { class: "disclaimer", text: "This record says what this orangec reported for these exact bytes; the SHA-256 names them. It is evidence about the pre-alpha Orange 2026 checks (syntax, names, types, and literal values), not a proof that a design is correct or secure." }));
  body.append(wrap);
}

function renderTokens(body, tab) {
  if (!S.session.orangec) { noCompilerNote(body); return; }
  const source = tab.editor.getValue();
  const fresh = tab.tokens && tab.tokens.source === source;
  body.append(h("div", { class: "panel-empty" },
    "The token stream orangec reads. Tabula's colours follow the same lexical rules. ",
    button(fresh ? "Refresh" : "Show tokens", "list", () => loadTokens(tab), { small: true, kind: "outline" })));
  if (!tab.tokens) return;
  if (!fresh) body.append(h("div", { class: "panel-empty", text: "These tokens are from before your latest edit." }));
  const starts = lineStarts(tab.tokens.source);
  const table = h("table", { class: "token-table" });
  const shown = tab.tokens.list.slice(0, 4000);
  for (const token of shown) {
    const line = lineOf(starts, token.start);
    const column = [...tab.tokens.source.slice(starts[line], token.start)].length + 1;
    table.append(h("tr", { onclick: () => { if (fresh) tab.editor.reveal(token.start, Math.max(token.end, token.start)); } },
      h("td", { class: "span", text: `${line + 1}:${column}` }),
      h("td", { class: "kind", text: token.kind }),
      h("td", { text: JSON.stringify(token.spelling) })));
  }
  body.append(table);
  if (tab.tokens.list.length > shown.length) body.append(h("div", { class: "panel-empty", text: `Showing ${shown.length} of ${tab.tokens.list.length} tokens.` }));
}

async function loadTokens(tab) {
  const source = tab.editor.getValue();
  try {
    const response = await api.lex(diskTextOf(tab, source));
    tab.tokens = { source, list: response.tokens };
  } catch (error) {
    toast(`orangec lex failed: ${error.message}`, true);
    return;
  }
  if (tab === S.active && S.panelTab === "tokens") renderPanel();
}

// ---------- Status bar ----------

const statusItems = {};

function renderStatus() {
  if (!statusItems.compiler) {
    statusItems.compiler = h("button", { class: "status-item", type: "button", onclick: () => { S.panelTab = "problems"; store.set("panelTab", "problems"); togglePanel(true); } });
    statusItems.live = h("button", { class: "status-item", type: "button", onclick: () => {
      S.live = !S.live;
      store.set("live", S.live);
      if (S.live && S.active && S.active.kind === "file") scheduleRun(S.active, 10);
      renderStatus();
    } });
    statusItems.saved = h("span", { class: "status-item" });
    statusItems.cursor = h("span", { class: "status-item" });
    statusItems.eol = h("span", { class: "status-item", title: "Line endings are kept as the file had them" });
    statusItems.edition = h("span", { class: "status-item", text: "Orange 2026" });
    els.statusbar.append(statusItems.compiler, statusItems.live, statusItems.saved, h("span", { class: "status-spacer" }), statusItems.cursor, statusItems.eol, h("span", { class: "status-item", text: "UTF-8" }), statusItems.edition);
  }
  const tab = S.active;
  const file = tab && tab.kind === "file";
  const compiler = statusItems.compiler;
  compiler.className = "status-item";
  if (!S.session.orangec) {
    compiler.classList.add("bad");
    compiler.replaceChildren(h("span", { class: "pulse" }), "orangec not found");
  } else if (!file) {
    compiler.replaceChildren(icon("check"), S.session.orangec.version);
  } else if (tab.running) {
    compiler.classList.add("busy");
    compiler.replaceChildren(h("span", { class: "pulse" }), "Checking…");
  } else if (tab.hostError) {
    compiler.classList.add("bad");
    compiler.replaceChildren(h("span", { class: "pulse" }), "orangec could not run");
  } else if (!tab.result) {
    compiler.replaceChildren(icon("check"), S.live ? "Waiting to check" : "Not checked");
  } else if (tab.diagnostics.length) {
    compiler.classList.add("bad");
    compiler.replaceChildren(h("span", { class: "pulse" }), plural(tab.diagnostics.length, "problem"));
  } else if (tab.result.response.success) {
    compiler.classList.add("ok");
    compiler.replaceChildren(icon("check"), tab.result.source === tab.editor.getValue() ? "Accepted by orangec" : "Accepted before this edit");
  } else {
    compiler.classList.add("bad");
    compiler.replaceChildren(h("span", { class: "pulse" }), "Rejected");
  }
  compiler.title = S.session.orangec ? `${S.session.orangec.version} · ${S.session.orangec.path}` : "Start Tabula with --orangec <path>";
  statusItems.live.hidden = !S.session.orangec;
  statusItems.live.replaceChildren(icon(S.live ? "eye" : "pencil"), S.live ? "Live checks on" : "Live checks off");
  statusItems.live.title = S.live ? "orangec checks the file as you type. Click to check only when you press Check or Evaluate." : "Click to let orangec check the file as you type.";
  statusItems.saved.textContent = file ? (tab.dirty ? "Unsaved changes" : "Saved") : tab && tab.kind === "doc" ? `Reading ${tab.path}` : "";
  statusItems.eol.textContent = file ? (tab.eol === "\r\n" ? "CRLF" : "LF") : "";
  statusItems.eol.hidden = !file;
  renderCursor();
}

function renderCursor() {
  const tab = S.active;
  if (!statusItems.cursor) return;
  if (!tab || tab.kind !== "file") {
    statusItems.cursor.textContent = "";
    return;
  }
  const { line, column, selection } = tab.cursor;
  statusItems.cursor.textContent = `Ln ${line}, Col ${column}${selection ? ` (${selection} selected)` : ""}`;
}

// ---------- About ----------

function showAbout() {
  const session = S.session;
  const content = h("div", {},
    h("p", { text: "Tabula is a small, mouse-first workbench for writing Orange. It runs only on this computer, talks to orangec for every check, and reads The Orange Book and the manuals straight from the Orange checkout." }),
    h("dl", {},
      h("dt", { text: "Tabula" }), h("dd", { text: session.tabula }),
      h("dt", { text: "Workspace" }), h("dd", { text: session.workspaceRoot }),
      h("dt", { text: "Notes" }), h("dd", { text: ".tabula/notes (Markdown)" }),
      h("dt", { text: "Trash" }), h("dd", { text: ".tabula/trash" }),
      h("dt", { text: "orangec" }), h("dd", { text: session.orangec ? `${session.orangec.version}\n${session.orangec.path}` : "not found; start Tabula with --orangec <path>" }),
      h("dt", { text: "Library" }), h("dd", { text: session.library ? session.library.root : "not found; start Tabula with --library <checkout>" })));
  const actions = [{ label: "Close", value: null, kind: session.library ? undefined : "primary" }];
  if (session.library) actions.push({ label: "Read the Tabula manual", value: "manual", kind: "primary" });
  dialog({ title: "About Tabula", content, actions, wide: true }).then((choice) => {
    if (choice === "manual") openDoc("tabula/README.md");
  });
}
