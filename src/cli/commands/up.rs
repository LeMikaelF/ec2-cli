use indicatif::{ProgressBar, ProgressStyle};

use crate::aws::client::AwsClients;
use crate::aws::ec2::instance::{launch_instance, wait_for_running, wait_for_ssm_ready};
use crate::aws::infrastructure::Infrastructure;
use crate::profile::ProfileLoader;
use crate::user_data::generate_user_data;
use crate::{Ec2CliError, Result};

pub async fn execute(
    profile_name: Option<String>,
    instance_name: Option<String>,
    link: bool,
) -> Result<()> {
    // Load profile
    let loader = ProfileLoader::new();
    let profile_name = profile_name.unwrap_or_else(|| "default".to_string());
    let profile = loader.load(&profile_name)?;
    profile.validate()?;

    // Generate instance name if not provided
    let name = instance_name.unwrap_or_else(|| {
        petname::petname(2, "-").unwrap_or_else(|| "ec2-instance".to_string())
    });

    println!("Launching EC2 instance '{}'...", name);
    println!("  Profile: {}", profile.name);
    println!("  Instance type: {}", profile.instance.instance_type);

    // Initialize AWS clients
    let spinner = create_spinner("Connecting to AWS...");
    let clients = AwsClients::new().await?;
    spinner.finish_with_message("Connected to AWS");

    // Get or create infrastructure
    let spinner = create_spinner("Checking infrastructure...");
    let infra = Infrastructure::get_or_create(&clients).await?;
    spinner.finish_with_message("Infrastructure ready");

    // Get project name from current directory (for git repo setup)
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

    // Generate user data
    let user_data = generate_user_data(&profile, project_name.as_deref());

    // Launch instance
    let spinner = create_spinner("Launching instance...");
    let instance_id = launch_instance(&clients, &infra, &profile, &name, &user_data).await?;
    spinner.finish_with_message(format!("Instance launched: {}", instance_id));

    // Wait for instance to be running
    let spinner = create_spinner("Waiting for instance to start...");
    wait_for_running(&clients, &instance_id, 300).await?;
    spinner.finish_with_message("Instance running");

    // Wait for SSM agent to be ready
    let spinner = create_spinner("Waiting for SSM agent...");
    wait_for_ssm_ready(&clients, &instance_id, 600).await?;
    spinner.finish_with_message("SSM agent ready");

    // Save state
    crate::state::save_instance(&name, &instance_id, &profile.name, &clients.region)?;

    // Create link file if requested
    if link {
        create_link_file(&name)?;
        println!("  Linked to current directory");
    }

    println!();
    println!("Instance '{}' is ready!", name);
    println!("  Instance ID: {}", instance_id);
    println!("  Connect with: ec2-cli ssh {}", name);

    if let Some(ref proj) = project_name {
        println!("  Push code with: ec2-cli push {}", name);
        println!("  Git remote: ec2-user@{}:/home/ec2-user/repos/{}.git", instance_id, proj);
    }

    Ok(())
}

fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

fn create_link_file(name: &str) -> Result<()> {
    let link_dir = std::env::current_dir()?.join(".ec2-cli");
    std::fs::create_dir_all(&link_dir)?;

    let link_file = link_dir.join("instance");
    std::fs::write(&link_file, name)?;

    Ok(())
}
