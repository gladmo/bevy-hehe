//! Animation systems for rising stars and the attract idle hint.
use bevy::prelude::*;

use crate::{
    AttractAnimState, AttractSymbol, IdleTimer, RisingStar, StarSpawnTimer,
    ATTRACT_IDLE_SECS, ATTRACT_MAX_DURATION, ATTRACT_MIN_DURATION, ATTRACT_SPARKLE_INTERVAL,
    STAR_SPAWN_INTERVAL,
};
use crate::board::{Board, BoardCell};
use crate::items::ItemDatabase;

/// Spawn rising white star "✦" animations on auto-generator cells every STAR_SPAWN_INTERVAL.
pub(crate) fn tick_star_spawners(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<StarSpawnTimer>,
    board: Res<Board>,
    db: Res<ItemDatabase>,
    asset_server: Res<AssetServer>,
    cell_query: Query<(Entity, &BoardCell)>,
) {
    timer.0 += time.delta_secs();
    if timer.0 < STAR_SPAWN_INTERVAL {
        return;
    }
    timer.0 -= STAR_SPAWN_INTERVAL;

    let font: Handle<Font> = asset_server.load("fonts/SourceHanSansSC-Medium.otf");

    for (entity, cell) in &cell_query {
        let is_auto_gen = board.cells[cell.index]
            .item_id
            .as_deref()
            .and_then(|id| db.get(id))
            .map(|def| def.is_auto_generator)
            .unwrap_or(false);

        if is_auto_gen {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Text::new("✦"),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(4.0),
                        left: Val::Px(20.0),
                        ..default()
                    },
                    RisingStar {
                        elapsed: 0.0,
                        lifetime: 1.8,
                    },
                    Pickable::IGNORE,
                    ZIndex(100),
                ));
            });
        }
    }
}

/// Animate rising stars: move upward and fade out, then despawn.
pub(crate) fn animate_rising_stars(
    mut commands: Commands,
    time: Res<Time>,
    mut star_q: Query<(Entity, &mut RisingStar, &mut Node, &mut TextColor)>,
) {
    for (entity, mut star, mut node, mut color) in &mut star_q {
        star.elapsed += time.delta_secs();
        let t = (star.elapsed / star.lifetime).min(1.0);

        node.top = Val::Px(4.0 - t * 48.0);
        color.0 = Color::srgba(1.0, 1.0, 1.0, (1.0 - t) * 0.9);

        if star.elapsed >= star.lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Reset the idle timer on any user input; otherwise increment it each frame.
///
/// Runs in [`GameSet::Logic`] before [`tick_attract_animation`] so the attract
/// system always observes the most up-to-date idle elapsed time.
pub(crate) fn tick_idle_timer(
    time: Res<Time>,
    mut idle: ResMut<IdleTimer>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
) {
    let any_input = mouse.get_just_pressed().next().is_some()
        || mouse.get_just_released().next().is_some()
        || keys.get_just_pressed().next().is_some()
        || touches.iter_just_pressed().next().is_some()
        || touches.iter_just_released().next().is_some();

    if any_input {
        idle.elapsed = 0.0;
    } else {
        idle.elapsed += time.delta_secs();
    }
}

/// Manage the idle attract animation: find all mergeable pairs, cycle through
/// them, and spawn golden "✦" sparkle children on each pair's cells.
///
/// The animation starts after [`ATTRACT_IDLE_SECS`] seconds of inactivity and
/// stops immediately when the player interacts again.  Each pair is shown for a
/// duration in the range [`ATTRACT_MIN_DURATION`]..=[`ATTRACT_MAX_DURATION`].
pub(crate) fn tick_attract_animation(
    time: Res<Time>,
    idle: Res<IdleTimer>,
    board: Res<Board>,
    db: Res<ItemDatabase>,
    mut anim: ResMut<AttractAnimState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cell_query: Query<(Entity, &BoardCell)>,
    symbol_query: Query<Entity, With<AttractSymbol>>,
) {
    let delta = time.delta_secs();

    // ── Helper: despawn all existing sparkle symbols ──────────────────────────
    let despawn_symbols = |cmds: &mut Commands, q: &Query<Entity, With<AttractSymbol>>| {
        for entity in q {
            cmds.entity(entity).despawn();
        }
    };

    // ── Deactivate while the player is active ─────────────────────────────────
    if idle.elapsed < ATTRACT_IDLE_SECS {
        if anim.active {
            anim.active = false;
            anim.pairs.clear();
            despawn_symbols(&mut commands, &symbol_query);
        }
        return;
    }

    // ── Refresh pairs whenever the board changes or on initial activation ─────
    let need_refresh = board.is_changed() || (!anim.active && anim.pairs.is_empty());
    if need_refresh {
        // Group board cells by item ID (only items that can be merged).
        let mut id_groups: std::collections::HashMap<&str, Vec<usize>> =
            std::collections::HashMap::new();
        for (idx, cell) in board.cells.iter().enumerate() {
            if let Some(id) = cell.item_id.as_deref() {
                if db.can_merge(id, id) {
                    id_groups.entry(id).or_default().push(idx);
                }
            }
        }

        let mut new_pairs: Vec<(usize, usize)> = Vec::new();
        for indices in id_groups.values() {
            for i in 0..indices.len() {
                for j in (i + 1)..indices.len() {
                    new_pairs.push((indices[i], indices[j]));
                }
            }
        }

        if new_pairs.is_empty() {
            // No mergeable pairs: stop the animation.
            if anim.active {
                anim.active = false;
                despawn_symbols(&mut commands, &symbol_query);
            }
            anim.pairs.clear();
            return;
        }

        let pairs_differ = new_pairs != anim.pairs;
        anim.pairs = new_pairs;
        anim.active = true;

        if pairs_differ || anim.current_pair >= anim.pairs.len() {
            // Pairs changed or current index is out of range: restart from pair 0.
            anim.current_pair = 0;
            anim.pair_elapsed = 0.0;
            anim.sparkle_timer = ATTRACT_SPARKLE_INTERVAL; // spawn immediately
            despawn_symbols(&mut commands, &symbol_query);
        }
    }

    if !anim.active || anim.pairs.is_empty() {
        return;
    }

    // ── Advance timers ────────────────────────────────────────────────────────
    anim.pair_elapsed += delta;
    anim.sparkle_timer += delta;

    // ── Spawn a sparkle wave when the interval elapses ────────────────────────
    if anim.sparkle_timer >= ATTRACT_SPARKLE_INTERVAL {
        anim.sparkle_timer -= ATTRACT_SPARKLE_INTERVAL;

        let (idx_a, idx_b) = anim.pairs[anim.current_pair];
        let (col_a, row_a) = Board::pos(idx_a);
        let (col_b, row_b) = Board::pos(idx_b);
        let dir_x = (col_b as f32 - col_a as f32).signum();
        let dir_y = (row_b as f32 - row_a as f32).signum();

        let font: Handle<Font> = asset_server.load("fonts/SourceHanSansSC-Medium.otf");

        // Helper: spawn a sparkle as a child of the given cell entity.
        let spawn_sparkle =
            |parent: &mut ChildSpawnerCommands, sx: f32, sy: f32, f: Handle<Font>| {
                parent.spawn((
                    Text::new("✦"),
                    TextFont {
                        font: f,
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 0.88, 0.30, 0.0)),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(10.0),
                        left: Val::Px(18.0),
                        ..default()
                    },
                    AttractSymbol {
                        elapsed: 0.0,
                        lifetime: 0.75,
                        dir_x: sx,
                        dir_y: sy,
                    },
                    Pickable::IGNORE,
                    ZIndex(100),
                ));
            };

        // Sparkle on cell A moves toward cell B.
        if let Some((entity_a, _)) = cell_query.iter().find(|(_, c)| c.index == idx_a) {
            let f = font.clone();
            commands
                .entity(entity_a)
                .with_children(|p| spawn_sparkle(p, dir_x, dir_y, f));
        }
        // Sparkle on cell B moves toward cell A.
        if let Some((entity_b, _)) = cell_query.iter().find(|(_, c)| c.index == idx_b) {
            commands
                .entity(entity_b)
                .with_children(|p| spawn_sparkle(p, -dir_x, -dir_y, font));
        }
    }

    // ── Advance to the next pair when the current one has played long enough ──
    if anim.pair_elapsed >= anim.pair_duration {
        anim.pair_elapsed -= anim.pair_duration;
        anim.current_pair = (anim.current_pair + 1) % anim.pairs.len();

        // Vary the duration using a deterministic but irregular sequence.
        // 137 is a prime-step Halton-like scatter; 53 offsets from zero; 100 is
        // the integer range that maps to the [0, 1) fraction.
        let frac = ((anim.current_pair * 137 + 53) % 100) as f32 / 100.0;
        anim.pair_duration =
            ATTRACT_MIN_DURATION + frac * (ATTRACT_MAX_DURATION - ATTRACT_MIN_DURATION);

        // Despawn sparkles from the old pair.
        despawn_symbols(&mut commands, &symbol_query);
        // Trigger immediate sparkle spawn for the new pair.
        anim.sparkle_timer = ATTRACT_SPARKLE_INTERVAL;
    }
}

/// Animate the attract sparkle symbols: translate toward the other cell and fade.
///
/// Runs in [`GameSet::Visuals`].
pub(crate) fn animate_attract_symbols(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut AttractSymbol, &mut Node, &mut TextColor)>,
) {
    for (entity, mut sym, mut node, mut color) in &mut q {
        sym.elapsed += time.delta_secs();
        let t = (sym.elapsed / sym.lifetime).min(1.0);

        // Translate up to 14 px vertically and 22 px horizontally toward the other cell.
        node.top = Val::Px(10.0 + sym.dir_y * t * 14.0);
        node.left = Val::Px(18.0 + sym.dir_x * t * 22.0);

        // Fade in then out using a sine envelope.
        let alpha = (t * std::f32::consts::PI).sin() * 0.9;
        color.0 = Color::srgba(1.0, 0.88, 0.30, alpha);

        if sym.elapsed >= sym.lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Apply a pulsing golden border to the two cells in the currently highlighted
/// attract pair.  Runs in [`GameSet::Visuals`] *after* [`update_cell_visuals`]
/// so the glow takes precedence over the normal border colour for those cells.
pub(crate) fn animate_attract_cells(
    anim: Res<AttractAnimState>,
    mut cell_query: Query<(&BoardCell, &mut BorderColor)>,
) {
    if !anim.active || anim.pairs.is_empty() {
        return;
    }

    let (idx_a, idx_b) = anim.pairs[anim.current_pair];

    // Pulse at ~1.5 Hz: base alpha 0.40, sine amplitude 0.55 → range [−0.15, 0.95].
    // Clamped to [0, 1] to avoid negative or over-saturated alpha values.
    let alpha = 0.40 + 0.55 * (anim.pair_elapsed * std::f32::consts::TAU * 1.5).sin();
    let alpha = alpha.clamp(0.0, 1.0);

    for (cell, mut border) in &mut cell_query {
        if cell.index == idx_a || cell.index == idx_b {
            border.set_if_neq(BorderColor::all(Color::srgba(0.88, 0.72, 0.30, alpha)));
        }
    }
}
