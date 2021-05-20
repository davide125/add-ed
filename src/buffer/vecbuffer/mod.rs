//! Holds the VecBuffer, a simple Vector based buffer implementation.

use core::iter::Iterator;

use super::*;
use crate::error_consts::*;

//#[test]
//mod test;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Line {
  tag: char,
  matched: bool,
  text: String,
}

/// VecBuffer, the default Buffer implementation
///
/// It is based on storing the text in a Vector of lines.
/// Regex functionality is imported from the Regex crate.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct VecBuffer {
  saved: bool,
  // Chars used for tagging. No tag equates to NULL in the char
  buffer: Vec<Line>,
  clipboard: Vec<Line>,
}
impl VecBuffer {
  /// Create a new empty buffer. It is considered saved while unchanged.
  pub fn new() -> Self
  {
    Self{
      saved: true,
      buffer: Vec::new(),
      clipboard: Vec::new(),
    }
  }
}
impl Buffer for VecBuffer {
  // Index operations, get and verify
  fn len(&self) -> usize {
      self.buffer.len()
  }
  fn get_tag(&self, tag: char)
    -> Result<usize, &'static str>
  {
    let mut index = 0;
    for line in &self.buffer[..] {
      if &tag == &line.tag { return Ok(index); }
      index += 1;
    }
    Err(NO_MATCH)
  }
  fn get_matching(&self, pattern: &str, curr_line: usize, backwards: bool)
    -> Result<usize, &'static str>
  {
    verify_index(self, curr_line)?;
    use regex::RegexBuilder;
    let regex = RegexBuilder::new(pattern)
      .multi_line(true)
      .build()
      .map_err(|_| INVALID_REGEX)
    ?;
    // Figure out how far to iterate
    let length = if ! backwards {
      self.buffer.len().saturating_sub(curr_line + 1)
    } else {
      curr_line
    };

    // Since the range must be positive we subtract from bufferlen for backwards
    for index in 0 .. length {
      if backwards {
        if regex.is_match(&(self.buffer[curr_line - 1 - index].text)) {
          return Ok(curr_line - 1 - index)
        }
      } else {
        if regex.is_match(&(self.buffer[curr_line + index + 1].text)) {
          return Ok(curr_line + index + 1)
        }
      }
    }
    Err(NO_MATCH)
  }

  // For macro commands
  fn mark_matching(&mut self, pattern: &str, selection: (usize, usize), inverse: bool)
    -> Result<(), &'static str>
  {
    use regex::RegexBuilder;
    verify_selection(self, selection)?;
    let regex = RegexBuilder::new(pattern)
      .multi_line(true)
      .build()
      .map_err(|_| INVALID_REGEX)
    ?;
    for index in 0 .. self.len() {
      if index >= selection.0 && index <= selection.1 {
        self.buffer[index].matched = regex.is_match(&(self.buffer[index].text)) ^ inverse;
      }
      else {
        self.buffer[index].matched = false;
      }
    }
    Ok(())
  }
  fn get_marked(&mut self)
    -> Result<Option<usize>, &'static str>
  {
    for index in 0 .. self.buffer.len() {
      if self.buffer[index].matched {
        self.buffer[index].matched = false;
        return Ok(Some(index));
      }
    }
    Ok(None)
  }

  // Simple buffer modifications:
  fn tag_line(&mut self, index: usize, tag: char)
    -> Result<(), &'static str>
  {
    // Overwrite current char with given char
    self.buffer[index].tag = tag;
    Ok(())
  }
  // Take an iterator over &str as data
  fn insert<'a>(&mut self, data: &mut dyn Iterator<Item = &'a str>, index: usize)
    -> Result<(), &'static str>
  {
    // Possible TODO: preallocate for the insert
    verify_index(self, index)?;
    self.saved = false;
    // To minimise time complexity we split the vector immediately
    let mut tail = self.buffer.split_off(index);
    // Then append the insert data
    for line in data {
      self.buffer.push(Line{tag: '\0', matched: false, text: line.to_string()});
    }
    // And finally the cut off tail
    self.buffer.append(&mut tail);
    Ok(())
  }
  fn cut(&mut self, selection: (usize, usize)) -> Result<(), &'static str>
  {
    verify_selection(self, selection)?;
    self.saved = false;
    let mut tail = self.buffer.split_off(selection.1 + 1);
    self.clipboard = self.buffer.split_off(selection.0);
    self.buffer.append(&mut tail);
    Ok(())
  }
  fn change<'a>(&mut self, data: &mut dyn Iterator<Item = &'a str>, selection: (usize, usize))
    -> Result<(), &'static str>
  {
    verify_selection(self, selection)?;
    self.saved = false;
    let mut tail = self.buffer.split_off(selection.1 + 1);
    self.clipboard = self.buffer.split_off(selection.0);
    for line in data {
      self.buffer.push(Line{tag: '\0', matched: false, text: line.to_string()});
    }
    self.buffer.append(&mut tail);
    Ok(())
  }
  fn mov(&mut self, selection: (usize, usize), index: usize) -> Result<(), &'static str> {
    verify_selection(self, selection)?;
    verify_index(self, index)?;
    // Operation varies depending on moving forward or back
    if index <= selection.0 {
      // split out the relevant parts of the buffer
      let mut tail = self.buffer.split_off(selection.1 + 1);
      let mut data = self.buffer.split_off(selection.0);
      let mut middle = self.buffer.split_off(index.saturating_sub(1));
      // Reassemble
      self.buffer.append(&mut data);
      self.buffer.append(&mut middle);
      self.buffer.append(&mut tail);
      Ok(())
    }
    else if index >= selection.1 {
      // split out the relevant parts of the buffer
      let mut tail = self.buffer.split_off(index);
      let mut middle = self.buffer.split_off(selection.1 + 1);
      let mut data = self.buffer.split_off(selection.0);
      // Reassemble
      self.buffer.append(&mut middle);
      self.buffer.append(&mut data);
      self.buffer.append(&mut tail);
      Ok(())
    }
    else {
      Err(MOVE_INTO_SELF)
    }
  }
  fn mov_copy(&mut self, selection: (usize, usize), index: usize) -> Result<(), &'static str> {
    verify_selection(self, selection)?;
    verify_index(self, index)?;
    // Get the data
    let mut data = Vec::new();
    for line in &self.buffer[selection.0 ..= selection.1] {
      data.push(line.clone());
    }
    // Insert it, subtract one if copying to before selection
    let i = if index <= selection.0 {
      index.saturating_sub(1)
    }
    else {
      index
    };
    let mut tail = self.buffer.split_off(i);
    self.buffer.append(&mut data);
    self.buffer.append(&mut tail);
    Ok(())
  }
  fn join(&mut self, selection: (usize, usize)) -> Result<(), &'static str> {
    verify_selection(self, selection)?;
    // Take out the lines that should go away efficiently
    let mut tail = self.buffer.split_off(selection.1 + 1);
    let data = self.buffer.split_off(selection.0 + 1);
    self.buffer.append(&mut tail);
    // Add their contents to the line left in
    for line in data {
      self.buffer[selection.0].text.pop(); // Remove the existing newline
      self.buffer[selection.0].text.push_str(&line.text); // Add in the line
    }
    Ok(())
  }
  fn copy(&mut self, selection: (usize, usize)) -> Result<(), &'static str> {
    verify_selection(self, selection)?;
    self.clipboard = Vec::new();
    // copy out each line in selection
    for line in &self.buffer[selection.0 ..= selection.1] {
      self.clipboard.push(line.clone());
    }
    Ok(())
  }
  fn paste(&mut self, index: usize) -> Result<usize, &'static str> {
    verify_index(self, index)?;
    // Cut off the tail in one go, to reduce time complexity
    let mut tmp = self.buffer.split_off(index);
    // Then append copies of all lines in clipboard
    for line in &self.clipboard {
      self.buffer.push(line.clone());
    }
    // Finally put back the tail
    self.buffer.append(&mut tmp);
    Ok(self.clipboard.len())
  }
  fn search_replace(&mut self, pattern: (&str, &str), selection: (usize, usize), global: bool) -> Result<(usize, usize), &'static str>
  {
    use regex::RegexBuilder;
    // ensure that the selection is valid
    verify_selection(self, selection)?;
    self.saved = false; // TODO: actually check if changes are made
    // Compile the regex used to match/extract data
    let regex = RegexBuilder::new(pattern.0)
      .multi_line(true)
      .build()
      .map_err(|_| INVALID_REGEX)
    ?;

    let mut selection_after = selection;
    // Cut out the whole selection from buffer
    let mut tail = self.buffer.split_off(selection.1 + 1);
    let before = self.buffer.split_off(selection.0 + 1);
    // Save ourselves a little bit of copying/allocating
    let mut tmp = self.buffer.pop().unwrap();
    // Then join all selected lines together
    for line in before {
      tmp.text.push_str(&line.text);
    }
    // Run the search-replace over it
    let mut after = if global {
      regex.replace_all(&tmp.text, pattern.1).to_string()
    }
    else {
      regex.replace(&tmp.text, pattern.1).to_string()
    };
    // If there is no newline at the end, join next line
    if !after.ends_with('\n') {
      if tail.len() > 0 {
        after.push_str(&tail.remove(0).text);
      }
      else {
        after.push('\n');
      }
    }
    // Split on newlines and add all lines to the buffer
    for line in after.lines() {
      self.buffer.push(Line{tag: '\0', matched: false, text: format!("{}\n", line)});
    }
    // Get the end of the affected area from current bufferlen
    selection_after.1 = self.buffer.len(); 
    // Then put the tail back
    self.buffer.append(&mut tail); 
    Ok(selection_after)
  }

  // File operations
  fn read_from(&mut self, path: &str, index: Option<usize>, must_exist: bool)
    -> Result<usize, &'static str>
  {
    if let Some(i) = index { verify_index(self, i)?; }
    let data = file::read_file(path, must_exist)?;
    let len = data.len();
    let mut iter = data.iter().map(| string | &string[..]);
    let i = match index {
      Some(i) => i,
      // Since .change is not safe on an empty selection and we actually just wish to delete everything
      None => {
        self.buffer.clear();
        0
      },
    };
    self.insert(&mut iter, i)?;
    Ok(len)
  }
  fn write_to(&mut self, selection: Option<(usize, usize)>, path: &str, append: bool)
    -> Result<(), &'static str>
  {
    let data = match selection {
      Some(sel) => self.get_selection(sel)?,
      None => Box::new(self.buffer[..].iter().map(|line| &line.text[..])),
    };
    file::write_file(path, data, append)?;
    if selection == Some((0, self.len().saturating_sub(1))) || selection.is_none() {
      self.saved = true;
    }
    Ok(())
  }
  fn saved(&self) -> bool {
    self.saved
  }

  // The output command
  fn get_selection<'a>(&'a self, selection: (usize, usize))
    -> Result<Box<dyn Iterator<Item = &'a str> + 'a>, &'static str>
  {
    verify_selection(self, selection)?;
    let tmp = self.buffer[selection.0 ..= selection.1].iter().map(|line| &line.text[..]);
    Ok(Box::new(tmp))
  }
}
