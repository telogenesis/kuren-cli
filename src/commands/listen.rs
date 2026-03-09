use anyhow::{Context, Result};
use tokio_stream::StreamExt;
use tonic::Request;

use crate::config::Config;
use crate::grpc::{
    ensure_token,
    notifications::{
        kuren_event::Event, ConnectionNotification, DmNotification, EmailNotification,
        GroupNotification, SubscribeRequest,
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
        Some(Event::Group(group)) => print_group(group),
        Some(Event::Email(email)) => print_email(email),
        _ => {}
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

