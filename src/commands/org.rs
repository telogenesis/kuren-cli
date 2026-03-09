use anyhow::{Context, Result};
use tonic::Request;

use crate::config::Config;
use crate::grpc::{
    ensure_token, organization::ListMembersRequest, organization::ListMyOrganizationsRequest,
    organization_client, with_auth,
};

/// List organizations the current agent is a member of
pub async fn list() -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = organization_client(&config).await?;

    let request = with_auth(Request::new(ListMyOrganizationsRequest {}), &token);

    let response = client
        .list_my_organizations(request)
        .await
        .context("Failed to list organizations")?
        .into_inner();

    if response.organizations.is_empty() {
        println!("You are not a member of any organizations.");
        return Ok(());
    }

    println!("Your organizations:\n");
    for org in response.organizations {
        let money_status = if org.money_enabled {
            " [money enabled]"
        } else {
            ""
        };
        println!("  {} - {}{}", org.id, org.name, money_status);
        println!("    Created: {}", org.created_at);
        println!();
    }

    Ok(())
}

/// List members of an organization
pub async fn members(org_id: String) -> Result<()> {
    let mut config = Config::load()?;
    let token = ensure_token(&mut config).await?;

    let mut client = organization_client(&config).await?;

    let request = with_auth(
        Request::new(ListMembersRequest {
            organization_id: org_id.clone(),
        }),
        &token,
    );

    let response = client
        .list_members(request)
        .await
        .context("Failed to list members")?
        .into_inner();

    if response.members.is_empty() {
        println!("Organization {} has no members.", org_id);
        return Ok(());
    }

    println!("Members of organization {}:\n", org_id);
    for member in response.members {
        println!("  @{}", member.handle);
        println!("    Agent ID: {}", member.agent_id);
        println!("    Joined: {}", member.joined_at);
        println!();
    }

    Ok(())
}
