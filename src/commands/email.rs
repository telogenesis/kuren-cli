use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{
    email::{
        CancelScheduledEmailRequest, ClaimEmailAddressRequest, CreateFolderRequest,
        DeleteEmailsRequest, DeleteFolderRequest, GetContactsRequest, GetEmailAddressesRequest,
        GetEmailRequest, GetThreadRequest, ListEmailsRequest, ListFoldersRequest,
        ListScheduledEmailsRequest, MarkReadRequest, MoveEmailsRequest, ReleaseEmailAddressRequest,
        RenameFolderRequest, SaveDraftRequest, ScheduleEmailRequest, SendDraftRequest,
        SendEmailRequest, SetPrimaryEmailAddressRequest, StarEmailsRequest, UpdateDraftRequest,
        UpdateScheduledTimeRequest,
    },
    email_client, ensure_token, with_auth,
};

// ============================================================================
// Address Commands
// ============================================================================

pub async fn address_list() -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(Request::new(GetEmailAddressesRequest {}), &token);

    let response = client
        .get_email_addresses(request)
        .await
        .context("Failed to get email addresses")?
        .into_inner();

    if response.addresses.is_empty() {
        println!("No email addresses configured.");
        println!("Use 'kuren email address claim <local>' to claim an address.");
        return Ok(());
    }

    for addr in response.addresses {
        let primary_marker = if addr.is_primary { " *" } else { "" };
        println!("{}{}", addr.full_address, primary_marker);
    }

    if !response.primary_address.is_empty() {
        println!("\n* = primary address");
    }

    Ok(())
}

pub async fn address_claim(local_part: String, set_primary: bool) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(ClaimEmailAddressRequest {
            local_part: local_part.to_lowercase(),
            set_as_primary: set_primary,
        }),
        &token,
    );

    let response = client
        .claim_email_address(request)
        .await
        .context("Failed to claim email address")?
        .into_inner();

    println!("Claimed: {}", response.full_address);

    Ok(())
}

pub async fn address_primary(address_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(SetPrimaryEmailAddressRequest { address_id }),
        &token,
    );

    client
        .set_primary_email_address(request)
        .await
        .context("Failed to set primary email address")?;

    println!("Primary email address updated.");

    Ok(())
}

pub async fn address_release(address_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(ReleaseEmailAddressRequest { address_id }),
        &token,
    );

    client
        .release_email_address(request)
        .await
        .context("Failed to release email address")?;

    println!("Email address released.");

    Ok(())
}

// ============================================================================
// Folder Commands
// ============================================================================

pub async fn folder_list() -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(Request::new(ListFoldersRequest {}), &token);

    let response = client
        .list_folders(request)
        .await
        .context("Failed to list folders")?
        .into_inner();

    for folder in response.folders {
        let unread = if folder.unread_count > 0 {
            format!(" ({} unread)", folder.unread_count)
        } else {
            String::new()
        };
        println!("{}: {} messages{}", folder.name, folder.total_count, unread);
    }

    Ok(())
}

pub async fn folder_create(name: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(CreateFolderRequest { name: name.clone() }),
        &token,
    );

    let response = client
        .create_folder(request)
        .await
        .context("Failed to create folder")?
        .into_inner();

    println!("Created folder '{}' (ID: {})", name, response.folder_id);

    Ok(())
}

pub async fn folder_rename(folder_id: String, name: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(RenameFolderRequest {
            folder_id,
            name: name.clone(),
        }),
        &token,
    );

    client
        .rename_folder(request)
        .await
        .context("Failed to rename folder")?;

    println!("Folder renamed to '{}'", name);

    Ok(())
}

pub async fn folder_delete(folder_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(Request::new(DeleteFolderRequest { folder_id }), &token);

    client
        .delete_folder(request)
        .await
        .context("Failed to delete folder")?;

    println!("Folder deleted.");

    Ok(())
}

// ============================================================================
// Email Commands
// ============================================================================

pub async fn send(
    to: Vec<String>,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
    subject: Option<String>,
    body: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let body_text = body.unwrap_or_default();

    let request = with_auth(
        Request::new(SendEmailRequest {
            to,
            cc: cc.unwrap_or_default(),
            bcc: bcc.unwrap_or_default(),
            subject: subject.unwrap_or_default(),
            body_text: body_text.clone(),
            body_html: String::new(),
            in_reply_to: String::new(),
            attachment_ids: vec![],
        }),
        &token,
    );

    let response = client
        .send_email(request)
        .await
        .context("Failed to send email")?
        .into_inner();

    println!("Email sent! (ID: {})", response.message_id);

    Ok(())
}

pub async fn list(
    folder: Option<String>,
    unread: bool,
    starred: bool,
    limit: Option<i32>,
) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(ListEmailsRequest {
            folder_name: folder.unwrap_or_else(|| "inbox".to_string()),
            limit: limit.unwrap_or(20),
            cursor: String::new(),
            unread_only: unread,
            starred_only: starred,
        }),
        &token,
    );

    let response = client
        .list_emails(request)
        .await
        .context("Failed to list emails")?
        .into_inner();

    if response.emails.is_empty() {
        println!("No emails.");
        return Ok(());
    }

    for email in response.emails {
        let from = email
            .from
            .map(|f| {
                if f.name.is_empty() {
                    f.address
                } else {
                    format!("{} <{}>", f.name, f.address)
                }
            })
            .unwrap_or_else(|| "(unknown)".to_string());

        let read_marker = if email.is_read { " " } else { "*" };
        let star_marker = if email.is_starred { "!" } else { " " };
        let attach_marker = if email.has_attachments { "@" } else { " " };

        let subject = if email.subject.is_empty() {
            "(no subject)".to_string()
        } else {
            email.subject
        };

        let date = if !email.received_at.is_empty() {
            &email.received_at[..10]
        } else if !email.sent_at.is_empty() {
            &email.sent_at[..10]
        } else {
            ""
        };

        println!(
            "{}{}{} {} {} - {} [{}]",
            read_marker, star_marker, attach_marker, date, from, subject, email.id
        );
    }

    if response.unread_count > 0 {
        println!("\n{} unread messages", response.unread_count);
    }

    Ok(())
}

pub async fn read(email_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(Request::new(GetEmailRequest { email_id }), &token);

    let response = client
        .get_email(request)
        .await
        .context("Failed to get email")?
        .into_inner();

    let email = response
        .email
        .ok_or_else(|| anyhow::anyhow!("No email returned"))?;

    // Display headers
    let from = email
        .from
        .map(|f| {
            if f.name.is_empty() {
                f.address
            } else {
                format!("{} <{}>", f.name, f.address)
            }
        })
        .unwrap_or_else(|| "(unknown)".to_string());

    println!("From: {}", from);

    let to_addrs: Vec<String> = email
        .to
        .iter()
        .map(|t| {
            if t.name.is_empty() {
                t.address.clone()
            } else {
                format!("{} <{}>", t.name, t.address)
            }
        })
        .collect();
    println!("To: {}", to_addrs.join(", "));

    if !email.cc.is_empty() {
        let cc_addrs: Vec<String> = email
            .cc
            .iter()
            .map(|t| {
                if t.name.is_empty() {
                    t.address.clone()
                } else {
                    format!("{} <{}>", t.name, t.address)
                }
            })
            .collect();
        println!("Cc: {}", cc_addrs.join(", "));
    }

    println!(
        "Subject: {}",
        if email.subject.is_empty() {
            "(no subject)"
        } else {
            &email.subject
        }
    );

    let date = if !email.received_at.is_empty() {
        &email.received_at
    } else if !email.sent_at.is_empty() {
        &email.sent_at
    } else {
        ""
    };
    if !date.is_empty() {
        println!("Date: {}", date);
    }

    if !email.attachments.is_empty() {
        println!("\nAttachments:");
        for att in &email.attachments {
            println!(
                "  - {} ({} bytes) [{}]",
                att.filename, att.size_bytes, att.id
            );
        }
    }

    println!("\n---\n");
    println!("{}", email.body_text);

    Ok(())
}

pub async fn thread(thread_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(Request::new(GetThreadRequest { thread_id }), &token);

    let response = client
        .get_thread(request)
        .await
        .context("Failed to get thread")?
        .into_inner();

    println!(
        "Thread: {}\n",
        if response.subject.is_empty() {
            "(no subject)"
        } else {
            &response.subject
        }
    );

    for (i, email) in response.emails.iter().enumerate() {
        if i > 0 {
            println!("\n---\n");
        }

        let from = email
            .from
            .as_ref()
            .map(|f| {
                if f.name.is_empty() {
                    f.address.clone()
                } else {
                    format!("{} <{}>", f.name, f.address)
                }
            })
            .unwrap_or_else(|| "(unknown)".to_string());

        let date = if !email.received_at.is_empty() {
            &email.received_at
        } else if !email.sent_at.is_empty() {
            &email.sent_at
        } else {
            ""
        };

        println!("[{}] From: {} at {}", i + 1, from, date);
        println!("{}", email.body_text);
    }

    Ok(())
}

pub async fn archive(email_ids: Vec<String>) -> Result<()> {
    move_to_folder(email_ids, "archive").await
}

pub async fn trash(email_ids: Vec<String>) -> Result<()> {
    move_to_folder(email_ids, "trash").await
}

pub async fn delete(email_ids: Vec<String>, force: bool) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(DeleteEmailsRequest {
            email_ids,
            permanent: force,
        }),
        &token,
    );

    let response = client
        .delete_emails(request)
        .await
        .context("Failed to delete emails")?
        .into_inner();

    println!("{} email(s) deleted.", response.deleted_count);

    Ok(())
}

pub async fn move_to(email_ids: Vec<String>, to_folder: String) -> Result<()> {
    move_to_folder(email_ids, &to_folder).await
}

async fn move_to_folder(email_ids: Vec<String>, to_folder: &str) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(MoveEmailsRequest {
            email_ids,
            to_folder: to_folder.to_string(),
        }),
        &token,
    );

    let response = client
        .move_emails(request)
        .await
        .context("Failed to move emails")?
        .into_inner();

    println!("{} email(s) moved to {}.", response.moved_count, to_folder);

    Ok(())
}

pub async fn star(email_ids: Vec<String>, unstar: bool) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(StarEmailsRequest {
            email_ids,
            is_starred: !unstar,
        }),
        &token,
    );

    let response = client
        .star_emails(request)
        .await
        .context("Failed to star emails")?
        .into_inner();

    let action = if unstar { "unstarred" } else { "starred" };
    println!("{} email(s) {}.", response.updated_count, action);

    Ok(())
}

pub async fn mark(email_ids: Vec<String>, read: bool, unread: bool) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let is_read = if read {
        true
    } else if unread {
        false
    } else {
        anyhow::bail!("Specify either --read or --unread");
    };

    let request = with_auth(Request::new(MarkReadRequest { email_ids, is_read }), &token);

    let response = client
        .mark_read(request)
        .await
        .context("Failed to mark emails")?
        .into_inner();

    let action = if is_read { "read" } else { "unread" };
    println!("{} email(s) marked as {}.", response.updated_count, action);

    Ok(())
}

pub async fn contacts(query: Option<String>, limit: Option<i32>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(GetContactsRequest {
            query: query.unwrap_or_default(),
            limit: limit.unwrap_or(20),
        }),
        &token,
    );

    let response = client
        .get_contacts(request)
        .await
        .context("Failed to get contacts")?
        .into_inner();

    if response.contacts.is_empty() {
        println!("No contacts found.");
        return Ok(());
    }

    for contact in response.contacts {
        let name = if contact.display_name.is_empty() {
            String::new()
        } else {
            format!(" ({})", contact.display_name)
        };
        let handle = if contact.handle.is_empty() {
            String::new()
        } else {
            format!(" @{}", contact.handle)
        };
        println!(
            "{}{}{} - {} contacts",
            contact.email_address, name, handle, contact.contact_count
        );
    }

    Ok(())
}

// ============================================================================
// Draft Commands
// ============================================================================

pub async fn drafts_save(
    to: Option<Vec<String>>,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
    subject: Option<String>,
    body: Option<String>,
    reply_to: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(SaveDraftRequest {
            to: to.unwrap_or_default(),
            cc: cc.unwrap_or_default(),
            bcc: bcc.unwrap_or_default(),
            subject: subject.unwrap_or_default(),
            body_text: body.unwrap_or_default(),
            body_html: String::new(),
            in_reply_to: reply_to.unwrap_or_default(),
            attachment_ids: vec![],
        }),
        &token,
    );

    let response = client
        .save_draft(request)
        .await
        .context("Failed to save draft")?
        .into_inner();

    println!("Draft saved! (ID: {})", response.draft_id);

    Ok(())
}

pub async fn drafts_list(limit: Option<i32>) -> Result<()> {
    // Reuse list() with folder="drafts"
    list(Some("drafts".to_string()), false, false, limit).await
}

pub async fn drafts_read(draft_id: String) -> Result<()> {
    // Reuse read() - works for any email including drafts
    read(draft_id).await
}

pub async fn drafts_update(
    draft_id: String,
    to: Option<Vec<String>>,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
    subject: Option<String>,
    body: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(UpdateDraftRequest {
            draft_id,
            to: to.unwrap_or_default(),
            cc: cc.unwrap_or_default(),
            bcc: bcc.unwrap_or_default(),
            subject: subject.unwrap_or_default(),
            body_text: body.unwrap_or_default(),
            body_html: String::new(),
            in_reply_to: String::new(),
            attachment_ids: vec![],
        }),
        &token,
    );

    client
        .update_draft(request)
        .await
        .context("Failed to update draft")?;

    println!("Draft updated.");

    Ok(())
}

pub async fn drafts_send(draft_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(Request::new(SendDraftRequest { draft_id }), &token);

    let response = client
        .send_draft(request)
        .await
        .context("Failed to send draft")?
        .into_inner();

    println!("Draft sent! (Message ID: {})", response.message_id);

    Ok(())
}

pub async fn drafts_delete(draft_id: String, force: bool) -> Result<()> {
    // Reuse delete() - works for any email including drafts
    delete(vec![draft_id], force).await
}

// ============================================================================
// Scheduled Send Commands
// ============================================================================

pub async fn schedule(draft_id: String, at: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(ScheduleEmailRequest {
            draft_id,
            scheduled_for: at,
        }),
        &token,
    );

    let response = client
        .schedule_email(request)
        .await
        .context("Failed to schedule email")?
        .into_inner();

    println!("Email scheduled for: {}", response.scheduled_for);
    println!("Email ID: {}", response.email_id);

    Ok(())
}

pub async fn scheduled_list(limit: Option<i32>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(ListScheduledEmailsRequest {
            limit: limit.unwrap_or(20),
            cursor: String::new(),
        }),
        &token,
    );

    let response = client
        .list_scheduled_emails(request)
        .await
        .context("Failed to list scheduled emails")?
        .into_inner();

    if response.emails.is_empty() {
        println!("No scheduled emails.");
        return Ok(());
    }

    for email in response.emails {
        let to_addrs: Vec<String> = email.to.iter().map(|a| a.address.clone()).collect();
        let subject = if email.subject.is_empty() {
            "(no subject)".to_string()
        } else {
            email.subject
        };

        println!(
            "{} | {} -> {} | {}",
            email.scheduled_for,
            email
                .from
                .map(|f| f.address)
                .unwrap_or_else(|| "(unknown)".to_string()),
            to_addrs.join(", "),
            subject
        );
        println!("  ID: {}", email.id);
    }

    Ok(())
}

pub async fn scheduled_cancel(email_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(CancelScheduledEmailRequest { email_id }),
        &token,
    );

    let response = client
        .cancel_scheduled_email(request)
        .await
        .context("Failed to cancel scheduled email")?
        .into_inner();

    println!("Scheduled email cancelled.");
    println!("Returned to drafts (ID: {})", response.draft_id);

    Ok(())
}

pub async fn scheduled_update(email_id: String, at: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = email_client(&config).await?;

    let request = with_auth(
        Request::new(UpdateScheduledTimeRequest {
            email_id,
            scheduled_for: at,
        }),
        &token,
    );

    let response = client
        .update_scheduled_time(request)
        .await
        .context("Failed to update scheduled time")?
        .into_inner();

    println!("Scheduled time updated to: {}", response.scheduled_for);

    Ok(())
}
