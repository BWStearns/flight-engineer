use anyhow::{Context, Ok, Result};
use mavlink::common::MavCmd::*;
use mavlink::common::MavMessage::*;
use mavlink::{common::*, MavConnection};
use std::time::Duration;

pub struct Vehicle {
    pub connection: Box<dyn MavConnection<MavMessage>>,
    pub params: std::collections::HashMap<String, f32>,
    pub receive_timeout: Duration,
}

impl Vehicle {
    pub fn new(connection: Box<dyn MavConnection<MavMessage>>) -> Self {
        Self {
            connection,
            params: std::collections::HashMap::new(),
            receive_timeout: Duration::from_secs(1),
        }
    }

    pub fn connect(connection_string: &str) -> Result<Self> {
        let connection = mavlink::connect(connection_string)?;
        Ok(Self::new(connection))
    }

    pub fn receive(&mut self) -> Result<(MavMessage, u64)> {
        let (header, message) = self.connection.recv()?;
        Ok((message, header.sequence as u64))
    }
    pub fn request_stream(&mut self) -> Result<()> {
        let stream_command = REQUEST_DATA_STREAM(mavlink::common::REQUEST_DATA_STREAM_DATA {
            target_system: 0,
            target_component: 0,
            req_stream_id: 0,
            req_message_rate: 10,
            start_stop: 1,
        });
        self.connection
            .send(&mavlink::MavHeader::default(), &stream_command)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////
    /// Reboot
    ////////////////////////////////////////////////////////

    pub async fn reboot(&mut self) -> Result<()> {
        let reboot_command = MavMessage::COMMAND_LONG(COMMAND_LONG_DATA {
            target_system: 1,
            target_component: 1,
            command: MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN,
            confirmation: 0,
            param1: 1.0, // 1 = reboot autopilot
            param2: 0.0, // 0 = do not reboot companion computer
            param3: 0.0,
            param4: 0.0,
            param5: 0.0,
            param6: 0.0,
            param7: 0.0,
        });
        self.connection
            .send(&mavlink::MavHeader::default(), &reboot_command)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////
    /// PARAMETERS
    ////////////////////////////////////////////////////////
    pub async fn request_parameters(&mut self) -> Result<()> {
        let param_request = MavMessage::PARAM_REQUEST_LIST(PARAM_REQUEST_LIST_DATA {
            target_system: 1,
            target_component: 1,
        });
        self.connection
            .send(&mavlink::MavHeader::default(), &param_request)?;
        Ok(())
    }

    pub async fn populate_params(&mut self) -> Result<()> {
        println!("Requesting parameter list...");
        self.request_parameters().await?;

        // Create a map to store parameters
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(10);

        while start_time.elapsed() < timeout {
            match self.receive() {
                std::result::Result::Ok((msg, _)) => {
                    if let MavMessage::PARAM_VALUE(param) = msg {
                        let param_id = String::from_utf8_lossy(&param.param_id)
                            .trim_matches('\0')
                            .to_string();
                        self.params.insert(param_id.clone(), param.param_value);

                        // Check if we've received all parameters
                        if self.params.len() as u16 == param.param_count {
                            println!("{:#?}", &param);
                            println!("Received all {} parameters", self.params.len());
                            break;
                        }
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
        if start_time.elapsed() >= timeout {
            println!(
                "Timeout waiting for parameters. Received {} parameters.",
                self.params.len()
            );
            return Err(anyhow::anyhow!("Timeout waiting for parameters"));
        } else {
            Ok(())
        }
    }

    // pub fn set_receive_timeout(&mut self, duration: Duration) -> Result<()> {
    //     self.connection.set_receive_timeout(duration)?;
    //     Ok(())
    // }
}
