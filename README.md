# musiccast-mqtt

*Base code blatantly copied from https://github.com/FruitieX/adb-mqtt. Thanks!*

*Then modified to support MusicCast API over http with the help of, for example, https://github.com/foxthefox/yamaha-yxc-nodejs/tree/master. Thanks!*

This is a rust implementation of translating a Yamaha MusicCast device into an MQTT device.
It also subscribes to an MQTT topic e.g. `home/devices/musiccast/<device_id>/set` and allows controlling the Yamaha musiccast device.

## Running

Make sure you have a recent version of Rust installed.

1. Clone this repo
2. Copy Settings.toml.example -> Settings.toml
3. Configure Settings.toml to match your setup (see below)
4. `cargo run`

## MQTT protocol

MQTT messages use the following JSON format:

```
{
  "power": false,
}
```