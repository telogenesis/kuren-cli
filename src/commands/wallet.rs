use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{
    ensure_token,
    payment::GetBalanceRequest,
    payment::OffRampRequest,
    payment_client,
    wallet::AddWalletMemberRequest,
    wallet::CreateWalletRequest,
    wallet::GetWalletsRequest,
    wallet::RemoveWalletMemberRequest,
    wallet::SetDefaultWalletRequest,
    wallet_client, with_auth,
};

/// List all wallets for the current agent
pub async fn list() -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = wallet_client(&config).await?;
    let request = with_auth(Request::new(GetWalletsRequest {}), &token);

    let response = client
        .get_wallets(request)
        .await
        .context("Failed to get wallets")?
        .into_inner();

    if response.wallets.is_empty() {
        println!("No wallets found. Create one with 'kuren wallet create <provider>'");
        return Ok(());
    }

    println!(
        "{:<38} {:<8} {:<12} {:<14} {:<8} {}",
        "ID", "Provider", "Name", "Balance", "Default", "Network"
    );
    println!("{}", "-".repeat(90));

    for w in response.wallets {
        let balance = w.balance as f64 / 1_000_000.0;
        let name = if w.display_name.is_empty() {
            "-".to_string()
        } else {
            w.display_name
        };
        let default_marker = if w.is_default { "*" } else { "" };
        let network = if w.network.is_empty() {
            "-".to_string()
        } else {
            w.network
        };

        println!(
            "{:<38} {:<8} {:<12} {:>13.6} {:<8} {}",
            w.id, w.provider, name, balance, default_marker, network
        );
    }

    Ok(())
}

/// Create a new wallet
pub async fn create(provider: String, name: Option<String>, network: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    println!("Creating {} wallet...", provider);

    let mut client = wallet_client(&config).await?;
    let request = with_auth(
        Request::new(CreateWalletRequest {
            provider: provider.clone(),
            display_name: name.unwrap_or_default(),
            network: network.clone(),
        }),
        &token,
    );

    let response = client
        .create_wallet(request)
        .await
        .context("Failed to create wallet")?
        .into_inner();

    println!("Wallet created!");
    println!("  ID: {}", response.id);
    println!("  Provider: {}", response.provider);
    if !response.display_name.is_empty() {
        println!("  Name: {}", response.display_name);
    }
    if !response.solana_address.is_empty() {
        println!("  Solana address: {}", response.solana_address);
        println!("  Network: {}", response.network);
    }
    println!();
    println!("Set as default: kuren wallet default {}", response.id);

    Ok(())
}

/// Set a wallet as the default
pub async fn set_default(wallet_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = wallet_client(&config).await?;
    let request = with_auth(
        Request::new(SetDefaultWalletRequest {
            wallet_id: wallet_id.clone(),
        }),
        &token,
    );

    let response = client
        .set_default_wallet(request)
        .await
        .context("Failed to set default wallet")?
        .into_inner();

    println!("Default wallet set to {} ({})", response.id, response.provider);

    Ok(())
}

pub async fn balance(wallet_id: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = payment_client(&config).await?;
    let request = with_auth(
        Request::new(GetBalanceRequest {
            channel: String::new(),
            wallet_id: wallet_id.unwrap_or_default(),
        }),
        &token,
    );

    let response = client
        .get_balance(request)
        .await
        .context("Failed to get balance")?
        .into_inner();

    let usdc = response.usdc_amount as f64 / 1_000_000.0;

    println!("Balance: {:.6} USDC ({})", usdc, response.provider);
    if !response.wallet_id.is_empty() {
        println!("Wallet: {}", response.wallet_id);
    }
    if !response.solana_address.is_empty() {
        println!("Deposit address: {}", response.solana_address);
    }

    Ok(())
}

/// Add a member to a wallet
pub async fn member_add(wallet_id: String, handle: String, permission: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let handle = handle.trim_start_matches('@').to_string();
    let permissions: Vec<String> = permission.split(',').map(|s| s.trim().to_string()).collect();

    println!(
        "Adding @{} to wallet {} with permissions: {}...",
        handle, wallet_id, permissions.join(", ")
    );

    let mut client = wallet_client(&config).await?;
    let request = with_auth(
        Request::new(AddWalletMemberRequest {
            wallet_id: wallet_id.clone(),
            agent_handle: handle.clone(),
            permissions: permissions.clone(),
        }),
        &token,
    );

    client
        .add_wallet_member(request)
        .await
        .context("Failed to add wallet member")?;

    println!("Added @{} to wallet {} with permissions: [{}].", handle, wallet_id, permissions.join(", "));

    Ok(())
}

/// Remove a member from a wallet
pub async fn member_remove(wallet_id: String, handle: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let handle = handle.trim_start_matches('@').to_string();

    println!("Removing @{} from wallet {}...", handle, wallet_id);

    let mut client = wallet_client(&config).await?;
    let request = with_auth(
        Request::new(RemoveWalletMemberRequest {
            wallet_id: wallet_id.clone(),
            agent_handle: handle.clone(),
        }),
        &token,
    );

    client
        .remove_wallet_member(request)
        .await
        .context("Failed to remove wallet member")?;

    println!("Removed @{} from wallet {}.", handle, wallet_id);

    Ok(())
}

pub async fn withdraw(
    address: String,
    amount: f64,
    memo: Option<String>,
    wallet_id: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    if amount <= 0.0 {
        anyhow::bail!("Amount must be positive");
    }

    let usdc_amount = (amount * 1_000_000.0) as i64;
    if usdc_amount <= 0 {
        anyhow::bail!("Amount too small");
    }

    println!("Withdrawing {:.6} USDC to {}...", amount, address);

    let mut client = payment_client(&config).await?;
    let request = with_auth(
        Request::new(OffRampRequest {
            wallet_id: wallet_id.unwrap_or_default(),
            destination_address: address.clone(),
            usdc_amount,
            memo: memo.unwrap_or_default(),
        }),
        &token,
    );

    let response = client
        .off_ramp(request)
        .await
        .context("Failed to withdraw")?
        .into_inner();

    let new_balance = response.new_balance as f64 / 1_000_000.0;

    println!("Withdrawal initiated!");
    println!("  Withdrawal ID: {}", response.withdrawal_id);
    println!("  Solana TX: {}", response.solana_tx_sig);
    println!("  Status: {}", response.status);
    println!("  New balance: {:.6} USDC", new_balance);

    Ok(())
}
