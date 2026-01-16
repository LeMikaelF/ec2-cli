use crate::{Ec2CliError, Result};

/// Check if SSH config has the required SSM proxy configuration
pub fn check_ssh_config() -> Result<SshConfigStatus> {
    let home = std::env::var("HOME").map_err(|_| {
        Ec2CliError::SshConfig("Cannot determine home directory".to_string())
    })?;

    let ssh_config_path = std::path::Path::new(&home).join(".ssh").join("config");

    if !ssh_config_path.exists() {
        return Ok(SshConfigStatus::Missing);
    }

    let content = std::fs::read_to_string(&ssh_config_path)?;

    // Check for SSM proxy configuration
    // Looking for patterns like "Host i-*" or "Host mi-*" with ProxyCommand
    let has_instance_host = content.contains("Host i-*") || content.contains("Host mi-*");
    let has_proxy_command = content.contains("ProxyCommand") && content.contains("ssm");

    if has_instance_host && has_proxy_command {
        Ok(SshConfigStatus::Configured)
    } else {
        Ok(SshConfigStatus::NeedsConfiguration)
    }
}

/// Generate the SSH config block for SSM
pub fn generate_ssh_config_block() -> String {
    r#"# EC2 SSH via SSM Session Manager
Host i-* mi-*
    User ubuntu
    ProxyCommand sh -c "aws ssm start-session --target %h --document-name AWS-StartSSHSession --parameters 'portNumber=%p'"
"#
    .to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SshConfigStatus {
    Configured,
    NeedsConfiguration,
    Missing,
}

impl std::fmt::Display for SshConfigStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SshConfigStatus::Configured => write!(f, "configured"),
            SshConfigStatus::NeedsConfiguration => write!(f, "needs configuration"),
            SshConfigStatus::Missing => write!(f, "missing"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ssh_config() {
        let config = generate_ssh_config_block();
        assert!(config.contains("Host i-*"));
        assert!(config.contains("ProxyCommand"));
        assert!(config.contains("ssm"));
    }
}
