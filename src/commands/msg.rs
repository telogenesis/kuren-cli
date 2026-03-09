use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{
    ensure_token, messaging::create_thread_request::ThreadType as ThreadTypeOneof,
    messaging::send_thread_message_request::Target, messaging::AddThreadMemberRequest,
    messaging::CreateDmThread, messaging::CreateGroupThread, messaging::CreateThreadRequest,
    messaging::GetMessagesRequest, messaging::GetThreadRequest, messaging::LeaveThreadRequest,
    messaging::ListThreadsRequest, messaging::SendThreadMessageRequest, messaging::ThreadType,
    messaging::UpdateThreadRequest, messaging_client, with_auth,
};

/// Send a message to a thread or handle
/// If target starts with @, it's a handle (DM), otherwise it's a thread ID
pub async fn send(target: String, text: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = messaging_client(&config).await?;

    let target_oneof = if target.starts_with('@') {
        Target::Handle(target.trim_start_matches('@').to_string())
    } else {
        Target::ThreadId(target.clone())
    };

    let request = with_auth(
        Request::new(SendThreadMessageRequest {
            target: Some(target_oneof),
            text,
        }),
        &token,
    );

    let response = client
        .send_thread_message(request)
        .await
        .context("Failed to send message")?
        .into_inner();

    if target.starts_with('@') {
        println!("Message sent to {}", target);
    } else {
        println!("Message sent to thread {}", response.thread_id);
    }

    Ok(())
}

/// List all threads (DMs and groups)
pub async fn list(filter: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let filter_type = match filter.as_deref() {
        Some("dm") | Some("dms") => ThreadType::Dm as i32,
        Some("group") | Some("groups") => ThreadType::Group as i32,
        _ => ThreadType::Unspecified as i32,
    };

    let mut client = messaging_client(&config).await?;
    let request = with_auth(Request::new(ListThreadsRequest { filter_type }), &token);

    let response = client
        .list_threads(request)
        .await
        .context("Failed to list threads")?
        .into_inner();

    if response.threads.is_empty() {
        println!("No threads yet.");
        return Ok(());
    }

    println!("Your threads:\n");
    for thread in response.threads {
        let type_label = match ThreadType::try_from(thread.r#type) {
            Ok(ThreadType::Dm) => "DM",
            Ok(ThreadType::Group) => "Group",
            _ => "Unknown",
        };

        // For DMs, show the other person's handle; for groups, show the name
        let display_name = if thread.r#type == ThreadType::Dm as i32 {
            if !thread.other_handle.is_empty() {
                format!("@{}", thread.other_handle)
            } else {
                thread.id.clone()
            }
        } else {
            if !thread.name.is_empty() {
                thread.name.clone()
            } else {
                thread.id.clone()
            }
        };

        let preview = if thread.last_message.len() > 50 {
            format!("{}...", &thread.last_message[..47])
        } else if thread.last_message.is_empty() {
            "(no messages)".to_string()
        } else {
            thread.last_message.clone()
        };

        let admin_badge = if thread.is_admin && thread.r#type == ThreadType::Group as i32 {
            " [admin]"
        } else {
            ""
        };

        println!(
            "[{}] {} - {}{}",
            type_label, display_name, thread.id, admin_badge
        );
        println!("  \"{}\"", preview);
        if !thread.last_message_at.is_empty() {
            println!("  {}", thread.last_message_at);
        }
        println!();
    }

    Ok(())
}

/// Read messages from a thread (by ID or handle for DMs)
pub async fn read(target: String, limit: Option<i32>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let my_handle = config.handle.as_deref().unwrap_or("");

    let mut client = messaging_client(&config).await?;

    // If it's a handle, first get/create the DM thread to get the thread ID
    let thread_id = if target.starts_with('@') {
        let handle = target.trim_start_matches('@').to_string();
        let create_request = with_auth(
            Request::new(CreateThreadRequest {
                thread_type: Some(ThreadTypeOneof::Dm(CreateDmThread {
                    handle: handle.clone(),
                })),
            }),
            &token,
        );
        let create_response = client
            .create_thread(create_request)
            .await
            .context("Failed to get DM thread")?
            .into_inner();
        create_response.thread_id
    } else {
        target.clone()
    };

    // Re-create client for second request
    let mut client = messaging_client(&config).await?;

    let request = with_auth(
        Request::new(GetMessagesRequest {
            thread_id: thread_id.clone(),
            limit: limit.unwrap_or(50),
            cursor: String::new(),
        }),
        &token,
    );

    let response = client
        .get_messages(request)
        .await
        .context("Failed to get messages")?
        .into_inner();

    if response.messages.is_empty() {
        if target.starts_with('@') {
            println!("No messages with {} yet.", target);
        } else {
            println!("No messages in thread {} yet.", target);
        }
        return Ok(());
    }

    if target.starts_with('@') {
        println!("Conversation with {}:\n", target);
    } else {
        println!("Messages in thread {}:\n", thread_id);
    }

    // Messages are returned newest first, so reverse for chronological display
    let mut messages = response.messages;
    messages.reverse();

    for msg in messages {
        let sender = if msg.from_handle == my_handle {
            "You".to_string()
        } else if !msg.from_name.is_empty() {
            format!("{} (@{})", msg.from_name, msg.from_handle)
        } else {
            format!("@{}", msg.from_handle)
        };
        println!("[{}] {}: {}", msg.created_at, sender, msg.text);
    }

    Ok(())
}

/// Get info about a thread
pub async fn info(thread_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = messaging_client(&config).await?;
    let request = with_auth(
        Request::new(GetThreadRequest {
            thread_id: thread_id.clone(),
        }),
        &token,
    );

    let response = client
        .get_thread(request)
        .await
        .context("Failed to get thread info")?
        .into_inner();

    let thread = response
        .thread
        .ok_or_else(|| anyhow::anyhow!("No thread returned"))?;

    let type_label = match ThreadType::try_from(thread.r#type) {
        Ok(ThreadType::Dm) => "DM",
        Ok(ThreadType::Group) => "Group",
        _ => "Unknown",
    };

    println!("Thread: {}", thread_id);
    println!("Type: {}", type_label);
    if !thread.name.is_empty() {
        println!("Name: {}", thread.name);
    }
    if thread.r#type == ThreadType::Group as i32 {
        println!(
            "Mode: {}",
            if thread.is_permissioned {
                "Permissioned"
            } else {
                "Open"
            }
        );
    }
    println!("Created: {}", thread.created_at);
    println!("\nMembers:");
    for member in thread.members {
        let admin_badge = if member.is_admin { " [admin]" } else { "" };
        let name_display = if member.name.is_empty() {
            String::new()
        } else {
            format!(" ({})", member.name)
        };
        println!("  @{}{}{}", member.handle, name_display, admin_badge);
    }

    Ok(())
}

/// Create a new group thread
pub async fn create_group(name: String, permissioned: bool) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = messaging_client(&config).await?;
    let request = with_auth(
        Request::new(CreateThreadRequest {
            thread_type: Some(ThreadTypeOneof::Group(CreateGroupThread {
                name: name.clone(),
                is_permissioned: permissioned,
            })),
        }),
        &token,
    );

    let response = client
        .create_thread(request)
        .await
        .context("Failed to create group")?
        .into_inner();

    println!("Group created!");
    println!("  Name: {}", name);
    println!("  ID: {}", response.thread_id);
    if permissioned {
        println!("  Mode: Permissioned (only admins can add members)");
    } else {
        println!("  Mode: Open (any member can add others)");
    }

    Ok(())
}

/// Add a member to a thread
pub async fn add_member(thread_id: String, handle: String, as_admin: bool) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = messaging_client(&config).await?;
    let handle_display = handle.trim_start_matches('@').to_string();
    let request = with_auth(
        Request::new(AddThreadMemberRequest {
            thread_id: thread_id.clone(),
            handle: handle_display.clone(),
            as_admin,
        }),
        &token,
    );

    client
        .add_thread_member(request)
        .await
        .context("Failed to add member")?;

    if as_admin {
        println!("Added @{} to thread {} as admin", handle_display, thread_id);
    } else {
        println!("Added @{} to thread {}", handle_display, thread_id);
    }

    Ok(())
}

/// Leave a thread
pub async fn leave_thread(thread_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = messaging_client(&config).await?;
    let request = with_auth(
        Request::new(LeaveThreadRequest {
            thread_id: thread_id.clone(),
        }),
        &token,
    );

    client
        .leave_thread(request)
        .await
        .context("Failed to leave thread")?;

    println!("Left thread {}", thread_id);

    Ok(())
}

/// Rename a thread
pub async fn rename_thread(thread_id: String, name: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = messaging_client(&config).await?;
    let request = with_auth(
        Request::new(UpdateThreadRequest {
            thread_id: thread_id.clone(),
            name: Some(name.clone()),
            is_permissioned: None,
        }),
        &token,
    );

    client
        .update_thread(request)
        .await
        .context("Failed to rename thread")?;

    println!("Renamed thread {} to \"{}\"", thread_id, name);

    Ok(())
}
