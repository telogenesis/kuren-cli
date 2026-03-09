use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{
    ensure_token, social::GetProfileRequest, social::UpdateProfileRequest, social_client, with_auth,
};

pub async fn view(handle: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = social_client(&config).await?;

    let handle = handle
        .map(|h| h.trim_start_matches('@').to_string())
        .unwrap_or_default();

    let request = with_auth(Request::new(GetProfileRequest { handle }), &token);

    let response = client
        .get_profile(request)
        .await
        .context("Failed to get profile")?
        .into_inner();

    let profile = response
        .profile
        .ok_or_else(|| anyhow::anyhow!("No profile returned"))?;

    println!(
        "Name: {}",
        if profile.name.is_empty() {
            "(not set)"
        } else {
            &profile.name
        }
    );
    println!("Handle: @{}", profile.handle);
    println!(
        "Bio: {}",
        if profile.bio.is_empty() {
            "(not set)"
        } else {
            &profile.bio
        }
    );
    println!(
        "Account: {}",
        if profile.is_public {
            "Public"
        } else {
            "Private"
        }
    );
    println!("Member since: {}", profile.created_at);

    Ok(())
}

pub async fn set(name: Option<String>, bio: Option<String>, is_public: Option<bool>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    if name.is_none() && bio.is_none() && is_public.is_none() {
        anyhow::bail!("At least one of --name, --bio, --public, or --private is required");
    }

    let mut client = social_client(&config).await?;

    let request = with_auth(
        Request::new(UpdateProfileRequest {
            name,
            bio,
            is_public,
        }),
        &token,
    );

    let response = client
        .update_profile(request)
        .await
        .context("Failed to update profile")?
        .into_inner();

    let profile = response
        .profile
        .ok_or_else(|| anyhow::anyhow!("No profile returned"))?;

    println!("Profile updated!");
    println!(
        "Name: {}",
        if profile.name.is_empty() {
            "(not set)"
        } else {
            &profile.name
        }
    );
    println!("Handle: @{}", profile.handle);
    println!(
        "Bio: {}",
        if profile.bio.is_empty() {
            "(not set)"
        } else {
            &profile.bio
        }
    );
    println!(
        "Account: {}",
        if profile.is_public {
            "Public"
        } else {
            "Private"
        }
    );

    Ok(())
}
