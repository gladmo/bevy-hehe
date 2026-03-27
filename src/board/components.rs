use bevy::prelude::*;

/// Tag component for board cell UI entities.
#[derive(Component, Debug, Clone)]
pub struct BoardCell {
    pub index: usize,
}

/// Tag component for the image inside a board cell.
#[derive(Component, Debug, Clone)]
pub struct CellImage {
    pub index: usize,
}

/// Tag component for the crown icon overlay (bottom-right) on max-level cells.
#[derive(Component, Debug, Clone)]
pub struct CellCrownIcon {
    pub index: usize,
}

/// Tag component for the energy icon overlay (top-right) on gourd cells.
#[derive(Component, Debug, Clone)]
pub struct CellEnergyIcon {
    pub index: usize,
}

/// Tag component for the selected-state border overlay on board cells.
#[derive(Component, Debug, Clone)]
pub struct CellSelectedOverlay {
    pub index: usize,
}

/// Tag component for the board grid container.
#[derive(Component, Debug)]
pub struct BoardGrid;
