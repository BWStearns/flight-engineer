use anyhow::Result;
use clap::{command, Parser, Subcommand};
use flight_engineer::Vehicle;
use mavlink::common::MavMessage;
use std::time::Duration;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Connection string (e.g., serial:/dev/ttyUSB0:57600)
    connection_string: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Listen for attitude messages
    Telem,
    /// Request and display parameter list
    Params,
    /// Reboot the vehicle
    Reboot,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut vehicle = Vehicle::connect(&args.connection_string)?;

    // Set a reasonable timeout for receiving messages
    // vehicle.set_receive_timeout(Duration::from_secs(1))?;

    match args.command {
        Commands::Telem => {
            println!("Listening for attitude messages...");
            vehicle.request_stream();
            loop {
                match vehicle.receive() {
                    Ok((msg, _)) => {
                        if let MavMessage::ATTITUDE(attitude) = msg {
                            println!(
                                "Roll: {:.2}° Pitch: {:.2}° Yaw: {:.2}°",
                                attitude.roll.to_degrees(),
                                attitude.pitch.to_degrees(),
                                attitude.yaw.to_degrees()
                            );
                        }
                    }
                    Err(e) => {
                        if !e.is::<std::io::Error>()
                            || e.downcast_ref::<std::io::Error>()
                                .map_or(false, |e| e.kind() != std::io::ErrorKind::TimedOut)
                        {
                            eprintln!("Error receiving message: {}", e);
                            break;
                        }
                    }
                }
            }
        }
        Commands::Params => {
            vehicle.populate_params().await?;
            println!("Received parameters:");
            for (param_id, param_value) in vehicle.params.iter() {
                println!("{}: {}", param_id, param_value);
            }
        }
        Commands::Reboot => {
            vehicle.reboot().await?;
            println!("Reboot command sent.");
        }
    }

    Ok(())
}
