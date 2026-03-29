//! Activity (lobby) screen UI builders for 合合游戏 (HeHe Game).
//!
//! Contains the startup / teardown systems and spawn-helper functions that
//! construct the activity screen UI hierarchy (top HUD, icon columns, bottom nav).
use bevy::prelude::*;

use crate::{
    ActivityIconsContainer, ActivityScreenRoot, EnterBoardButton, HideActivityButton,
    SettingsCenterButton, SettingsDropdown, SettingsOptionButton, VersionInfoPopup,
};
use super::spawn_hud_row;

// ── Palette for the activity / lobby screen ───────────────────────────────────

const ACT_BG: Color = Color::srgb(0.95, 0.90, 0.80);
const ACT_ICON_BG: Color = Color::srgb(0.96, 0.95, 0.86);
const ACT_ICON_BORDER: Color = Color::srgb(0.82, 0.71, 0.55);
const ACT_BADGE_BG: Color = Color::srgb(0.98, 0.98, 0.95);
const ACT_BADGE_BORDER: Color = Color::srgb(0.82, 0.71, 0.55);
const ACT_BADGE_TEXT: Color = Color::srgb(0.55, 0.27, 0.07);
const ACT_ENTER_BG: Color = Color::srgb(0.22, 0.44, 0.22);
const ACT_ENTER_BORDER: Color = Color::srgb(0.45, 0.75, 0.45);
const HUD_BTN_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.50);

/// Build the activity / lobby screen UI.  Runs on `OnEnter(GameScreen::Activity)`.
pub(crate) fn setup_activity_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font: Handle<Font> = asset_server.load("fonts/SourceHanSansSC-Regular.ttf");

    // Pre-load the setting icon handle so it can be passed into the dropdown closure.
    let setting_icon: Handle<Image> = asset_server.load("images/hud/main_icon_setting.png");

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(ACT_BG),
            ActivityScreenRoot,
        ))
        .with_children(|root| {
            // Row 1 – HUD (level badge + currency pills)
            spawn_hud_row(root, &font, &asset_server, "24", "2367", "58", "42");

            // Row 2 – action buttons (right-aligned, below HUD)
            spawn_activity_action_row(root, &asset_server);

            // Icon columns wrapped in a togglable container
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: px(0.0),
                    left: px(0.0),
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
                ActivityIconsContainer,
            ))
            .with_children(|icons| {
                spawn_activity_left_column(icons, &font);
                spawn_activity_right_column(icons, &font);
            });

            spawn_activity_bottom_nav(root, &font);
        });

    // Settings dropdown (absolute popup, hidden by default).
    // Tagged with ActivityScreenRoot so it is torn down with the rest of the screen.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(110.0),
            right: px(16.0),
            flex_direction: FlexDirection::Column,
            min_width: px(140.0),
            border_radius: BorderRadius::all(px(8.0)),
            border: UiRect::all(px(1.0)),
            padding: UiRect::all(px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.10, 0.10, 0.10, 0.92)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
        ZIndex(200),
        Visibility::Hidden,
        SettingsDropdown,
        ActivityScreenRoot,
    ))
    .with_children(|dd| {
        // "设置" option row
        dd.spawn((
            Button,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8.0),
                padding: UiRect::axes(px(10.0), px(8.0)),
                border_radius: BorderRadius::all(px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
            SettingsOptionButton,
        ))
        .with_children(|row| {
            row.spawn((
                Node { width: px(20.0), height: px(20.0), ..default() },
                ImageNode::new(setting_icon),
                Pickable::IGNORE,
            ));
            row.spawn((
                Text::new("设置"),
                TextFont { font: font.clone(), font_size: 13.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });

    // Version-info popup (absolute, hidden by default; clicking it closes it).
    commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            top: px(110.0),
            right: px(16.0),
            min_width: px(180.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(6.0),
            border_radius: BorderRadius::all(px(10.0)),
            border: UiRect::all(px(1.0)),
            padding: UiRect::all(px(16.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.10, 0.10, 0.10, 0.95)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.25)),
        ZIndex(300),
        Visibility::Hidden,
        VersionInfoPopup,
        ActivityScreenRoot,
    ))
    .with_children(|popup| {
        popup.spawn((
            Text::new("游戏版本信息"),
            TextFont { font: font.clone(), font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.85, 0.78, 0.60)),
        ));
        popup.spawn((
            Text::new(concat!("版本：", env!("BUILD_TIME"))),
            TextFont { font: font.clone(), font_size: 12.0, ..default() },
            TextColor(Color::WHITE),
        ));
        popup.spawn((
            Text::new("点击关闭"),
            TextFont { font: font.clone(), font_size: 10.0, ..default() },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.50)),
        ));
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

// ── Activity action button row ─────────────────────────────────────────────────

fn spawn_activity_action_row(
    root: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
) {
    let hide_icon: Handle<Image> =
        asset_server.load("images/hud/main_icon_hide_activity.png");
    let storage_icon: Handle<Image> = asset_server.load("images/hud/main_icon_storage.png");
    let shop_icon: Handle<Image> = asset_server.load("images/hud/main_icon_shop.png");

    root.spawn((
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            column_gap: px(8.0),
            padding: UiRect::axes(px(12.0), px(6.0)),
            ..default()
        },
        ZIndex(60),
    ))
    .with_children(|row| {
        // Hide-activity toggle
        row.spawn((
            Button,
            Node {
                width: px(36.0),
                height: px(36.0),
                border_radius: BorderRadius::all(px(8.0)),
                border: UiRect::all(px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(HUD_BTN_BG),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
            HideActivityButton,
        ))
        .with_children(|btn| {
            btn.spawn((
                Node { width: px(24.0), height: px(24.0), ..default() },
                ImageNode::new(hide_icon),
                Pickable::IGNORE,
            ));
        });

        // Settings-center button (opens dropdown)
        row.spawn((
            Button,
            Node {
                width: px(36.0),
                height: px(36.0),
                border_radius: BorderRadius::all(px(8.0)),
                border: UiRect::all(px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(HUD_BTN_BG),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
            SettingsCenterButton,
        ))
        .with_children(|btn| {
            btn.spawn((
                Node { width: px(24.0), height: px(24.0), ..default() },
                ImageNode::new(storage_icon),
                Pickable::IGNORE,
            ));
        });

        // Shop button (no interaction required)
        row.spawn((
            Button,
            Node {
                width: px(36.0),
                height: px(36.0),
                border_radius: BorderRadius::all(px(8.0)),
                border: UiRect::all(px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(HUD_BTN_BG),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Node { width: px(24.0), height: px(24.0), ..default() },
                ImageNode::new(shop_icon),
                Pickable::IGNORE,
            ));
        });
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
            top: px(108.0),
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
            top: px(108.0),
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
