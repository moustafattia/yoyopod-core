#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cursor {
    UnderlineDots { count: usize, focus: usize },
    RowGlow,
}
