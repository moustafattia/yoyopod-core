#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modal {
    Loading { title: String, message: String },
    Error { title: String, message: String },
}
