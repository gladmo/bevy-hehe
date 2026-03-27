//! 合合游戏 (HeHe Game) — Bevy 0.18.1 implementation
//!
//! An ancient Chinese tea house themed merge puzzle game.
//! Two same-level, same-chain pieces merge into the next level.
//! Generator pieces produce child pieces (costs 1 stamina).
//! Auto-generator pieces (老母鸡) produce child pieces automatically over time.
//! Fulfill orders by submitting the required items to earn coins.

mod board;
mod config;
mod economy;
mod items;
mod orders;
mod systems;
mod ui;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use board::Board;
use economy::Economy;
use items::ItemDatabase;
use orders::Orders;
use systems::{
    animate_attract_icons, animate_rising_stars, handle_cell_interaction,
    handle_double_stamina_toggle, handle_drag_input, handle_order_submit, tick_attract_animation,
    tick_auto_generators, tick_economy, tick_idle_timer, tick_orders, tick_star_spawners,
    update_cell_visuals, update_double_stamina_button, update_drag_ghost, update_economy_ui,
    update_item_detail_bar, update_message_bar, update_order_icons, update_orders_ui,
};
use ui::{preload_images, setup_initial_board, setup_ui};

// ── Audio ─────────────────────────────────────────────────────────────────────


/// Marker component for the background-music entity so we can query it later.
#[derive(Component)]
struct BgmSink;

/// Spawns the looping background-music entity as soon as the game starts.
///
/// On WASM, browsers block audio autoplay until the user interacts with the
/// page, so the sink is started in a paused state and resumed by
/// [`unlock_bgm_on_interaction`] on the first input event.
fn setup_bgm(asset_server: Res<AssetServer>, mut commands: Commands) {
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
#[cfg(target_arch = "wasm32")]
fn unlock_bgm_on_interaction(
    mut unlocked: Local<bool>,
    sinks: Query<&AudioSink, With<BgmSink>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
) {
    if *unlocked {
        return;
    }
    let interacted = mouse.get_just_pressed().next().is_some()
        || keys.get_just_pressed().next().is_some()
        || touches.iter_just_pressed().next().is_some();
    if interacted {
        for sink in &sinks {
            sink.play();
        }
        *unlocked = true;
    }
}

// ── System sets ───────────────────────────────────────────────────────────────

/// Separates game-logic systems from their dependent visual-update systems so
/// that change-detection guards in the visual systems always see up-to-date data
/// within the same frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum GameSet {
    /// Tick and input-handling systems that mutate game state.
    Logic,
    /// Read-only (or nearly so) systems that refresh the UI.
    Visuals,
}

// ── Window ────────────────────────────────────────────────────────────────────

const WINDOW_W: u32 = 1280;
const WINDOW_H: u32 = 820;

// ── Layout ────────────────────────────────────────────────────────────────────

pub(crate) const TOP_BAR_H: f32 = 65.0;
/// Height of the item-detail bar shown at the bottom of the screen.
pub(crate) const DETAIL_BAR_H: f32 = 85.0;
pub(crate) const SECONDS_PER_MINUTE: f32 = 60.0;
/// Minimum pointer movement (physical pixels) before a press becomes a drag gesture.
pub(crate) const DRAG_THRESHOLD_PIXELS: f32 = 8.0;

// ── Palette ───────────────────────────────────────────────────────────────────

pub(crate) const BG: Color = Color::srgb(0.13, 0.10, 0.07);
pub(crate) const TOP_BAR_BG: Color = Color::srgb(0.09, 0.07, 0.04);
pub(crate) const BOARD_BG: Color = Color::srgb(0.16, 0.13, 0.10);
pub(crate) const DETAIL_BAR_BG: Color = Color::srgb(0.11, 0.09, 0.06);
pub(crate) const ORDER_BG: Color = Color::srgb(0.12, 0.09, 0.07);
pub(crate) const CELL_EMPTY: Color = Color::srgb(0.20, 0.17, 0.14);
pub(crate) const CELL_EMPTY_ALT: Color = Color::srgb(0.26, 0.22, 0.17);
pub(crate) const CELL_HOVERED: Color = Color::srgb(0.32, 0.26, 0.18);
pub(crate) const CELL_SELECTED: Color = Color::srgb(0.55, 0.45, 0.20);
pub(crate) const TEXT_MAIN: Color = Color::srgb(0.96, 0.91, 0.78);
pub(crate) const TEXT_MUTED: Color = Color::srgb(0.65, 0.60, 0.48);
pub(crate) const ACCENT: Color = Color::srgb(0.88, 0.72, 0.30);
pub(crate) const ACCENT_GREEN: Color = Color::srgb(0.40, 0.80, 0.45);
pub(crate) const ORDER_SLOT_BG: Color = Color::srgb(0.18, 0.14, 0.10);
/// Semi-transparent dark overlay used on icon/card backgrounds.
pub(crate) const OVERLAY_ALPHA: f32 = 0.25;

// ── Resources ─────────────────────────────────────────────────────────────────

/// Accumulated time per board cell for auto-generator ticking (cell index → secs).
#[derive(Resource, Default, Debug)]
pub(crate) struct AutoGenTimers(pub(crate) std::collections::HashMap<usize, f32>);

/// Pending eggs per auto-generator cell (cell index → count of stored eggs, max 6).
///
/// 老母鸡 accumulates up to 6 eggs (one per hour). Stored eggs are auto-placed to
/// adjacent empty cells; if no adjacent space is available the player can click the
/// hen to place one egg in the nearest empty cell (no stamina cost).
#[derive(Resource, Default, Debug)]
pub(crate) struct EggStorage(pub(crate) std::collections::HashMap<usize, u32>);

/// Number of pieces generated by each auto-generator cell in the current batch
/// (cell index → count). Resets to 0 after the cooldown expires.
pub(crate) const AUTO_GEN_BATCH_LIMIT: u32 = 12;

/// Seconds an auto-generator must wait after producing [`AUTO_GEN_BATCH_LIMIT`] pieces.
pub(crate) const AUTO_GEN_COOLDOWN_SECS: f32 = 120.0; // 2 minutes

/// Counts how many pieces each auto-generator cell has produced in its current
/// batch (cell index → count, 0..<AUTO_GEN_BATCH_LIMIT; resets to 0 when limit is hit).
#[derive(Resource, Default, Debug)]
pub(crate) struct AutoGenCounts(pub(crate) std::collections::HashMap<usize, u32>);

/// Remaining cooldown time (seconds) per auto-generator cell (cell index → secs).
/// While the value is > 0 the cell may not produce new pieces.
#[derive(Resource, Default, Debug)]
pub(crate) struct AutoGenCooldowns(pub(crate) std::collections::HashMap<usize, f32>);

/// Remaining activations for limited-use click-generators (cell index → remaining uses).
/// Used by generators with `max_generate_count > 0`; removed when the generator is consumed.
#[derive(Resource, Default, Debug)]
pub(crate) struct GeneratorUsesRemaining(pub(crate) std::collections::HashMap<usize, u32>);

/// Accumulated time for spawning rising-star animations on auto-generator cells.
#[derive(Resource, Default)]
pub(crate) struct StarSpawnTimer(pub(crate) f32);

/// Temporary message shown in the order panel.
#[derive(Resource, Default, Debug)]
pub(crate) struct MessageBar {
    pub(crate) text: String,
    pub(crate) timer: f32,
}

impl MessageBar {
    pub(crate) fn set(&mut self, msg: impl Into<String>) {
        self.text = msg.into();
        self.timer = 3.5;
    }
}

/// Tag for the message bar text label entity.
#[derive(Component)]
pub(crate) struct MessageLabel;

/// Tag for order submit buttons (so we can style them separately from board cells).
#[derive(Component)]
pub(crate) struct SubmitBtn;

/// Tracks the state of an in-progress piece drag.
#[derive(Resource, Default, Debug)]
pub(crate) struct DragState {
    /// Board cell index the drag originated from.
    pub(crate) source: Option<usize>,
    /// Physical-pixel cursor position when the mouse button was pressed.
    pub(crate) press_pos: Vec2,
    /// Latest physical-pixel cursor position (updated every frame while dragging).
    pub(crate) cursor_phys: Vec2,
    /// Latest logical-pixel cursor position (used to position the ghost node).
    pub(crate) cursor_logical: Vec2,
    /// Whether the cursor has moved far enough to be considered a real drag.
    pub(crate) dragging: bool,
    /// Asset path of the icon shown in the drag ghost.
    pub(crate) icon_path: Option<String>,
    /// Touch ID being tracked; also used to suppress mouse events while a touch
    /// gesture is active (None when driven by mouse).
    pub(crate) touch_id: Option<u64>,
}

/// Tag component for the floating drag-ghost UI entity.
#[derive(Component)]
pub(crate) struct DragGhost;

/// Tag for the item-detail bar item icon.
#[derive(Component)]
pub(crate) struct DetailIcon;

/// Tag for the item-detail bar primary text (name + level).
#[derive(Component)]
pub(crate) struct DetailName;

/// Tag for the item-detail bar secondary text (hint / action description).
#[derive(Component)]
pub(crate) struct DetailHint;

/// Component for a rising white star animation on auto-generator cells.
#[derive(Component)]
pub(crate) struct RisingStar {
    /// Elapsed time since spawn (seconds).
    pub(crate) elapsed: f32,
    /// Total lifetime of this star (seconds).
    pub(crate) lifetime: f32,
}

/// Interval in seconds between rising-star spawns on auto-generator cells.
pub(crate) const STAR_SPAWN_INTERVAL: f32 = 2.0;

// ── Attract animation ─────────────────────────────────────────────────────────

/// Seconds of user inactivity before the attract animation starts.
pub(crate) const ATTRACT_IDLE_SECS: f32 = 5.0;
/// Minimum seconds each pair is highlighted.
pub(crate) const ATTRACT_MIN_DURATION: f32 = 1.5;
/// Maximum seconds each pair is highlighted.
pub(crate) const ATTRACT_MAX_DURATION: f32 = 2.0;
/// Seconds to pause between pair switches when multiple pairs exist.
pub(crate) const ATTRACT_PAUSE_SECS: f32 = 2.0;
/// Maximum horizontal pixel offset applied to icons during the attract animation.
pub(crate) const ATTRACT_ICON_MAX_MOVE_X: f32 = 16.0;
/// Maximum vertical pixel offset applied to icons during the attract animation.
pub(crate) const ATTRACT_ICON_MAX_MOVE_Y: f32 = 8.0;

/// Tag for the 仓库 (warehouse) button in the bottom bar.
#[derive(Component)]
pub(crate) struct WarehouseButton;

/// Tag for the 活动 (activity) button in the bottom bar.
#[derive(Component)]
pub(crate) struct ActivityButton;

/// Resource that tracks whether double-stamina mode is active.
///
/// When active, non-auto-generator pieces consume 2 stamina per activation
/// instead of 1, but the generated child piece is 1 level higher than usual.
#[derive(Resource, Default)]
pub(crate) struct DoubleStaminaMode {
    pub(crate) active: bool,
}

/// Tag for the double-stamina toggle button in the top-right corner of the top bar.
#[derive(Component)]
pub(crate) struct DoubleStaminaButton;

/// Tag for the text label inside the double-stamina toggle button.
#[derive(Component)]
pub(crate) struct DoubleStaminaLabel;

// ── Idle / Attract-animation resources ───────────────────────────────────────

/// Tracks elapsed time since the last user input (mouse/touch/key).
///
/// Reset to zero on every interaction; incremented every frame. Used by
/// [`tick_attract_animation`] to decide when to start the hint animation.
#[derive(Resource, Default)]
pub(crate) struct IdleTimer {
    pub(crate) elapsed: f32,
}

/// State machine for the idle "attract" animation that shows mergeable pairs.
#[derive(Resource)]
pub(crate) struct AttractAnimState {
    /// Ordered list of all mergeable pairs `(cell_idx_a, cell_idx_b)` on the board.
    pub(crate) pairs: Vec<(usize, usize)>,
    /// Index into `pairs` for the currently displayed pair.
    pub(crate) current_pair: usize,
    /// Elapsed time (secs) the current pair has been shown.
    pub(crate) pair_elapsed: f32,
    /// Duration (secs) to display the current pair before advancing.
    pub(crate) pair_duration: f32,
    /// Whether the animation is currently in the inter-pair pause.
    pub(crate) pausing: bool,
    /// Elapsed seconds of the current inter-pair pause.
    pub(crate) pause_elapsed: f32,
    /// Whether the attract animation is currently running.
    pub(crate) active: bool,
}

impl Default for AttractAnimState {
    fn default() -> Self {
        Self {
            pairs: Vec::new(),
            current_pair: 0,
            pair_elapsed: 0.0,
            pair_duration: ATTRACT_MIN_DURATION,
            pausing: false,
            pause_elapsed: 0.0,
            active: false,
        }
    }
}

/// Component added to [`CellImage`] nodes while they are part of the active
/// attract-animation pair.  Stores the direction toward the other cell so the
/// visual system can animate the icon's scale and translation correctly.
#[derive(Component, Clone, Debug, PartialEq)]
pub(crate) struct AttractIconAnim {
    /// Signed direction toward the other cell along the column axis
    /// (+1 = right, −1 = left, 0 = same column).
    pub(crate) dir_x: f32,
    /// Signed direction toward the other cell along the row axis
    /// (+1 = down, −1 = up, 0 = same row).
    pub(crate) dir_y: f32,
}

/// Holds [`Handle<Image>`] values for every item icon loaded at startup.
///
/// Keeping these handles alive ensures Bevy never unloads the textures, so all
/// icon images are ready in the GPU texture cache the first time any cell or
/// order card tries to display them.
#[derive(Resource, Default)]
pub(crate) struct PreloadedImages(pub(crate) Vec<Handle<Image>>);

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "合合游戏 (HeHe Game)".to_string(),
                    resolution: (WINDOW_W, WINDOW_H).into(),
                    resizable: false,
                    // On WASM, fit into the browser canvas rather than opening a new window.
                    #[cfg(target_arch = "wasm32")]
                    canvas: Some("#bevy-canvas".to_string()),
                    #[cfg(target_arch = "wasm32")]
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
            // Disable .meta file loading: the assets directory ships no .meta files,
            // so the default behaviour causes spurious 404 HTTP requests in WASM.
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .insert_resource(ClearColor(BG))
    .init_resource::<Board>()
    .init_resource::<Economy>()
    .init_resource::<Orders>()
    .insert_resource(ItemDatabase::new())
    .insert_resource(AutoGenTimers::default())
    .insert_resource(EggStorage::default())
    .insert_resource(AutoGenCounts::default())
    .insert_resource(AutoGenCooldowns::default())
    .insert_resource(GeneratorUsesRemaining::default())
    .insert_resource(StarSpawnTimer::default())
    .insert_resource(MessageBar::default())
    .insert_resource(DragState::default())
    .insert_resource(IdleTimer::default())
    .insert_resource(AttractAnimState::default())
    .insert_resource(DoubleStaminaMode::default())
    .init_resource::<PreloadedImages>()
    .add_systems(Startup, preload_images)
    .add_systems(Startup, setup_initial_board)
    .add_systems(Startup, setup_ui.after(setup_initial_board))
    .add_systems(Startup, setup_bgm);
    // On WASM, resume the paused BGM on the first user interaction to work
    // around browsers' autoplay policy.
    #[cfg(target_arch = "wasm32")]
    app.add_systems(Update, unlock_bgm_on_interaction);
    app
        // Logic systems run first; visual systems run after, ensuring change-detection
        // guards in visual systems always observe the latest game state.
        .configure_sets(Update, GameSet::Logic.before(GameSet::Visuals))
        .add_systems(
            Update,
            (
                tick_economy,
                tick_orders,
                tick_auto_generators,
                tick_star_spawners,
                tick_idle_timer,
                tick_attract_animation,
                handle_drag_input,
                handle_cell_interaction,
                handle_order_submit,
                handle_double_stamina_toggle,
            )
                .in_set(GameSet::Logic),
        )
        .add_systems(
            Update,
            (
                update_drag_ghost,
                update_cell_visuals,
                update_economy_ui,
                update_orders_ui,
                update_order_icons,
                update_item_detail_bar,
                update_message_bar,
                update_double_stamina_button,
                animate_rising_stars,
                animate_attract_icons.after(update_cell_visuals),
            )
                .in_set(GameSet::Visuals),
        )
        .run();
}
