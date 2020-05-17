use std::error::Error;
use std::ffi::{c_void, CStr};
use std::fmt;

use coreaudio_sys::{
    kCFNumberIntType, kCFStringEncodingUTF8, kCFTypeArrayCallBacks, kCFTypeDictionaryKeyCallBacks,
    kCFTypeDictionaryValueCallBacks, noErr, CFArrayAppendValue, CFArrayCreateMutable, CFArrayRef,
    CFDataGetBytes, CFDataGetLength, CFDataRef, CFDictionaryAddValue, CFDictionaryCreateMutable,
    CFDictionaryRef, CFMutableArrayRef, CFMutableDictionaryRef, CFNumberCreate, CFNumberRef,
    CFRange, CFRelease, CFRetain, CFStringCreateExternalRepresentation, CFStringCreateWithBytes,
    CFStringCreateWithCString, CFStringGetSystemEncoding, CFStringRef, OSStatus,
};

#[derive(Debug)]
pub struct CFError(OSStatus);

pub struct CFString(CFStringRef);
pub struct CFDictionary(CFDictionaryRef);
pub struct CFMutableDictionary(CFMutableDictionaryRef);
pub struct CFNumber(CFNumberRef);
pub struct CFArray(CFArrayRef);
pub struct CFMutableArray(CFMutableArrayRef);
pub struct CFData(CFDataRef);

pub fn check_os_status(s: OSStatus) -> Result<(), CFError> {
    if s == noErr as OSStatus {
        Ok(())
    } else {
        Err(CFError(s))
    }
}

impl fmt::Display for CFError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OSStatus({:x})", self.0)
    }
}

impl Error for CFError {}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {}

impl CFString {
    pub fn new_retained(s: CFStringRef) -> Self {
        CFString(s)
    }

    pub fn new(s: &str) -> Self {
        unsafe {
            CFString(CFStringCreateWithBytes(
                std::ptr::null_mut(),
                s.as_ptr(),
                s.len() as i64,
                kCFStringEncodingUTF8,
                0,
            ))
        }
    }

    pub fn from_cstr(s: &CStr) -> Self {
        unsafe {
            CFString(CFStringCreateWithCString(
                std::ptr::null_mut(),
                s.as_ptr(),
                CFStringGetSystemEncoding(),
            ))
        }
    }

    pub fn as_void_ptr(&self) -> *const c_void {
        self.0 as *const c_void
    }

    pub fn to_string(&self) -> String {
        let data_ref = unsafe {
            CFStringCreateExternalRepresentation(std::ptr::null(), self.0, kCFStringEncodingUTF8, 0)
        };

        assert!(!data_ref.is_null());

        let data = CFData(data_ref);

        String::from_utf8(data.to_vec()).expect("Invalid UTF-8")
    }
}

impl Drop for CFString {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0 as *const c_void);
        }
    }
}

impl CFDictionary {
    pub fn as_void_ptr(&self) -> *const c_void {
        self.0 as *const c_void
    }
}

impl Drop for CFDictionary {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0 as *const c_void);
        }
    }
}

impl CFMutableDictionary {
    pub fn new() -> Self {
        unsafe {
            CFMutableDictionary(CFDictionaryCreateMutable(
                std::ptr::null_mut(),
                0,
                &kCFTypeDictionaryKeyCallBacks,
                &kCFTypeDictionaryValueCallBacks,
            ))
        }
    }

    pub fn insert(&mut self, key: *const c_void, value: *const c_void) {
        unsafe { CFDictionaryAddValue(self.0, key, value) }
    }

    pub fn clone_immutable(&self) -> CFDictionary {
        unsafe { CFDictionary(CFRetain(self.0 as *const c_void) as CFDictionaryRef) }
    }
}

impl Drop for CFMutableDictionary {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0 as *const c_void);
        }
    }
}

impl CFNumber {
    pub fn new(value: i32) -> Self {
        unsafe {
            CFNumber(CFNumberCreate(
                std::ptr::null_mut(),
                kCFNumberIntType as i64,
                &value as *const i32 as *const c_void,
            ))
        }
    }

    pub fn as_void_ptr(&self) -> *const c_void {
        self.0 as *const c_void
    }
}

impl Drop for CFNumber {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0 as *const c_void);
        }
    }
}

impl CFArray {
    pub fn new_retained(a: CFArrayRef) -> Self {
        CFArray(a)
    }

    pub fn as_void_ptr(&self) -> *const c_void {
        self.0 as *const c_void
    }
}

impl Drop for CFArray {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0 as *const c_void);
        }
    }
}

impl CFMutableArray {
    pub fn new() -> Self {
        unsafe {
            CFMutableArray(CFArrayCreateMutable(
                std::ptr::null_mut(),
                0,
                &kCFTypeArrayCallBacks,
            ))
        }
    }

    pub fn push(&mut self, value: *const c_void) {
        unsafe { CFArrayAppendValue(self.0, value) };
    }

    pub fn clone_immutable(&self) -> CFArray {
        unsafe { CFArray(CFRetain(self.0 as *const c_void) as CFArrayRef) }
    }
}

impl Drop for CFMutableArray {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0 as *const c_void);
        }
    }
}

impl CFData {
    pub fn to_vec(&self) -> Vec<u8> {
        let len = unsafe { CFDataGetLength(self.0) };

        let mut vec = Vec::with_capacity(len as usize);

        unsafe {
            CFDataGetBytes(
                self.0,
                CFRange {
                    length: len,
                    location: 0,
                },
                vec.as_mut_ptr(),
            );
            vec.set_len(len as usize);
        }

        vec
    }
}

impl Drop for CFData {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0 as *const c_void);
        }
    }
}
