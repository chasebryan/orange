//! Orange language edition selection.

use std::fmt;
use std::str::FromStr;

/// A frozen set of language rules selected for one source compilation.
///
/// Edition names are calendar years. The pre-alpha compiler supports only the
/// initial `2026` edition, but callers must still select an edition explicitly
/// or accept [`Edition::default`]. This prevents edition state from being added
/// as an incompatible afterthought.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Edition {
    /// The initial, pre-alpha Orange language edition.
    #[default]
    E2026,
}

impl Edition {
    /// The newest edition understood by this compiler.
    pub const CURRENT: Self = Self::E2026;

    /// Returns the stable textual edition name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::E2026 => "2026",
        }
    }
}

impl fmt::Display for Edition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Edition {
    type Err = ParseEditionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "2026" => Ok(Self::E2026),
            _ => Err(ParseEditionError),
        }
    }
}

/// Returned when an edition name is not supported by this compiler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseEditionError;

impl fmt::Display for ParseEditionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("unsupported Orange edition; supported editions: 2026")
    }
}

impl std::error::Error for ParseEditionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_initial_edition() {
        assert_eq!("2026".parse(), Ok(Edition::E2026));
        assert_eq!(Edition::CURRENT.to_string(), "2026");
    }

    #[test]
    fn rejects_unknown_editions() {
        let error = "2027".parse::<Edition>().unwrap_err();
        assert_eq!(
            error.to_string(),
            "unsupported Orange edition; supported editions: 2026"
        );
    }
}
