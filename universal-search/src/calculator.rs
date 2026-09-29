use std::process::Command;

/// Trait for calculator backend to allow mocking in tests
pub trait CalculatorBackend {
    fn calculate(&self, expression: &str) -> Option<String>;
    fn launch(&self, expression: &str) -> Result<(), std::io::Error>;
}

/// Real gnome-calculator backend
pub struct GnomeCalculatorBackend;

impl CalculatorBackend for GnomeCalculatorBackend {
    fn calculate(&self, expression: &str) -> Option<String> {
        let output = Command::new("gnome-calculator")
            .arg("-s")
            .arg(expression)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let result = String::from_utf8_lossy(&output.stdout);
                    let result = result.trim();

                    // Return the result if it's not empty and not an error
                    if !result.is_empty() && !result.contains("Error") {
                        Some(result.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn launch(&self, expression: &str) -> Result<(), std::io::Error> {
        Command::new("gnome-calculator")
            .arg("-e")
            .arg(expression)
            .spawn()?;

        Ok(())
    }
}

/// Default calculator instance using gnome-calculator
static DEFAULT_BACKEND: GnomeCalculatorBackend = GnomeCalculatorBackend;

/// Calculate a mathematical expression using the default backend
/// Returns Some(result) if the expression can be evaluated, None otherwise
pub fn calculate(expression: &str) -> Option<String> {
    calculate_with_backend(&DEFAULT_BACKEND, expression)
}

/// Launch calculator with the default backend
pub fn launch_calculator(expression: &str) -> Result<(), std::io::Error> {
    launch_calculator_with_backend(&DEFAULT_BACKEND, expression)
}

/// Calculate using a specific backend (for testing)
pub fn calculate_with_backend(backend: &dyn CalculatorBackend, expression: &str) -> Option<String> {
    // Only try to evaluate if the expression is at least 3 characters
    // and contains mathematical operators or numbers
    if expression.len() < 3 || !looks_like_math(expression) {
        return None;
    }

    backend.calculate(expression)
}

/// Launch calculator using a specific backend (for testing)
pub fn launch_calculator_with_backend(
    backend: &dyn CalculatorBackend,
    expression: &str,
) -> Result<(), std::io::Error> {
    backend.launch(expression)
}

/// Check if a string looks like it could be a mathematical expression
fn looks_like_math(text: &str) -> bool {
    // Contains mathematical operators or patterns
    let math_chars = ['+', '-', '*', '/', '(', ')', '^', '%', '.'];

    // Check for mathematical functions
    let math_functions = ["sqrt", "sin", "cos", "tan", "log", "ln", "exp"];
    let has_function = math_functions.iter().any(|&func| text.contains(func));

    // Must contain at least one math operator or be a function call
    let has_operator = text.chars().any(|c| math_chars.contains(&c));

    // Must contain at least one digit
    let has_digit = text.chars().any(|c| c.is_ascii_digit());

    // Should not be just a number or just operators
    let is_just_number = text.chars().all(|c| c.is_ascii_digit() || c == '.');
    let is_just_operators = text.chars().all(|c| math_chars.contains(&c));

    // Basic check: avoid common non-math words
    let common_words = ["hello", "world", "test", "file", "app", "search"];
    let is_common_word = common_words
        .iter()
        .any(|&word| text.to_lowercase().contains(word));

    (has_operator || has_function)
        && has_digit
        && !is_just_number
        && !is_just_operators
        && !is_common_word
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_math() {
        // Valid math expressions
        assert!(looks_like_math("2+2"));
        assert!(looks_like_math("10*5"));
        assert!(looks_like_math("(5+3)/2"));
        assert!(looks_like_math("3.14*2"));
        assert!(looks_like_math("100-25"));
        assert!(looks_like_math("2^8"));
        assert!(looks_like_math("sqrt(16)"));

        // Invalid expressions
        assert!(!looks_like_math("hello"));
        assert!(!looks_like_math("test"));
        assert!(!looks_like_math("abc"));
        assert!(!looks_like_math("++"));
        assert!(!looks_like_math("123")); // Just numbers without operators
        assert!(!looks_like_math("+-")); // Just operators without numbers
    }
}
