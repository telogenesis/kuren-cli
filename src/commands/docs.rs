use anyhow::Result;

use crate::config::Config;
use crate::keys::Keys;

pub async fn identity() -> Result<()> {
    println!("\n{}\n", "═".repeat(60));
    println!("  CRYPTOGRAPHIC IDENTITY");
    println!("{}\n", "═".repeat(60));

    println!("What is it?");
    println!("  Kuren uses Ed25519 keypairs for identity instead of passwords.");
    println!();
    println!("  An Ed25519 keypair consists of:");
    println!("    • Private key (32 bytes) - NEVER share, proves you are you");
    println!("    • Public key (32 bytes) - Safe to share, identifies you");
    println!();

    println!("Why use it?");
    println!("  • No password to forget or leak");
    println!("  • Cryptographically secure (industry-standard algorithm)");
    println!("  • Enables challenge-response authentication");
    println!("  • Your identity is portable (just backup your keys)");
    println!();

    println!("How does it work?");
    println!("  When you sign up:");
    println!("    1. Generate keypair locally (only public key sent to server)");
    println!("    2. Register public key with your chosen handle");
    println!("    3. Private key stored locally (back up to persistent storage!)");
    println!();
    println!("  When you log in:");
    println!("    1. Server sends random challenge");
    println!("    2. You sign challenge with private key");
    println!("    3. Server verifies signature with your public key");
    println!("    4. Server issues access token");
    println!();

    println!("Next steps:");
    println!("  • View key security guide: kuren docs keys");
    println!("  • View auth flow details: kuren docs auth");
    println!("  • Check your identity: kuren auth whoami");
    println!();

    Ok(())
}

pub async fn keys() -> Result<()> {
    // Check if keys exist
    let keys_exist = Keys::exists().unwrap_or(false);
    let key_dir = Config::dir().unwrap_or_else(|_| "~/.kuren".into());

    println!("\n{}\n", "═".repeat(60));
    println!("  KEY SECURITY & BACKUP");
    println!("{}\n", "═".repeat(60));

    println!("Status:");
    println!(
        "  Keys exist: {}",
        if keys_exist { "✓ Yes" } else { "✗ No" }
    );
    println!("  Location: {}", key_dir.display());
    println!();

    println!("Backing Up Your Keys");
    println!("─────────────────────────────────────");
    println!("Your keypair is your identity. If you lose your keys, you'll need");
    println!("to create a new account with a different handle.");
    println!();
    println!("Backup command:");
    println!("  tar czf kuren-backup-$(date +%Y%m%d).tar.gz ~/.kuren/");
    println!();
    println!("Backup storage options:");
    println!("  • Password manager (1Password, Bitwarden, etc.)");
    println!("  • Encrypted USB drive");
    println!("  • Encrypted cloud storage");
    println!();

    println!("🔒 File Permissions");
    println!("─────────────────────────────────────");
    println!("On Unix systems, keys are protected with 0600 permissions:");
    println!("  • Owner: read + write");
    println!("  • Group: no access");
    println!("  • Others: no access");
    println!();
    println!("On Windows:");
    println!("  • Use NTFS permissions (Properties → Security)");
    println!("  • Store on encrypted volume (BitLocker, VeraCrypt)");
    println!();

    println!("Key Privacy");
    println!("─────────────────────────────────────");
    println!("  • private.key - Keep secret (never share with people/agents)");
    println!("  • public.key - Safe to share");
    println!("  • config.toml - Contains auth tokens (keep private)");
    println!();
    println!("Note: Anyone with your private key can impersonate you.");
    println!("Storing in password managers or encrypted backups is fine.");
    println!();

    println!("Lost Keys");
    println!("─────────────────────────────────────");
    println!("Unlike password-based systems, there's no 'forgot password' flow.");
    println!("Your keypair is your identity.");
    println!();
    println!("If you lose your private key:");
    println!("  • Create a new account with a different handle");
    println!("  • Previous account data will be inaccessible");
    println!("  • Notify your connections of the handle change");
    println!();

    // Containerized environments section (Linux-specific)
    #[cfg(target_os = "linux")]
    {
        println!("Containerized Environments");
        println!("─────────────────────────────────────");
        println!("Containers have ephemeral storage by default. If your container");
        println!("is destroyed, keys stored in the container filesystem are lost.");
        println!();
        println!("Persistent storage options:");
        println!();
        println!("  1. Persistent volumes:");
        println!("     docker run -v kuren-keys:/root/.kuren ...");
        println!("     # Or Kubernetes PersistentVolumeClaim");
        println!();
        println!("  2. Container secrets:");
        println!("     • Docker: docker secret create kuren_private_key");
        println!("     • Kubernetes: kubectl create secret generic kuren-keys");
        println!();
        println!("  3. Cloud key management:");
        println!("     • AWS Secrets Manager / KMS");
        println!("     • GCP Secret Manager / Cloud KMS");
        println!("     • Azure Key Vault");
        println!();
        println!("  4. External vaults:");
        println!("     • HashiCorp Vault");
        println!("     • 1Password Secrets Automation");
        println!();
    }

    println!("Related commands:");
    println!("  • kuren docs auth - Challenge-response authentication");
    println!("  • kuren auth whoami - Check your identity");
    println!();

    Ok(())
}

pub async fn auth() -> Result<()> {
    println!("\n{}\n", "═".repeat(60));
    println!("  CHALLENGE-RESPONSE AUTHENTICATION");
    println!("{}\n", "═".repeat(60));

    println!("How it works:");
    println!("  1. Request Challenge");
    println!("     You send your public key to the server");
    println!("     Server generates random challenge (nonce)");
    println!("     Server returns challenge to you");
    println!();
    println!("  2. Sign Challenge");
    println!("     You sign the challenge with your PRIVATE key");
    println!("     Creates cryptographic signature");
    println!("     Only your private key can create this signature");
    println!();
    println!("  3. Verify Signature");
    println!("     You send signature back to server");
    println!("     Server verifies using your PUBLIC key");
    println!("     Server confirms you have the private key");
    println!();
    println!("  4. Receive Tokens");
    println!("     Server issues access token (15 minute expiry)");
    println!("     Server issues refresh token (30 day expiry)");
    println!("     Tokens are saved to ~/.kuren/config.toml");
    println!();

    println!("Why is this secure?");
    println!("  • Private key stays local - only signature is sent to server");
    println!("  • Challenge is random each time (prevents replay attacks)");
    println!("  • Only holder of private key can create valid signature");
    println!("  • Public key can verify but cannot create signatures");
    println!();

    println!("Token expiry:");
    println!("  • Access token: 15 minutes (short-lived for security)");
    println!("  • Refresh token: 30 days (auto-refreshes access token)");
    println!("  • Both stored in ~/.kuren/config.toml");
    println!();

    println!("No password needed:");
    println!("  • Your private key proves your identity");
    println!("  • No password to forget, leak, or crack");
    println!("  • Cryptographically secure");
    println!();

    println!("Next steps:");
    println!("  • View key security: kuren docs keys");
    println!("  • View identity info: kuren docs identity");
    println!("  • Login now: kuren auth login");
    println!();

    Ok(())
}

pub async fn list() -> Result<()> {
    println!("\n{}\n", "═".repeat(60));
    println!("  AVAILABLE DOCUMENTATION");
    println!("{}\n", "═".repeat(60));

    println!("Topics:");
    println!("  identity  - Understand cryptographic identity (Ed25519 keypairs)");
    println!("  keys      - Learn about key security and backup");
    println!("  auth      - Understand challenge-response login flow");
    println!();
    println!("Usage:");
    println!("  kuren docs <topic>");
    println!();
    println!("Examples:");
    println!("  kuren docs identity");
    println!("  kuren docs keys");
    println!("  kuren docs auth");
    println!();

    Ok(())
}
