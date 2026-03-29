//! Board screen UI builders for 合合游戏 (HeHe Game).
//!
//! Contains the startup systems and spawn-helper functions that construct the
//! board screen UI hierarchy (top bar, order row, board grid, bottom bar).
use bevy::prelude::*;

use crate::board::{Board, BoardCell, BoardGrid, CellCrownIcon, CellEnergyIcon, CellImage, CellSelectedOverlay, BOARD_COLS, BOARD_ROWS};
use crate::config::load_board_init;
use crate::economy::{CoinsLabel, GemsLabel, LevelLabel, StaminaLabel};
use crate::items::ItemDatabase;
use crate::orders::{OrderItemIcon, OrderPanel, OrderSubmitButton, Orders};
use crate::{
    ActivityButton, BoardScreenRoot, DetailHint, DetailIcon, DetailName,
    DragGhost, EnergyX1Button, EnergyX2Button,
    MessageLabel, PreloadedImages, SubmitBtn, WarehouseButton, ACCENT, BOARD_BG, CELL_EMPTY,
    CELL_EMPTY_ALT, DETAIL_BAR_BG, DETAIL_BAR_H, ORDER_BG, ORDER_SLOT_BG, OVERLAY_ALPHA,
    TEXT_MAIN, TEXT_MUTED, TOP_BAR_BG,
};
use super::spawn_hud_row;

/// Height of the horizontal order row at the top of the content area.
pub(crate) const ORDER_ROW_H: f32 = 88.0;

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

/// Startup system: place the initial set of items on the board from `board_init.csv`.
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
    mut board: ResMut<crate::board::Board>,
    db: Res<ItemDatabase>,
    asset_server: Res<AssetServer>,
) {
    // Mark the board as changed so `update_cell_visuals` refreshes all cell
    // images on the next frame.  This is necessary because the board UI is
    // recreated from scratch every time the player enters the board screen, but
    // the underlying `Board` resource keeps its data across screen transitions.
    // Without this, pieces would remain invisible on the 2nd+ visit because
    // `update_cell_visuals` skips its image-update block when the board has not
    // changed.
    board.set_changed();

    orders.fill_orders(&db);

    let font: Handle<Font> = asset_server.load("fonts/SourceHanSansSC-Regular.ttf");

    // Root — full viewport, column layout
    let mut hud_handles = None;
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
            hud_handles = Some(spawn_top_bar(root, &font, &asset_server));
            spawn_order_row(root, &font);
            spawn_board_grid(root, &asset_server);
            spawn_bottom_bar(root, &font);
        });

    // Insert label components on the HUD text nodes so visual systems can
    // update them with live economy values.
    if let Some(handles) = hud_handles {
        commands.entity(handles.level_text).insert(LevelLabel);
        commands.entity(handles.stamina_text).insert(StaminaLabel);
        commands.entity(handles.coins_text).insert(CoinsLabel);
        commands.entity(handles.gems_text).insert(GemsLabel);
    }

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

// ── Top bar (HUD row 1 + action row 2) ───────────────────────────────────────

/// Spawns the two-row board HUD.
///
/// Row 1 – level badge + stamina / coins / gems pills (activity-screen style).
/// Row 2 – energy-multiplier buttons (×1 / ×2) + shop button.
///
/// Returns [`HudRowHandles`] so the caller can insert live-update label
/// components (`LevelLabel`, `StaminaLabel`, etc.) on the returned text nodes.
fn spawn_top_bar(
    root: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    asset_server: &AssetServer,
) -> super::HudRowHandles {
    // Row 1: shared activity-style HUD
    let handles = spawn_hud_row(root, font, asset_server, "100/100", "0", "0", "1");

    // Row 2: energy multiplier buttons + shop
    let energy1: Handle<Image> =
        asset_server.load("images/hud/farm_chessboard_img_energy_1.png");
    let energy2: Handle<Image> =
        asset_server.load("images/hud/farm_chessboard_img_energy_2.png");
    let shop_icon: Handle<Image> = asset_server.load("images/hud/main_icon_shop.png");

    root.spawn((
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(8.0),
            padding: UiRect::axes(px(12.0), px(4.0)),
            border: UiRect::bottom(px(2.0)),
            ..default()
        },
        BackgroundColor(TOP_BAR_BG),
        BorderColor::all(Color::srgb(0.35, 0.28, 0.18)),
    ))
    .with_children(|row| {
        // ×1 energy button (active by default)
        row.spawn((
            Button,
            Node {
                width: px(40.0),
                height: px(40.0),
                border_radius: BorderRadius::all(px(8.0)),
                border: UiRect::all(px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.16, 0.10)),
            BorderColor::all(Color::srgb(0.88, 0.50, 0.20)), // active by default
            EnergyX1Button,
        ))
        .with_children(|btn| {
            btn.spawn((
                Node { width: px(28.0), height: px(28.0), ..default() },
                ImageNode::new(energy1),
                Pickable::IGNORE,
            ));
        });

        // ×2 energy button (inactive by default)
        row.spawn((
            Button,
            Node {
                width: px(40.0),
                height: px(40.0),
                border_radius: BorderRadius::all(px(8.0)),
                border: UiRect::all(px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.16, 0.10)),
            BorderColor::all(Color::srgb(0.40, 0.32, 0.20)),
            EnergyX2Button,
        ))
        .with_children(|btn| {
            btn.spawn((
                Node { width: px(28.0), height: px(28.0), ..default() },
                ImageNode::new(energy2),
                Pickable::IGNORE,
            ));
        });

        // Shop button (no interaction required)
        row.spawn((
            Button,
            Node {
                width: px(40.0),
                height: px(40.0),
                border_radius: BorderRadius::all(px(8.0)),
                border: UiRect::all(px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.50)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Node { width: px(28.0), height: px(28.0), ..default() },
                ImageNode::new(shop_icon),
                Pickable::IGNORE,
            ));
        });
    });

    handles
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
