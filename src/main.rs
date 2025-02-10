use anyhow::Result;
use clap::{command, Parser, Subcommand};
use flight_engineer::Vehicle;
use mavlink::common::MavMessage;
use rust_socketio::{
    asynchronous::{Client, ClientBuilder},
    Payload, RawClient,
};
use serde_json::json;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tungstenite;
use tungstenite::{client::IntoClientRequest, connect, Message};
use url::Url;

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "\n
This is a command line interface for the flight engineer library.

You can call it like `cargo run serial:/dev/tty.usbmodem01:57600 telem` where the first argument
is the connection string and the second argument is the subcommand. The connection string is how
to connect to the vehicle, which may vary depending whether you're talking via serial, usb, radio,
etc.
"
)]
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

// #[derive(Debug, Serialize, Deserialize)]
// struct TelemUpdate {}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut vehicle = Vehicle::connect(&args.connection_string)?;
    let telem_state = Arc::new(Mutex::new(vehicle.telem));

    // Make a tokio task that sends telemetry messages to the server every 100ms
    // Send vehicle.telem to the server every 100ms as JSON using a websocket

    let telem = Arc::clone(&telem_state);
    tokio::spawn(async move {
        println!("Connecting to websocket...");
        let socket = ClientBuilder::new("http://localhost:5150/")
            .namespace("/")
            .connect()
            .await
            .expect("Failed to connect to websocket");

        println!("Connected to websocket!");
        loop {
            println!("Sending telemetry!");
            let json = json!(&*telem.lock().unwrap());
            socket
                .emit("update_state", Payload::Text(vec![json]))
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("Sent telemetry");
        }
    });

    match args.command {
        Commands::Telem => {
            println!("Listening for telemetry messages...");
            let _ = vehicle.request_stream();
            loop {
                match vehicle.receive() {
                    Ok((msg, _)) => {
                        telem_state.lock().unwrap().update_from_mavlink(msg);
                        // dbg!(&telem_state);
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
                println!("{}: {:#?}", param_id, param_value);
            }
        }
        Commands::Reboot => {
            vehicle.reboot().await?;
            println!("Reboot command sent.");
        }
    }

    Ok(())
}
