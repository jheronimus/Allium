mod audio;
mod battery;
mod framebuffer;
mod input;
mod power;
mod screen;
pub mod traits;

use anyhow::Result;
use async_trait::async_trait;
use std::os::unix::process::CommandExt;

use crate::battery::Battery;
use crate::display::settings::DisplaySettings;
use crate::platform::minime::battery::MinimeBattery;
use crate::platform::minime::framebuffer::MinimeDisplay;
use crate::platform::minime::input::MinimeInput;
use crate::platform::minime::traits::Traits;
use crate::platform::{KeyEvent, Platform};

pub use traits::Aspect;

pub struct MinimePlatform {
    traits: Traits,
    input: MinimeInput,
}

pub struct SuspendContext {
    brightness: u8,
}

impl MinimePlatform {
    fn load_traits() -> Result<Traits> {
        Traits::load()
    }
}

#[async_trait(?Send)]
impl Platform for MinimePlatform {
    type Display = MinimeDisplay;
    type Battery = Box<dyn Battery>;
    type SuspendContext = SuspendContext;

    fn new() -> Result<Self> {
        let traits = Self::load_traits()?;
        let input = MinimeInput::new(&traits)?;
        Ok(Self { traits, input })
    }

    async fn poll(&mut self) -> KeyEvent {
        self.input.poll().await
    }

    fn display(&mut self) -> Result<Self::Display> {
        MinimeDisplay::new(&self.traits)
    }

    fn battery(&self) -> Result<Self::Battery> {
        Ok(Box::new(MinimeBattery::new(
            self.traits.power_battery_sysfs.clone(),
            self.traits.power_charger_online_path.clone(),
        )))
    }

    fn shutdown(&self) -> Result<()> {
        std::process::Command::new("sync").status()?;
        let error = std::process::Command::new("poweroff").exec();
        Err(error.into())
    }

    fn suspend(&self) -> Result<Self::SuspendContext> {
        let brightness = screen::get_brightness(&self.traits)?;
        screen::set_brightness(&self.traits, 0)?;
        screen::blank(&self.traits, true)?;
        Ok(SuspendContext { brightness })
    }

    fn unsuspend(&self, ctx: Self::SuspendContext) -> Result<()> {
        screen::blank(&self.traits, false)?;
        screen::set_brightness(&self.traits, ctx.brightness)
    }

    fn set_volume(&mut self, volume: i32) -> Result<()> {
        audio::set_volume(&self.traits, volume)
    }

    fn get_brightness(&self) -> Result<u8> {
        screen::get_brightness(&self.traits)
    }

    fn set_brightness(&mut self, brightness: u8) -> Result<()> {
        screen::set_brightness(&self.traits, brightness)
    }

    fn set_display_settings(&mut self, _settings: &mut DisplaySettings) -> Result<()> {
        Ok(())
    }

    fn device_model() -> String {
        Traits::load()
            .map(|traits| traits.device_model)
            .unwrap_or_else(|_| "Minime".to_owned())
    }

    fn firmware() -> String {
        std::fs::read_to_string("/etc/minime-version")
            .map(|version| version.trim().to_owned())
            .unwrap_or_else(|_| "Minime".to_owned())
    }

    fn has_wifi() -> bool {
        Traits::load().is_ok_and(|traits| traits.wifi_interface.is_some())
    }

    fn has_lid() -> bool {
        Traits::load().is_ok_and(|traits| traits.input_lid_device_name.is_some())
    }
}
