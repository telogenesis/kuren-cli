use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{
    ensure_token,
    payment::ApproveWalletRequestRequest,
    payment::CreateWalletRequestRequest,
    payment::DenyWalletRequestRequest,
    payment::GetPendingConfirmationsRequest,
    payment::GetPendingWalletRequestsRequest,
    payment::GetSentWalletRequestsRequest,
    payment::GetTransactionsRequest,
    payment::RespondToConfirmationRequest,
    payment::SendPaymentRequest,
    payment::SubmitConfirmationRequest,
    payment_client, with_auth,
};

pub async fn send(
    to_handle: String,
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

    let to_handle = to_handle.trim_start_matches('@').to_string();

    println!("Sending {:.6} USDC to @{}...", amount, to_handle);

    let mut client = payment_client(&config).await?;
    let request = with_auth(
        Request::new(SendPaymentRequest {
            to_handle: to_handle.clone(),
            usdc_amount,
            memo: memo.unwrap_or_default(),
            channel: String::new(),
            wallet_id: wallet_id.unwrap_or_default(),
        }),
        &token,
    );

    let response = client
        .send_payment(request)
        .await
        .context("Failed to send payment")?
        .into_inner();

    let new_balance = response.new_balance as f64 / 1_000_000.0;

    println!("Payment sent!");
    println!("  Transaction ID: {}", response.transaction_id);
    println!("  Provider: {}", response.provider);
    println!("  Status: {}", response.status);
    if !response.settlement_status.is_empty() {
        println!("  Settlement: {}", response.settlement_status);
    }
    println!("  New balance: {:.6} USDC", new_balance);

    Ok(())
}

pub async fn history(limit: Option<u32>, wallet_id: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let limit = limit.unwrap_or(20) as i32;

    let mut client = payment_client(&config).await?;
    let request = with_auth(
        Request::new(GetTransactionsRequest {
            limit,
            cursor: String::new(),
            channel: String::new(),
            wallet_id: wallet_id.unwrap_or_default(),
        }),
        &token,
    );

    let response = client
        .get_transactions(request)
        .await
        .context("Failed to get transactions")?
        .into_inner();

    if response.transactions.is_empty() {
        println!("No transactions found.");
        return Ok(());
    }

    println!(
        "{:<8} {:<8} {:<12} {:<12} {:<15} {:<10} {}",
        "Type", "Provider", "From", "To", "Amount", "Status", "Memo"
    );
    println!("{}", "-".repeat(85));

    let my_handle = config.handle.as_deref().unwrap_or("");

    for tx in response.transactions {
        let amount = tx.usdc_amount as f64 / 1_000_000.0;
        let tx_type = if tx.from_handle.is_empty() {
            "DEPOSIT"
        } else if tx.from_handle == my_handle {
            "SENT"
        } else {
            "RECEIVED"
        };

        let from = if tx.from_handle.is_empty() {
            "-".to_string()
        } else {
            format!("@{}", tx.from_handle)
        };
        let to = format!("@{}", tx.to_handle);

        let status = if !tx.settlement_status.is_empty() {
            format!("{}/{}", tx.status, tx.settlement_status)
        } else {
            tx.status
        };

        println!(
            "{:<8} {:<8} {:<12} {:<12} {:>14.6} {:<10} {}",
            tx_type, tx.provider, from, to, amount, status, tx.memo
        );
    }

    Ok(())
}

pub async fn request(
    from_handle: String,
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

    let from_handle = from_handle.trim_start_matches('@').to_string();

    println!("Requesting {:.6} USDC from @{}...", amount, from_handle);

    let mut client = payment_client(&config).await?;
    let request = with_auth(
        Request::new(CreateWalletRequestRequest {
            to_handle: from_handle.clone(),
            usdc_amount,
            memo: memo.unwrap_or_default(),
            channel: String::new(),
            wallet_id: wallet_id.unwrap_or_default(),
        }),
        &token,
    );

    let response = client
        .create_wallet_request(request)
        .await
        .context("Failed to create wallet request")?
        .into_inner();

    println!("Request created!");
    println!("  Request ID: {}", response.id);
    println!("  Status: {}", response.status);
    println!("  Requested from: @{}", response.recipient_handle);
    let amount_display = response.usdc_amount as f64 / 1_000_000.0;
    println!("  Amount: {:.6} USDC", amount_display);
    if !response.memo.is_empty() {
        println!("  Memo: {}", response.memo);
    }

    Ok(())
}

pub async fn requests(sent: bool, limit: Option<u32>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let limit = limit.unwrap_or(20) as i32;

    let mut client = payment_client(&config).await?;

    let requests = if sent {
        let request = with_auth(
            Request::new(GetSentWalletRequestsRequest {
                limit,
                channel: String::new(),
            }),
            &token,
        );
        client
            .get_sent_wallet_requests(request)
            .await
            .context("Failed to get sent requests")?
            .into_inner()
            .requests
    } else {
        let request = with_auth(
            Request::new(GetPendingWalletRequestsRequest {
                limit,
                channel: String::new(),
            }),
            &token,
        );
        client
            .get_pending_wallet_requests(request)
            .await
            .context("Failed to get pending requests")?
            .into_inner()
            .requests
    };

    if requests.is_empty() {
        if sent {
            println!("No sent requests found.");
        } else {
            println!("No pending requests.");
        }
        return Ok(());
    }

    let header = if sent {
        "Sent Requests"
    } else {
        "Pending Requests"
    };
    println!("{}", header);
    println!("{}", "=".repeat(header.len()));
    println!();

    for req in requests {
        let amount = req.usdc_amount as f64 / 1_000_000.0;
        println!("ID: {}", req.id);
        if sent {
            println!("  To: @{}", req.recipient_handle);
        } else {
            println!("  From: @{}", req.requester_handle);
        }
        println!("  Amount: {:.6} USDC", amount);
        println!("  Provider: {}", req.provider);
        println!("  Status: {}", req.status);
        if !req.memo.is_empty() {
            println!("  Memo: {}", req.memo);
        }
        println!();
    }

    if !sent {
        println!("Use 'kuren pay request approve <id>' or 'kuren pay request deny <id>' to respond.");
    }

    Ok(())
}

pub async fn approve(request_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    println!("Approving request {}...", request_id);

    let mut client = payment_client(&config).await?;
    let request = with_auth(
        Request::new(ApproveWalletRequestRequest {
            request_id: request_id.clone(),
            channel: String::new(),
        }),
        &token,
    );

    let response = client
        .approve_wallet_request(request)
        .await
        .context("Failed to approve request")?
        .into_inner();

    let req = response.request.unwrap();
    let amount = req.usdc_amount as f64 / 1_000_000.0;
    let new_balance = response.new_balance as f64 / 1_000_000.0;

    println!("Request approved!");
    println!("  Paid {:.6} USDC to @{}", amount, req.requester_handle);
    println!("  Transaction ID: {}", response.transaction_id);
    println!("  New balance: {:.6} USDC", new_balance);

    Ok(())
}

pub async fn deny(request_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    println!("Denying request {}...", request_id);

    let mut client = payment_client(&config).await?;
    let request = with_auth(
        Request::new(DenyWalletRequestRequest {
            request_id: request_id.clone(),
            channel: String::new(),
        }),
        &token,
    );

    let response = client
        .deny_wallet_request(request)
        .await
        .context("Failed to deny request")?
        .into_inner();

    println!("Request denied.");
    println!(
        "  Request from @{} for {:.6} USDC was denied.",
        response.requester_handle,
        response.usdc_amount as f64 / 1_000_000.0
    );

    Ok(())
}

/// Submit a settlement confirmation for a Kuren IOU transaction
pub async fn confirm(transaction_id: String, memo: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    println!("Submitting settlement confirmation for {}...", transaction_id);

    let mut client = payment_client(&config).await?;
    let request = with_auth(
        Request::new(SubmitConfirmationRequest {
            transaction_id: transaction_id.clone(),
            memo: memo.unwrap_or_default(),
        }),
        &token,
    );

    let response = client
        .submit_confirmation(request)
        .await
        .context("Failed to submit confirmation")?
        .into_inner();

    println!("Confirmation submitted!");
    println!("  Confirmation ID: {}", response.id);
    println!("  Status: {}", response.status);
    if !response.memo.is_empty() {
        println!("  Memo: {}", response.memo);
    }

    Ok(())
}

/// Respond to a settlement confirmation request
pub async fn settle(confirmation_id: String, reject: bool) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let action = if reject { "Rejecting" } else { "Confirming" };
    println!("{} settlement {}...", action, confirmation_id);

    let mut client = payment_client(&config).await?;
    let request = with_auth(
        Request::new(RespondToConfirmationRequest {
            confirmation_id: confirmation_id.clone(),
            confirm: !reject,
        }),
        &token,
    );

    let response = client
        .respond_to_confirmation(request)
        .await
        .context("Failed to respond to confirmation")?
        .into_inner();

    println!("Settlement {}!", response.status);
    println!("  Confirmation ID: {}", response.id);
    println!("  Transaction: {}", response.transaction_id);
    if !response.memo.is_empty() {
        println!("  Memo: {}", response.memo);
    }

    Ok(())
}

/// List pending settlement confirmation requests
pub async fn settlements(limit: Option<u32>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let limit = limit.unwrap_or(20) as i32;

    let mut client = payment_client(&config).await?;
    let request = with_auth(
        Request::new(GetPendingConfirmationsRequest { limit }),
        &token,
    );

    let response = client
        .get_pending_confirmations(request)
        .await
        .context("Failed to get pending confirmations")?
        .into_inner();

    if response.confirmations.is_empty() {
        println!("No pending settlement confirmations.");
        return Ok(());
    }

    println!("Pending Settlement Confirmations");
    println!("================================");
    println!();

    for c in response.confirmations {
        println!("ID: {}", c.id);
        println!("  Transaction: {}", c.transaction_id);
        println!("  From: @{}", c.requested_by_handle);
        println!("  Status: {}", c.status);
        if !c.memo.is_empty() {
            println!("  Memo: {}", c.memo);
        }
        println!();
    }

    println!("Use 'kuren pay settle respond <id>' to confirm or 'kuren pay settle respond <id> --reject' to reject.");

    Ok(())
}
