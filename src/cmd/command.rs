use crate::Buffer;
use crate::State;

// A command abstraction
#[derive(Debug)]
pub enum Cmd<'a> {
    Quit(&'a mut State), // Quit
    ForceQuit(&'a mut State), // Quit ignoring unsaved changes
    Perror(&'a mut State), // Print last error
    SetPerror(&'a mut State), // Toggle printing errors as they occur
    // Undo(&'a mut State), // Revert buffer one command back (including Undo)

    Print(Print<'a>), // Print the defined lines in defined mode

    Insert(Insert<'a>), // Insert given text at given index
    Delete(Delete<'a>), // Delete the selection
    Change(Change<'a>), // Replace selection with given text

    Move(Move<'a>), // Moves selection to given index
    Copy(Copy<'a>), // Copy the selection to given index
    Join(Join<'a>), // Join the selection into one line

    // Cut(Cut<'a>), // Cut selection from buffer
    // Paste(Paste<'a>), // Append what was last cut to selection

    // Global(Global<'a>), // Apply the given commands to lines matching expr.
    Substitute(Substitute<'a>), // Regex search and replace over selection

    // ForceOpen(Open<'a>), // Open given file into new buffer ignoring unsaved
    Open(Open<'a>), // Open given file into new buffer
    Read(Read<'a>), // Append contents of given file to selection
    Write(Write<'a>), // Write the selection to given file
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Print<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub selection: (usize, usize),
    // Flags
    pub n: bool,
    pub h: bool,
    pub l: bool,
}
impl <'a> Print<'a> {
    // A method to create a print command after another command
    // To handle the p, n, l flags in non-print commands
    fn from_state(
        state: &'a mut State,
        n: bool,
        h: bool,
        l: bool,
    ) -> Self {
        Self {
            selection: state.selection
                .unwrap_or_else(||(0 as usize, state.buffer.len())),
            state: state,
            n: n,
            h: h,
            l: l
        }
    }
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Insert<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub index: usize,
    pub input: Vec<String>,
    // Flags
    pub p: bool,
    pub n: bool,
    pub h: bool,
    pub l: bool,
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Delete<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub selection: (usize, usize),
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Change<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub selection: (usize, usize),
    pub input: Vec<String>,
    // Flags
    pub p: bool,
    pub n: bool,
    pub h: bool,
    pub l: bool,
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Move<'a> {
  #[derivative(Debug="ignore")]
  pub state: &'a mut State,
  pub selection: (usize, usize),
  pub index: usize,
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Copy<'a> {
  #[derivative(Debug="ignore")]
  pub state: &'a mut State,
  pub selection: (usize, usize),
  pub index: usize,
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Join<'a> {
  #[derivative(Debug="ignore")]
  pub state: &'a mut State,
  pub selection: (usize, usize),
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Substitute<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub selection: (usize, usize),
    pub expression: (&'a str, &'a str),
    // Flags
    pub g: bool,
    pub p: bool,
    pub n: bool,
    pub h: bool,
    pub l: bool,
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Read<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub index: usize,
    pub path: &'a str,
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Open<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub path: &'a str,
}
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Write<'a> {
    #[derivative(Debug="ignore")]
    pub state: &'a mut State,
    pub selection: (usize, usize),
    pub path: &'a str,
}
impl <'a> Cmd<'a> {
    /// A simple debug printout
    pub fn _debug_print(&self) {
        println!("{:?}", self);
    }
    /// Execute the command with stored arguments
    /// Returns the selection after command execution or error msg
    pub fn execute(self) -> Result<(), &'static str> {
        match self {
            Self::Quit(state) => {
                if state.buffer.saved() {
                    state.done = true;
                    Ok(())
                }
                else {
                    Err("Unsaved changes!")
                }
            },
            Self::ForceQuit(state) => { state.done = true; Ok(()) },
            Self::Perror(state) => { println!("{:?}", state.error); Ok(()) },
            Self::SetPerror(state) => {
                state.print_errors = ! state.print_errors;
                Ok(())
            },

            //Self::Undo => { state.done = true; Ok(()) },
            Self::Print(print) => {
                // Format from the buffer
                let output = print.state.buffer.get_selection(
                    print.selection,
                )?;
                // Send it to the printer
                crate::io::format_print(
                    &print.state.syntax_lib,
                    &print.state.theme_lib.themes["base16-ocean.dark"],
                    print.state.file.as_ref().unwrap_or(&"".to_string()),
                    output,
                    print.selection.0,
                    print.n,
                    print.h,
                    print.l,
                );
                // And update the selection
                print.state.selection = Some(print.selection);
                Ok(())
            },
            Self::Insert(mut insert) => {
                // Calculate the start and end of the inserted text
                let mut end = insert.index + insert.input.len();
                let mut start = insert.index;
                // 0 is allowed and needs special handling
                if insert.index != 0 { start -= 1; end -= 1; }
                // Perform the insert
                insert.state.buffer.insert(
                    &mut insert.input,
                    insert.index,
                )?;
                // Update the selection, shift -1 for insert specific weird
                insert.state.selection = Some((start, end));
                // Print if requested
                if insert.p || insert.n || insert.l {
                    Self::Print(Print::from_state(
                        insert.state,
                        insert.n,
                        insert.h,
                        insert.l,
                    )).execute()?;
                }
                Ok(())
            },
            Self::Delete(delete) => {
                delete.state.buffer.delete(delete.selection)?;
                delete.state.selection = None;
                Ok(())
            },
            Self::Change(mut change) => {
                // Calculate the start and end of the changeed text
                let end = change.selection.0 + change.input.len();
                let start = change.selection.0;
                change.state.buffer.change(
                    &mut change.input,
                    change.selection
                )?;
                // Update the selection
                change.state.selection = Some((start, end));
                // Print if requested
                if change.p || change.n || change.l {
                    Self::Print(Print::from_state(
                        change.state,
                        change.n,
                        change.h,
                        change.l,
                    )).execute()?;
                }
                Ok(())
            },
            Self::Move(mut mov) => {
      let new_selection = (mov.index, mov.index + mov.selection.1 - mov.selection.0);
              mov.state.buffer.mov(mov.selection, mov.index)?;
              mov.state.selection = Some(new_selection);
              Ok(())
            },  
        Self::Copy(mut copy) => {
        let new_selection = (copy.index, copy.index + copy.selection.1 - copy.selection.0);
              copy.state.buffer.copy(copy.selection, copy.index)?;
              copy.state.selection = Some(new_selection);
              Ok(())
            },
      Self::Join(mut join) => {
        let new_selection = (join.selection.0, join.selection.0 + 1);
        join.state.buffer.join(join.selection)?;
        join.state.selection = Some(new_selection);
        Ok(())
      },
            Self::Substitute(mut substitute) => {
                let new_selection =
                    substitute.state.buffer.search_replace(
                        substitute.expression,
                        substitute.selection,
                        substitute.g
                    )?;
                substitute.state.selection = Some(new_selection);
                // TODO: Add print option to substitute
                Ok(())
            },
            Self::Open(mut open) => {
                let path = match open.path {
                    "" => return Err("No file specified"),
                    x => x,
                };
                let mut data = crate::file::read_file(path)?;
                let datalen = data.len();
                open.state.buffer = crate::buffer::VecBuffer::new();
                open.state.buffer.insert(&mut data, 0)?;
                open.state.file = Some(path.to_string());
                open.state.selection = Some((0, datalen));
                Ok(())
            }
            Self::Read(mut read) => {
                let path = match read.path {
                    "" => match &read.state.file {
                        Some(path) => path,
                        None => return Err("No file specified"),
                    },
                    x => x,
                };
                let mut data = crate::file::read_file(path)?;
                let end = read.index + data.len();
                read.state.buffer.insert(&mut data, read.index)?;
                read.state.selection = Some((read.index, end));
                Ok(())
            }
            Self::Write(mut write) => {
                let path = match write.path {
                    "" => match &write.state.file {
                        Some(path) => path,
                        None => return Err("No file specified"),
                    },
                    x => x,
                };
                let data = write.state.buffer.get_selection(write.selection)?;
                crate::file::write_file(path, data)?;
                if write.selection == (0, write.state.buffer.len()) {
                    write.state.buffer.set_saved();
                }
                write.state.selection = Some(write.selection);
                Ok(())
            }
        }
    }
}
