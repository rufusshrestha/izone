// izone/src/web_auth.rs
//
// Password handling for `izone web`.
//
// - The default password is compiled in only as a salted Argon2id hash (DEFAULT_PASSWORD_HASH).
//   Generate it with `izone web hash-password`.
// - A changed password is stored as an Argon2id hash, encrypted with AES-256-GCM, in the user's
//   config directory. The 256-bit key is kept separately: macOS Keychain, Windows Credential
//   Manager, or a 0600 key file under the user's local data directory on Linux. No sudo needed.
// - `izone web reset-password` deletes both, so the default password applies again.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::password_hash::{rand_core::OsRng as SaltRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

/// Salted Argon2id hash (PHC string) of the default web password.
/// Empty until set: run `izone web hash-password` and paste the output here.
pub const DEFAULT_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$oF/EtZU/WwaSM+93SugS/Q$F+yUw+T08PndKihHD4Jh9m9cjxZ1O7mUffPy0Rp+7G0";

pub const MIN_PASSWORD_LEN: usize = 10;
pub const MAX_PASSWORD_LEN: usize = 128;

const FILE_MAGIC: &[u8; 4] = b"IZW1";
const AAD: &[u8] = b"izone-web-password-v1";
const NONCE_LEN: usize = 12;

// ----------------------------------------------------------------------------
// Hashing
// ----------------------------------------------------------------------------

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut SaltRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("could not hash password: {}", e))
}

/// Constant-time verification against a PHC hash string. Invalid hashes never verify.
pub fn verify_password(password: &str, phc: &str) -> bool {
    match PasswordHash::new(phc) {
        Ok(hash) => Argon2::default().verify_password(password.as_bytes(), &hash).is_ok(),
        Err(_) => false,
    }
}

pub fn default_hash_is_valid() -> bool {
    !DEFAULT_PASSWORD_HASH.is_empty() && PasswordHash::new(DEFAULT_PASSWORD_HASH).is_ok()
}

pub fn check_new_password(password: &str) -> Result<(), String> {
    let n = password.chars().count();
    if n < MIN_PASSWORD_LEN {
        return Err(format!("Password must be at least {} characters", MIN_PASSWORD_LEN));
    }
    if n > MAX_PASSWORD_LEN {
        return Err(format!("Password must be at most {} characters", MAX_PASSWORD_LEN));
    }
    if verify_password(password, DEFAULT_PASSWORD_HASH) {
        return Err("Choose a password different from the default".to_string());
    }
    Ok(())
}

// ----------------------------------------------------------------------------
// Encrypted storage of the changed password hash
// ----------------------------------------------------------------------------

fn config_dir() -> Result<PathBuf, String> {
    dirs::config_dir().map(|p| p.join("izone")).ok_or_else(|| "could not find the user config directory".to_string())
}

pub fn credential_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("web-password.enc"))
}

/// Modification time of the stored credential (None if there isn't one); used to notice
/// changes made by another process, e.g. `izone web reset-password` while the server runs.
pub fn credential_stamp() -> Option<SystemTime> {
    credential_path().ok().and_then(|p| fs::metadata(p).ok()).and_then(|m| m.modified().ok())
}

/// Write a file readable only by the current user (0600 on Unix), atomically.
fn write_private(path: &PathBuf, data: &[u8]) -> Result<(), String> {
    let dir = path.parent().ok_or("invalid path")?;
    create_private_dir(dir)?;
    let tmp = path.with_extension("tmp");
    {
        let mut opts = fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts.open(&tmp).map_err(|e| format!("could not write {}: {}", tmp.display(), e))?;
        f.write_all(data).map_err(|e| format!("could not write {}: {}", tmp.display(), e))?;
        f.sync_all().ok();
    }
    fs::rename(&tmp, path).map_err(|e| format!("could not save {}: {}", path.display(), e))
}

fn create_private_dir(dir: &std::path::Path) -> Result<(), String> {
    #[cfg(unix)]
    let result = {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)
    };
    #[cfg(not(unix))]
    let result = fs::create_dir_all(dir);
    result.map_err(|e| format!("could not create {}: {}", dir.display(), e))
}

fn remove_if_exists(path: &PathBuf) -> Result<bool, String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("could not delete {}: {}", path.display(), e)),
    }
}

/// AES-256-GCM: magic | 12-byte random nonce | ciphertext+tag. The magic is also bound as AAD.
fn encrypt_with(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce), Payload { msg: plaintext, aad: AAD })
        .map_err(|_| "encryption failed".to_string())?;
    let mut data = Vec::with_capacity(4 + NONCE_LEN + ct.len());
    data.extend_from_slice(FILE_MAGIC);
    data.extend_from_slice(&nonce);
    data.extend_from_slice(&ct);
    Ok(data)
}

fn decrypt_with(key: &[u8; 32], data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 4 + NONCE_LEN + 16 || &data[..4] != FILE_MAGIC {
        return None;
    }
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    cipher
        .decrypt(Nonce::from_slice(&data[4..4 + NONCE_LEN]), Payload { msg: &data[4 + NONCE_LEN..], aad: AAD })
        .ok()
}

/// Store the hash of a changed password (encrypted).
pub fn save_password_hash(phc: &str) -> Result<(), String> {
    let key = match key_store::load()? {
        Some(k) => k,
        None => {
            let mut k = [0u8; 32];
            OsRng.fill_bytes(&mut k);
            key_store::save(&k)?;
            k
        }
    };
    write_private(&credential_path()?, &encrypt_with(&key, phc.as_bytes())?)
}

/// Load the stored (changed) password hash. Ok(None) = no stored password, use the default.
pub fn load_password_hash() -> Result<Option<String>, String> {
    let path = credential_path()?;
    let data = match fs::read(&path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("could not read {}: {}", path.display(), e)),
    };
    let broken = || format!("the stored web password in {} can't be read; run `izone web reset-password`", path.display());
    let key = key_store::load()?.ok_or_else(broken)?;
    let pt = decrypt_with(&key, &data).ok_or_else(broken)?;
    let phc = String::from_utf8(pt).map_err(|_| broken())?;
    PasswordHash::new(&phc).map_err(|_| broken())?;
    Ok(Some(phc))
}

/// Delete the stored password and its key. Returns true if anything was removed.
/// The password file goes first: without it the default applies, even if the key can't be
/// deleted (e.g. the macOS Keychain is locked or unavailable over SSH). That case is a warning.
pub fn reset_password() -> Result<bool, String> {
    let removed_file = remove_if_exists(&credential_path()?)?;
    let removed_key = match key_store::delete() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Warning: {} (the password itself was removed; delete the \"izone-web\" entry manually if it remains)", e);
            false
        }
    };
    Ok(removed_file || removed_key)
}

pub fn storage_description() -> String {
    let file = credential_path().map(|p| p.display().to_string()).unwrap_or_else(|_| "?".into());
    format!("{} (AES-256-GCM, key in {})", file, key_store::location())
}

// ----------------------------------------------------------------------------
// Key storage: OS secret store on macOS/Windows, private key file on Linux/other
// ----------------------------------------------------------------------------

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod key_store {
    const SERVICE: &str = "izone-web";
    const ACCOUNT: &str = "password-encryption-key";

    fn entry() -> Result<keyring::Entry, String> {
        keyring::Entry::new(SERVICE, ACCOUNT).map_err(|e| format!("could not open {}: {}", location(), e))
    }

    pub fn load() -> Result<Option<[u8; 32]>, String> {
        match entry()?.get_secret() {
            Ok(v) if v.len() == 32 => {
                let mut k = [0u8; 32];
                k.copy_from_slice(&v);
                Ok(Some(k))
            }
            Ok(_) => Err(format!("the key in {} is invalid; run `izone web reset-password`", location())),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!("could not read the key from {}: {}", location(), e)),
        }
    }

    pub fn save(key: &[u8; 32]) -> Result<(), String> {
        entry()?.set_secret(key).map_err(|e| format!("could not save the key to {}: {}", location(), e))
    }

    pub fn delete() -> Result<bool, String> {
        match entry()?.delete_credential() {
            Ok(()) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(e) => Err(format!("could not delete the key from {}: {}", location(), e)),
        }
    }

    pub fn location() -> &'static str {
        if cfg!(target_os = "macos") { "the macOS Keychain" } else { "Windows Credential Manager" }
    }
}

/// Key kept in a private (0600) file. Used on Linux and other Unix systems.
#[cfg_attr(any(target_os = "macos", target_os = "windows"), allow(dead_code))]
mod file_key {
    use std::path::PathBuf;

    pub fn load(p: &PathBuf) -> Result<Option<[u8; 32]>, String> {
        match std::fs::read(p) {
            Ok(v) if v.len() == 32 => {
                let mut k = [0u8; 32];
                k.copy_from_slice(&v);
                Ok(Some(k))
            }
            Ok(_) => Err(format!("the key in {} is invalid; run `izone web reset-password`", p.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("could not read {}: {}", p.display(), e)),
        }
    }

    pub fn save(p: &PathBuf, key: &[u8; 32]) -> Result<(), String> {
        super::write_private(p, key)
    }

    pub fn delete(p: &PathBuf) -> Result<bool, String> {
        super::remove_if_exists(p)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod key_store {
    use super::file_key;
    use std::path::PathBuf;

    fn path() -> Result<PathBuf, String> {
        dirs::data_local_dir()
            .map(|p| p.join("izone").join("web-password.key"))
            .ok_or_else(|| "could not find the user data directory".to_string())
    }

    pub fn load() -> Result<Option<[u8; 32]>, String> {
        file_key::load(&path()?)
    }

    pub fn save(key: &[u8; 32]) -> Result<(), String> {
        file_key::save(&path()?, key)
    }

    pub fn delete() -> Result<bool, String> {
        file_key::delete(&path()?)
    }

    pub fn location() -> String {
        path().map(|p| format!("{} (mode 0600)", p.display())).unwrap_or_else(|_| "a private key file".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(name: &str) -> PathBuf {
        let mut b = [0u8; 8];
        OsRng.fill_bytes(&mut b);
        let d = std::env::temp_dir().join(format!("izone-test-{}-{:x}", name, u64::from_le_bytes(b)));
        create_private_dir(&d).unwrap();
        d
    }

    #[test]
    fn hash_is_salted_and_verifies() {
        let a = hash_password("correct horse battery").unwrap();
        let b = hash_password("correct horse battery").unwrap();
        assert!(a.starts_with("$argon2id$"));
        assert_ne!(a, b, "same password must give different salted hashes");
        assert!(verify_password("correct horse battery", &a));
        assert!(!verify_password("correct horse batterY", &a));
        assert!(!verify_password("", &a));
        assert!(!verify_password("anything", "not a hash"));
        assert!(!verify_password("anything", ""));
    }

    #[test]
    fn new_password_rules() {
        assert!(check_new_password("short").is_err());
        assert!(check_new_password(&"x".repeat(MAX_PASSWORD_LEN + 1)).is_err());
        assert!(check_new_password("a long enough password").is_ok());
    }

    #[test]
    fn encryption_roundtrip_and_tamper_detection() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let msg = b"$argon2id$v=19$m=19456,t=2,p=1$salt$hash";
        let a = encrypt_with(&key, msg).unwrap();
        let b = encrypt_with(&key, msg).unwrap();
        assert_ne!(a, b, "random nonce: same plaintext must encrypt differently");
        assert!(!a.windows(msg.len()).any(|w| w == msg), "plaintext must not appear in output");
        assert_eq!(decrypt_with(&key, &a).unwrap(), msg);
        let mut tampered = a.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 1;
        assert!(decrypt_with(&key, &tampered).is_none(), "tampering must be detected");
        let mut other = key;
        other[0] ^= 1;
        assert!(decrypt_with(&other, &a).is_none(), "wrong key must fail");
        assert!(decrypt_with(&key, b"IZW1short").is_none());
        assert!(decrypt_with(&key, &[]).is_none());
    }

    #[test]
    fn key_file_is_private_and_removable() {
        let dir = tmp_dir("key");
        let p = dir.join("sub").join("web-password.key");
        assert_eq!(file_key::load(&p).unwrap(), None);
        let key = [7u8; 32];
        file_key::save(&p, &key).unwrap();
        assert_eq!(file_key::load(&p).unwrap(), Some(key));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&p).unwrap().permissions().mode() & 0o777, 0o600);
            assert_eq!(fs::metadata(p.parent().unwrap()).unwrap().permissions().mode() & 0o777, 0o700);
        }
        std::fs::write(&p, b"too short").unwrap();
        assert!(file_key::load(&p).is_err(), "corrupt key must be reported");
        assert!(file_key::delete(&p).unwrap());
        assert!(!file_key::delete(&p).unwrap());
        fs::remove_dir_all(dir).ok();
    }
}
