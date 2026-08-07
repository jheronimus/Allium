use std::process::Command;

use anyhow::Result;

use crate::platform::minime::traits::Traits;

/// Volume control via ALSA mixer, using trait card/mixer names.
pub fn set_volume(traits: &Traits, volume: i32) -> Result<()> {
    let status = Command::new("amixer")
        .args([
            "-q",
            "-D",
            &traits.audio_card,
            "sset",
            &traits.audio_mixer,
            &format!("{}%", volume.clamp(0, 20) * 5),
        ])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("amixer exited with {status}"))
    }
}
