use crate::cli::Command;
use crate::command::{CommandClient, CommandResponse};
use crate::scheduler::CronusScheduler;
use anyhow::Result;
use clap::Parser;
use fork::{chdir, fork, setsid, Fork};
use serde_json::{json, Value};
use std::path::Path;
use std::process::exit;
use uuid::Uuid;

mod cli;
mod command;
mod job;
mod nng_socket;
mod scheduler;

fn main() {
    match run() {
        Ok(Some(data)) => println!("{data}"),
        Ok(None) => {}
        Err(e) => {
            println!("{}", json!({"error": e.to_string()}));
            exit(-1)
        }
    }
}

/// Asynchronously runs the Cronus task execution manager.
///
/// This function matches the command line arguments to the corresponding command variant and executes the command.
/// It handles all the commands that the Cronus task execution manager can process, including starting and stopping the service,
/// adding and deleting jobs, listing jobs, and running the service.
///
/// # Returns
///
/// * `Result<Option<Value>>` - The result of running the command.
/// If the command is executed successfully, it returns a `Result::Ok(Option<Value>)`
/// where the `Option<Value>` is a JSON value that represents the result of the command.
/// If there is an error executing the command, it returns a `CronusResult::Err(CronusError)`
/// where the `CronusError` represents the error that occurred.
#[tokio::main]
async fn run() -> Result<Option<Value>> {
    let response = match Command::parse() {
        Command::Start { name, path } => {
            if !is_service_running(&name, &path)? {
                run_service(&name, &path)?;
            }
            CommandResponse::ServiceRunning
        }
        Command::Stop { name, path } => match CommandClient::conn(&name, &path) {
            Ok(cc) => cc.stop_service()?,
            Err(_) => CommandResponse::ServiceNotRunning,
        },
        Command::Add {
            name,
            path,
            cron,
            cmd,
        } => match CommandClient::conn(&name, &path) {
            Ok(cc) => cc.add_job(cron, cmd.into_job())?,
            Err(_) => CommandResponse::ServiceNotRunning,
        },
        Command::Delete { name, path, id } => match CommandClient::conn(&name, &path) {
            Ok(cc) => {
                Uuid::parse_str(&id).map_err(anyhow::Error::from)?;
                cc.delete_job(id)?
            }
            Err(_) => CommandResponse::ServiceNotRunning,
        },
        Command::List { name, path } => match CommandClient::conn(&name, &path) {
            Ok(cc) => cc.list_jobs()?,
            Err(_) => CommandResponse::ServiceNotRunning,
        },
        Command::Run { name, path } => {
            let scheduler = CronusScheduler::new(name, &path).await?;
            scheduler.run().await?
        }
        Command::Status { name, path } => match CommandClient::conn(&name, &path) {
            Ok(cc) => cc.ping_service()?,
            Err(_) => CommandResponse::ServiceNotRunning,
        },
    };
    Ok(response.to_json_msg())
}

/// Checks if the Cronus service is running.
///
/// This function sends a ping to the Cronus service and checks the response to determine if the service is running.
///
/// # Arguments
///
/// * `name` - The name of the Cronus service.
/// * `path` - The path where the Cronus service is located.
///
/// # Returns
///
/// * `Result<bool>` - Returns `Ok(true)` if the service is running,
///   `Ok(false)` if the service is not running,
///   and `Err(CronusError)` if there was an error checking the service status.
fn is_service_running(name: &str, path: &Path) -> Result<bool> {
    if let Ok(cc) = CommandClient::conn(name, path)
        && let Ok(CommandResponse::ServiceRunning) = cc.ping_service()
    {
        return Ok(true);
    }
    Ok(false)
}

/// Starts a Cronus service.
///
/// This function starts a new instance of the Cronus service in a new process.
///
/// # Arguments
///
/// * `name` - The name of the Cronus service.
/// * `path` - The path where the Cronus service is located.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the service is started successfully,
///   and `Err(CronusError)` if there was an error starting the service.
fn run_service(name: &str, path: &Path) -> Result<()> {
    let cronus = std::env::current_exe()?;
    let daemon = match fork() {
        Ok(Fork::Parent(_)) => return Ok(()),
        Ok(Fork::Child) => setsid().and_then(|_| {
            chdir()?;
            fork()
        }),
        Err(n) => Err(n),
    };
    match daemon {
        Ok(Fork::Child) => {
            std::process::Command::new(cronus)
                .arg("run")
                .arg("--name")
                .arg(name)
                .arg("--path")
                .arg(path)
                .spawn()?;
            exit(0);
        }
        _ => exit(0),
    }
}
