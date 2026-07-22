#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextRange {
    pub start: Position,
    pub end: Position,
}

impl TextRange {
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub const fn empty(at: Position) -> Self {
        Self::new(at, at)
    }

    pub fn normalized(self) -> Self {
        if self.start <= self.end {
            self
        } else {
            Self::new(self.end, self.start)
        }
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CursorState {
    pub position: Position,
    pub preferred_column: usize,
    pub selection_anchor: Option<Position>,
}

impl CursorState {
    pub const fn collapsed(position: Position) -> Self {
        Self {
            position,
            preferred_column: position.column,
            selection_anchor: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cursor {
    state: CursorState,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

impl Cursor {
    pub const fn new() -> Self {
        Self {
            state: CursorState::collapsed(Position::new(0, 0)),
        }
    }

    pub const fn position(&self) -> Position {
        self.state.position
    }

    pub const fn preferred_column(&self) -> usize {
        self.state.preferred_column
    }

    pub fn selection(&self) -> Option<TextRange> {
        self.state
            .selection_anchor
            .map(|anchor| TextRange::new(anchor, self.state.position).normalized())
            .filter(|range| !range.is_empty())
    }

    pub const fn snapshot(&self) -> CursorState {
        self.state
    }

    pub fn restore(&mut self, state: CursorState) {
        self.state = state;
    }

    pub fn start_selection(&mut self) {
        self.state
            .selection_anchor
            .get_or_insert(self.state.position);
    }

    pub fn clear_selection(&mut self) {
        self.state.selection_anchor = None;
    }

    pub fn set_position(&mut self, position: Position, update_preferred_column: bool) {
        self.state.position = position;
        if update_preferred_column {
            self.state.preferred_column = position.column;
        }
    }

    pub fn collapse_to(&mut self, position: Position) {
        self.state = CursorState::collapsed(position);
    }
}
