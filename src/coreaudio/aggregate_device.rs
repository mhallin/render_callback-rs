use std::ffi::{c_void, CStr};
use std::fmt;
use std::mem::MaybeUninit;

use coreaudio_sys::{
    kAudioAggregateDeviceIsPrivateKey, kAudioAggregateDeviceNameKey,
    kAudioAggregateDevicePropertyFullSubDeviceList, kAudioAggregateDeviceUIDKey,
    kAudioHardwarePropertyPlugInForBundleID, kAudioObjectPropertyElementMaster,
    kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject, kAudioPlugInCreateAggregateDevice,
    kAudioPlugInDestroyAggregateDevice, AudioDeviceID, AudioObjectGetPropertyData, AudioObjectID,
    AudioObjectPropertyAddress, AudioObjectSetPropertyData, AudioValueTranslation,
    CFMutableArrayRef, CFMutableDictionaryRef, CFStringRef,
};

use crate::traits::Backend;

use super::backend::CABackend;
use super::cf::{
    check_os_status, CFError, CFMutableArray, CFMutableDictionary, CFNumber, CFString,
};
use super::device::CADevice;

const AGGREGATE_DEVICE_UID: &str = "com.github.mhallin.Audioshop";

pub struct AggregateDevice {
    plugin_id: AudioObjectID,
    device: CADevice,
    input: CADevice,
    output: CADevice,
}

impl AggregateDevice {
    pub fn new(backend: &CABackend, input: CADevice, output: CADevice) -> Result<Self, CFError> {
        let audio_plugin_id = get_audio_plugin_id()?;

        let device = match find_existing_aggregate_device(backend)? {
            Some(device) => device,
            None => create_aggregate_device(audio_plugin_id)?,
        };

        let aggregate_device = AggregateDevice {
            plugin_id: audio_plugin_id,
            device,
            input,
            output,
        };

        aggregate_device.refresh_sub_device_array()?;

        Ok(aggregate_device)
    }

    pub fn device(&self) -> CADevice {
        self.device
    }

    pub fn input(&self) -> CADevice {
        self.input
    }

    pub fn output(&self) -> CADevice {
        self.output
    }

    pub fn set_input(&mut self, input: CADevice) -> Result<(), CFError> {
        self.input = input;
        self.refresh_sub_device_array()
    }

    pub fn set_output(&mut self, output: CADevice) -> Result<(), CFError> {
        self.output = output;
        self.refresh_sub_device_array()
    }

    fn refresh_sub_device_array(&self) -> Result<(), CFError> {
        let sub_device_array = {
            let mut array = CFMutableArray::new();
            array.push(self.input.uid()?.as_void_ptr());

            if self.input != self.output {
                array.push(self.output.uid()?.as_void_ptr());
            }
            array
        };

        let property_addr = AudioObjectPropertyAddress {
            mSelector: kAudioAggregateDevicePropertyFullSubDeviceList,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMaster,
        };

        unsafe {
            check_os_status(AudioObjectSetPropertyData(
                self.device.id(),
                &property_addr,
                0,
                std::ptr::null(),
                std::mem::size_of::<CFMutableArrayRef>() as u32,
                (&sub_device_array.as_void_ptr() as *const _) as *mut c_void,
            ))?;
        }

        Ok(())
    }
}

fn get_audio_plugin_id() -> Result<AudioObjectID, CFError> {
    let bundle_name = CFString::new("com.apple.audio.CoreAudio");

    let mut object_id = MaybeUninit::<AudioObjectID>::uninit();

    let mut translation = AudioValueTranslation {
        mInputData: (&bundle_name.as_void_ptr() as *const _) as *mut c_void,
        mInputDataSize: std::mem::size_of::<CFStringRef>() as u32,
        mOutputData: object_id.as_mut_ptr() as *mut c_void,
        mOutputDataSize: std::mem::size_of::<AudioObjectID>() as u32,
    };

    let property_addr = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyPlugInForBundleID,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

    let mut size = std::mem::size_of::<AudioValueTranslation>() as u32;

    unsafe {
        check_os_status(AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &property_addr,
            0,
            std::ptr::null(),
            &mut size,
            &mut translation as *mut AudioValueTranslation as *mut c_void,
        ))?;

        Ok(object_id.assume_init())
    }
}

impl Drop for AggregateDevice {
    fn drop(&mut self) {
        unsafe {
            let property_addr = AudioObjectPropertyAddress {
                mSelector: kAudioPlugInDestroyAggregateDevice,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMaster,
            };

            let mut size = std::mem::size_of::<AudioDeviceID>() as u32;
            let mut device_id = self.device.id();
            check_os_status(AudioObjectGetPropertyData(
                self.plugin_id,
                &property_addr,
                0,
                std::ptr::null(),
                &mut size,
                &mut device_id as *mut AudioDeviceID as *mut c_void,
            ))
            .expect("Could not destroy aggregate device");
        }
    }
}

impl fmt::Debug for AggregateDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AggregateDevice")
            .field("aggregate_device_id", &self.device)
            .field("input", &self.input)
            .field("output", &self.output)
            .finish()
    }
}

fn find_existing_aggregate_device(backend: &CABackend) -> Result<Option<CADevice>, CFError> {
    for device in backend.all_devices()? {
        if device.uid()?.to_string() == AGGREGATE_DEVICE_UID {
            return Ok(Some(device));
        }
    }

    Ok(None)
}

fn create_aggregate_device(audio_plugin_id: AudioObjectID) -> Result<CADevice, CFError> {
    let mut aggregate_dict = CFMutableDictionary::new();
    aggregate_dict.insert(
        CFString::from_cstr(&CStr::from_bytes_with_nul(kAudioAggregateDeviceNameKey).unwrap())
            .as_void_ptr(),
        CFString::new("Audioshop aggregate device").as_void_ptr(),
    );

    aggregate_dict.insert(
        CFString::from_cstr(&CStr::from_bytes_with_nul(kAudioAggregateDeviceUIDKey).unwrap())
            .as_void_ptr(),
        CFString::new(AGGREGATE_DEVICE_UID).as_void_ptr(),
    );

    aggregate_dict.insert(
        CFString::from_cstr(&CStr::from_bytes_with_nul(kAudioAggregateDeviceIsPrivateKey).unwrap())
            .as_void_ptr(),
        CFNumber::new(1).as_void_ptr(),
    );

    let aggregate_audio_device_id = {
        let property_addr = AudioObjectPropertyAddress {
            mSelector: kAudioPlugInCreateAggregateDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMaster,
        };

        let mut aggregate_audio_device = MaybeUninit::<AudioDeviceID>::uninit();
        let mut size = std::mem::size_of::<AudioDeviceID>() as u32;
        let aggregate_dict_ptr = aggregate_dict.as_void_ptr();

        unsafe {
            check_os_status(AudioObjectGetPropertyData(
                audio_plugin_id,
                &property_addr,
                std::mem::size_of::<CFMutableDictionaryRef>() as u32,
                &aggregate_dict_ptr as *const _ as *mut c_void,
                &mut size,
                aggregate_audio_device.as_mut_ptr() as *mut c_void,
            ))?;

            aggregate_audio_device.assume_init()
        }
    };

    Ok(CADevice::new(aggregate_audio_device_id))
}
