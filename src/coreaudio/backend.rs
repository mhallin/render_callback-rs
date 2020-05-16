use std::ffi::c_void;
use std::mem::MaybeUninit;

use coreaudio_sys::{
    kAudioHardwarePropertyDefaultInputDevice, kAudioHardwarePropertyDefaultOutputDevice,
    kAudioHardwarePropertyDevices, kAudioObjectPropertyElementMaster,
    kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyScopeWildcard, kAudioObjectSystemObject,
    AudioDeviceID, AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize,
    AudioObjectPropertyAddress,
};

use crate::traits::{Backend, RenderCallback};

use super::cf::{check_os_status, CFError};
use super::device::CADevice;
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
        let property_addr = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeWildcard,
            mElement: kAudioObjectPropertyElementMaster,
        };

        let mut devices_size = 0;
        unsafe {
            check_os_status(AudioObjectGetPropertyDataSize(
                kAudioObjectSystemObject,
                &property_addr,
                0,
                std::ptr::null(),
                &mut devices_size,
            ))?;
        }

        unsafe {
            let mut device_ids =
                vec![CADevice::uninit(); devices_size as usize / std::mem::size_of::<CADevice>()];

            check_os_status(AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_addr,
                0,
                std::ptr::null(),
                &mut devices_size,
                device_ids.as_mut_ptr() as *mut _,
            ))?;

            Ok(device_ids)
        }
    }

    fn default_input_device(&self) -> Result<CADevice, CFError> {
        let property_addr = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDefaultInputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMaster,
        };

        let mut device_id = MaybeUninit::<AudioDeviceID>::uninit();
        let mut size = std::mem::size_of::<AudioDeviceID>() as u32;
        unsafe {
            check_os_status(AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_addr,
                0,
                std::ptr::null(),
                &mut size,
                device_id.as_mut_ptr() as *mut c_void,
            ))?;
            Ok(CADevice(device_id.assume_init()))
        }
    }

    fn default_output_device(&self) -> Result<CADevice, CFError> {
        let property_addr = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDefaultOutputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMaster,
        };

        let mut device_id = MaybeUninit::<AudioDeviceID>::uninit();
        let mut size = std::mem::size_of::<AudioDeviceID>() as u32;
        unsafe {
            check_os_status(AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_addr,
                0,
                std::ptr::null(),
                &mut size,
                device_id.as_mut_ptr() as *mut c_void,
            ))?;
            Ok(CADevice(device_id.assume_init()))
        }
    }

    fn start_session(
        &self,
        input_device: Self::Device,
        output_device: Self::Device,
        callback: Box<RenderCallback<Self>>,
    ) -> Result<Self::Session, Self::Error> {
        CASession::new_started(self, input_device, output_device, callback)
    }
}
