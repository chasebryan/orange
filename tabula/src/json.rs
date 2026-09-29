//! A small JSON writer for Tabula's HTTP responses.
//!
//! Tabula only ever *writes* JSON; request bodies are plain text, so no JSON
//! parser is needed. Strings are escaped per RFC 8259, and `<`, `>`, `&`,
//! U+2028, and U+2029 are also escaped so a response stays inert even if a
//! browser were tricked into treating it as markup or script.

use std::fmt::Write as _;

/// A JSON value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Json {
    /// `null`
    Null,
    /// `true` or `false`
    Bool(bool),
    /// A non-negative integer.
    Number(u64),
    /// A signed integer.
    Signed(i64),
    /// A string.
    String(String),
    /// An array.
    Array(Vec<Json>),
    /// An object whose members keep their insertion order.
    Object(Vec<(String, Json)>),
}

impl Json {
    /// Builds a string value.
    #[must_use]
    pub fn str(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Builds an object from `(name, value)` pairs.
    #[must_use]
    pub fn object<const N: usize>(members: [(&str, Json); N]) -> Self {
        Self::Object(
            members
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value))
                .collect(),
        )
    }

    /// Builds a string value, or `null` when absent.
    #[must_use]
    pub fn opt_str(value: Option<&str>) -> Self {
        value.map_or(Self::Null, |text| Self::String(text.to_owned()))
    }

    /// Builds a number from a `usize`, saturating at `u64::MAX`.
    #[must_use]
    pub fn usize(value: usize) -> Self {
        Self::Number(u64::try_from(value).unwrap_or(u64::MAX))
    }

    /// Serializes the value to a compact JSON string.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::new();
        self.write_to(&mut out);
        out
    }

    fn write_to(&self, out: &mut String) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(true) => out.push_str("true"),
            Self::Bool(false) => out.push_str("false"),
            Self::Number(value) => {
                let _ = write!(out, "{value}");
            }
            Self::Signed(value) => {
                let _ = write!(out, "{value}");
            }
            Self::String(value) => write_string(out, value),
            Self::Array(items) => {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index != 0 {
                        out.push(',');
                    }
                    item.write_to(out);
                }
                out.push(']');
            }
            Self::Object(members) => {
                out.push('{');
                for (index, (name, value)) in members.iter().enumerate() {
                    if index != 0 {
                        out.push(',');
                    }
                    write_string(out, name);
                    out.push(':');
                    value.write_to(out);
                }
                out.push('}');
            }
        }
    }
}

fn write_string(out: &mut String, value: &str) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' | '>' | '&' | '\u{2028}' | '\u{2029}' => {
                let _ = write!(out, "\\u{:04x}", u32::from(character));
            }
            control if u32::from(control) < 0x20 || control == '\u{7f}' => {
                let _ = write!(out, "\\u{:04x}", u32::from(control));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::Json;

    #[test]
    fn renders_scalars_and_containers() {
        let value = Json::object([
            ("ok", Json::Bool(true)),
            ("none", Json::Null),
            ("n", Json::Number(42)),
            ("neg", Json::Signed(-7)),
            ("list", Json::Array(vec![Json::str("a"), Json::Bool(false)])),
        ]);
        assert_eq!(
            value.render(),
            r#"{"ok":true,"none":null,"n":42,"neg":-7,"list":["a",false]}"#
        );
    }

    #[test]
    fn escapes_strings_defensively() {
        let value = Json::str("a\"b\\c\n\r\t\u{0}\u{1f}\u{7f}<script>&\u{2028}\u{2029}é𝔽");
        assert_eq!(
            value.render(),
            "\"a\\\"b\\\\c\\n\\r\\t\\u0000\\u001f\\u007f\\u003cscript\\u003e\\u0026\\u2028\\u2029é𝔽\""
        );
    }

    #[test]
    fn empty_containers() {
        assert_eq!(Json::Array(Vec::new()).render(), "[]");
        assert_eq!(Json::Object(Vec::new()).render(), "{}");
    }
}
