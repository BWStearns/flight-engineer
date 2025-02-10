use mavlink::common::MavMessage::*;
use mavlink::common::*;
use mavlink::{common::MavLandedState, error::MessageReadError};
use serde::{Deserialize, Serialize};
use std::{
    env,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
struct Position {
    lat: f32,
    lon: f32,
    alt: f32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
struct Attitude {
    roll: f32,
    pitch: f32,
    yaw: f32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
struct CameraAttitude {
    roll: f32,
    pitch: f32,
    yaw: f32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub struct Telemetry {
    battery: u16,
    position: Position,
    attitude: Attitude,
    camera_attitude: CameraAttitude,
    px4_mode: u8,
    heading: f32,
    airspeed: f32,
    climb_rate: f32,
    healthy: bool,
    is_armed: bool,
    is_landed: bool,
    timestamp: u64,
    time_since_boot: u64,
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn update_from_mavlink(&mut self, message: MavMessage) {
        match message {
            MavMessage::HEARTBEAT(_heartbeat) => {}
            MavMessage::SYSTEM_TIME(system_time) => {
                self.timestamp = system_time.time_unix_usec as u64;
                self.time_since_boot = system_time.time_boot_ms as u64;
            }
            MavMessage::GLOBAL_POSITION_INT(global_position_int) => {
                self.time_since_boot = global_position_int.time_boot_ms as u64;
                self.position.lat = global_position_int.lat as f32 / 1e7;
                self.position.lon = global_position_int.lon as f32 / 1e7;
                self.position.alt = global_position_int.alt as f32 / 1e3;
            }
            MavMessage::ATTITUDE(attitude) => {
                self.time_since_boot = attitude.time_boot_ms as u64;
                self.attitude.roll = attitude.roll;
                self.attitude.pitch = attitude.pitch;
                self.attitude.yaw = attitude.yaw;
            }
            MavMessage::VFR_HUD(vfr_hud) => {
                self.airspeed = vfr_hud.airspeed;
                self.climb_rate = vfr_hud.climb;
            }
            MavMessage::SYS_STATUS(_sys_status) => {
                // self.battery = sys_status.battery_remaining;
            }
            MavMessage::GPS_RAW_INT(gps_raw_int) => {
                self.heading = gps_raw_int.cog as f32;
            }
            // MavMessage::COMMAND_ACK(_command_ack) => {
            //     // if command_ack.command == MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN {
            //     //     self.px4_mode = command_ack.result;
            //     // }
            // }
            // Skip unrelated messages
            _ => {}
        }
    }
}
