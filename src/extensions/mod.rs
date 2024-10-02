#[cfg(feature = "gui")]
pub mod gui;

#[cfg(not(feature = "gui"))]
pub mod no_gui;
