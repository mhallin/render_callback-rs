use std::fmt;

use coreaudio_sys::AudioDeviceID;

use crate::traits::Device;

use super::backend::CABackend;
use super::cf::{CFError, CFString};
use super::properties::{self, element, scope, selector};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CADevice(pub(crate) AudioDeviceID);

impl CADevice {
    pub unsafe fn uninit() -> Self {
        CADevice(0)
    }

    pub fn new(id: AudioDeviceID) -> Self {
        CADevice(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn uid(&self) -> Result<CFString, CFError> {
        unsafe {
            properties::get(
                element::Master,
                scope::Output,
                selector::DevicePropertyDeviceUID,
                self.0,
            )
        }
    }
}

impl fmt::Debug for CADevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceID")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("input_count", &self.num_inputs())
            .field("output_count", &self.num_outputs())
            .finish()
    }
}

impl Device<CABackend> for CADevice {
    fn num_inputs(&self) -> Result<usize, CFError> {
        let inputs = unsafe {
            properties::get(
                element::Master,
                scope::Input,
                selector::DevicePropertyStreamConfiguration,
                self.0,
            )?
        };
        Ok(inputs.mNumberBuffers as usize)
    }

    fn num_outputs(&self) -> Result<usize, CFError> {
        let outputs = unsafe {
            properties::get(
                element::Master,
                scope::Output,
                selector::DevicePropertyStreamConfiguration,
                self.0,
            )?
        };
        Ok(outputs.mNumberBuffers as usize)
    }

    fn name(&self) -> Result<String, CFError> {
        let cfstr = unsafe {
            properties::get(
                element::Master,
                scope::Wildcard,
                selector::ObjectPropertyName,
                self.0,
            )?
        };

        Ok(cfstr.to_string())
    }

    fn set_nominal_sample_rate(&mut self, sample_rate: f64) -> Result<(), CFError> {
        unsafe {
            properties::set(
                element::Master,
                scope::Wildcard,
                selector::DevicePropertyNominalSampleRate,
                self.0,
                &sample_rate,
            )
        }
    }

    fn nominal_sample_rate(&self) -> Result<f64, CFError> {
        unsafe {
            properties::get(
                element::Master,
                scope::Wildcard,
                selector::DevicePropertyNominalSampleRate,
                self.0,
            )
        }
    }

    fn actual_sample_rate(&self) -> Result<f64, CFError> {
        unsafe {
            properties::get(
                element::Master,
                scope::Wildcard,
                selector::DevicePropertyActualSampleRate,
                self.0,
            )
        }
    }
}
