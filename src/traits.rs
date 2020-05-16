use std::error::Error;
use std::fmt::Debug;

pub type RenderCallback<B> =
    dyn FnMut(&[<B as Backend>::AudioBuffers], &mut [<B as Backend>::AudioBuffers]) + Send;

pub trait Backend: Sized {
    type Session: Session<Self>;
    type Device: Device<Self> + Debug + Clone;
    type Error: Error;
    type AudioBuffers: AudioBuffers;

    fn new() -> Result<Self, Self::Error>;

    fn all_devices(&self) -> Result<Vec<Self::Device>, Self::Error>;
    fn default_input_device(&self) -> Result<Self::Device, Self::Error>;
    fn default_output_device(&self) -> Result<Self::Device, Self::Error>;

    fn start_session(
        &self,
        input_device: Self::Device,
        output_device: Self::Device,
        callback: Box<RenderCallback<Self>>,
    ) -> Result<Self::Session, Self::Error>;
}

pub trait Session<B: Backend>: Sized {
    fn input_device(&self) -> Result<B::Device, B::Error>;
    fn output_device(&self) -> Result<B::Device, B::Error>;
    
    fn set_input_device(&mut self, device: B::Device) -> Result<(), B::Error>;
    fn set_output_device(&mut self, device: B::Device) -> Result<(), B::Error>;
}

pub trait Device<B: Backend> {
    fn num_inputs(&self) -> Result<usize, B::Error>;
    fn num_outputs(&self) -> Result<usize, B::Error>;
    fn name(&self) -> Result<String, B::Error>;
}

pub trait AudioBuffers {
    fn num_frames(&self) -> usize;
    fn num_channels(&self) -> usize;

    fn interleaved_frames(&self) -> &[f32];
    fn interleaved_frames_mut(&mut self) -> &mut [f32];
}
