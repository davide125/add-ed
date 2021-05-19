//! Contains the Buffer trait and any build in implementations.

// Include a general file handler
// Separated to enable URL based buffers and other creative solutions
pub mod file;

// Include the buffer implementations based on features

#[cfg(feature = "vecbuffer")]
mod vecbuffer;
#[cfg(feature = "vecbuffer")]
pub use vecbuffer::*;

/// Trait that defines a buffer supporting 'ed's base commands
pub trait Buffer {

  // Functions for resolving and verifying indices in the parser
  /// Return the number of lines stored in the buffer
  fn len(&self)
    -> usize ;
  /// Get line tagged with given letter. Not found is error
  fn get_tag(&self, tag: char)
    -> Result<usize, &'static str> ;
  /// Return the nearest previous/following index in the selection that contains the regex pattern
  fn get_matching(&self, pattern: &str, curr_line: usize, backwards: bool)
    -> Result<usize, &'static str> ;

  // Regex matching for the macro commands ('g', 'v', 'G', 'V')
  /// Set the matched flag on all lines matching given pattern
  fn mark_matching(&mut self, pattern: &str, selection: (usize, usize), inverse: bool)
    -> Result<(), &'static str> ;
  /// Get a line with the matched flag set, clearing that line's flag
  fn get_marked(&mut self)
    -> Result<Option<usize>, &'static str> ;

  // Simple buffer modifications, but with possibly complex storage
  /// Mark a line with a letter, non letter chars should error
  fn tag_line(&mut self, index: usize, tag: char) 
    -> Result<(), &'static str> ;
  /// Takes a iterator over lines in strings and inserts at given index
  fn insert<'a>(&mut self, data: &mut dyn Iterator<Item = &'a str>, index: usize)
    -> Result<(), &'static str> ;
  /// Cut the selection from the buffer, into the clipboard
  fn cut(&mut self, selection: (usize, usize))
    -> Result<(), &'static str> ;
  /// Equal to cut of selection and insert at start of selection.
  fn change<'a>(&mut self, data: &mut dyn Iterator<Item = &'a str>, selection: (usize, usize))
    -> Result<(), &'static str> ;
  /// Move selection to index
  fn mov(&mut self, selection: (usize, usize), index: usize)
    -> Result<(), &'static str> ;
  /// Moves a copy of the selection to index
  fn mov_copy(&mut self, selection: (usize, usize), index: usize)
    -> Result<(), &'static str> ;
  /// Join all lines in selection into one line
  fn join(&mut self, selection: (usize, usize))
    -> Result<(), &'static str> ;
  /// Copy selected lines into clipboard
  fn copy(&mut self, selection: (usize, usize))
    -> Result<(), &'static str> ;
  /// Paste the clipboard contents to given index
  /// Leave clipboard unchanged
  fn paste(&mut self, index: usize)
    -> Result<usize, &'static str> ;
  /// Perform regex search and replace on the selection changing pattern.0 to pattern.1
  /// If pattern is empty, should re-use stored pattern from previous s command
  /// Returns selection, since it may delete or add lines
  fn search_replace(&mut self, pattern: (&str, &str), selection: (usize, usize), global: bool)
    -> Result<(usize, usize), &'static str> ;

  // Save/load commands. Here to enable creative Buffers, such as ssh+sed for remote editing
  /// Read to the buffer from given path
  /// If index is None replaces current buffer with read lines
  /// Return number of lines read
  fn read_from(&mut self, path: &str, index: Option<usize>, must_exist: bool)
    -> Result<usize, &'static str> ;
  /// Write the buffer to given path
  fn write_to(&mut self, selection: Option<(usize, usize)>, path: &str, append: bool)
    -> Result<(), &'static str> ;
  /// Returns true if no changes have been made since last saving
  fn saved(&self)
    -> bool ;

  // Finally, the basic output command.
  /// Return the given selection without any formatting
  fn get_selection<'a>(&'a self, selection: (usize, usize))
    -> Result<Box<dyn Iterator<Item = &'a str> + 'a>, &'static str> ;
}

// General index and selection validation functions
// These are good to run before using arguments to your buffer

/// Verify that the index is between 0 and buffer.len() inclusive.
///
/// That means it is valid to move to the index in question, but may not be valid to read from.
pub fn verify_index(
  buffer: &impl Buffer,
  index: usize,
) -> Result<(), &'static str> {
  // Indices are valid at len, since that is needed to append to the buffer
  if index > buffer.len() { return Err(crate::error_consts::INDEX_TOO_BIG); }
  Ok(())
}

/// Verify that all lines in the selection exist and that it isn't empty.
///
/// This will always error if buffer.len() == 0, since there are no lines that exist.
pub fn verify_selection(
  buffer: &impl Buffer,
  selection: (usize, usize),
) -> Result<(), &'static str> {
  // A selection must contain something to be valid
  if selection.0 >= selection.1 { return Err(crate::error_consts::SELECTION_EMPTY); }
  // It cannot contain non-existent lines, such as index buffer.len() and beyond
  if selection.1 >= buffer.len() { return Err(crate::error_consts::INDEX_TOO_BIG); }
  Ok(())
}