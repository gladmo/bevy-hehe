//! Audio setup for 合合游戏 (HeHe Game).
//!
//! Manages background music and sound effects. Audio assets are pre-loaded at
//! startup via [`setup_audio`] and stored in the [`GameAudio`] resource so that
//! game systems can look up a sound handle by code (e.g. `"button_click"`) and
//! spawn an [`AudioPlayer`] entity without touching the asset server.
use bevy::prelude::*;

use crate::config;

/// Holds pre-loaded [`Handle<AudioSource>`] values for every SFX entry in `audio.csv`.
///
/// Populated at startup by [`setup_audio`]; systems that want to play a sound
/// look up the code (e.g. `"button_click"`) and spawn an [`AudioPlayer`] entity
/// with [`PlaybackSettings::DESPAWN`].
#[derive(Resource, Default)]
pub(crate) struct GameAudio {
    pub(crate) sounds: std::collections::HashMap<String, Handle<AudioSource>>,
}

impl GameAudio {
    /// Return the handle for the given audio code, if it was loaded.
    pub(crate) fn get(&self, code: &str) -> Option<Handle<AudioSource>> {
        self.sounds.get(code).cloned()
    }

    /// Return the merge sound for a piece that produces a result at `result_level`.
    ///
    /// Tries `merge_lv{result_level}` first; falls back to `merge_lv9`.
    pub(crate) fn merge_sfx(&self, result_level: u32) -> Option<Handle<AudioSource>> {
        let code = format!("merge_lv{result_level}");
        self.get(&code).or_else(|| self.get("merge_lv9"))
    }
}

/// Startup system: pre-load every audio asset listed in `audio.csv` (except the
/// BGM which is handled by [`setup_bgm`]).  Handles are stored in [`GameAudio`]
/// so that SFX systems can look them up by code without touching the asset server.
pub(crate) fn setup_audio(
    asset_server: Res<AssetServer>,
    mut game_audio: ResMut<GameAudio>,
) {
    for def in config::load_audio() {
        let handle: Handle<AudioSource> = asset_server.load(def.audio_path.clone());
        game_audio.sounds.insert(def.audio_code, handle);
    }
}

/// Marker component for the background-music entity so we can query it later.
#[derive(Component)]
pub(crate) struct BgmSink;

/// Spawns the looping background-music entity as soon as the game starts.
///
/// On WASM, browsers block audio autoplay until the user interacts with the
/// page, so the sink is started in a paused state and resumed by
/// [`unlock_bgm_on_interaction`] on the first input event.
pub(crate) fn setup_bgm(asset_server: Res<AssetServer>, mut commands: Commands) {
    let bgm_path = config::load_audio()
        .into_iter()
        .find(|a| a.audio_code == "bgm_main")
        .map(|a| a.audio_path)
        .unwrap_or_else(|| "audio/bgm_SpringFestival_V1.wav".to_string());

    commands.spawn((
        BgmSink,
        AudioPlayer::new(asset_server.load(bgm_path)),
        #[cfg(not(target_arch = "wasm32"))]
        PlaybackSettings::LOOP,
        #[cfg(target_arch = "wasm32")]
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            paused: true,
            ..default()
        },
    ));
}

/// WASM only: resumes the background music on the first mouse-click, key-press,
/// or touch-start event, working around browsers' autoplay policy.
///
/// Two separate flags are used to handle a common race condition on mobile: the
/// user may interact with the page before the BGM [`AudioSink`] is ready (the
/// audio asset is still downloading).  [`had_interaction`] records that we owe
/// a `play()` call; [`unlocked`] is only set once we actually manage to call
/// it, so we keep retrying on subsequent frames until the sink exists.
#[cfg(target_arch = "wasm32")]
pub(crate) fn unlock_bgm_on_interaction(
    mut had_interaction: Local<bool>,
    mut unlocked: Local<bool>,
    sinks: Query<&AudioSink, With<BgmSink>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
) {
    if *unlocked {
        return;
    }

    // Latch the first user gesture (touch, click, or key).
    if !*had_interaction {
        let interacted = mouse.get_just_pressed().next().is_some()
            || keys.get_just_pressed().next().is_some()
            || touches.iter_just_pressed().next().is_some();
        if interacted {
            *had_interaction = true;
        }
    }

    // Once we have seen an interaction, try to play the sink every frame until
    // it exists.  On mobile networks the audio asset may still be downloading
    // when the first touch fires, so the AudioSink component is not yet present.
    if *had_interaction {
        for sink in &sinks {
            sink.play();
            *unlocked = true;
        }
    }
}
