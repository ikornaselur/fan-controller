use anyhow::{bail, Result};
use log::{debug, error, info, warn};
use rumqttc::{Client, Event, LastWill, MqttOptions, Packet, QoS};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::pid::{PidController, PidOutput};

const DISCOVERY_PREFIX: &str = "homeassistant";

pub struct MqttConfig {
    pub broker: String,
    pub port: u16,
    pub prefix: String,
    pub hostname: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub struct MqttHandle {
    client: Client,
    connection: rumqttc::Connection,
    prefix: String,
    hostname: String,
}

impl MqttHandle {
    pub fn new(config: MqttConfig) -> Result<Self> {
        let status_topic = format!("{}/status", config.prefix);

        let client_id = format!("fan-controller-{}", config.hostname);
        let mut opts = MqttOptions::new(&client_id, &config.broker, config.port);
        opts.set_keep_alive(Duration::from_secs(30));
        opts.set_last_will(LastWill::new(
            &status_topic,
            "offline",
            QoS::AtLeastOnce,
            true,
        ));

        if let (Some(user), Some(pass)) = (&config.username, &config.password) {
            opts.set_credentials(user, pass);
        }

        let (client, mut connection) = Client::new(opts, 64);

        // Drive the connection until we get a ConnAck
        info!(
            "Connecting to MQTT broker {}:{}",
            config.broker, config.port
        );
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let mut connected = false;

        while std::time::Instant::now() < deadline {
            match connection.recv_timeout(Duration::from_secs(1)) {
                Ok(Ok(Event::Incoming(Packet::ConnAck(ack)))) => {
                    info!("MQTT connected: {:?}", ack);
                    connected = true;
                    break;
                }
                Ok(Ok(event)) => {
                    debug!("MQTT event during connect: {:?}", event);
                }
                Ok(Err(e)) => {
                    bail!("MQTT connection failed: {}", e);
                }
                Err(_) => continue,
            }
        }

        if !connected {
            bail!("MQTT connection timed out after 10s");
        }

        Ok(Self {
            client,
            connection,
            prefix: config.prefix,
            hostname: config.hostname,
        })
    }

    fn device_json(&self) -> serde_json::Value {
        json!({
            "identifiers": [&self.prefix],
            "name": format!("{} fan controller", self.hostname),
            "manufacturer": "absalon.dev",
            "model": "Raspberry Pi PWM Fan Controller",
            "sw_version": env!("CARGO_PKG_VERSION"),
        })
    }

    /// Publish HA discovery configs and mark as online.
    pub fn publish_discovery(&mut self, pid: &PidController) -> Result<()> {
        let device = self.device_json();
        let status_topic = format!("{}/status", self.prefix);
        let p = &self.prefix;

        // Temperature sensor
        self.publish_retained(
            &format!("{}/sensor/{}/temperature/config", DISCOVERY_PREFIX, p),
            &json!({
                "name": "Temperature",
                "unique_id": format!("{}_temperature", p),
                "device": device,
                "state_topic": format!("{}/temperature/state", p),
                "unit_of_measurement": "\u{00b0}C",
                "device_class": "temperature",
                "state_class": "measurement",
                "availability_topic": status_topic,
            })
            .to_string(),
        )?;

        // Duty cycle sensor
        self.publish_retained(
            &format!("{}/sensor/{}/duty_cycle/config", DISCOVERY_PREFIX, p),
            &json!({
                "name": "Duty Cycle",
                "unique_id": format!("{}_duty_cycle", p),
                "device": device,
                "state_topic": format!("{}/duty_cycle/state", p),
                "unit_of_measurement": "%",
                "state_class": "measurement",
                "icon": "mdi:fan",
                "availability_topic": status_topic,
            })
            .to_string(),
        )?;

        // PID error sensor
        self.publish_retained(
            &format!("{}/sensor/{}/error/config", DISCOVERY_PREFIX, p),
            &json!({
                "name": "PID Error",
                "unique_id": format!("{}_error", p),
                "device": device,
                "state_topic": format!("{}/error/state", p),
                "unit_of_measurement": "\u{00b0}C",
                "state_class": "measurement",
                "icon": "mdi:delta",
                "availability_topic": status_topic,
            })
            .to_string(),
        )?;

        // PID integral sensor
        self.publish_retained(
            &format!("{}/sensor/{}/integral/config", DISCOVERY_PREFIX, p),
            &json!({
                "name": "PID Integral",
                "unique_id": format!("{}_integral", p),
                "device": device,
                "state_topic": format!("{}/integral/state", p),
                "state_class": "measurement",
                "icon": "mdi:sigma",
                "availability_topic": status_topic,
            })
            .to_string(),
        )?;

        // PID derivative sensor
        self.publish_retained(
            &format!("{}/sensor/{}/derivative/config", DISCOVERY_PREFIX, p),
            &json!({
                "name": "PID Derivative",
                "unique_id": format!("{}_derivative", p),
                "device": device,
                "state_topic": format!("{}/derivative/state", p),
                "state_class": "measurement",
                "icon": "mdi:chart-line-variant",
                "availability_topic": status_topic,
            })
            .to_string(),
        )?;

        // Target temp number
        self.publish_retained(
            &format!("{}/number/{}/target_temp/config", DISCOVERY_PREFIX, p),
            &json!({
                "name": "Target Temperature",
                "unique_id": format!("{}_target_temp", p),
                "device": device,
                "state_topic": format!("{}/target_temp/state", p),
                "command_topic": format!("{}/target_temp/set", p),
                "min": 30,
                "max": 70,
                "step": 0.5,
                "mode": "box",
                "unit_of_measurement": "\u{00b0}C",
                "device_class": "temperature",
                "availability_topic": status_topic,
            })
            .to_string(),
        )?;

        // Kp number
        self.publish_retained(
            &format!("{}/number/{}/kp/config", DISCOVERY_PREFIX, p),
            &json!({
                "name": "Kp (Proportional)",
                "unique_id": format!("{}_kp", p),
                "device": device,
                "state_topic": format!("{}/kp/state", p),
                "command_topic": format!("{}/kp/set", p),
                "min": 0,
                "max": 1,
                "step": 0.001,
                "mode": "box",
                "icon": "mdi:tune",
                "availability_topic": status_topic,
            })
            .to_string(),
        )?;

        // Ki number
        self.publish_retained(
            &format!("{}/number/{}/ki/config", DISCOVERY_PREFIX, p),
            &json!({
                "name": "Ki (Integral)",
                "unique_id": format!("{}_ki", p),
                "device": device,
                "state_topic": format!("{}/ki/state", p),
                "command_topic": format!("{}/ki/set", p),
                "min": 0,
                "max": 0.1,
                "step": 0.0001,
                "mode": "box",
                "icon": "mdi:tune",
                "availability_topic": status_topic,
            })
            .to_string(),
        )?;

        // Kd number
        self.publish_retained(
            &format!("{}/number/{}/kd/config", DISCOVERY_PREFIX, p),
            &json!({
                "name": "Kd (Derivative)",
                "unique_id": format!("{}_kd", p),
                "device": device,
                "state_topic": format!("{}/kd/state", p),
                "command_topic": format!("{}/kd/set", p),
                "min": 0,
                "max": 1,
                "step": 0.001,
                "mode": "box",
                "icon": "mdi:tune",
                "availability_topic": status_topic,
            })
            .to_string(),
        )?;

        // Subscribe to command topics
        self.client
            .subscribe(format!("{}/+/set", self.prefix), QoS::AtLeastOnce)?;

        // Publish online status
        self.publish_retained(&status_topic, "online")?;

        // Publish current PID state so HA has initial values
        self.publish_pid_state(pid)?;

        // Drain events to flush the queued publishes to the broker
        self.drain_pending();

        info!("MQTT discovery published, subscribed to commands");
        Ok(())
    }

    /// Publish current sensor values.
    pub fn publish_state(&self, temp: f32, output: &PidOutput) -> Result<()> {
        self.client.publish(
            format!("{}/temperature/state", self.prefix),
            QoS::AtMostOnce,
            false,
            format!("{:.1}", temp),
        )?;
        self.client.publish(
            format!("{}/duty_cycle/state", self.prefix),
            QoS::AtMostOnce,
            false,
            format!("{:.1}", output.duty_cycle * 100.0),
        )?;
        self.client.publish(
            format!("{}/error/state", self.prefix),
            QoS::AtMostOnce,
            false,
            format!("{:.2}", output.error),
        )?;
        self.client.publish(
            format!("{}/integral/state", self.prefix),
            QoS::AtMostOnce,
            false,
            format!("{:.2}", output.integral),
        )?;
        self.client.publish(
            format!("{}/derivative/state", self.prefix),
            QoS::AtMostOnce,
            false,
            format!("{:.3}", output.derivative),
        )?;
        Ok(())
    }

    /// Publish current PID config values (for HA number entity states).
    pub fn publish_pid_state(&self, pid: &PidController) -> Result<()> {
        self.client.publish(
            format!("{}/target_temp/state", self.prefix),
            QoS::AtMostOnce,
            true,
            format!("{:.1}", pid.target()),
        )?;
        self.client.publish(
            format!("{}/kp/state", self.prefix),
            QoS::AtMostOnce,
            true,
            format!("{}", pid.kp()),
        )?;
        self.client.publish(
            format!("{}/ki/state", self.prefix),
            QoS::AtMostOnce,
            true,
            format!("{}", pid.ki()),
        )?;
        self.client.publish(
            format!("{}/kd/state", self.prefix),
            QoS::AtMostOnce,
            true,
            format!("{}", pid.kd()),
        )?;
        Ok(())
    }

    /// Process pending MQTT events, handling any incoming commands.
    /// Returns after `timeout` has elapsed, or early if `running` becomes false.
    pub fn poll(&mut self, timeout: Duration, pid: &mut PidController, running: &Arc<AtomicBool>) {
        let deadline = std::time::Instant::now() + timeout;

        loop {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }

            let recv_timeout = remaining.min(Duration::from_millis(100));
            match self.connection.recv_timeout(recv_timeout) {
                Ok(Ok(event)) => self.handle_event(event, pid),
                Ok(Err(e)) => {
                    error!("MQTT connection error: {}", e);
                    break;
                }
                Err(_timeout) => continue,
            }
        }
    }

    /// Disconnect and drain the connection so the process can exit.
    pub fn shutdown(&mut self) {
        let _ = self.client.disconnect();
        self.drain_pending();
    }

    /// Drain pending events from the connection (up to 2 seconds).
    fn drain_pending(&mut self) {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match self
                .connection
                .recv_timeout(remaining.min(Duration::from_millis(100)))
            {
                Ok(Ok(event)) => {
                    debug!("MQTT drain: {:?}", event);
                }
                Ok(Err(e)) => {
                    debug!("MQTT drain error: {}", e);
                    break;
                }
                Err(_) => break,
            }
        }
    }

    fn handle_event(&self, event: Event, pid: &mut PidController) {
        if let Event::Incoming(Packet::Publish(publish)) = event {
            let topic = &publish.topic;
            let payload = match std::str::from_utf8(&publish.payload) {
                Ok(s) => s,
                Err(_) => return,
            };

            debug!("MQTT command: {} = {}", topic, payload);

            let suffix = match topic.strip_prefix(&format!("{}/", self.prefix)) {
                Some(s) => s,
                None => return,
            };

            match suffix {
                "target_temp/set" => {
                    if let Ok(val) = payload.parse::<f32>() {
                        info!("MQTT: setting target temp to {:.1}\u{00b0}C", val);
                        pid.set_target(val);
                        let _ = self.client.publish(
                            format!("{}/target_temp/state", self.prefix),
                            QoS::AtMostOnce,
                            true,
                            format!("{:.1}", val),
                        );
                    } else {
                        warn!("MQTT: invalid target_temp value: {}", payload);
                    }
                }
                "kp/set" => {
                    if let Ok(val) = payload.parse::<f32>() {
                        info!("MQTT: setting Kp to {}", val);
                        pid.set_kp(val);
                        let _ = self.client.publish(
                            format!("{}/kp/state", self.prefix),
                            QoS::AtMostOnce,
                            true,
                            format!("{}", val),
                        );
                    }
                }
                "ki/set" => {
                    if let Ok(val) = payload.parse::<f32>() {
                        info!("MQTT: setting Ki to {}", val);
                        pid.set_ki(val);
                        let _ = self.client.publish(
                            format!("{}/ki/state", self.prefix),
                            QoS::AtMostOnce,
                            true,
                            format!("{}", val),
                        );
                    }
                }
                "kd/set" => {
                    if let Ok(val) = payload.parse::<f32>() {
                        info!("MQTT: setting Kd to {}", val);
                        pid.set_kd(val);
                        let _ = self.client.publish(
                            format!("{}/kd/state", self.prefix),
                            QoS::AtMostOnce,
                            true,
                            format!("{}", val),
                        );
                    }
                }
                _ => {
                    debug!("MQTT: unknown command topic: {}", topic);
                }
            }
        }
    }

    fn publish_retained(&self, topic: &str, payload: &str) -> Result<()> {
        self.client
            .publish(topic, QoS::AtLeastOnce, true, payload)?;
        Ok(())
    }
}
