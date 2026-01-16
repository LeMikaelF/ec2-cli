# ec2-cli

Ephemeral EC2 Development Environment Manager - Launch temporary, private EC2 instances for remote development with automatic Docker and Rust toolchain setup.

## Features

- **Private VPC**: Instances run in an isolated VPC with no internet access
- **SSM Access**: Connect via AWS Systems Manager Session Manager (no SSH keys needed)
- **Docker Pre-installed**: Docker is automatically installed and configured on all instances
- **Rust Toolchain**: Rust, rustfmt, and clippy installed by default
- **Git Integration**: Automatic bare repo setup with post-receive hooks for code syncing
- **Resource Tagging**: Configurable tags (including Username) applied to all AWS resources
- **Profile System**: Customizable instance configurations via JSON5 profiles

## Prerequisites

- AWS CLI configured with credentials (`aws configure`)
- [Session Manager Plugin](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html)
- Git
- SSH config for SSM proxy (see below)

## Installation

```bash
cargo install --path .
```

## Quick Start

1. Initialize and check prerequisites:
   ```bash
   ec2-cli config init
   ```
   This will prompt you for your Username tag (used to identify your resources).

2. Launch an instance:
   ```bash
   ec2-cli up
   ```

3. Connect to your instance:
   ```bash
   ec2-cli ssh <instance-name>
   ```

4. Push code to your instance:
   ```bash
   ec2-cli push <instance-name>
   ```

5. Terminate when done:
   ```bash
   ec2-cli destroy <instance-name>
   ```

## Commands

| Command | Description |
|---------|-------------|
| `ec2-cli up [--profile NAME] [--name NAME] [--link]` | Launch a new instance |
| `ec2-cli destroy <NAME> [--force]` | Terminate an instance |
| `ec2-cli ssh <NAME> [-c COMMAND]` | SSH into instance via SSM |
| `ec2-cli scp <NAME> <SRC> <DEST> [--recursive]` | Copy files to/from instance |
| `ec2-cli push <NAME> [--branch BRANCH]` | Push code to instance bare repo |
| `ec2-cli pull <NAME> [--branch BRANCH]` | Pull from instance bare repo |
| `ec2-cli status [NAME]` | Show instance status |
| `ec2-cli list [--all]` | List managed instances |
| `ec2-cli logs <NAME> [--follow]` | View cloud-init logs |
| `ec2-cli config init` | Initialize config and check prerequisites |
| `ec2-cli config show` | Show current configuration |
| `ec2-cli config tags set <KEY> <VALUE>` | Set a custom resource tag |
| `ec2-cli config tags list` | List configured tags |
| `ec2-cli config tags remove <KEY>` | Remove a custom tag |
| `ec2-cli profile list` | List available profiles |
| `ec2-cli profile show <NAME>` | Show profile details |
| `ec2-cli profile validate <NAME>` | Validate a profile |

## SSH Configuration

Add to `~/.ssh/config`:

```
# EC2 SSH via SSM Session Manager
Host i-* mi-*
    User ubuntu
    ProxyCommand sh -c "aws ssm start-session --target %h --document-name AWS-StartSSHSession --parameters 'portNumber=%p'"
```

## Profiles

Profiles define instance configuration. Create profiles in:
- Global: `~/.config/ec2-cli/profiles/`
- Local: `.ec2-cli/profiles/` (project-specific)

Example profile (`~/.config/ec2-cli/profiles/large.json5`):
```json5
{
  "name": "large",
  "instance": {
    "type": "t3.xlarge",
    "ami": {
      "type": "ubuntu-24.04",
      "architecture": "x86_64"
    },
    "storage": {
      "root_volume": {
        "size_gb": 100,
        "type": "gp3"
      }
    }
  },
  "packages": {
    "system": ["build-essential", "libssl-dev", "pkg-config", "git"],
    "rust": {
      "enabled": true,
      "channel": "stable",
      "components": ["rustfmt", "clippy"]
    },
    "cargo": ["cargo-watch", "cargo-edit"]
  },
  "environment": {
    "RUST_BACKTRACE": "1"
  }
}
```

Supported AMI types:
- `ubuntu-24.04` (default)
- `ubuntu-22.04`

## Custom Tags

All AWS resources are tagged with:
- `ec2-cli:managed=true` (identifies managed resources)
- `ec2-cli:name=<name>` (instance/resource name)
- `Name=ec2-cli-<name>` (AWS console display)
- Custom tags from `ec2-cli config tags set`

Set your Username tag for resource identification:
```bash
ec2-cli config tags set Username myusername
```

## Security

- **Private VPC**: No internet gateway, all traffic via VPC endpoints
- **IMDSv2 Required**: Prevents SSRF credential theft attacks
- **Encrypted Volumes**: All EBS volumes are encrypted
- **Minimal IAM**: Instance role has only SSM permissions
- **No Public IP**: Instances are not publicly accessible

## Directory Structure

```
~/.config/ec2-cli/
├── config.json          # Custom tags and settings
└── profiles/            # Global profiles

~/.local/state/ec2-cli/
└── state.json           # Instance state tracking

.ec2-cli/                # Project-local (optional)
├── instance             # Linked instance name
└── profiles/            # Local profiles
```

## License

MIT
