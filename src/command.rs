use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::job::{Job, JobInfo};
use crate::nng_socket::NngIpcSocket;
use crate::CronusResult;

/// `Command` is an enumeration that represents the different types of commands that can be issued.
///
/// # Variants
///
/// * `AddJob` - Represents a command to add a job. It contains a cron string and a `Job` instance.
/// * `ListJobs` - Represents a command to list all jobs.
/// * `DeleteJob` - Represents a command to delete a job. It contains the id of the job to be deleted.
/// * `StopService` - Represents a command to stop the service.
/// * `PingService` - Represents a command to ping the service.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    AddJob { cron: String, job: Job },
    ListJobs,
    DeleteJob { id: String },
    StopService,
    PingService,
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

    /// Creates a new `PingService` command.
    ///
    /// # Returns
    ///
    /// * `Command` - Returns a `Command::PingService` variant.
    pub fn new_ping_service() -> Self {
        Self::PingService
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
/// * `ServiceRunning` - Represents a response for a successful `PingService` command.
/// * `ServiceStopped` - Represents a response for a successful `StopService` command.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CommandResponse {
    JobAdded(String),
    JobList(Vec<JobInfo>),
    JobDeleted,
    ServiceRunning,
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

    /// Converts the `CommandResponse` instance into a JSON message.
    ///
    /// This function serializes the `CommandResponse` instance into a JSON string. The structure of the JSON string depends on the variant of the `CommandResponse` instance.
    ///
    /// # Returns
    ///
    /// * `String` - Returns a JSON string that represents the `CommandResponse` instance.
    pub fn to_json_msg(&self) -> String {
        let json_msg = match self {
            Self::JobAdded(id) => json!({"job_id": id}),
            Self::JobList(jobs) => json!(jobs),
            Self::JobDeleted => json!({"message": "Job deleted"}),
            Self::ServiceRunning => json!({"message": "Service running"}),
            Self::ServiceStopped => json!({"message": "Service stopped"}),
        };
        json_msg.to_string()
    }
}

/// `CommandClient` is a struct that wraps an `NngIpcSocket` instance.
///
/// It provides methods to send different types of `Command` instances to the socket and receive `CommandResponse` instances.
///
/// # Fields
///
/// * `NngIpcSocket` - An instance of `NngIpcSocket` that is used to send and receive commands.
pub struct CommandClient(NngIpcSocket);

impl CommandClient {
    /// Creates a new `CommandProxy` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - A string that represents the name of the socket.
    /// * `path` - A `PathBuf` that represents the path of the socket.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandClient>` - Returns a `CronusResult` that contains a `CommandClient` instance on success or an error.
    pub fn new(name: &str, path: &Path) -> CronusResult<Self> {
        Ok(Self(NngIpcSocket::new_dial(&path.join(name))?))
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
    pub fn add_job(&self, corn: String, job: Job) -> CronusResult<CommandResponse> {
        self.cmd_request(Command::new_add_job(corn, job))
    }

    /// Sends a `ListJobs` command to the socket.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse` instance on success or an error.
    pub fn list_jobs(&self) -> CronusResult<CommandResponse> {
        self.cmd_request(Command::new_list_jobs())
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
        self.cmd_request(Command::new_delete_job(id))
    }

    /// Sends a `StopService` command to the socket.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse` instance on success or an error.
    pub fn stop_service(&self) -> CronusResult<CommandResponse> {
        self.cmd_request(Command::new_stop_service())
    }

    /// Sends a `PingService` command to the socket.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse` instance on success or an error.
    pub fn ping_service(&self) -> CronusResult<CommandResponse> {
        self.cmd_request(Command::new_ping_service())
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
    fn cmd_request(&self, cmd: Command) -> CronusResult<CommandResponse> {
        self.0.send(&cmd.to_bytes()?)?;
        let msg = self.0.recv()?;
        CommandResponse::from_bytes(&msg[..])
    }
}
