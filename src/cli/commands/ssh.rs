use std::borrow::Cow;

use shell_escape::escape;

use super::ssm_ssh_options;
use crate::state::{get_instance, resolve_instance_name};
use crate::{Ec2CliError, Result};

pub fn execute(name: String, command: Option<String>) -> Result<()> {
    // Resolve instance name
    let name = resolve_instance_name(Some(&name))?;

    // Get instance from state
    let instance_state = get_instance(&name)?
        .ok_or_else(|| Ec2CliError::InstanceNotFound(name.clone()))?;

    let target = format!(
        "{}@{}",
        instance_state.username, instance_state.instance_id
    );
    let ssm_opts = ssm_ssh_options();

    match command {
        Some(cmd) => println!("ssh {} {} {}", ssm_opts, target, escape(Cow::Borrowed(&cmd))),
        None => println!("ssh {} {}", ssm_opts, target),
    }

    Ok(())
}
