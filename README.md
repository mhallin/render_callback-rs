# Just give me a render callback

For when you just want to read or write data from or to some audio hardware.

```rust
use render_callback::{AudioBuffers, Backend, CurrentPlatformBackend};

fn main() {
    let backend = CurrentPlatformBackend::new().unwrap();

    let mut session = {
        let input_device = backend.default_input_device().unwrap();
        let output_device = backend.default_output_device().unwrap();
        // Or, use backend.all_devices() to iterate over available devices and pick the ones you want

        backend.start_session(
            44100.0, // Sample rate
            input_device,
            output_device,
            Box::new(|input, output| {
                // Just a &[f32] with one sample per input channel interleaved
                let interleaved_inputs = input.interleaved_frames();
                let num_input_channels = input.num_channels();
                let num_input_frames = input.num_frames(); // interleaved_inputs.len() / num_input_channels

                // Just a &mut [f32] with one sample per output channel interleaved
                let interleaved_outputs = output.interleaved_frames_mut();
                let num_output_channels = output.num_channels();
                let num_output_frames = output.num_frames(); // interleaved_outputs.len() / num_output_channels

                // The API guarantees that there are an equal amount of input frames
                // and output frames, but the channel counts might differ.
                assert_eq!(num_input_frames, num_output_frames);

                // Do stuff with them
            })
        ).unwrap()
    };

    // Do other stuff in your application. Maybe set up an MPSC channel to
    // communicate with the real-time thread?
    
    // Audio will stop when you drop the session object, so make sure to keep it
    // alive as long as you need.
}
```

Look at [src/traits.rs](src/traits.rs) for the complete API.
