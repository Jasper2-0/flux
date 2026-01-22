//! Demo 15: Playback Settings
//!
//! This example demonstrates the playback/audio system:
//! - BPM-based timing and beat calculations
//! - Beat quantization
//! - Audio clip management
//! - Soundtrack handling
//! - Playback state control (play, pause, stop)
//! - Loop regions
//! - Serialization/deserialization
//!
//! Run with: `cargo run --example 15_playback_settings`

use flux_graph::playback::{AudioClip, AudioSource, PlaybackSettings, SyncMode};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 15: Playback Settings             ║");
    println!("╚════════════════════════════════════════╝\n");

    // Create playback settings
    let mut playback = PlaybackSettings::with_bpm(128.0);
    println!("Playback settings:");
    println!("  BPM: {}", playback.bpm);
    println!("  Beat duration: {:.3}s", playback.beat_duration());
    println!("  Audio source: {:?}", playback.audio_source);
    println!("  Sync mode: {:?}", playback.sync_mode);

    // Beat calculations
    println!("\nBeat calculations at 128 BPM:");
    for seconds in [0.0, 0.5, 1.0, 2.0, 4.0] {
        let beat = playback.beat_at_time(seconds);
        let fraction = playback.beat_fraction(seconds);
        let measure = playback.measure_at_time(seconds, 4);
        println!(
            "  {:.1}s -> beat {:.2}, fraction {:.2}, measure {:.2}",
            seconds, beat, fraction, measure
        );
    }

    // Quantization
    println!("\nBeat quantization:");
    for time in [0.23, 0.52, 1.31] {
        let quantized = playback.quantize_to_beat(time);
        println!("  {:.2}s -> {:.3}s (nearest beat)", time, quantized);
    }

    // Add audio clips
    let mut clip1 = AudioClip::new("/audio/drums.wav");
    clip1.start_time = 0.0;
    clip1.end_time = 60.0;
    playback.add_clip(clip1);

    let mut clip2 = AudioClip::new("/audio/synth.wav");
    clip2.start_time = 8.0;
    clip2.end_time = 30.0;
    playback.add_clip(clip2);

    // Set main soundtrack
    let _soundtrack_id = playback.set_soundtrack("/audio/main_track.mp3", 180.0);
    println!("\nAudio clips:");
    println!("  Total clips: {}", playback.audio_clips.len());
    for clip in &playback.audio_clips {
        println!(
            "    - {} ({:.0}s-{:.0}s) vol:{} {}{}",
            clip.name,
            clip.start_time,
            clip.end_time,
            clip.volume,
            if clip.is_soundtrack { "[SOUNDTRACK]" } else { "" },
            if clip.muted { "[MUTED]" } else { "" }
        );
    }

    // Get clips at specific times
    println!("\nClips at different times:");
    for t in [0.0, 10.0, 50.0] {
        let clips: Vec<_> = playback.get_clips_at_time(t).collect();
        println!("  t={:.0}s: {} active clips", t, clips.len());
    }

    // Main soundtrack
    if let Some(track) = playback.get_main_soundtrack() {
        println!("\nMain soundtrack: {}", track.file_path);
        println!("  Duration: {:.0}s", track.duration());
    }

    // Playback control
    println!("\nPlayback control:");
    println!("  State: {:?}", playback.state);
    playback.play();
    println!("  After play(): {:?}", playback.state);
    playback.pause();
    println!("  After pause(): {:?}", playback.state);
    playback.toggle();
    println!("  After toggle(): {:?}", playback.state);
    playback.stop();
    println!("  After stop(): {:?}", playback.state);

    // Loop control
    playback.set_loop_range(16.0, 32.0);
    println!("\nLoop settings:");
    println!("  Loop enabled: {}", playback.loop_playback);
    println!(
        "  Loop range: {:.0}s - {:.0}s",
        playback.loop_start, playback.loop_end
    );

    println!("\nLoop time wrapping:");
    for t in [8.0, 16.0, 24.0, 32.0, 40.0, 48.0] {
        let wrapped = playback.apply_loop(t);
        println!("  {:.0}s -> {:.0}s", t, wrapped);
    }

    // Different configurations
    let mut live_settings = PlaybackSettings::new();
    live_settings.audio_source = AudioSource::ExternalDevice;
    live_settings.sync_mode = SyncMode::Tapping;
    live_settings.enable_beat_locking = true;
    live_settings.audio_gain_factor = 1.5;

    println!("\nLive performance settings:");
    println!("  Audio source: {:?}", live_settings.audio_source);
    println!("  Sync mode: {:?}", live_settings.sync_mode);
    println!("  Beat locking: {}", live_settings.enable_beat_locking);
    println!("  Audio gain: {}", live_settings.audio_gain_factor);

    // Serialization
    let json = playback.to_json();
    let restored = PlaybackSettings::from_json(&json).unwrap();
    println!("\nSerialization test:");
    println!("  Original BPM: {}", playback.bpm);
    println!("  Restored BPM: {}", restored.bpm);
    println!(
        "  Clips preserved: {}",
        restored.audio_clips.len() == playback.audio_clips.len()
    );
}
