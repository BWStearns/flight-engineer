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
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
