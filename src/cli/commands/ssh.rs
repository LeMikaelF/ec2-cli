use std::process::{Command, Stdio};

use crate::state::{get_instance, resolve_instance_name};
use crate::{Ec2CliError, Result};

pub fn execute(name: String, command: Option<String>) -> Result<()> {
    // Resolve instance name
    let name = resolve_instance_name(Some(&name))?;

    // Get instance from state
    let instance_state = get_instance(&name)?
        .ok_or_else(|| Ec2CliError::InstanceNotFound(name.clone()))?;

    let instance_id = &instance_state.instance_id;
    let username = &instance_state.username;

    if let Some(cmd) = command {
        // Run command via SSH
        run_ssh_command(instance_id, username, &cmd)
    } else {
        // Start interactive session
        start_interactive_session(instance_id, username)
    }
}

fn start_interactive_session(instance_id: &str, username: &str) -> Result<()> {
    // Use SSH via SSM proxy
    let status = Command::new("ssh")
        .arg(format!("{}@{}", username, instance_id))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| Ec2CliError::SshCommand(e.to_string()))?;

    if !status.success() {
        return Err(Ec2CliError::SshCommand(format!(
            "SSH session exited with code: {:?}",
            status.code()
        )));
    }

    Ok(())
}

fn run_ssh_command(instance_id: &str, username: &str, command: &str) -> Result<()> {
    let status = Command::new("ssh")
        .arg(format!("{}@{}", username, instance_id))
        .arg(command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| Ec2CliError::SshCommand(e.to_string()))?;

    if !status.success() {
        return Err(Ec2CliError::SshCommand(format!(
            "SSH command exited with code: {:?}",
            status.code()
        )));
    }

    Ok(())
}
