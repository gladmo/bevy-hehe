//! 合合游戏 (HeHe Game) — Bevy 0.18.1 implementation
//!
//! An ancient Chinese tea house themed merge puzzle game.
//! Two same-level, same-chain pieces merge into the next level.
//! Generator pieces produce child pieces (costs 1 stamina).
//! Auto-generator pieces (老母鸡) produce child pieces automatically over time.
//! Fulfill orders by submitting the required items to earn coins.

mod board;
mod economy;
mod items;
mod orders;

use bevy::prelude::*;

use board::{Board, BoardCell, BoardGrid, CellImage, CellText, ClickAction, BOARD_COLS, BOARD_ROWS};
use economy::{CoinsLabel, Economy, GemsLabel, LevelLabel, StaminaLabel};
use items::ItemDatabase;
use orders::{format_time, OrderItemText, OrderPanel, OrderSubmitButton, OrderTimeText, Orders};

// ── Window ────────────────────────────────────────────────────────────────────

const WINDOW_W: u32 = 1280;
const WINDOW_H: u32 = 820;

// ── Layout ────────────────────────────────────────────────────────────────────

const TOP_BAR_H: f32 = 65.0;
/// Height of the item-detail bar shown at the bottom of the screen.
const DETAIL_BAR_H: f32 = 85.0;
const SECONDS_PER_MINUTE: f32 = 60.0;

// ── Palette ───────────────────────────────────────────────────────────────────

const BG: Color = Color::srgb(0.13, 0.10, 0.07);
const TOP_BAR_BG: Color = Color::srgb(0.09, 0.07, 0.04);
const BOARD_BG: Color = Color::srgb(0.16, 0.13, 0.10);
const DETAIL_BAR_BG: Color = Color::srgb(0.11, 0.09, 0.06);
const ORDER_BG: Color = Color::srgb(0.12, 0.09, 0.07);
const CELL_EMPTY: Color = Color::srgb(0.20, 0.17, 0.14);
const CELL_HOVERED: Color = Color::srgb(0.32, 0.26, 0.18);
const CELL_SELECTED: Color = Color::srgb(0.55, 0.45, 0.20);
const TEXT_MAIN: Color = Color::srgb(0.96, 0.91, 0.78);
const TEXT_MUTED: Color = Color::srgb(0.65, 0.60, 0.48);
const ACCENT: Color = Color::srgb(0.88, 0.72, 0.30);
const ACCENT_GREEN: Color = Color::srgb(0.40, 0.80, 0.45);
const ORDER_SLOT_BG: Color = Color::srgb(0.18, 0.14, 0.10);
const ORDER_SUBMIT_BG: Color = Color::srgb(0.25, 0.45, 0.20);
/// Semi-transparent dark overlay used on icon/card backgrounds.
const OVERLAY_ALPHA: f32 = 0.25;

// ── Resources ─────────────────────────────────────────────────────────────────

/// Accumulated time per board cell for auto-generator ticking (cell index → secs).
#[derive(Resource, Default, Debug)]
struct AutoGenTimers(std::collections::HashMap<usize, f32>);

/// Temporary message shown in the order panel.
#[derive(Resource, Default, Debug)]
struct MessageBar {
    text: String,
    timer: f32,
}

impl MessageBar {
    fn set(&mut self, msg: impl Into<String>) {
        self.text = msg.into();
        self.timer = 3.5;
    }
}

/// Tag for the message bar text label entity.
#[derive(Component)]
struct MessageLabel;

/// Tag for order submit buttons (so we can style them separately from board cells).
#[derive(Component)]
struct SubmitBtn;

/// Tracks the state of an in-progress piece drag.
#[derive(Resource, Default, Debug)]
struct DragState {
    /// Board cell index the drag originated from.
    source: Option<usize>,
    /// Physical-pixel cursor position when the mouse button was pressed.
    press_pos: Vec2,
    /// Latest physical-pixel cursor position (updated every frame while dragging).
    cursor_phys: Vec2,
    /// Latest logical-pixel cursor position (used to position the ghost node).
    cursor_logical: Vec2,
    /// Whether the cursor has moved far enough to be considered a real drag.
    dragging: bool,
    /// Asset path of the icon shown in the drag ghost.
    icon_path: Option<&'static str>,
}

/// Tag component for the floating drag-ghost UI entity.
#[derive(Component)]
struct DragGhost;

/// Tag for the item-detail bar item icon.
#[derive(Component)]
struct DetailIcon;

/// Tag for the item-detail bar primary text (name + level).
#[derive(Component)]
struct DetailName;

/// Tag for the item-detail bar secondary text (hint / action description).
#[derive(Component)]
struct DetailHint;

/// Tag for the order icon image in each order slot.
#[derive(Component)]
struct OrderIcon {
    order_id: u32,
}

/// Tag for the 仓库 (warehouse) button in the bottom bar.
#[derive(Component)]
struct WarehouseButton;

/// Tag for the 活动 (activity) button in the bottom bar.
#[derive(Component)]
struct ActivityButton;

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
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
        }))
        .insert_resource(ClearColor(BG))
        .init_resource::<Board>()
        .init_resource::<Economy>()
        .init_resource::<Orders>()
        .insert_resource(ItemDatabase::new())
        .insert_resource(AutoGenTimers::default())
        .insert_resource(MessageBar::default())
        .insert_resource(DragState::default())
        .add_systems(Startup, setup_initial_board)
        .add_systems(Startup, setup_ui.after(setup_initial_board))
        .add_systems(
            Update,
            (
                tick_economy,
                tick_orders,
                tick_auto_generators,
                handle_drag_input,
                handle_cell_interaction,
                handle_order_submit,
                update_drag_ghost.after(handle_drag_input),
                update_cell_visuals.after(handle_drag_input),
                update_economy_ui,
                update_orders_ui,
                update_order_icons,
                update_item_detail_bar,
                update_message_bar,
            ),
        )
        .run();
}

// ── Startup systems ───────────────────────────────────────────────────────────

/// Place the initial items on the board.
fn setup_initial_board(mut board: ResMut<Board>) {
    // One of each generator in the top row
    board.place(Board::idx(0, 0), "poultry_1"); // 老母鸡 (auto)
    board.place(Board::idx(1, 0), "teapot_1"); // 茶壶
    board.place(Board::idx(2, 0), "basket_1"); // 食篓
    board.place(Board::idx(3, 0), "craftBox_1"); // 手作盒
    board.place(Board::idx(4, 0), "dresser_1"); // 妆奁
    board.place(Board::idx(5, 0), "loom_1"); // 织布机

    // Starter child items for immediate play
    board.place(Board::idx(0, 1), "egg_1");
    board.place(Board::idx(1, 1), "egg_1");
    board.place(Board::idx(2, 1), "coolTea_1");
    board.place(Board::idx(3, 1), "coolTea_1");
    board.place(Board::idx(4, 1), "dough_1");
    board.place(Board::idx(5, 1), "dough_1");
    board.place(Board::idx(6, 1), "lantern_1");
    board.place(Board::idx(0, 2), "lantern_1");
    board.place(Board::idx(1, 2), "ring_1");
    board.place(Board::idx(2, 2), "ring_1");
    board.place(Board::idx(3, 2), "fabric_1");
    board.place(Board::idx(4, 2), "fabric_1");
}

/// Build the full UI hierarchy.
fn setup_ui(
    mut commands: Commands,
    mut orders: ResMut<Orders>,
    db: Res<ItemDatabase>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);
    orders.fill_orders(&db);

    let font: Handle<Font> = asset_server.load("NotoSansCJK-Regular.ttf");

    // Root — full viewport, column layout
    commands
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|root| {
            spawn_top_bar(root, &font);
            spawn_order_row(root, &font);
            spawn_board_grid(root, &font, &db);
            spawn_bottom_bar(root, &font);
        });

    // Drag ghost — root-level absolute node that floats above all other UI.
    // `Pickable::IGNORE` ensures it never blocks pointer events on cells below it.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(56.0),
            height: Val::Px(56.0),
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            display: Display::None,
            ..default()
        },
        ImageNode::default(),
        ZIndex(1000),
        Pickable::IGNORE,
        DragGhost,
    ));
}

// ── Top bar ───────────────────────────────────────────────────────────────────

fn spawn_top_bar(root: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    root.spawn((
        Node {
            width: percent(100.0),
            height: px(TOP_BAR_H),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::axes(px(20.0), px(8.0)),
            border: UiRect::bottom(px(2.0)),
            ..default()
        },
        BackgroundColor(TOP_BAR_BG),
        BorderColor::all(Color::srgb(0.35, 0.28, 0.18)),
    ))
    .with_children(|bar| {
        // Left: Game title
        bar.spawn((
            Text::new("🏮 合合游戏"),
            TextFont {
                font: font.clone(),
                font_size: 24.0,
                ..default()
            },
            TextColor(ACCENT),
        ));

        // Center: Stats row
        bar.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(24.0),
            ..default()
        })
        .with_children(|stats| {
            spawn_stat_card(stats, "等级", LevelLabel, "1", TEXT_MAIN, font);
            spawn_stat_card(
                stats,
                "⚡体力",
                StaminaLabel,
                "100/100",
                ACCENT_GREEN,
                font,
            );
            spawn_stat_card(
                stats,
                "💰铜板",
                CoinsLabel,
                "0",
                Color::srgb(0.95, 0.80, 0.25),
                font,
            );
            spawn_stat_card(
                stats,
                "💎宝石",
                GemsLabel,
                "0",
                Color::srgb(0.55, 0.75, 0.95),
                font,
            );
        });

        // Right: placeholder to balance the flex layout
        bar.spawn(Node {
            width: px(100.0),
            ..default()
        });
    });
}

fn spawn_stat_card<M: Component>(
    bar: &mut ChildSpawnerCommands,
    label: &str,
    marker: M,
    initial: &str,
    value_color: Color,
    font: &Handle<Font>,
) {
    bar.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(10.0), px(4.0)),
            border: UiRect::all(px(1.0)),
            border_radius: BorderRadius::all(px(6.0)),
            row_gap: px(2.0),
            min_width: px(70.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, OVERLAY_ALPHA)),
        BorderColor::all(Color::srgb(0.30, 0.24, 0.16)),
    ))
    .with_children(|card| {
        card.spawn((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: 11.0,
                ..default()
            },
            TextColor(TEXT_MUTED),
        ));
        card.spawn((
            Text::new(initial),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(value_color),
            marker,
        ));
    });
}

// ── Order row (top, horizontal scroll) ───────────────────────────────────────

/// Height of the horizontal order row at the top of the content area.
const ORDER_ROW_H: f32 = 145.0;

fn spawn_order_row(root: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    root.spawn((
        Node {
            width: percent(100.0),
            height: px(ORDER_ROW_H),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            padding: UiRect::axes(px(10.0), px(8.0)),
            column_gap: px(10.0),
            overflow: Overflow::scroll_x(),
            border: UiRect::bottom(px(2.0)),
            ..default()
        },
        BackgroundColor(ORDER_BG),
        BorderColor::all(Color::srgb(0.28, 0.22, 0.15)),
        OrderPanel,
    ))
    .with_children(|panel| {
        for slot in 0..3usize {
            spawn_order_card(panel, slot, font);
        }
    });
}

fn spawn_order_card(panel: &mut ChildSpawnerCommands, slot: usize, font: &Handle<Font>) {
    panel
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(px(10.0)),
                column_gap: px(10.0),
                min_width: px(240.0),
                flex_shrink: 0.0,
                border_radius: BorderRadius::all(px(8.0)),
                border: UiRect::all(px(1.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(ORDER_SLOT_BG),
            BorderColor::all(Color::srgb(0.30, 0.25, 0.18)),
        ))
        .with_children(|s| {
            // Item icon (left)
            s.spawn((
                Node {
                    width: px(52.0),
                    height: px(52.0),
                    flex_shrink: 0.0,
                    border_radius: BorderRadius::all(px(6.0)),
                    border: UiRect::all(px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, OVERLAY_ALPHA)),
                BorderColor::all(Color::srgb(0.28, 0.22, 0.15)),
                ImageNode::default(),
                OrderIcon {
                    order_id: slot as u32,
                },
            ));

            // Text + button column (right)
            s.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(4.0),
                flex_grow: 1.0,
                ..default()
            })
            .with_children(|col| {
                // Item description
                col.spawn((
                    Text::new("（空）"),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(TEXT_MUTED),
                    OrderItemText {
                        order_id: slot as u32,
                    },
                ));

                // Time remaining
                col.spawn((
                    Text::new(""),
                    TextFont {
                        font: font.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(TEXT_MUTED),
                    OrderTimeText {
                        order_id: slot as u32,
                    },
                ));

                // Submit button
                col.spawn((
                    Button,
                    Node {
                        width: percent(100.0),
                        height: px(26.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.20, 0.20, 0.18)),
                    BorderColor::all(Color::srgb(0.35, 0.30, 0.20)),
                    OrderSubmitButton {
                        order_id: slot as u32,
                    },
                    SubmitBtn,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("提交"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(TEXT_MUTED),
                    ));
                });
            });
        });
}

// ── Board grid (full-width, fills remaining space) ────────────────────────────

fn spawn_board_grid(
    root: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    db: &ItemDatabase,
) {
    root.spawn((
        Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::flex(BOARD_COLS as u16, 1.0),
            grid_template_rows: RepeatedGridTrack::flex(BOARD_ROWS as u16, 1.0),
            width: percent(100.0),
            flex_grow: 1.0,
            row_gap: px(3.0),
            column_gap: px(3.0),
            padding: UiRect::all(px(6.0)),
            ..default()
        },
        BackgroundColor(BOARD_BG),
        BoardGrid,
    ))
    .with_children(|grid| {
        for idx in 0..(BOARD_COLS * BOARD_ROWS) {
            spawn_cell(grid, idx, font, db);
        }
    });
}

// ── Bottom bar (item detail + warehouse / activity buttons) ───────────────────

fn spawn_bottom_bar(root: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    root.spawn((
        Node {
            width: percent(100.0),
            height: px(DETAIL_BAR_H),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(10.0), px(8.0)),
            column_gap: px(10.0),
            border: UiRect::top(px(2.0)),
            ..default()
        },
        BackgroundColor(DETAIL_BAR_BG),
        BorderColor::all(Color::srgb(0.30, 0.24, 0.16)),
    ))
    .with_children(|bar| {
        // Left: 仓库 button
        spawn_bottom_action_btn(bar, "🏪 仓库", font, true);

        // Center: item detail (icon + name/hint text)
        bar.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(12.0),
            flex_grow: 1.0,
            ..default()
        })
        .with_children(|center| {
            // Item icon
            center.spawn((
                Node {
                    width: px(56.0),
                    height: px(56.0),
                    border_radius: BorderRadius::all(px(8.0)),
                    flex_shrink: 0.0,
                    border: UiRect::all(px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.20)),
                BorderColor::all(Color::srgb(0.30, 0.24, 0.16)),
                ImageNode::default(),
                DetailIcon,
            ));

            // Text column
            center
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(5.0),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new("点击棋子查看详情"),
                        TextFont {
                            font: font.clone(),
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(TEXT_MAIN),
                        DetailName,
                    ));
                    col.spawn((
                        Text::new("拖拽同类同级棋子可合成"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(TEXT_MUTED),
                        DetailHint,
                    ));
                    col.spawn((
                        Text::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.5,
                            ..default()
                        },
                        TextColor(TEXT_MAIN),
                        MessageLabel,
                    ));
                });
        });

        // Right: 活动 button
        spawn_bottom_action_btn(bar, "🎉 活动", font, false);
    });
}

/// Spawn a styled action button for the bottom bar.
fn spawn_bottom_action_btn(
    bar: &mut ChildSpawnerCommands,
    label: &str,
    font: &Handle<Font>,
    is_warehouse: bool,
) {
    let mut entity = bar.spawn((
        Button,
        Node {
            width: px(80.0),
            height: px(60.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(1.0)),
            border_radius: BorderRadius::all(px(8.0)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(Color::srgb(0.20, 0.16, 0.10)),
        BorderColor::all(Color::srgb(0.40, 0.32, 0.20)),
    ));
    if is_warehouse {
        entity.insert(WarehouseButton);
    } else {
        entity.insert(ActivityButton);
    }
    entity.with_children(|btn| {
        btn.spawn((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: 13.0,
                ..default()
            },
            TextColor(ACCENT),
        ));
    });
}

fn spawn_cell(
    grid: &mut ChildSpawnerCommands,
    idx: usize,
    font: &Handle<Font>,
    _db: &ItemDatabase,
) {
    grid.spawn((
        Button,
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(1.0)),
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        BackgroundColor(CELL_EMPTY),
        BorderColor::all(Color::srgb(0.25, 0.22, 0.17)),
        BoardCell { index: idx },
    ))
    .with_children(|cell| {
        // Item icon image (hidden by default)
        cell.spawn((
            Node {
                width: px(48.0),
                height: px(48.0),
                display: Display::None,
                ..default()
            },
            ImageNode::default(),
            CellImage { index: idx },
        ));

        // Item name / level label
        cell.spawn((
            Text::new(""),
            TextFont {
                font: font.clone(),
                font_size: 9.0,
                ..default()
            },
            TextColor(TEXT_MAIN),
            TextLayout::new_with_justify(Justify::Center),
            CellText { index: idx },
        ));
    });
}

// ── Update systems ────────────────────────────────────────────────────────────

fn tick_economy(time: Res<Time>, mut economy: ResMut<Economy>) {
    economy.tick(time.delta_secs());
}

fn tick_orders(
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

fn tick_auto_generators(
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
            if !board.place_near(idx, &gen_id) {
                message.set("棋盘已满，自动生成失败！");
            }
        }
    }
}

fn handle_cell_interaction(
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
                    } else if let Some(gen_id) = item.generates_id {
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
                                economy.stamina = (economy.stamina + 1).min(economy.max_stamina);
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
            ClickAction::None => {}
        }
    }
}

fn handle_order_submit(
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

/// Handles the full lifecycle of a drag gesture:
/// press → movement threshold → ghost appears → release → move or merge.
fn handle_drag_input(
    mut drag: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
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
        if !drag.dragging && cursor_phys.distance(drag.press_pos) > 8.0 {
            drag.dragging = true;
        }
    }

    // ── Mouse released ────────────────────────────────────────────────────────
    if mouse.just_released(MouseButton::Left) {
        if drag.dragging {
            if let Some(src) = drag.source {
                // Find the cell under the release position
                let mut target_idx: Option<usize> = None;
                for (cell, transform, computed) in &cell_query {
                    if ui_hit_test(cursor_phys, transform, computed) {
                        target_idx = Some(cell.index);
                        break;
                    }
                }

                if let Some(tgt) = target_idx {
                    if tgt != src {
                        let action = board.handle_drag(src, tgt, &db);
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
                            _ => {}
                        }
                    }
                }
            }
        }
        // Always reset drag state on release
        drag.source = None;
        drag.dragging = false;
    }
}

/// Moves the drag-ghost node to the cursor and loads the correct icon image.
fn update_drag_ghost(
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
fn update_cell_visuals(
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
    mut text_query: Query<(&CellText, &mut Text)>,
    mut image_query: Query<(&CellImage, &mut Node, &mut ImageNode)>,
) {
    for (cell, interaction, mut bg, mut border) in &mut cell_query {
        let idx = cell.index;
        let item_id = board.cells[idx].item_id.as_deref();
        let selected = board.selected == Some(idx);
        let is_drag_source = drag.dragging && drag.source == Some(idx);

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
                BackgroundColor(CELL_EMPTY)
            }
        } else {
            BackgroundColor(CELL_EMPTY)
        };

        *border = if is_drag_source {
            BorderColor::all(Color::srgba(0.88, 0.72, 0.30, 0.50))
        } else if selected {
            BorderColor::all(ACCENT)
        } else {
            BorderColor::all(Color::srgb(0.25, 0.22, 0.17))
        };
    }

    for (ct, mut text) in &mut text_query {
        let idx = ct.index;
        **text = match board.cells[idx].item_id.as_deref() {
            Some(id) => match db.get(id) {
                Some(def) => {
                    let tag = if def.is_auto_generator {
                        "[自动]"
                    } else if def.is_generator {
                        "[生成]"
                    } else {
                        ""
                    };
                    format!("{}\nLv{}{}", def.name, def.level, tag)
                }
                None => "?".to_string(),
            },
            None => String::new(),
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

fn update_economy_ui(
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

fn update_orders_ui(
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

fn update_message_bar(
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
fn update_item_detail_bar(
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
                        format!("自动生成，{:.0} 分钟 / 次", def.auto_gen_interval_secs / SECONDS_PER_MINUTE)
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
fn update_order_icons(
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
