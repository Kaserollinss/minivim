use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Change,
    Yank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    WordFwd,
    WordEnd,
    WordBack,
    LineStart,
    FirstNonBlank,
    LineEnd,
    FileStart,
    FileEnd,
    /// f/t/F/T — `till` stops before the target, `backward` searches left.
    Find {
        target: char,
        till: bool,
        backward: bool,
    },
}

/// How an operator consumes the span a motion produces. `dw` and `de` differ
/// only in this, and `dj` takes whole lines because `Down` is linewise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionKind {
    Exclusive,
    Inclusive,
    Linewise,
}

impl Motion {
    pub fn kind(self) -> MotionKind {
        match self {
            Motion::Up | Motion::Down | Motion::FileStart | Motion::FileEnd => {
                MotionKind::Linewise
            }
            Motion::WordEnd | Motion::LineEnd => MotionKind::Inclusive,
            Motion::Find { till: false, .. } => MotionKind::Inclusive,
            _ => MotionKind::Exclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Motion(Motion, usize),
    /// Doubled operator: `dd`, `yy`, `cc`.
    CurrentLine(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertKind {
    Before,
    After,
    LineStart,
    LineEnd,
    OpenBelow,
    OpenAbove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleAction {
    DeleteChar,
    PasteAfter,
    PasteBefore,
    JoinLines,
    Undo,
    Redo,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Move(Motion, usize),
    Operate { op: Operator, target: Target },
    EnterInsert(InsertKind),
    EnterVisual,
    EnterCommandLine,
    Simple(SimpleAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseResult {
    /// Grammatically incomplete — keep reading.
    Pending,
    Complete(Action),
    /// Bad sequence or <Esc>; state has been reset.
    Invalid,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum State {
    #[default]
    Start,
    /// Saw f/t/F/T, waiting for the character to search for.
    AwaitingFindChar { till: bool, backward: bool },
}

#[derive(Debug, Default)]
pub struct Parser {
    /// Count typed before the operator (the 2 in `2d3w`).
    count1: Option<usize>,
    /// Count typed after the operator (the 3 in `2d3w`).
    count2: Option<usize>,
    operator: Option<Operator>,
    state: State,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, key: KeyEvent) -> ParseResult {
        // Ctrl-chords never participate in the grammar.
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            self.reset();
            return match key.code {
                KeyCode::Char('q') => ParseResult::Complete(Action::Simple(SimpleAction::Quit)),
                KeyCode::Char('r') => ParseResult::Complete(Action::Simple(SimpleAction::Redo)),
                _ => ParseResult::Invalid,
            };
        }

        let c = match key.code {
            KeyCode::Char(c) => c,
            KeyCode::Esc => {
                self.reset();
                return ParseResult::Invalid;
            }
            KeyCode::Left => return self.motion(Motion::Left),
            KeyCode::Right => return self.motion(Motion::Right),
            KeyCode::Up => return self.motion(Motion::Up),
            KeyCode::Down => return self.motion(Motion::Down),
            _ => {
                self.reset();
                return ParseResult::Invalid;
            }
        };

        if let State::AwaitingFindChar { till, backward } = self.state {
            self.state = State::Start;
            return self.motion(Motion::Find {
                target: c,
                till,
                backward,
            });
        }

        // `0` is a motion when no count is being typed, and a digit when one is.
        if c.is_ascii_digit() && !(c == '0' && self.current_count().is_none()) {
            self.push_digit(c);
            return ParseResult::Pending;
        }

        // A repeated operator means "current line, linewise".
        if let (Some(pending), Some(op)) = (self.operator, operator_for(c))
            && pending == op {
                let count = self.effective_count();
                self.reset();
                return ParseResult::Complete(Action::Operate {
                    op: pending,
                    target: Target::CurrentLine(count),
                });
            }

        if let Some(m) = motion_for(c) {
            return self.motion(m);
        }

        match c {
            'f' | 't' | 'F' | 'T' => {
                self.state = State::AwaitingFindChar {
                    till: c.eq_ignore_ascii_case(&'t'),
                    backward: c.is_ascii_uppercase(),
                };
                ParseResult::Pending
            }
            _ if self.operator.is_some() => {
                // An operator is pending and this key is not a valid range.
                self.reset();
                ParseResult::Invalid
            }
            _ => {
                if let Some(op) = operator_for(c) {
                    self.operator = Some(op);
                    return ParseResult::Pending;
                }
                let action = match c {
                    'i' => Action::EnterInsert(InsertKind::Before),
                    'a' => Action::EnterInsert(InsertKind::After),
                    'I' => Action::EnterInsert(InsertKind::LineStart),
                    'A' => Action::EnterInsert(InsertKind::LineEnd),
                    'o' => Action::EnterInsert(InsertKind::OpenBelow),
                    'O' => Action::EnterInsert(InsertKind::OpenAbove),
                    'v' => Action::EnterVisual,
                    ':' => Action::EnterCommandLine,
                    'x' => Action::Simple(SimpleAction::DeleteChar),
                    'p' => Action::Simple(SimpleAction::PasteAfter),
                    'P' => Action::Simple(SimpleAction::PasteBefore),
                    'J' => Action::Simple(SimpleAction::JoinLines),
                    'u' => Action::Simple(SimpleAction::Undo),
                    _ => {
                        self.reset();
                        return ParseResult::Invalid;
                    }
                };
                self.reset();
                ParseResult::Complete(action)
            }
        }
    }

    /// Resolve a motion: it either completes a pending operator or moves the cursor.
    fn motion(&mut self, m: Motion) -> ParseResult {
        let count = self.effective_count();
        let op = self.operator;
        self.reset();
        match op {
            Some(op) => ParseResult::Complete(Action::Operate {
                op,
                target: Target::Motion(m, count),
            }),
            None => ParseResult::Complete(Action::Move(m, count)),
        }
    }

    /// Counts multiply: `2d3w` acts on 6 words, not 2 or 3.
    fn effective_count(&self) -> usize {
        self.count1.unwrap_or(1) * self.count2.unwrap_or(1)
    }

    /// Digits land in the second slot once an operator has been seen.
    fn current_count(&self) -> Option<usize> {
        if self.operator.is_some() {
            self.count2
        } else {
            self.count1
        }
    }

    fn push_digit(&mut self, c: char) {
        let d = c.to_digit(10).unwrap_or(0) as usize;
        let slot = if self.operator.is_some() {
            &mut self.count2
        } else {
            &mut self.count1
        };
        *slot = Some(slot.unwrap_or(0).saturating_mul(10).saturating_add(d));
    }

    fn reset(&mut self) {
        self.count1 = None;
        self.count2 = None;
        self.operator = None;
        self.state = State::Start;
    }
}

fn operator_for(c: char) -> Option<Operator> {
    match c {
        'd' => Some(Operator::Delete),
        'c' => Some(Operator::Change),
        'y' => Some(Operator::Yank),
        _ => None,
    }
}

fn motion_for(c: char) -> Option<Motion> {
    match c {
        'h' => Some(Motion::Left),
        'l' => Some(Motion::Right),
        'k' => Some(Motion::Up),
        'j' => Some(Motion::Down),
        'w' => Some(Motion::WordFwd),
        'e' => Some(Motion::WordEnd),
        'b' => Some(Motion::WordBack),
        '0' => Some(Motion::LineStart),
        '^' => Some(Motion::FirstNonBlank),
        '$' => Some(Motion::LineEnd),
        'G' => Some(Motion::FileEnd),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(p: &mut Parser, keys: &str) -> ParseResult {
        let mut result = ParseResult::Pending;
        for c in keys.chars() {
            result = p.feed(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        result
    }

    #[test]
    fn bare_motion_has_count_one() {
        let mut p = Parser::new();
        assert_eq!(press(&mut p, "w"), ParseResult::Complete(Action::Move(Motion::WordFwd, 1)));
    }

    #[test]
    fn counted_motion() {
        let mut p = Parser::new();
        assert_eq!(press(&mut p, "12j"), ParseResult::Complete(Action::Move(Motion::Down, 12)));
    }

    #[test]
    fn zero_is_a_motion_without_a_count() {
        let mut p = Parser::new();
        assert_eq!(press(&mut p, "0"), ParseResult::Complete(Action::Move(Motion::LineStart, 1)));
    }

    #[test]
    fn zero_is_a_digit_within_a_count() {
        let mut p = Parser::new();
        assert_eq!(press(&mut p, "10k"), ParseResult::Complete(Action::Move(Motion::Up, 10)));
    }

    #[test]
    fn operator_alone_is_pending() {
        let mut p = Parser::new();
        assert_eq!(press(&mut p, "d"), ParseResult::Pending);
    }

    #[test]
    fn operator_with_motion() {
        let mut p = Parser::new();
        assert_eq!(
            press(&mut p, "dw"),
            ParseResult::Complete(Action::Operate {
                op: Operator::Delete,
                target: Target::Motion(Motion::WordFwd, 1),
            })
        );
    }

    #[test]
    fn counts_multiply() {
        let mut p = Parser::new();
        assert_eq!(
            press(&mut p, "2d3w"),
            ParseResult::Complete(Action::Operate {
                op: Operator::Delete,
                target: Target::Motion(Motion::WordFwd, 6),
            })
        );
    }

    #[test]
    fn doubled_operator_is_linewise() {
        let mut p = Parser::new();
        assert_eq!(
            press(&mut p, "3dd"),
            ParseResult::Complete(Action::Operate {
                op: Operator::Delete,
                target: Target::CurrentLine(3),
            })
        );
    }

    #[test]
    fn change_does_not_enter_insert_while_pending() {
        // The original bug: `c` must not be dispatched as a mode change.
        let mut p = Parser::new();
        assert_eq!(press(&mut p, "c"), ParseResult::Pending);
    }

    #[test]
    fn find_waits_for_its_target() {
        let mut p = Parser::new();
        assert_eq!(press(&mut p, "f"), ParseResult::Pending);
        assert_eq!(
            press(&mut p, "x"),
            ParseResult::Complete(Action::Move(
                Motion::Find { target: 'x', till: false, backward: false },
                1
            ))
        );
    }

    #[test]
    fn escape_cancels_a_pending_operator() {
        let mut p = Parser::new();
        press(&mut p, "2d");
        p.feed(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(press(&mut p, "w"), ParseResult::Complete(Action::Move(Motion::WordFwd, 1)));
    }
}
