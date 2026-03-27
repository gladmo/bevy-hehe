//! Update systems for 合合游戏 (HeHe Game).
//!
//! The systems are organised into four submodules by responsibility:
//! - [`animation`] — rising-star and attract-hint animations
//! - [`drag`] — drag-and-drop input handling
//! - [`logic`] — tick and input systems that mutate game state
//! - [`visuals`] — UI refresh systems (read-only or nearly so)

mod animation;
mod drag;
mod logic;
mod visuals;

pub(crate) use animation::{
    animate_attract_icons, animate_rising_stars, tick_attract_animation,
    tick_idle_timer, tick_star_spawners,
};
pub(crate) use drag::{handle_drag_input, update_drag_ghost};
pub(crate) use logic::{
    handle_button_click, handle_cell_interaction, handle_double_stamina_toggle, handle_order_submit,
    tick_auto_generators, tick_economy, tick_orders,
};
pub(crate) use visuals::{
    update_cell_visuals, update_double_stamina_button, update_economy_ui, update_item_detail_bar,
    update_message_bar, update_order_icons, update_orders_ui,
};
