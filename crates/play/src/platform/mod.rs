#[cfg(all(target_os = "linux", feature = "minime"))]
pub mod minime;

#[cfg(not(all(target_os = "linux", feature = "minime")))]
pub mod mock;

#[cfg(all(target_os = "linux", feature = "minime"))]
pub fn init_logging() -> anyhow::Result<()> {
    minime::init_logging()
}

#[cfg(not(all(target_os = "linux", feature = "minime")))]
pub fn init_logging() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(all(target_os = "linux", feature = "minime"))]
pub type DefaultPlatform = minime::MinimePlatform;

#[cfg(not(all(target_os = "linux", feature = "minime")))]
pub type DefaultPlatform = mock::MockPlatform;

#[cfg(all(target_os = "linux", feature = "minime"))]
pub fn set_governor(governor: &str) {
    minime::set_governor(governor);
}

#[cfg(not(all(target_os = "linux", feature = "minime")))]
pub fn set_governor(_governor: &str) {}
