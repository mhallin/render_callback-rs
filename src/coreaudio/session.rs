use std::ffi::c_void;

use coreaudio_sys::{
    noErr, AudioBuffer, AudioBufferList, AudioDeviceCreateIOProcID, AudioDeviceDestroyIOProcID,
    AudioDeviceID, AudioDeviceIOProcID, AudioDeviceStart, AudioDeviceStop, AudioTimeStamp,
    OSStatus,
};

use crate::traits::{AudioBuffers, Session};

use super::aggregate_device::AggregateDevice;
use super::backend::CABackend;
use super::cf::{check_os_status, CFError};
use super::device::CADevice;

pub type RenderCallback = dyn FnMut(&[InterleavedBuffer], &mut [InterleavedBuffer]) + Send;

pub struct CASession {
    device: AggregateDevice,
    callback: Option<(AudioDeviceIOProcID, Box<RenderCallback>)>,
}

impl CASession {
    pub fn new_started(
        backend: &CABackend,
        input_device: CADevice,
        output_device: CADevice,
        callback: Box<RenderCallback>,
    ) -> Result<Box<Self>, CFError> {
        let aggregate_device = AggregateDevice::new(backend, input_device, output_device)?;
        let device = aggregate_device.device();
        let mut session = Box::new(CASession {
            device: aggregate_device,
            callback: None,
        });

        let mut proc_id = std::mem::MaybeUninit::<AudioDeviceIOProcID>::uninit();
        unsafe {
            check_os_status(AudioDeviceCreateIOProcID(
                device.id(),
                Some(session_io_proc),
                session.as_mut() as *mut CASession as *mut c_void,
                proc_id.as_mut_ptr(),
            ))?;

            let proc_id = proc_id.assume_init();
            session.callback = Some((proc_id, callback));

            check_os_status(AudioDeviceStart(device.id(), proc_id))?;
        }

        Ok(session)
    }

    pub fn aggregate_device(&self) -> &AggregateDevice {
        &self.device
    }

    pub fn aggregate_device_mut(&mut self) -> &mut AggregateDevice {
        &mut self.device
    }
}

impl Drop for CASession {
    fn drop(&mut self) {
        if let Some((proc_id, _)) = &mut self.callback {
            unsafe {
                check_os_status(AudioDeviceStop(self.device.device().id(), *proc_id))
                    .expect("Could not stop session");
                check_os_status(AudioDeviceDestroyIOProcID(
                    self.device.device().id(),
                    *proc_id,
                ))
                .expect("Could not destroy IOProcID");
            }
        }
    }
}

unsafe extern "C" fn session_io_proc(
    _in_device: AudioDeviceID,
    _in_now: *const AudioTimeStamp,
    in_input_data: *const AudioBufferList,
    _in_input_time: *const AudioTimeStamp,
    out_output_data: *mut AudioBufferList,
    _in_output_time: *const AudioTimeStamp,
    in_client_data: *mut c_void,
) -> OSStatus {
    let session_ptr = in_client_data as *mut CASession;
    if let (Some(session), Some(in_input_data), Some(out_output_data)) = (
        session_ptr.as_mut(),
        in_input_data.as_ref(),
        out_output_data.as_mut(),
    ) {
        if let Some((_, callback)) = &mut session.callback {
            let input_buffers = {
                let ptr = in_input_data.mBuffers.as_ptr() as *const InterleavedBuffer;
                let len = in_input_data.mNumberBuffers as usize;

                std::slice::from_raw_parts(ptr, len)
            };

            let output_buffers = {
                let ptr = out_output_data.mBuffers.as_ptr() as *mut InterleavedBuffer;
                let len = out_output_data.mNumberBuffers as usize;

                std::slice::from_raw_parts_mut(ptr, len)
            };

            callback(input_buffers, output_buffers);
        }
    }

    noErr as OSStatus
}

impl Session<CABackend> for Box<CASession> {
    fn input_device(&self) -> Result<CADevice, CFError> {
        Ok(self.aggregate_device().input())
    }

    fn output_device(&self) -> Result<CADevice, CFError> {
        Ok(self.aggregate_device().output())
    }

    fn set_input_device(&mut self, device: CADevice) -> Result<(), CFError> {
        self.aggregate_device_mut().set_input(device)
    }

    fn set_output_device(&mut self, device: CADevice) -> Result<(), CFError> {
        self.aggregate_device_mut().set_output(device)
    }
}

pub struct InterleavedBuffer(AudioBuffer);

impl AudioBuffers for InterleavedBuffer {
    fn num_frames(&self) -> usize {
        (self.0.mDataByteSize / (4 * self.0.mNumberChannels)) as usize
    }

    fn num_channels(&self) -> usize {
        self.0.mNumberChannels as usize
    }

    fn interleaved_frames(&self) -> &[f32] {
        let ptr = self.0.mData as *const f32;
        let len = self.num_frames() * self.num_channels();

        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    fn interleaved_frames_mut(&mut self) -> &mut [f32] {
        let ptr = self.0.mData as *mut f32;
        let len = self.num_frames() * self.num_channels();

        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }
}
