//! Animation systems for rising stars and the attract idle hint.
use bevy::prelude::*;

use crate::{
    AttractAnimState, AttractIconAnim, IdleTimer, RisingStar, StarSpawnTimer,
    ATTRACT_IDLE_SECS, ATTRACT_ICON_MAX_MOVE_X, ATTRACT_ICON_MAX_MOVE_Y,
    ATTRACT_MAX_DURATION, ATTRACT_MIN_DURATION, ATTRACT_PAUSE_SECS,
    STAR_SPAWN_INTERVAL,
};
use crate::board::{Board, BoardCell, CellImage};
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

/// Add `AttractIconAnim` to the two `CellImage` entities in the active pair.
fn attach_pair_anim(
    commands: &mut Commands,
    cell_image_query: &Query<(Entity, &CellImage)>,
    anim: &AttractAnimState,
) {
    let (idx_a, idx_b) = anim.pairs[anim.current_pair];
    let (col_a, row_a) = Board::pos(idx_a);
    let (col_b, row_b) = Board::pos(idx_b);
    let dir_x = (col_b as f32 - col_a as f32).signum();
    let dir_y = (row_b as f32 - row_a as f32).signum();

    for (entity, ci) in cell_image_query {
        if ci.index == idx_a {
            commands
                .entity(entity)
                .insert(AttractIconAnim { dir_x, dir_y });
        } else if ci.index == idx_b {
            commands
                .entity(entity)
                .insert(AttractIconAnim { dir_x: -dir_x, dir_y: -dir_y });
        }
    }
}

/// Remove `AttractIconAnim` from every `CellImage` entity.
fn detach_all_anim(commands: &mut Commands, cell_image_query: &Query<(Entity, &CellImage)>) {
    for (entity, _) in cell_image_query {
        commands.entity(entity).remove::<AttractIconAnim>();
    }
}

/// Compute the display duration for a given pair index using a deterministic
/// Halton-like scatter so consecutive pairs have varied but reproducible durations.
fn pair_duration_for(pair_index: usize) -> f32 {
    let frac = ((pair_index * 137 + 53) % 100) as f32 / 100.0;
    ATTRACT_MIN_DURATION + frac * (ATTRACT_MAX_DURATION - ATTRACT_MIN_DURATION)
}

/// Manage the idle attract animation: find all mergeable pairs, cycle through
/// them, and animate their cell icons (scale + translate toward each other).
///
/// The animation starts after [`ATTRACT_IDLE_SECS`] seconds of inactivity and
/// stops immediately when the player interacts again.  Each pair is shown for a
/// duration in the range [`ATTRACT_MIN_DURATION`]..=[`ATTRACT_MAX_DURATION`],
/// followed by a [`ATTRACT_PAUSE_SECS`] pause before the next pair is shown.
pub(crate) fn tick_attract_animation(
    time: Res<Time>,
    idle: Res<IdleTimer>,
    board: Res<Board>,
    db: Res<ItemDatabase>,
    mut anim: ResMut<AttractAnimState>,
    mut commands: Commands,
    cell_image_query: Query<(Entity, &CellImage)>,
) {
    let delta = time.delta_secs();

    // ── Deactivate while the player is active ─────────────────────────────────
    if idle.elapsed < ATTRACT_IDLE_SECS {
        if anim.active {
            anim.active = false;
            anim.pausing = false;
            anim.pause_elapsed = 0.0;
            anim.pairs.clear();
            detach_all_anim(&mut commands, &cell_image_query);
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
                anim.pausing = false;
                detach_all_anim(&mut commands, &cell_image_query);
            }
            anim.pairs.clear();
            return;
        }

        let pairs_differ = new_pairs != anim.pairs;
        anim.pairs = new_pairs;

        if pairs_differ || !anim.active || anim.current_pair >= anim.pairs.len() {
            // Pairs changed or current index is out of range: restart from pair 0.
            anim.current_pair = 0;
            anim.pair_elapsed = 0.0;
            anim.pausing = false;
            anim.pause_elapsed = 0.0;
            anim.active = true;
            detach_all_anim(&mut commands, &cell_image_query);
            attach_pair_anim(&mut commands, &cell_image_query, &anim);
        } else {
            anim.active = true;
        }
        return;
    }

    if !anim.active || anim.pairs.is_empty() {
        return;
    }

    // ── Handle inter-pair pause ────────────────────────────────────────────────
    if anim.pausing {
        anim.pause_elapsed += delta;
        if anim.pause_elapsed >= ATTRACT_PAUSE_SECS {
            anim.pausing = false;
            anim.pause_elapsed = 0.0;
            // `pairs` is guaranteed non-empty by the guard on line 224 above.
            anim.current_pair = (anim.current_pair + 1) % anim.pairs.len();
            anim.pair_elapsed = 0.0;
            anim.pair_duration = pair_duration_for(anim.current_pair);
            detach_all_anim(&mut commands, &cell_image_query);
            attach_pair_anim(&mut commands, &cell_image_query, &anim);
        }
        return;
    }

    // ── Advance pair timer ────────────────────────────────────────────────────
    anim.pair_elapsed += delta;

    // ── Advance to the next pair when the current one has played long enough ──
    if anim.pair_elapsed >= anim.pair_duration {
        anim.pair_elapsed = 0.0;
        if anim.pairs.len() > 1 {
            // Multiple pairs: enter a 2-second pause before showing the next pair.
            anim.pausing = true;
            anim.pause_elapsed = 0.0;
            detach_all_anim(&mut commands, &cell_image_query);
        }
        // Single pair: reset elapsed so the sine-wave animation loops from the
        // start again, and keep AttractIconAnim attached for uninterrupted pulsing.
    }
}

/// Animate the icons of the active attract pair: scale up to 1.5× and
/// translate toward the other cell.  Icons without [`AttractIconAnim`] are
/// reset to their identity transform.
///
/// Runs in [`GameSet::Visuals`] after [`update_cell_visuals`].
pub(crate) fn animate_attract_icons(
    anim: Res<AttractAnimState>,
    mut cell_image_query: Query<(Option<&AttractIconAnim>, &mut Transform), With<CellImage>>,
) {
    // Smooth sine-wave envelope: 0 → 1 → 0 over ~1 second, repeating.
    // Using |sin(π · t)| gives a ping-pong that starts and ends at 0.
    let anim_t = if anim.active && !anim.pausing {
        (anim.pair_elapsed * std::f32::consts::PI).sin().abs()
    } else {
        0.0
    };

    let scale = 1.0 + 0.5 * anim_t; // 1.0 at rest → 1.5 at peak

    for (opt_icon_anim, mut transform) in &mut cell_image_query {
        let new_transform = if let Some(icon_anim) = opt_icon_anim {
            let tx = icon_anim.dir_x * ATTRACT_ICON_MAX_MOVE_X * anim_t;
            let ty = -icon_anim.dir_y * ATTRACT_ICON_MAX_MOVE_Y * anim_t;
            Transform {
                scale: Vec3::new(scale, scale, 1.0),
                translation: Vec3::new(tx, ty, 0.0),
                ..default()
            }
        } else {
            Transform::IDENTITY
        };
        transform.set_if_neq(new_transform);
    }
}
