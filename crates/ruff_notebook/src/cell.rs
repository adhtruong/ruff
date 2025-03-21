use std::fmt;
use std::ops::{Deref, DerefMut};

use itertools::Itertools;

use ruff_text_size::{TextRange, TextSize};

use crate::schema::{Cell, SourceValue};

impl fmt::Display for SourceValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceValue::String(string) => f.write_str(string),
            SourceValue::StringArray(string_array) => {
                for string in string_array {
                    f.write_str(string)?;
                }
                Ok(())
            }
        }
    }
}

impl Cell {
    /// Return the [`SourceValue`] of the cell.
    pub(crate) fn source(&self) -> &SourceValue {
        match self {
            Cell::Code(cell) => &cell.source,
            Cell::Markdown(cell) => &cell.source,
            Cell::Raw(cell) => &cell.source,
        }
    }

    /// Update the [`SourceValue`] of the cell.
    pub(crate) fn set_source(&mut self, source: SourceValue) {
        match self {
            Cell::Code(cell) => cell.source = source,
            Cell::Markdown(cell) => cell.source = source,
            Cell::Raw(cell) => cell.source = source,
        }
    }

    /// Return `true` if it's a valid code cell.
    ///
    /// A valid code cell is a cell where the cell type is [`Cell::Code`] and the
    /// source doesn't contain a cell magic.
    pub(crate) fn is_valid_code_cell(&self) -> bool {
        let source = match self {
            Cell::Code(cell) => &cell.source,
            _ => return false,
        };
        // Ignore cells containing cell magic as they act on the entire cell
        // as compared to line magic which acts on a single line.
        !match source {
            SourceValue::String(string) => Self::is_magic_cell(string.lines()),
            SourceValue::StringArray(string_array) => {
                Self::is_magic_cell(string_array.iter().map(String::as_str))
            }
        }
    }

    /// Returns `true` if a cell should be ignored due to the use of cell magics.
    fn is_magic_cell<'a>(lines: impl Iterator<Item = &'a str>) -> bool {
        let mut lines = lines.peekable();

        // Detect automatic line magics (automagic), which aren't supported by the parser. If a line
        // magic uses automagic, Jupyter doesn't allow following it with non-magic lines anyway, so
        // we aren't missing out on any valid Python code.
        //
        // For example, this is valid:
        // ```jupyter
        // cat /path/to/file
        // cat /path/to/file
        // ```
        //
        // But this is invalid:
        // ```jupyter
        // cat /path/to/file
        // x = 1
        // ```
        //
        // See: https://ipython.readthedocs.io/en/stable/interactive/magics.html
        if lines
            .peek()
            .and_then(|line| line.split_whitespace().next())
            .is_some_and(|token| {
                matches!(
                    token,
                    "alias"
                        | "alias_magic"
                        | "autoawait"
                        | "autocall"
                        | "automagic"
                        | "bookmark"
                        | "cd"
                        | "code_wrap"
                        | "colors"
                        | "conda"
                        | "config"
                        | "debug"
                        | "dhist"
                        | "dirs"
                        | "doctest_mode"
                        | "edit"
                        | "env"
                        | "gui"
                        | "history"
                        | "killbgscripts"
                        | "load"
                        | "load_ext"
                        | "loadpy"
                        | "logoff"
                        | "logon"
                        | "logstart"
                        | "logstate"
                        | "logstop"
                        | "lsmagic"
                        | "macro"
                        | "magic"
                        | "mamba"
                        | "matplotlib"
                        | "micromamba"
                        | "notebook"
                        | "page"
                        | "pastebin"
                        | "pdb"
                        | "pdef"
                        | "pdoc"
                        | "pfile"
                        | "pinfo"
                        | "pinfo2"
                        | "pip"
                        | "popd"
                        | "pprint"
                        | "precision"
                        | "prun"
                        | "psearch"
                        | "psource"
                        | "pushd"
                        | "pwd"
                        | "pycat"
                        | "pylab"
                        | "quickref"
                        | "recall"
                        | "rehashx"
                        | "reload_ext"
                        | "rerun"
                        | "reset"
                        | "reset_selective"
                        | "run"
                        | "save"
                        | "sc"
                        | "set_env"
                        | "sx"
                        | "system"
                        | "tb"
                        | "time"
                        | "timeit"
                        | "unalias"
                        | "unload_ext"
                        | "who"
                        | "who_ls"
                        | "whos"
                        | "xdel"
                        | "xmode"
                )
            })
        {
            return true;
        }

        // Detect cell magics (which operate on multiple lines).
        lines.any(|line| line.trim_start().starts_with("%%"))
    }
}

/// Cell offsets are used to keep track of the start and end offsets of each
/// cell in the concatenated source code.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CellOffsets(Vec<TextSize>);

impl CellOffsets {
    /// Create a new [`CellOffsets`] with the given capacity.
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Push a new offset to the end of the [`CellOffsets`].
    pub(crate) fn push(&mut self, offset: TextSize) {
        self.0.push(offset);
    }

    /// Returns the range of the cell containing the given offset, if any.
    pub fn containing_range(&self, offset: TextSize) -> Option<TextRange> {
        self.iter().tuple_windows().find_map(|(start, end)| {
            if *start <= offset && offset < *end {
                Some(TextRange::new(*start, *end))
            } else {
                None
            }
        })
    }
}

impl Deref for CellOffsets {
    type Target = [TextSize];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CellOffsets {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
