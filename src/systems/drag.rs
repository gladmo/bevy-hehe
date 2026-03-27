//! Drag-and-drop input handling systems.
use bevy::prelude::*;

use crate::{DragGhost, DragState, DRAG_THRESHOLD_PIXELS, MessageBar};
use crate::board::{Board, BoardCell, ClickAction};
use crate::economy::Economy;
use crate::items::ItemDatabase;

/// Returns `true` when the physical-pixel `cursor` lies within the UI node described by
/// `computed` at `transform`.
fn ui_hit_test(cursor: Vec2, transform: &UiGlobalTransform, computed: &ComputedNode) -> bool {
    computed.contains_point(*transform, cursor)
}

/// Shared logic executed when a drag gesture is completed (mouse or touch).
/// Applies merge / move / swap to the board and updates the UI message.
fn finish_drag(
    src: usize,
    release_phys: Vec2,
    board: &mut Board,
    db: &ItemDatabase,
    economy: &mut Economy,
    message: &mut MessageBar,
    cell_query: &Query<(&BoardCell, &UiGlobalTransform, &ComputedNode)>,
) {
    let mut target_idx: Option<usize> = None;
    for (cell, transform, computed) in cell_query {
        if ui_hit_test(release_phys, transform, computed) {
            target_idx = Some(cell.index);
            break;
        }
    }
    if let Some(tgt) = target_idx {
        if tgt != src {
            let action = board.handle_drag(src, tgt, db);
            match action {
                ClickAction::Merged { result, .. } => {
                    if let Some(item) = db.get(&result) {
                        let hint = if item.is_generator { "（生成器！）" } else { "" };
                        message.set(format!(
                            "合成成功！{} Lv{}{}",
                            item.name, item.level, hint
                        ));
                        economy.add_exp(10 * item.level as u64);
                    }
                }
                ClickAction::Moved { item, .. } => {
                    if let Some(def) = db.get(&item) {
                        message.set(format!("移动了 {}", def.name));
                    }
                }
                ClickAction::Swapped { .. } => {
                    message.set("已互换位置");
                }
                _ => {}
            }
        }
    }
}

/// Handles the full lifecycle of a drag gesture (mouse **and** touch):
/// press → movement threshold → ghost appears → release → move or merge.
pub(crate) fn handle_drag_input(
    mut drag: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    mut board: ResMut<Board>,
    db: Res<ItemDatabase>,
    mut economy: ResMut<Economy>,
    mut message: ResMut<MessageBar>,
    cell_query: Query<(&BoardCell, &UiGlobalTransform, &ComputedNode)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = window.scale_factor();

    // ── Touch pressed ─────────────────────────────────────────────────────────
    for touch in touches.iter_just_pressed() {
        if drag.touch_id.is_some() {
            continue; // already tracking a finger
        }
        let cursor_logical = touch.position();
        let cursor_phys = cursor_logical * scale;
        drag.source = None;
        drag.dragging = false;
        drag.touch_id = Some(touch.id());
        for (cell, transform, computed) in &cell_query {
            if ui_hit_test(cursor_phys, transform, computed) {
                if let Some(item_id) = board.cells[cell.index].item_id.as_deref() {
                    if let Some(def) = db.get(item_id) {
                        drag.source = Some(cell.index);
                        drag.press_pos = cursor_phys;
                        drag.cursor_phys = cursor_phys;
                        drag.cursor_logical = cursor_logical;
                        drag.icon_path = def.icon_path.clone();
                    }
                }
                break;
            }
        }
    }

    // ── Touch movement ────────────────────────────────────────────────────────
    if let Some(touch_id) = drag.touch_id {
        for touch in touches.iter() {
            if touch.id() == touch_id {
                let cursor_logical = touch.position();
                let cursor_phys = cursor_logical * scale;
                drag.cursor_phys = cursor_phys;
                drag.cursor_logical = cursor_logical;
                if !drag.dragging && cursor_phys.distance(drag.press_pos) > DRAG_THRESHOLD_PIXELS {
                    drag.dragging = true;
                }
                break;
            }
        }
    }

    // ── Touch released ────────────────────────────────────────────────────────
    for touch in touches.iter_just_released() {
        if Some(touch.id()) != drag.touch_id {
            continue;
        }
        if drag.dragging {
            if let Some(src) = drag.source {
                let cursor_logical = touch.position();
                let cursor_phys = cursor_logical * scale;
                finish_drag(
                    src,
                    cursor_phys,
                    &mut board,
                    &db,
                    &mut economy,
                    &mut message,
                    &cell_query,
                );
            }
        }
        drag.source = None;
        drag.dragging = false;
        drag.touch_id = None;
    }

    // ── Mouse input (skip when a touch gesture is active) ─────────────────────
    if drag.touch_id.is_some() {
        return;
    }

    // Physical pixels for hit-testing against UiGlobalTransform.
    let Some(cursor_phys) = window.physical_cursor_position() else {
        return;
    };
    // Logical pixels for positioning the ghost node (Val::Px is logical).
    let Some(cursor_logical) = window.cursor_position() else {
        return;
    };

    // ── Mouse pressed ─────────────────────────────────────────────────────────
    if mouse.just_pressed(MouseButton::Left) {
        drag.source = None;
        drag.dragging = false;
        for (cell, transform, computed) in &cell_query {
            if ui_hit_test(cursor_phys, transform, computed) {
                if let Some(item_id) = board.cells[cell.index].item_id.as_deref() {
                    if let Some(def) = db.get(item_id) {
                        drag.source = Some(cell.index);
                        drag.press_pos = cursor_phys;
                        drag.cursor_phys = cursor_phys;
                        drag.cursor_logical = cursor_logical;
                        drag.icon_path = def.icon_path.clone();
                    }
                }
                break;
            }
        }
    }

    // ── Track movement ────────────────────────────────────────────────────────
    if drag.source.is_some() && mouse.pressed(MouseButton::Left) {
        drag.cursor_phys = cursor_phys;
        drag.cursor_logical = cursor_logical;
        if !drag.dragging && cursor_phys.distance(drag.press_pos) > DRAG_THRESHOLD_PIXELS {
            drag.dragging = true;
        }
    }

    // ── Mouse released ────────────────────────────────────────────────────────
    if mouse.just_released(MouseButton::Left) {
        if drag.dragging {
            if let Some(src) = drag.source {
                finish_drag(
                    src,
                    cursor_phys,
                    &mut board,
                    &db,
                    &mut economy,
                    &mut message,
                    &cell_query,
                );
            }
        }
        // Always reset drag state on release
        drag.source = None;
        drag.dragging = false;
    }
}

/// Moves the drag-ghost node to the cursor and loads the correct icon image.
///
/// Skips all work when `DragState` has not been mutated this frame, which
/// is the common case when no drag gesture is in progress.
pub(crate) fn update_drag_ghost(
    drag: Res<DragState>,
    asset_server: Res<AssetServer>,
    mut ghost_q: Query<(&mut Node, &mut ImageNode), With<DragGhost>>,
) {
    // DragState is only mutated by handle_drag_input when input events occur.
    // Skip the ghost update entirely when nothing has changed.
    if !drag.is_changed() {
        return;
    }

    let Ok((mut node, mut img)) = ghost_q.single_mut() else {
        return;
    };

    if drag.dragging {
        node.display = Display::Flex;
        // Centre the ghost on the cursor (ghost is 56×56 logical pixels)
        node.left = Val::Px(drag.cursor_logical.x - 28.0);
        node.top = Val::Px(drag.cursor_logical.y - 28.0);
        if let Some(ref path) = drag.icon_path {
            img.image = asset_server.load(path.clone());
        }
    } else {
        node.display = Display::None;
    }
}
