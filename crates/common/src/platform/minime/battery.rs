use std::path::PathBuf;

use anyhow::Result;

use crate::battery::Battery;

pub struct MinimeBattery {
    battery_sysfs: Option<PathBuf>,
    charger_online_path: Option<PathBuf>,
    charging: bool,
    percentage: i32,
}

impl MinimeBattery {
    pub fn new(battery_sysfs: Option<PathBuf>, charger_online_path: Option<PathBuf>) -> Self {
        Self {
            battery_sysfs,
            charger_online_path,
            charging: false,
            percentage: 100,
        }
    }

    fn read_number<T>(path: &std::path::Path) -> Result<T>
    where
        T: std::str::FromStr,
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        Ok(std::fs::read_to_string(path)?
            .trim()
            .parse()
            .map_err(|e: T::Err| anyhow::anyhow!("invalid number in {}: {e}", path.display()))?)
    }
}

impl Battery for MinimeBattery {
    fn update(&mut self) -> Result<()> {
        if let Some(dir) = self.battery_sysfs.as_deref() {
            self.percentage = Self::read_number(&dir.join("capacity"))?;
            let status = std::fs::read_to_string(dir.join("status")).unwrap_or_default();
            self.charging = matches!(status.trim(), "Charging" | "Full");
        }
        if let Some(path) = self.charger_online_path.as_deref() {
            self.charging = Self::read_number::<i32>(path)? != 0;
        }
        Ok(())
    }

    fn percentage(&self) -> i32 {
        self.percentage
    }

    fn charging(&self) -> bool {
        self.charging
    }
}
