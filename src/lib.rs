mod coreaudio;
mod traits;

pub use traits::*;

pub use coreaudio::Backend as CurrentPlatformBackend;

pub type CurrentPlatformSession = <CurrentPlatformBackend as traits::Backend>::Session;
pub type CurrentPlatformDevice = <CurrentPlatformBackend as traits::Backend>::Device;
pub type CurrentPlatformError = <CurrentPlatformBackend as traits::Backend>::Error;
pub type CurrentPlatformAudioBuffers = <CurrentPlatformBackend as traits::Backend>::AudioBuffers;
