use rodio::{source::SineWave, OutputStream, Sink};

use crate::errors::{Error, Result};

const BEEP_FREQ: f32 = 550.0;

pub struct AudioOutput {
    _stream: OutputStream,
    sink: Sink,
    enabled: bool,
}

impl AudioOutput {
    pub fn try_new() -> Result<Self> {
        let (_stream, stream_handle) =
            OutputStream::try_default().map_err(|e| Error::AudioOutputError(e.to_string()))?;
        let sink =
            Sink::try_new(&stream_handle).map_err(|e| Error::AudioOutputError(e.to_string()))?;
        
        Ok(Self {
            _stream,
            sink,
            enabled: false,
        })
    }

    pub fn start(&mut self) {
        let beep = SineWave::new(BEEP_FREQ);
        self.sink.append(beep);
        self.enabled = true;
    }

    pub fn stop(&mut self) {
        if self.enabled {
            self.sink.stop();
        }
        self.enabled = false;
    }
}
