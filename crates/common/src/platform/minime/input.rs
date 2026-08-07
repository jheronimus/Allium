use std::time::Duration;

use anyhow::{Context, Result};
use evdev::{Device, EventStream, EventType};
use futures::future::select_all;

use crate::platform::minime::traits::Traits;
use crate::platform::{Key, KeyEvent};

/// Polls all Minime input devices (gamepad, power, volume) and maps evdev
/// keycodes to UI `Key` events using the traits' keycode table.
pub struct MinimeInput {
    streams: Vec<EventStream>,
    keycodes: Vec<(u16, Key)>,
}

impl MinimeInput {
    pub fn new(traits: &Traits) -> Result<Self> {
        let names = [
            traits.input_gamepad_device_name.as_str(),
            traits.input_power_device_name.as_str(),
            traits.input_volume_device_name.as_str(),
        ];
        let mut streams = Vec::new();
        for name in names {
            if let Some(stream) = open_input(name)? {
                streams.push(stream);
            }
        }
        let keycodes = traits
            .keycodes
            .iter()
            .map(|(&code, &key)| (code, key))
            .collect();
        Ok(Self { streams, keycodes })
    }

    pub async fn poll(&mut self) -> KeyEvent {
        loop {
            let events = self
                .streams
                .iter_mut()
                .map(|stream| Box::pin(stream.next_event()));
            let (event, _, _) = select_all(events).await;
            if let Ok(event) = event
                && event.event_type() == EventType::KEY
                && let Some(key) = self
                    .keycodes
                    .iter()
                    .find_map(|&(code, key)| (code == event.code()).then_some(key))
            {
                return match event.value() {
                    0 => KeyEvent::Released(key),
                    1 => KeyEvent::Pressed(key),
                    2 => KeyEvent::Autorepeat(key),
                    _ => continue,
                };
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

fn open_input(expected_name: &str) -> Result<Option<EventStream>> {
    for entry in std::fs::read_dir("/dev/input").context("failed to read /dev/input")? {
        let path = entry?.path();
        if !path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("event"))
        {
            continue;
        }
        let device = Device::open(&path)?;
        if device.name() == Some(expected_name) {
            return device.into_event_stream().map(Some).map_err(Into::into);
        }
    }
    Ok(None)
}
