//! Update systems for 合合游戏 (HeHe Game).
use bevy::prelude::*;

use crate::{
    AutoGenTimers, DetailHint, DetailIcon, DetailName, DragGhost, DragState, MessageBar,
    MessageLabel, OrderIcon, SubmitBtn, ACCENT, CELL_EMPTY, CELL_EMPTY_ALT, CELL_HOVERED,
    CELL_SELECTED, DRAG_THRESHOLD_PIXELS, ORDER_SUBMIT_BG, SECONDS_PER_MINUTE,
};
use crate::board::{Board, BoardCell, CellImage, ClickAction, BOARD_COLS};
use crate::economy::{CoinsLabel, Economy, GemsLabel, LevelLabel, StaminaLabel};
use crate::items::ItemDatabase;
use crate::orders::{format_time, OrderItemText, OrderSubmitButton, OrderTimeText, Orders};

pub(crate) fn tick_economy(time: Res<Time>, mut economy: ResMut<Economy>) {
    economy.tick(time.delta_secs());
}

pub(crate) fn tick_orders(
    time: Res<Time>,
    mut orders: ResMut<Orders>,
    db: Res<ItemDatabase>,
    mut message: ResMut<MessageBar>,
) {
    let expired = orders.tick(time.delta_secs());
    if !expired.is_empty() {
        message.set(format!("{} 个订单已超时！", expired.len()));
        orders.fill_orders(&db);
    }
}

pub(crate) fn tick_auto_generators(
    time: Res<Time>,
    mut board: ResMut<Board>,
    db: Res<ItemDatabase>,
    mut timers: ResMut<AutoGenTimers>,
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
                let gen_id = def
                    .pick_generated_item(&mut rand::thread_rng())?
                    .to_string();
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
            if !board.place_near(idx, &gen_id) {
                message.set("棋盘已满，自动生成失败！");
            }
        }
    }
}

pub(crate) fn handle_cell_interaction(
    mut board: ResMut<Board>,
    db: Res<ItemDatabase>,
    mut economy: ResMut<Economy>,
    mut message: ResMut<MessageBar>,
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
                        message.set(format!(
                            "{} {} 自动生成中，{:.0}分钟后产出",
                            item.emoji,
                            item.name,
                            item.auto_gen_interval_secs / SECONDS_PER_MINUTE,
                        ));
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
                            format!(
                                "— 自动生成，{:.0}分钟/次",
                                item.auto_gen_interval_secs / SECONDS_PER_MINUTE
                            )
                        } else if item.is_generator {
                            "— 再次点击生成（耗1体力）".to_string()
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
            let item_name = order.item_name.clone();
            let item_emoji = order.item_emoji.clone();
            let needed_qty = order.quantity;
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
            } else {
                message.set(format!(
                    "需要 {}×{} {}，请先合成！",
                    needed_qty, item_emoji, item_name,
                ));
            }
        } else {
            message.set("此处暂无订单");
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
pub(crate) fn update_drag_ghost(
    drag: Res<DragState>,
    asset_server: Res<AssetServer>,
    mut ghost_q: Query<(&mut Node, &mut ImageNode), With<DragGhost>>,
) {
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
) {
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

        *bg = if is_drag_source {
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

        *border = if is_drag_source {
            BorderColor::all(Color::srgba(0.88, 0.72, 0.30, 0.50))
        } else if selected {
            BorderColor::all(ACCENT)
        } else {
            BorderColor::all(Color::srgb(0.25, 0.22, 0.17))
        };
    }

    for (ci, mut node, mut img) in &mut image_query {
        let idx = ci.index;
        match board.cells[idx].item_id.as_deref() {
            Some(id) => {
                if let Some(def) = db.get(id) {
                    if let Some(icon_path) = def.icon_path {
                        img.image = asset_server.load(icon_path);
                        node.display = Display::Flex;
                    } else {
                        node.display = Display::None;
                    }
                } else {
                    node.display = Display::None;
                }
            }
            None => {
                node.display = Display::None;
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
    if let Ok(mut t) = stamina_q.single_mut() {
        **t = format!("{}/{}", economy.stamina, economy.max_stamina);
    }
    if let Ok(mut t) = coins_q.single_mut() {
        **t = economy.coins.to_string();
    }
    if let Ok(mut t) = gems_q.single_mut() {
        **t = economy.gems.to_string();
    }
    if let Ok(mut t) = level_q.single_mut() {
        **t = economy.level.to_string();
    }
}

pub(crate) fn update_orders_ui(
    orders: Res<Orders>,
    mut item_text_q: Query<
        (&OrderItemText, &mut Text),
        (Without<OrderTimeText>, Without<SubmitBtn>),
    >,
    mut time_text_q: Query<
        (&OrderTimeText, &mut Text),
        (Without<OrderItemText>, Without<SubmitBtn>),
    >,
    mut submit_q: Query<
        (&OrderSubmitButton, &mut BackgroundColor, &mut BorderColor),
        (
            With<SubmitBtn>,
            Without<OrderItemText>,
            Without<OrderTimeText>,
        ),
    >,
) {
    for (slot_cmp, mut text) in &mut item_text_q {
        let slot = slot_cmp.order_id as usize;
        **text = if let Some(order) = orders.orders.get(slot) {
            format!(
                "{} {} ×{}  奖励{}铜板",
                order.item_emoji, order.item_name, order.quantity, order.coin_reward
            )
        } else {
            "（空）".to_string()
        };
    }

    for (slot_cmp, mut text) in &mut time_text_q {
        let slot = slot_cmp.order_id as usize;
        **text = if let Some(order) = orders.orders.get(slot) {
            format!("剩余：{}", format_time(order.time_remaining_secs))
        } else {
            String::new()
        };
    }

    for (submit, mut bg, mut border) in &mut submit_q {
        let slot = submit.order_id as usize;
        if orders.orders.get(slot).is_some() {
            *bg = BackgroundColor(ORDER_SUBMIT_BG);
            *border = BorderColor::all(Color::srgb(0.40, 0.65, 0.30));
        } else {
            *bg = BackgroundColor(Color::srgb(0.18, 0.18, 0.16));
            *border = BorderColor::all(Color::srgb(0.28, 0.25, 0.20));
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
pub(crate) fn update_item_detail_bar(
    board: Res<Board>,
    db: Res<ItemDatabase>,
    asset_server: Res<AssetServer>,
    mut name_q: Query<&mut Text, (With<DetailName>, Without<DetailHint>)>,
    mut hint_q: Query<&mut Text, (With<DetailHint>, Without<DetailName>)>,
    mut icon_q: Query<&mut ImageNode, With<DetailIcon>>,
) {
    if let Some(selected_idx) = board.selected {
        if let Some(item_id) = board.cells[selected_idx].item_id.as_deref() {
            if let Some(def) = db.get(item_id) {
                if let Ok(mut t) = name_q.single_mut() {
                    **t = format!("{} {} Lv{}", def.emoji, def.name, def.level);
                }
                if let Ok(mut t) = hint_q.single_mut() {
                    **t = if def.is_auto_generator {
                        format!(
                            "自动生成，{:.0} 分钟 / 次",
                            def.auto_gen_interval_secs / SECONDS_PER_MINUTE
                        )
                    } else if def.is_generator {
                        "再次点击消耗 1 体力生成子棋".to_string()
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

/// Refresh the icon image shown in each order slot.
pub(crate) fn update_order_icons(
    orders: Res<Orders>,
    db: Res<ItemDatabase>,
    asset_server: Res<AssetServer>,
    mut icon_q: Query<(&OrderIcon, &mut ImageNode)>,
) {
    for (order_icon, mut img) in &mut icon_q {
        let slot = order_icon.order_id as usize;
        if let Some(order) = orders.orders.get(slot) {
            if let Some(def) = db.get(&order.item_id) {
                if let Some(path) = def.icon_path {
                    img.image = asset_server.load(path);
                    continue;
                }
            }
        }
        img.image = Handle::default();
    }
}
