//! CSV configuration loaders for 合合游戏 (HeHe Game).
//!
//! All configuration tables are embedded at compile time using `include_str!`
//! so they work on both native and WASM targets without any async loading.
//!
//! Configuration files live in `assets/config/`:
//! - `items.csv`          — item definitions
//! - `item_generates.csv` — weighted generation options per generator item
//! - `board_init.csv`     — initial board layout
//! - `orders.csv`         — order templates
//! - `audio.csv`          — audio effect definitions

use crate::items::types::{ChainType, GenerationOption, ItemDef};

/// Raw CSV content embedded at compile time.
const ITEMS_CSV: &str = include_str!("../../assets/config/items.csv");
const ITEM_GENERATES_CSV: &str = include_str!("../../assets/config/item_generates.csv");
const BOARD_INIT_CSV: &str = include_str!("../../assets/config/board_init.csv");
const ORDERS_CSV: &str = include_str!("../../assets/config/orders.csv");
const AUDIO_CSV: &str = include_str!("../../assets/config/audio.csv");

/// Expected number of columns in `items.csv`.
const ITEMS_CSV_COLUMNS: usize = 16;

// ── CSV helpers ───────────────────────────────────────────────────────────────

/// Parse a boolean field ("true" / "false").
fn parse_bool(s: &str) -> bool {
    s.trim() == "true"
}

/// Parse an optional string field — empty string becomes `None`.
fn parse_opt_str(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() { None } else { Some(t.to_string()) }
}

/// Parse a `ChainType` from a string name.
fn parse_chain(s: &str) -> ChainType {
    match s.trim() {
        "Poultry"    => ChainType::Poultry,
        "Egg"        => ChainType::Egg,
        "Teapot"     => ChainType::Teapot,
        "CoolTea"    => ChainType::CoolTea,
        "RiceBall"   => ChainType::RiceBall,
        "Basket"     => ChainType::Basket,
        "Dough"      => ChainType::Dough,
        "Watermelon" => ChainType::Watermelon,
        "CraftBox"   => ChainType::CraftBox,
        "Lantern"    => ChainType::Lantern,
        "Dresser"    => ChainType::Dresser,
        "Ring"       => ChainType::Ring,
        "PeaceLock"  => ChainType::PeaceLock,
        "Loom"       => ChainType::Loom,
        "Fabric"     => ChainType::Fabric,
        "Pouch"      => ChainType::Pouch,
        "RedBox"     => ChainType::RedBox,
        "GreenBox"   => ChainType::GreenBox,
        "Gold"       => ChainType::Gold,
        "Gourd"      => ChainType::Gourd,
        "Ruby"       => ChainType::Ruby,
        other => panic!("Unknown chain type '{other}' in items.csv (check your chain name spelling)"),
    }
}

// ── Public loaders ────────────────────────────────────────────────────────────

/// Load all item definitions from the embedded `items.csv` and
/// `item_generates.csv` configuration files.
///
/// # CSV columns — items.csv
/// `id, chain, level, name, icon_path, is_generator, is_auto_generator,
/// generates_id, auto_gen_interval_secs, merge_result_id, bg_r, bg_g, bg_b,
/// generates_count, consumes_on_generate, max_generate_count`
///
/// # CSV columns — item_generates.csv
/// `item_id, generates_item_id, weight`
pub fn load_items() -> Vec<ItemDef> {
    use std::collections::HashMap;

    // ── 1. Build the generation-options map ───────────────────────────────────
    let mut gen_map: HashMap<String, Vec<GenerationOption>> = HashMap::new();

    for (line_no, line) in ITEM_GENERATES_CSV.lines().enumerate() {
        if line_no < 3 || line.trim().is_empty() {
            continue; // skip 3-row header and blank lines
        }
        let cols: Vec<&str> = line.splitn(3, ',').collect();
        if cols.len() < 3 {
            bevy::log::warn!(
                "item_generates.csv line {}: expected 3 columns, got {}; skipping",
                line_no + 1,
                cols.len()
            );
            continue;
        }
        let item_id = cols[0].trim().to_string();
        let gen_id  = cols[1].trim().to_string();
        let weight: u32 = cols[2].trim().parse().unwrap_or(0);
        gen_map
            .entry(item_id)
            .or_default()
            .push(GenerationOption { item_id: gen_id, weight });
    }

    // ── 2. Parse items.csv ────────────────────────────────────────────────────
    let mut items = Vec::new();

    for (line_no, line) in ITEMS_CSV.lines().enumerate() {
        if line_no < 3 || line.trim().is_empty() {
            continue; // skip 3-row header and blank lines
        }
        let cols: Vec<&str> = line.splitn(ITEMS_CSV_COLUMNS, ',').collect();
        if cols.len() < ITEMS_CSV_COLUMNS {
            bevy::log::warn!(
                "items.csv line {}: expected {} columns, got {}; skipping",
                line_no + 1,
                ITEMS_CSV_COLUMNS,
                cols.len()
            );
            continue;
        }

        let id               = cols[0].trim().to_string();
        let chain            = parse_chain(cols[1]);
        let level: u32       = cols[2].trim().parse().unwrap_or(1);
        let name             = cols[3].trim().to_string();
        let icon_path        = parse_opt_str(cols[4]);
        let is_generator     = parse_bool(cols[5]);
        let is_auto_generator = parse_bool(cols[6]);
        let generates_id     = parse_opt_str(cols[7]);
        let auto_gen_interval_secs: f32 = cols[8].trim().parse().unwrap_or(0.0);
        let merge_result_id  = parse_opt_str(cols[9]);
        let bg_r: f32        = cols[10].trim().parse().unwrap_or(0.5);
        let bg_g: f32        = cols[11].trim().parse().unwrap_or(0.5);
        let bg_b: f32        = cols[12].trim().parse().unwrap_or(0.5);
        let generates_count: u32 = cols[13].trim().parse().unwrap_or(1);
        let consumes_on_generate = parse_bool(cols[14]);
        let max_generate_count: u32 = cols[15].trim().parse().unwrap_or(0);

        let generates = gen_map.remove(&id).unwrap_or_default();

        items.push(ItemDef {
            id,
            chain,
            level,
            name,
            icon_path,
            is_generator,
            is_auto_generator,
            generates,
            generates_id,
            auto_gen_interval_secs,
            merge_result_id,
            bg_color: (bg_r, bg_g, bg_b),
            generates_count,
            consumes_on_generate,
            max_generate_count,
        });
    }

    items
}

/// Load the initial board layout from the embedded `board_init.csv`.
///
/// # CSV format
/// The file uses a grid layout where each data row (after a 3-row header)
/// corresponds to a board row, and each comma-separated column corresponds
/// to a board column. Empty cells are represented by empty strings.
///
/// Header rows (first 3 lines):
/// - Row 1: column names (`col_0` … `col_6`)
/// - Row 2: field types
/// - Row 3: Chinese descriptions
///
/// Returns a `Vec<(col, row, item_id)>`.
pub fn load_board_init() -> Vec<(usize, usize, String)> {
    let mut entries = Vec::new();

    for (line_no, line) in BOARD_INIT_CSV.lines().enumerate() {
        if line_no < 3 || line.trim().is_empty() {
            continue; // skip 3-row header and blank lines
        }
        let board_row = line_no - 3;
        for (board_col, cell) in line.split(',').enumerate() {
            let item_id = cell.trim().to_string();
            if !item_id.is_empty() {
                entries.push((board_col, board_row, item_id));
            }
        }
    }

    entries
}

/// Load order templates from the embedded `orders.csv`.
///
/// # CSV columns
/// `items, coin_reward`
///
/// The `items` column is a semicolon-separated list of item IDs (1–3 items).
///
/// Returns a `Vec<(Vec<String>, u64)>` where each element is
/// `(item_ids, coin_reward)`.
pub fn load_orders() -> Vec<(Vec<String>, u64)> {
    let mut templates = Vec::new();

    for (line_no, line) in ORDERS_CSV.lines().enumerate() {
        if line_no < 3 || line.trim().is_empty() {
            continue; // skip 3-row header and blank lines
        }
        // Split only on the last comma to support multi-item fields with semicolons.
        let split_pos = match line.rfind(',') {
            Some(p) => p,
            None => continue,
        };
        let items_field  = &line[..split_pos];
        let reward_field = &line[split_pos + 1..];

        let coin_reward: u64 = reward_field.trim().parse().unwrap_or(0);
        let item_ids: Vec<String> = items_field
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if !item_ids.is_empty() {
            templates.push((item_ids, coin_reward));
        }
    }

    templates
}

// ── Audio ─────────────────────────────────────────────────────────────────────

/// A single audio effect definition loaded from `audio.csv`.
#[allow(dead_code)]
pub struct AudioDef {
    /// Unique code used to look up this audio entry (e.g. `"bgm_main"`).
    pub audio_code: String,
    /// Asset path relative to the `assets/` directory (e.g. `"audio/bgm_SpringFestival_V1.wav"`).
    pub audio_path: String,
    /// Human-readable description of the audio clip.
    pub description: String,
}

/// Load all audio definitions from the embedded `audio.csv`.
///
/// # CSV columns
/// `audio_code, audio_path, description`
///
/// Returns a `Vec<AudioDef>` in file order.
pub fn load_audio() -> Vec<AudioDef> {
    let mut entries = Vec::new();

    for (line_no, line) in AUDIO_CSV.lines().enumerate() {
        if line_no < 3 || line.trim().is_empty() {
            continue; // skip 3-row header and blank lines
        }
        // splitn(3, ',') means the third element captures the rest of the
        // line, so description values containing commas are handled correctly.
        let cols: Vec<&str> = line.splitn(3, ',').collect();
        if cols.len() < 3 {
            bevy::log::warn!(
                "audio.csv line {}: expected 3 columns, got {}; skipping",
                line_no + 1,
                cols.len()
            );
            continue;
        }
        entries.push(AudioDef {
            audio_code:  cols[0].trim().to_string(),
            audio_path:  cols[1].trim().to_string(),
            description: cols[2].trim().to_string(),
        });
    }

    entries
}
