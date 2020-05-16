use std::ffi::c_void;
use std::fmt;
use std::mem::MaybeUninit;

use coreaudio_sys::{
    kAudioDevicePropertyDeviceName, kAudioDevicePropertyDeviceUID, kAudioDevicePropertyScopeInput,
    kAudioDevicePropertyScopeOutput, kAudioDevicePropertyStreamConfiguration,
    kAudioObjectPropertyElementMaster, kAudioObjectPropertyScopeWildcard, AudioBufferList,
    AudioDeviceID, AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize,
    AudioObjectPropertyAddress, CFStringRef,
};

use crate::traits::Device;

use super::backend::CABackend;
use super::cf::{check_os_status, CFError, CFString};

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
        let property_addr = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceUID,
            mScope: kAudioDevicePropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMaster,
        };

        let mut input_device_uid = MaybeUninit::<CFStringRef>::uninit();
        let mut size = std::mem::size_of::<CFStringRef>() as u32;
        unsafe {
            check_os_status(AudioObjectGetPropertyData(
                self.0,
                &property_addr,
                0,
                std::ptr::null(),
                &mut size,
                input_device_uid.as_mut_ptr() as *mut c_void,
            ))?;
            Ok(CFString::new_retained(input_device_uid.assume_init()))
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
        let property_addr = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: kAudioDevicePropertyScopeInput,
            mElement: kAudioObjectPropertyElementMaster,
        };

        let mut size = 0;
        unsafe {
            check_os_status(AudioObjectGetPropertyDataSize(
                self.0,
                &property_addr,
                0,
                std::ptr::null(),
                &mut size,
            ))?;
        }

        Ok(size as usize / std::mem::size_of::<AudioBufferList>())
    }

    fn num_outputs(&self) -> Result<usize, CFError> {
        let property_addr = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: kAudioDevicePropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMaster,
        };

        let mut size = 0;
        unsafe {
            check_os_status(AudioObjectGetPropertyDataSize(
                self.0,
                &property_addr,
                0,
                std::ptr::null(),
                &mut size,
            ))?;
        }

        Ok(size as usize / std::mem::size_of::<AudioBufferList>())
    }

    fn name(&self) -> Result<String, CFError> {
        let property_addr = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceName,
            mScope: kAudioObjectPropertyScopeWildcard,
            mElement: kAudioObjectPropertyElementMaster,
        };

        let mut name_len = 64;
        let mut name = String::with_capacity(name_len as usize);
        unsafe {
            check_os_status(AudioObjectGetPropertyData(
                self.0,
                &property_addr,
                0,
                std::ptr::null(),
                &mut name_len,
                name.as_mut_ptr() as *mut _,
            ))?;
            name.as_mut_vec().set_len(name_len as usize - 1);
        }

        Ok(name)
    }
}
