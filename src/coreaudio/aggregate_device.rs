use std::ffi::{c_void, CStr};
use std::fmt;
use std::mem::MaybeUninit;

use coreaudio_sys::{
    kAudioAggregateDeviceIsPrivateKey, kAudioAggregateDeviceNameKey, kAudioAggregateDeviceUIDKey,
    kAudioObjectSystemObject, AudioObjectID, AudioValueTranslation, CFStringRef,
};

use crate::traits::Backend;

use super::backend::CABackend;
use super::cf::{CFError, CFMutableArray, CFMutableDictionary, CFNumber, CFString};
use super::device::CADevice;
use super::properties::{self, element, scope, selector};

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

        properties::set(
            element::Master,
            scope::Global,
            selector::AggregateDevicePropertyFullSubDeviceList,
            self.device.id(),
            &sub_device_array.clone_immutable(),
        )
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

    properties::translate(
        element::Master,
        scope::Global,
        selector::HardwarePropertyPlugInForBundleID,
        kAudioObjectSystemObject,
        &mut translation,
    )?;

    unsafe { Ok(object_id.assume_init()) }
}

impl Drop for AggregateDevice {
    fn drop(&mut self) {
        properties::translate(
            element::Master,
            scope::Global,
            selector::PlugInDestroyAggregateDevice,
            self.plugin_id,
            &mut self.device,
        )
        .expect("Could not destroy aggregate device");
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

    properties::get_qualified(
        element::Master,
        scope::Global,
        selector::PlugInCreateAggregateDevice,
        &aggregate_dict.clone_immutable(),
        audio_plugin_id,
    )
}
