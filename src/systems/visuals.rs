//! Read-only (or nearly so) systems that refresh the UI.
use bevy::prelude::*;

use crate::{
    DetailHint, DetailIcon, DetailName, DoubleStaminaButton, DoubleStaminaLabel, DoubleStaminaMode,
    MessageBar, MessageLabel, SubmitBtn, ACCENT, CELL_EMPTY, CELL_EMPTY_ALT, CELL_HOVERED,
    CELL_SELECTED, DragState, EggStorage, SECONDS_PER_MINUTE,
};
use crate::board::{Board, BoardCell, CellImage, BOARD_COLS};
use crate::economy::{CoinsLabel, Economy, GemsLabel, LevelLabel, StaminaLabel};
use crate::items::ItemDatabase;
use crate::orders::{OrderItemIcon, OrderSubmitButton, Orders};

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
                .and_then(|def| def.icon_path.clone());

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
    // Economy is only marked changed when stamina/coins/gems/level actually
    // change — `tick_economy` uses `bypass_change_detection` for the timer
    // accumulation and only calls `set_changed()` when `stamina` increments.
    // This guard lets the system return immediately on idle frames, eliminating
    // 4 string-format allocations and comparisons per frame.
    if !economy.is_changed() {
        return;
    }
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
        With<SubmitBtn>,
    >,
) {
    if !orders.is_changed() && !board.is_changed() {
        return;
    }

    // Build a frequency map of item IDs present on the board (single pass).
    let mut board_counts: std::collections::HashMap<&str, u32> =
        std::collections::HashMap::new();
    for cell in &board.cells {
        if let Some(id) = &cell.item_id {
            *board_counts.entry(id.as_str()).or_insert(0) += 1;
        }
    }

    // Complete overlay: shown when all required items are present on the board.
    for (submit, mut node, mut bg) in &mut submit_q {
        let slot = submit.order_id as usize;
        let can_complete = if let Some(order) = orders.orders.get(slot) {
            // Count how many of each item the order needs, then verify the board has enough.
            let mut needed: std::collections::HashMap<&str, u32> =
                std::collections::HashMap::new();
            for item in &order.items {
                *needed.entry(item.as_str()).or_insert(0) += 1;
            }
            needed
                .iter()
                .all(|(&id, &qty)| board_counts.get(id).copied().unwrap_or(0) >= qty)
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
        // Only write to Text when the displayed string hasn't changed — avoids
        // marking the Text component as changed (and triggering a re-render)
        // on every frame during the countdown.
        if let Ok(mut t) = label_q.single_mut() {
            if **t != message.text {
                **t = message.text.clone();
            }
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
    double_stamina: Res<DoubleStaminaMode>,
    mut name_q: Query<&mut Text, (With<DetailName>, Without<DetailHint>)>,
    mut hint_q: Query<&mut Text, (With<DetailHint>, Without<DetailName>)>,
    mut icon_q: Query<&mut ImageNode, With<DetailIcon>>,
) {
    // Board is only marked changed when items move, merge, or the selection changes.
    // Egg storage changes once per hour (or on click), so both triggers are cheap.
    if !board.is_changed() && !egg_storage.is_changed() && !double_stamina.is_changed() {
        return;
    }

    if let Some(selected_idx) = board.selected {
        if let Some(item_id) = board.cells[selected_idx].item_id.as_deref() {
            if let Some(def) = db.get(item_id) {
                if let Ok(mut t) = name_q.single_mut() {
                    **t = format!("{} Lv{}", def.name, def.level);
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
                        let stamina_cost = if double_stamina.active { 2 } else { 1 };
                        if economy.stamina >= stamina_cost {
                            format!(
                                "再次点击消耗 {} 体力生成子棋（剩余体力：{}/{}）",
                                stamina_cost, economy.stamina, economy.max_stamina
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
                    let new_handle = if let Some(ref path) = def.icon_path {
                        asset_server.load(path.clone())
                    } else {
                        Handle::default()
                    };
                    // Only update (and mark ImageNode changed) when the handle differs.
                    if img.image != new_handle {
                        img.image = new_handle;
                    }
                }
                return;
            }
        }
    }

    // Nothing selected — show default hint
    if let Ok(mut t) = name_q.single_mut() {
        let default_name = "点击棋子查看详情";
        if **t != default_name {
            **t = default_name.to_string();
        }
    }
    if let Ok(mut t) = hint_q.single_mut() {
        let default_hint = "拖拽同类同级棋子可合成";
        if **t != default_hint {
            **t = default_hint.to_string();
        }
    }
    if let Ok(mut img) = icon_q.single_mut() {
        if img.image != Handle::default() {
            img.image = Handle::default();
        }
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
                .and_then(|def| def.icon_path.clone())
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

/// Update the double-stamina toggle button's appearance and label text.
///
/// Runs every frame only when `DoubleStaminaMode` has changed, keeping
/// the button label and colors in sync with the current mode state.
pub(crate) fn update_double_stamina_button(
    mode: Res<DoubleStaminaMode>,
    mut btn_q: Query<(&mut BackgroundColor, &mut BorderColor), With<DoubleStaminaButton>>,
    mut label_q: Query<(&mut Text, &mut TextColor), With<DoubleStaminaLabel>>,
) {
    if !mode.is_changed() {
        return;
    }
    if let Ok((mut bg, mut border)) = btn_q.single_mut() {
        if mode.active {
            bg.set_if_neq(BackgroundColor(Color::srgb(0.50, 0.20, 0.08)));
            border.set_if_neq(BorderColor::all(Color::srgb(0.88, 0.50, 0.20)));
        } else {
            bg.set_if_neq(BackgroundColor(Color::srgb(0.20, 0.16, 0.10)));
            border.set_if_neq(BorderColor::all(Color::srgb(0.40, 0.32, 0.20)));
        }
    }
    if let Ok((mut text, mut color)) = label_q.single_mut() {
        let new_label = if mode.active { "×2 体力 🔥" } else { "×1 体力" };
        if **text != new_label {
            **text = new_label.to_string();
        }
        let new_color = if mode.active {
            TextColor(Color::srgb(1.0, 0.75, 0.35))
        } else {
            TextColor(Color::srgb(0.65, 0.60, 0.48))
        };
        color.set_if_neq(new_color);
    }
}
