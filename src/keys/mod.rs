use anyhow::{Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use std::path::PathBuf;

use crate::config::Config;

const PRIVATE_KEY_FILE: &str = "private.key";
const PUBLIC_KEY_FILE: &str = "public.key";

/// Ed25519 keypair manager
pub struct Keys {
    signing_key: SigningKey,
}

impl Keys {
    /// Generate a new keypair
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Load keypair from disk
    pub fn load() -> Result<Self> {
        let path = Self::private_key_path()?;
        let bytes = std::fs::read(&path)
            .with_context(|| format!("Failed to read private key: {}", path.display()))?;

        if bytes.len() != 32 {
            anyhow::bail!("Invalid private key size");
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);
        let signing_key = SigningKey::from_bytes(&key_bytes);

        Ok(Self { signing_key })
    }

    /// Check if keys exist on disk
    pub fn exists() -> Result<bool> {
        Ok(Self::private_key_path()?.exists())
    }

    /// Save keypair to disk
    pub fn save(&self) -> Result<()> {
        let dir = Config::dir()?;
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;

        // Save private key
        let priv_path = Self::private_key_path()?;
        std::fs::write(&priv_path, self.signing_key.to_bytes())
            .with_context(|| format!("Failed to write private key: {}", priv_path.display()))?;

        // Set restrictive permissions on private key
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&priv_path, std::fs::Permissions::from_mode(0o600))?;
        }

        // Save public key
        let pub_path = Self::public_key_path()?;
        std::fs::write(&pub_path, self.public_key_bytes())
            .with_context(|| format!("Failed to write public key: {}", pub_path.display()))?;

        Ok(())
    }

    /// Get public key bytes (32 bytes)
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }

    /// Get verifying (public) key
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Get path to private key file
    fn private_key_path() -> Result<PathBuf> {
        Ok(Config::dir()?.join(PRIVATE_KEY_FILE))
    }

    /// Get path to public key file
    fn public_key_path() -> Result<PathBuf> {
        Ok(Config::dir()?.join(PUBLIC_KEY_FILE))
    }
}
