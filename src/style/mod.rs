//! Module used for styling and writing into specific regions of terminal.
//! (will) Contains support for different themes and allows regions to write to terminal without knowing their containers.
mod canvas;
mod style;

pub use canvas::*;
pub use style::*;