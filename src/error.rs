use thiserror::Error;

#[derive(Error, Debug)]
pub enum Ec2CliError {
    // AWS Errors
    #[error("AWS SDK error: {0}")]
    AwsSdk(String),

    #[error("AWS EC2 error: {0}")]
    Ec2(String),

    #[error("AWS SSM error: {0}")]
    Ssm(String),

    #[error("AWS IAM error: {0}")]
    Iam(String),

    #[error("AWS credentials not found or invalid")]
    AwsCredentials,

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Resource already exists: {0}")]
    ResourceAlreadyExists(String),

    // Profile Errors
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Invalid profile: {0}")]
    ProfileInvalid(String),

    #[error("Profile validation failed: {0}")]
    ProfileValidation(String),

    // Instance Errors
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("Instance name already in use: {0}")]
    InstanceNameExists(String),

    #[error("Instance not ready: {0}")]
    InstanceNotReady(String),

    #[error("Instance in unexpected state: {0}")]
    InstanceState(String),

    // State Errors
    #[error("State file error: {0}")]
    StateFile(String),

    #[error("State file corrupted: {0}")]
    StateCorrupted(String),

    // Git Errors
    #[error("Git error: {0}")]
    Git(String),

    #[error("Not a git repository")]
    NotGitRepo,

    #[error("Git remote already exists: {0}")]
    GitRemoteExists(String),

    // SSH/SCP Errors
    #[error("Session Manager plugin not found. Install from: https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html")]
    SessionManagerPluginNotFound,

    #[error("SSH command failed: {0}")]
    SshCommand(String),

    #[error("SCP transfer failed: {0}")]
    ScpTransfer(String),

    // Path Errors
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    // Config Errors
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Prerequisites not met: {0}")]
    Prerequisites(String),

    #[error("No default VPC found in region. Please specify a VPC ID in config.")]
    NoDefaultVpc,

    #[error("VPC not found: {0}")]
    VpcNotFound(String),

    #[error("Subnet not found: {0}")]
    SubnetNotFound(String),

    #[error("No subnets found in VPC: {0}")]
    NoSubnetsInVpc(String),

    #[error("Subnet must be configured. Run 'ec2-cli config init' first.")]
    SubnetNotConfigured,

    // File/IO Errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    // Timeout
    #[error("Operation timed out: {0}")]
    Timeout(String),

    // User cancelled
    #[error("Operation cancelled by user")]
    Cancelled,

    // Generic
    #[error("{0}")]
    Other(String),
}

impl Ec2CliError {
    pub fn aws_sdk(err: impl std::fmt::Display) -> Self {
        Ec2CliError::AwsSdk(err.to_string())
    }

    pub fn ec2(err: impl std::fmt::Display) -> Self {
        Ec2CliError::Ec2(err.to_string())
    }

    pub fn ssm(err: impl std::fmt::Display) -> Self {
        Ec2CliError::Ssm(err.to_string())
    }

    pub fn iam(err: impl std::fmt::Display) -> Self {
        Ec2CliError::Iam(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Ec2CliError>;
