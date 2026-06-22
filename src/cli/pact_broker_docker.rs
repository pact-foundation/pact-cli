use clap::{ArgMatches, Command};
use std::process::{Command as Cmd, ExitCode};

pub fn add_docker_broker_subcommand() -> Command {
    Command::new("docker")
        .about("Run the Pact Broker as a Docker container")
        .subcommand(Command::new("start").about("Start the Pact Broker as a Docker container"))
        .subcommand(Command::new("stop").about("Stop the Pact Broker Docker container"))
        .subcommand(Command::new("remove").about("Remove the Pact Broker Docker container"))
}
pub fn run(args: &ArgMatches) -> Result<(), ExitCode> {
    match args.subcommand() {
        Some(("start", _args)) => {
            let command_args = vec![
                "run",
                "-d",
                "--name",
                "pact-broker",
                "-p",
                "9292:9292",
                "--env",
                "PACT_BROKER_PORT=9292",
                "--env",
                "PACT_BROKER_DATABASE_URL=sqlite:////tmp/pact_broker.sqlite",
                "--env",
                "'PACT_BROKER_BASE_URL=http://localhost http://localhost http://localhost:9292 http://pact-broker:9292 https://host.docker.internal http://host.docker.internal http://host.docker.internal:9292'",
                "pactfoundation/pact-broker:latest",
            ];

            println!(
                "Starting Pact Broker Docker container with command: docker {}",
                command_args.join(" ")
            );

            let output = Cmd::new("docker")
                .args(&command_args)
                .output()
                .expect("Failed to execute Docker command");

            if output.status.success() {
                println!("Docker container started successfully");
                Ok(())
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr);
                println!("Failed to start Docker container: {}", error_message);
                Err(ExitCode::from(output.status.code().unwrap_or(1) as u8))
            }
        }
        Some(("stop", _args)) => {
            let output = Cmd::new("docker")
                .arg("stop")
                .arg("pact-broker")
                .output()
                .expect("Failed to execute Docker command");

            if output.status.success() {
                println!("Docker container stopped successfully");
                Ok(())
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr);
                println!("Failed to stop Docker container: {}", error_message);
                Err(ExitCode::from(1))
            }
        }
        Some(("remove", _args)) => {
            let output = Cmd::new("docker")
                .arg("rm")
                .arg("pact-broker")
                .output()
                .expect("Failed to execute Docker command");

            if output.status.success() {
                println!("Docker container removed successfully");
                Ok(())
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr);
                println!("Failed to remove Docker container: {}", error_message);
                Err(ExitCode::from(1))
            }
        }
        _ => {
            println!("⚠️  No option provided, try running docker --help");

            Ok(())
        }
    }
}
