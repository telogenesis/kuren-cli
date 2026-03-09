use anyhow::Result;

use crate::config::Config;

pub async fn run() -> Result<()> {
    let mut config = Config::load()?;

    if !config.is_logged_in() {
        println!("Not logged in.");
        return Ok(());
    }

    config.clear_tokens();
    config.save()?;

    println!("Logged out successfully.");
    Ok(())
}
