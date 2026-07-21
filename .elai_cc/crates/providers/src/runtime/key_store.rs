//! J-30 T1 -- per-provider BYOK key store (env -> 0600 file fallback).
//!
//! Lookup order mirrors the live Anthropic `resolve_startup_auth_source`
//! waterfall: a non-empty env var (exact name) wins, then a 0600-mode
//! `provider_keys.json` keyed by `provider_id`. The store reuses
//! `SecretString` so key material never leaks through Debug/Display and
//! the buffer is zeroised on drop. `store` writes atomically: tmp file +
//! fsync + rename, Unix 0600, Windows best-effort restrictive perms with
//! no new deps. No token-exchange / device-authorization arms live here
//! (I-50 owns those flows); this module is the plain-key tier only.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use elai_core::config::default_config_home;

use crate::runtime::secret::SecretString;

const STORE_FILE_NAME: &str = "provider_keys.json";

#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("key store io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("key store parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("key store home directory not set (ELAI_CONFIG_HOME / HOME)")]
    HomeNotSet,
}

fn env_non_empty(name: &str) -> Result<Option<SecretString>, KeyStoreError> {
    match std::env::var_os(name) {
        Some(value) => {
            let trimmed = value.to_string_lossy();
            if trimmed.trim().is_empty() {
                return Ok(None);
            }
            Ok(Some(SecretString::new(trimmed.into_owned())))
        }
        None => Ok(None),
    }
}

fn store_path() -> Result<PathBuf, KeyStoreError> {
    let home = default_config_home();
    if home.as_os_str().is_empty() {
        return Err(KeyStoreError::HomeNotSet);
    }
    Ok(home.join(STORE_FILE_NAME))
}

fn read_store_map(path: &Path) -> Result<BTreeMap<String, String>, KeyStoreError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(err) => return Err(err.into()),
    };
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    if buf.trim().is_empty() {
        return Ok(BTreeMap::new());
    }
    let map: BTreeMap<String, String> = serde_json::from_str(&buf)?;
    Ok(map)
}

pub fn lookup(provider_id: &str, env_name: &str) -> Result<Option<SecretString>, KeyStoreError> {
    if let Some(key) = env_non_empty(env_name)? {
        return Ok(Some(key));
    }
    let path = store_path()?;
    let map = read_store_map(&path)?;
    Ok(map
        .get(provider_id)
        .filter(|s| !s.is_empty())
        .map(|s| SecretString::new(s.clone())))
}

// WIRE-DARK[J-30-T5] store's production consumer is the T5 `elai providers
// configure` CLI flow (out of scope for T1 per task spec). Registered as a
// deferred-wiring stub: the T5 slice wires this into the CLI configure
// command. `lookup` is wired in this slice via anthropic.rs.
pub fn store(provider_id: &str, key: &SecretString) -> Result<(), KeyStoreError> {
    let path = store_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut map = read_store_map(&path)?;
    map.insert(provider_id.to_string(), key.expose_secret().to_string());
    write_store_atomic(&path, &map)?;
    Ok(())
}

fn write_store_atomic(path: &Path, map: &BTreeMap<String, String>) -> Result<(), KeyStoreError> {
    let mut tmp_path = OsString::from(path);
    tmp_path.push(".tmp");
    let tmp_path = PathBuf::from(tmp_path);

    {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)?;
        set_restrictive_perms(&file);
        let mut writer = BufWriter::new(file);
        let body = serde_json::to_string_pretty(map)?;
        writer.write_all(body.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        let file = writer
            .into_inner()
            .map_err(|err| std::io::Error::other(err.to_string()))?;
        file.sync_all()?;
    }
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn set_restrictive_perms(file: &File) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = file.set_permissions(fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    {
        let _ = file;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_config_home<F: FnOnce(&Path)>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("ELAI_CONFIG_HOME", dir.path());
        f(dir.path());
        std::env::remove_var("ELAI_CONFIG_HOME");
    }

    #[test]
    fn env_wins_over_file() {
        with_config_home(|home| {
            let _ = store("anthropic", &SecretString::new("file-key".to_string()));
            std::env::set_var("ANTHROPIC_API_KEY", "env-key");
            let got = lookup("anthropic", "ANTHROPIC_API_KEY").expect("lookup");
            std::env::remove_var("ANTHROPIC_API_KEY");
            assert_eq!(got.expect("some").expose_secret(), "env-key");
            assert!(home.join(STORE_FILE_NAME).exists());
        });
    }

    #[test]
    fn file_fallback_when_env_empty() {
        with_config_home(|_| {
            std::env::remove_var("ANTHROPIC_API_KEY");
            let _ = store("anthropic", &SecretString::new("file-key".to_string()));
            let got = lookup("anthropic", "ANTHROPIC_API_KEY").expect("lookup");
            assert_eq!(got.expect("some").expose_secret(), "file-key");
        });
    }

    #[test]
    fn lookup_returns_none_when_absent() {
        with_config_home(|_| {
            std::env::remove_var("ANTHROPIC_API_KEY");
            let got = lookup("anthropic", "ANTHROPIC_API_KEY").expect("lookup");
            assert!(got.is_none());
        });
    }

    #[test]
    fn store_is_atomic_and_persistent() {
        with_config_home(|home| {
            let _ = store("openai", &SecretString::new("sk-abc".to_string()));
            let path = home.join(STORE_FILE_NAME);
            assert!(path.exists());
            assert!(!home.join("provider_keys.json.tmp").exists());
            let got = lookup("openai", "OPENAI_API_KEY").expect("lookup");
            assert_eq!(got.expect("some").expose_secret(), "sk-abc");
        });
    }

    #[test]
    fn empty_env_falls_through_to_file() {
        with_config_home(|_| {
            std::env::set_var("ANTHROPIC_API_KEY", "   ");
            let _ = store("anthropic", &SecretString::new("file-key".to_string()));
            let got = lookup("anthropic", "ANTHROPIC_API_KEY").expect("lookup");
            std::env::remove_var("ANTHROPIC_API_KEY");
            assert_eq!(got.expect("some").expose_secret(), "file-key");
        });
    }
}
