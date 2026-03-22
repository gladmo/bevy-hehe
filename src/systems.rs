//! Update systems for 合合游戏 (HeHe Game).
use bevy::prelude::*;

use crate::{
    AutoGenTimers, EggStorage, DetailHint, DetailIcon, DetailName, DragGhost, DragState,
    MessageBar, MessageLabel, RisingStar, StarSpawnTimer, STAR_SPAWN_INTERVAL, SubmitBtn,
    ACCENT, CELL_EMPTY, CELL_EMPTY_ALT, CELL_HOVERED, CELL_SELECTED,
    DRAG_THRESHOLD_PIXELS, SECONDS_PER_MINUTE,
};
use crate::board::{Board, BoardCell, CellImage, ClickAction, BOARD_COLS};
use crate::economy::{CoinsLabel, Economy, GemsLabel, LevelLabel, StaminaLabel};
use crate::items::ItemDatabase;
use crate::orders::{OrderItemIcon, OrderSubmitButton, Orders};

pub(crate) fn tick_economy(time: Res<Time>, mut economy: ResMut<Economy>) {
    economy.tick(time.delta_secs());
}

pub(crate) fn tick_orders(
    mut orders: ResMut<Orders>,
    db: Res<ItemDatabase>,
) {
    // Ensure all order slots are filled whenever one becomes vacant.
    if orders.orders.len() < crate::orders::MAX_ORDERS {
        orders.fill_orders(&db);
    }
}

pub(crate) fn tick_auto_generators(
    time: Res<Time>,
    mut board: ResMut<Board>,
    db: Res<ItemDatabase>,
    mut timers: ResMut<AutoGenTimers>,
    mut egg_storage: ResMut<EggStorage>,
    mut message: ResMut<MessageBar>,
) {
    let delta = time.delta_secs();

    // Collect auto-generator info (avoid borrow conflicts)
    let generators: Vec<(usize, f32, String)> = board
        .cells
        .iter()
        .enumerate()
        .filter_map(|(idx, cell)| {
            let id = cell.item_id.as_deref()?;
            let def = db.get(id)?;
            if def.is_auto_generator {
                let gen_id = def.generates_id?.to_string();
                Some((idx, def.auto_gen_interval_secs, gen_id))
            } else {
                None
            }
        })
        .collect();

    for (idx, interval, gen_id) in generators {
        let acc = timers.0.entry(idx).or_insert(0.0);
        *acc += delta;
        if *acc >= interval {
            *acc -= interval;
            // Add one egg to storage (max 6 stored eggs)
            let stored = egg_storage.0.entry(idx).or_insert(0);
            if *stored < 6 {
                *stored += 1;
            }
            // Warn when the board is completely full and there are pending eggs
            // (check here so the message fires once per interval, not every frame)
            let stored_now = *egg_storage.0.get(&idx).unwrap_or(&0);
            if stored_now > 0 && board.first_empty().is_none() {
                message.set("棋盘已满，无法放置鸡蛋！");
            }
        }

        // Try to auto-place a pending egg to the nearest empty cell each frame
        let stored = *egg_storage.0.get(&idx).unwrap_or(&0);
        if stored > 0 {
            if let Some(near_idx) = board.nearest_empty(idx) {
                board.place(near_idx, &gen_id);
                let s = egg_storage.0.entry(idx).or_insert(0);
                *s = s.saturating_sub(1);
            }
        }
    }
}

pub(crate) fn handle_cell_interaction(
    mut board: ResMut<Board>,
    db: Res<ItemDatabase>,
    mut economy: ResMut<Economy>,
    mut message: ResMut<MessageBar>,
    mut egg_storage: ResMut<EggStorage>,
    interaction_query: Query<(&Interaction, &BoardCell), Changed<Interaction>>,
) {
    for (interaction, cell) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let action = board.handle_click(cell.index, &db);
        match action {
            ClickAction::Merged { result, .. } => {
                if let Some(item) = db.get(&result) {
                    let hint = if item.is_generator {
                        "（生成器！）"
                    } else {
                        ""
                    };
                    message.set(format!(
                        "合成成功！{} {} Lv{}{}",
                        item.emoji, item.name, item.level, hint
                    ));
                    economy.add_exp(10 * item.level as u64);
                }
            }
            ClickAction::GeneratorActivated(idx, item_id) => {
                if let Some(item) = db.get(&item_id) {
                    if item.is_auto_generator {
                        // 老母鸡: place an egg (no stamina cost).
                        // Consumes a stored egg from EggStorage when available;
                        // otherwise produces one on the spot so that double-clicking
                        // always works regardless of how long the hen has been running.
                        if let Some(gen_id) = item.generates_id {
                            if board.place_near(idx, gen_id) {
                                let pending = egg_storage.0.get(&idx).copied().unwrap_or(0);
                                if pending > 0 {
                                    let s = egg_storage.0.entry(idx).or_insert(0);
                                    *s = s.saturating_sub(1);
                                    let remaining = *s;
                                    if remaining > 0 {
                                        message.set(format!(
                                            "放置了鸡蛋！还有 {} 枚待放置",
                                            remaining
                                        ));
                                    } else {
                                        message.set("鸡蛋已全部放置！");
                                    }
                                } else {
                                    message.set(format!("产出了 {} 鸡蛋！", item.emoji));
                                }
                            } else {
                                message.set("棋盘已满，无法放置鸡蛋！");
                            }
                        }
                    } else if item.is_generator {
                        let mut rng = rand::thread_rng();
                        if let Some(gen_id) = item.pick_generated_item(&mut rng) {
                            if economy.spend_stamina(1) {
                                if board.place_near(idx, gen_id) {
                                    if let Some(gen_item) = db.get(gen_id) {
                                        message.set(format!(
                                            "生成了 {} {}！剩余体力 {}",
                                            gen_item.emoji, gen_item.name, economy.stamina,
                                        ));
                                    }
                                } else {
                                    // Board full — refund stamina
                                    economy.stamina =
                                        (economy.stamina + 1).min(economy.max_stamina);
                                    message.set("棋盘已满，无法生成！");
                                }
                            } else {
                                message.set(format!(
                                    "体力不足（{}），等待恢复（2分钟+1）",
                                    economy.stamina
                                ));
                            }
                        }
                    }
                }
            }
            ClickAction::Selected(idx) => {
                if let Some(id) = board.cells[idx].item_id.clone() {
                    if let Some(item) = db.get(&id) {
                        let hint = if item.is_auto_generator {
                            let pending = egg_storage.0.get(&idx).copied().unwrap_or(0);
                            format!(
                                "— 自动产蛋（每 {:.0} 分钟 1 枚，存 {}/6 枚），再次点击放置到最近空位",
                                item.auto_gen_interval_secs / SECONDS_PER_MINUTE,
                                pending
                            )
                        } else if item.is_generator {
                            if economy.stamina >= 1 {
                                format!("— 再次点击生成（耗1体力，剩余体力：{}）", economy.stamina)
                            } else {
                                "— 体力不足，无法生成子棋".to_string()
                            }
                        } else if item.merge_result_id.is_some() {
                            "— 点击同类同级棋子合成".to_string()
                        } else {
                            "— 最高级！".to_string()
                        };
                        message.set(format!(
                            "已选 {} {} Lv{} {}",
                            item.emoji, item.name, item.level, hint
                        ));
                    }
                }
            }
            ClickAction::Moved { item, .. } => {
                if let Some(def) = db.get(&item) {
                    message.set(format!("移动了 {} {}", def.emoji, def.name));
                }
            }
            ClickAction::Deselected => {
                message.set("取消选中");
            }
            ClickAction::Swapped { .. } => {}
            ClickAction::None => {}
        }
    }
}

pub(crate) fn handle_order_submit(
    mut board: ResMut<Board>,
    mut economy: ResMut<Economy>,
    mut orders: ResMut<Orders>,
    db: Res<ItemDatabase>,
    mut message: ResMut<MessageBar>,
    interaction_query: Query<(&Interaction, &OrderSubmitButton), Changed<Interaction>>,
) {
    for (interaction, submit_btn) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let slot = submit_btn.order_id as usize;
        if let Some(order) = orders.orders.get(slot) {
            let order_id = order.id;
            let board_items: Vec<Option<String>> =
                board.cells.iter().map(|c| c.item_id.clone()).collect();

            if let Some((reward, cells)) = orders.try_fulfill(order_id, &board_items) {
                for cell_idx in cells {
                    board.cells[cell_idx].item_id = None;
                    board.dirty = true;
                }
                economy.add_coins(reward);
                economy.add_exp(50);
                orders.fill_orders(&db);
                message.set(format!("订单完成！获得 {} 铜板", reward));
            }
        }
    }
}

// ── Drag-and-drop systems ─────────────────────────────────────────────────────

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
                            "合成成功！{} {} Lv{}{}",
                            item.emoji, item.name, item.level, hint
                        ));
                        economy.add_exp(10 * item.level as u64);
                    }
                }
                ClickAction::Moved { item, .. } => {
                    if let Some(def) = db.get(&item) {
                        message.set(format!("移动了 {} {}", def.emoji, def.name));
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
                        drag.icon_path = def.icon_path;
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
                        drag.icon_path = def.icon_path;
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
        if let Some(path) = drag.icon_path {
            img.image = asset_server.load(path);
        }
    } else {
        node.display = Display::None;
    }
}

/// Refresh all 63 board cell visuals.
///
/// This system is change-detection aware:
/// - It skips entirely when neither the board, the drag state, nor any cell's
///   `Interaction` component has changed this frame.
/// - Background and border colors use `set_if_neq` so only cells whose visual
///   state truly differs mark their components as changed for the renderer.
/// - Cell icon images are only reloaded when the board itself changes (item
///   placements / merges), not on hover or drag-position updates.
pub(crate) fn update_cell_visuals(
    board: Res<Board>,
    drag: Res<DragState>,
    db: Res<ItemDatabase>,
    asset_server: Res<AssetServer>,
    mut cell_query: Query<(
        &BoardCell,
        &Interaction,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut image_query: Query<(&CellImage, &mut Node, &mut ImageNode)>,
    // Lightweight query: detects whether any cell's hover/press state changed.
    changed_interactions: Query<(), (Changed<Interaction>, With<BoardCell>)>,
) {
    let board_changed = board.is_changed();
    let drag_changed = drag.is_changed();

    // Skip the entire system when nothing that affects visuals has changed.
    if !board_changed && !drag_changed && changed_interactions.is_empty() {
        return;
    }

    for (cell, interaction, mut bg, mut border) in &mut cell_query {
        let idx = cell.index;
        let item_id = board.cells[idx].item_id.as_deref();
        let selected = board.selected == Some(idx);
        let is_drag_source = drag.dragging && drag.source == Some(idx);

        let col = idx % BOARD_COLS;
        let row = idx / BOARD_COLS;
        let base_empty_bg = if (col + row) % 2 == 0 {
            CELL_EMPTY
        } else {
            CELL_EMPTY_ALT
        };

        let new_bg = if is_drag_source {
            // Dim the source cell while the piece is being dragged
            BackgroundColor(Color::srgba(0.30, 0.25, 0.15, 0.45))
        } else if selected {
            BackgroundColor(CELL_SELECTED)
        } else if *interaction == Interaction::Hovered {
            BackgroundColor(CELL_HOVERED)
        } else if let Some(id) = item_id {
            if let Some(def) = db.get(id) {
                let (r, g, b) = def.bg_color;
                BackgroundColor(Color::srgb(
                    r * 0.45 + 0.08,
                    g * 0.45 + 0.07,
                    b * 0.45 + 0.05,
                ))
            } else {
                BackgroundColor(base_empty_bg)
            }
        } else {
            BackgroundColor(base_empty_bg)
        };

        let new_border = if is_drag_source {
            BorderColor::all(Color::srgba(0.88, 0.72, 0.30, 0.50))
        } else if selected {
            BorderColor::all(ACCENT)
        } else {
            BorderColor::all(Color::srgb(0.25, 0.22, 0.17))
        };

        // Only mutate (and mark changed for the renderer) when the value differs.
        bg.set_if_neq(new_bg);
        border.set_if_neq(new_border);
    }

    // Icon images only change when items are placed, moved, or merged — not on hover.
    if board_changed {
        for (ci, mut node, mut img) in &mut image_query {
            let idx = ci.index;
            let icon_path = board.cells[idx]
                .item_id
                .as_deref()
                .and_then(|id| db.get(id))
                .and_then(|def| def.icon_path);

            match icon_path {
                Some(path) => {
                    let new_handle = asset_server.load(path);
                    // Only update (and mark ImageNode changed) when the handle differs.
                    if img.image != new_handle {
                        img.image = new_handle;
                    }
                    if node.display != Display::Flex {
                        node.display = Display::Flex;
                    }
                }
                None => {
                    if node.display != Display::None {
                        node.display = Display::None;
                    }
                }
            }
        }
    }
}

pub(crate) fn update_economy_ui(
    economy: Res<Economy>,
    mut stamina_q: Query<
        &mut Text,
        (
            With<StaminaLabel>,
            Without<CoinsLabel>,
            Without<GemsLabel>,
            Without<LevelLabel>,
        ),
    >,
    mut coins_q: Query<
        &mut Text,
        (
            With<CoinsLabel>,
            Without<StaminaLabel>,
            Without<GemsLabel>,
            Without<LevelLabel>,
        ),
    >,
    mut gems_q: Query<
        &mut Text,
        (
            With<GemsLabel>,
            Without<CoinsLabel>,
            Without<StaminaLabel>,
            Without<LevelLabel>,
        ),
    >,
    mut level_q: Query<
        &mut Text,
        (
            With<LevelLabel>,
            Without<CoinsLabel>,
            Without<GemsLabel>,
            Without<StaminaLabel>,
        ),
    >,
) {
    // Economy ticks every frame (stamina timer), so is_changed() is always true.
    // Use value comparison to avoid marking Text as changed when the displayed
    // value hasn't actually changed (stamina only changes every 2 minutes, etc.).
    if let Ok(mut t) = stamina_q.single_mut() {
        let new_val = format!("{}/{}", economy.stamina, economy.max_stamina);
        if **t != new_val {
            **t = new_val;
        }
    }
    if let Ok(mut t) = coins_q.single_mut() {
        let new_val = economy.coins.to_string();
        if **t != new_val {
            **t = new_val;
        }
    }
    if let Ok(mut t) = gems_q.single_mut() {
        let new_val = economy.gems.to_string();
        if **t != new_val {
            **t = new_val;
        }
    }
    if let Ok(mut t) = level_q.single_mut() {
        let new_val = economy.level.to_string();
        if **t != new_val {
            **t = new_val;
        }
    }
}

pub(crate) fn update_orders_ui(
    board: Res<Board>,
    orders: Res<Orders>,
    mut submit_q: Query<
        (&OrderSubmitButton, &mut Node, &mut BackgroundColor),
        (
            With<SubmitBtn>,
        ),
    >,
) {
    if !orders.is_changed() && !board.is_changed() {
        return;
    }

    // Complete overlay: shown when all required items are present on the board.
    for (submit, mut node, mut bg) in &mut submit_q {
        let slot = submit.order_id as usize;
        let can_complete = if let Some(order) = orders.orders.get(slot) {
            // Check that for each required item there is a matching board cell
            let mut remaining: Vec<&str> = order.items.iter().map(|s| s.as_str()).collect();
            for cell in &board.cells {
                if let Some(id) = &cell.item_id {
                    if let Some(pos) = remaining.iter().position(|&r| r == id.as_str()) {
                        remaining.remove(pos);
                    }
                }
            }
            remaining.is_empty()
        } else {
            false
        };

        let new_display = if can_complete { Display::Flex } else { Display::None };
        if node.display != new_display {
            node.display = new_display;
        }
        if can_complete {
            bg.set_if_neq(BackgroundColor(Color::srgba(0.12, 0.40, 0.12, 0.82)));
        }
    }
}

pub(crate) fn update_message_bar(
    time: Res<Time>,
    mut message: ResMut<MessageBar>,
    mut label_q: Query<&mut Text, With<MessageLabel>>,
) {
    if message.timer > 0.0 {
        message.timer -= time.delta_secs();
        if let Ok(mut t) = label_q.single_mut() {
            **t = message.text.clone();
        }
    } else if !message.text.is_empty() {
        message.text.clear();
        if let Ok(mut t) = label_q.single_mut() {
            **t = String::new();
        }
    }
}

/// Update the item-detail bar below the board whenever the board selection changes.
///
/// Skips entirely when neither the board nor the egg storage has changed this frame,
/// avoiding redundant text and image updates on every idle frame.
pub(crate) fn update_item_detail_bar(
    board: Res<Board>,
    db: Res<ItemDatabase>,
    asset_server: Res<AssetServer>,
    egg_storage: Res<EggStorage>,
    economy: Res<Economy>,
    mut name_q: Query<&mut Text, (With<DetailName>, Without<DetailHint>)>,
    mut hint_q: Query<&mut Text, (With<DetailHint>, Without<DetailName>)>,
    mut icon_q: Query<&mut ImageNode, With<DetailIcon>>,
) {
    // Board is only marked changed when items move, merge, or the selection changes.
    // Egg storage changes once per hour (or on click), so both triggers are cheap.
    if !board.is_changed() && !egg_storage.is_changed() {
        return;
    }

    if let Some(selected_idx) = board.selected {
        if let Some(item_id) = board.cells[selected_idx].item_id.as_deref() {
            if let Some(def) = db.get(item_id) {
                if let Ok(mut t) = name_q.single_mut() {
                    **t = format!("{} {} Lv{}", def.emoji, def.name, def.level);
                }
                if let Ok(mut t) = hint_q.single_mut() {
                    **t = if def.is_auto_generator {
                        let pending =
                            egg_storage.0.get(&selected_idx).copied().unwrap_or(0);
                        format!(
                            "自动产蛋（每 {:.0} 分钟 1 枚，最多存 6 枚）| 已存 {} 枚 | 再次点击放置到最近空位",
                            def.auto_gen_interval_secs / SECONDS_PER_MINUTE,
                            pending
                        )
                    } else if def.is_generator {
                        if economy.stamina >= 1 {
                            format!(
                                "再次点击消耗 1 体力生成子棋（剩余体力：{}/{}）",
                                economy.stamina, economy.max_stamina
                            )
                        } else {
                            "体力不足！无法生成子棋（等待体力恢复）".to_string()
                        }
                    } else if def.merge_result_id.is_some() {
                        "拖拽或点击同类同级棋子可合成升级".to_string()
                    } else {
                        "✨ 最高级！".to_string()
                    };
                }
                if let Ok(mut img) = icon_q.single_mut() {
                    if let Some(path) = def.icon_path {
                        img.image = asset_server.load(path);
                    } else {
                        img.image = Handle::default();
                    }
                }
                return;
            }
        }
    }

    // Nothing selected — show default hint
    if let Ok(mut t) = name_q.single_mut() {
        **t = "点击棋子查看详情".to_string();
    }
    if let Ok(mut t) = hint_q.single_mut() {
        **t = "拖拽同类同级棋子可合成".to_string();
    }
    if let Ok(mut img) = icon_q.single_mut() {
        img.image = Handle::default();
    }
}

/// Refresh the icon image shown in each item slot of each order card.
///
/// Uses a per-slot item-ID cache stored on the `OrderItemIcon` component to avoid
/// calling `asset_server.load` and marking `ImageNode` as changed on every
/// frame when the order contents haven't actually changed.
pub(crate) fn update_order_icons(
    orders: Res<Orders>,
    db: Res<ItemDatabase>,
    asset_server: Res<AssetServer>,
    mut icon_q: Query<(&mut OrderItemIcon, &mut ImageNode, &mut Node)>,
) {
    for (mut order_icon, mut img, mut node) in &mut icon_q {
        let slot = order_icon.order_id as usize;
        let item_idx = order_icon.item_index as usize;

        let current_item_id = orders
            .orders
            .get(slot)
            .and_then(|o| o.items.get(item_idx))
            .map(|s| s.as_str());

        // Skip if the item in this slot hasn't changed since the last update.
        if current_item_id == order_icon.cached_item_id.as_deref() {
            continue;
        }

        // Update the cache without triggering change detection on OrderItemIcon.
        order_icon.bypass_change_detection().cached_item_id =
            current_item_id.map(|s| s.to_string());

        if let Some(item_id) = current_item_id {
            let new_handle = db
                .get(item_id)
                .and_then(|def| def.icon_path)
                .map(|path| asset_server.load(path))
                .unwrap_or_default();
            img.image = new_handle;
            node.display = Display::Flex;
        } else {
            img.image = Handle::default();
            node.display = Display::None;
        }
    }
}

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
