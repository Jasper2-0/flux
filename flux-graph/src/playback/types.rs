//! Playback type definitions

use serde::{Deserialize, Serialize};

/// Audio source type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AudioSource {
    /// Use the project's embedded soundtrack
    #[default]
    ProjectSoundtrack,
    /// Use an external audio device/input
    ExternalDevice,
}

/// Playback sync mode
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SyncMode {
    /// Sync to timeline (frame-accurate)
    #[default]
    Timeline,
    /// Sync to manual tapping/beat input
    Tapping,
    /// Free-running (no sync)
    FreeRun,
}

/// Playback state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlaybackState {
    /// Playback is stopped
    #[default]
    Stopped,
    /// Playback is active
    Playing,
    /// Playback is paused
    Paused,
}
