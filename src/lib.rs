mod coreaudio;
mod traits;

pub use coreaudio::Backend as CurrentPlatformBackend;
pub use traits::*;
