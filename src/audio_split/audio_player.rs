use std::{fmt, sync::Arc};

use rodio::Player;

use crate::audio_split::error::Error;

pub trait AudioPlayer: Sized {
    fn init() -> Result<Self, Error>;
    fn get_player(&self) -> Arc<Player>;
}

pub struct RodioPlayer {
    _stream_handle: rodio::MixerDeviceSink,
    player: Arc<Player>,
}

impl AudioPlayer for RodioPlayer {
    fn init() -> Result<Self, Error> {
        let stream_handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let player = rodio::Player::connect_new(stream_handle.mixer());

        Ok(Self {
            _stream_handle: stream_handle,
            player: Arc::new(player),
        })
    }
    fn get_player(&self) -> Arc<Player> {
        self.player.clone()
    }
}

impl fmt::Debug for RodioPlayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RodioPlayer")
    }
}
