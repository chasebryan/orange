//! Orange language edition selection.

use std::fmt;
use std::str::FromStr;

macro_rules! define_editions {
    ($current:ident; $($(#[$variant_doc:meta])* $variant:ident => $name:literal,)+) => {
        /// A frozen set of language rules selected for one source compilation.
        ///
        /// Edition names are calendar years. The pre-alpha compiler supports only the
        /// initial `2026` edition, but callers must still select an edition explicitly
        /// or accept [`Edition::default`]. This prevents edition state from being added
        /// as an incompatible afterthought.
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum Edition {
            $($(#[$variant_doc])* $variant,)+
        }

        impl Edition {
            /// The newest edition understood by this compiler.
            pub const CURRENT: Self = Self::$current;

            /// All supported editions in chronological order.
            pub const SUPPORTED: &'static [Self] = &[$(Self::$variant,)+];

            /// Returns the stable textual edition name.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }
        }

        impl FromStr for Edition {
            type Err = ParseEditionError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($name => Ok(Self::$variant),)+
                    _ => Err(ParseEditionError),
                }
            }
        }

        impl Default for Edition {
            fn default() -> Self {
                Self::CURRENT
            }
        }
    };
}

define_editions! {
    E2026;
    /// The initial, pre-alpha Orange language edition.
    E2026 => "2026",
}

impl fmt::Display for Edition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Returned when an edition name is not supported by this compiler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseEditionError;

impl fmt::Display for ParseEditionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("unsupported Orange edition; supported editions: ")?;
        for (index, edition) in Edition::SUPPORTED.iter().enumerate() {
            if index > 0 {
                formatter.write_str(", ")?;
            }
            formatter.write_str(edition.as_str())?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseEditionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_initial_edition() {
        assert_eq!(Edition::SUPPORTED, &[Edition::E2026]);
        assert_eq!(Edition::SUPPORTED.last(), Some(&Edition::CURRENT));
        assert_eq!(Edition::default(), Edition::CURRENT);
        assert!(Edition::SUPPORTED.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(Edition::SUPPORTED.iter().all(|edition| {
            edition.as_str().parse::<Edition>() == Ok(*edition)
                && edition.as_str().bytes().all(|byte| byte.is_ascii_digit())
        }));
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
