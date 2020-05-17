use coreaudio_sys::kAudioObjectSystemObject;

use crate::traits::{Backend, RenderCallback};

use super::cf::CFError;
use super::device::CADevice;
use super::properties::{self, element, scope, selector};
use super::session::{CASession, InterleavedBuffer};

pub struct CABackend;

impl Backend for CABackend {
    type Session = Box<CASession>;
    type Error = CFError;
    type Device = CADevice;
    type AudioBuffers = InterleavedBuffer;

    fn new() -> Result<Self, Self::Error> {
        Ok(CABackend)
    }

    fn all_devices(&self) -> Result<Vec<CADevice>, CFError> {
        properties::get(
            element::Master,
            scope::Wildcard,
            selector::HardwarePropertyDevices,
            kAudioObjectSystemObject,
        )
    }

    fn default_input_device(&self) -> Result<CADevice, CFError> {
        properties::get(
            element::Master,
            scope::Global,
            selector::HardwarePropertyDefaultInputDevice,
            kAudioObjectSystemObject,
        )
    }

    fn default_output_device(&self) -> Result<CADevice, CFError> {
        properties::get(
            element::Master,
            scope::Global,
            selector::HardwarePropertyDefaultOutputDevice,
            kAudioObjectSystemObject,
        )
    }

    fn start_session(
        &self,
        sample_rate: f64,
        input_device: Self::Device,
        output_device: Self::Device,
        callback: Box<RenderCallback<Self>>,
    ) -> Result<Self::Session, Self::Error> {
        CASession::new_started(self, sample_rate, input_device, output_device, callback)
    }
}
