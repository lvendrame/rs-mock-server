use serde_json::{Map, Value};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Comparer {
    #[default]
    Equal,
    Different,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
    IsNull,
    IsNotNull,
}

impl TryFrom<&str> for Comparer {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "=" => Ok(Comparer::Equal),
            "!=" | "<>" => Ok(Comparer::Different),
            ">" => Ok(Comparer::GreaterThan),
            ">=" => Ok(Comparer::GreaterThanOrEqual),
            "<" => Ok(Comparer::LessThan),
            "<=" => Ok(Comparer::LessThanOrEqual),
            "LIKE" | "like" => Ok(Comparer::Like),
            "IS NULL" | "is null" => Ok(Comparer::IsNull),
            "IS NOT NULL" | "is not null" => Ok(Comparer::IsNotNull),
            _ => Err(format!("Invalid comparer operator: '{}'", value)),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Criteria {
    pub field: String,
    pub comparer: Comparer,
    pub value: Value
}

impl TryFrom<&str> for Criteria {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<_> = value.split(" ").collect();
        if parts.len() < 2 || parts.len() > 3 {
            return Err(format!("Invalid Criteria: '{}'", value));
        }
        let field = parts[0].to_string();
        let comparer = Comparer::try_from(value)?;
        let value = if parts.len() > 2 {
            Some(Value::from(parts[2]))
        } else {
            None
        };

        Criteria::try_new(field, comparer, value)
    }
}

impl Criteria {

    pub fn try_new(field: String, comparer: Comparer, value: Option<Value>) -> Result<Self, String> {
        let is_valid = match comparer {
            Comparer::Equal => validate_for_equal(&value),
            Comparer::Different => validate_for_different(&value),
            Comparer::GreaterThan => validate_for_greater_or_less_than(&value),
            Comparer::GreaterThanOrEqual => validate_for_greater_or_less_than(&value),
            Comparer::LessThan => validate_for_greater_or_less_than(&value),
            Comparer::LessThanOrEqual => validate_for_greater_or_less_than(&value),
            Comparer::Like => validate_for_like(&value),
            Comparer::IsNull => value.is_none(),
            Comparer::IsNotNull => value.is_none(),
        };

        if !is_valid {
            return Err(format!("The value {:?} is not allowed for the comparer {:?}", value, comparer));
        }

        Ok(Self{
            field,
            comparer,
            value: value.unwrap_or(Value::Null)
        })
    }

    pub fn compare_item(&self, item: &Map<String, Value>) -> bool {
        match item.get(&self.field) {
            Some(value) => self.compare_with(value),
            None => false,
        }
    }

    pub fn compare_with(&self, value: &Value) -> bool {
        match self.comparer {
            Comparer::Equal => self.value == *value,
            Comparer::Different => self.value != *value,
            Comparer::GreaterThan => compare_numbers(value, &self.value, |a, b| a > b),
            Comparer::GreaterThanOrEqual => compare_numbers(value, &self.value, |a, b| a >= b),
            Comparer::LessThan => compare_numbers(value, &self.value, |a, b| a < b),
            Comparer::LessThanOrEqual => compare_numbers(value, &self.value, |a, b| a <= b),
            Comparer::Like => compare_like(&self.value, value),
            Comparer::IsNull => value.is_null(),
            Comparer::IsNotNull => !value.is_null(),
        }
    }

}

fn validate_for_equal(value: &Option<Value>) -> bool {
    if let Some(value) = value {
        return match value {
            Value::Null => false,
            Value::Bool(_) => true,
            Value::Number(_) => true,
            Value::String(_) => true,
            Value::Array(_) => false,
            Value::Object(_) => false,
        };
    }
    false
}

fn validate_for_different(value: &Option<Value>) -> bool {
    if let Some(value) = value {
        return match value {
            Value::Null => false,
            Value::Bool(_) => true,
            Value::Number(_) => true,
            Value::String(_) => true,
            Value::Array(_) => false,
            Value::Object(_) => false,
        };
    }
    false
}

fn validate_for_greater_or_less_than(value: &Option<Value>) -> bool {
    if let Some(value) = value {
        return match value {
            Value::Null => false,
            Value::Bool(_) => false,
            Value::Number(_) => true,
            Value::String(_) => false,
            Value::Array(_) => false,
            Value::Object(_) => false,
        };
    }
    false
}

fn validate_for_like(value: &Option<Value>) -> bool {
    if let Some(value) = value {
        return match value {
            Value::Null => false,
            Value::Bool(_) => false,
            Value::Number(_) => false,
            Value::String(_) => true,
            Value::Array(_) => false,
            Value::Object(_) => false,
        };
    }
    false
}

fn compare_numbers<F>(actual_value: &Value, criteria_value: &Value, op: F) -> bool
where
    F: Fn(f64, f64) -> bool,
{
    match (actual_value.as_f64(), criteria_value.as_f64()) {
        (Some(a), Some(b)) => op(a, b),
        _ => false,
    }
}

fn compare_like(criteria_value: &Value, actual_value: &Value) -> bool {
    match (criteria_value.as_str(), actual_value.as_str()) {
        (Some(pattern), Some(text)) => {
            // Convert SQL LIKE pattern to regex
            // First, escape all regex special characters except % and _
            let mut regex_pattern = String::new();
            for ch in pattern.chars() {
                match ch {
                    '%' => regex_pattern.push_str(".*"),
                    '_' => regex_pattern.push('.'),
                    // Escape regex special characters
                    '\\' | '^' | '$' | '.' | '[' | ']' | '|' | '(' | ')' | '?' | '*' | '+' | '{' | '}' => {
                        regex_pattern.push('\\');
                        regex_pattern.push(ch);
                    },
                    _ => regex_pattern.push(ch),
                }
            }

            if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
                regex.is_match(text)
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_comparer_default() {
        assert_eq!(Comparer::default(), Comparer::Equal);
    }

    #[test]
    fn test_comparer_try_from_valid_operators() {
        assert_eq!(Comparer::try_from("=").unwrap(), Comparer::Equal);
        assert_eq!(Comparer::try_from("!=").unwrap(), Comparer::Different);
        assert_eq!(Comparer::try_from("<>").unwrap(), Comparer::Different);
        assert_eq!(Comparer::try_from(">").unwrap(), Comparer::GreaterThan);
        assert_eq!(Comparer::try_from(">=").unwrap(), Comparer::GreaterThanOrEqual);
        assert_eq!(Comparer::try_from("<").unwrap(), Comparer::LessThan);
        assert_eq!(Comparer::try_from("<=").unwrap(), Comparer::LessThanOrEqual);
        assert_eq!(Comparer::try_from("LIKE").unwrap(), Comparer::Like);
        assert_eq!(Comparer::try_from("like").unwrap(), Comparer::Like);
        assert_eq!(Comparer::try_from("IS NULL").unwrap(), Comparer::IsNull);
        assert_eq!(Comparer::try_from("is null").unwrap(), Comparer::IsNull);
        assert_eq!(Comparer::try_from("IS NOT NULL").unwrap(), Comparer::IsNotNull);
        assert_eq!(Comparer::try_from("is not null").unwrap(), Comparer::IsNotNull);
    }

    #[test]
    fn test_comparer_try_from_invalid_operator() {
        assert!(Comparer::try_from("invalid").is_err());
        assert!(Comparer::try_from("==").is_err());
        assert!(Comparer::try_from("").is_err());
        assert!(Comparer::try_from("null").is_err());
    }

    #[test]
    fn test_criteria_try_new_valid_equal() {
        let criteria = Criteria::try_new(
            "name".to_string(),
            Comparer::Equal,
            Some(json!("John")),
        );
        assert!(criteria.is_ok());
        let criteria = criteria.unwrap();
        assert_eq!(criteria.field, "name");
        assert_eq!(criteria.comparer, Comparer::Equal);
        assert_eq!(criteria.value, json!("John"));
    }

    #[test]
    fn test_criteria_try_new_valid_different() {
        let criteria = Criteria::try_new(
            "age".to_string(),
            Comparer::Different,
            Some(json!(25)),
        );
        assert!(criteria.is_ok());
    }

    #[test]
    fn test_criteria_try_new_valid_numeric_comparisons() {
        let criteria = Criteria::try_new(
            "score".to_string(),
            Comparer::GreaterThan,
            Some(json!(100)),
        );
        assert!(criteria.is_ok());

        let criteria = Criteria::try_new(
            "score".to_string(),
            Comparer::LessThanOrEqual,
            Some(json!(50.5)),
        );
        assert!(criteria.is_ok());
    }

    #[test]
    fn test_criteria_try_new_valid_like() {
        let criteria = Criteria::try_new(
            "email".to_string(),
            Comparer::Like,
            Some(json!("%@gmail.com")),
        );
        assert!(criteria.is_ok());
    }

    #[test]
    fn test_criteria_try_new_valid_null_checks() {
        let criteria = Criteria::try_new(
            "optional_field".to_string(),
            Comparer::IsNull,
            None,
        );
        assert!(criteria.is_ok());

        let criteria = Criteria::try_new(
            "required_field".to_string(),
            Comparer::IsNotNull,
            None,
        );
        assert!(criteria.is_ok());
    }

    #[test]
    fn test_criteria_try_new_invalid_equal_with_array() {
        let criteria = Criteria::try_new(
            "tags".to_string(),
            Comparer::Equal,
            Some(json!(["tag1", "tag2"])),
        );
        assert!(criteria.is_err());
    }

    #[test]
    fn test_criteria_try_new_invalid_greater_than_with_string() {
        let criteria = Criteria::try_new(
            "name".to_string(),
            Comparer::GreaterThan,
            Some(json!("John")),
        );
        assert!(criteria.is_err());
    }

    #[test]
    fn test_criteria_try_new_invalid_like_with_number() {
        let criteria = Criteria::try_new(
            "age".to_string(),
            Comparer::Like,
            Some(json!(25)),
        );
        assert!(criteria.is_err());
    }

    #[test]
    fn test_criteria_try_new_invalid_null_checks_with_value() {
        let criteria = Criteria::try_new(
            "field".to_string(),
            Comparer::IsNull,
            Some(json!("value")),
        );
        assert!(criteria.is_err());

        let criteria = Criteria::try_new(
            "field".to_string(),
            Comparer::IsNotNull,
            Some(json!("value")),
        );
        assert!(criteria.is_err());
    }

    #[test]
    fn test_criteria_compare_with_equal() {
        let criteria = Criteria::try_new(
            "name".to_string(),
            Comparer::Equal,
            Some(json!("John")),
        ).unwrap();

        assert!(criteria.compare_with(&json!("John")));
        assert!(!criteria.compare_with(&json!("Jane")));
        assert!(!criteria.compare_with(&json!(123)));
    }

    #[test]
    fn test_criteria_compare_with_different() {
        let criteria = Criteria::try_new(
            "name".to_string(),
            Comparer::Different,
            Some(json!("John")),
        ).unwrap();

        assert!(!criteria.compare_with(&json!("John")));
        assert!(criteria.compare_with(&json!("Jane")));
        assert!(criteria.compare_with(&json!(123)));
    }

    #[test]
    fn test_criteria_compare_with_numeric_comparisons() {
        let gt_criteria = Criteria::try_new(
            "age".to_string(),
            Comparer::GreaterThan,
            Some(json!(25)),
        ).unwrap();

        assert!(gt_criteria.compare_with(&json!(30)));
        assert!(!gt_criteria.compare_with(&json!(25)));
        assert!(!gt_criteria.compare_with(&json!(20)));
        assert!(!gt_criteria.compare_with(&json!("25")));

        let lte_criteria = Criteria::try_new(
            "score".to_string(),
            Comparer::LessThanOrEqual,
            Some(json!(100.5)),
        ).unwrap();

        assert!(lte_criteria.compare_with(&json!(100.5)));
        assert!(lte_criteria.compare_with(&json!(99.9)));
        assert!(!lte_criteria.compare_with(&json!(101)));
    }

    #[test]
    fn test_criteria_compare_with_like() {
        let criteria = Criteria::try_new(
            "email".to_string(),
            Comparer::Like,
            Some(json!("%@gmail.com")),
        ).unwrap();

        assert!(criteria.compare_with(&json!("john@gmail.com")));
        assert!(criteria.compare_with(&json!("user123@gmail.com")));
        assert!(!criteria.compare_with(&json!("john@yahoo.com")));
        assert!(!criteria.compare_with(&json!(123)));

        let underscore_criteria = Criteria::try_new(
            "code".to_string(),
            Comparer::Like,
            Some(json!("A_C")),
        ).unwrap();

        assert!(underscore_criteria.compare_with(&json!("ABC")));
        assert!(underscore_criteria.compare_with(&json!("A1C")));
        assert!(!underscore_criteria.compare_with(&json!("ABCC")));
        assert!(!underscore_criteria.compare_with(&json!("AC")));
    }

    #[test]
    fn test_criteria_compare_with_null_checks() {
        let is_null_criteria = Criteria::try_new(
            "optional".to_string(),
            Comparer::IsNull,
            None,
        ).unwrap();

        assert!(is_null_criteria.compare_with(&json!(null)));
        assert!(!is_null_criteria.compare_with(&json!("value")));
        assert!(!is_null_criteria.compare_with(&json!(0)));

        let is_not_null_criteria = Criteria::try_new(
            "required".to_string(),
            Comparer::IsNotNull,
            None,
        ).unwrap();

        assert!(!is_not_null_criteria.compare_with(&json!(null)));
        assert!(is_not_null_criteria.compare_with(&json!("value")));
        assert!(is_not_null_criteria.compare_with(&json!(0)));
        assert!(is_not_null_criteria.compare_with(&json!(false)));
    }

    #[test]
    fn test_criteria_compare_item() {
        let criteria = Criteria::try_new(
            "name".to_string(),
            Comparer::Equal,
            Some(json!("John")),
        ).unwrap();

        let mut item = Map::new();
        item.insert("name".to_string(), json!("John"));
        item.insert("age".to_string(), json!(30));

        assert!(criteria.compare_item(&item));

        item.insert("name".to_string(), json!("Jane"));
        assert!(!criteria.compare_item(&item));

        // Test with missing field
        item.remove("name");
        assert!(!criteria.compare_item(&item));
    }

    #[test]
    fn test_criteria_compare_item_with_complex_data() {
        let age_criteria = Criteria::try_new(
            "age".to_string(),
            Comparer::GreaterThanOrEqual,
            Some(json!(18)),
        ).unwrap();

        let email_criteria = Criteria::try_new(
            "email".to_string(),
            Comparer::Like,
            Some(json!("%@company.com")),
        ).unwrap();

        let mut user = Map::new();
        user.insert("name".to_string(), json!("Alice"));
        user.insert("age".to_string(), json!(25));
        user.insert("email".to_string(), json!("alice@company.com"));

        assert!(age_criteria.compare_item(&user));
        assert!(email_criteria.compare_item(&user));

        user.insert("age".to_string(), json!(17));
        assert!(!age_criteria.compare_item(&user));

        user.insert("email".to_string(), json!("alice@gmail.com"));
        assert!(!email_criteria.compare_item(&user));
    }

    #[test]
    fn test_compare_numbers_helper() {
        assert!(compare_numbers(&json!(10), &json!(5), |a, b| a > b));
        assert!(!compare_numbers(&json!(5), &json!(10), |a, b| a > b));
        assert!(compare_numbers(&json!(5.5), &json!(5.5), |a, b| a >= b));
        assert!(!compare_numbers(&json!("10"), &json!(5), |a, b| a > b));
        assert!(!compare_numbers(&json!(10), &json!("5"), |a, b| a > b));
    }

    #[test]
    fn test_compare_like_helper() {
        assert!(compare_like(&json!("hello%"), &json!("hello world")));
        assert!(compare_like(&json!("h_llo"), &json!("hello")));
        assert!(!compare_like(&json!("hello"), &json!("hello world")));
        assert!(!compare_like(&json!("hello%"), &json!(123)));
        assert!(!compare_like(&json!(123), &json!("hello")));

        // Test with complex patterns
        assert!(compare_like(&json!("%test%"), &json!("this is a test case")));
        assert!(compare_like(&json!("start_%_end"), &json!("start_X_end")));
        assert!(compare_like(&json!("start_%_end"), &json!("start_XX_end")));

        // Additional tests for exact single character matching
        assert!(compare_like(&json!("a_c"), &json!("abc")));
        assert!(compare_like(&json!("a_c"), &json!("a1c")));
        assert!(!compare_like(&json!("a_c"), &json!("ac")));
        assert!(!compare_like(&json!("a_c"), &json!("abcc")));

        // Test patterns that should NOT match for start_%_end
        assert!(!compare_like(&json!("start_%_end"), &json!("start_end"))); // Missing the required single char
        assert!(!compare_like(&json!("start_%_end"), &json!("start_en"))); // Wrong ending
    }

    #[test]
    fn test_validation_functions() {
        // Test validate_for_equal
        assert!(validate_for_equal(&Some(json!("string"))));
        assert!(validate_for_equal(&Some(json!(123))));
        assert!(validate_for_equal(&Some(json!(true))));
        assert!(!validate_for_equal(&Some(json!(null))));
        assert!(!validate_for_equal(&Some(json!([1, 2, 3]))));
        assert!(!validate_for_equal(&Some(json!({"key": "value"}))));
        assert!(!validate_for_equal(&None));

        // Test validate_for_different
        assert!(validate_for_different(&Some(json!("string"))));
        assert!(validate_for_different(&Some(json!(123))));
        assert!(validate_for_different(&Some(json!(false))));
        assert!(!validate_for_different(&Some(json!(null))));
        assert!(!validate_for_different(&Some(json!([1, 2, 3]))));
        assert!(!validate_for_different(&None));

        // Test validate_for_greater_or_less_than
        assert!(validate_for_greater_or_less_than(&Some(json!(123))));
        assert!(validate_for_greater_or_less_than(&Some(json!(45.6))));
        assert!(!validate_for_greater_or_less_than(&Some(json!("string"))));
        assert!(!validate_for_greater_or_less_than(&Some(json!(true))));
        assert!(!validate_for_greater_or_less_than(&Some(json!(null))));
        assert!(!validate_for_greater_or_less_than(&None));

        // Test validate_for_like
        assert!(validate_for_like(&Some(json!("pattern"))));
        assert!(!validate_for_like(&Some(json!(123))));
        assert!(!validate_for_like(&Some(json!(true))));
        assert!(!validate_for_like(&Some(json!(null))));
        assert!(!validate_for_like(&Some(json!([1, 2, 3]))));
        assert!(!validate_for_like(&None));
    }

    #[test]
    fn test_criteria_edge_cases() {
        // Test with floating point precision
        let criteria = Criteria::try_new(
            "price".to_string(),
            Comparer::Equal,
            Some(json!(99.99)),
        ).unwrap();

        assert!(criteria.compare_with(&json!(99.99)));
        assert!(!criteria.compare_with(&json!(99.9900001))); // Slight difference

        // Test with large numbers
        let large_num_criteria = Criteria::try_new(
            "value".to_string(),
            Comparer::LessThan,
            Some(json!(1e10)),
        ).unwrap();

        assert!(large_num_criteria.compare_with(&json!(1e9)));
        assert!(!large_num_criteria.compare_with(&json!(1e11)));
    }

    #[test]
    fn test_criteria_boolean_comparisons() {
        let criteria = Criteria::try_new(
            "active".to_string(),
            Comparer::Equal,
            Some(json!(true)),
        ).unwrap();

        assert!(criteria.compare_with(&json!(true)));
        assert!(!criteria.compare_with(&json!(false)));

        let different_criteria = Criteria::try_new(
            "disabled".to_string(),
            Comparer::Different,
            Some(json!(false)),
        ).unwrap();

        assert!(different_criteria.compare_with(&json!(true)));
        assert!(!different_criteria.compare_with(&json!(false)));
    }
}

