// Runs before the stylesheet so the first paint is already in the chosen
// theme. The ids must match THEMES in app.js, which applies the theme again.
try {
  const stored = JSON.parse(localStorage.getItem("tabula.theme"));
  const light = matchMedia("(prefers-color-scheme: light)").matches;
  document.documentElement.dataset.theme = ["ink", "paper", "tokyo", "corporate"].includes(stored) ? stored : light ? "paper" : "ink";
} catch {
  // Storage is unavailable; app.js falls back to the system theme.
}
