use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{auth::GetProfileRequest, auth_client, ensure_token, with_auth};
use crate::keys::Keys;

pub async fn run() -> Result<()> {
    let config = Config::load()?;

    // Show local info even if not logged in
    if Keys::exists()? {
        let keys = Keys::load()?;
        println!("Local keys found:");
        println!("  Public key: {}", hex::encode(keys.public_key_bytes()));
        println!();
    } else {
        println!("No local keys found. Run 'kuren signup <handle>' to create an identity.");
        return Ok(());
    }

    // If logged in, fetch profile from server
    let mut config = config;
    match ensure_token(&mut config).await {
        Ok(token) => {
            let mut client = auth_client(&config).await?;
            let request = with_auth(Request::new(GetProfileRequest {}), &token);

            let response = client
                .get_profile(request)
                .await
                .context("Failed to fetch profile")?;

            let profile = response.into_inner();
            println!("Server profile:");
            println!("  Handle:     @{}", profile.handle);
            println!("  Agent ID:   {}", profile.agent_id);
            println!("  Public key: {}", hex::encode(&profile.public_key));
            println!("  Created:    {}", profile.created_at);
        }
        Err(_) => {
            if let Some(handle) = &config.handle {
                println!("Registered handle: @{}", handle);
            }
            println!("Not logged in. Run 'kuren login' to authenticate.");
        }
    }

    Ok(())
}
