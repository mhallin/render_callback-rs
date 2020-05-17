use std::ffi::c_void;
use std::{alloc, mem, ptr};

use super::cf::{check_os_status, CFArray, CFDictionary, CFError, CFString};
use super::device::CADevice;

use coreaudio_sys::{
    AudioDeviceID, AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectID,
    AudioObjectPropertyAddress, AudioObjectPropertyElement, AudioObjectPropertyScope,
    AudioObjectPropertySelector, AudioObjectSetPropertyData, AudioValueTranslation,
};

pub trait Element {
    fn element() -> AudioObjectPropertyElement;
}

pub trait Scope {
    fn scope() -> AudioObjectPropertyScope;
}

pub trait GettablePropertyType: Sized {
    fn get(obj: AudioObjectID, addr: AudioObjectPropertyAddress) -> Result<Self, CFError>;
}

pub trait SettablePropertyType: Sized {
    fn set(
        obj: AudioObjectID,
        addr: AudioObjectPropertyAddress,
        value: &Self,
    ) -> Result<(), CFError>;
}

pub trait TranslatablePropertyType: Sized {
    fn translate(
        obj: AudioObjectID,
        addr: AudioObjectPropertyAddress,
        value: &mut Self,
    ) -> Result<(), CFError>;
}

pub trait QualifiedGettablePropertyType<TInput>: Sized {
    fn get_qualified(
        obj: AudioObjectID,
        addr: AudioObjectPropertyAddress,
        qualifier: &TInput,
    ) -> Result<Self, CFError>;
}

pub trait Selector {
    type Type;

    fn selector() -> AudioObjectPropertySelector;
}

pub fn get<El: Element, Sc: Scope, Se: Selector>(
    _element: El,
    _scope: Sc,
    _selector: Se,
    obj: AudioObjectID,
) -> Result<Se::Type, CFError>
where
    Se::Type: GettablePropertyType,
{
    Se::Type::get(
        obj,
        AudioObjectPropertyAddress {
            mElement: El::element(),
            mScope: Sc::scope(),
            mSelector: Se::selector(),
        },
    )
}

pub fn get_qualified<El: Element, Sc: Scope, Se: Selector, TInput>(
    _element: El,
    _scope: Sc,
    _selector: Se,
    qualifier: &TInput,
    obj: AudioObjectID,
) -> Result<Se::Type, CFError>
where
    Se::Type: QualifiedGettablePropertyType<TInput>,
{
    Se::Type::get_qualified(
        obj,
        AudioObjectPropertyAddress {
            mElement: El::element(),
            mScope: Sc::scope(),
            mSelector: Se::selector(),
        },
        qualifier,
    )
}

pub fn set<El: Element, Sc: Scope, Se: Selector>(
    _element: El,
    _scope: Sc,
    _selector: Se,
    obj: AudioObjectID,
    value: &Se::Type,
) -> Result<(), CFError>
where
    Se::Type: SettablePropertyType,
{
    Se::Type::set(
        obj,
        AudioObjectPropertyAddress {
            mElement: El::element(),
            mScope: Sc::scope(),
            mSelector: Se::selector(),
        },
        value,
    )
}

pub fn translate<El: Element, Sc: Scope, Se: Selector>(
    _element: El,
    _scope: Sc,
    _selector: Se,
    obj: AudioObjectID,
    value: &mut Se::Type,
) -> Result<(), CFError>
where
    Se::Type: TranslatablePropertyType,
{
    Se::Type::translate(
        obj,
        AudioObjectPropertyAddress {
            mElement: El::element(),
            mScope: Sc::scope(),
            mSelector: Se::selector(),
        },
        value,
    )
}

pub mod element {
    use coreaudio_sys::*;

    use super::Element;

    /// The AudioObjectPropertyElement value for properties that apply to the
    /// master element or to the entire scope.    
    pub struct Master;

    impl Element for Master {
        fn element() -> AudioObjectPropertyElement {
            kAudioObjectPropertyElementMaster
        }
    }
}

pub mod scope {
    use coreaudio_sys::*;

    use super::Scope;

    /// The AudioObjectPropertyScope for properties that apply to the object as
    /// a whole. All objects have a global scope and for most it is their only
    /// scope.    
    pub struct Global;
    impl Scope for Global {
        fn scope() -> AudioObjectPropertyScope {
            kAudioObjectPropertyScopeGlobal
        }
    }

    /// The AudioObjectPropertyScope for properties that apply to the input side
    /// of an object.
    pub struct Input;
    impl Scope for Input {
        fn scope() -> AudioObjectPropertyScope {
            kAudioObjectPropertyScopeInput
        }
    }

    /// The AudioObjectPropertyScope for properties that apply to the output
    /// side of an object.    
    pub struct Output;
    impl Scope for Output {
        fn scope() -> AudioObjectPropertyScope {
            kAudioObjectPropertyScopeOutput
        }
    }

    /// The wildcard value for AudioObjectPropertyScopes.
    pub struct Wildcard;
    impl Scope for Wildcard {
        fn scope() -> AudioObjectPropertyScope {
            kAudioObjectPropertyScopeWildcard
        }
    }
}

pub mod selector {
    use coreaudio_sys::*;

    use super::{CADevice, CFArray, CFString, Selector};

    /// An array of the AudioObjectIDs that represent all the devices currently
    /// available to the system.
    pub struct HardwarePropertyDevices;
    impl Selector for HardwarePropertyDevices {
        type Type = Vec<CADevice>;

        fn selector() -> AudioObjectPropertySelector {
            kAudioHardwarePropertyDevices
        }
    }

    /// The AudioObjectID of the default input AudioDevice.
    pub struct HardwarePropertyDefaultInputDevice;
    impl Selector for HardwarePropertyDefaultInputDevice {
        type Type = CADevice;

        fn selector() -> AudioObjectPropertySelector {
            kAudioHardwarePropertyDefaultInputDevice
        }
    }

    /// The AudioObjectID of the default output AudioDevice.
    pub struct HardwarePropertyDefaultOutputDevice;
    impl Selector for HardwarePropertyDefaultOutputDevice {
        type Type = CADevice;

        fn selector() -> AudioObjectPropertySelector {
            kAudioHardwarePropertyDefaultOutputDevice
        }
    }

    /// Using an AudioValueTranslation structure, this property translates the
    /// input CFString containing a bundle ID into the AudioObjectID of the
    /// AudioPlugIn that corresponds to it. This property will return
    /// kAudioObjectUnkown if the given bundle ID doesn't match any
    /// AudioPlugIns.
    pub struct HardwarePropertyPlugInForBundleID;
    impl Selector for HardwarePropertyPlugInForBundleID {
        type Type = AudioValueTranslation;

        fn selector() -> AudioObjectPropertySelector {
            kAudioHardwarePropertyPlugInForBundleID
        }
    }

    /// A CFArray of CFStrings that contain the UIDs of all the devices, active
    /// or inactive, contained in the AudioAggregateDevice. The order of the
    /// items in the array is significant and is used to determine the order of
    /// the streams of the AudioAggregateDevice. The caller is responsible for
    /// releasing the returned CFObject.    
    pub struct AggregateDevicePropertyFullSubDeviceList;
    impl Selector for AggregateDevicePropertyFullSubDeviceList {
        type Type = CFArray;

        fn selector() -> AudioObjectPropertySelector {
            kAudioAggregateDevicePropertyFullSubDeviceList
        }
    }

    /// This property is used to tell a plug-in to destroy an
    /// AudioAggregateDevice. Like kAudioPlugInCreateAggregateDevice, this
    /// property is read only. The value of the property is the AudioObjectID of
    /// the AudioAggregateDevice to destroy.
    pub struct PlugInDestroyAggregateDevice;
    impl Selector for PlugInDestroyAggregateDevice {
        type Type = CADevice;

        fn selector() -> AudioObjectPropertySelector {
            kAudioPlugInDestroyAggregateDevice
        }
    }

    /// This property is used to tell a plug-in to create a new
    /// AudioAggregateDevice. Its value is only read. The qualifier data for
    /// this property is a CFDictionary containing a description of the
    /// AudioAggregateDevice to create. The keys for the CFDictionary are
    /// defined in the AudioAggregateDevice Constants section. The value of the
    /// property that gets returned is the AudioObjectID of the newly created
    /// device.
    pub struct PlugInCreateAggregateDevice;
    impl Selector for PlugInCreateAggregateDevice {
        type Type = CADevice;

        fn selector() -> AudioObjectPropertySelector {
            kAudioPlugInCreateAggregateDevice
        }
    }

    /// A CFString that contains a persistent identifier for the AudioDevice. An
    /// AudioDevice's UID is persistent across boots. The content of the UID
    /// string is a black box and may contain information that is unique to a
    /// particular instance of an AudioDevice's hardware or unique to the CPU.
    /// Therefore they are not suitable for passing between CPUs or for
    /// identifying similar models of hardware. The caller is responsible for
    /// releasing the returned CFObject.    
    pub struct DevicePropertyDeviceUID;
    impl Selector for DevicePropertyDeviceUID {
        type Type = CFString;

        fn selector() -> AudioObjectPropertySelector {
            kAudioDevicePropertyDeviceUID
        }
    }

    /// This property returns the stream configuration of the device in an
    /// AudioBufferList (with the buffer pointers set to NULL) which describes
    /// the list of streams and the number of channels in each stream. This
    /// corresponds to what will be passed into the IOProc.    
    pub struct DevicePropertyStreamConfiguration;
    impl Selector for DevicePropertyStreamConfiguration {
        type Type = Box<AudioBufferList>;

        fn selector() -> AudioObjectPropertySelector {
            kAudioDevicePropertyStreamConfiguration
        }
    }

    /// A CFString that contains the human readable name of the object. The
    /// caller is responsible for releasing the returned CFObject.    
    pub struct ObjectPropertyName;
    impl Selector for ObjectPropertyName {
        type Type = CFString;

        fn selector() -> AudioObjectPropertySelector {
            kAudioObjectPropertyName
        }
    }

    /// A Float64 that indicates the current nominal sample rate of the
    /// AudioDevice.
    pub struct DevicePropertyNominalSampleRate;
    impl Selector for DevicePropertyNominalSampleRate {
        type Type = f64;

        fn selector() -> AudioObjectPropertySelector {
            kAudioDevicePropertyNominalSampleRate
        }
    }

    /// A Float64 that indicates the current actual sample rate of the
    /// AudioDevice as measured by its time stamps.    
    pub struct DevicePropertyActualSampleRate;
    impl Selector for DevicePropertyActualSampleRate {
        type Type = f64;

        fn selector() -> AudioObjectPropertySelector {
            kAudioDevicePropertyActualSampleRate
        }
    }
}

impl GettablePropertyType for f64 {
    fn get(obj: AudioObjectID, addr: AudioObjectPropertyAddress) -> Result<Self, CFError> {
        unsafe {
            let mut value = mem::MaybeUninit::<f64>::uninit();
            let mut size = mem::size_of::<Self>() as u32;

            check_os_status(AudioObjectGetPropertyData(
                obj,
                &addr,
                0,
                ptr::null(),
                &mut size,
                value.as_mut_ptr() as *mut c_void,
            ))?;

            Ok(value.assume_init())
        }
    }
}

impl SettablePropertyType for f64 {
    fn set(
        obj: AudioObjectID,
        addr: AudioObjectPropertyAddress,
        value: &Self,
    ) -> Result<(), CFError> {
        unsafe {
            let size = mem::size_of::<Self>() as u32;

            check_os_status(AudioObjectSetPropertyData(
                obj,
                &addr,
                0,
                ptr::null(),
                size,
                value as *const Self as *const c_void,
            ))
        }
    }
}

impl GettablePropertyType for Vec<CADevice> {
    fn get(obj: AudioObjectID, addr: AudioObjectPropertyAddress) -> Result<Self, CFError> {
        unsafe {
            let mut devices_size = 0;
            check_os_status(AudioObjectGetPropertyDataSize(
                obj,
                &addr,
                0,
                ptr::null(),
                &mut devices_size,
            ))?;

            let mut device_ids =
                vec![CADevice::uninit(); devices_size as usize / mem::size_of::<CADevice>()];

            check_os_status(AudioObjectGetPropertyData(
                obj,
                &addr,
                0,
                ptr::null(),
                &mut devices_size,
                device_ids.as_mut_ptr() as *mut _,
            ))?;

            Ok(device_ids)
        }
    }
}

impl GettablePropertyType for CADevice {
    fn get(obj: AudioObjectID, addr: AudioObjectPropertyAddress) -> Result<Self, CFError> {
        unsafe {
            let mut device_id = mem::MaybeUninit::<AudioDeviceID>::uninit();
            let mut size = mem::size_of::<AudioDeviceID>() as u32;
            check_os_status(AudioObjectGetPropertyData(
                obj,
                &addr,
                0,
                ptr::null(),
                &mut size,
                device_id.as_mut_ptr() as *mut c_void,
            ))?;
            Ok(CADevice(device_id.assume_init()))
        }
    }
}

impl TranslatablePropertyType for CADevice {
    fn translate(
        obj: AudioObjectID,
        addr: AudioObjectPropertyAddress,
        value: &mut Self,
    ) -> Result<(), CFError> {
        unsafe {
            let mut size = mem::size_of::<AudioDeviceID>() as u32;
            check_os_status(AudioObjectGetPropertyData(
                obj,
                &addr,
                0,
                ptr::null(),
                &mut size,
                value as *mut Self as *mut c_void,
            ))
        }
    }
}

impl QualifiedGettablePropertyType<CFDictionary> for CADevice {
    fn get_qualified(
        obj: AudioObjectID,
        addr: AudioObjectPropertyAddress,
        qualifier: &CFDictionary,
    ) -> Result<Self, CFError> {
        use coreaudio_sys::CFDictionaryRef;

        unsafe {
            let aggregate_dict_ptr = qualifier.as_void_ptr();
            let mut device_id = mem::MaybeUninit::<AudioDeviceID>::uninit();

            let mut size = mem::size_of::<AudioDeviceID>() as u32;
            check_os_status(AudioObjectGetPropertyData(
                obj,
                &addr,
                std::mem::size_of::<CFDictionaryRef>() as u32,
                &aggregate_dict_ptr as *const _ as *mut c_void,
                &mut size,
                device_id.as_mut_ptr() as *mut c_void,
            ))?;

            Ok(CADevice(device_id.assume_init()))
        }
    }
}

impl GettablePropertyType for CFArray {
    fn get(obj: AudioObjectID, addr: AudioObjectPropertyAddress) -> Result<Self, CFError> {
        use coreaudio_sys::CFArrayRef;

        unsafe {
            let mut array = mem::MaybeUninit::<CFArrayRef>::uninit();
            let mut size = mem::size_of::<CFArrayRef>() as u32;
            check_os_status(AudioObjectGetPropertyData(
                obj,
                &addr,
                0,
                ptr::null(),
                &mut size,
                array.as_mut_ptr() as *mut c_void,
            ))?;
            Ok(CFArray::new_retained(array.assume_init()))
        }
    }
}

impl SettablePropertyType for CFArray {
    fn set(
        obj: AudioObjectID,
        addr: AudioObjectPropertyAddress,
        value: &Self,
    ) -> Result<(), CFError> {
        unsafe {
            check_os_status(AudioObjectSetPropertyData(
                obj,
                &addr,
                0,
                std::ptr::null(),
                std::mem::size_of::<Self>() as u32,
                (&value.as_void_ptr() as *const _) as *mut c_void,
            ))
        }
    }
}

impl TranslatablePropertyType for AudioValueTranslation {
    fn translate(
        obj: AudioObjectID,
        addr: AudioObjectPropertyAddress,
        value: &mut Self,
    ) -> Result<(), CFError> {
        let mut size = std::mem::size_of::<AudioValueTranslation>() as u32;

        unsafe {
            check_os_status(AudioObjectGetPropertyData(
                obj,
                &addr,
                0,
                std::ptr::null(),
                &mut size,
                value as *mut AudioValueTranslation as *mut c_void,
            ))?;
        }
        Ok(())
    }
}

impl GettablePropertyType for CFString {
    fn get(obj: AudioObjectID, addr: AudioObjectPropertyAddress) -> Result<Self, CFError> {
        use coreaudio_sys::CFStringRef;
        unsafe {
            let mut value = mem::MaybeUninit::<CFStringRef>::uninit();
            let mut size = mem::size_of::<CFStringRef>() as u32;
            check_os_status(AudioObjectGetPropertyData(
                obj,
                &addr,
                0,
                std::ptr::null(),
                &mut size,
                value.as_mut_ptr() as *mut c_void,
            ))?;
            Ok(CFString::new_retained(value.assume_init()))
        }
    }
}

impl GettablePropertyType for Box<coreaudio_sys::AudioBufferList> {
    fn get(obj: AudioObjectID, addr: AudioObjectPropertyAddress) -> Result<Self, CFError> {
        use coreaudio_sys::AudioBufferList;
        unsafe {
            let mut size = 0;
            check_os_status(AudioObjectGetPropertyDataSize(
                obj,
                &addr,
                0,
                std::ptr::null(),
                &mut size,
            ))?;

            let layout = alloc::Layout::from_size_align_unchecked(
                (size as usize).max(mem::size_of::<AudioBufferList>()),
                mem::align_of::<AudioBufferList>(),
            );
            let buffer = alloc::alloc(layout);

            check_os_status(AudioObjectGetPropertyData(
                obj,
                &addr,
                0,
                std::ptr::null(),
                &mut size,
                buffer as *mut c_void,
            ))?;

            Ok(Box::from_raw(buffer as *mut AudioBufferList))
        }
    }
}
