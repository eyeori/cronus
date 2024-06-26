use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use chrono::Local;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, RwLock};
use tokio::try_join;
use tokio_cron_scheduler::{JobBuilder, JobScheduler};
use uuid::Uuid;

use crate::command::{Command, CommandResponse};
use crate::job::{Job, JobInfo};
use crate::nng_socket::NngIpcSocket;
use crate::CronusResult;

/// `CronusScheduler` is a struct that represents a scheduler for cron jobs.
///
/// It provides methods to parse and handle commands that are related to the management of cron jobs.
///
/// # Fields
///
/// * `cmd_parser` - A `Pin<Box<dyn Future<Output=CronusResult<()>>>>` that represents a future for parsing commands.
/// * `cmd_handler` - A `Pin<Box<dyn Future<Output=CronusResult<()>>>>` that represents a future for handling commands.
pub struct CronusScheduler {
    cmd_parser: Pin<Box<dyn Future<Output = CronusResult<()>>>>,
    cmd_handler: Pin<Box<dyn Future<Output = CronusResult<()>>>>,
}

impl CronusScheduler {
    /// Constructs a new `CronusScheduler`.
    ///
    /// This function initializes a new `JobScheduler`, starts it, and sets up command receivers.
    /// It also initializes the command parser and handler.
    ///
    /// # Arguments
    ///
    /// * `name` - A string that represents the name of the command path.
    /// * `path` - A `PathBuf` that represents the path of the command.
    ///
    /// # Returns
    ///
    /// * `CronusResult<Self>` - Returns a `CronusResult` that contains a `CronusScheduler` if successful, or an error if not.
    pub async fn new(name: String, path: PathBuf) -> CronusResult<Self> {
        // init scheduler
        let scheduler = JobScheduler::new().await?;
        scheduler.start().await?;

        // init cmd receiver
        let (cmd_sender, cmd_receiver) = mpsc::channel(1024);
        let (cmd_res_sender, cmd_res_receiver) = mpsc::channel(1024);

        // init parser and handler
        let cmd_parser = Box::pin(Self::parse_command(
            path.join(name),
            cmd_sender,
            cmd_res_receiver,
        ));
        let cmd_handler = Box::pin(Self::handle_command(
            scheduler,
            cmd_receiver,
            cmd_res_sender,
        ));

        Ok(Self {
            cmd_parser,
            cmd_handler,
        })
    }

    /// Runs the `CronusScheduler`.
    ///
    /// This function concurrently runs the command parser and handler of the `CronusScheduler`.
    /// If either the command parser or handler fails, it will return the error.
    /// If both the command parser and handler complete successfully, it will return `CommandResponse::ServiceStopped`.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse::ServiceStopped` if successful, or an error if not.
    pub async fn run(self) -> CronusResult<CommandResponse> {
        try_join!(self.cmd_parser, self.cmd_handler)?;
        Ok(CommandResponse::ServiceStopped)
    }

    /// Parses commands received from the command server.
    ///
    /// This function listens for commands from the command server, converts them from bytes to `Command` objects,
    /// and sends them to the command sender. If a `Command::StopService` command is received, it stops the service
    /// and returns. It also sends command responses back to the command server.
    ///
    /// # Arguments
    ///
    /// * `cmd_path` - A `PathBuf` that represents the path of the command server.
    /// * `cmd_sender` - A `Sender<Command>` that is used to send commands to the command handler.
    /// * `mut cmd_res_receiver` - A `Receiver<CommandResponse>` that is used to receive command responses from the command handler.
    ///
    /// # Returns
    ///
    /// * `CronusResult<()>` - Returns a `CronusResult` that contains `()` if successful, or an error if not.
    async fn parse_command(
        cmd_path: PathBuf,
        cmd_sender: Sender<Command>,
        mut cmd_res_receiver: Receiver<CommandResponse>,
    ) -> CronusResult<()> {
        let cmd_server = NngIpcSocket::new_listen(cmd_path)?;
        loop {
            let msg = cmd_server.recv()?;
            let cmd = Command::from_bytes(&msg[..])?;
            let stop_service = cmd == Command::StopService;
            cmd_sender.send(cmd).await?;
            if let Some(res) = cmd_res_receiver.recv().await {
                cmd_server.send(&res.to_bytes()?)?;
            }
            if stop_service {
                return Ok(());
            }
        }
    }

    /// Handles commands received from the command receiver.
    ///
    /// This function listens for commands from the command receiver and handles them accordingly.
    /// It maintains a loop that continues to listen for commands until the receiver is closed.
    /// The commands are handled based on their type: `AddJob`, `ListJobs`, `DeleteJob`, and `StopService`.
    /// For each command, it calls the appropriate handler function and sends the response back to the command sender.
    /// If a `Command::StopService` command is received, it stops the service and returns.
    ///
    /// # Arguments
    ///
    /// * `mut scheduler` - A mutable `JobScheduler` that is used to manage jobs.
    /// * `mut cmd_receiver` - A mutable `Receiver<Command>` that is used to receive commands.
    /// * `cmd_res_sender` - A `Sender<CommandResponse>` that is used to send command responses.
    ///
    /// # Returns
    ///
    /// * `CronusResult<()>` - Returns a `CronusResult` that contains `()` if successful, or an error if not.
    async fn handle_command(
        mut scheduler: JobScheduler,
        mut cmd_receiver: Receiver<Command>,
        cmd_res_sender: Sender<CommandResponse>,
    ) -> CronusResult<()> {
        let jobs = Arc::new(RwLock::new(HashMap::new()));
        loop {
            if let Some(cmd) = cmd_receiver.recv().await {
                let res = match cmd {
                    Command::AddJob { cron, job } => {
                        Self::handle_cmd_add_job(&scheduler, jobs.clone(), cron, job).await?
                    }
                    Command::ListJobs => {
                        Self::handle_cmd_list_job(&scheduler, jobs.clone()).await?
                    }
                    Command::DeleteJob { id } => {
                        Self::handle_cmd_delete_job(&scheduler, jobs.clone(), Uuid::parse_str(&id)?)
                            .await?
                    }
                    Command::StopService => Self::handle_cmd_stop_service(&mut scheduler).await?,
                };
                cmd_res_sender.send(res).await?;
            } else {
                return Ok(());
            }
        }
    }

    /// Handles the `AddJob` command.
    ///
    /// This function creates a new cron job and adds it to the job scheduler.
    /// It also adds the job to the jobs map.
    ///
    /// # Arguments
    ///
    /// * `scheduler` - A reference to the `JobScheduler` that is used to manage jobs.
    /// * `jobs` - An `Arc<RwLock<HashMap<Uuid, Job>>>` that is used to store jobs.
    /// * `cron` - A `String` that represents the cron schedule for the job.
    /// * `job` - A `Job` that represents the job to be added.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse::JobAdded` if successful, or an error if not.
    async fn handle_cmd_add_job(
        scheduler: &JobScheduler,
        jobs: Arc<RwLock<HashMap<Uuid, Job>>>,
        cron: String,
        job: Job,
    ) -> CronusResult<CommandResponse> {
        let business = job.clone().to_business();
        let cron_job = JobBuilder::new()
            .with_timezone(Local)
            .with_cron_job_type()
            .with_schedule(cron.as_ref())?
            .with_run_async(Box::new(move |id, mut scheduler| {
                let business = business.clone();
                Box::pin(async move {
                    if let Ok(Some(ts)) = scheduler.next_tick_for_job(id).await {
                        business(ts);
                    }
                })
            }))
            .build()?;
        let id = cron_job.guid();
        scheduler.add(cron_job).await?;
        jobs.write().await.insert(id, job);
        Ok(CommandResponse::JobAdded(id.to_string()))
    }

    /// Handles the `ListJobs` command.
    ///
    /// This function retrieves a list of all jobs from the job scheduler and the jobs map.
    /// It creates a `JobInfo` object for each job, which includes the job's ID, cron schedule, last run time, next run time, and the job itself.
    /// It then returns a `CommandResponse::JobList` that contains the list of `JobInfo` objects.
    ///
    /// # Arguments
    ///
    /// * `scheduler` - A reference to the `JobScheduler` that is used to manage jobs.
    /// * `jobs` - An `Arc<RwLock<HashMap<Uuid, Job>>>` that is used to store jobs.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse::JobList` if successful, or an error if not.
    async fn handle_cmd_list_job(
        scheduler: &JobScheduler,
        jobs: Arc<RwLock<HashMap<Uuid, Job>>>,
    ) -> CronusResult<CommandResponse> {
        let mut job_list = Vec::new();
        let jobs = jobs.read().await.clone();
        let metadata = scheduler.context().metadata_storage.clone();
        let mut metadata = metadata.write().await;
        for (id, job) in jobs {
            if let Some(job_data) = metadata.get(id).await? {
                let id = if let Some(id) = &job_data.id {
                    Uuid::from(id).to_string()
                } else {
                    Default::default()
                };
                let cron = if let Some(schedule) = job_data.schedule() {
                    String::from(schedule)
                } else {
                    Default::default()
                };
                let job = JobInfo {
                    id,
                    cron,
                    last_run: job_data.last_tick,
                    next_run: Some(job_data.next_tick),
                    job,
                };
                job_list.push(job);
            }
        }
        Ok(CommandResponse::JobList(job_list))
    }

    /// Handles the `DeleteJob` command.
    ///
    /// This function removes a job from the job scheduler and the jobs map.
    /// It uses the job's ID to find and remove the job.
    ///
    /// # Arguments
    ///
    /// * `scheduler` - A reference to the `JobScheduler` that is used to manage jobs.
    /// * `jobs` - An `Arc<RwLock<HashMap<Uuid, Job>>>` that is used to store jobs.
    /// * `id` - A `Uuid` that represents the ID of the job to be deleted.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse::JobDeleted` if successful, or an error if not.
    async fn handle_cmd_delete_job(
        scheduler: &JobScheduler,
        jobs: Arc<RwLock<HashMap<Uuid, Job>>>,
        id: Uuid,
    ) -> CronusResult<CommandResponse> {
        scheduler.remove(&id).await?;
        jobs.write().await.retain(|job_id, _| job_id.ne(&id));
        Ok(CommandResponse::JobDeleted)
    }

    /// Handles the `StopService` command.
    ///
    /// This function shuts down the job scheduler and returns a `CommandResponse::ServiceStopped`.
    ///
    /// # Arguments
    ///
    /// * `scheduler` - A mutable reference to the `JobScheduler` that is used to manage jobs.
    ///
    /// # Returns
    ///
    /// * `CronusResult<CommandResponse>` - Returns a `CronusResult` that contains a `CommandResponse::ServiceStopped` if successful, or an error if not.
    async fn handle_cmd_stop_service(
        scheduler: &mut JobScheduler,
    ) -> CronusResult<CommandResponse> {
        scheduler.shutdown().await?;
        Ok(CommandResponse::ServiceStopped)
    }
}
