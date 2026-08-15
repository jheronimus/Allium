use std::fs;

use anyhow::{Context, Result};
use evdev::{Device, FFEffectData, FFEffectKind, FFReplay, FFTrigger};

use crate::platform::minime::traits::Traits;

/// Power LED and CPU speed control driven by trait paths. Mirrors the MinUI
/// `MINIME_power*` HAL. Currently unused by the Platform trait (LED/CPU-speed
/// features land later); kept as the complete traits-driven interface so the
/// file matches traits.c 1:1.
#[allow(dead_code)]
pub fn set_led(traits: &Traits, enabled: bool) -> Result<()> {
    let Some(path) = traits.power_led_path.as_deref() else {
        return Ok(());
    };
    std::fs::write(path, if enabled { "1" } else { "0" })
        .with_context(|| format!("failed to write {}", path.display()))
}

/// Rumble the motor via the input-device force-feedback interface. The motor
/// is exposed as an input device (e.g. "pwm-vibrator") with FF_RUMBLE; upload
/// a rumble effect and play/stop it.
#[allow(dead_code)]
pub fn set_rumble(traits: &Traits, enabled: bool) -> Result<()> {
    let Some(name) = traits.input_rumble_device_name.as_deref() else {
        return Ok(());
    };
    let Some(mut device) = open_input_by_name(name)? else {
        return Ok(());
    };

    let effect = FFEffectData {
        direction: 0,
        trigger: FFTrigger::default(),
        replay: FFReplay {
            length: 1000,
            delay: 0,
        },
        kind: FFEffectKind::Rumble {
            strong_magnitude: if enabled { 0xffff } else { 0 },
            weak_magnitude: if enabled { 0xffff } else { 0 },
        },
    };
    let mut effect = device.upload_ff_effect(effect)?;
    if enabled {
        effect.play(-1)?; // loop until stopped
    } else {
        effect.stop()?;
    }
    Ok(())
}

fn open_input_by_name(expected_name: &str) -> Result<Option<Device>> {
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
            return Ok(Some(device));
        }
    }
    Ok(None)
}

/// Set CPU speed level. 0=menu, 1=powersave, 2=normal, 3=performance.
/// Mirrors the MinUI HAL semantics from ADR 0018: levels 0-2 use schedutil
/// with a max-frequency cap, level 3 pins the performance governor.
#[allow(dead_code)]
pub fn set_cpu_speed(traits: &Traits, speed: i32) -> Result<()> {
    let Some(governor_path) = traits.cpu_governor_path.as_deref() else {
        return Ok(());
    };
    let Some(clock_path) = traits.cpu_clock_path.as_deref() else {
        return Ok(());
    };

    let (governor, clock) = match speed {
        s if s <= 0 => ("schedutil", traits.cpu_clock_menu),
        1 => ("schedutil", traits.cpu_clock_powersave),
        2 => ("schedutil", traits.cpu_clock_normal),
        _ => ("performance", traits.cpu_clock_performance),
    };

    fs::write(governor_path, governor)
        .with_context(|| format!("failed to write {}", governor_path.display()))?;
    if let Some(clock) = clock {
        fs::write(clock_path, clock.to_string())
            .with_context(|| format!("failed to write {}", clock_path.display()))?;
    }
    Ok(())
}
