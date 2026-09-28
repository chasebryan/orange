// Exact values printed by `orangec eval`, shown in the radix you think in.
// Every rendering is itself a valid Orange 2026 literal, digit separators
// included, so a value can be copied straight back into a spec.

export function parseValue(type, text) {
  try {
    if (/^-?[0-9]+$/.test(text)) return BigInt(text);
    if (/^0x[0-9a-fA-F]+$/.test(text)) return BigInt(text);
    if (/^0b[01]+$/.test(text)) return BigInt(text);
  } catch { /* not a number */ }
  return null;
}

// The fixed width of a word type such as Word[8], or null.
export function wordWidth(type) {
  const match = /^Word\[(\d+)\]$/.exec(type);
  return match ? Number(match[1]) : null;
}

function group(digits, size) {
  let out = "";
  for (let i = digits.length; i > 0; i -= size) {
    const chunk = digits.slice(Math.max(0, i - size), i);
    out = out ? `${chunk}_${out}` : chunk;
  }
  return out;
}

// Renders `value` (a BigInt) in radix 10, 16, or 2. Words are zero-padded to
// their width.
export function formatValue(value, radix, width = null) {
  const negative = value < 0n;
  const magnitude = negative ? -value : value;
  let digits = magnitude.toString(radix);
  if (width && radix === 2) digits = digits.padStart(width, "0");
  if (width && radix === 16) digits = digits.padStart(Math.ceil(width / 4), "0");
  const grouped = radix === 10 ? (digits.length > 4 ? group(digits, 3) : digits) : group(digits, radix === 2 ? 4 : 4);
  const prefix = radix === 16 ? "0x" : radix === 2 ? "0b" : "";
  return `${negative ? "-" : ""}${prefix}${grouped}`;
}

export function bitLength(value) {
  const magnitude = value < 0n ? -value : value;
  return magnitude === 0n ? 0 : magnitude.toString(2).length;
}

// Bits of a word, most significant first.
export function wordBits(value, width) {
  return value.toString(2).padStart(width, "0").split("").map((bit) => bit === "1");
}
