use anyhow::Result;

pub async fn run() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: v{}", current_version);
    println!("Checking for updates...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner("telogenesis")
        .repo_name("kuren")
        .bin_name("kuren")
        .current_version(current_version)
        .target(&self_update::get_target())
        .build()?
        .update()?;

    if status.updated() {
        println!("Updated to v{}!", status.version());
    } else {
        println!("Already up to date.");
    }

    Ok(())
}
