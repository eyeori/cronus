use crate::job::Job;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// The Cronus system.
///
/// # Variants
///
/// * `Start` - Starts the Cronus service.
/// * `Stop` - Stops the Cronus service.
/// * `Add` - Adds a cron job to the Cronus service.
/// * `Delete` - Deletes a cron job from the Cronus service.
/// * `List` - Lists the cron jobs on the Cronus service.
/// * `Run` - Runs the Cronus service.
/// * `Status` - Get Cronus service status.
#[derive(Parser)]
#[command(version, about, long_about = "Scheduled task execution manager")]
#[command(propagate_version = true)]
pub enum Command {
    /// Start cronus service
    Start {
        /// Cronus service command acceptance name
        #[arg(short, long, default_value = "cronus")]
        name: String,

        /// Cronus service nng ipc communication file path
        #[arg(short, long, default_value = "/tmp")]
        path: PathBuf,
    },
    /// Stop cronus service
    Stop {
        /// Cronus service command acceptance name
        #[arg(short, long, default_value = "cronus")]
        name: String,

        /// Cronus service nng ipc communication file path
        #[arg(short, long, default_value = "/tmp")]
        path: PathBuf,
    },
    /// Add a cron job to cronus service
    Add {
        /// Cronus service command acceptance name
        #[arg(short, long, default_value = "cronus")]
        name: String,

        /// Cronus service nng ipc communication file path
        #[arg(short, long, default_value = "/tmp")]
        path: PathBuf,

        /// Cron expression for the job to be added to cronus service
        #[arg(short, long)]
        cron: String,

        #[command(subcommand)]
        cmd: AddSubCommand,
    },
    /// Delete a cron job from cronus service
    Delete {
        /// Cronus service command acceptance name
        #[arg(short, long, default_value = "cronus")]
        name: String,

        /// Cronus service nng ipc communication file path
        #[arg(short, long, default_value = "/tmp")]
        path: PathBuf,

        /// Cron job id to be deleted from cronus service
        #[arg(short, long)]
        id: String,
    },
    /// List cron job on cronus service
    List {
        /// Cronus service command acceptance name
        #[arg(short, long, default_value = "cronus")]
        name: String,

        /// Cronus service nng ipc communication file path
        #[arg(short, long, default_value = "/tmp")]
        path: PathBuf,
    },
    /// Run cronus service
    Run {
        /// Cronus service command acceptance name
        #[arg(short, long, default_value = "cronus")]
        name: String,

        /// Cronus service nng ipc communication file path
        #[arg(short, long, default_value = "/tmp")]
        path: PathBuf,
    },
    /// Get cronus service status
    Status {
        /// Cronus service command acceptance name
        #[arg(short, long, default_value = "cronus")]
        name: String,

        /// Cronus service nng ipc communication file path
        #[arg(short, long, default_value = "/tmp")]
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
/// * `Cmd` - Represents a command job. It contains the path to the command and the arguments for the command.
/// * `Rhai` - Represents a Rhai job. It contains the Rhai script code.
/// * `RhaiFile` - Represents a Rhai file job. It contains the path to the Rhai script file.
#[derive(Subcommand, Debug)]
pub enum AddSubCommand {
    /// Command Job
    Cmd {
        /// Command path
        #[arg(short, long)]
        cmd: PathBuf,

        /// Command args
        #[arg(short, long)]
        args: Vec<String>,
    },
    /// Rhai Job
    Rhai {
        /// Rhai script code
        #[arg(short, long)]
        script: String,
    },
    /// Rhai file Job
    RhaiFile {
        /// Rhai script file path
        #[arg(short, long)]
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
    pub fn into_job(self) -> Job {
        match self {
            AddSubCommand::Cmd { cmd, args } => Job::command(&cmd, args),
            AddSubCommand::Rhai { script } => Job::rhai_script(script),
            AddSubCommand::RhaiFile { script_file } => Job::rhai_script_file(&script_file),
        }
    }
}
