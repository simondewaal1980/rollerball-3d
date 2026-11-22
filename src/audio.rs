//! Modular pinball audio.
//!
//! Sounds are described by `assets/audio/sounds.ron`, so the person can drop
//! WAV or OGG files into `assets/audio/` without recompiling. Every missing
//! file is replaced by a small MSX-PSG-style square-wave patch generated in
//! code. Playback is through Kira, which mixes many overlapping voices; each
//! effect also has a volume budget so bumper stacks do not clip.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use bevy::log::{info, warn};
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_kira_audio::prelude::{Frame, StaticSoundData, StaticSoundSettings};

use bevy_kira_audio::AudioSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundEvent {
    Start,
    PlungerCharge,
    PlungerRelease,
    FlipperHit,
    BumperHit,
    RolloverLane,
    SlotSpin,
    SlotWin,
    ChuteDrop,
    BonusCountdownTick,
    BallDrain,
    GameOver,
    Kickback,
    ExtraBall,
    Tilt,
}

impl SoundEvent {
    pub const ALL: [SoundEvent; 15] = [
        SoundEvent::Start,
        SoundEvent::PlungerCharge,
        SoundEvent::PlungerRelease,
        SoundEvent::FlipperHit,
        SoundEvent::BumperHit,
        SoundEvent::RolloverLane,
        SoundEvent::SlotSpin,
        SoundEvent::SlotWin,
        SoundEvent::ChuteDrop,
        SoundEvent::BonusCountdownTick,
        SoundEvent::BallDrain,
        SoundEvent::GameOver,
        SoundEvent::Kickback,
        SoundEvent::ExtraBall,
        SoundEvent::Tilt,
    ];

    pub fn key(self) -> &'static str {
        match self {
            SoundEvent::Start => "start",
            SoundEvent::PlungerCharge => "plunger_charge",
            SoundEvent::PlungerRelease => "plunger_release",
            SoundEvent::FlipperHit => "flipper_hit",
            SoundEvent::BumperHit => "bumper_hit",
            SoundEvent::RolloverLane => "rollover_lane",
            SoundEvent::SlotSpin => "slot_spin",
            SoundEvent::SlotWin => "slot_win",
            SoundEvent::ChuteDrop => "chute_drop",
            SoundEvent::BonusCountdownTick => "bonus_countdown_tick",
            SoundEvent::BallDrain => "ball_drain",
            SoundEvent::GameOver => "game_over",
            SoundEvent::Kickback => "kickback",
            SoundEvent::ExtraBall => "extra_ball",
            SoundEvent::Tilt => "tilt",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        let key = key.trim().to_ascii_lowercase().replace('-', "_");
        Self::ALL.into_iter().find(|e| e.key() == key)
    }

    /// Default file name and loudness. The person only has to drop a file with
    /// this name into assets/audio/ and adjust the RON file if wanted.
    fn default_file(self) -> &'static str {
        match self {
            SoundEvent::Start => "start.wav",
            SoundEvent::PlungerCharge => "plunger_charge.wav",
            SoundEvent::PlungerRelease => "plunger_release.wav",
            SoundEvent::FlipperHit => "flipper_hit.wav",
            SoundEvent::BumperHit => "bumper_hit.wav",
            SoundEvent::RolloverLane => "rollover_lane.wav",
            SoundEvent::SlotSpin => "slot_spin.wav",
            SoundEvent::SlotWin => "slot_win.ogg",
            SoundEvent::ChuteDrop => "chute_drop.wav",
            SoundEvent::BonusCountdownTick => "bonus_countdown_tick.wav",
            SoundEvent::BallDrain => "ball_drain.wav",
            SoundEvent::GameOver => "game_over.wav",
            SoundEvent::Kickback => "kickback.wav",
            SoundEvent::ExtraBall => "extra_ball.wav",
            SoundEvent::Tilt => "tilt.wav",
        }
    }

    fn default_volume_db(self) -> f32 {
        match self {
            SoundEvent::Start => -11.0,
            SoundEvent::PlungerCharge => -13.0,
            SoundEvent::PlungerRelease => -9.0,
            SoundEvent::FlipperHit => -14.0,
            SoundEvent::BumperHit => -11.0,
            SoundEvent::RolloverLane => -13.0,
            SoundEvent::SlotSpin => -13.0,
            SoundEvent::SlotWin => -7.0,
            SoundEvent::ChuteDrop => -11.0,
            SoundEvent::BonusCountdownTick => -14.0,
            SoundEvent::BallDrain => -9.0,
            SoundEvent::GameOver => -7.0,
            SoundEvent::Kickback => -10.0,
            SoundEvent::ExtraBall => -9.0,
            SoundEvent::Tilt => -9.0,
        }
    }
}

/// Dedicated Kira channel for gameplay effects. Keeping every one-shot sound
/// on one track lets a single compressor guard the summed output.
#[derive(Resource)]
pub struct SfxChannel;

/// A separate, gentler channel for the looping background chiptune. Keeping it
/// off the SFX track means the compressor never pumps the music when a pile of
/// bumpers fires at once.
#[derive(Resource)]
pub struct MusicChannel;

/// State shown in the sidebar and used by the N key.
#[derive(Resource)]
pub struct MusicState {
    pub enabled: bool,
    pub volume_db: f32,
    pub source: String,
    pub external: bool,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            enabled: true,
            volume_db: -16.0,
            source: "procedural chiptune".into(),
            external: false,
        }
    }
}

/// Build the effect track. The compressor catches a pile-up of simultaneous
/// bumper/slingshot hits; individual sounds are already trimmed low.
pub fn sfx_track() -> AudioTrack {
    let mut track = AudioTrack::new();
    track.add_effect(
        CompressorBuilder::new()
            .threshold(-14.0)
            .ratio(3.0)
            .attack_duration(Duration::from_millis(4))
            .release_duration(Duration::from_millis(140)),
    );
    track
}

/// Queue of sound events produced by gameplay this frame.
#[derive(Resource, Default)]
pub struct SoundQueue(pub Vec<SoundEvent>);

impl SoundQueue {
    pub fn push(&mut self, event: SoundEvent) {
        // A generous ceiling still allows a whole pile of bumper hits while
        // protecting the output from a runaway feedback loop.
        if self.0.len() < 16 {
            self.0.push(event);
        }
    }
}

#[derive(Resource, Default)]
pub struct SoundBank {
    /// More than one file per event is allowed; one is chosen per hit.
    pub handles: HashMap<SoundEvent, Vec<Handle<AudioSource>>>,
    pub volume_db: HashMap<SoundEvent, f32>,
    pub fallback_events: Vec<SoundEvent>,
    pub loaded_files: usize,
    rng: u32,
}

impl SoundBank {
    fn pick(&mut self, event: SoundEvent) -> Option<Handle<AudioSource>> {
        let len = self.handles.get(&event).map(Vec::len).unwrap_or(0);
        if len == 0 {
            return None;
        }
        // Small xorshift: cheap, deterministic per run, and good enough to
        // rotate the three bumper takes so repeats do not sound identical.
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        let index = (self.rng as usize) % len;
        self.handles.get(&event).map(|list| list[index].clone())
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct SoundEntry {
    event: String,
    file: String,
    #[serde(default = "default_volume")]
    volume_db: f32,
}

fn default_volume() -> f32 {
    -10.0
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct MusicEntry {
    file: String,
    #[serde(default = "default_music_volume")]
    volume_db: f32,
}

fn default_music_volume() -> f32 {
    -16.0
}

impl Default for MusicEntry {
    fn default() -> Self {
        Self {
            file: "music_theme.wav".into(),
            volume_db: default_music_volume(),
        }
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct SoundConfig {
    #[serde(default)]
    sounds: Vec<SoundEntry>,
    #[serde(default)]
    music: MusicEntry,
}

/// Find the asset folder no matter where the game is started from.
///
/// The usual cases are: started from the project folder (`assets/`), started
/// from `target/debug` (`../../assets`), or a packaged build with an `assets`
/// folder next to the executable. Every candidate is checked for the audio
/// configuration before it is accepted.
pub fn assets_root() -> PathBuf {
    let mut candidates = vec![PathBuf::from("assets")];

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("assets"));
            // target/debug -> app/assets, target/release -> app/assets.
            candidates.push(dir.join("../../assets"));
            // A packaged folder that keeps assets one level above the binary.
            candidates.push(dir.join("../assets"));
        }
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));

    for candidate in &candidates {
        if candidate.join("audio").join("sounds.ron").exists() {
            // Canonicalise so a dropped-in wav/ogg loads whether the game is
            // started from the project folder or from anywhere else.
            return std::fs::canonicalize(candidate).unwrap_or_else(|_| candidate.clone());
        }
    }

    // No configuration found: keep the conventional relative folder so the
    // game can still create/read audio there and fall back to the synth.
    PathBuf::from("assets")
}

/// Decode a WAV or OGG file up-front. Unlike `asset_server.load`, this tells
/// us immediately whether the file really is playable, so a corrupt, empty or
/// missing file can fall back to the synth instead of leaving a silent handle.
fn decode_audio(path: &PathBuf) -> Result<StaticSoundData, String> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if extension != "wav" && extension != "ogg" {
        return Err(format!("unsupported extension .{extension}"));
    }
    StaticSoundData::from_file(path).map_err(|err| err.to_string())
}

fn load_config(root: &PathBuf) -> SoundConfig {
    let path = root.join("audio").join("sounds.ron");
    if let Ok(text) = std::fs::read_to_string(&path) {
        match ron::from_str::<SoundConfig>(&text) {
            Ok(config) => return config,
            Err(err) => warn!("Could not read {}: {err}", path.display()),
        }
    }
    // Defaults are also the fallback if the RON file is absent or broken.
    SoundConfig {
        sounds: SoundEvent::ALL
            .into_iter()
            .map(|event| SoundEntry {
                event: event.key().to_string(),
                file: event.default_file().to_string(),
                volume_db: event.default_volume_db(),
            })
            .collect(),
        music: MusicEntry::default(),
    }
}

/// Startup system: read the RON mapping and build a handle for every event,
/// either from a real file or from the procedural synthesizer.
pub fn setup_audio(
    mut commands: Commands,
    mut sources: ResMut<Assets<AudioSource>>,
    audio: Res<AudioChannel<SfxChannel>>,
    music: Res<AudioChannel<MusicChannel>>,
) {
    // Master headroom: individual effects are already trimmed. The channel's
    // compressor is the final safety net when several bumpers and slingshots
    // land in the same physics tick, so a stack cannot clip.
    audio.set_volume(-6.0);
    let root = assets_root();
    let config = load_config(&root);
    let audio_dir = root.join("audio");
    let mut bank = SoundBank {
        rng: 0x9e37_79b9,
        ..Default::default()
    };

    for event in SoundEvent::ALL {
        let entries: Vec<&SoundEntry> = config
            .sounds
            .iter()
            .filter(|entry| SoundEvent::from_key(&entry.event) == Some(event))
            .collect();

        let (files, volume) = if entries.is_empty() {
            (
                vec![event.default_file().to_string()],
                event.default_volume_db(),
            )
        } else {
            (
                entries.iter().map(|entry| entry.file.clone()).collect(),
                entries[0].volume_db,
            )
        };

        let mut handles = Vec::new();
        for file in &files {
            let path = audio_dir.join(file);
            match decode_audio(&path) {
                Ok(sound) => {
                    bank.loaded_files += 1;
                    handles.push(sources.add(AudioSource { sound }));
                }
                Err(reason) => {
                    if path.exists() {
                        warn!(
                            "Audio: could not decode {} ({reason}), skipping it",
                            path.display()
                        );
                    }
                }
            }
        }

        // If every take for an event is missing, synthesize one so the event
        // still has a voice instead of silently doing nothing.
        if handles.is_empty() {
            bank.fallback_events.push(event);
            let sound = synth_sound(event);
            handles.push(sources.add(AudioSource { sound }));
        }

        bank.handles.insert(event, handles);
        bank.volume_db.insert(event, volume);
    }

    let fallback_count = bank.fallback_events.len();
    if bank.loaded_files > 0 {
        info!(
            "Audio: {} external sound file(s) ready, {} event(s) using the procedural synth",
            bank.loaded_files, fallback_count
        );
    } else {
        info!("Audio: no sound files found, using the MSX-style procedural synth");
    }
    if !bank.fallback_events.is_empty() {
        let names = bank
            .fallback_events
            .iter()
            .map(|event| event.key())
            .collect::<Vec<_>>()
            .join(", ");
        info!("Audio fallback in use for: {names}");
    }

    commands.insert_resource(bank);

    // ------------------------------------------------------------------
    // Background music: an external loop if present, otherwise an original
    // procedural 8-bit tune. It starts on the menu and keeps playing across
    // games; N mutes it. The music has its own channel, so the SFX compressor
    // never ducks it.
    // ------------------------------------------------------------------
    let music_file = audio_dir.join(&config.music.file);
    let (music_sound, external, source) = match decode_audio(&music_file) {
        Ok(sound) => (sound, true, config.music.file.clone()),
        Err(reason) => {
            if music_file.exists() {
                warn!(
                    "Music: could not decode {} ({reason}), using the procedural chiptune",
                    music_file.display()
                );
            }
            (synth_music(), false, "procedural chiptune".to_string())
        }
    };
    let music_handle = sources.add(AudioSource {
        sound: music_sound,
    });
    music.set_volume(config.music.volume_db);
    music.play(music_handle).looped();
    commands.insert_resource(MusicState {
        enabled: true,
        volume_db: config.music.volume_db,
        source,
        external,
    });
    info!(
        "Music: {} at {:.1} dB{}",
        if external {
            "external 8-bit loop"
        } else {
            "procedural 8-bit loop"
        },
        config.music.volume_db,
        if external { "" } else { " (file missing)" }
    );
}

/// N toggles the background music without touching the effects.
pub fn toggle_music(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<MusicState>,
    music: Res<AudioChannel<MusicChannel>>,
) {
    if !keys.just_pressed(KeyCode::KeyN) {
        return;
    }
    state.enabled = !state.enabled;
    music.set_volume(if state.enabled {
        state.volume_db
    } else {
        -80.0
    });
}

/// Drain the gameplay sound queue and play each event on the main Kira track.
pub fn play_queued_sounds(
    mut queue: ResMut<SoundQueue>,
    mut bank: ResMut<SoundBank>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    for event in queue.0.drain(..) {
        let Some(handle) = bank.pick(event) else {
            continue;
        };
        let volume = bank.volume_db.get(&event).copied().unwrap_or(-10.0);
        // Kira mixes these voices; low individual levels leave headroom.
        audio
            .play(handle)
            .with_volume(Decibels(volume));
    }
}

// ---------------------------------------------------------------------------
// Procedural PSG-style fallback synth
// ---------------------------------------------------------------------------

const SAMPLE_RATE: u32 = 44_100;

#[derive(Clone, Copy)]
enum Wave {
    Square,
    Triangle,
    Noise,
}

fn synth_sound(event: SoundEvent) -> StaticSoundData {
    let mut frames = vec![Frame::ZERO; (SAMPLE_RATE as f32 * 1.15) as usize];

    match event {
        SoundEvent::Start => {
            for (i, f) in [523.25f32, 659.25, 783.99, 1046.5].iter().enumerate() {
                tone(
                    &mut frames,
                    i as f32 * 0.085,
                    0.10,
                    *f,
                    *f,
                    0.20,
                    Wave::Square,
                );
            }
            for f in [1046.5f32, 1318.5, 1568.0] {
                tone(&mut frames, 0.34, 0.60, f, f, 0.09, Wave::Square);
            }
            tone(&mut frames, 0.34, 0.60, 261.63, 261.63, 0.15, Wave::Triangle);
        }
        SoundEvent::PlungerCharge => {
            tone(&mut frames, 0.00, 0.55, 110.0, 330.0, 0.20, Wave::Square);
            tone(&mut frames, 0.00, 0.55, 220.0, 660.0, 0.07, Wave::Triangle);
        }
        SoundEvent::PlungerRelease => {
            tone(&mut frames, 0.00, 0.16, 700.0, 90.0, 0.30, Wave::Square);
            tone(&mut frames, 0.00, 0.08, 1400.0, 500.0, 0.10, Wave::Noise);
        }
        SoundEvent::FlipperHit => {
            tone(&mut frames, 0.00, 0.055, 260.0, 190.0, 0.26, Wave::Square);
            tone(&mut frames, 0.01, 0.04, 90.0, 70.0, 0.12, Wave::Noise);
        }
        SoundEvent::BumperHit => {
            tone(&mut frames, 0.00, 0.09, 900.0, 620.0, 0.24, Wave::Square);
            tone(&mut frames, 0.05, 0.08, 1200.0, 800.0, 0.10, Wave::Triangle);
        }
        SoundEvent::RolloverLane => {
            tone(&mut frames, 0.00, 0.075, 880.0, 880.0, 0.20, Wave::Square);
            tone(&mut frames, 0.075, 0.09, 1320.0, 1320.0, 0.20, Wave::Square);
        }
        SoundEvent::SlotSpin => {
            for i in 0..4 {
                let f = 500.0 + i as f32 * 120.0;
                tone(
                    &mut frames,
                    i as f32 * 0.075,
                    0.05,
                    f,
                    f,
                    0.16,
                    Wave::Square,
                );
            }
        }
        SoundEvent::SlotWin => {
            for (i, f) in [523.25f32, 659.25, 783.99, 1046.5].iter().enumerate() {
                tone(
                    &mut frames,
                    i as f32 * 0.10,
                    0.24,
                    *f,
                    *f,
                    0.20,
                    Wave::Square,
                );
            }
        }
        SoundEvent::ChuteDrop => {
            tone(&mut frames, 0.00, 0.26, 520.0, 90.0, 0.22, Wave::Triangle);
        }
        SoundEvent::BonusCountdownTick => {
            tone(&mut frames, 0.00, 0.035, 1250.0, 1250.0, 0.16, Wave::Square);
        }
        SoundEvent::BallDrain => {
            tone(&mut frames, 0.00, 0.45, 360.0, 60.0, 0.25, Wave::Triangle);
            tone(&mut frames, 0.10, 0.30, 180.0, 45.0, 0.14, Wave::Square);
        }
        SoundEvent::GameOver => {
            for (i, f) in [392.0f32, 329.63, 261.63].iter().enumerate() {
                tone(
                    &mut frames,
                    i as f32 * 0.24,
                    0.34,
                    *f,
                    *f,
                    0.20,
                    Wave::Square,
                );
            }
        }
        SoundEvent::Kickback => {
            tone(&mut frames, 0.00, 0.18, 110.0, 55.0, 0.30, Wave::Triangle);
            tone(&mut frames, 0.00, 0.10, 220.0, 700.0, 0.16, Wave::Square);
            tone(&mut frames, 0.00, 0.09, 1600.0, 400.0, 0.12, Wave::Noise);
        }
        SoundEvent::ExtraBall => {
            for (i, f) in [659.25f32, 783.99, 987.77, 1318.5].iter().enumerate() {
                tone(
                    &mut frames,
                    i as f32 * 0.09,
                    0.16,
                    *f,
                    *f,
                    0.19,
                    Wave::Square,
                );
            }
        }
        SoundEvent::Tilt => {
            for i in 0..12 {
                let f = if i % 2 == 0 { 116.0 } else { 104.0 };
                tone(
                    &mut frames,
                    i as f32 * 0.05,
                    0.045,
                    f,
                    f,
                    0.17,
                    Wave::Square,
                );
            }
            tone(&mut frames, 0.00, 0.60, 58.0, 58.0, 0.10, Wave::Triangle);
        }
    }

    StaticSoundData {
        sample_rate: SAMPLE_RATE,
        frames: frames.into(),
        settings: StaticSoundSettings::default(),
        slice: None,
    }
}

/// Original 8-bar pinball chiptune used when `music_theme.wav` is absent.
/// Three PSG voices (square lead, square arpeggio, triangle bass) and a noise
/// drum kit, in A minor with an Am-F-C-G progression. It is generated at
/// start-up, so the game always has a soundtrack without shipping a sample.
fn synth_music() -> StaticSoundData {
    const BPM: f32 = 126.0;
    let beat = 60.0 / BPM;
    let bars = 8;
    let total = (bars as f32 * 4.0 * beat * SAMPLE_RATE as f32) as usize;
    let mut frames = vec![Frame::ZERO; total];

    let midi = |n: i32| 440.0 * 2.0f32.powf((n as f32 - 69.0) / 12.0);
    let chords: [[i32; 3]; 4] = [
        [57, 60, 64], // Am
        [53, 57, 60], // F
        [60, 64, 67], // C
        [55, 59, 62], // G
    ];
    let bass_roots = [45, 41, 36, 43];
    let melody: [[i32; 8]; 8] = [
        [69, 72, 76, 72, 71, 69, 67, 64],
        [65, 69, 72, 69, 67, 65, 64, 62],
        [64, 67, 72, 76, 74, 72, 71, 67],
        [67, 71, 74, 71, 69, 67, 65, 62],
        [69, 72, 76, 79, 76, 72, 71, 69],
        [65, 69, 72, 77, 76, 72, 69, 65],
        [64, 67, 72, 76, 79, 76, 72, 67],
        [67, 71, 74, 79, 77, 74, 71, 67],
    ];

    for bar in 0..bars {
        let bar_start = bar as f32 * 4.0 * beat;
        let chord = chords[bar % 4];
        let root = bass_roots[bar % 4];
        let mel = melody[bar];

        // Lead: one square note per eighth, slightly staccato.
        for step in 0..8 {
            let t = bar_start + step as f32 * beat * 0.5;
            tone(
                &mut frames,
                t,
                beat * 0.43,
                midi(mel[step]),
                midi(mel[step]),
                0.085,
                Wave::Square,
            );
        }

        // A quiet 16th-note arpeggio adds the sparkling pinball motion.
        for step in 0..8 {
            let t = bar_start + step as f32 * beat * 0.5;
            for sub in 0..2 {
                let note = chord[(step * 2 + sub) % 3] + 12;
                tone(
                    &mut frames,
                    t + sub as f32 * beat * 0.25,
                    beat * 0.20,
                    midi(note),
                    midi(note),
                    0.026,
                    Wave::Square,
                );
            }
        }

        // Triangle bass on every beat, with an octave lift on the offbeat.
        for step in 0..8 {
            let t = bar_start + step as f32 * beat * 0.5;
            let note = if step % 2 == 0 { root } else { root + 7 };
            tone(
                &mut frames,
                t,
                beat * 0.38,
                midi(note),
                midi(note),
                0.075,
                Wave::Triangle,
            );
        }

        // Drums: kick on 1 and 3, snare on 2 and 4, hats on eighths.
        for beat_i in 0..4 {
            let t = bar_start + beat_i as f32 * beat;
            if beat_i % 2 == 0 {
                tone(&mut frames, t, 0.13, 120.0, 46.0, 0.11, Wave::Triangle);
            } else {
                tone(&mut frames, t, 0.10, 1_800.0, 700.0, 0.055, Wave::Noise);
            }
            tone(&mut frames, t + beat * 0.5, 0.035, 6_000.0, 5_000.0, 0.018, Wave::Noise);
        }
    }

    // Normalise to a comfortable loop peak, leaving headroom for the effects.
    let peak = frames
        .iter()
        .map(|f| f.left.abs().max(f.right.abs()))
        .fold(0.0f32, f32::max)
        .max(1e-6);
    let gain = 0.62 / peak;
    for frame in &mut frames {
        frame.left *= gain;
        frame.right *= gain;
    }

    StaticSoundData {
        sample_rate: SAMPLE_RATE,
        frames: frames.into(),
        settings: StaticSoundSettings::default(),
        slice: None,
    }
}

fn tone(
    frames: &mut [Frame],
    start_secs: f32,
    duration: f32,
    freq_from: f32,
    freq_to: f32,
    amplitude: f32,
    wave: Wave,
) {
    let start = (start_secs * SAMPLE_RATE as f32) as usize;
    let count = (duration * SAMPLE_RATE as f32) as usize;
    let total = frames.len();
    let mut phase = 0.0f32;
    let mut noise_state = 0x1234_5678u32;

    for i in 0..count {
        let idx = start + i;
        if idx >= total {
            break;
        }
        let t = i as f32 / SAMPLE_RATE as f32;
        let p = i as f32 / count.max(1) as f32;
        let freq = freq_from + (freq_to - freq_from) * p;
        phase += freq / SAMPLE_RATE as f32;
        if phase >= 1.0 {
            phase -= 1.0;
        }

        // Small attack and exponential-ish decay, like a PSG envelope.
        let attack = (t / 0.004).clamp(0.0, 1.0);
        let decay = (1.0 - p).powf(1.4);
        let env = attack * decay;

        let value = match wave {
            Wave::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Wave::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
            Wave::Noise => {
                noise_state ^= noise_state << 13;
                noise_state ^= noise_state >> 17;
                noise_state ^= noise_state << 5;
                if noise_state & 1 == 0 {
                    1.0
                } else {
                    -1.0
                }
            }
        };

        let sample = value * env * amplitude;
        frames[idx].left += sample;
        frames[idx].right += sample;
    }
}
