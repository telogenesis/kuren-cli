use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{
    ensure_token, notes::CreateNoteRequest, notes::DeleteNoteRequest, notes::GetNoteRequest,
    notes::ListNotesRequest, notes::SearchNotesRequest, notes::UpdateNoteRequest, notes_client,
    with_auth,
};

/// Create a new note
pub async fn create(title: String, content: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = notes_client(&config).await?;
    let request = with_auth(
        Request::new(CreateNoteRequest {
            title: title.clone(),
            content: content.unwrap_or_default(),
        }),
        &token,
    );

    let response = client
        .create_note(request)
        .await
        .context("Failed to create note")?
        .into_inner();

    println!("Note created: {} ({})", title, response.id);

    Ok(())
}

/// Get a specific note
pub async fn get(id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = notes_client(&config).await?;
    let request = with_auth(Request::new(GetNoteRequest { id: id.clone() }), &token);

    let response = client
        .get_note(request)
        .await
        .context("Failed to get note")?
        .into_inner();

    println!("# {}", response.title);
    println!();
    if !response.content.is_empty() {
        println!("{}", response.content);
        println!();
    }
    println!("ID: {}", response.id);
    println!("Created: {}", response.created_at);
    println!("Updated: {}", response.updated_at);

    Ok(())
}

/// List notes
pub async fn list(limit: Option<i32>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = notes_client(&config).await?;
    let request = with_auth(
        Request::new(ListNotesRequest {
            limit: limit.unwrap_or(0),
            cursor: String::new(),
        }),
        &token,
    );

    let response = client
        .list_notes(request)
        .await
        .context("Failed to list notes")?
        .into_inner();

    if response.notes.is_empty() {
        println!("No notes yet.");
        return Ok(());
    }

    for note in response.notes {
        println!("{} - {}", note.title, note.id);
        if !note.snippet.is_empty() {
            let snippet = if note.snippet.len() > 80 {
                format!("{}...", &note.snippet[..77])
            } else {
                note.snippet
            };
            println!("  {}", snippet);
        }
        println!("  {}", note.updated_at);
        println!();
    }

    Ok(())
}

/// Edit a note
pub async fn edit(id: String, title: Option<String>, content: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = notes_client(&config).await?;
    let request = with_auth(
        Request::new(UpdateNoteRequest {
            id: id.clone(),
            title,
            content,
        }),
        &token,
    );

    let response = client
        .update_note(request)
        .await
        .context("Failed to update note")?
        .into_inner();

    println!("Updated: {} ({})", response.title, response.id);

    Ok(())
}

/// Delete a note
pub async fn delete(id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = notes_client(&config).await?;
    let request = with_auth(Request::new(DeleteNoteRequest { id: id.clone() }), &token);

    client
        .delete_note(request)
        .await
        .context("Failed to delete note")?;

    println!("Deleted note {}", id);

    Ok(())
}

/// Search notes
pub async fn search(query: String, limit: Option<i32>) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = notes_client(&config).await?;
    let request = with_auth(
        Request::new(SearchNotesRequest {
            query: query.clone(),
            limit: limit.unwrap_or(0),
        }),
        &token,
    );

    let response = client
        .search_notes(request)
        .await
        .context("Failed to search notes")?
        .into_inner();

    if response.notes.is_empty() {
        println!("No notes matching \"{}\".", query);
        return Ok(());
    }

    println!("Results for \"{}\":\n", query);
    for note in response.notes {
        println!("{} - {}", note.title, note.id);
        if !note.snippet.is_empty() {
            let snippet = if note.snippet.len() > 80 {
                format!("{}...", &note.snippet[..77])
            } else {
                note.snippet
            };
            println!("  {}", snippet);
        }
        println!();
    }

    Ok(())
}
