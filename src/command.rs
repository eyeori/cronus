use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::CronusResult;
use crate::job::{Job, JobInfo};
use crate::nng_socket::NngIpcSocket;

/// `Command` is an enumeration that represents the different types of commands that can be issued.
///
/// # Variants
///
/// * `AddJob` - Represents a command to add a job. It contains a cron string and a `Job` instance.
/// * `ListJobs` - Represents a command to list all jobs.
/// * `DeleteJob` - Represents a command to delete a job. It contains the id of the job to be deleted.
/// * `StopService` - Represents a command to stop the service.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    AddJob { cron: String, job: Job },
    ListJobs,
    DeleteJob { id: String },
    StopService,
}

impl Command {
    /// Creates a new `AddJob` command.
    ///
    /// # Arguments
    ///
    /// * `cron` - A cron string that represents the schedule of the job.
    /// * `job` - A `Job` instance that represents the job to be added.
    ///
    /// # Returns
    ///
    /// * `Command` - Returns a `Command::AddJob` variant.
    pub fn new_add_job(cron: String, job: Job) -> Self {
        Self::AddJob { cron, job }
    }

    /// Creates a new `ListJobs` command.
    ///
    /// # Returns
    ///
    /// * `Command` - Returns a `Command::ListJobs` variant.
    pub fn new_list_jobs() -> Self {
        Self::ListJobs
    }

    /// Creates a new `DeleteJob` command.
    ///
    /// # Arguments
    ///
    /// * `id` - A string that represents the id of the job to be deleted.
    ///
    /// # Returns
    ///
    /// * `Command` - Returns a `Command::DeleteJob` variant.
    pub fn new_delete_job(id: String) -> Self {
        Self::DeleteJob { id }
    }

    /// Creates a new `StopService` command.
    ///
    /// # Returns
    ///
    /// * `Command` - Returns a `Command::StopService` variant.
    pub fn new_stop_service() -> Self {
        Self::StopService
    }

    /// Converts the `Command` instance into a byte vector.
    ///
    /// # Returns
    ///
    /// * `CronusResult<Vec<u8>>` - Returns a `CronusResult` that contains a byte vector on success or an error.
    pub fn to_bytes(&self) -> CronusResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(Into::into)
    }

    /// Creates a `Command` instance from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `cmd` - A byte slice that represents the `Command` instance.
    ///
    /// # Returns
    ///
    /// * `CronusResult<Command>` - Returns a `CronusResult` that contains a `Command` instance on success or an error.
    pub fn from_bytes(cmd: &[u8]) -> CronusResult<Self> {
        serde_json::from_slice::<Self>(cmd).map_err(Into::into)
    }
}

/// `CommandResponse` is an enumeration that represents the different types of responses that can be returned by commands.
///
/// # Variants
///
/// * `JobAdded(String)` - Represents a response for a successful `AddJob` command. It contains a string that represents the id of the added job.
/// * `JobList(Vec<JobInfo>)` - Represents a response for a `ListJobs` command. It contains a vector of `JobInfo` instances that represent the list of jobs.
/// * `JobDeleted` - Represents a response for a successful `DeleteJob` command.
/// * `ServiceStopped` - Represents a response for a successful `StopService` command.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CommandResponse {
    JobAdded(String),
    JobList(Vec<JobInfo>),
    JobDeleted,
    ServiceStopped,
}

impl CommandResponse {
    /// Converts the `CommandResponse` instance into a byte vector.
    ///
    /// # Returns
    ///
    /// * `CronusResult<Vec<u8>>` - Returns a `CronusResult` that contains a byte vector on success or an error.
    pub fn to_bytes(&self) -> CronusResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(Into::into)
    }

    /// Creates a `CommandResponse` instance from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `cmd` - A byte slice that represents the `CommandResponse` instance.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse` instance on success or an error.
    pub fn from_bytes(cmd: &[u8]) -> CronusResult<Self> {
        serde_json::from_slice::<Self>(cmd).map_err(Into::into)
    }
}

/// `CommandProxy` is a struct that wraps an `NngIpcSocket` instance.
///
/// It provides methods to send different types of `Command` instances to the socket and receive `CommandResponse` instances.
///
/// # Fields
///
/// * `NngIpcSocket` - An instance of `NngIpcSocket` that is used to send and receive commands.
pub struct CommandProxy(NngIpcSocket);

impl CommandProxy {
    /// Creates a new `CommandProxy` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - A string that represents the name of the socket.
    /// * `path` - A `PathBuf` that represents the path of the socket.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandProxy>` - Returns a `CronusResult` that contains a `CommandProxy` instance on success or an error.
    pub fn new(name: String, path: PathBuf) -> CronusResult<Self> {
        Ok(Self(NngIpcSocket::new_dial_sync(path.join(name))?))
    }

    /// Sends an `AddJob` command to the socket.
    ///
    /// # Arguments
    ///
    /// * `corn` - A cron string that represents the schedule of the job.
    /// * `job` - A `Job` instance that represents the job to be added.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse` instance on success or an error.
    pub fn add_cmd_job(&self, corn: String, job: Job) -> CronusResult<CommandResponse> {
        self.send_cmd(Command::new_add_job(corn, job))
    }

    /// Sends a `ListJobs` command to the socket.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse` instance on success or an error.
    pub fn list_jobs(&self) -> CronusResult<CommandResponse> {
        self.send_cmd(Command::new_list_jobs())
    }

    /// Sends a `DeleteJob` command to the socket.
    ///
    /// # Arguments
    ///
    /// * `id` - A string that represents the id of the job to be deleted.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse` instance on success or an error.
    pub fn delete_job(&self, id: String) -> CronusResult<CommandResponse> {
        self.send_cmd(Command::new_delete_job(id))
    }

    /// Sends a `StopService` command to the socket.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse` instance on success or an error.
    pub fn shutdown(&self) -> CronusResult<CommandResponse> {
        self.send_cmd(Command::new_stop_service())
    }

    /// Sends a `Command` instance to the socket and receives a `CommandResponse` instance.
    ///
    /// # Arguments
    ///
    /// * `cmd` - A `Command` instance that represents the command to be sent.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse` instance on success or an error.
    fn send_cmd(&self, cmd: Command) -> CronusResult<CommandResponse> {
        self.0.send(&cmd.to_bytes()?)?;
        let msg = self.0.recv()?;
        CommandResponse::from_bytes(&msg[..])
    }
}