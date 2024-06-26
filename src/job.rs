use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// `Job` is an enumeration that represents the different types of jobs that can be scheduled.
///
/// # Variants
///
/// * `Command(PathBuf, Vec<String>)` - Represents a command job. It contains a `PathBuf` that represents the path of the command and a vector of strings that represent the arguments of the command.
/// * `RhaiScript(String)` - Represents a Rhai script job. It contains a string that represents the Rhai script.
/// * `RhaiScriptFile(PathBuf)` - Represents a Rhai script file job. It contains a `PathBuf` that represents the path of the Rhai script file.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Job {
    Command(PathBuf, Vec<String>),
    RhaiScript(String),
    RhaiScriptFile(PathBuf),
}

impl Job {
    /// Creates a new `Command` variant of `Job`.
    ///
    /// # Arguments
    ///
    /// * `cmd_path` - A `PathBuf` that represents the path of the command.
    /// * `args` - A vector of strings that represent the arguments of the command.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns a new `Command` variant of `Job`.
    pub fn new_command(cmd_path: PathBuf, args: Vec<String>) -> Self {
        Job::Command(cmd_path, args)
    }

    /// Creates a new `RhaiScript` variant of `Job`.
    ///
    /// # Arguments
    ///
    /// * `script` - An instance of a type implementing `ToString` that represents the Rhai script.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns a new `RhaiScript` variant of `Job`.
    pub fn new_rhai_script(script: impl ToString) -> Self {
        Job::RhaiScript(script.to_string())
    }

    /// Creates a new `RhaiScriptFile` variant of `Job`.
    ///
    /// # Arguments
    ///
    /// * `file` - A `PathBuf` that represents the path of the Rhai script file.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns a new `RhaiScriptFile` variant of `Job`.
    pub fn new_rhai_script_file(file: PathBuf) -> Self {
        Job::RhaiScriptFile(file)
    }

    /// Converts a `Job` instance into a business function.
    ///
    /// This method matches the `Job` variant and calls the corresponding method to convert it into a business function.
    ///
    /// # Arguments
    ///
    /// * `self` - The instance of `Job` that needs to be converted.
    ///
    /// # Returns
    ///
    /// * `Arc<dyn Fn(DateTime<Utc>) + Send + Sync>` - Returns an `Arc` containing a dynamic function that takes a `DateTime<Utc>` as an argument and implements `Send` and `Sync`.
    pub fn to_business(self) -> Arc<dyn Fn(DateTime<Utc>) + Send + Sync> {
        match self {
            Job::Command(cmd_path, args) => Job::command_to_business(cmd_path, args),
            Job::RhaiScript(script) => Job::rhai_script_to_business(script),
            Job::RhaiScriptFile(file) => Job::rhai_script_file_to_business(file),
        }
    }

    /// Converts a `Command` variant of `Job` into a business function.
    ///
    /// This function creates a new process for the command and its arguments. The process is then spawned asynchronously.
    ///
    /// # Arguments
    ///
    /// * `cmd_path` - A `PathBuf` that represents the path of the command.
    /// * `args` - A vector of strings that represent the arguments of the command.
    ///
    /// # Returns
    ///
    /// * `Arc<dyn Fn(DateTime<Utc>) + Send + Sync>` - Returns an `Arc` containing a dynamic function that takes a `DateTime<Utc>` as an argument and implements `Send` and `Sync`.
    fn command_to_business(
        cmd_path: PathBuf,
        args: Vec<String>,
    ) -> Arc<dyn Fn(DateTime<Utc>) + Send + Sync> {
        Arc::new(move |_| {
            let mut cmd = std::process::Command::new(cmd_path.clone());
            for arg in &args {
                cmd.arg(arg);
            }
            _ = cmd.spawn();
        })
    }

    /// Converts a `RhaiScript` variant of `Job` into a business function.
    ///
    /// This function runs the Rhai script asynchronously.
    ///
    /// # Arguments
    ///
    /// * `script` - A string that represents the Rhai script.
    ///
    /// # Returns
    ///
    /// * `Arc<dyn Fn(DateTime<Utc>) + Send + Sync>` - Returns an `Arc` containing a dynamic function that takes a `DateTime<Utc>` as an argument and implements `Send` and `Sync`.
    fn rhai_script_to_business(script: String) -> Arc<dyn Fn(DateTime<Utc>) + Send + Sync> {
        Arc::new(move |_| {
            _ = rhai::run(&script);
        })
    }

    /// Converts a `RhaiScriptFile` variant of `Job` into a business function.
    ///
    /// This function runs the Rhai script file asynchronously.
    ///
    /// # Arguments
    ///
    /// * `file` - A `PathBuf` that represents the path of the Rhai script file.
    ///
    /// # Returns
    ///
    /// * `Arc<dyn Fn(DateTime<Utc>) + Send + Sync>` - Returns an `Arc` containing a dynamic function that takes a `DateTime<Utc>` as an argument and implements `Send` and `Sync`.
    fn rhai_script_file_to_business(file: PathBuf) -> Arc<dyn Fn(DateTime<Utc>) + Send + Sync> {
        Arc::new(move |_| {
            _ = rhai::run_file(file.clone());
        })
    }
}

/// `JobInfo` is a structure that represents the information of a job.
///
/// # Fields
///
/// * `id` - A string that represents the unique identifier of the job.
/// * `cron` - A string that represents the cron schedule of the job.
/// * `last_run` - An `Option<u64>` that represents the last run time of the job in Unix timestamp. It is `None` if the job has never been run.
/// * `next_run` - An `Option<u64>` that represents the next run time of the job in Unix timestamp. It is `None` if the job is not scheduled to run.
/// * `job` - A `Job` that represents the job itself.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: String,
    pub cron: String,
    pub last_run: Option<u64>,
    pub next_run: Option<u64>,
    pub job: Job,
}
