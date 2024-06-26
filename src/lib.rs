pub mod command;
pub mod job;
mod nng_socket;
pub mod scheduler;

pub type CronusResult<T> = Result<T, Box<dyn std::error::Error>>;
