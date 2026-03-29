//! UI setup for 合合游戏 (HeHe Game).
//!
//! Split into two submodules by screen:
//! - [`activity`] — lobby / activity screen
//! - [`board`]    — main merge-puzzle board screen

mod activity;
mod board;

pub(crate) use activity::{setup_activity_screen, teardown_activity_screen};
pub(crate) use board::{preload_images, setup_board_screen, setup_initial_board, teardown_board_screen};

// ── Shared HUD helpers ────────────────────────────────────────────────────────

use bevy::prelude::*;

// ── Colour constants shared between the activity and board HUDs ───────────────

const HUD_PILL_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.50);

/// Entity IDs for the dynamic-text nodes inside the shared HUD row.
/// Callers that need live updating (board screen) insert the appropriate
/// label components (`LevelLabel`, `StaminaLabel`, …) on these entities.
pub(crate) struct HudRowHandles {
    pub level_text: Entity,
    pub stamina_text: Entity,
    pub coins_text: Entity,
    pub gems_text: Entity,
}

/// Spawn the activity-style HUD row (level badge + three currency pills).
///
/// Returns [`HudRowHandles`] so the caller can insert label components for
/// live value updates (required by the board screen).
///
/// `initial_*` are the text values displayed before any system updates them.
pub(crate) fn spawn_hud_row(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    asset_server: &AssetServer,
    initial_stamina: &str,
    initial_coins: &str,
    initial_gems: &str,
    initial_level: &str,
) -> HudRowHandles {
    let stamina_icon: Handle<Image> = asset_server.load("images/hud/goods_icon_101.png");
    let coins_icon: Handle<Image> = asset_server.load("images/hud/goods_icon_102.png");
    let gems_icon: Handle<Image> = asset_server.load("images/hud/goods_icon_103.png");

    let mut level_text_id = Entity::PLACEHOLDER;
    let mut stamina_text_id = Entity::PLACEHOLDER;
    let mut coins_text_id = Entity::PLACEHOLDER;
    let mut gems_text_id = Entity::PLACEHOLDER;

    parent
        .spawn((
            Node {
                width: percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::axes(px(16.0), px(8.0)),
                column_gap: px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.15)),
        ))
        .with_children(|hud| {
            // Level badge (circular)
            hud.spawn((
                Node {
                    width: px(48.0),
                    height: px(48.0),
                    border_radius: BorderRadius::all(px(24.0)),
                    border: UiRect::all(px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_shrink: 0.0,
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                BorderColor::all(Color::srgb(0.82, 0.71, 0.55)),
            ))
            .with_children(|badge| {
                let id = badge
                    .spawn((
                        Text::new(initial_level),
                        TextFont { font: font.clone(), font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.15, 0.12, 0.08)),
                    ))
                    .id();
                level_text_id = id;
            });

            // Currency pills row (flex-grow fills remaining space)
            hud.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(6.0),
                flex_grow: 1.0,
                ..default()
            })
            .with_children(|row| {
                stamina_text_id = spawn_hud_pill(
                    row,
                    font,
                    stamina_icon,
                    initial_stamina,
                    Color::srgba(0.20, 0.78, 0.40, 0.40),
                    true,
                );
                coins_text_id = spawn_hud_pill(
                    row,
                    font,
                    coins_icon,
                    initial_coins,
                    Color::srgba(0.88, 0.68, 0.20, 0.40),
                    false,
                );
                gems_text_id = spawn_hud_pill(
                    row,
                    font,
                    gems_icon,
                    initial_gems,
                    Color::srgba(0.80, 0.20, 0.30, 0.40),
                    true,
                );
            });
        });

    HudRowHandles {
        level_text: level_text_id,
        stamina_text: stamina_text_id,
        coins_text: coins_text_id,
        gems_text: gems_text_id,
    }
}

/// Spawn a single currency pill (icon image + value text + optional "+" button).
///
/// Returns the `Entity` of the value-text node so callers can insert a label
/// component on it.
fn spawn_hud_pill(
    row: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    icon_handle: Handle<Image>,
    initial_value: &str,
    border_color: Color,
    with_plus: bool,
) -> Entity {
    let mut text_id = Entity::PLACEHOLDER;
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
        BackgroundColor(HUD_PILL_BG),
        BorderColor::all(border_color),
    ))
    .with_children(|pill| {
        // Icon image
        pill.spawn((
            Node {
                width: px(20.0),
                height: px(20.0),
                border_radius: BorderRadius::all(px(10.0)),
                ..default()
            },
            ImageNode::new(icon_handle),
        ));

        // Value text
        let id = pill
            .spawn((
                Text::new(initial_value),
                TextFont { font: font.clone(), font_size: 11.0, ..default() },
                TextColor(Color::WHITE),
            ))
            .id();
        text_id = id;

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
    text_id
}

