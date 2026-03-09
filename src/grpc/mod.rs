use anyhow::{Context, Result};
use tonic::transport::Channel;
use tonic::Request;

use crate::config::Config;

pub mod auth {
    tonic::include_proto!("kuren.auth");
}

pub mod payment {
    tonic::include_proto!("kuren.payment");
}

pub mod social {
    tonic::include_proto!("kuren.social");
}

pub mod messaging {
    tonic::include_proto!("kuren.messaging");
}

pub mod notifications {
    tonic::include_proto!("kuren.notifications");
}

pub mod email {
    tonic::include_proto!("kuren.email");
}

pub mod commerce {
    tonic::include_proto!("kuren.commerce");
}

pub mod notes {
    tonic::include_proto!("kuren.notes");
}

pub mod organization {
    tonic::include_proto!("kuren.organization");
}

pub mod wallet {
    tonic::include_proto!("kuren.wallet");
}

pub use auth::auth_service_client::AuthServiceClient;
pub use commerce::agent_commerce_service_client::AgentCommerceServiceClient;
pub use commerce::commerce_service_client::CommerceServiceClient;
pub use email::email_service_client::EmailServiceClient;
pub use messaging::messaging_service_client::MessagingServiceClient;
pub use notes::notes_service_client::NotesServiceClient;
pub use notifications::notification_service_client::NotificationServiceClient;
pub use organization::organization_service_client::OrganizationServiceClient;
pub use payment::payment_service_client::PaymentServiceClient;
pub use social::social_service_client::SocialServiceClient;
pub use wallet::wallet_service_client::WalletServiceClient;

/// Create a gRPC channel to the server
pub async fn connect(config: &Config) -> Result<Channel> {
    let url = config.server_url();
    let mut endpoint = Channel::from_shared(url.clone())
        .with_context(|| format!("Invalid server URL: {}", url))?;

    if url.starts_with("https://") {
        endpoint =
            endpoint.tls_config(tonic::transport::ClientTlsConfig::new().with_webpki_roots())?;
    }

    endpoint
        .connect()
        .await
        .with_context(|| format!("Failed to connect to server at {}", url))
}

/// Add authorization header to a request
pub fn with_auth<T>(mut request: Request<T>, token: &str) -> Request<T> {
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );
    request
}

/// Ensure we have a valid access token, refreshing if expired.
///
/// Returns the access token string. If the token is expired and a refresh
/// token is available, attempts to refresh automatically and saves the new
/// tokens to config.
pub async fn ensure_token(config: &mut Config) -> Result<String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // If we have an access token, check if it's still valid
    if let Some(token) = &config.access_token {
        match config.token_expires_at {
            Some(expires_at) if now >= expires_at - 30 => {
                // Token is expired (or about to expire), fall through to refresh
            }
            None => {
                // No expiry stored (old config), return token and let server validate
                return Ok(token.clone());
            }
            _ => {
                // Token is still valid
                return Ok(token.clone());
            }
        }
    }

    // Token is expired or missing — try to refresh
    if let Some(refresh_token) = config.refresh_token.clone() {
        let mut client = auth_client(config).await?;
        let request = Request::new(auth::RefreshRequest { refresh_token });

        match client.refresh_token(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                let new_token = resp.access_token.clone();
                config.access_token = Some(resp.access_token);
                config.refresh_token = Some(resp.refresh_token);
                config.token_expires_at = Some(now + resp.expires_in);
                config.save()?;
                return Ok(new_token);
            }
            Err(_) => {
                config.clear_tokens();
                config.save()?;
                anyhow::bail!("Session expired. Run 'kuren login' to re-authenticate.");
            }
        }
    }

    anyhow::bail!("Not logged in. Run 'kuren login' first.");
}

/// Create an authenticated auth service client
pub async fn auth_client(config: &Config) -> Result<AuthServiceClient<Channel>> {
    let channel = connect(config).await?;
    Ok(AuthServiceClient::new(channel))
}

/// Create an authenticated payment service client
pub async fn payment_client(config: &Config) -> Result<PaymentServiceClient<Channel>> {
    let channel = connect(config).await?;
    Ok(PaymentServiceClient::new(channel))
}

/// Create a social service client
pub async fn social_client(config: &Config) -> Result<SocialServiceClient<Channel>> {
    let channel = connect(config).await?;
    Ok(SocialServiceClient::new(channel))
}

/// Create a messaging service client
pub async fn messaging_client(config: &Config) -> Result<MessagingServiceClient<Channel>> {
    let channel = connect(config).await?;
    Ok(MessagingServiceClient::new(channel))
}

/// Create a notifications service client
pub async fn notifications_client(config: &Config) -> Result<NotificationServiceClient<Channel>> {
    let channel = connect(config).await?;
    Ok(NotificationServiceClient::new(channel))
}

/// Create an email service client
pub async fn email_client(config: &Config) -> Result<EmailServiceClient<Channel>> {
    let channel = connect(config).await?;
    Ok(EmailServiceClient::new(channel))
}

/// Create a commerce service client (for merchants)
pub async fn commerce_client(config: &Config) -> Result<CommerceServiceClient<Channel>> {
    let channel = connect(config).await?;
    Ok(CommerceServiceClient::new(channel))
}

/// Create an agent commerce service client
pub async fn agent_commerce_client(config: &Config) -> Result<AgentCommerceServiceClient<Channel>> {
    let channel = connect(config).await?;
    Ok(AgentCommerceServiceClient::new(channel))
}

/// Create a notes service client
pub async fn notes_client(config: &Config) -> Result<NotesServiceClient<Channel>> {
    let channel = connect(config).await?;
    Ok(NotesServiceClient::new(channel))
}

/// Create an organization service client
pub async fn organization_client(config: &Config) -> Result<OrganizationServiceClient<Channel>> {
    let channel = connect(config).await?;
    Ok(OrganizationServiceClient::new(channel))
}

/// Create a wallet service client
pub async fn wallet_client(config: &Config) -> Result<WalletServiceClient<Channel>> {
    let channel = connect(config).await?;
    Ok(WalletServiceClient::new(channel))
}
