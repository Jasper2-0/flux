//! Audio clip definitions

use serde::{Deserialize, Serialize};

use flux_core::Id;

/// Audio clip reference
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioClip {
    /// Unique identifier for this clip
    pub id: Id,
    /// Path to the audio file
    pub file_path: String,
    /// Display name for the clip
    pub name: String,
    /// Start time in the timeline (seconds)
    pub start_time: f64,
    /// End time in the timeline (seconds)
    pub end_time: f64,
    /// Volume multiplier (0.0 - 1.0+)
    pub volume: f32,
    /// Whether the clip is muted
    pub muted: bool,
    /// Whether this is the main soundtrack
    pub is_soundtrack: bool,
}

impl AudioClip {
    /// Create a new audio clip
    pub fn new(file_path: &str) -> Self {
        Self {
            id: Id::new(),
            file_path: file_path.to_string(),
            name: file_path
                .rsplit('/')
                .next()
                .unwrap_or(file_path)
                .to_string(),
            start_time: 0.0,
            end_time: 0.0, // 0 means until end of file
            volume: 1.0,
            muted: false,
            is_soundtrack: false,
        }
    }

    /// Create a new soundtrack clip
    pub fn soundtrack(file_path: &str, duration: f64) -> Self {
        Self {
            id: Id::new(),
            file_path: file_path.to_string(),
            name: file_path
                .rsplit('/')
                .next()
                .unwrap_or(file_path)
                .to_string(),
            start_time: 0.0,
            end_time: duration,
            volume: 1.0,
            muted: false,
            is_soundtrack: true,
        }
    }

    /// Get the duration of this clip's timeline range
    pub fn duration(&self) -> f64 {
        if self.end_time > self.start_time {
            self.end_time - self.start_time
        } else {
            0.0 // Unknown/unlimited
        }
    }

    /// Check if a time falls within this clip's range
    pub fn contains_time(&self, time: f64) -> bool {
        if self.end_time <= self.start_time {
            time >= self.start_time
        } else {
            time >= self.start_time && time < self.end_time
        }
    }
}

impl Default for AudioClip {
    fn default() -> Self {
        Self {
            id: Id::new(),
            file_path: String::new(),
            name: String::new(),
            start_time: 0.0,
            end_time: 0.0,
            volume: 1.0,
            muted: false,
            is_soundtrack: false,
        }
    }
}
