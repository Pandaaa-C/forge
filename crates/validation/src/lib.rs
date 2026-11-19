#[derive(Debug, Clone, PartialEq)]
pub struct FieldError {
    pub field: &'static str,
    pub message: String,
}

pub trait Validate {
    fn validate(&self) -> Result<(), Vec<FieldError>>;
}
