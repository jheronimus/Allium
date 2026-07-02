#![allow(unused)]
#[cfg(feature = "minime")]
mod audio;
#[cfg(feature = "minime")]
mod commands;
#[cfg(feature = "minime")]
mod config;
#[cfg(feature = "minime")]
mod content;
#[cfg(feature = "minime")]
mod controls;
#[cfg(feature = "minime")]
mod core;
#[cfg(feature = "minime")]
mod core_options;
#[cfg(feature = "minime")]
mod dump;
#[cfg(feature = "minime")]
mod hud;
#[cfg(feature = "minime")]
mod input;
#[cfg(feature = "minime")]
mod paths;
#[cfg(feature = "minime")]
mod platform;
#[cfg(feature = "minime")]
mod real_main;
#[cfg(feature = "minime")]
mod save;
#[cfg(feature = "minime")]
mod settings;
#[cfg(feature = "minime")]
mod unzip;
#[cfg(feature = "minime")]
mod video;

#[cfg(feature = "minime")]
fn main() -> anyhow::Result<()> {
    real_main::main()
}

#[cfg(not(feature = "minime"))]
fn main() {}
