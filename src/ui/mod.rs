//! This module defines the UI trait
//!
//! It is used to allow exchanging the UI of hired and to insert a dummy UI for script input.

use super::Buffer;

mod dummy_ui;
pub use dummy_ui::DummyUI;

/// The UI trait used to abstract all common UI operations
pub trait UI {
  /// A basic print for output of commands
  fn print(&mut self,
    data: &str,
  ) -> Result<(), &'static str>;

  /// Get a command for parsing and execution
  ///
  /// * Buffer passed in to allow for interactive viewing during input, just ignore if not used
  /// * Must return a single line to be parsed, trimming optional
  fn get_command(&mut self,
    buffer: & dyn Buffer,
  ) -> Result<String, &'static str>;

  /// Get input lines until given character is entered alone on a line
  ///
  /// * Buffer passed in to allow for interactive viewing during input, just ignore if not used
  /// * Must return a vector newline terminated strings and not return the terminating line
  fn get_input(&mut self,
    buffer: & dyn Buffer,
    terminator: char,
  ) -> Result<Vec<String>, &'static str>;

  /// Print the given selection with the given options
  ///
  /// Depending on UI this may mean changing viewport settings and moving to given selection.
  fn print_selection(&mut self,
    buffer: & dyn Buffer,
    selection: (usize, usize),
    numbered: bool,
    literal: bool,
  ) -> Result<(), &'static str>;
}