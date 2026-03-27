/// Item chain data loader for 合合游戏 (HeHe Game).
///
/// Item definitions are loaded from the CSV configuration files in
/// `assets/config/` via the `config` module. This keeps the data
/// editable as plain CSV files without rewriting Rust source code.

/// Return all item definitions, loaded from `assets/config/items.csv`
/// and `assets/config/item_generates.csv`.
pub use crate::config::load_items as all_items;
