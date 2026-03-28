//! UI setup for 合合游戏 (HeHe Game).
//! Contains startup systems and UI spawn helpers.
use bevy::prelude::*;

use crate::board::{Board, BoardCell, BoardGrid, CellCrownIcon, CellEnergyIcon, CellImage, CellSelectedOverlay, BOARD_COLS, BOARD_ROWS};
use crate::config::load_board_init;
use crate::economy::{CoinsLabel, GemsLabel, LevelLabel, StaminaLabel};
use crate::items::ItemDatabase;
use crate::orders::{OrderItemIcon, OrderPanel, OrderSubmitButton, Orders};
use crate::{
    ActivityButton, ActivityScreenRoot, BoardScreenRoot, DetailHint, DetailIcon, DetailName,
    DoubleStaminaButton, DoubleStaminaLabel, DragGhost, EnterBoardButton, MessageLabel,
    PreloadedImages, SubmitBtn, WarehouseButton, ACCENT, ACCENT_GREEN, BOARD_BG, CELL_EMPTY,
    CELL_EMPTY_ALT, DETAIL_BAR_BG, DETAIL_BAR_H, ORDER_BG, ORDER_SLOT_BG, OVERLAY_ALPHA,
    TEXT_MAIN, TEXT_MUTED, TOP_BAR_BG, TOP_BAR_H,
};

/// Height of the horizontal order row at the top of the content area.
pub const ORDER_ROW_H: f32 = 88.0;

/// Startup system: load every item icon into the asset server at game start.
///
/// Storing the returned [`Handle<Image>`] values in [`PreloadedImages`] keeps
/// the assets alive so they are always ready in the GPU texture cache when any
/// board cell or order card first tries to render them.
pub(crate) fn preload_images(
    db: Res<ItemDatabase>,
    asset_server: Res<AssetServer>,
    mut preloaded: ResMut<PreloadedImages>,
) {
    for item in db.items.values() {
        if let Some(ref path) = item.icon_path {
            preloaded.0.push(asset_server.load(path.clone()));
        }
    }
    // Pre-load chessboard overlay icons so they are ready immediately.
    preloaded.0.push(asset_server.load("images/chessboard/chessboard_icon_crown.png"));
    preloaded.0.push(asset_server.load("images/chessboard/chessboard_icon_energy.png"));
    preloaded.0.push(asset_server.load("images/chessboard/chessboard_item_selected.png"));
}

pub(crate) fn setup_initial_board(mut board: ResMut<Board>) {
    for (col, row, item_id) in load_board_init() {
        board.place(Board::idx(col, row), &item_id);
    }
}

/// Build the board screen UI hierarchy.
///
/// Runs on `OnEnter(GameScreen::Board)`.  The camera is spawned once at
/// `Startup` by a dedicated system and is not recreated here.
pub(crate) fn setup_board_screen(
    mut commands: Commands,
    mut orders: ResMut<Orders>,
    db: Res<ItemDatabase>,
    asset_server: Res<AssetServer>,
) {
    orders.fill_orders(&db);

    let font: Handle<Font> = asset_server.load("fonts/SourceHanSansSC-Regular.ttf");

    // Root — full viewport, column layout
    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BoardScreenRoot,
        ))
        .with_children(|root| {
            spawn_top_bar(root, &font);
            spawn_order_row(root, &font);
            spawn_board_grid(root, &asset_server);
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
        BoardScreenRoot,
    ));
}

/// Despawn all board-screen UI entities on `OnExit(GameScreen::Board)`.
pub(crate) fn teardown_board_screen(
    mut commands: Commands,
    query: Query<Entity, With<BoardScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
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
        // Center: Stats row
        bar.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(24.0),
            ..default()
        })
        .with_children(|stats| {
            spawn_stat_card(stats, "等级", LevelLabel, "1", TEXT_MAIN, font);
            spawn_stat_card(stats, "体力", StaminaLabel, "100/100", ACCENT_GREEN, font);
            spawn_stat_card(
                stats,
                "金币",
                CoinsLabel,
                "0",
                Color::srgb(0.95, 0.80, 0.25),
                font,
            );
            spawn_stat_card(
                stats,
                "红宝石",
                GemsLabel,
                "0",
                Color::srgb(0.55, 0.75, 0.95),
                font,
            );
        });

        // Right: double-stamina toggle button (top-right corner)
        bar.spawn((
            Button,
            Node {
                width: px(100.0),
                height: px(44.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(6.0)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.16, 0.10)),
            BorderColor::all(Color::srgb(0.40, 0.32, 0.20)),
            DoubleStaminaButton,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("×1 体力"),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(TEXT_MUTED),
                DoubleStaminaLabel,
            ));
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
                padding: UiRect::axes(px(8.0), px(6.0)),
                column_gap: px(6.0),
                flex_shrink: 0.0,
                border_radius: BorderRadius::all(px(8.0)),
                border: UiRect::all(px(1.0)),
                align_items: AlignItems::Center,
                overflow: Overflow::hidden(),
                ..default()
            },
            BackgroundColor(ORDER_SLOT_BG),
            BorderColor::all(Color::srgb(0.30, 0.25, 0.18)),
        ))
        .with_children(|s| {
            // Up to 3 item icon slots (hidden by default; shown when order occupies the slot).
            for item_idx in 0..3usize {
                s.spawn((
                    Node {
                        width: px(60.0),
                        height: px(60.0),
                        flex_shrink: 0.0,
                        border_radius: BorderRadius::all(px(6.0)),
                        border: UiRect::all(px(1.0)),
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, OVERLAY_ALPHA)),
                    BorderColor::all(Color::srgb(0.28, 0.22, 0.15)),
                    ImageNode::default(),
                    OrderItemIcon {
                        order_id: slot as u32,
                        item_index: item_idx as u32,
                        cached_item_id: None,
                    },
                ));
            }

            // Complete overlay — absolute, covers the entire card.
            // Shown (Display::Flex) only when the board contains all required items.
            s.spawn((
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.40, 0.12, 0.82)),
                BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                OrderSubmitButton {
                    order_id: slot as u32,
                },
                SubmitBtn,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("✓ 完成"),
                    TextFont {
                        font: font.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 1.0, 0.85)),
                ));
            });
        });
}

// ── Board grid (full-width, fills remaining space) ────────────────────────────

fn spawn_board_grid(root: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
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
            spawn_cell(grid, idx, asset_server);
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
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(DETAIL_BAR_BG),
        BorderColor::all(Color::srgb(0.30, 0.24, 0.16)),
    ))
    .with_children(|bar| {
        // Left: 仓库 button
        spawn_bottom_action_btn(bar, "仓库", font, true);

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
        spawn_bottom_action_btn(bar, "活动", font, false);
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

fn spawn_cell(grid: &mut ChildSpawnerCommands, idx: usize, asset_server: &AssetServer) {
    let col = idx % BOARD_COLS;
    let row = idx / BOARD_COLS;
    let cell_bg = if (col + row) % 2 == 0 {
        CELL_EMPTY
    } else {
        CELL_EMPTY_ALT
    };

    // Overlay icon size: 1/3 of the 48px cell image per side (16px × 16px = 1/9 of cell area).
    let overlay_size = 16.0_f32;

    let crown_handle: Handle<Image> =
        asset_server.load("images/chessboard/chessboard_icon_crown.png");
    let energy_handle: Handle<Image> =
        asset_server.load("images/chessboard/chessboard_icon_energy.png");
    let selected_handle: Handle<Image> =
        asset_server.load("images/chessboard/chessboard_item_selected.png");

    grid.spawn((
        Button,
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(1.0)),
            border_radius: BorderRadius::all(px(4.0)),
            position_type: PositionType::Relative,
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

        // Selected-state border overlay (full cell, hidden by default)
        cell.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                display: Display::None,
                ..default()
            },
            ImageNode::new(selected_handle),
            Pickable::IGNORE,
            CellSelectedOverlay { index: idx },
        ));

        // Crown icon (bottom-right, max-level indicator, hidden by default)
        cell.spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(1.0),
                bottom: Val::Px(1.0),
                width: px(overlay_size),
                height: px(overlay_size),
                display: Display::None,
                ..default()
            },
            ImageNode::new(crown_handle),
            Pickable::IGNORE,
            CellCrownIcon { index: idx },
        ));

        // Energy icon (top-right, gourd item indicator, hidden by default)
        cell.spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(1.0),
                top: Val::Px(1.0),
                width: px(overlay_size),
                height: px(overlay_size),
                display: Display::None,
                ..default()
            },
            ImageNode::new(energy_handle),
            Pickable::IGNORE,
            CellEnergyIcon { index: idx },
        ));
    });
}

// ── Activity screen ───────────────────────────────────────────────────────────

// Palette for the activity / lobby screen.
const ACT_BG: Color = Color::srgb(0.95, 0.90, 0.80);
const ACT_ICON_BG: Color = Color::srgb(0.96, 0.95, 0.86);
const ACT_ICON_BORDER: Color = Color::srgb(0.82, 0.71, 0.55);
const ACT_BADGE_BG: Color = Color::srgb(0.98, 0.98, 0.95);
const ACT_BADGE_BORDER: Color = Color::srgb(0.82, 0.71, 0.55);
const ACT_BADGE_TEXT: Color = Color::srgb(0.55, 0.27, 0.07);
const ACT_HUD_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.50);
const ACT_ENTER_BG: Color = Color::srgb(0.22, 0.44, 0.22);
const ACT_ENTER_BORDER: Color = Color::srgb(0.45, 0.75, 0.45);

/// Build the activity / lobby screen UI.  Runs on `OnEnter(GameScreen::Activity)`.
pub(crate) fn setup_activity_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/SourceHanSansSC-Regular.ttf");

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(ACT_BG),
            ActivityScreenRoot,
        ))
        .with_children(|root| {
            spawn_activity_top_hud(root, &font);
            spawn_activity_left_column(root, &font);
            spawn_activity_right_column(root, &font);
            spawn_activity_bottom_nav(root, &font);
        });
}

/// Despawn all activity-screen UI entities on `OnExit(GameScreen::Activity)`.
pub(crate) fn teardown_activity_screen(
    mut commands: Commands,
    query: Query<Entity, With<ActivityScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ── Activity top HUD ──────────────────────────────────────────────────────────

fn spawn_activity_top_hud(root: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(16.0), px(12.0)),
            column_gap: px(10.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.15)),
        ZIndex(50),
    ))
    .with_children(|hud| {
        // Level badge (circular)
        hud.spawn((
            Node {
                width: px(56.0),
                height: px(56.0),
                border_radius: BorderRadius::all(px(28.0)),
                border: UiRect::all(px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::WHITE),
            BorderColor::all(ACT_ICON_BORDER),
        ))
        .with_children(|badge| {
            badge.spawn((
                Text::new("42"),
                TextFont { font: font.clone(), font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.15, 0.12, 0.08)),
            ));
        });

        // Currency pills row
        hud.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(6.0),
            flex_grow: 1.0,
            ..default()
        })
        .with_children(|row| {
            spawn_currency_pill(row, font, "⚡", "24",   Color::srgba(0.20, 0.78, 0.40, 0.40), true);
            spawn_currency_pill(row, font, "🪙", "2367", Color::srgba(0.88, 0.68, 0.20, 0.40), false);
            spawn_currency_pill(row, font, "💎", "58",   Color::srgba(0.80, 0.20, 0.30, 0.40), true);
        });

        // Settings / menu button
        hud.spawn((
            Button,
            Node {
                padding: UiRect::axes(px(12.0), px(4.0)),
                border_radius: BorderRadius::all(px(999.0)),
                border: UiRect::all(px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(ACT_HUD_BG),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("•••"),
                TextFont { font: font.clone(), font_size: 13.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn spawn_currency_pill(
    row: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    icon: &str,
    value: &str,
    border_color: Color,
    with_plus: bool,
) {
    row.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(8.0), px(0.0)),
            height: px(28.0),
            border_radius: BorderRadius::all(px(999.0)),
            border: UiRect::all(px(1.0)),
            column_gap: px(4.0),
            min_width: px(90.0),
            justify_content: if with_plus {
                JustifyContent::SpaceBetween
            } else {
                JustifyContent::Start
            },
            ..default()
        },
        BackgroundColor(ACT_HUD_BG),
        BorderColor::all(border_color),
    ))
    .with_children(|pill| {
        pill.spawn((
            Node {
                width: px(20.0),
                height: px(20.0),
                border_radius: BorderRadius::all(px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.40)),
        ))
        .with_children(|ic| {
            ic.spawn((
                Text::new(icon),
                TextFont { font: font.clone(), font_size: 11.0, ..default() },
            ));
        });

        pill.spawn((
            Text::new(value),
            TextFont { font: font.clone(), font_size: 11.0, ..default() },
            TextColor(Color::WHITE),
        ));

        if with_plus {
            pill.spawn((
                Node {
                    width: px(16.0),
                    height: px(16.0),
                    border_radius: BorderRadius::all(px(8.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.20, 0.75, 0.65, 0.90)),
            ))
            .with_children(|plus| {
                plus.spawn((
                    Text::new("+"),
                    TextFont { font: font.clone(), font_size: 10.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        }
    });
}

// ── Activity icon columns ─────────────────────────────────────────────────────

struct ActivityEntry {
    icon: &'static str,
    badge: &'static str,
    has_dot: bool,
}

fn spawn_activity_left_column(root: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    let entries = [
        ActivityEntry { icon: "🎪", badge: "13天12时", has_dot: true  },
        ActivityEntry { icon: "📦", badge: "6天11时",  has_dot: false },
        ActivityEntry { icon: "🏠", badge: "1天19时",  has_dot: false },
        ActivityEntry { icon: "🎒", badge: "免费好礼", has_dot: false },
        ActivityEntry { icon: "🐱", badge: "1天11时",  has_dot: false },
    ];
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(88.0),
            left: px(8.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(20.0),
            ..default()
        },
        ZIndex(40),
    ))
    .with_children(|col| {
        for e in &entries {
            spawn_activity_icon_entry(col, font, e);
        }
    });
}

fn spawn_activity_right_column(root: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    let entries = [
        ActivityEntry { icon: "🦊", badge: "6天11时", has_dot: false },
        ActivityEntry { icon: "🎁", badge: "6天9时",  has_dot: false },
        ActivityEntry { icon: "🎀", badge: "3时13分", has_dot: false },
    ];
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(88.0),
            right: px(8.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(20.0),
            ..default()
        },
        ZIndex(40),
    ))
    .with_children(|col| {
        for e in &entries {
            spawn_activity_icon_entry(col, font, e);
        }
    });
}

fn spawn_activity_icon_entry(
    col: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    entry: &ActivityEntry,
) {
    col.spawn(Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(4.0),
        ..default()
    })
    .with_children(|item| {
        item.spawn((
            Button,
            Node {
                position_type: PositionType::Relative,
                width: px(64.0),
                height: px(64.0),
                border_radius: BorderRadius::all(px(32.0)),
                border: UiRect::all(px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(ACT_ICON_BG),
            BorderColor::all(ACT_ICON_BORDER),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(entry.icon),
                TextFont { font: font.clone(), font_size: 28.0, ..default() },
            ));
            if entry.has_dot {
                btn.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(2.0),
                        right: px(2.0),
                        width: px(16.0),
                        height: px(16.0),
                        border_radius: BorderRadius::all(px(8.0)),
                        border: UiRect::all(px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.94, 0.27, 0.27)),
                    BorderColor::all(Color::WHITE),
                    Pickable::IGNORE,
                ))
                .with_children(|dot| {
                    dot.spawn((
                        Text::new("!"),
                        TextFont { font: font.clone(), font_size: 9.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });

        item.spawn((
            Node {
                padding: UiRect::axes(px(8.0), px(2.0)),
                border_radius: BorderRadius::all(px(999.0)),
                border: UiRect::all(px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(ACT_BADGE_BG),
            BorderColor::all(ACT_BADGE_BORDER),
        ))
        .with_children(|badge| {
            badge.spawn((
                Text::new(entry.badge),
                TextFont { font: font.clone(), font_size: 11.0, ..default() },
                TextColor(ACT_BADGE_TEXT),
            ));
        });
    });
}

// ── Activity bottom navigation ─────────────────────────────────────────────────

fn spawn_activity_bottom_nav(root: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(16.0),
            left: Val::Px(0.0),
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(px(24.0)),
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        ZIndex(50),
        Pickable::IGNORE,
    ))
    .with_children(|nav| {
        // Left: large book / quest button
        nav.spawn((
            Button,
            Node {
                position_type: PositionType::Relative,
                width: px(80.0),
                height: px(80.0),
                border_radius: BorderRadius::all(px(40.0)),
                border: UiRect::all(px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(ACT_ICON_BG),
            BorderColor::all(ACT_ICON_BORDER),
            Pickable::default(),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("📖"),
                TextFont { font: font.clone(), font_size: 36.0, ..default() },
            ));
            btn.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: px(4.0),
                    right: px(4.0),
                    width: px(18.0),
                    height: px(18.0),
                    border_radius: BorderRadius::all(px(9.0)),
                    border: UiRect::all(px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.94, 0.27, 0.27)),
                BorderColor::all(Color::WHITE),
                Pickable::IGNORE,
            ))
            .with_children(|dot| {
                dot.spawn((
                    Text::new("!"),
                    TextFont { font: font.clone(), font_size: 10.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        });

        // Right: "enter board" button — bottom-right CTA
        nav.spawn((
            Button,
            Node {
                flex_direction: FlexDirection::Column,
                width: px(96.0),
                height: px(96.0),
                border_radius: BorderRadius::all(px(48.0)),
                border: UiRect::all(px(3.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(2.0),
                ..default()
            },
            BackgroundColor(ACT_ENTER_BG),
            BorderColor::all(ACT_ENTER_BORDER),
            EnterBoardButton,
            Pickable::default(),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("🍵"),
                TextFont { font: font.clone(), font_size: 38.0, ..default() },
            ));
            btn.spawn((
                Text::new("进入游戏"),
                TextFont { font: font.clone(), font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.80, 1.0, 0.80)),
            ));
        });
    });
}
