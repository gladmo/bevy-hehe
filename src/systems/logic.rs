//! Tick and input-handling systems that mutate game state.
use bevy::prelude::*;

use crate::{
    AutoGenTimers, EggStorage, MessageBar, SECONDS_PER_MINUTE,
};
use crate::board::{Board, BoardCell, ClickAction};
use crate::economy::Economy;
use crate::items::ItemDatabase;
use crate::orders::{OrderSubmitButton, Orders};

pub(crate) fn tick_economy(time: Res<Time>, mut economy: ResMut<Economy>) {
    // Update the internal stamina timer without triggering Bevy's change
    // detection on every frame.  Change detection is only activated when
    // `stamina` actually increments (once every ~2 minutes), preventing
    // `update_economy_ui` from running its string-format comparisons 60×/second
    // on frames where no observable value has changed.
    let changed = {
        let inner = economy.bypass_change_detection();
        let old_stamina = inner.stamina;
        inner.tick(time.delta_secs());
        inner.stamina != old_stamina
    };
    if changed {
        economy.set_changed();
    }
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
