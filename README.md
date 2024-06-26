# Cronus Task Execution Manager

Cronus is a task execution manager written in Rust. It allows you to schedule tasks using cron expressions and execute
them at the specified times.

## Features

- Schedule tasks using cron expressions
- Add, delete, and list jobs
- Start and stop the Cronus service

## Installation

Make sure you have Rust and Cargo installed on your system. Then, clone this repository and build the project using
Cargo.

```bash
git clone https://github.com/eyeori/cronus.git
cd cronus
cargo build --release
```

The executable will be located in the target/release directory.

## Usage

Here's how you can use the different commands of the Cronus task execution manager:

- Start the service: ```./cronus start```
- Stop the service: ```./cronus stop```
- Add a job: ```./cronus add -c "<cron>" <sub_command> <cmd_args>```
- Delete a job: ```./cronus delete -i "<job_id>"```
- List jobs: ```./cronus list```

Replace ```<cron>``` with the cron expression for the schedule, ```<sub_command>``` and ```<cmd_args>``` with the
command you want to execute, and ```<job_id>``` with the id of the job you want to delete.

## Contributing

Contributions are welcome! Please feel free to submit a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
