use eyre::{eyre, Result};
use rumqttc::QoS;
use serde::{Deserialize, Serialize};
use core::fmt;
use std::time::Duration;

use crate::{
    app_config::MusicCastConfig,
    mqtt::{MqttClient, MqttDevice},
};

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct MusicCastState {
    pub power: MusicCastPowerString,
    pub sleep: u16,
    pub volume: u16,
    pub mute: bool,
    pub max_volume: u16,
    pub input: String,
    pub subwoofer_volume: u16,
    pub actual_volume: ActualVolume,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct ActualVolume {
    pub mode: String,
    pub value: f32,
    pub unit: String,
}

#[derive(Debug)]
enum MusicCastCmd {
    Power(bool),
    Volume(u16),
    Mute(bool),
    Input(MusicCastInputString),
    SubwooferVolume(u16),
    GetStatus,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum MusicCastInputString {
    Optical,
    Aux,
    Bluetooth,
}

impl fmt::Display for MusicCastInputString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MusicCastInputString::Optical => write!(f, "optical"),
            MusicCastInputString::Aux => write!(f, "aux"),
            MusicCastInputString::Bluetooth => write!(f, "bluetooth"),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum MusicCastPowerString {
    on,
    standby,
    off,
}

impl fmt::Display for MusicCastPowerString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MusicCastPowerString::on => write!(f, "on"),
            MusicCastPowerString::standby => write!(f, "standby"),
            MusicCastPowerString::off => write!(f, "off"),
        }
    }
}

async fn get_state(config: MusicCastConfig) -> Result<MusicCastState> {
    let url = format!("http://{}/YamahaExtendedControl/v1/main", config.ip);
    println!("url: {}", url);
    let result: MusicCastState = surf::get(format!("{}/getStatus", url)).recv_json().await.map_err(|e| eyre!(e))?;
    Ok(result)
}

async fn send_command(config: MusicCastConfig, cmd: MusicCastCmd) -> Result<()> {
    let url = format!("http://{}/YamahaExtendedControl/v1/main", config.ip);
    println!("url: {}, command: {:?}", url, cmd);
    let _result = match cmd {
        MusicCastCmd::GetStatus => Err(eyre!("Use get_state instead")),
        MusicCastCmd::Power(state) => {
            let state = if state { "on" } else { "standby" };
            let final_url = format!("{}/setPower?power={}", url, state);
            println!("final_url: {}", final_url);
            surf::get(final_url).recv_json().await.map_err(|e| eyre!(e))?;
            Ok(())
        },
        MusicCastCmd::Mute(state) => {
            let state = if state { "true" } else { "false" };
            surf::get(format!("{}/setMute?enable={}", url, state)).recv_json().await.map_err(|e| eyre!(e))?;
            Ok(())
        },
        MusicCastCmd::Volume(volume) => {
            surf::get(format!("{}/setVolume?volume={}", url, volume)).recv_json().await.map_err(|e| eyre!(e))?;
            Ok(())
        },
        MusicCastCmd::Input(input) => {
            surf::get(format!("{}/setInput?input={}", url, input)).recv_json().await.map_err(|e| eyre!(e))?;
            Ok(())
        },
        MusicCastCmd::SubwooferVolume(volume) => {
            surf::get(format!("{}/setSubwooferVolume?volume={}", url, volume)).recv_json().await.map_err(|e| eyre!(e))?;
            Ok(())
        },
    };
    println!("result: {:?}", _result);
    Ok(())
}

pub fn init(mut mqtt_client: MqttClient, musiccast_config: MusicCastConfig) {
    // create tokio task that will update the device state to mqtt when it changes
    let cfg = musiccast_config.clone();
    tokio::spawn(async move {
        loop {
            let device = get_state(cfg.clone()).await.unwrap();
            
            let topic = mqtt_client.topic.clone().replace('+', cfg.name.as_str());
            
            mqtt_client
            .client
            .publish(
                topic,
                QoS::AtMostOnce,
                false,
                serde_json::to_vec(&device).unwrap(),
            )
            .await
            .unwrap();
        
        tokio::time::sleep(Duration::from_secs(cfg.poll_rate.unwrap_or(5))).await;
        }
    });

    tokio::spawn(async move {
        loop {
            mqtt_client
                .rx
                .changed()
                .await
                .expect("Expected rx channel never to close");
            let device = mqtt_client.rx.borrow().clone();

            println!("Received MQTT update! Device: {:?}", device);

            if let Some(d) = device {
                let _ = match Some(d) {
                    Some(MqttDevice { power: Some(power), .. }) => {
                        send_command(musiccast_config.clone(), MusicCastCmd::Power(power)).await
                    },
                    Some(MqttDevice { volume: Some(volume), .. }) => {
                        send_command(musiccast_config.clone(), MusicCastCmd::Volume(volume)).await
                    },
                    Some(MqttDevice { mute: Some(mute), .. }) => {
                        send_command(musiccast_config.clone(), MusicCastCmd::Mute(mute)).await
                    },
                    // Some(MqttDevice { input: Some(input), .. }) => {
                    //     send_command(musiccast_config.clone(), MusicCastCmd::Input(input)).await
                    // },
                    _ => Ok(())
                };
            };
        }
    });
}
