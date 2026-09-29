// The page's connection to its Tabula session. The launch link carries the
// session token as `?k=`; it is kept for this origin and sent on every API call
// in the X-Tabula-Token header, then removed from the address bar.

const KEY = "tabula.token";

function readToken() {
  const params = new URLSearchParams(location.search);
  const fromLink = params.get("k");
  if (fromLink && /^[0-9a-f]{32}$/.test(fromLink)) {
    try { localStorage.setItem(KEY, fromLink); } catch { /* private mode */ }
    history.replaceState(null, "", location.pathname + location.hash);
    return fromLink;
  }
  try { return localStorage.getItem(KEY) || ""; } catch { return ""; }
}

const token = readToken();

export class ApiError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}

async function call(method, path, params, body) {
  const query = params ? `?${new URLSearchParams(Object.entries(params).filter(([, v]) => v !== undefined && v !== null))}` : "";
  let response;
  try {
    response = await fetch(`/api/${path}${query}`, {
      method,
      headers: { "X-Tabula-Token": token, ...(body !== undefined ? { "Content-Type": "text/plain; charset=utf-8" } : {}) },
      body,
      cache: "no-store",
      credentials: "omit",
    });
  } catch {
    throw new ApiError(0, "Tabula is not running. Start it again from your terminal.");
  }
  const type = response.headers.get("Content-Type") || "";
  if (!type.startsWith("application/json")) {
    if (!response.ok) throw new ApiError(response.status, `request failed (${response.status})`);
    return response;
  }
  const data = await response.json();
  if (!response.ok || data.ok === false) throw new ApiError(response.status, data.error || `request failed (${response.status})`);
  return data;
}

export const api = {
  hasToken: () => token.length > 0,
  session: () => call("GET", "session"),
  tree: () => call("GET", "tree"),
  read: (path) => call("GET", "file", { path }),
  save: (path, text) => call("POST", "file/save", { path }, text),
  create: (path, text) => call("POST", "file/create", { path }, text),
  createFolder: (path) => call("POST", "folder/create", { path }),
  rename: (from, to) => call("POST", "file/rename", { from, to }),
  remove: (path) => call("POST", "file/delete", { path }),
  check: (source) => call("POST", "orangec/check", null, source),
  evaluate: (source) => call("POST", "orangec/eval", null, source),
  lex: (source) => call("POST", "orangec/lex", null, source),
  library: () => call("GET", "library"),
  doc: (path) => call("GET", "library/doc", { path }),
  search: (q) => call("GET", "library/search", { q }),
  assetUrl: (path) => `/api/library/asset?${new URLSearchParams({ path })}`,
  notes: () => call("GET", "notes"),
  note: (name) => call("GET", "note", { name }),
  saveNote: (name, text) => call("POST", "note/save", { name }, text),
  createNote: (name, text = "") => call("POST", "note/create", { name }, text),
  renameNote: (from, to) => call("POST", "note/rename", { from, to }),
  removeNote: (name) => call("POST", "note/delete", { name }),
};

// Images in the Library need the token too, so they are fetched and shown
// through object URLs rather than plain <img src> requests.
export async function loadAsset(path) {
  const response = await fetch(api.assetUrl(path), { headers: { "X-Tabula-Token": token }, cache: "no-store", credentials: "omit" });
  if (!response.ok) throw new ApiError(response.status, "image unavailable");
  return URL.createObjectURL(await response.blob());
}
