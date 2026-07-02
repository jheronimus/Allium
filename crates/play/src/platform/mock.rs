use crate::commands::ControlEvent;
use crate::input::JoypadState;
use crate::video::ScaleMode;
use anyhow::Result;

pub struct MockPlatform {
    pub video: MockVideo,
}

pub struct MockVideo;

impl MockVideo {
    pub fn present(
        &mut self,
        _frame: &crate::video::CapturedFrame,
        _format: crate::video::VideoFrameFormat,
    ) -> Result<bool> {
        Ok(true)
    }

    pub fn set_scale(
        &mut self,
        _scale: ScaleMode,
        _base_width: u32,
        _base_height: u32,
        _aspect_ratio: f32,
    ) -> Result<()> {
        Ok(())
    }

    pub fn set_effect(&mut self, _effect: crate::settings::ScreenEffect) {}

    pub fn set_sharpness(&mut self, _sharpness: crate::settings::ScreenSharpness) {}
}

impl MockPlatform {
    pub fn new(
        _core_id: &str,
        _source_width: u32,
        _source_height: u32,
        _aspect_ratio: f32,
        _scale: ScaleMode,
        _sample_rate: u32,
        _audio_consumer: crate::audio::AudioConsumer,
    ) -> Result<Self> {
        Ok(Self { video: MockVideo })
    }

    pub fn poll_input(&mut self, _joypad: &mut JoypadState) -> Vec<ControlEvent> {
        Vec::new()
    }

    pub fn cpu_usage(&mut self) -> Option<f64> {
        None
    }

    pub fn skip_presentation_when_paused(&self) -> bool {
        true
    }

    pub async fn wait_for_shutdown(&mut self) {}
}
