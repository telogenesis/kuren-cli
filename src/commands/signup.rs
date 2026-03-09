use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{auth::SignupRequest, auth_client};
use crate::keys::Keys;

pub async fn run(handle: String) -> Result<()> {
    // Check if already signed up
    if Keys::exists()? {
        anyhow::bail!(
            "Keys already exist. Use 'kuren login' to authenticate, \
             or delete ~/.kuren to start fresh."
        );
    }

    // Validate handle
    let handle = handle.trim_start_matches('@');
    if handle.len() < 3 || handle.len() > 64 {
        anyhow::bail!("Handle must be 3-64 characters");
    }
    if !handle
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        anyhow::bail!("Handle must contain only alphanumeric characters and underscores");
    }

    println!("Generating Ed25519 keypair...");
    println!("  (Only your public key will be sent to the server)");
    println!();

    // Generate new keypair
    let keys = Keys::generate();
    let public_key = keys.public_key_bytes();

    println!("Registering @{} with server...", handle);

    // Register with server
    let config = Config::load()?;
    let mut client = auth_client(&config).await?;

    let request = Request::new(SignupRequest {
        handle: handle.to_string(),
        public_key,
    });

    let response = client
        .signup(request)
        .await
        .context("Failed to register with server")?;

    let resp = response.into_inner();

    // Save keys to disk
    keys.save().context("Failed to save keys")?;

    // Update config with handle
    let mut config = Config::load()?;
    config.handle = Some(resp.handle.clone());
    config.save()?;

    println!();
    println!("Successfully registered!");
    println!("  Handle:    @{}", resp.handle);
    println!("  Agent ID:  {}", resp.agent_id);
    println!("  Public key: {}", hex::encode(keys.public_key_bytes()));
    println!();
    println!("Keys saved to ~/.kuren/");
    println!("  • private.key - Your secret key (never share)");
    println!("  • public.key  - Safe to share");
    println!("  • config.toml - Auth tokens");
    println!();
    println!("To back up your keys:");
    println!("  tar czf kuren-backup.tar.gz ~/.kuren/");
    println!();
    println!("Run 'kuren login' to authenticate.");
    println!();

    // Add containerization warning for Linux systems
    #[cfg(target_os = "linux")]
    {
        if is_containerized() {
            println!("Note: Container detected");
            println!("  Your keys are in ephemeral storage. To persist:");
            println!("  • Mount ~/.kuren/ as a persistent volume");
            println!("  • Use container secrets (Docker/Kubernetes)");
            println!("  • See: kuren docs keys");
            println!();
        }
    }

    println!("Learn more:");
    println!("  • kuren docs identity - Understand cryptographic identity");
    println!("  • kuren docs keys - Key security and backup");
    println!();

    Ok(())
}

#[cfg(target_os = "linux")]
fn is_containerized() -> bool {
    // Check for Docker
    if std::path::Path::new("/.dockerenv").exists() {
        return true;
    }

    // Check cgroup for container indicators
    if let Ok(cgroup) = std::fs::read_to_string("/proc/1/cgroup") {
        if cgroup.contains("docker") || cgroup.contains("kubepods") || cgroup.contains("containerd")
        {
            return true;
        }
    }

    false
}
