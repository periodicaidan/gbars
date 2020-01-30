use portaudio as pa;
use portaudio::stream::{NonBlocking, Output, OutputCallbackArgs};
use classic::gb_types as gb;

pub enum WaveDuty {
    Eighth,         // _-------_-------
    Quarter,        // __------__------
    Half,           // ____----____----
    ThreeQuarter    // ______--______--
}

pub struct Audio {
    pa: pa::PortAudio,
    host: pa::HostApiIndex,
    stream: pa::Stream<NonBlocking, Output<f32>>,
    controller: gb::SoundController
}

impl Audio {
    const SAMPLE_RATE: f64 = 44100.0;
    const FRAMES_PER_BUFFER: u32 = 64;

    pub fn init() -> Self {
        let portaudio = pa::PortAudio::new()?;
        let host = portaudio.default_host_api()?;
        let mut settings = portaudio.default_output_stream_settings(
            2, Self::SAMPLE_RATE, Self::FRAMES_PER_BUFFER
        )?;
        settings.flags = pa::stream_flags::CLIP_OFF;

        let mut stream = portaudio.open_non_blocking_stream(
            settings, |pa::OutputStreamCallbackArgs { frames, buffer, time, flag }| {
                let mut idx = 0;
                for _ in 0..frames {
                    // We'll be using one interleaved buffer, alternating left-right-left-right...
                    // Since each sound terminal has two channels, we'll alternate like this:
                    // ST1.channel1 - ST2.channel1 - ST1.channel2 - ST2.channel2 - ST1.channel1...

                    idx += 4;
                }

                pa::Continue
            }
        )?;

        Self {
            pa: portaudio,
            host,
            stream,
            controller: gb::SoundController {}
        }
    }

    pub fn play(&mut self) -> Result<(), pa::Error> {
        self.stream.start()
    }

    pub fn stop(&mut self) -> Result<(), pa::Error> {
        self.stream.stop()
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        if self.stream.is_active()? {
            self.stop()?;
        }

        self.stream.close()?;
    }
}