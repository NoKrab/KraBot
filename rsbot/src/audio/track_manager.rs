use serenity::prelude::Mutex;
use serenity::voice;
use serenity::voice::{AudioSource, Handler, LockedAudio};

pub struct AudioTrack {
    source: Box<AudioSource>,
    title: String,
}

impl AudioTrack {
    pub fn new(source: Box<AudioSource>, title: String) -> AudioTrack {
        AudioTrack { source, title }
    }
}

pub struct TrackManager {
    queue: Vec<AudioTrack>,
    current: usize,
}

impl TrackManager {
    pub fn new() -> TrackManager {
        TrackManager { queue: Vec::new(), current: 0 }
    }
    pub fn add_track(mut self, track: AudioTrack) {
        self.queue.push(track);
    }
    pub fn clear_queue(mut self) {
        self.queue.clear();
    }
}
