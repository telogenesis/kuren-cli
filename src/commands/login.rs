use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{auth::ChallengeRequest, auth::VerifyRequest, auth_client};
use crate::keys::Keys;

pub async fn run() -> Result<()> {
    // Load keys
    if !Keys::exists()? {
        anyhow::bail!("No keys found. Run 'kuren signup <handle>' first.");
    }

    let keys = Keys::load().context("Failed to load keys")?;
    let public_key = keys.public_key_bytes();

    println!("Authenticating with challenge-response protocol...");
    println!();
    println!("  1. Requesting challenge from server...");

    // Connect to server
    let config = Config::load()?;
    let mut client = auth_client(&config).await?;

    // Request challenge
    let challenge_req = Request::new(ChallengeRequest {
        public_key: public_key.clone(),
    });

    let challenge_resp = client
        .request_challenge(challenge_req)
        .await
        .context("Failed to request challenge")?
        .into_inner();

    println!("  2. Signing challenge with your private key...");

    // Sign challenge
    let signature = keys.sign(&challenge_resp.challenge);

    println!("  3. Verifying signature with server...");

    // Verify challenge
    let verify_req = Request::new(VerifyRequest {
        challenge_id: challenge_resp.challenge_id,
        signature: signature.to_bytes().to_vec(),
    });

    let token_resp = client
        .verify_challenge(verify_req)
        .await
        .context("Failed to verify challenge")?
        .into_inner();

    println!("  4. Receiving tokens...");
    println!();

    // Save tokens
    let mut config = Config::load()?;
    config.access_token = Some(token_resp.access_token);
    config.refresh_token = Some(token_resp.refresh_token);
    config.token_expires_at = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + token_resp.expires_in,
    );
    config.save()?;

    println!("Logged in successfully!");
    if let Some(handle) = &config.handle {
        println!("Welcome back, @{}!", handle);
    }
    println!();
    println!("Token expiry: Access (15 min) | Refresh (30 days)");

    Ok(())
}
