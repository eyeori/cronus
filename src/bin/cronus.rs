use std::path::PathBuf;

use fork::{daemon, Fork};
use structopt::StructOpt;
use uuid::Uuid;

use cronus::command::{CommandClient, CommandResponse};
use cronus::CronusResult;
use cronus::job::Job;
use cronus::scheduler::CronusScheduler;

/// The `Command` enum.
///
/// This enum represents the different commands that the Cronus task execution manager can handle.
/// Each variant of the enum corresponds to a different command, and contains the arguments for that command.
///
/// # Variants
///
/// * `Start` - Starts the Cronus service.
/// * `Stop` - Stops the Cronus service.
/// * `Add` - Adds a cron job to the Cronus service.
/// * `Delete` - Deletes a cron job from the Cronus service.
/// * `List` - Lists the cron jobs on the Cronus service.
/// * `Run` - Runs the Cronus service.
#[derive(StructOpt, Debug)]
#[structopt(name = "Cronus", about = "Scheduled task execution manager")]
enum Command {
    #[structopt(about = "Start cronus service")]
    Start {
        #[structopt(
            short,
            long,
            default_value = "cronus",
            long_help = "Cronus service command acceptance name"
        )]
        name: String,

        #[structopt(
            short,
            long,
            default_value = "/tmp",
            long_help = "Cronus service command acceptance path"
        )]
        path: PathBuf,
    },
    #[structopt(about = "Stop cronus service")]
    Stop {
        #[structopt(
            short,
            long,
            default_value = "cronus",
            long_help = "Cronus service command acceptance name"
        )]
        name: String,

        #[structopt(
            short,
            long,
            default_value = "/tmp",
            long_help = "Cronus service command acceptance path"
        )]
        path: PathBuf,
    },
    #[structopt(about = "Add a cron job to cronus service")]
    Add {
        #[structopt(
            short,
            long,
            default_value = "cronus",
            long_help = "Cronus service command acceptance name"
        )]
        name: String,

        #[structopt(
            short,
            long,
            default_value = "/tmp",
            long_help = "Cronus service command acceptance path"
        )]
        path: PathBuf,

        #[structopt(
            short,
            long,
            long_help = "Corn expression for the job to be added to cronus service"
        )]
        corn: String,

        #[structopt(subcommand)]
        sub_cmd: AddSubCommand,
    },
    #[structopt(about = "Delete cron job from cronus service")]
    Delete {
        #[structopt(
            short,
            long,
            default_value = "cronus",
            long_help = "Cronus service command acceptance name"
        )]
        name: String,

        #[structopt(
            short,
            long,
            default_value = "/tmp",
            long_help = "Cronus service command acceptance path"
        )]
        path: PathBuf,

        #[structopt(
            short,
            long,
            long_help = "Corn job id to be deleted from cronus service"
        )]
        id: String,
    },
    #[structopt(about = "List cron job on cronus service")]
    List {
        #[structopt(
            short,
            long,
            default_value = "cronus",
            long_help = "Cronus service command acceptance name"
        )]
        name: String,

        #[structopt(
            short,
            long,
            default_value = "/tmp",
            long_help = "Cronus service command acceptance path"
        )]
        path: PathBuf,
    },

    #[structopt(about = "Run cronus service")]
    Run {
        #[structopt(
            short,
            long,
            default_value = "cronus",
            long_help = "Cronus service command acceptance name"
        )]
        name: String,

        #[structopt(
            short,
            long,
            default_value = "/tmp",
            long_help = "Cronus service command acceptance path"
        )]
        path: PathBuf,
    },
}

/// The `AddSubCommand` enum.
///
/// This enum represents the different subcommands that can be used with the `Add` command of the Cronus task execution manager.
/// Each variant of the enum corresponds to a different subcommand, and contains the arguments for that subcommand.
///
/// # Variants
///
/// * `CmdJob` - Represents a command job. It contains the path to the command and the arguments for the command.
/// * `RhaiJob` - Represents a Rhai job. It contains the Rhai script code.
/// * `RhaiFileJob` - Represents a Rhai file job. It contains the path to the Rhai script file.
#[derive(StructOpt, Debug)]
enum AddSubCommand {
    #[structopt(about = "Command Job")]
    CmdJob {
        #[structopt(short, long, parse(from_os_str), long_help = "Command path")]
        cmd: PathBuf,

        #[structopt(short, long, long_help = "Command args")]
        args: Vec<String>,
    },
    #[structopt(about = "Rhai Job")]
    RhaiJob {
        #[structopt(short, long, long_help = "Rhai script code")]
        script: String,
    },
    #[structopt(about = "Rhai file Job")]
    RhaiFileJob {
        #[structopt(short, long, parse(from_os_str), long_help = "Rhai script file path")]
        script_file: PathBuf,
    },
}

impl AddSubCommand {
    /// Converts the `AddSubCommand` enum into a `Job`.
    ///
    /// This method is used to convert the different subcommands that can be used with the `Add` command of the Cronus task execution manager into a `Job`.
    /// Each variant of the `AddSubCommand` enum corresponds to a different type of job, and this method will create a `Job` of the appropriate type based on the variant.
    ///
    /// # Returns
    ///
    /// * `Job` - The `Job` that corresponds to the `AddSubCommand`.
    fn into_job(self) -> Job {
        match self {
            AddSubCommand::CmdJob { cmd, args } => Job::new_command(cmd, args),
            AddSubCommand::RhaiJob { script } => Job::new_rhai_script(script),
            AddSubCommand::RhaiFileJob { script_file } => Job::new_rhai_script_file(script_file),
        }
    }
}

#[tokio::main]
async fn main() -> CronusResult<()> {
    let response = match Command::from_args() {
        Command::Start { name, path } => {
            let cronus = std::env::current_exe()?;
            match daemon(false, false) {
                Ok(Fork::Child) => {
                    std::process::Command::new(cronus)
                        .arg("run")
                        .arg("--name")
                        .arg(name)
                        .arg("--path")
                        .arg(path)
                        .spawn()?;
                    std::process::exit(0);
                }
                _ => std::process::exit(0),
            }
        }
        Command::Stop { name, path } => {
            let cc = CommandClient::new(name, path)?;
            cc.stop_service()?
        }
        Command::Add {
            name,
            path,
            corn,
            sub_cmd,
        } => {
            let cc = CommandClient::new(name, path)?;
            cc.add_job(corn, sub_cmd.into_job())?
        }
        Command::Delete { name, path, id } => {
            Uuid::parse_str(&id).map_err(|_| "Invalid job id")?;
            let cc = CommandClient::new(name, path)?;
            cc.delete_job(id)?
        }
        Command::List { name, path } => {
            let cc = CommandClient::new(name, path)?;
            cc.list_jobs()?
        }
        Command::Run { name, path } => {
            let scheduler = CronusScheduler::new(name, path).await?;
            scheduler.run().await?
        }
    };

    match response {
        CommandResponse::JobAdded(id) => {
            println!("Job added with id: {}", id);
        }
        CommandResponse::JobList(jobs) => {
            for job in jobs {
                println!("{:?}", job);
            }
        }
        _ => {}
    }
    Ok(())
}
