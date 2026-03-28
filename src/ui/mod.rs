//! UI setup for 合合游戏 (HeHe Game).
//!
//! Split into two submodules by screen:
//! - [`activity`] — lobby / activity screen
//! - [`board`]    — main merge-puzzle board screen

mod activity;
mod board;

pub(crate) use activity::{setup_activity_screen, teardown_activity_screen};
pub(crate) use board::{preload_images, setup_board_screen, setup_initial_board, teardown_board_screen};
