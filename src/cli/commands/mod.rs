pub mod config;
pub mod destroy;
pub mod list;
pub mod logs;
pub mod pull;
pub mod push;
pub mod scp;
pub mod ssh;
pub mod status;
pub mod up;

/// Returns SSH options for routing connections through AWS SSM Session Manager.
/// These options configure SSH to use SSM as a proxy and disable host key checking
/// since instance IDs won't be in known_hosts.
pub fn ssm_ssh_options() -> &'static str {
    concat!(
        "-o 'ProxyCommand=sh -c \"aws ssm start-session --target %h ",
        "--document-name AWS-StartSSHSession --parameters portNumber=%p\"' ",
        "-o StrictHostKeyChecking=no ",
        "-o UserKnownHostsFile=/dev/null"
    )
}
