use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};

use crate::platform::Key;

pub const TRAITS_PATH: &str = "/mnt/sdcard/.minime/traits";

/// Screen aspect ratio as expressed by the `screen_aspect` trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aspect {
    R4x3,
    R3x2,
    R16x9,
    R1x1,
    Other(u32, u32),
}

impl Aspect {
    pub fn width_ratio(self) -> u32 {
        match self {
            Aspect::R4x3 => 4,
            Aspect::R3x2 => 3,
            Aspect::R16x9 => 16,
            Aspect::R1x1 => 1,
            Aspect::Other(w, _) => w,
        }
    }

    pub fn height_ratio(self) -> u32 {
        match self {
            Aspect::R4x3 => 3,
            Aspect::R3x2 => 2,
            Aspect::R16x9 => 9,
            Aspect::R1x1 => 1,
            Aspect::Other(_, h) => h,
        }
    }
}

impl std::fmt::Display for Aspect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.width_ratio(), self.height_ratio())
    }
}

/// Fully-resolved Minime device traits, parsed from the flat file emitted by
/// `init.d/traits` at boot. Mirrors the MinUI `MinimeTraits` struct: the file
/// is the single source of truth and this parser fails loudly on unknown or
/// malformed keys rather than silently producing a half-populated struct.
#[derive(Debug, Clone)]
pub struct Traits {
    // [device]
    pub device_id: String,
    pub device_model: String,

    // [screen]
    pub screen_width: u32,
    pub screen_height: u32,
    pub screen_rotation: u32,
    pub screen_rotation_kernel: Option<u32>,
    pub screen_aspect: Aspect,
    pub screen_refresh_rate: u32,
    pub screen_backlight_path: Option<PathBuf>,
    pub screen_backlight_max: Option<u32>,
    pub screen_blank_path: Option<PathBuf>,
    pub screen2_width: Option<u32>,
    pub screen2_height: Option<u32>,
    pub screen2_rotation: Option<u32>,
    pub screen2_aspect: Option<Aspect>,
    pub screen2_refresh_rate: Option<u32>,
    pub screen2_backlight_path: Option<PathBuf>,
    pub screen2_blank_path: Option<PathBuf>,
    pub screen2_touch: bool,
    pub screen2_touch_device_name: Option<String>,

    // [cpu]
    pub cpu_governor_path: Option<PathBuf>,
    pub cpu_clock_path: Option<PathBuf>,
    pub cpu_clock_menu: Option<u32>,
    pub cpu_clock_powersave: Option<u32>,
    pub cpu_clock_normal: Option<u32>,
    pub cpu_clock_performance: Option<u32>,
    pub cpu_undervolt_supported: bool,
    pub cpu_thermal_path: Option<PathBuf>,

    // [gpu]
    pub gpu_device: String,
    pub gpu_device2: Option<String>,
    /// Stable connector identifier from the file, e.g. "HDMI-A-1". The card
    /// number prefix is dynamic (DRM minor index, first-come-first-serve) and
    /// is resolved at load time into `gpu_hdmi_state_path`.
    pub gpu_hdmi_connector: Option<String>,
    /// Resolved DRM sysfs status path, e.g. `/sys/class/drm/card0-HDMI-A-1/status`.
    pub gpu_hdmi_state_path: Option<PathBuf>,
    pub gpu_driver: Option<String>,
    pub gpu_clock_min: Option<u32>,
    pub gpu_clock_max: Option<u32>,

    // [audio]
    pub audio_card: String,
    pub audio_mixer: String,
    pub audio_jack_device_name: Option<String>,
    pub audio_mic: bool,

    // [input]
    pub input_gamepad_device_name: String,
    pub input_power_device_name: String,
    pub input_volume_device_name: String,
    pub input_lid_device_name: Option<String>,
    pub input_rumble_device_name: Option<String>,
    pub input_touch: bool,
    pub input_touch_device_name: Option<String>,
    pub keycodes: HashMap<u16, Key>,
    pub axis_lx: Option<u16>,
    pub axis_ly: Option<u16>,
    pub axis_rx: Option<u16>,
    pub axis_ry: Option<u16>,
    pub axis_min: Option<i32>,
    pub axis_center: Option<i32>,
    pub axis_max: Option<i32>,
    pub axis_lx_invert: bool,
    pub axis_ly_invert: bool,
    pub axis_rx_invert: bool,
    pub axis_ry_invert: bool,

    // [wireless]
    pub wifi_interface: Option<String>,
    pub bluetooth_interface: Option<String>,

    // [power]
    pub power_battery_sysfs: Option<PathBuf>,
    pub power_charger_online_path: Option<PathBuf>,
    pub power_led_path: Option<PathBuf>,

    // [usb]
    pub usb_otg: bool,
    pub usb_host_ports: u32,
    pub usb_device_mode: bool,
    pub usb_controller_mode: bool,

    // [storage]
    pub storage_sd_node: Option<PathBuf>,
    pub storage_sd2_node: Option<PathBuf>,
    pub storage_emmc_node: Option<PathBuf>,
}

impl Traits {
    pub fn load() -> Result<Self> {
        let raw = fs::read_to_string(TRAITS_PATH)
            .context(format!("failed to read Minime traits at {TRAITS_PATH}"))?;
        Self::parse(&raw)
    }

    pub fn parse(input: &str) -> Result<Self> {
        let values = parse_values(input);

        // Strict validation: every key present in the file must be a known
        // schema key. This catches typos and schema drift immediately.
        for key in values.keys() {
            if !KNOWN_KEYS.contains(key) {
                return Err(anyhow!("unknown trait: {key}"));
            }
        }

        let traits = Self {
            device_id: required(&values, "device_id")?.to_owned(),
            device_model: required(&values, "device_model")?.to_owned(),

            screen_width: required_number(&values, "screen_width")?,
            screen_height: required_number(&values, "screen_height")?,
            screen_rotation: number(&values, "screen_rotation").unwrap_or(0),
            screen_rotation_kernel: optional(&values, "screen_rotation_kernel")
                .and_then(|s| s.parse().ok()),
            screen_aspect: parse_aspect(values.get("screen_aspect")).unwrap_or_else(|| {
                derive_aspect((
                    required_number(&values, "screen_width").unwrap_or_default(),
                    required_number(&values, "screen_height").unwrap_or_default(),
                ))
            }),
            screen_refresh_rate: number(&values, "screen_refresh_rate").unwrap_or(60),
            screen_backlight_path: optional_path(&values, "screen_backlight_path"),
            screen_backlight_max: optional(&values, "screen_backlight_max")
                .and_then(|s| s.parse().ok()),
            screen_blank_path: optional_path(&values, "screen_blank_path"),
            screen2_width: optional(&values, "screen2_width").and_then(|s| s.parse().ok()),
            screen2_height: optional(&values, "screen2_height").and_then(|s| s.parse().ok()),
            screen2_rotation: optional(&values, "screen2_rotation").and_then(|s| s.parse().ok()),
            screen2_aspect: parse_aspect(values.get("screen2_aspect")),
            screen2_refresh_rate: optional(&values, "screen2_refresh_rate")
                .and_then(|s| s.parse().ok()),
            screen2_backlight_path: optional_path(&values, "screen2_backlight_path"),
            screen2_blank_path: optional_path(&values, "screen2_blank_path"),
            screen2_touch: optional(&values, "screen2_touch").is_some_and(|s| s == "1"),
            screen2_touch_device_name: optional(&values, "screen2_touch_device_name"),

            cpu_governor_path: optional_path(&values, "cpu_governor_path"),
            cpu_clock_path: optional_path(&values, "cpu_clock_path"),
            cpu_clock_menu: optional(&values, "cpu_clock_menu").and_then(|s| s.parse().ok()),
            cpu_clock_powersave: optional(&values, "cpu_clock_powersave")
                .and_then(|s| s.parse().ok()),
            cpu_clock_normal: optional(&values, "cpu_clock_normal").and_then(|s| s.parse().ok()),
            cpu_clock_performance: optional(&values, "cpu_clock_performance")
                .and_then(|s| s.parse().ok()),
            cpu_undervolt_supported: optional(&values, "cpu_undervolt_supported")
                .is_some_and(|s| s == "1"),
            cpu_thermal_path: optional_path(&values, "cpu_thermal_path"),

            gpu_device: required(&values, "gpu_device")?.to_owned(),
            gpu_device2: optional(&values, "gpu_device2"),
            gpu_hdmi_connector: optional(&values, "gpu_hdmi_connector"),
            gpu_hdmi_state_path: None,
            gpu_driver: optional(&values, "gpu_driver"),
            gpu_clock_min: optional(&values, "gpu_clock_min").and_then(|s| s.parse().ok()),
            gpu_clock_max: optional(&values, "gpu_clock_max").and_then(|s| s.parse().ok()),

            audio_card: required(&values, "audio_card")?.to_owned(),
            audio_mixer: required(&values, "audio_mixer")?.to_owned(),
            audio_jack_device_name: optional(&values, "audio_jack_device_name"),
            audio_mic: optional(&values, "audio_mic").is_some_and(|s| s == "1"),

            input_gamepad_device_name: required(&values, "input_gamepad_device_name")?.to_owned(),
            input_power_device_name: required(&values, "input_power_device_name")?.to_owned(),
            input_volume_device_name: required(&values, "input_volume_device_name")?.to_owned(),
            input_lid_device_name: optional(&values, "input_lid_device_name"),
            input_rumble_device_name: optional(&values, "input_rumble_device_name"),
            input_touch: optional(&values, "input_touch").is_some_and(|s| s == "1"),
            input_touch_device_name: optional(&values, "input_touch_device_name"),
            keycodes: parse_keycodes(&values)?,
            axis_lx: optional(&values, "input_axis_lx").and_then(|s| s.parse().ok()),
            axis_ly: optional(&values, "input_axis_ly").and_then(|s| s.parse().ok()),
            axis_rx: optional(&values, "input_axis_rx").and_then(|s| s.parse().ok()),
            axis_ry: optional(&values, "input_axis_ry").and_then(|s| s.parse().ok()),
            axis_min: optional(&values, "input_axis_min").and_then(|s| s.parse().ok()),
            axis_center: optional(&values, "input_axis_center").and_then(|s| s.parse().ok()),
            axis_max: optional(&values, "input_axis_max").and_then(|s| s.parse().ok()),
            axis_lx_invert: optional(&values, "input_axis_lx_invert").is_some_and(|s| s == "1"),
            axis_ly_invert: optional(&values, "input_axis_ly_invert").is_some_and(|s| s == "1"),
            axis_rx_invert: optional(&values, "input_axis_rx_invert").is_some_and(|s| s == "1"),
            axis_ry_invert: optional(&values, "input_axis_ry_invert").is_some_and(|s| s == "1"),

            wifi_interface: optional(&values, "wifi_interface"),
            bluetooth_interface: optional(&values, "bluetooth_interface"),

            power_battery_sysfs: optional_path(&values, "power_battery_sysfs"),
            power_charger_online_path: optional_path(&values, "power_charger_online_path"),
            power_led_path: optional_path(&values, "power_led_path"),

            usb_otg: optional(&values, "usb_otg").is_some_and(|s| s == "1"),
            usb_host_ports: optional(&values, "usb_host_ports")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            usb_device_mode: optional(&values, "usb_device_mode").is_some_and(|s| s == "1"),
            usb_controller_mode: optional(&values, "usb_controller_mode").is_some_and(|s| s == "1"),

            storage_sd_node: optional_path(&values, "storage_sd_node"),
            storage_sd2_node: optional_path(&values, "storage_sd2_node"),
            storage_emmc_node: optional_path(&values, "storage_emmc_node"),
        };

        Ok(resolve_hdmi_connector(traits))
    }
    /// The set of evdev device names the UI should open for game input:
    /// gamepad, power, and volume.
    pub fn input_device_names(&self) -> Vec<&str> {
        [
            self.input_gamepad_device_name.as_str(),
            self.input_power_device_name.as_str(),
            self.input_volume_device_name.as_str(),
        ]
        .to_vec()
    }

    /// The evdev name of the lid switch device, if the device has a clamshell.
    pub fn lid_device_name(&self) -> Option<&str> {
        self.input_lid_device_name.as_deref()
    }
}

/// Reduce width/height to a normalized aspect ratio.
fn derive_aspect((w, h): (u32, u32)) -> Aspect {
    let gcd = |mut a: u32, mut b: u32| {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    };
    let g = gcd(w, h);
    Aspect::Other(w / g, h / g)
}

fn parse_aspect(value: Option<&&str>) -> Option<Aspect> {
    let (w, h) = value?.split_once(':')?;
    let w: u32 = w.parse().ok()?;
    let h: u32 = h.parse().ok()?;
    Some(Aspect::Other(w, h))
}

const NA: &str = "na";

/// Resolve the stable `gpu_hdmi_connector` (e.g. "HDMI-A-1") to the actual
/// DRM sysfs status path. The card number prefix ("card0", "card1", ...) is
/// the DRM primary-minor index, allocated first-come-first-serve at probe
/// time, so it is NOT part of the trait file — we scan /sys/class/drm/card*
/// and pick the connector whose suffix matches. Returns `traits` unchanged
/// when no connector is configured or none is found.
fn resolve_hdmi_connector(mut traits: Traits) -> Traits {
    let Some(connector) = traits.gpu_hdmi_connector.clone() else {
        return traits;
    };

    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(dash) = name.find('-') {
                if &name[dash + 1..] == connector {
                    traits.gpu_hdmi_state_path =
                        Some(PathBuf::from(format!("/sys/class/drm/{name}/status")));
                    break;
                }
            }
        }
    }
    traits
}

fn parse_values(input: &str) -> HashMap<&str, &str> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('['))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim(), value.trim()))
        .collect()
}

fn required<'a>(values: &'a HashMap<&str, &str>, key: &str) -> Result<&'a str> {
    values
        .get(key)
        .copied()
        .filter(|value| !value.is_empty() && *value != NA)
        .ok_or_else(|| anyhow!("missing required trait: {key}"))
}

fn required_number<T>(values: &HashMap<&str, &str>, key: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    required(values, key)?
        .parse()
        .with_context(|| format!("invalid trait: {key}"))
}

fn optional<'a>(values: &'a HashMap<&str, &str>, key: &str) -> Option<String> {
    values
        .get(key)
        .filter(|value| !value.is_empty() && **value != NA)
        .map(|value| (*value).to_owned())
}

fn optional_path(values: &HashMap<&str, &str>, key: &str) -> Option<PathBuf> {
    optional(values, key).map(PathBuf::from)
}

fn number<T>(values: &HashMap<&str, &str>, key: &str) -> Option<T>
where
    T: std::str::FromStr,
{
    optional(values, key).and_then(|s| s.parse().ok())
}

const KEYS: &[(&str, Key)] = &[
    ("key_a", Key::A),
    ("key_b", Key::B),
    ("key_c", Key::C),
    ("key_x", Key::X),
    ("key_y", Key::Y),
    ("key_z", Key::Z),
    ("key_up", Key::Up),
    ("key_down", Key::Down),
    ("key_left", Key::Left),
    ("key_right", Key::Right),
    ("key_start", Key::Start),
    ("key_select", Key::Select),
    ("key_l1", Key::L),
    ("key_r1", Key::R),
    ("key_l2", Key::L2),
    ("key_r2", Key::R2),
    ("key_l3", Key::Unknown),
    ("key_r3", Key::Unknown),
    ("key_menu", Key::Menu),
    ("key_power", Key::Power),
    ("key_vol_down", Key::VolDown),
    ("key_vol_up", Key::VolUp),
];

fn parse_keycodes(values: &HashMap<&str, &str>) -> Result<HashMap<u16, Key>> {
    let mut keycodes = HashMap::new();
    for (name, key) in KEYS {
        if let Some(value) = values
            .get(name)
            .filter(|value| !value.is_empty() && **value != NA)
        {
            keycodes.insert(
                value
                    .parse()
                    .with_context(|| format!("invalid trait: {name}"))?,
                *key,
            );
        }
    }
    Ok(keycodes)
}

/// Complete list of keys accepted by the parser. Anything else is an error.
const KNOWN_KEYS: &[&str] = &[
    "device_id",
    "device_model",
    "screen_width",
    "screen_height",
    "screen_rotation",
    "screen_rotation_kernel",
    "screen_aspect",
    "screen_refresh_rate",
    "screen_backlight_path",
    "screen_backlight_max",
    "screen_blank_path",
    "screen2_width",
    "screen2_height",
    "screen2_rotation",
    "screen2_aspect",
    "screen2_refresh_rate",
    "screen2_backlight_path",
    "screen2_blank_path",
    "screen2_touch",
    "screen2_touch_device_name",
    "cpu_governor_path",
    "cpu_clock_path",
    "cpu_clock_menu",
    "cpu_clock_powersave",
    "cpu_clock_normal",
    "cpu_clock_performance",
    "cpu_undervolt_supported",
    "cpu_thermal_path",
    "gpu_device",
    "gpu_device2",
    "gpu_hdmi_connector",
    "gpu_driver",
    "gpu_clock_min",
    "gpu_clock_max",
    "audio_card",
    "audio_mixer",
    "audio_jack_device_name",
    "audio_mic",
    "input_gamepad_device_name",
    "input_power_device_name",
    "input_volume_device_name",
    "input_lid_device_name",
    "input_rumble_device_name",
    "input_touch",
    "input_touch_device_name",
    "key_a",
    "key_b",
    "key_c",
    "key_x",
    "key_y",
    "key_z",
    "key_up",
    "key_down",
    "key_left",
    "key_right",
    "key_start",
    "key_select",
    "key_l1",
    "key_r1",
    "key_l2",
    "key_r2",
    "key_l3",
    "key_r3",
    "key_menu",
    "key_power",
    "key_vol_down",
    "key_vol_up",
    "input_axis_lx",
    "input_axis_ly",
    "input_axis_rx",
    "input_axis_ry",
    "input_axis_min",
    "input_axis_center",
    "input_axis_max",
    "input_axis_lx_invert",
    "input_axis_ly_invert",
    "input_axis_rx_invert",
    "input_axis_ry_invert",
    "wifi_interface",
    "bluetooth_interface",
    "power_battery_sysfs",
    "power_charger_online_path",
    "power_led_path",
    "usb_otg",
    "usb_host_ports",
    "usb_device_mode",
    "usb_controller_mode",
    "storage_sd_node",
    "storage_sd2_node",
    "storage_emmc_node",
];

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
device_id=rg35xx-sp-v1
device_model=Anbernic RG35XX SP
screen_width=640
screen_height=480
screen_rotation=0
screen_aspect=4:3
screen_refresh_rate=60
screen_backlight_path=/sys/class/backlight/backlight/brightness
screen_backlight_max=10
screen_blank_path=/sys/class/graphics/fb0/blank
gpu_device=/dev/fb0
gpu_hdmi_connector=HDMI-A-1
audio_card=default
audio_mixer=Line Out
audio_jack_device_name=H616 Audio Codec Headphone Jack
input_gamepad_device_name=gpio-keys-gamepad
input_power_device_name=axp20x-pek
input_volume_device_name=gpio-keys-volume
input_lid_device_name=gpio-keys-lid
key_a=305
key_b=304
key_up=544
key_down=545
key_menu=316
wifi_interface=wlan0
bluetooth_interface=hci0
power_battery_sysfs=/sys/class/power_supply/battery
power_charger_online_path=/sys/class/power_supply/axp20x-usb/online
";

    #[test]
    fn parses_full_schema() {
        let t = Traits::parse(SAMPLE).unwrap();
        assert_eq!(t.device_model, "Anbernic RG35XX SP");
        assert_eq!(t.screen_width, 640);
        assert_eq!(t.screen_height, 480);
        assert_eq!(t.screen_aspect.to_string(), "4:3");
        assert_eq!(t.screen_backlight_max, Some(10));
        assert_eq!(t.gpu_device, "/dev/fb0");
        assert_eq!(t.audio_mixer, "Line Out");
        assert_eq!(
            t.audio_jack_device_name.as_deref(),
            Some("H616 Audio Codec Headphone Jack")
        );
        assert_eq!(t.input_lid_device_name.as_deref(), Some("gpio-keys-lid"));
        assert_eq!(t.keycodes.get(&305), Some(&Key::A));
        assert_eq!(t.keycodes.get(&544), Some(&Key::Up));
        assert_eq!(t.wifi_interface.as_deref(), Some("wlan0"));
        assert_eq!(t.bluetooth_interface.as_deref(), Some("hci0"));
    }

    #[test]
    fn rejects_unknown_key() {
        let error = format!("{SAMPLE}not_a_real_trait=1\n");
        assert!(Traits::parse(&error).is_err());
    }

    #[test]
    fn rejects_missing_required() {
        assert!(Traits::parse("device_model=Anbernic RG35XX SP\n").is_err());
    }

    #[test]
    fn derives_aspect_when_absent() {
        let input = SAMPLE.replace("screen_aspect=4:3\n", "");
        let t = Traits::parse(&input).unwrap();
        assert_eq!(t.screen_aspect.to_string(), "4:3");
    }
}
