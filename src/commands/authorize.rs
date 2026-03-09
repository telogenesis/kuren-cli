use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{auth::ApproveDeviceAuthRequest, auth_client, ensure_token, with_auth};

pub async fn run(user_code: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    // Normalize user code (uppercase, handle with/without dash)
    let user_code = user_code.to_uppercase().replace(' ', "-");

    println!("Approving authorization for code: {}", user_code);

    let mut client = auth_client(&config).await?;
    let request = with_auth(Request::new(ApproveDeviceAuthRequest { user_code }), &token);

    let response = client
        .approve_device_auth(request)
        .await
        .context("Failed to approve authorization")?
        .into_inner();

    println!();
    println!("Approved!");
    println!("  Application: {}", response.client_name);
    println!("  Scopes granted: {}", response.scopes.join(", "));
    println!();
    println!("The application can now access your Kuren account with the permissions above.");

    Ok(())
}
