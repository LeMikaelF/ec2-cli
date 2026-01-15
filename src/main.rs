use clap::{Parser, Subcommand};

mod error;

pub use error::{Ec2CliError, Result};

#[derive(Parser)]
#[command(name = "ec2-cli")]
#[command(about = "Ephemeral EC2 Development Environment Manager")]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch a new EC2 instance
    Up {
        /// Profile name to use (default if omitted)
        #[arg(short, long)]
        profile: Option<String>,

        /// Custom instance name
        #[arg(short, long)]
        name: Option<String>,

        /// Link instance to current directory
        #[arg(short, long)]
        link: bool,
    },

    /// Terminate instance and cleanup resources
    Destroy {
        /// Instance name
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// SSH into instance via SSM Session Manager
    Ssh {
        /// Instance name
        name: String,

        /// Command to execute
        #[arg(short = 'c', long)]
        command: Option<String>,
    },

    /// Copy files to/from EC2 instance via SSM
    Scp {
        /// Instance name
        name: String,

        /// Source path (prefix with : for remote)
        src: String,

        /// Destination path (prefix with : for remote)
        dest: String,

        /// Copy directories recursively
        #[arg(short, long)]
        recursive: bool,
    },

    /// Push code to EC2 bare repo
    Push {
        /// Instance name
        name: String,

        /// Branch to push
        #[arg(short, long)]
        branch: Option<String>,
    },

    /// Pull from EC2 bare repo
    Pull {
        /// Instance name
        name: String,

        /// Branch to pull
        #[arg(short, long)]
        branch: Option<String>,
    },

    /// Show instance status
    Status {
        /// Instance name (optional if linked)
        name: Option<String>,
    },

    /// List managed instances
    List {
        /// Show all instances including terminated
        #[arg(short, long)]
        all: bool,
    },

    /// Manage EC2 profiles
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    /// Configure CLI and check prerequisites
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// View cloud-init logs from instance
    Logs {
        /// Instance name
        name: String,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// List available profiles
    List,

    /// Show profile details
    Show {
        /// Profile name
        name: String,
    },

    /// Validate a profile
    Validate {
        /// Profile name
        name: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Initialize configuration and check prerequisites
    Init,

    /// Show current configuration
    Show,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Up { profile, name, link } => {
            println!("Launching EC2 instance...");
            println!("Profile: {:?}, Name: {:?}, Link: {}", profile, name, link);
            todo!("Implement up command")
        }
        Commands::Destroy { name, force } => {
            println!("Destroying instance: {}, Force: {}", name, force);
            todo!("Implement destroy command")
        }
        Commands::Ssh { name, command } => {
            println!("SSH to: {}, Command: {:?}", name, command);
            todo!("Implement ssh command")
        }
        Commands::Scp {
            name,
            src,
            dest,
            recursive,
        } => {
            println!(
                "SCP {} -> {} on {}, Recursive: {}",
                src, dest, name, recursive
            );
            todo!("Implement scp command")
        }
        Commands::Push { name, branch } => {
            println!("Push to: {}, Branch: {:?}", name, branch);
            todo!("Implement push command")
        }
        Commands::Pull { name, branch } => {
            println!("Pull from: {}, Branch: {:?}", name, branch);
            todo!("Implement pull command")
        }
        Commands::Status { name } => {
            println!("Status for: {:?}", name);
            todo!("Implement status command")
        }
        Commands::List { all } => {
            println!("List instances, All: {}", all);
            todo!("Implement list command")
        }
        Commands::Profile { command } => match command {
            ProfileCommands::List => {
                println!("Listing profiles...");
                todo!("Implement profile list")
            }
            ProfileCommands::Show { name } => {
                println!("Showing profile: {}", name);
                todo!("Implement profile show")
            }
            ProfileCommands::Validate { name } => {
                println!("Validating profile: {}", name);
                todo!("Implement profile validate")
            }
        },
        Commands::Config { command } => match command {
            ConfigCommands::Init => {
                println!("Initializing configuration...");
                todo!("Implement config init")
            }
            ConfigCommands::Show => {
                println!("Showing configuration...");
                todo!("Implement config show")
            }
        },
        Commands::Logs { name, follow } => {
            println!("Logs for: {}, Follow: {}", name, follow);
            todo!("Implement logs command")
        }
    }
}
