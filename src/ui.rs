//! UI setup for 合合游戏 (HeHe Game).
//! Contains startup systems and UI spawn helpers.
use bevy::prelude::*;

use crate::board::{Board, BoardCell, BoardGrid, CellImage, BOARD_COLS, BOARD_ROWS};
use crate::economy::{CoinsLabel, GemsLabel, LevelLabel, StaminaLabel};
use crate::items::ItemDatabase;
use crate::orders::{OrderItemIcon, OrderPanel, OrderSubmitButton, Orders};
use crate::{
    ActivityButton, DetailHint, DetailIcon, DetailName, DragGhost, MessageLabel, SubmitBtn,
    WarehouseButton, ACCENT, ACCENT_GREEN, BOARD_BG, CELL_EMPTY, CELL_EMPTY_ALT, DETAIL_BAR_BG,
    DETAIL_BAR_H, ORDER_BG, ORDER_SLOT_BG, OVERLAY_ALPHA, TEXT_MAIN, TEXT_MUTED, TOP_BAR_BG,
    TOP_BAR_H,
};

/// Height of the horizontal order row at the top of the content area.
pub const ORDER_ROW_H: f32 = 105.0;

pub(crate) fn setup_initial_board(mut board: ResMut<Board>) {
    board.place(Board::idx(0, 0), "poultry_2");
    board.place(Board::idx(1, 0), "fabric_1");
    // board.place(Board::idx(2, 0), ""); // 小猫
    // board.place(Board::idx(3, 0), "");
    // board.place(Board::idx(4, 0), "");
    // board.place(Board::idx(5, 0), "");
    // board.place(Board::idx(6, 0), "");

    board.place(Board::idx(0, 1), "ring_1");
    board.place(Board::idx(1, 1), "dough_7");
    board.place(Board::idx(2, 1), "watermelon_4");
    board.place(Board::idx(3, 1), "coolTea_1");
    // board.place(Board::idx(4, 1), "");
    // board.place(Board::idx(5, 1), "");
    // board.place(Board::idx(6, 1), "");

    // board.place(Board::idx(0, 2), ""); // 金币lv1
    board.place(Board::idx(1, 2), "dough_7");
    board.place(Board::idx(2, 2), "dough_6");
    board.place(Board::idx(3, 2), "dough_6");
    // board.place(Board::idx(4, 2), "");
    // board.place(Board::idx(5, 2), "");
    // board.place(Board::idx(6, 2), "");

    board.place(Board::idx(0, 3), "dough_4");
    board.place(Board::idx(1, 3), "basket_1");
    board.place(Board::idx(2, 3), "dough_3");
    board.place(Board::idx(3, 3), "watermelon_2");
    board.place(Board::idx(4, 3), "dough_2");
    board.place(Board::idx(5, 3), "craftBox_3");
    // board.place(Board::idx(6, 3), "");

    board.place(Board::idx(0, 4), "dough_3");
    board.place(Board::idx(1, 4), "basket_1");
    board.place(Board::idx(2, 4), "basket_2");
    board.place(Board::idx(3, 4), "basket_3");
    board.place(Board::idx(4, 4), "dough_5");
    board.place(Board::idx(5, 4), "coolTea_3");
    // board.place(Board::idx(6, 4), "");

    board.place(Board::idx(0, 5), "teapot_1");
    board.place(Board::idx(1, 5), "watermelon_3");
    board.place(Board::idx(2, 5), "watermelon_1");
    board.place(Board::idx(3, 5), "dough_5");
    board.place(Board::idx(4, 5), "dough_3");
    board.place(Board::idx(5, 5), "watermelon_3");
    board.place(Board::idx(6, 5), "fabric_3");

    // board.place(Board::idx(0, 6), "");
    board.place(Board::idx(1, 6), "ring_1");
    board.place(Board::idx(2, 6), "teapot_2");
    // board.place(Board::idx(3, 6), ""); // 三条小鱼干
    board.place(Board::idx(4, 6), "poultry_1");
    board.place(Board::idx(5, 6), "dresser_2");
    // board.place(Board::idx(6, 6), "");

    // board.place(Board::idx(0, 7), "");
    // board.place(Board::idx(1, 7), "");
    // board.place(Board::idx(2, 7), "");
    // board.place(Board::idx(3, 7), "");
    // board.place(Board::idx(4, 7), "");
    // board.place(Board::idx(5, 7), "");
    board.place(Board::idx(6, 7), "greenBox_1");

    board.place(Board::idx(0, 8), "poultry_6");
    board.place(Board::idx(1, 8), "teapot_4");
    board.place(Board::idx(2, 8), "basket_5");
    board.place(Board::idx(3, 8), "craftBox_5");
    board.place(Board::idx(4, 8), "dresser_5");
    board.place(Board::idx(5, 8), "loom_5");
    board.place(Board::idx(6, 8), "redBox_1");
}

/// Build the full UI hierarchy.
pub(crate) fn setup_ui(
    mut commands: Commands,
    mut orders: ResMut<Orders>,
    db: Res<ItemDatabase>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);
    orders.fill_orders(&db);

    let font: Handle<Font> = asset_server.load("fonts/SourceHanSansSC-Medium.otf");

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
            spawn_board_grid(root);
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
            spawn_stat_card(stats, "⚡体力", StaminaLabel, "100/100", ACCENT_GREEN, font);
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

fn spawn_order_row(root: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    root.spawn((
        Node {
            width: percent(100.0),
            height: px(ORDER_ROW_H),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            padding: UiRect::axes(px(10.0), px(6.0)),
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
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(8.0)),
                row_gap: px(6.0),
                min_width: px(196.0),
                flex_shrink: 0.0,
                border_radius: BorderRadius::all(px(8.0)),
                border: UiRect::all(px(1.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(ORDER_SLOT_BG),
            BorderColor::all(Color::srgb(0.30, 0.25, 0.18)),
        ))
        .with_children(|s| {
            // Icons row: up to 3 large item icons
            s.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(6.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|row| {
                for item_pos in 0..3usize {
                    row.spawn((
                        Node {
                            width: px(54.0),
                            height: px(54.0),
                            flex_shrink: 0.0,
                            border_radius: BorderRadius::all(px(6.0)),
                            border: UiRect::all(px(1.0)),
                            display: Display::None, // hidden until an order occupies this slot
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, OVERLAY_ALPHA)),
                        BorderColor::all(Color::srgb(0.28, 0.22, 0.15)),
                        ImageNode::default(),
                        OrderItemIcon {
                            slot_index: slot,
                            item_pos,
                            cached_item_id: None,
                        },
                    ));
                }
            });

            // "完成" button — visible only when all required items are on the board
            s.spawn((
                Button,
                Node {
                    width: percent(90.0),
                    height: px(26.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(1.0)),
                    border_radius: BorderRadius::all(px(4.0)),
                    display: Display::None, // hidden until fulfillable
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.45, 0.20)),
                BorderColor::all(Color::srgb(0.40, 0.65, 0.30)),
                OrderSubmitButton {
                    order_id: slot as u32,
                },
                SubmitBtn,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("完成"),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(TEXT_MAIN),
                ));
            });
        });
}

// ── Board grid (full-width, fills remaining space) ────────────────────────────

fn spawn_board_grid(root: &mut ChildSpawnerCommands) {
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
            spawn_cell(grid, idx);
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

fn spawn_cell(grid: &mut ChildSpawnerCommands, idx: usize) {
    let col = idx % BOARD_COLS;
    let row = idx / BOARD_COLS;
    let cell_bg = if (col + row) % 2 == 0 {
        CELL_EMPTY
    } else {
        CELL_EMPTY_ALT
    };

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
        BackgroundColor(cell_bg),
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
    });
}
