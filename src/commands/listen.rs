use anyhow::{Context, Result};
use tokio_stream::StreamExt;
use tonic::Request;

use crate::config::Config;
use crate::grpc::{
    ensure_token,
    notifications::{
        kuren_event::Event, ConnectionNotification, DmNotification, EmailNotification,
        GroupNotification, PaymentNotification, PaymentRequestNotification, SubscribeRequest,
        SubscriptionNotification, WalletRequestNotification,
    },
    notifications_client, with_auth,
};

/// Listen for all notifications with optional filtering
pub async fn listen(only: Option<Vec<String>>, exclude: Option<Vec<String>>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = notifications_client(&config).await?;

    let request = with_auth(
        Request::new(SubscribeRequest {
            include_categories: only.unwrap_or_default(),
            exclude_categories: exclude.unwrap_or_default(),
            include_types: vec![],
            exclude_types: vec![],
        }),
        &token,
    );

    println!("Listening for notifications... (Ctrl+C to stop)\n");

    let mut stream = client
        .subscribe(request)
        .await
        .context("Failed to subscribe to notifications")?
        .into_inner();

    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                print_event(&event.event);
            }
            Err(e) => {
                eprintln!("Error receiving notification: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Listen for DM notifications only
pub async fn listen_dm() -> Result<()> {
    listen(Some(vec!["dm".to_string()]), None).await
}

/// Listen for connection notifications with optional filtering
pub async fn listen_connection(
    only: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = notifications_client(&config).await?;

    // Convert type filters to full paths (e.g., "request" -> "connection.request")
    let include_types: Vec<String> = only
        .unwrap_or_default()
        .into_iter()
        .map(|t| format!("connection.{}", t))
        .collect();
    let exclude_types: Vec<String> = exclude
        .unwrap_or_default()
        .into_iter()
        .map(|t| format!("connection.{}", t))
        .collect();

    let request = with_auth(
        Request::new(SubscribeRequest {
            include_categories: if include_types.is_empty() {
                vec!["connection".to_string()]
            } else {
                vec![]
            },
            exclude_categories: vec![],
            include_types,
            exclude_types,
        }),
        &token,
    );

    println!("Listening for connection notifications... (Ctrl+C to stop)\n");

    let mut stream = client
        .subscribe(request)
        .await
        .context("Failed to subscribe to notifications")?
        .into_inner();

    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                print_event(&event.event);
            }
            Err(e) => {
                eprintln!("Error receiving notification: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Listen for group notifications with optional filtering
pub async fn listen_group(only: Option<Vec<String>>, exclude: Option<Vec<String>>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = notifications_client(&config).await?;

    // Convert type filters to full paths (e.g., "message" -> "group.message")
    let include_types: Vec<String> = only
        .unwrap_or_default()
        .into_iter()
        .map(|t| format!("group.{}", t))
        .collect();
    let exclude_types: Vec<String> = exclude
        .unwrap_or_default()
        .into_iter()
        .map(|t| format!("group.{}", t))
        .collect();

    let request = with_auth(
        Request::new(SubscribeRequest {
            include_categories: if include_types.is_empty() {
                vec!["group".to_string()]
            } else {
                vec![]
            },
            exclude_categories: vec![],
            include_types,
            exclude_types,
        }),
        &token,
    );

    println!("Listening for group notifications... (Ctrl+C to stop)\n");

    let mut stream = client
        .subscribe(request)
        .await
        .context("Failed to subscribe to notifications")?
        .into_inner();

    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                print_event(&event.event);
            }
            Err(e) => {
                eprintln!("Error receiving notification: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Listen for payment notifications
pub async fn listen_payment() -> Result<()> {
    listen(Some(vec!["payment".to_string()]), None).await
}

/// Listen for email notifications with optional filtering
pub async fn listen_email(only: Option<Vec<String>>, exclude: Option<Vec<String>>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = notifications_client(&config).await?;

    // Convert type filters to full paths (e.g., "received" -> "email.received")
    let include_types: Vec<String> = only
        .unwrap_or_default()
        .into_iter()
        .map(|t| format!("email.{}", t))
        .collect();
    let exclude_types: Vec<String> = exclude
        .unwrap_or_default()
        .into_iter()
        .map(|t| format!("email.{}", t))
        .collect();

    let request = with_auth(
        Request::new(SubscribeRequest {
            include_categories: if include_types.is_empty() {
                vec!["email".to_string()]
            } else {
                vec![]
            },
            exclude_categories: vec![],
            include_types,
            exclude_types,
        }),
        &token,
    );

    println!("Listening for email notifications... (Ctrl+C to stop)\n");

    let mut stream = client
        .subscribe(request)
        .await
        .context("Failed to subscribe to notifications")?
        .into_inner();

    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                print_event(&event.event);
            }
            Err(e) => {
                eprintln!("Error receiving notification: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn print_event(event: &Option<Event>) {
    match event {
        Some(Event::Dm(dm)) => print_dm(dm),
        Some(Event::Connection(conn)) => print_connection(conn),
        Some(Event::Payment(payment)) => print_payment(payment),
        Some(Event::Group(group)) => print_group(group),
        Some(Event::Email(email)) => print_email(email),
        Some(Event::PaymentRequest(pr)) => print_payment_request(pr),
        Some(Event::Subscription(sub)) => print_subscription(sub),
        Some(Event::WalletRequest(wr)) => print_wallet_request(wr),
        None => {}
    }
}

fn print_dm(dm: &DmNotification) {
    let name = if dm.from_name.is_empty() {
        format!("@{}", dm.from_handle)
    } else {
        format!("{} (@{})", dm.from_name, dm.from_handle)
    };
    println!("[DM] {}: \"{}\"", name, dm.preview);
}

fn print_connection(conn: &ConnectionNotification) {
    let name = if conn.from_name.is_empty() {
        format!("@{}", conn.from_handle)
    } else {
        format!("{} (@{})", conn.from_name, conn.from_handle)
    };

    match conn.r#type.as_str() {
        "request" => {
            if conn.message.is_empty() {
                println!("[Connection] {} sent you a connection request", name);
            } else {
                println!(
                    "[Connection] {} sent you a connection request: \"{}\"",
                    name, conn.message
                );
            }
        }
        "accepted" => {
            println!("[Connection] {} accepted your connection request", name);
        }
        "declined" => {
            println!("[Connection] {} declined your connection request", name);
        }
        _ => {
            println!("[Connection] {} - {}", name, conn.r#type);
        }
    }
}

fn print_payment(payment: &PaymentNotification) {
    let name = if payment.from_name.is_empty() {
        format!("@{}", payment.from_handle)
    } else {
        format!("{} (@{})", payment.from_name, payment.from_handle)
    };

    // Convert micro-units to USDC
    let usdc = payment.usdc_amount as f64 / 1_000_000.0;

    if payment.memo.is_empty() {
        println!("[Payment] {} sent you ${:.2} USDC", name, usdc);
    } else {
        println!(
            "[Payment] {} sent you ${:.2} USDC: \"{}\"",
            name, usdc, payment.memo
        );
    }
}

fn print_group(group: &GroupNotification) {
    let name = if group.from_name.is_empty() {
        format!("@{}", group.from_handle)
    } else {
        format!("{} (@{})", group.from_name, group.from_handle)
    };

    match group.r#type.as_str() {
        "message" => {
            println!(
                "[Group: {}] {}: \"{}\"",
                group.group_name, name, group.preview
            );
        }
        "added" => {
            println!("[Group] {} added you to \"{}\"", name, group.group_name);
        }
        "renamed" => {
            println!(
                "[Group] {} renamed the group to \"{}\"",
                name, group.group_name
            );
        }
        _ => {
            println!("[Group: {}] {} - {}", group.group_name, name, group.r#type);
        }
    }
}

fn print_email(email: &EmailNotification) {
    let from = if email.from_name.is_empty() {
        email.from_address.clone()
    } else {
        format!("{} <{}>", email.from_name, email.from_address)
    };

    let handle_info = if !email.from_handle.is_empty() {
        format!(" (@{})", email.from_handle)
    } else {
        String::new()
    };

    match email.r#type.as_str() {
        "received" => {
            let subject = if email.subject.is_empty() {
                "(no subject)".to_string()
            } else {
                email.subject.clone()
            };
            let attachment = if email.has_attachments { " [+]" } else { "" };
            println!(
                "[Email] From {}{}: \"{}\"{}\n  {}",
                from, handle_info, subject, attachment, email.snippet
            );
        }
        "sent" => {
            println!("[Email] Sent successfully");
        }
        "bounced" => {
            println!(
                "[Email] Bounce: {} - {}",
                email.from_address, email.error_message
            );
        }
        "complaint" => {
            println!("[Email] Complaint from {}", email.from_address);
        }
        _ => {
            println!("[Email] {} - {}", from, email.r#type);
        }
    }
}

fn print_payment_request(pr: &PaymentRequestNotification) {
    let usdc = pr.usdc_amount as f64 / 1_000_000.0;

    let product_info = if pr.product_name.is_empty() {
        String::new()
    } else {
        format!(" for \"{}\"", pr.product_name)
    };

    let sub_info = if pr.is_subscription {
        format!(" (subscription: {})", pr.billing_interval)
    } else {
        String::new()
    };

    println!(
        "[Payment Request] {} requests ${:.2} USDC{}{}",
        pr.merchant_name, usdc, product_info, sub_info
    );
    println!("  Expires: {}", pr.expires_at);
    println!("  Use 'kuren payments pending' to view and respond.");
}

fn print_subscription(sub: &SubscriptionNotification) {
    let usdc = sub.usdc_amount as f64 / 1_000_000.0;
    let refund = sub.refund_amount as f64 / 1_000_000.0;

    match sub.r#type.as_str() {
        "charged" => {
            println!(
                "[Subscription] {} charged ${:.2} USDC for \"{}\"",
                sub.merchant_name, usdc, sub.product_name
            );
            if !sub.next_billing_at.is_empty() {
                println!("  Next billing: {}", sub.next_billing_at);
            }
        }
        "failed" => {
            println!(
                "[Subscription] Payment failed for \"{}\" from {}",
                sub.product_name, sub.merchant_name
            );
            if !sub.failure_reason.is_empty() {
                println!("  Reason: {}", sub.failure_reason);
            }
        }
        "canceled" => {
            println!(
                "[Subscription] \"{}\" from {} has been canceled",
                sub.product_name, sub.merchant_name
            );
            if refund > 0.0 {
                println!("  Refunded: ${:.2} USDC", refund);
            }
        }
        "refunded" => {
            println!(
                "[Subscription] Refund of ${:.2} USDC from {} for \"{}\"",
                refund, sub.merchant_name, sub.product_name
            );
        }
        "trial_ending" => {
            println!(
                "[Subscription] Trial ending soon for \"{}\" from {}",
                sub.product_name, sub.merchant_name
            );
            println!("  First charge: ${:.2} USDC", usdc);
        }
        "trial_ended" => {
            println!(
                "[Subscription] Trial ended for \"{}\" from {}",
                sub.product_name, sub.merchant_name
            );
        }
        _ => {
            println!(
                "[Subscription] {} - {} ({})",
                sub.merchant_name, sub.product_name, sub.r#type
            );
        }
    }
}

fn print_wallet_request(wr: &WalletRequestNotification) {
    let name = if wr.from_name.is_empty() {
        format!("@{}", wr.from_handle)
    } else {
        format!("{} (@{})", wr.from_name, wr.from_handle)
    };

    let usdc = wr.usdc_amount as f64 / 1_000_000.0;

    match wr.r#type.as_str() {
        "requested" => {
            if wr.memo.is_empty() {
                println!(
                    "[Wallet Request] {} requests ${:.2} USDC from you",
                    name, usdc
                );
            } else {
                println!(
                    "[Wallet Request] {} requests ${:.2} USDC from you: \"{}\"",
                    name, usdc, wr.memo
                );
            }
            println!("  Expires: {}", wr.expires_at);
            println!("  Use 'kuren pay request list' to view and respond.");
        }
        "approved" => {
            println!(
                "[Wallet Request] {} approved your request for ${:.2} USDC",
                name, usdc
            );
        }
        "denied" => {
            println!(
                "[Wallet Request] {} denied your request for ${:.2} USDC",
                name, usdc
            );
        }
        "expired" => {
            println!(
                "[Wallet Request] Your request to {} for ${:.2} USDC has expired",
                name, usdc
            );
        }
        _ => {
            println!(
                "[Wallet Request] {} - {} ${:.2} USDC",
                name, wr.r#type, usdc
            );
        }
    }
}
