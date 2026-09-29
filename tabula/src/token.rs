//! The per-launch session token that admits the browser to Tabula's API.
//!
//! The token is 128 bits from the operating system's random source. The page
//! receives it once, in the launch link, and sends it back in the
//! `X-Tabula-Token` header, which a cross-origin page cannot attach without a
//! CORS preflight that Tabula never approves. Comparison runs in constant time
//! for equal-length inputs.

use std::fmt::Write as _;
use std::io;

/// Length of the token in bytes.
pub const TOKEN_BYTES: usize = 16;

/// A session token.
#[derive(Clone, Eq, PartialEq)]
pub struct SessionToken {
    hex: String,
}

impl std::fmt::Debug for SessionToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SessionToken(..)")
    }
}

impl SessionToken {
    /// Draws a fresh token from the operating system.
    ///
    /// # Errors
    ///
    /// Returns an error when no operating-system random source is available.
    pub fn generate() -> io::Result<Self> {
        let bytes = os_random()?;
        Ok(Self::from_bytes(&bytes))
    }

    /// Builds a token from known bytes (for tests).
    #[must_use]
    pub fn from_bytes(bytes: &[u8; TOKEN_BYTES]) -> Self {
        let mut hex = String::with_capacity(TOKEN_BYTES.saturating_mul(2));
        for byte in bytes {
            let _ = write!(hex, "{byte:02x}");
        }
        Self { hex }
    }

    /// The lowercase hexadecimal form placed in the launch link.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.hex
    }

    /// Compares a presented token in constant time with respect to its
    /// contents. Length mismatches return early; the length is public.
    #[must_use]
    pub fn matches(&self, presented: &str) -> bool {
        let expected = self.hex.as_bytes();
        let presented = presented.as_bytes();
        if expected.len() != presented.len() {
            return false;
        }
        let difference = expected
            .iter()
            .zip(presented)
            .fold(0_u8, |accumulator, (left, right)| {
                accumulator | (left ^ right)
            });
        std::hint::black_box(difference) == 0
    }
}

#[cfg(unix)]
fn os_random() -> io::Result<[u8; TOKEN_BYTES]> {
    use std::io::Read as _;
    let mut bytes = [0_u8; TOKEN_BYTES];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    if bytes.iter().all(|byte| *byte == 0) {
        return Err(io::Error::other("random source returned zeros"));
    }
    Ok(bytes)
}

#[cfg(not(unix))]
fn os_random() -> io::Result<[u8; TOKEN_BYTES]> {
    // Without an external crate the standard library exposes the operating
    // system's random source only through `RandomState`, whose SipHash keys
    // are drawn from it. Hashing distinct inputs under those secret keys gives
    // unpredictable output without the keys.
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher as _, Hasher as _};
    let mut bytes = [0_u8; TOKEN_BYTES];
    for (index, chunk) in bytes.chunks_mut(8).enumerate() {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_usize(index);
        hasher.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos()),
        );
        for (target, source) in chunk.iter_mut().zip(hasher.finish().to_le_bytes()) {
            *target = source;
        }
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::SessionToken;

    #[test]
    fn renders_hex_and_matches_exactly() {
        let token = SessionToken::from_bytes(&[0xab; 16]);
        assert_eq!(token.as_str(), "ab".repeat(16));
        assert!(token.matches(&"ab".repeat(16)));
        assert!(!token.matches(&"ab".repeat(15)));
        assert!(!token.matches(&format!("{}ac", "ab".repeat(15))));
        assert!(!token.matches(""));
        assert_eq!(format!("{token:?}"), "SessionToken(..)");
    }

    #[test]
    fn generated_tokens_differ() {
        let first = SessionToken::generate().unwrap();
        let second = SessionToken::generate().unwrap();
        assert_eq!(first.as_str().len(), 32);
        assert_ne!(first, second);
    }
}
