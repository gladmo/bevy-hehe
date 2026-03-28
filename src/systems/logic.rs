//! Tick and input-handling systems that mutate game state.
use bevy::prelude::*;

use crate::{
    ActivityButton, AutoGenCooldowns, AutoGenCounts, AutoGenTimers, DoubleStaminaButton,
    DoubleStaminaMode, EggStorage, GameAudio, GeneratorUsesRemaining, MessageBar,
    SECONDS_PER_MINUTE, AUTO_GEN_BATCH_LIMIT, AUTO_GEN_COOLDOWN_SECS, WarehouseButton,
};
use crate::board::{Board, BoardCell, ClickAction};
use crate::economy::Economy;
use crate::items::types::ChainType;
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
    mut counts: ResMut<AutoGenCounts>,
    mut cooldowns: ResMut<AutoGenCooldowns>,
    mut message: ResMut<MessageBar>,
) {
    let delta = time.delta_secs();

    // Tick down any active cooldowns first.
    for secs in cooldowns.0.values_mut() {
        *secs = (*secs - delta).max(0.0);
    }

    // Collect auto-generator info (avoid borrow conflicts)
    let generators: Vec<(usize, f32, String)> = board
        .cells
        .iter()
        .enumerate()
        .filter_map(|(idx, cell)| {
            let id = cell.item_id.as_deref()?;
            let def = db.get(id)?;
            if def.is_auto_generator {
                let gen_id = def.generates_id.clone()?;
                Some((idx, def.auto_gen_interval_secs, gen_id))
            } else {
                None
            }
        })
        .collect();

    for (idx, interval, gen_id) in generators {
        // Skip generation while this cell is on cooldown.
        let cd = cooldowns.0.entry(idx).or_insert(0.0);
        if *cd > 0.0 {
            // Try to auto-place any eggs that were already stored before cooldown.
            let stored = *egg_storage.0.get(&idx).unwrap_or(&0);
            if stored > 0 {
                if let Some(near_idx) = board.nearest_empty(idx) {
                    board.place(near_idx, &gen_id);
                    let s = egg_storage.0.entry(idx).or_insert(0);
                    *s = s.saturating_sub(1);
                }
            }
            continue;
        }

        let acc = timers.0.entry(idx).or_insert(0.0);
        *acc += delta;
        if *acc >= interval {
            *acc -= interval;
            // Add one egg to storage (max 6 stored eggs)
            let stored = egg_storage.0.entry(idx).or_insert(0);
            if *stored < 6 {
                *stored += 1;

                // Track how many pieces this cell has produced in this batch.
                let count = counts.0.entry(idx).or_insert(0);
                *count += 1;
                if *count >= AUTO_GEN_BATCH_LIMIT {
                    // Enter cooldown and reset the batch counter.
                    *count = 0;
                    *cd = AUTO_GEN_COOLDOWN_SECS;
                    message.set(format!(
                        "母棋已生成 {} 个棋子，冷却 {:.0} 分钟后可继续生成！",
                        AUTO_GEN_BATCH_LIMIT,
                        AUTO_GEN_COOLDOWN_SECS / SECONDS_PER_MINUTE
                    ));
                }
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
    mut commands: Commands,
    mut board: ResMut<Board>,
    db: Res<ItemDatabase>,
    mut economy: ResMut<Economy>,
    mut message: ResMut<MessageBar>,
    mut egg_storage: ResMut<EggStorage>,
    mut gen_uses: ResMut<GeneratorUsesRemaining>,
    cooldowns: Res<AutoGenCooldowns>,
    counts: Res<AutoGenCounts>,
    double_stamina: Res<DoubleStaminaMode>,
    game_audio: Res<GameAudio>,
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
                        "合成成功！{} Lv{}{}",
                        item.name, item.level, hint
                    ));
                    economy.add_exp(10 * item.level as u64);
                    // Play merge SFX: merge_lv{result_level}, fallback to merge_lv9.
                    if let Some(sfx) = game_audio.merge_sfx(item.level) {
                        commands.spawn((AudioPlayer::new(sfx), PlaybackSettings::DESPAWN));
                    }
                }
            }
            ClickAction::GeneratorActivated(idx, item_id) => {
                if let Some(item) = db.get(&item_id) {
                    if item.is_auto_generator {
                        // 老母鸡: place an egg (no stamina cost).
                        // Consumes a stored egg from EggStorage when available;
                        // otherwise produces one on the spot so that double-clicking
                        // always works regardless of how long the hen has been running.
                        if let Some(ref gen_id) = item.generates_id {
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
                                    let gen_name = db.get(gen_id).map(|g| g.name.as_str()).unwrap_or("鸡蛋");
                                    message.set(format!("产出了{}！", gen_name));
                                }
                            } else {
                                message.set("棋盘已满，无法放置鸡蛋！");
                            }
                        }
                    } else if item.is_generator {
                        let count = item.generates_count.max(1);
                        let consumes = item.consumes_on_generate;
                        let max_uses = item.max_generate_count;
                        // Double-stamina mode: consume 2 stamina instead of 1,
                        // but produce child pieces that are 1 level higher.
                        // (老母鸡 auto-generators are exempt from this mode.)
                        let stamina_cost = if double_stamina.active { 2 } else { 1 };
                        if economy.spend_stamina(stamina_cost) {
                            let mut rng = rand::thread_rng();
                            let mut placed = 0u32;
                            let mut last_gen_id: Option<String> = None;
                            for _ in 0..count {
                                if let Some(gen_id) = item.pick_generated_item(&mut rng) {
                                    // In double-stamina mode, upgrade the generated piece
                                    // by 1 level (use its merge_result_id when available).
                                    let actual_gen_id = if double_stamina.active {
                                        db.get(&gen_id)
                                            .and_then(|def| def.merge_result_id.clone())
                                            .unwrap_or_else(|| gen_id.clone())
                                    } else {
                                        gen_id
                                    };
                                    if board.place_near(idx, &actual_gen_id) {
                                        placed += 1;
                                        last_gen_id = Some(actual_gen_id);
                                    } else {
                                        // Board full — stop early
                                        break;
                                    }
                                }
                            }
                            if placed == 0 {
                                // Nothing placed — refund stamina (no cap: stamina may legitimately
                                // exceed max_stamina via uncapped sources such as gourd tools).
                                economy.stamina += stamina_cost;
                                message.set("棋盘已满，无法生成！");
                            } else {
                                // Determine whether this generator should be consumed.
                                let should_consume = if max_uses > 0 {
                                    // Limited-use generator: decrement remaining uses.
                                    // Uses or_insert to initialise the counter on first activation.
                                    let entry = gen_uses.0.entry(idx).or_insert(max_uses);
                                    *entry = entry.saturating_sub(1);
                                    let left = *entry;
                                    if left == 0 {
                                        gen_uses.0.remove(&idx);
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    consumes
                                };
                                if should_consume {
                                    board.cells[idx].item_id = None;
                                    board.dirty = true;
                                }
                                // Build the message based on remaining uses.
                                let remaining_left = if max_uses > 0 {
                                    gen_uses.0.get(&idx).copied()
                                } else {
                                    None
                                };
                                if let Some(gen_item) = last_gen_id.as_deref().and_then(|id| db.get(id)) {
                                    if let Some(left) = remaining_left {
                                        message.set(format!(
                                            "生成了 {}！剩余 {} 次，体力 {}",
                                            gen_item.name, left, economy.stamina,
                                        ));
                                    } else {
                                        message.set(format!(
                                            "生成了 {}！剩余体力 {}",
                                            gen_item.name, economy.stamina,
                                        ));
                                    }
                                } else {
                                    // Fallback for generators without a named item (or batch).
                                    message.set(format!(
                                        "生成了 {} 个棋子！剩余体力 {}",
                                        placed, economy.stamina,
                                    ));
                                }
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
            ClickAction::Selected(idx) => {
                if let Some(id) = board.cells[idx].item_id.clone() {
                    if let Some(item) = db.get(&id) {
                        let hint = if item.is_auto_generator {
                            let pending = egg_storage.0.get(&idx).copied().unwrap_or(0);
                            let cd = cooldowns.0.get(&idx).copied().unwrap_or(0.0);
                            let batch_count = counts.0.get(&idx).copied().unwrap_or(0);
                            if cd > 0.0 {
                                format!(
                                    "— 冷却中（{:.0} 秒后恢复），存 {}/6 枚，再次点击放置",
                                    cd,
                                    pending
                                )
                            } else {
                                format!(
                                    "— 自动产蛋（每 {:.0} 分钟 1 枚，本批 {}/{} 枚，存 {}/6 枚），再次点击放置到最近空位",
                                    item.auto_gen_interval_secs / SECONDS_PER_MINUTE,
                                    batch_count,
                                    AUTO_GEN_BATCH_LIMIT,
                                    pending
                                )
                            }
                        } else if item.is_generator {
                            let stamina_cost = if double_stamina.active { 2 } else { 1 };
                            if economy.stamina >= stamina_cost {
                                if item.max_generate_count > 0 {
                                    let remaining = gen_uses
                                        .0
                                        .get(&idx)
                                        .copied()
                                        .unwrap_or(item.max_generate_count);
                                    format!(
                                        "— 再次点击生成（剩余 {} 次，耗{}体力，剩余体力：{}）",
                                        remaining, stamina_cost, economy.stamina
                                    )
                                } else {
                                    format!(
                                        "— 再次点击生成（耗{}体力，剩余体力：{}）",
                                        stamina_cost, economy.stamina
                                    )
                                }
                            } else {
                                "— 体力不足，无法生成子棋".to_string()
                            }
                        } else if item.merge_result_id.is_some() {
                            "— 点击同类同级棋子合成".to_string()
                        } else {
                            "— 最高级！".to_string()
                        };
                        message.set(format!(
                            "已选 {} Lv{} {}",
                            item.name, item.level, hint
                        ));
                    }
                }
            }
            ClickAction::Moved { item, .. } => {
                if let Some(def) = db.get(&item) {
                    message.set(format!("移动了 {}", def.name));
                }
            }
            ClickAction::ToolUsed(idx, item_id) => {
                if let Some(item) = db.get(&item_id) {
                    let level = item.level;
                    let multiplier = 2.5f32.powi((level as i32) - 1);
                    match item.chain {
                        ChainType::Gold => {
                            let reward = (2.0_f32 * multiplier).ceil() as u64;
                            economy.add_coins(reward);
                            message.set(format!("获得 {} 金币！", reward));
                        }
                        ChainType::Gourd => {
                            let reward = (2.55_f32 * multiplier).ceil() as i32;
                            economy.add_stamina_uncapped(reward);
                            message.set(format!("获得 {} 体力！", reward));
                        }
                        ChainType::Ruby => {
                            let reward = (1.7_f32 * multiplier).ceil() as u32;
                            economy.add_gems(reward);
                            message.set(format!("获得 {} 红宝石！", reward));
                        }
                        _ => {}
                    }
                    // Consume the tool item
                    board.cells[idx].item_id = None;
                    board.dirty = true;
                    board.selected = None;
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
    mut commands: Commands,
    mut board: ResMut<Board>,
    mut economy: ResMut<Economy>,
    mut orders: ResMut<Orders>,
    db: Res<ItemDatabase>,
    mut message: ResMut<MessageBar>,
    game_audio: Res<GameAudio>,
    interaction_query: Query<(&Interaction, &OrderSubmitButton), Changed<Interaction>>,
) {
    for (interaction, submit_btn) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // Play the order-submit (button press) sound every time the button is pressed.
        if let Some(sfx) = game_audio.get("order_serve") {
            commands.spawn((AudioPlayer::new(sfx), PlaybackSettings::DESPAWN));
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
                message.set(format!("订单完成！获得 {} 金币", reward));
                // Play the order-complete sound on successful fulfillment.
                if let Some(sfx) = game_audio.get("order_complete") {
                    commands.spawn((AudioPlayer::new(sfx), PlaybackSettings::DESPAWN));
                }
            }
        }
    }
}

/// Toggle the double-stamina mode when the button in the top-right corner is pressed.
pub(crate) fn handle_double_stamina_toggle(
    mut mode: ResMut<DoubleStaminaMode>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<DoubleStaminaButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            mode.active = !mode.active;
        }
    }
}

/// Play the generic button-click sound for UI buttons that don't have their own SFX.
///
/// Covers [`DoubleStaminaButton`], [`WarehouseButton`], and [`ActivityButton`].
pub(crate) fn handle_button_click(
    mut commands: Commands,
    game_audio: Res<GameAudio>,
    double_stamina_query: Query<&Interaction, (Changed<Interaction>, With<DoubleStaminaButton>)>,
    warehouse_query: Query<&Interaction, (Changed<Interaction>, With<WarehouseButton>)>,
    activity_query: Query<&Interaction, (Changed<Interaction>, With<ActivityButton>)>,
) {
    let any_pressed = double_stamina_query
        .iter()
        .chain(warehouse_query.iter())
        .chain(activity_query.iter())
        .any(|i| *i == Interaction::Pressed);

    if any_pressed {
        if let Some(sfx) = game_audio.get("button_click") {
            commands.spawn((AudioPlayer::new(sfx), PlaybackSettings::DESPAWN));
        }
    }
}
