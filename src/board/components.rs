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

/// Tag component for the board grid container.
#[derive(Component, Debug)]
pub struct BoardGrid;
