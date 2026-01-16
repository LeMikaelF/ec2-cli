use crate::profile::Profile;

/// Generate cloud-init user data script from profile
pub fn generate_user_data(profile: &Profile, project_name: Option<&str>) -> String {
    let mut script = String::from("#!/bin/bash\nset -ex\n\n");

    // Log to file for debugging
    script.push_str("exec > >(tee /var/log/ec2-cli-init.log) 2>&1\n\n");

    // Wait for cloud-init to complete basic setup
    script.push_str("echo 'Waiting for cloud-init...'\n");
    script.push_str("cloud-init status --wait || true\n\n");

    // Install system packages
    if !profile.packages.system.is_empty() {
        script.push_str("echo 'Installing system packages...'\n");
        let packages = profile.packages.system.join(" ");

        // Detect package manager
        script.push_str("if command -v dnf &> /dev/null; then\n");
        script.push_str(&format!("    dnf install -y {}\n", packages));
        script.push_str("elif command -v yum &> /dev/null; then\n");
        script.push_str(&format!("    yum install -y {}\n", packages));
        script.push_str("elif command -v apt-get &> /dev/null; then\n");
        script.push_str("    apt-get update\n");
        script.push_str(&format!("    apt-get install -y {}\n", packages));
        script.push_str("fi\n\n");
    }

    // Install Rust if enabled
    if profile.packages.rust.enabled {
        script.push_str("echo 'Installing Rust...'\n");
        script.push_str("su - ec2-user -c '\n");
        script.push_str("curl --proto \"=https\" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y");

        // Add channel if not stable
        if profile.packages.rust.channel != "stable" {
            script.push_str(&format!(
                " --default-toolchain {}",
                profile.packages.rust.channel
            ));
        }
        script.push_str("\n");

        // Source cargo env and install components
        script.push_str("source ~/.cargo/env\n");

        if !profile.packages.rust.components.is_empty() {
            let components = profile.packages.rust.components.join(" ");
            script.push_str(&format!("rustup component add {}\n", components));
        }
        script.push_str("'\n\n");

        // Install cargo packages
        if !profile.packages.cargo.is_empty() {
            script.push_str("echo 'Installing cargo packages...'\n");
            script.push_str("su - ec2-user -c '\n");
            script.push_str("source ~/.cargo/env\n");
            for pkg in &profile.packages.cargo {
                script.push_str(&format!("cargo install {}\n", pkg));
            }
            script.push_str("'\n\n");
        }
    }

    // Set environment variables
    if !profile.environment.is_empty() {
        script.push_str("echo 'Setting environment variables...'\n");
        script.push_str("cat >> /home/ec2-user/.bashrc << 'ENVEOF'\n");
        for (key, value) in &profile.environment {
            script.push_str(&format!("export {}=\"{}\"\n", key, value));
        }
        script.push_str("ENVEOF\n\n");
    }

    // Create directories for git repos
    script.push_str("echo 'Setting up git directories...'\n");
    script.push_str("mkdir -p /home/ec2-user/repos\n");
    script.push_str("mkdir -p /home/ec2-user/work\n");
    script.push_str("chown -R ec2-user:ec2-user /home/ec2-user/repos /home/ec2-user/work\n\n");

    // Set up git repo for the project if name provided
    if let Some(name) = project_name {
        script.push_str(&format!("echo 'Setting up git repo for {}...'\n", name));
        script.push_str(&format!(
            "su - ec2-user -c 'git init --bare /home/ec2-user/repos/{}.git'\n",
            name
        ));

        // Create post-receive hook
        script.push_str(&format!(
            r#"cat > /home/ec2-user/repos/{}.git/hooks/post-receive << 'HOOKEOF'
#!/bin/bash
GIT_WORK_TREE=/home/ec2-user/work/{} git checkout -f
HOOKEOF
"#,
            name, name
        ));
        script.push_str(&format!(
            "chmod +x /home/ec2-user/repos/{}.git/hooks/post-receive\n",
            name
        ));
        script.push_str(&format!(
            "chown -R ec2-user:ec2-user /home/ec2-user/repos/{}.git\n",
            name
        ));
        script.push_str(&format!(
            "mkdir -p /home/ec2-user/work/{}\n",
            name
        ));
        script.push_str(&format!(
            "chown -R ec2-user:ec2-user /home/ec2-user/work/{}\n\n",
            name
        ));
    }

    // Signal completion
    script.push_str("echo 'ec2-cli initialization complete!'\n");
    script.push_str("touch /home/ec2-user/.ec2-cli-ready\n");

    script
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;

    #[test]
    fn test_generate_basic_user_data() {
        let profile = Profile::default_profile();
        let script = generate_user_data(&profile, Some("test-project"));

        assert!(script.contains("#!/bin/bash"));
        assert!(script.contains("rustup"));
        assert!(script.contains("git init --bare"));
        assert!(script.contains("test-project"));
        assert!(script.contains(".ec2-cli-ready"));
    }

    #[test]
    fn test_generate_without_project() {
        let profile = Profile::default_profile();
        let script = generate_user_data(&profile, None);

        assert!(script.contains("#!/bin/bash"));
        assert!(!script.contains("git init --bare"));
    }
}
