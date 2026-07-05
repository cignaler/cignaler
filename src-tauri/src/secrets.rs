//! Token storage backed by the OS keyring (macOS Keychain, Windows Credential
//! Manager, Linux Secret Service). When no keyring backend is available (e.g.
//! Linux without a Secret Service daemon), callers fall back to storing the
//! token in the database.

use keyring::Entry;
use tracing::warn;

/// Placeholder stored in the database's api_key column when the real token
/// lives in the OS keyring.
pub const KEYRING_SENTINEL: &str = "__keyring__";

const SERVICE: &str = "com.ostwi.dev.cignaler";

fn entry(server_name: &str) -> keyring::Result<Entry> {
    Entry::new(SERVICE, server_name)
}

/// Try to store a token in the OS keyring. Returns true when stored securely,
/// false when the keyring is unavailable and the caller must fall back to
/// database storage.
pub fn store_token(server_name: &str, token: &str) -> bool {
    match entry(server_name).and_then(|e| e.set_password(token)) {
        Ok(()) => true,
        Err(e) => {
            warn!(
                "OS keyring unavailable for server '{}', falling back to database storage: {}",
                server_name, e
            );
            false
        }
    }
}

/// Read a token from the OS keyring.
pub fn get_token(server_name: &str) -> Option<String> {
    match entry(server_name).and_then(|e| e.get_password()) {
        Ok(token) => Some(token),
        Err(e) => {
            warn!("Failed to read token for server '{}' from keyring: {}", server_name, e);
            None
        }
    }
}

/// Remove a token from the OS keyring. Errors are ignored — the entry may
/// legitimately not exist (database-fallback storage).
pub fn delete_token(server_name: &str) {
    if let Ok(e) = entry(server_name) {
        let _ = e.delete_credential();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_roundtrip() {
        let name = "__cignaler_secrets_test__";

        // Environments without a keyring backend (e.g. headless Linux CI)
        // legitimately fall back to database storage — nothing to assert.
        if !store_token(name, "test-token-123") {
            eprintln!("keyring unavailable, skipping roundtrip assertions");
            return;
        }

        assert_eq!(get_token(name).as_deref(), Some("test-token-123"));

        // Overwrite works
        assert!(store_token(name, "rotated-456"));
        assert_eq!(get_token(name).as_deref(), Some("rotated-456"));

        delete_token(name);
        assert_eq!(get_token(name), None);
    }
}
