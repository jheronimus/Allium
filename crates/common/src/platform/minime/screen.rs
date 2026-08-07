use std::fs;

use anyhow::{Context, Result};

use crate::platform::minime::traits::Traits;

/// Backlight and panel-blank helpers driven by trait paths.
pub fn get_brightness(traits: &Traits) -> Result<u8> {
    let Some(path) = traits.screen_backlight_path.as_deref() else {
        return Ok(0);
    };
    Ok(fs::read_to_string(path)?.trim().parse()?)
}

pub fn set_brightness(traits: &Traits, brightness: u8) -> Result<()> {
    let Some(path) = traits.screen_backlight_path.as_deref() else {
        return Ok(());
    };
    fs::write(path, brightness.to_string())
        .with_context(|| format!("failed to write {}", path.display()))
}

pub fn blank(traits: &Traits, blank: bool) -> Result<()> {
    let Some(path) = traits.screen_blank_path.as_deref() else {
        return Ok(());
    };
    fs::write(path, if blank { "4" } else { "0" })
        .with_context(|| format!("failed to write {}", path.display()))
}
