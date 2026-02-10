use std::{fmt, sync::Arc};

use rodio::Sink;

use crate::audio_split::error::Error;

pub trait AudioPlayer: Sized {
    fn init() -> Result<Self, Error>;
    fn get_sink(&self) -> Arc<Sink>;
}

pub struct RodioPlayer {
    _stream_handle: rodio::OutputStream,
    sink: Arc<Sink>,
}

impl AudioPlayer for RodioPlayer {
    fn init() -> Result<Self, Error> {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
        let sink = rodio::Sink::connect_new(stream_handle.mixer());

        Ok(Self {
            _stream_handle: stream_handle,
            sink: Arc::new(sink),
        })
    }
    fn get_sink(&self) -> Arc<Sink> {
        self.sink.clone()
    }
}

impl fmt::Debug for RodioPlayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RodioPlayer")
    }
}
