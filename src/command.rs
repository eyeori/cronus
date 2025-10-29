use crate::job::{Job, JobInfo};
use crate::nng_socket::NngIpcSocket;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

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
    /// Converts the `Command` instance into a byte vector.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<u8>>` - Returns a `Result` that contains a byte vector on success or an error.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
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
    /// * `Result<Command>` - Returns a `Result` that contains a `Command` instance on success or an error.
    pub fn from_bytes(cmd: &[u8]) -> Result<Self> {
        serde_json::from_slice::<Self>(cmd).map_err(Into::into)
    }
}

/// `CommandResponse` is an enumeration that represents the different types of responses that can be returned by commands.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CommandResponse {
    JobAdded(String),
    JobList(Vec<JobInfo>),
    JobDeleted,
    ServiceRunning,
    ServiceStopping,
    ServiceStopped,
    ServiceNotRunning,
    Nothing,
}

impl CommandResponse {
    /// Converts the `CommandResponse` instance into a byte vector.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<u8>>` - Returns a `Result` that contains a byte vector on success or an error.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
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
    /// * `Result<CommandResponse>` - Returns a `Result` that contains a `CommandResponse` instance on success or an error.
    pub fn from_bytes(cmd: &[u8]) -> Result<Self> {
        serde_json::from_slice::<Self>(cmd).map_err(Into::into)
    }

    /// Converts the `CommandResponse` instance into a JSON message.
    ///
    /// This function serializes the `CommandResponse` instance into a JSON value. The structure of the JSON string depends on the variant of the `CommandResponse` instance.
    ///
    /// # Returns
    ///
    /// * `Option<Value>` - Returns a JSON value that represents the `CommandResponse` instance.
    pub fn to_json_msg(&self) -> Option<Value> {
        match self {
            Self::JobAdded(id) => Some(json!({"job_id": id})),
            Self::JobList(jobs) => Some(json!(jobs)),
            Self::JobDeleted => Some(json!({"message": "Job deleted"})),
            Self::ServiceRunning => Some(json!({"message": "Service running"})),
            Self::ServiceStopping => Some(json!({"message": "Service stopping"})),
            Self::ServiceStopped => Some(json!({"message": "Service stopped"})),
            Self::ServiceNotRunning => Some(json!({"message": "Service not running"})),
            Self::Nothing => None,
        }
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
    /// * `path` - A `Path` that represents the path of the socket.
    ///
    /// # Returns
    ///
    /// * `Result<CommandClient>` - Returns a `Result` that contains a `CommandClient` instance on success or an error.
    pub fn conn(name: &str, path: &Path) -> Result<Self> {
        Ok(Self(NngIpcSocket::new_dial(&path.join(name))?))
    }

    /// Sends an `AddJob` command to the socket.
    ///
    /// # Arguments
    ///
    /// * `cron` - A cron string that represents the schedule of the job.
    /// * `job` - A `Job` instance that represents the job to be added.
    ///
    /// # Returns
    ///
    /// * `Result<CommandResponse>` - Returns a `Result` that contains a `CommandResponse` instance on success or an error.
    pub fn add_job(&self, cron: String, job: Job) -> Result<CommandResponse> {
        self.cmd_request(Command::AddJob { cron, job })
    }

    /// Sends a `ListJobs` command to the socket.
    ///
    /// # Returns
    ///
    /// * `Result<CommandResponse>` - Returns a `Result` that contains a `CommandResponse` instance on success or an error.
    pub fn list_jobs(&self) -> Result<CommandResponse> {
        self.cmd_request(Command::ListJobs)
    }

    /// Sends a `DeleteJob` command to the socket.
    ///
    /// # Arguments
    ///
    /// * `id` - A string that represents the id of the job to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<CommandResponse>` - Returns a `Result` that contains a `CommandResponse` instance on success or an error.
    pub fn delete_job(&self, id: String) -> Result<CommandResponse> {
        self.cmd_request(Command::DeleteJob { id })
    }

    /// Sends a `StopService` command to the socket.
    ///
    /// # Returns
    ///
    /// * `Result<CommandResponse>` - Returns a `Result` that contains a `CommandResponse` instance on success or an error.
    pub fn stop_service(&self) -> Result<CommandResponse> {
        self.cmd_request(Command::StopService)
    }

    /// Sends a `PingService` command to the socket.
    ///
    /// # Returns
    ///
    /// * `Result<CommandResponse>` - Returns a `Result` that contains a `CommandResponse` instance on success or an error.
    pub fn ping_service(&self) -> Result<CommandResponse> {
        self.cmd_request(Command::PingService)
    }

    /// Sends a `Command` instance to the socket and receives a `CommandResponse` instance.
    ///
    /// # Arguments
    ///
    /// * `cmd` - A `Command` instance that represents the command to be sent.
    ///
    /// # Returns
    ///
    /// * `Result<CommandResponse>` - Returns a `Result` that contains a `CommandResponse` instance on success or an error.
    fn cmd_request(&self, cmd: Command) -> Result<CommandResponse> {
        self.0.send(&cmd.to_bytes()?)?;
        let msg = self.0.recv()?;
        CommandResponse::from_bytes(&msg[..])
    }
}
