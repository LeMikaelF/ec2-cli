# EC2-CLI: Ephemeral EC2 Development Environment Manager

## Overview

A Rust CLI tool to launch private EC2 instances for remote development. Instances are not exposed to the internet - all access is via AWS Systems Manager Session Manager. Includes git integration to push/pull code between local and EC2.

## CLI Commands

All subcommands support `--help` for usage info.

```
ec2-cli up [--profile <name>] [--name <name>] [--link]   # Launch new EC2 (default profile if omitted)
ec2-cli destroy <name> [--force]                             # Terminate instance & cleanup
ec2-cli ssh <name> [-c <command>]                            # SSH via SSM Session Manager
ec2-cli scp <name> <src> <dest> [--recursive]                # Copy files to/from EC2 via SSM
ec2-cli push <name> [--branch <branch>]                      # Push code to EC2 bare repo
ec2-cli pull <name> [--branch <branch>]                      # Pull from EC2 bare repo
ec2-cli status [<name>]                                      # Show instance status
ec2-cli list [--all]                                         # List managed instances
ec2-cli profile list|show|validate                           # Manage EC2 profiles
ec2-cli config init|show                                     # Configure CLI & prereqs
ec2-cli logs <name> [--follow]                               # View cloud-init logs
```

### SCP Command
```
ec2-cli scp my-dev ./local-file.txt :/home/ec2-user/        # Local to remote
ec2-cli scp my-dev :/home/ec2-user/file.txt ./local/        # Remote to local
ec2-cli scp my-dev ./dir :/home/ec2-user/ --recursive       # Copy directory
```
Paths prefixed with `:` are remote paths on the EC2 instance.

### Instance Naming
- **`--name <name>`**: Specify a custom name (e.g., `ec2-cli up --name my-dev`)
- **Random names**: If no name given, generates memorable names like `happy-zebra`, `swift-falcon`
- **`--link` flag**: Creates `.ec2-cli/instance` file in current directory, so subsequent commands auto-detect which instance to use
- **Without `--link`**: Instance exists globally, must specify name in commands

## AWS Architecture

### Resources Created
- **Private VPC** with no internet gateway
- **VPC Endpoints**: ssm, ssmmessages, ec2messages, s3 (for package downloads)
- **Security Group**: No inbound rules, outbound only to VPC endpoints
- **IAM Instance Role**: Minimal SSM permissions only
- **EC2 Instance**: Launched in private subnet, no public IP

### Security
- No public IP or internet exposure
- All SSH access via SSM Session Manager (requires `session-manager-plugin`)
- IAM least-privilege for instance role
- Resource tagging for tracking (`ec2-cli:managed=true`)

## Profile System (JSON5)

Location: `~/.config/ec2-cli/profiles/` or `.ec2-cli/profiles/` (project-local)

```json5
{
  "name": "rust-dev",
  "instance": {
    "type": "c6i.xlarge",           // 4 vCPU, 8GB RAM - good for Rust
    "fallback_types": ["c6a.xlarge", "t3.xlarge"],
    "ami": { "type": "amazon-linux-2023", "architecture": "x86_64" },
    "storage": { "root_volume": { "size_gb": 50, "type": "gp3" } }
  },
  "packages": {
    "system": ["gcc", "gcc-c++", "openssl-devel", "pkg-config", "fuse3", "fuse3-devel", "git"],
    "rust": { "enabled": true, "channel": "stable", "components": ["rustfmt", "clippy"] },
    "cargo": ["cargo-watch", "sccache"]
  },
  "environment": { "RUSTC_WRAPPER": "sccache" }
}
```

**Default profile**: `t3.large` (2 vCPU, 8GB), 30GB gp3, Amazon Linux 2023, Rust stable

## Git Integration

1. **On launch**: Creates bare git repo at `/home/ec2-user/repos/<project>.git`
2. **Auto-adds remote**: `git remote add ec2-dev ec2-user@<instance-id>:/home/ec2-user/repos/<project>.git`
3. **Post-receive hook**: Auto-checkouts to `/home/ec2-user/work/<project>/` on push
4. **SSH config required** in `~/.ssh/config`:
   ```
   Host i-* mi-*
       User ec2-user
       ProxyCommand sh -c "aws ssm start-session --target %h --document-name AWS-StartSSHSession --parameters 'portNumber=%p'"
   ```

## State Management

**Global state**: `~/.local/state/ec2-cli/state.json`
- Maps instance names to AWS instance IDs
- Tracks all instances across all projects
- Reconciles with AWS on each command (removes terminated instances)

```json
{
  "instances": {
    "happy-zebra": { "instance_id": "i-abc123", "profile": "rust-dev", "region": "us-west-2", "created_at": "..." },
    "my-dev": { "instance_id": "i-def456", "profile": "default", "region": "us-west-2", "created_at": "..." }
  }
}
```

**Directory link** (optional): `.ec2-cli/instance`
- Created with `--link` flag on launch
- Contains just the instance name (e.g., `happy-zebra`)
- When present, commands in that directory auto-use that instance

## Project Structure

```
src/
├── main.rs                 # Entry point, clap CLI
├── cli/commands/           # Command implementations
│   ├── up.rs, destroy.rs, ssh.rs, scp.rs, push.rs, pull.rs, status.rs, list.rs
├── aws/
│   ├── client.rs           # AWS SDK client setup
│   ├── ec2/instance.rs     # EC2 operations
│   ├── infrastructure.rs   # VPC/SG/endpoint provisioning
├── profile/
│   ├── schema.rs           # Profile structs
│   ├── loader.rs           # Load from files/built-in
├── git/
│   ├── remote.rs           # Git remote management
│   ├── operations.rs       # Push/pull via subprocess
├── state/
│   ├── local.rs            # State file management
├── user_data/
│   └── generator.rs        # EC2 bootstrap script generation
└── error.rs                # Error types
```

## Key Dependencies

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
aws-config = "1.8"
aws-sdk-ec2 = "1.101"
aws-sdk-ssm = "1.60"
serde = { version = "1.0", features = ["derive"] }
json5 = "0.4"
git2 = "0.19"
thiserror = "2.0"
anyhow = "1.0"
indicatif = "0.17"      # Progress bars
dialoguer = "0.11"      # Interactive prompts
directories = "5.0"     # XDG paths
petname = "2.0"         # Random memorable names (happy-zebra, swift-falcon)
```

## Implementation Order

After completing each step, run a sub-agent to review the code and implement its suggestions before moving to the next step.

1. **Project setup**: Cargo.toml, main.rs with clap, error types
   - *Review*: Sub-agent reviews project structure, error handling patterns, CLI design
2. **Profile system**: Schema, loader, validation, default profile
   - *Review*: Sub-agent reviews schema design, validation logic, edge cases
3. **AWS infrastructure**: VPC, subnets, security groups, VPC endpoints, IAM role
   - *Review*: Sub-agent reviews security configuration, IAM policies, resource cleanup
4. **Up command**: AMI lookup, user data generation, instance launch, wait for ready
   - *Review*: Sub-agent reviews user data script, error handling, timeout logic
5. **State management**: Local state file, AWS tag reconciliation
   - *Review*: Sub-agent reviews state consistency, race conditions, file handling
6. **Git integration**: Add/remove remote, SSH config validation
   - *Review*: Sub-agent reviews git operations, SSH config parsing, error messages
7. **SSH command**: SSM session via subprocess
   - *Review*: Sub-agent reviews process spawning, signal handling, UX
8. **SCP command**: File transfer via SSM
   - *Review*: Sub-agent reviews path parsing, transfer logic, progress reporting
9. **Push/pull commands**: Git operations via subprocess
   - *Review*: Sub-agent reviews git subprocess calls, output handling
10. **Destroy command**: Instance termination, remote cleanup
    - *Review*: Sub-agent reviews cleanup completeness, confirmation flow
11. **Status/list commands**: Query AWS, format output
    - *Review*: Sub-agent reviews output formatting, filtering options
12. **Config init**: Validate prerequisites (AWS CLI, session-manager-plugin)
    - *Review*: Sub-agent reviews prerequisite checks, helpful error messages

## Verification

1. `cargo build` - Project compiles
2. `cargo test` - Unit tests pass
3. `ec2-cli config init` - Validates AWS credentials and prerequisites
4. `ec2-cli up` - Launches instance with default profile
5. `ec2-cli ssh <name>` - Can connect via SSM
6. `ec2-cli scp <name> ./test.txt :/home/ec2-user/` - File transfer works
7. `ec2-cli push <name> && ec2-cli ssh <name> -c "ls /home/ec2-user/work/"` - Code synced
8. `ec2-cli destroy <name>` - Cleans up all resources