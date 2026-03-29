//! Activity (lobby) screen UI builders for 合合游戏 (HeHe Game).
//!
//! Contains the startup / teardown systems and spawn-helper functions that
//! construct the activity screen UI hierarchy (top HUD, icon columns, bottom nav).
use bevy::prelude::*;

use crate::{ActivityScreenRoot, EnterBoardButton};

// ── Palette for the activity / lobby screen ───────────────────────────────────

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
