//! `SecretString` — in-crate newtype that hides credential strings
//! from accidental logging and zeroes its backing buffer on drop.
//!
//! We avoid the upstream `secrecy` + `zeroize` crates intentionally:
//! - CLAUDE.md HARD RULE: "No new dependencies without confirmation."
//! - The threat model here is `tracing::info!("got cfg {cfg:?}")`
//!   accidentally printing an API key, not defeating cold-boot
//!   forensics. A manual `Drop` impl that fills the buffer with zero
//!   bytes via `unsafe` byte-level mutation through `as_mut_vec` is
//!   sufficient for that.
//!
//! Invariants enforced here:
//! 1. `Debug` / `Display` NEVER reveal the inner value (we render
//!    `"SecretString(<redacted: N bytes>)"`).
//! 2. `expose_secret()` is the single sanctioned read path; call sites
//!    are easy to grep for ("expose_secret").
//! 3. On `Drop`, the buffer is overwritten with zero bytes before
//!    deallocation (best-effort -- the compiler may still leave copies
//!    in registers or moved-out temporaries; same caveat as `zeroize`).

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A newtype wrapper around `String` for credentials. See module-level
/// docs for the threat model.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString {
    inner: String,
}

impl SecretString {
    /// Wrap a `String` as a secret.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self { inner: value }
    }

    /// Borrow the secret. Call sites should be rare; grep for
    /// `expose_secret` to audit them.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.inner
    }

    /// Consume the wrapper and return the inner `String`. Used at
    /// adapter boundaries (e.g. handing the secret to
    /// `AnthropicAuthSource::ApiKey`). The original wrapper's `Drop`
    /// no longer runs, but the consuming type (e.g. `AnthropicClient`)
    /// owns the lifetime from here on.
    #[must_use]
    pub fn into_inner(mut self) -> String {
        std::mem::take(&mut self.inner)
    }

    /// Length of the underlying secret in bytes. Safe to surface
    /// through Debug because it leaks no key material.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Convenience.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretString(<redacted: {} bytes>)", self.inner.len())
    }
}

impl std::fmt::Display for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Same redaction as Debug -- we never want a `format!("{}", ...)`
        // to drop the secret into a log line.
        write!(f, "<redacted: {} bytes>", self.inner.len())
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        // Overwrite the heap buffer with zeros before it's freed. The
        // standard `String::clear` only resets `len`; the bytes stay in
        // the underlying allocation. We need a defensive zero-fill so
        // `tracing` events and copy-paste from heap dumps don't leak
        // recently-dropped secrets.
        //
        // SAFETY: `as_mut_vec` is unsafe because it permits writing
        // non-UTF-8 bytes. We immediately replace every byte with 0
        // and then truncate, so the resulting empty string is valid
        // UTF-8. We never observe the buffer between zeroing and
        // truncation.
        unsafe {
            let bytes = self.inner.as_mut_vec();
            for b in bytes.iter_mut() {
                *b = 0;
            }
            bytes.clear();
        }
    }
}

/// `Serialize` impl renders the redacted form. Adapters that need the
/// real secret on the wire must call `expose_secret()` and serialise
/// the string explicitly.
impl Serialize for SecretString {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("<redacted>")
    }
}

/// `Deserialize` accepts plain strings (config files, TOML).
impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_does_not_leak_inner() {
        let secret = SecretString::new("hunter2".to_string());
        let rendered = format!("{secret:?}");
        assert!(!rendered.contains("hunter2"));
        assert!(rendered.contains("redacted"));
    }

    #[test]
    fn display_does_not_leak_inner() {
        let secret = SecretString::new("hunter2".to_string());
        let rendered = format!("{secret}");
        assert!(!rendered.contains("hunter2"));
    }

    #[test]
    fn expose_secret_returns_inner() {
        let secret = SecretString::new("hunter2".to_string());
        assert_eq!(secret.expose_secret(), "hunter2");
    }

    #[test]
    fn serialise_redacts() {
        let secret = SecretString::new("hunter2".to_string());
        let rendered = serde_json::to_string(&secret).expect("serialise");
        assert!(!rendered.contains("hunter2"));
    }

    #[test]
    fn deserialise_round_trip() {
        let parsed: SecretString = serde_json::from_str("\"hunter2\"").expect("parse");
        assert_eq!(parsed.expose_secret(), "hunter2");
    }
}
