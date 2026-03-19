/// Board state and game logic for 合合游戏 (HeHe Game).
/// Manages the 7×9 grid, item placement, and merge operations.
use bevy::prelude::*;

use crate::items::ItemDatabase;

/// Board dimensions
pub const BOARD_COLS: usize = 7;
pub const BOARD_ROWS: usize = 9;
pub const BOARD_SIZE: usize = BOARD_COLS * BOARD_ROWS;

/// Represents one cell of the board.
#[derive(Debug, Clone, Default)]
pub struct Cell {
    /// The item ID occupying this cell, or None if empty.
    pub item_id: Option<String>,
}

/// The main board state resource.
#[derive(Resource, Debug)]
pub struct Board {
    pub cells: Vec<Cell>,
    /// Currently selected cell index (for click-to-select mechanic).
    pub selected: Option<usize>,
    /// Whether the board has changed since last UI update.
    pub dirty: bool,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            cells: vec![Cell::default(); BOARD_SIZE],
            selected: None,
            dirty: true,
        }
    }
}

impl Board {
    /// Convert (col, row) to linear index.
    pub fn idx(col: usize, row: usize) -> usize {
        row * BOARD_COLS + col
    }

    /// Convert linear index to (col, row).
    pub fn pos(idx: usize) -> (usize, usize) {
        (idx % BOARD_COLS, idx / BOARD_COLS)
    }

    /// Place an item at the given cell index. Returns false if cell is occupied.
    pub fn place(&mut self, idx: usize, item_id: &str) -> bool {
        if self.cells[idx].item_id.is_none() {
            self.cells[idx].item_id = Some(item_id.to_string());
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Remove and return the item at the given cell.
    #[allow(dead_code)]
    pub fn take(&mut self, idx: usize) -> Option<String> {
        let item = self.cells[idx].item_id.take();
        if item.is_some() {
            self.dirty = true;
        }
        item
    }

    /// Find the first empty cell. Returns None if board is full.
    pub fn first_empty(&self) -> Option<usize> {
        self.cells.iter().position(|c| c.item_id.is_none())
    }

    /// Find an empty cell adjacent (8-directional) to the given index.
    pub fn adjacent_empty(&self, idx: usize) -> Option<usize> {
        let (col, row) = Self::pos(idx);
        let col = col as isize;
        let row = row as isize;
        for dr in -1isize..=1 {
            for dc in -1isize..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let nc = col + dc;
                let nr = row + dr;
                if nc >= 0 && nc < BOARD_COLS as isize && nr >= 0 && nr < BOARD_ROWS as isize {
                    let ni = Self::idx(nc as usize, nr as usize);
                    if self.cells[ni].item_id.is_none() {
                        return Some(ni);
                    }
                }
            }
        }
        None
    }

    /// Place item in adjacent empty cell, or first empty cell if no adjacent.
    pub fn place_near(&mut self, source_idx: usize, item_id: &str) -> bool {
        if let Some(ni) = self.adjacent_empty(source_idx) {
            self.place(ni, item_id)
        } else if let Some(fi) = self.first_empty() {
            self.place(fi, item_id)
        } else {
            false
        }
    }

    /// Handle a cell click. Returns the action taken.
    pub fn handle_click(&mut self, clicked_idx: usize, db: &ItemDatabase) -> ClickAction {
        let clicked_item = self.cells[clicked_idx].item_id.clone();

        match self.selected {
            None => {
                // Nothing selected — select this cell if it has an item
                if clicked_item.is_some() {
                    self.selected = Some(clicked_idx);
                    ClickAction::Selected(clicked_idx)
                } else {
                    ClickAction::None
                }
            }
            Some(selected_idx) => {
                if selected_idx == clicked_idx {
                    // Clicked on already-selected cell
                    let item_id = clicked_item.as_deref().unwrap_or("");
                    if let Some(item_def) = db.get(item_id) {
                        if item_def.is_generator {
                            // Generator action — handled by caller
                            self.selected = None;
                            return ClickAction::GeneratorActivated(clicked_idx, item_id.to_string());
                        }
                    }
                    // Deselect
                    self.selected = None;
                    ClickAction::Deselected
                } else {
                    let selected_item = self.cells[selected_idx].item_id.clone();

                    // Try to merge
                    if let (Some(sel_id), Some(click_id)) = (&selected_item, &clicked_item) {
                        if db.can_merge(sel_id, click_id) {
                            let result_id = db
                                .get(sel_id)
                                .and_then(|i| i.merge_result_id)
                                .unwrap()
                                .to_string();
                            // Remove both items and place merged result at clicked cell
                            self.cells[selected_idx].item_id = None;
                            self.cells[clicked_idx].item_id = Some(result_id.clone());
                            self.selected = None;
                            self.dirty = true;
                            return ClickAction::Merged {
                                source: selected_idx,
                                target: clicked_idx,
                                result: result_id,
                            };
                        }
                    }

                    // Move selected item to empty clicked cell
                    if clicked_item.is_none() {
                        if let Some(sel_id) = selected_item {
                            self.cells[selected_idx].item_id = None;
                            self.cells[clicked_idx].item_id = Some(sel_id.clone());
                            self.selected = None;
                            self.dirty = true;
                            return ClickAction::Moved {
                                from: selected_idx,
                                to: clicked_idx,
                                item: sel_id,
                            };
                        }
                    }

                    // Select the clicked item instead
                    if clicked_item.is_some() {
                        self.selected = Some(clicked_idx);
                        return ClickAction::Selected(clicked_idx);
                    }

                    self.selected = None;
                    ClickAction::None
                }
            }
        }
    }
}

/// Outcome of a cell click action.
#[derive(Debug, Clone)]
pub enum ClickAction {
    None,
    Selected(usize),
    Deselected,
    Merged {
        #[allow(dead_code)]
        source: usize,
        #[allow(dead_code)]
        target: usize,
        result: String,
    },
    Moved {
        #[allow(dead_code)]
        from: usize,
        #[allow(dead_code)]
        to: usize,
        item: String,
    },
    GeneratorActivated(usize, String),
}

/// Tag component for board cell UI entities.
#[derive(Component, Debug, Clone)]
pub struct BoardCell {
    pub index: usize,
}

/// Tag component for the text inside a board cell.
#[derive(Component, Debug, Clone)]
pub struct CellText {
    pub index: usize,
}

/// Tag component for the image inside a board cell.
#[derive(Component, Debug, Clone)]
pub struct CellImage {
    pub index: usize,
}

/// Tag component for the board grid container.
#[derive(Component, Debug)]
pub struct BoardGrid;

impl Board {
    /// Handle a drag-and-drop from `from` to `to`.
    ///
    /// Tries to merge if both cells have compatible items, otherwise moves the
    /// dragged item to an empty target cell. Returns the action taken.
    pub fn handle_drag(&mut self, from: usize, to: usize, db: &ItemDatabase) -> ClickAction {
        let from_item = self.cells[from].item_id.clone();
        let to_item = self.cells[to].item_id.clone();

        // Try merge when both cells are occupied
        if let (Some(from_id), Some(to_id)) = (&from_item, &to_item) {
            if db.can_merge(from_id, to_id) {
                // `can_merge` guarantees `merge_result_id` is Some; guard defensively.
                let Some(result_id) = db.get(from_id).and_then(|i| i.merge_result_id) else {
                    return ClickAction::None;
                };
                let result_id = result_id.to_string();
                self.cells[from].item_id = None;
                self.cells[to].item_id = Some(result_id.clone());
                self.selected = None;
                self.dirty = true;
                return ClickAction::Merged {
                    source: from,
                    target: to,
                    result: result_id,
                };
            }
            // Incompatible items — cancel drag without moving
            return ClickAction::None;
        }

        // Move to empty target cell
        if to_item.is_none() {
            if let Some(from_id) = from_item {
                self.cells[from].item_id = None;
                self.cells[to].item_id = Some(from_id.clone());
                self.selected = None;
                self.dirty = true;
                return ClickAction::Moved {
                    from,
                    to,
                    item: from_id,
                };
            }
        }

        ClickAction::None
    }
}
