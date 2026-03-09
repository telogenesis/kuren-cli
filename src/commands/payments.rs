use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{
    agent_commerce_client,
    commerce::{
        ApprovePaymentRequestRequest, CancelMySubscriptionRequest, DenyPaymentRequestRequest,
        GetMySubscriptionRequest, GetPendingPaymentRequestsRequest, ListMySubscriptionsRequest,
        ListPurchaseHistoryRequest,
    },
    ensure_token, with_auth,
};

/// List pending payment requests from merchants
pub async fn pending() -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = agent_commerce_client(&config).await?;
    let request = with_auth(Request::new(GetPendingPaymentRequestsRequest {}), &token);

    let response = client
        .get_pending_payment_requests(request)
        .await
        .context("Failed to get pending payment requests")?
        .into_inner();

    if response.payment_requests.is_empty() {
        println!("No pending payment requests.");
        return Ok(());
    }

    println!(
        "{:<36} {:<20} {:<20} {:<15} {:<12}",
        "ID", "Merchant", "Product", "Amount", "Type"
    );
    println!("{}", "-".repeat(110));

    for req in response.payment_requests {
        let amount = req.usdc_amount as f64 / 1_000_000.0;
        let payment_type = if req.is_subscription {
            format!("Subscription ({})", req.billing_interval)
        } else {
            "One-time".to_string()
        };
        let product = if req.product_name.is_empty() {
            "-".to_string()
        } else {
            req.product_name.clone()
        };

        println!(
            "{:<36} {:<20} {:<20} {:>14.6} {:<12}",
            req.id,
            truncate(&req.merchant_name, 18),
            truncate(&product, 18),
            amount,
            payment_type
        );
    }

    println!();
    println!("Use 'kuren payments approve <id>' to approve a request.");
    println!("Use 'kuren payments deny <id>' to deny a request.");

    Ok(())
}

/// Approve a payment request
pub async fn approve(request_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = agent_commerce_client(&config).await?;
    let request = with_auth(
        Request::new(ApprovePaymentRequestRequest {
            request_id: request_id.clone(),
        }),
        &token,
    );

    let response = client
        .approve_payment_request(request)
        .await
        .context("Failed to approve payment request")?
        .into_inner();

    let new_balance = response.new_balance as f64 / 1_000_000.0;

    println!("Payment approved!");

    if let Some(pr) = &response.payment_request {
        let amount = pr.usdc_amount as f64 / 1_000_000.0;
        println!("  Merchant: {}", pr.merchant_name);
        if !pr.product_name.is_empty() {
            println!("  Product: {}", pr.product_name);
        }
        println!("  Amount: {:.6} USDC", amount);
    }

    if let Some(sub) = &response.subscription {
        println!();
        println!("Subscription created:");
        println!("  Subscription ID: {}", sub.id);
        println!("  Status: {}", sub.status);
        println!(
            "  Current period: {} to {}",
            sub.current_period_start, sub.current_period_end
        );
        if !sub.next_billing_at.is_empty() {
            println!("  Next billing: {}", sub.next_billing_at);
        }
    }

    println!();
    println!("New balance: {:.6} USDC", new_balance);

    Ok(())
}

/// Deny a payment request
pub async fn deny(request_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = agent_commerce_client(&config).await?;
    let request = with_auth(
        Request::new(DenyPaymentRequestRequest {
            request_id: request_id.clone(),
        }),
        &token,
    );

    let _response = client
        .deny_payment_request(request)
        .await
        .context("Failed to deny payment request")?
        .into_inner();

    println!("Payment request denied.");

    Ok(())
}

/// List active subscriptions
pub async fn subscriptions(status: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = agent_commerce_client(&config).await?;
    let request = with_auth(
        Request::new(ListMySubscriptionsRequest {
            status: status.unwrap_or_default(),
            limit: 50,
            cursor: String::new(),
        }),
        &token,
    );

    let response = client
        .list_my_subscriptions(request)
        .await
        .context("Failed to list subscriptions")?
        .into_inner();

    if response.subscriptions.is_empty() {
        println!("No subscriptions found.");
        return Ok(());
    }

    println!(
        "{:<36} {:<20} {:<20} {:<15} {:<10}",
        "ID", "Merchant", "Product", "Amount", "Status"
    );
    println!("{}", "-".repeat(105));

    for sub in response.subscriptions {
        let amount = sub.price_usdc as f64 / 1_000_000.0;
        let status_display = if sub.cancel_at_period_end {
            format!("{} (canceling)", sub.status)
        } else {
            sub.status.clone()
        };

        println!(
            "{:<36} {:<20} {:<20} {:>14.6} {:<10}",
            sub.id,
            truncate(&sub.merchant_name, 18),
            truncate(&sub.product_name, 18),
            amount,
            status_display
        );
    }

    println!();
    println!("Use 'kuren payments subscriptions cancel <id>' to cancel a subscription.");

    Ok(())
}

/// Get subscription details
pub async fn subscription_info(subscription_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = agent_commerce_client(&config).await?;
    let request = with_auth(
        Request::new(GetMySubscriptionRequest {
            subscription_id: subscription_id.clone(),
        }),
        &token,
    );

    let response = client
        .get_my_subscription(request)
        .await
        .context("Failed to get subscription")?
        .into_inner();

    let amount = response.price_usdc as f64 / 1_000_000.0;

    println!("Subscription Details");
    println!("{}", "=".repeat(40));
    println!("ID: {}", response.id);
    println!("Merchant: {}", response.merchant_name);
    println!("Product: {}", response.product_name);
    println!("Amount: {:.6} USDC", amount);
    println!("Status: {}", response.status);
    println!();
    println!("Current Period:");
    println!("  Start: {}", response.current_period_start);
    println!("  End: {}", response.current_period_end);
    if !response.trial_end.is_empty() {
        println!("  Trial ends: {}", response.trial_end);
    }
    if !response.next_billing_at.is_empty() {
        println!("  Next billing: {}", response.next_billing_at);
    }
    if response.cancel_at_period_end {
        println!();
        println!("Note: This subscription will be canceled at the end of the current period.");
    }
    if !response.canceled_at.is_empty() {
        println!();
        println!("Canceled at: {}", response.canceled_at);
    }

    Ok(())
}

/// Cancel a subscription
pub async fn cancel_subscription(subscription_id: String, immediate: bool) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = agent_commerce_client(&config).await?;
    let request = with_auth(
        Request::new(CancelMySubscriptionRequest {
            subscription_id: subscription_id.clone(),
            immediate,
        }),
        &token,
    );

    let response = client
        .cancel_my_subscription(request)
        .await
        .context("Failed to cancel subscription")?
        .into_inner();

    if immediate {
        println!("Subscription canceled immediately.");
        if response.refund_amount > 0 {
            let refund = response.refund_amount as f64 / 1_000_000.0;
            println!("Refund amount: {:.6} USDC", refund);
        }
    } else {
        println!("Subscription will be canceled at the end of the current billing period.");
        if let Some(sub) = &response.subscription {
            println!("Access continues until: {}", sub.current_period_end);
        }
    }

    Ok(())
}

/// List purchase history
pub async fn history(limit: Option<u32>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let limit = limit.unwrap_or(20) as i32;

    let mut client = agent_commerce_client(&config).await?;
    let request = with_auth(
        Request::new(ListPurchaseHistoryRequest {
            limit,
            cursor: String::new(),
        }),
        &token,
    );

    let response = client
        .list_purchase_history(request)
        .await
        .context("Failed to get purchase history")?
        .into_inner();

    if response.entries.is_empty() {
        println!("No purchase history found.");
        return Ok(());
    }

    println!(
        "{:<20} {:<20} {:<15} {:<20} {:<25}",
        "Merchant", "Product", "Amount", "Type", "Date"
    );
    println!("{}", "-".repeat(105));

    for entry in response.entries {
        let amount = entry.usdc_amount as f64 / 1_000_000.0;
        let entry_type = match entry.r#type.as_str() {
            "one_time" => "One-time",
            "subscription_start" => "Subscription Start",
            "subscription_renewal" => "Renewal",
            "refund" => "Refund",
            other => other,
        };

        println!(
            "{:<20} {:<20} {:>14.6} {:<20} {:<25}",
            truncate(&entry.merchant_name, 18),
            truncate(&entry.product_name, 18),
            amount,
            entry_type,
            entry.created_at
        );
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
