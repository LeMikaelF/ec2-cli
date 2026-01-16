use std::borrow::Cow;

use shell_escape::escape;

use super::ssm_ssh_options;
use crate::state::{get_instance, resolve_instance_name};
use crate::{Ec2CliError, Result};

pub fn execute(name: String, src: String, dest: String, recursive: bool) -> Result<()> {
    // Resolve instance name
    let name = resolve_instance_name(Some(&name))?;

    // Get instance from state
    let instance_state = get_instance(&name)?
        .ok_or_else(|| Ec2CliError::InstanceNotFound(name.clone()))?;

    // Parse source and destination to determine direction
    let (local_path, remote_path, is_upload) = parse_paths(&src, &dest)?;

    let escaped_remote_path = escape(Cow::Borrowed(&remote_path));
    let remote = format!(
        "{}@{}:{}",
        instance_state.username, instance_state.instance_id, escaped_remote_path
    );

    let ssm_opts = ssm_ssh_options();
    let recursive_flag = if recursive { "-r " } else { "" };
    let escaped_local = escape(Cow::Borrowed(&local_path));

    if is_upload {
        println!("scp {} {}{} {}", ssm_opts, recursive_flag, escaped_local, remote);
    } else {
        println!("scp {} {}{} {}", ssm_opts, recursive_flag, remote, escaped_local);
    }

    Ok(())
}

fn parse_paths(src: &str, dest: &str) -> Result<(String, String, bool)> {
    let src_is_remote = src.starts_with(':');
    let dest_is_remote = dest.starts_with(':');

    match (src_is_remote, dest_is_remote) {
        (false, true) => {
            // Upload: local src -> remote dest
            Ok((src.to_string(), dest[1..].to_string(), true))
        }
        (true, false) => {
            // Download: remote src -> local dest
            Ok((dest.to_string(), src[1..].to_string(), false))
        }
        (true, true) => Err(Ec2CliError::InvalidPath(
            "Both source and destination cannot be remote".to_string(),
        )),
        (false, false) => Err(Ec2CliError::InvalidPath(
            "One of source or destination must be remote (prefix with :)".to_string(),
        )),
    }
}
