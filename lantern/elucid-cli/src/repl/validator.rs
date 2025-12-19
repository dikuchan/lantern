use reedline::{ValidationResult, Validator};

pub struct QueryValidator;

impl Validator for QueryValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        let line = line.trim();

        if line.ends_with('|') || line.ends_with(',') {
            return ValidationResult::Incomplete;
        }

        // TODO: Validate brackets.

        ValidationResult::Complete
    }
}
