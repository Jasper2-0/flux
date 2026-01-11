//! Playback settings for symbols
//!
//! This module provides playback configuration including audio clips,
//! BPM settings, sync modes, and beat locking.

mod audio_clip;
mod types;

pub use audio_clip::AudioClip;
pub use types::{AudioSource, PlaybackState, SyncMode};

use serde::{Deserialize, Serialize};

use flux_core::Id;

/// Playback settings for a symbol
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaybackSettings {
    /// Whether playback is enabled
    pub enabled: bool,
    /// Tempo in beats per minute
    pub bpm: f64,
    /// Audio clips in this symbol
    pub audio_clips: Vec<AudioClip>,
    /// Audio source type
    pub audio_source: AudioSource,
    /// Sync mode
    pub sync_mode: SyncMode,
    /// Current playback state
    pub state: PlaybackState,
    /// Audio input device name (for external source)
    pub audio_input_device: Option<String>,
    /// Audio gain factor
    pub audio_gain_factor: f32,
    /// Audio decay factor (for reactive audio)
    pub audio_decay_factor: f32,
    /// Whether beat locking is enabled
    pub enable_beat_locking: bool,
    /// Beat lock offset in seconds
    pub beat_lock_offset_sec: f64,
    /// Loop playback
    pub loop_playback: bool,
    /// Start time for playback range
    pub loop_start: f64,
    /// End time for playback range
    pub loop_end: f64,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            bpm: 120.0,
            audio_clips: Vec::new(),
            audio_source: AudioSource::default(),
            sync_mode: SyncMode::default(),
            state: PlaybackState::default(),
            audio_input_device: None,
            audio_gain_factor: 1.0,
            audio_decay_factor: 0.95,
            enable_beat_locking: false,
            beat_lock_offset_sec: 0.0,
            loop_playback: false,
            loop_start: 0.0,
            loop_end: 0.0,
        }
    }
}

impl PlaybackSettings {
    /// Create new playback settings with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create playback settings with a specific BPM
    pub fn with_bpm(bpm: f64) -> Self {
        Self { bpm, ..Self::default() }
    }

    /// Get the main soundtrack clip, if any
    pub fn get_main_soundtrack(&self) -> Option<&AudioClip> {
        self.audio_clips.iter().find(|c| c.is_soundtrack && !c.muted)
    }

    /// Get all active (non-muted) clips
    pub fn get_active_clips(&self) -> impl Iterator<Item = &AudioClip> {
        self.audio_clips.iter().filter(|c| !c.muted)
    }

    /// Get clips that are active at a specific time
    pub fn get_clips_at_time(&self, time: f64) -> impl Iterator<Item = &AudioClip> {
        self.audio_clips
            .iter()
            .filter(move |c| !c.muted && c.contains_time(time))
    }

    /// Add an audio clip
    pub fn add_clip(&mut self, clip: AudioClip) -> Id {
        let id = clip.id;
        self.audio_clips.push(clip);
        id
    }

    /// Remove an audio clip by ID
    pub fn remove_clip(&mut self, id: Id) -> Option<AudioClip> {
        if let Some(pos) = self.audio_clips.iter().position(|c| c.id == id) {
            Some(self.audio_clips.remove(pos))
        } else {
            None
        }
    }

    /// Set the main soundtrack
    pub fn set_soundtrack(&mut self, file_path: &str, duration: f64) -> Id {
        // Clear any existing soundtrack flags
        for clip in &mut self.audio_clips {
            clip.is_soundtrack = false;
        }

        let clip = AudioClip::soundtrack(file_path, duration);
        let id = clip.id;
        self.audio_clips.push(clip);
        id
    }

    // === Beat Calculations ===

    /// Get the duration of one beat in seconds
    pub fn beat_duration(&self) -> f64 {
        if self.bpm > 0.0 {
            60.0 / self.bpm
        } else {
            1.0
        }
    }

    /// Get the current beat number at a given time
    pub fn beat_at_time(&self, time: f64) -> f64 {
        time / self.beat_duration()
    }

    /// Get the time at a specific beat number
    pub fn time_at_beat(&self, beat: f64) -> f64 {
        beat * self.beat_duration()
    }

    /// Quantize a time to the nearest beat
    pub fn quantize_to_beat(&self, time: f64) -> f64 {
        let beat = self.beat_at_time(time).round();
        self.time_at_beat(beat)
    }

    /// Get the beat fraction (0.0 - 1.0 within the current beat)
    pub fn beat_fraction(&self, time: f64) -> f64 {
        self.beat_at_time(time).fract()
    }

    /// Get the measure number (assuming 4/4 time)
    pub fn measure_at_time(&self, time: f64, beats_per_measure: u32) -> f64 {
        self.beat_at_time(time) / beats_per_measure as f64
    }

    // === Playback Control ===

    /// Start playback
    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
    }

    /// Toggle play/pause
    pub fn toggle(&mut self) {
        self.state = match self.state {
            PlaybackState::Playing => PlaybackState::Paused,
            PlaybackState::Paused | PlaybackState::Stopped => PlaybackState::Playing,
        };
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    // === Loop Control ===

    /// Set loop range
    pub fn set_loop_range(&mut self, start: f64, end: f64) {
        self.loop_start = start.min(end);
        self.loop_end = start.max(end);
        self.loop_playback = true;
    }

    /// Clear loop range
    pub fn clear_loop(&mut self) {
        self.loop_playback = false;
        self.loop_start = 0.0;
        self.loop_end = 0.0;
    }

    /// Apply loop wrapping to a time value
    pub fn apply_loop(&self, time: f64) -> f64 {
        if !self.loop_playback || self.loop_end <= self.loop_start {
            return time;
        }

        let loop_duration = self.loop_end - self.loop_start;
        if time < self.loop_start {
            time
        } else if time >= self.loop_end {
            self.loop_start + ((time - self.loop_start) % loop_duration)
        } else {
            time
        }
    }

    // === Serialization ===

    /// Serialize to JSON
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(json.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_settings_default() {
        let settings = PlaybackSettings::new();
        assert!(settings.enabled);
        assert_eq!(settings.bpm, 120.0);
        assert!(settings.audio_clips.is_empty());
        assert_eq!(settings.state, PlaybackState::Stopped);
    }

    #[test]
    fn test_playback_settings_with_bpm() {
        let settings = PlaybackSettings::with_bpm(140.0);
        assert_eq!(settings.bpm, 140.0);
    }

    #[test]
    fn test_audio_clip_creation() {
        let clip = AudioClip::new("/path/to/audio.wav");
        assert_eq!(clip.file_path, "/path/to/audio.wav");
        assert_eq!(clip.name, "audio.wav");
        assert_eq!(clip.volume, 1.0);
        assert!(!clip.muted);
        assert!(!clip.is_soundtrack);
    }

    #[test]
    fn test_audio_clip_soundtrack() {
        let clip = AudioClip::soundtrack("/path/to/track.mp3", 180.0);
        assert!(clip.is_soundtrack);
        assert_eq!(clip.end_time, 180.0);
        assert_eq!(clip.duration(), 180.0);
    }

    #[test]
    fn test_audio_clip_contains_time() {
        let mut clip = AudioClip::new("test.wav");
        clip.start_time = 10.0;
        clip.end_time = 20.0;

        assert!(!clip.contains_time(5.0));
        assert!(clip.contains_time(10.0));
        assert!(clip.contains_time(15.0));
        assert!(!clip.contains_time(20.0));
    }

    #[test]
    fn test_add_remove_clips() {
        let mut settings = PlaybackSettings::new();

        let id1 = settings.add_clip(AudioClip::new("clip1.wav"));
        let id2 = settings.add_clip(AudioClip::new("clip2.wav"));
        assert_eq!(settings.audio_clips.len(), 2);

        let removed = settings.remove_clip(id1);
        assert!(removed.is_some());
        assert_eq!(settings.audio_clips.len(), 1);
        assert_eq!(settings.audio_clips[0].id, id2);
    }

    #[test]
    fn test_set_soundtrack() {
        let mut settings = PlaybackSettings::new();
        settings.add_clip(AudioClip::new("clip1.wav"));

        let soundtrack_id = settings.set_soundtrack("main.mp3", 300.0);

        let soundtrack = settings.get_main_soundtrack();
        assert!(soundtrack.is_some());
        assert_eq!(soundtrack.unwrap().id, soundtrack_id);
        assert!(soundtrack.unwrap().is_soundtrack);
    }

    #[test]
    fn test_beat_calculations() {
        let settings = PlaybackSettings::with_bpm(120.0); // 2 beats per second

        assert!((settings.beat_duration() - 0.5).abs() < 1e-10);
        assert!((settings.beat_at_time(1.0) - 2.0).abs() < 1e-10);
        assert!((settings.time_at_beat(4.0) - 2.0).abs() < 1e-10);
        assert!((settings.quantize_to_beat(1.1) - 1.0).abs() < 1e-10);
        assert!((settings.beat_fraction(1.25) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_measure_calculation() {
        let settings = PlaybackSettings::with_bpm(120.0);
        // At 120 BPM with 4 beats per measure, 8 seconds = 4 measures
        let measure = settings.measure_at_time(8.0, 4);
        assert!((measure - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_playback_control() {
        let mut settings = PlaybackSettings::new();
        assert!(!settings.is_playing());

        settings.play();
        assert!(settings.is_playing());
        assert_eq!(settings.state, PlaybackState::Playing);

        settings.pause();
        assert!(!settings.is_playing());
        assert_eq!(settings.state, PlaybackState::Paused);

        settings.toggle();
        assert!(settings.is_playing());

        settings.stop();
        assert_eq!(settings.state, PlaybackState::Stopped);
    }

    #[test]
    fn test_loop_control() {
        let mut settings = PlaybackSettings::new();

        settings.set_loop_range(10.0, 20.0);
        assert!(settings.loop_playback);
        assert_eq!(settings.loop_start, 10.0);
        assert_eq!(settings.loop_end, 20.0);

        // Time before loop
        assert_eq!(settings.apply_loop(5.0), 5.0);
        // Time within loop
        assert_eq!(settings.apply_loop(15.0), 15.0);
        // Time at loop end wraps
        assert!((settings.apply_loop(20.0) - 10.0).abs() < 1e-10);
        // Time past loop wraps
        assert!((settings.apply_loop(25.0) - 15.0).abs() < 1e-10);

        settings.clear_loop();
        assert!(!settings.loop_playback);
    }

    #[test]
    fn test_clips_at_time() {
        let mut settings = PlaybackSettings::new();

        let mut clip1 = AudioClip::new("clip1.wav");
        clip1.start_time = 0.0;
        clip1.end_time = 10.0;
        settings.add_clip(clip1);

        let mut clip2 = AudioClip::new("clip2.wav");
        clip2.start_time = 5.0;
        clip2.end_time = 15.0;
        settings.add_clip(clip2);

        let mut clip3 = AudioClip::new("clip3.wav");
        clip3.start_time = 20.0;
        clip3.end_time = 30.0;
        clip3.muted = true;
        settings.add_clip(clip3);

        // At time 0, only clip1
        let clips_at_0: Vec<_> = settings.get_clips_at_time(0.0).collect();
        assert_eq!(clips_at_0.len(), 1);

        // At time 7, both clip1 and clip2
        let clips_at_7: Vec<_> = settings.get_clips_at_time(7.0).collect();
        assert_eq!(clips_at_7.len(), 2);

        // At time 25, clip3 is muted so none
        let clips_at_25: Vec<_> = settings.get_clips_at_time(25.0).collect();
        assert_eq!(clips_at_25.len(), 0);
    }

    #[test]
    fn test_serialization() {
        let mut settings = PlaybackSettings::with_bpm(140.0);
        settings.add_clip(AudioClip::new("test.wav"));
        settings.enable_beat_locking = true;

        let json = settings.to_json();
        let restored = PlaybackSettings::from_json(&json).unwrap();

        assert_eq!(restored.bpm, 140.0);
        assert_eq!(restored.audio_clips.len(), 1);
        assert!(restored.enable_beat_locking);
    }
}
