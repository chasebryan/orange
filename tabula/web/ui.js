// Small DOM helpers, icons, dialogs, toasts, and tooltips. Everything that
// shows text builds DOM nodes with textContent; only Tabula's own fixed icon
// markup is ever parsed as HTML.

const ICONS = {
  files: '<path d="M4 4.5A1.5 1.5 0 0 1 5.5 3h5l2 2h6A1.5 1.5 0 0 1 20 6.5v11a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 4 17.5z"/>',
  strata: '<path d="M12 3 3 7.5l9 4.5 9-4.5z"/><path d="m3 12 9 4.5 9-4.5"/><path d="m3 16.5 9 4.5 9-4.5"/>',
  book: '<path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H20v15H6.5A2.5 2.5 0 0 0 4 20.5z"/><path d="M4 20.5A2.5 2.5 0 0 0 6.5 23H20v-5"/><path d="M8 7h8M8 10.5h6"/>',
  notebook: '<path d="M6 3h11a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6z"/><path d="M6 3v18M3.5 7H6M3.5 12H6M3.5 17H6M10 8h5M10 11.5h5"/>',
  check: '<circle cx="12" cy="12" r="8.5"/><path d="m8.2 12.3 2.6 2.6 5-5.3"/>',
  play: '<path d="M7 4.8v14.4a.8.8 0 0 0 1.2.7l11.3-7.2a.8.8 0 0 0 0-1.4L8.2 4.1a.8.8 0 0 0-1.2.7z"/>',
  save: '<path d="M5 3h11l4 4v12.5a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 4 19.5v-15A1.5 1.5 0 0 1 5.5 3z"/><path d="M8 3v5h7V3M8 21v-7h8v7"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  folder: '<path d="M3.5 6.5A1.5 1.5 0 0 1 5 5h4.5l2 2H19a1.5 1.5 0 0 1 1.5 1.5v9A1.5 1.5 0 0 1 19 19H5a1.5 1.5 0 0 1-1.5-1.5z"/>',
  "folder-plus": '<path d="M3.5 6.5A1.5 1.5 0 0 1 5 5h4.5l2 2H19a1.5 1.5 0 0 1 1.5 1.5v9A1.5 1.5 0 0 1 19 19H5a1.5 1.5 0 0 1-1.5-1.5z"/><path d="M12 10.5v5M9.5 13h5"/>',
  file: '<path d="M6.5 3H14l4.5 4.5v12A1.5 1.5 0 0 1 17 21H6.5A1.5 1.5 0 0 1 5 19.5v-15A1.5 1.5 0 0 1 6.5 3z"/><path d="M14 3v4.5h4.5"/>',
  orange: '<circle cx="12" cy="13" r="7.5"/><path d="M12 5.5c0-1.6 1-2.5 2.6-2.5M12 5.5c-1.3-1.3-3.2-1.4-4.4-.6"/><circle cx="12" cy="13" r="2.2"/>',
  chevron: '<path d="m7 9.5 5 5 5-5"/>',
  close: '<path class="close-x" d="m7 7 10 10M17 7 7 17"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2.5v2M12 19.5v2M4.6 4.6 6 6M18 18l1.4 1.4M2.5 12h2M19.5 12h2M4.6 19.4 6 18M18 6l1.4-1.4"/>',
  moon: '<path d="M19.5 14.5A8 8 0 0 1 9.5 4.5a8 8 0 1 0 10 10z"/>',
  info: '<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5.5M12 7.6v.1"/>',
  refresh: '<path d="M19.5 12a7.5 7.5 0 1 1-2.2-5.3"/><path d="M19.5 4.5v4h-4"/>',
  trash: '<path d="M4.5 7h15M9.5 7V4.5h5V7M6.5 7l1 13h9l1-13"/>',
  rename: '<path d="M4 20h4L19 9l-4-4L4 16z"/><path d="m13.5 6.5 4 4"/>',
  search: '<circle cx="11" cy="11" r="6.5"/><path d="m16 16 4.5 4.5"/>',
  cite: '<path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1"/><path d="M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"/>',
  eye: '<path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z"/><circle cx="12" cy="12" r="2.8"/>',
  pencil: '<path d="M4 20h4L19 9l-4-4L4 16z"/>',
  panel: '<rect x="3.5" y="4.5" width="17" height="15" rx="1.5"/><path d="M3.5 14.5h17"/>',
  sidebar: '<rect x="3.5" y="4.5" width="17" height="15" rx="1.5"/><path d="M9 4.5v15"/>',
  external: '<path d="M14 4.5h5.5V10M19.5 4.5 11 13M18 14v4.5a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 4 18.5v-11A1.5 1.5 0 0 1 5.5 6H10"/>',
  shield: '<path d="M12 3 5 5.5v6c0 4.5 3 7.8 7 9.5 4-1.7 7-5 7-9.5v-6z"/><path d="m9 12 2.2 2.2L15.5 10"/>',
  hash: '<path d="M9.5 3.5 7.5 20.5M16.5 3.5l-2 17M4.5 9h16M3.5 15h16"/>',
  copy: '<rect x="8.5" y="8.5" width="12" height="12" rx="1.5"/><path d="M15.5 8.5v-3A1.5 1.5 0 0 0 14 4H5.5A1.5 1.5 0 0 0 4 5.5V14a1.5 1.5 0 0 0 1.5 1.5h3"/>',
  list: '<path d="M9 6.5h11M9 12h11M9 17.5h11"/><circle cx="4.8" cy="6.5" r=".9"/><circle cx="4.8" cy="12" r=".9"/><circle cx="4.8" cy="17.5" r=".9"/>',
  code: '<path d="m8.5 7-5 5 5 5M15.5 7l5 5-5 5"/>',
  sigma: '<path d="M17.5 5h-11l6 7-6 7h11"/>',
  back: '<path d="M10 6 4 12l6 6M4.5 12H20"/>',
  up: '<path d="m6 15 6-6 6 6"/>',
  bookmark: '<path d="M6.5 3.5h11v17l-5.5-4-5.5 4z"/>',
};

export function icon(name, className = "") {
  const holder = document.createElement("span");
  holder.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"${className ? ` class="${className}"` : ""}>${ICONS[name] || ""}</svg>`;
  return holder.firstChild;
}

// h("div", {class: "x", onclick: fn, title: "..."}, child, "text", ...)
export function h(tag, attrs, ...children) {
  const node = document.createElement(tag);
  if (attrs) {
    for (const [key, value] of Object.entries(attrs)) {
      if (value === undefined || value === null || value === false) continue;
      if (key === "class") node.className = value;
      else if (key === "text") node.textContent = value;
      else if (key.startsWith("on") && typeof value === "function") node.addEventListener(key.slice(2), value);
      else if (key === "dataset") Object.assign(node.dataset, value);
      else if (key in node && typeof value !== "string") node[key] = value;
      else node.setAttribute(key, value === true ? "" : String(value));
    }
  }
  append(node, children);
  return node;
}

function append(node, children) {
  for (const child of children) {
    if (child === null || child === undefined || child === false) continue;
    if (Array.isArray(child)) append(node, child);
    else node.append(child instanceof Node ? child : document.createTextNode(String(child)));
  }
}

// A toolbar-style button with an icon and a label.
export function button(label, iconName, onclick, options = {}) {
  const node = h("button", {
    class: `btn ${options.kind || ""} ${options.small ? "small" : ""}`.trim(),
    type: "button",
    title: options.title || label,
    onclick,
    "aria-label": label,
  });
  if (iconName) node.append(icon(iconName));
  if (label && !options.iconOnly) node.append(h("span", { class: "btn-label", text: label }));
  return node;
}

export function iconButton(iconName, title, onclick, className = "") {
  return h("button", { class: `icon-btn ${className}`.trim(), type: "button", title, "aria-label": title, onclick }, icon(iconName));
}

export function clear(node) {
  while (node.firstChild) node.firstChild.remove();
  return node;
}

// ---------- Dialogs ----------

const overlay = () => document.getElementById("overlay");

// Opens a modal dialog. `content` is a node or text; `actions` are
// [{label, value, kind}]. Resolves with the chosen value, or null on cancel.
export function dialog({ title, content, actions, wide = false, onOpen, validate }) {
  return new Promise((resolve) => {
    const root = overlay();
    clear(root);
    const errorText = h("div", { class: "error-text" });
    let done = false;
    const finish = (value) => {
      if (done) return;
      if (value !== null && validate) {
        const problem = validate(value);
        if (problem) {
          errorText.textContent = problem;
          return;
        }
      }
      done = true;
      root.classList.remove("open");
      clear(root);
      document.removeEventListener("keydown", onKey, true);
      resolve(value);
    };
    const onKey = (event) => {
      if (event.key === "Escape") { event.preventDefault(); finish(null); }
      if (event.key === "Enter" && event.target.tagName !== "TEXTAREA" && event.target.tagName !== "BUTTON") {
        const primary = actions.find((action) => action.kind === "primary");
        if (primary) { event.preventDefault(); finish(typeof primary.value === "function" ? primary.value() : primary.value); }
      }
    };
    const box = h("div", { class: `dialog ${wide ? "wide" : ""}`, role: "dialog", "aria-modal": "true", "aria-label": title },
      h("h2", { text: title }),
      typeof content === "string" ? h("p", { text: content }) : content,
      errorText,
      h("div", { class: "dialog-actions" },
        actions.map((action) => h("button", {
          class: `btn ${action.kind === "primary" ? "primary" : action.kind === "danger" ? "danger" : "outline"}`,
          type: "button",
          text: action.label,
          onclick: () => finish(typeof action.value === "function" ? action.value() : action.value),
        })),
      ),
    );
    root.append(box);
    root.classList.add("open");
    root.onclick = (event) => { if (event.target === root) finish(null); };
    document.addEventListener("keydown", onKey, true);
    if (onOpen) onOpen(box);
    const focusable = box.querySelector("input, textarea, select") || box.querySelector(".btn.primary");
    if (focusable) setTimeout(() => { focusable.focus(); if (focusable.select) focusable.select(); }, 20);
  });
}

// Asks for one line of text.
export async function ask({ title, message, label, value = "", confirm = "OK", validate }) {
  const input = h("input", { class: "field", type: "text", value, spellcheck: "false", autocomplete: "off" });
  const content = h("div", {}, message ? h("p", { text: message }) : null, label ? h("label", { text: label }) : null, input);
  const result = await dialog({
    title,
    content,
    validate: validate ? () => validate(input.value.trim()) : undefined,
    actions: [
      { label: "Cancel", value: null },
      { label: confirm, value: () => input.value.trim(), kind: "primary" },
    ],
  });
  return result;
}

export function confirmDialog({ title, message, confirm = "OK", danger = false, extra }) {
  const actions = [{ label: "Cancel", value: null }];
  if (extra) actions.push(extra);
  actions.push({ label: confirm, value: true, kind: danger ? "danger" : "primary" });
  return dialog({ title, content: message, actions });
}

// ---------- Toasts and tooltips ----------

let toastTimer = null;
export function toast(message, bad = false) {
  document.querySelectorAll(".toast").forEach((node) => node.remove());
  const node = h("div", { class: `toast ${bad ? "bad" : ""}`, role: "status", text: message });
  document.body.append(node);
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => node.remove(), bad ? 5200 : 2600);
}

export function showTooltip(x, y, ...content) {
  const tip = document.getElementById("tooltip");
  clear(tip);
  append(tip, content);
  tip.classList.add("show");
  const width = tip.offsetWidth;
  const height = tip.offsetHeight;
  const left = Math.min(Math.max(8, x), window.innerWidth - width - 8);
  const top = y + height + 16 > window.innerHeight ? y - height - 10 : y + 16;
  tip.style.left = `${left}px`;
  tip.style.top = `${top}px`;
}

export function hideTooltip() {
  document.getElementById("tooltip").classList.remove("show");
}

// ---------- Formatting ----------

export function relativeTime(seconds) {
  if (!seconds) return "";
  const delta = Date.now() / 1000 - seconds;
  if (delta < 60) return "just now";
  if (delta < 3600) return `${Math.floor(delta / 60)} min ago`;
  if (delta < 86400) return `${Math.floor(delta / 3600)} h ago`;
  if (delta < 86400 * 30) return `${Math.floor(delta / 86400)} d ago`;
  return new Date(seconds * 1000).toLocaleDateString();
}

export function plural(count, one, many = `${one}s`) {
  return `${count} ${count === 1 ? one : many}`;
}
