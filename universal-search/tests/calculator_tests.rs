use rusty_universal_search::calculator::{
    calculate_with_backend, launch_calculator_with_backend, CalculatorBackend,
};
use std::collections::HashMap;

/// Mock calculator backend for testing
struct MockCalculatorBackend {
    results: HashMap<String, String>,
    launch_calls: std::cell::RefCell<Vec<String>>,
}

impl MockCalculatorBackend {
    fn new() -> Self {
        let mut results = HashMap::new();

        // Define expected results for test expressions
        results.insert("2+2".to_string(), "4".to_string());
        results.insert("10-5".to_string(), "5".to_string());
        results.insert("3*4".to_string(), "12".to_string());
        results.insert("15/3".to_string(), "5".to_string());
        results.insert("(5+3)*2".to_string(), "16".to_string());
        results.insert("100/(4+1)".to_string(), "20".to_string());
        results.insert("2^3".to_string(), "8".to_string());
        results.insert("sqrt(16)".to_string(), "4".to_string());
        results.insert("3.14*2".to_string(), "6.28".to_string());
        results.insert("10.5+2.5".to_string(), "13".to_string());
        results.insert("7.5/2.5".to_string(), "3".to_string());
        results.insert("1+2+3+4+5+6+7+8+9+10".to_string(), "55".to_string());
        results.insert("2 + 2".to_string(), "4".to_string());
        results.insert("10 * 5".to_string(), "50".to_string());
        results.insert("100000+1".to_string(), "100001".to_string());
        results.insert("2500*2".to_string(), "5000".to_string());
        results.insert("15*8".to_string(), "120".to_string());
        results.insert("144/12".to_string(), "12".to_string());
        results.insert("25+37".to_string(), "62".to_string());
        results.insert("100-23".to_string(), "77".to_string());
        results.insert("(12+8)*5".to_string(), "100".to_string());
        results.insert("2^8".to_string(), "256".to_string());
        results.insert("1+1".to_string(), "2".to_string());
        results.insert("10*5".to_string(), "50".to_string());

        Self {
            results,
            launch_calls: std::cell::RefCell::new(Vec::new()),
        }
    }

    fn get_launch_calls(&self) -> Vec<String> {
        self.launch_calls.borrow().clone()
    }
}

impl CalculatorBackend for MockCalculatorBackend {
    fn calculate(&self, expression: &str) -> Option<String> {
        self.results.get(expression).cloned()
    }

    fn launch(&self, expression: &str) -> Result<(), std::io::Error> {
        self.launch_calls.borrow_mut().push(expression.to_string());
        Ok(())
    }
}

#[test]
fn test_basic_arithmetic() {
    let mock = MockCalculatorBackend::new();

    // Basic operations
    assert!(calculate_with_backend(&mock, "2+2").is_some());
    assert!(calculate_with_backend(&mock, "10-5").is_some());
    assert!(calculate_with_backend(&mock, "3*4").is_some());
    assert!(calculate_with_backend(&mock, "15/3").is_some());

    println!("Basic arithmetic tests completed");
}

#[test]
fn test_complex_expressions() {
    let mock = MockCalculatorBackend::new();

    // Complex expressions with parentheses
    assert!(calculate_with_backend(&mock, "(5+3)*2").is_some());
    assert!(calculate_with_backend(&mock, "100/(4+1)").is_some());
    assert!(calculate_with_backend(&mock, "2^3").is_some());
    assert!(calculate_with_backend(&mock, "sqrt(16)").is_some());

    println!("Complex expression tests completed");
}

#[test]
fn test_decimal_numbers() {
    let mock = MockCalculatorBackend::new();

    // Decimal operations
    assert!(calculate_with_backend(&mock, "3.14*2").is_some());
    assert!(calculate_with_backend(&mock, "10.5+2.5").is_some());
    assert!(calculate_with_backend(&mock, "7.5/2.5").is_some());

    println!("Decimal number tests completed");
}

#[test]
fn test_invalid_expressions() {
    let mock = MockCalculatorBackend::new();

    // Should return None for invalid expressions
    assert!(calculate_with_backend(&mock, "").is_none());
    assert!(calculate_with_backend(&mock, "ab").is_none());
    assert!(calculate_with_backend(&mock, "hello").is_none());
    assert!(calculate_with_backend(&mock, "test").is_none());
    assert!(calculate_with_backend(&mock, "++").is_none());
    assert!(calculate_with_backend(&mock, "123abc").is_none());

    // Too short expressions
    assert!(calculate_with_backend(&mock, "2").is_none());
    assert!(calculate_with_backend(&mock, "+").is_none());

    println!("Invalid expression tests completed");
}

#[test]
fn test_edge_cases() {
    let mock = MockCalculatorBackend::new();

    // Very long expressions
    let long_expr = "1+2+3+4+5+6+7+8+9+10";
    assert!(calculate_with_backend(&mock, long_expr).is_some());

    // Expressions with spaces
    assert!(calculate_with_backend(&mock, "2 + 2").is_some());
    assert!(calculate_with_backend(&mock, "10 * 5").is_some());

    println!("Edge case tests completed");
}

#[test]
fn test_large_numbers() {
    let mock = MockCalculatorBackend::new();

    // Test calculations with large numbers (gnome-calculator doesn't support 1e5 notation)
    assert!(calculate_with_backend(&mock, "100000+1").is_some()); // Large number calculation
    assert!(calculate_with_backend(&mock, "2500*2").is_some()); // Large number multiplication

    println!("Large number calculation tests completed");
}

#[test]
fn test_calculator_launch() {
    let mock = MockCalculatorBackend::new();

    // Test that launch_calculator doesn't panic
    let result = launch_calculator_with_backend(&mock, "2+2");
    assert!(result.is_ok());

    // Verify the launch was recorded
    let calls = mock.get_launch_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], "2+2");

    println!("Calculator launch test completed");
}

#[test]
fn test_realistic_calculations() {
    let mock = MockCalculatorBackend::new();

    // Test some realistic calculations that users might perform
    let expressions = [
        "15*8",     // Multiplication
        "144/12",   // Division
        "25+37",    // Addition
        "100-23",   // Subtraction
        "(12+8)*5", // Parentheses
        "2^8",      // Exponentiation
    ];

    for expr in &expressions {
        let result = calculate_with_backend(&mock, expr);
        assert!(result.is_some(), "Failed to calculate: {}", expr);
        if let Some(res) = result {
            println!("{} = {}", expr, res);
        }
    }

    println!("Realistic calculation tests completed");
}

#[test]
fn test_math_detection() {
    let mock = MockCalculatorBackend::new();

    // Test the internal math detection logic indirectly
    // by verifying that obviously non-math strings return None

    let non_math = [
        "firefox",
        "calculator",
        "search term",
        "file.txt",
        "hello world",
        "123", // Just numbers without operators
    ];

    for expr in &non_math {
        let result = calculate_with_backend(&mock, expr);
        // These should all be None because they don't look like math
        assert!(result.is_none(), "Should not calculate non-math: {}", expr);
    }

    println!("Math detection tests completed");
}

#[test]
fn test_minimum_length_requirement() {
    let mock = MockCalculatorBackend::new();

    // Test that expressions under 3 characters are ignored
    assert!(calculate_with_backend(&mock, "1").is_none());
    assert!(calculate_with_backend(&mock, "12").is_none());
    assert!(calculate_with_backend(&mock, "+").is_none());
    assert!(calculate_with_backend(&mock, "++").is_none());

    // But 3+ character math expressions should work
    assert!(calculate_with_backend(&mock, "1+1").is_some());

    println!("Minimum length requirement tests completed");
}

#[test]
fn test_integration_with_search_flow() {
    let mock = MockCalculatorBackend::new();

    // Test the integration pattern that would be used in the main search flow
    let test_queries = [
        ("2+2", true),      // Should calculate
        ("hello", false),   // Should not calculate
        ("firefox", false), // Should not calculate
        ("10*5", true),     // Should calculate
        ("ab", false),      // Should not calculate (too short)
        ("(5+3)*2", true),  // Should calculate
    ];

    for (query, should_calculate) in &test_queries {
        let result = calculate_with_backend(&mock, query);
        if *should_calculate {
            assert!(result.is_some(), "Should calculate: {}", query);
            if let Some(res) = result {
                println!("Query '{}' calculated as: {}", query, res);
            }
        } else {
            assert!(result.is_none(), "Should not calculate: {}", query);
            println!("Query '{}' correctly not calculated", query);
        }
    }

    println!("Integration flow tests completed");
}
