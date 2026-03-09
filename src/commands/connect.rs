use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{
    ensure_token, social::AcceptConnectionReq, social::DeclineConnectionReq,
    social::GetConnectionRequestsReq, social::GetConnectionsReq, social::RemoveConnectionReq,
    social::SendConnectionRequestReq, social_client, with_auth,
};

pub async fn send_request(to_handle: String, message: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let to_handle = to_handle.trim_start_matches('@').to_string();

    let mut client = social_client(&config).await?;
    let request = with_auth(
        Request::new(SendConnectionRequestReq {
            to_handle: to_handle.clone(),
            message: message.unwrap_or_default(),
        }),
        &token,
    );

    client
        .send_connection_request(request)
        .await
        .context("Failed to send connection request")?;

    println!("Connection request sent to @{}", to_handle);

    Ok(())
}

pub async fn list_requests() -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = social_client(&config).await?;
    let request = with_auth(Request::new(GetConnectionRequestsReq {}), &token);

    let response = client
        .get_connection_requests(request)
        .await
        .context("Failed to get connection requests")?
        .into_inner();

    if response.requests.is_empty() {
        println!("No pending connection requests.");
        return Ok(());
    }

    println!("Pending connection requests:\n");
    for req in response.requests {
        let name_display = if req.from_name.is_empty() {
            String::new()
        } else {
            format!(" ({})", req.from_name)
        };
        print!("@{}{}", req.from_handle, name_display);
        if !req.message.is_empty() {
            print!(": \"{}\"", req.message);
        }
        println!();
    }

    println!("\nUse 'kuren accept <handle>' or 'kuren decline <handle>' to respond.");

    Ok(())
}

pub async fn accept(from_handle: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let from_handle = from_handle.trim_start_matches('@').to_string();

    let mut client = social_client(&config).await?;
    let request = with_auth(
        Request::new(AcceptConnectionReq {
            from_handle: from_handle.clone(),
        }),
        &token,
    );

    client
        .accept_connection(request)
        .await
        .context("Failed to accept connection")?;

    println!("Connected with @{}", from_handle);

    Ok(())
}

pub async fn decline(from_handle: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let from_handle = from_handle.trim_start_matches('@').to_string();

    let mut client = social_client(&config).await?;
    let request = with_auth(
        Request::new(DeclineConnectionReq {
            from_handle: from_handle.clone(),
        }),
        &token,
    );

    client
        .decline_connection(request)
        .await
        .context("Failed to decline connection")?;

    println!("Declined connection request from @{}", from_handle);

    Ok(())
}

pub async fn list_connections() -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = social_client(&config).await?;
    let request = with_auth(Request::new(GetConnectionsReq {}), &token);

    let response = client
        .get_connections(request)
        .await
        .context("Failed to get connections")?
        .into_inner();

    if response.connections.is_empty() {
        println!("No connections yet.");
        return Ok(());
    }

    println!("Your connections:\n");
    for conn in response.connections {
        let name_display = if conn.name.is_empty() {
            String::new()
        } else {
            format!(" ({})", conn.name)
        };
        println!("@{}{}", conn.handle, name_display);
    }

    Ok(())
}

pub async fn disconnect(handle: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let handle = handle.trim_start_matches('@').to_string();

    let mut client = social_client(&config).await?;
    let request = with_auth(
        Request::new(RemoveConnectionReq {
            handle: handle.clone(),
        }),
        &token,
    );

    client
        .remove_connection(request)
        .await
        .context("Failed to remove connection")?;

    println!("Disconnected from @{}", handle);

    Ok(())
}
