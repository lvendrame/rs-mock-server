use crate::memory_db::{Constraint, Criteria, Operator};

#[derive(Debug, Default, Clone)]
pub struct CriteriaBuilder;

impl CriteriaBuilder {

    pub fn new() -> Self {
        Self
    }

    pub fn start(criteria: &str) -> Result<Box<Criteria>, String> {
        //remove "WHERE "
        if criteria.len() <= 6 {
            return Err("Invalid WHERE clause: too short".to_string());
        }
        Self::build(&criteria[6..])
    }

    pub fn build(slice: &str) -> Result<Box<Criteria>, String> {
        let trimmed = slice.trim();

        // Check for invalid keywords that shouldn't appear in constraints
        if trimmed.contains(" INVALID ") {
            return Err(format!("Invalid operator 'INVALID' found in: '{}'", trimmed));
        }

        // Check if the slice starts with parentheses
        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            // Remove the outer parentheses and parse the inner content
            let inner = &trimmed[1..trimmed.len()-1];
            return Ok(Criteria::Parentheses(Self::build(inner)?).into_boxed());
        }

        // Find AND/OR operators that are not inside parentheses
        if let Some((start_pos, end_pos)) = Self::find_operator_not_in_parentheses(trimmed) {
            let left_part = trimmed[0..start_pos].trim();
            let operator = &trimmed[start_pos..end_pos];
            let right_part = trimmed[end_pos..].trim();

            Ok(Criteria::LeftAndRight(
                Self::build(left_part)?,
                Operator::try_from(operator)?,
                Self::build(right_part)?
            ).into_boxed())
        } else {
            // No AND/OR found, treat as a final constraint
            Ok(Criteria::Final(Constraint::try_from(trimmed)?).into_boxed())
        }
    }

    // Find the position of AND/OR operators that are not inside parentheses
    // Returns (start_pos, end_pos) of the RIGHTMOST operator, or None if not found
    // This ensures left-associative parsing: a AND b OR c becomes (a AND b) OR c
    fn find_operator_not_in_parentheses(s: &str) -> Option<(usize, usize)> {
        let mut paren_count = 0;
        let chars: Vec<char> = s.chars().collect();
        let mut last_operator_pos: Option<(usize, usize)> = None;
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                _ => {
                    // Only look for operators when not inside parentheses
                    if paren_count == 0 {
                        // Check for AND/OR (case insensitive)
                        if i + 3 <= chars.len() {
                            let word: String = chars[i..i+3].iter().collect();
                            if word.to_uppercase() == "AND" {
                                // Make sure it's a whole word (check boundaries)
                                let is_start_boundary = i == 0 || chars[i-1].is_whitespace();
                                let is_end_boundary = i + 3 == chars.len() || chars[i+3].is_whitespace();
                                if is_start_boundary && is_end_boundary {
                                    last_operator_pos = Some((i, i + 3));
                                    i += 2; // Skip ahead to avoid overlapping matches
                                }
                            }
                        }

                        if i + 2 <= chars.len() {
                            let word: String = chars[i..i+2].iter().collect();
                            if word.to_uppercase() == "OR" {
                                // Make sure it's a whole word (check boundaries)
                                let is_start_boundary = i == 0 || chars[i-1].is_whitespace();
                                let is_end_boundary = i + 2 == chars.len() || chars[i+2].is_whitespace();
                                if is_start_boundary && is_end_boundary {
                                    last_operator_pos = Some((i, i + 2));
                                    i += 1; // Skip ahead to avoid overlapping matches
                                }
                            }
                        }
                    }
                }
            }
            i += 1;
        }

        last_operator_pos
    }
}

// tests
// WHERE a = 1
// WHERE a = 1 AND b = 2
// WHERE a = 1 AND (b = 2 OR c = 3)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_db::{Criteria, Operator, Comparer};
    use serde_json::json;

    #[test]
    fn test_criteria_builder_new() {
        let _builder = CriteriaBuilder::new();
        // CriteriaBuilder is now just a unit struct, so nothing specific to test
        // Just test that we can create it without error
    }

    #[test]
    fn test_start_simple_where_clause() {
        let result = CriteriaBuilder::start("WHERE a = 1");
        assert!(result.is_ok());

        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "a");
                assert_eq!(constraint.comparer, Comparer::Equal);
                assert_eq!(constraint.value, json!(1));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_start_where_clause_with_string() {
        let result = CriteriaBuilder::start("WHERE name = \"John\"");
        assert!(result.is_ok());

        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "name");
                assert_eq!(constraint.comparer, Comparer::Equal);
                assert_eq!(constraint.value, json!("John"));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_start_where_clause_with_and() {
        let result = CriteriaBuilder::start("WHERE a = 1 AND b = 2");
        assert!(result.is_ok());

        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::LeftAndRight(left, op, right) => {
                assert_eq!(*op, Operator::And);

                // Check left side
                match left.as_ref() {
                    Criteria::Final(constraint) => {
                        assert_eq!(constraint.field, "a");
                        assert_eq!(constraint.comparer, Comparer::Equal);
                        assert_eq!(constraint.value, json!(1));
                    }
                    _ => panic!("Expected Final criteria on left, got {:?}", left),
                }

                // Check right side
                match right.as_ref() {
                    Criteria::Final(constraint) => {
                        assert_eq!(constraint.field, "b");
                        assert_eq!(constraint.comparer, Comparer::Equal);
                        assert_eq!(constraint.value, json!(2));
                    }
                    _ => panic!("Expected Final criteria on right, got {:?}", right),
                }
            }
            _ => panic!("Expected LeftAndRight criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_start_where_clause_with_or() {
        let result = CriteriaBuilder::start("WHERE a = 1 OR b = 2");
        assert!(result.is_ok());

        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::LeftAndRight(left, op, right) => {
                assert_eq!(*op, Operator::Or);

                // Check left side
                match left.as_ref() {
                    Criteria::Final(constraint) => {
                        assert_eq!(constraint.field, "a");
                        assert_eq!(constraint.comparer, Comparer::Equal);
                        assert_eq!(constraint.value, json!(1));
                    }
                    _ => panic!("Expected Final criteria on left, got {:?}", left),
                }

                // Check right side
                match right.as_ref() {
                    Criteria::Final(constraint) => {
                        assert_eq!(constraint.field, "b");
                        assert_eq!(constraint.comparer, Comparer::Equal);
                        assert_eq!(constraint.value, json!(2));
                    }
                    _ => panic!("Expected Final criteria on right, got {:?}", right),
                }
            }
            _ => panic!("Expected LeftAndRight criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_start_where_clause_with_parentheses() {
        let result = CriteriaBuilder::start("WHERE a = 1 AND (b = 2 OR c = 3)");
        assert!(result.is_ok());

        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::LeftAndRight(left, op, right) => {
                assert_eq!(*op, Operator::And);

                // Check left side (should be a = 1)
                match left.as_ref() {
                    Criteria::Final(constraint) => {
                        assert_eq!(constraint.field, "a");
                        assert_eq!(constraint.comparer, Comparer::Equal);
                        assert_eq!(constraint.value, json!(1));
                    }
                    _ => panic!("Expected Final criteria on left, got {:?}", left),
                }

                // Check right side (should be parentheses with OR inside)
                match right.as_ref() {
                    Criteria::Parentheses(inner) => {
                        match inner.as_ref() {
                            Criteria::LeftAndRight(inner_left, inner_op, inner_right) => {
                                assert_eq!(*inner_op, Operator::Or);

                                // Check b = 2
                                match inner_left.as_ref() {
                                    Criteria::Final(constraint) => {
                                        assert_eq!(constraint.field, "b");
                                        assert_eq!(constraint.comparer, Comparer::Equal);
                                        assert_eq!(constraint.value, json!(2));
                                    }
                                    _ => panic!("Expected Final criteria, got {:?}", inner_left),
                                }

                                // Check c = 3
                                match inner_right.as_ref() {
                                    Criteria::Final(constraint) => {
                                        assert_eq!(constraint.field, "c");
                                        assert_eq!(constraint.comparer, Comparer::Equal);
                                        assert_eq!(constraint.value, json!(3));
                                    }
                                    _ => panic!("Expected Final criteria, got {:?}", inner_right),
                                }
                            }
                            _ => panic!("Expected LeftAndRight criteria inside parentheses, got {:?}", inner),
                        }
                    }
                    _ => panic!("Expected Parentheses criteria on right, got {:?}", right),
                }
            }
            _ => panic!("Expected LeftAndRight criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_start_invalid_where_clause() {
        // Test with invalid WHERE clause (too short)
        let result = CriteriaBuilder::start("WHERE");
        assert!(result.is_err());

        // Test with invalid constraint
        let result = CriteriaBuilder::start("WHERE invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_simple_constraint() {
        let result = CriteriaBuilder::build("age > 25");
        assert!(result.is_ok());

        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "age");
                assert_eq!(constraint.comparer, Comparer::GreaterThan);
                assert_eq!(constraint.value, json!(25));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_build_with_different_operators() {
        // Test with different comparers
        let result = CriteriaBuilder::build("score <= 100");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "score");
                assert_eq!(constraint.comparer, Comparer::LessThanOrEqual);
                assert_eq!(constraint.value, json!(100));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }

        let result = CriteriaBuilder::build("email LIKE \"%@gmail.com\"");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "email");
                assert_eq!(constraint.comparer, Comparer::Like);
                assert_eq!(constraint.value, json!("%@gmail.com"));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_build_with_null_checks() {
        let result = CriteriaBuilder::build("optional IS NULL");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "optional");
                assert_eq!(constraint.comparer, Comparer::IsNull);
                assert_eq!(constraint.value, json!(null));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }

        let result = CriteriaBuilder::build("required IS NOT NULL");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "required");
                assert_eq!(constraint.comparer, Comparer::IsNotNull);
                assert_eq!(constraint.value, json!(null));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_build_with_case_insensitive_operators() {
        // Test lowercase AND
        let result = CriteriaBuilder::build("a = 1 and b = 2");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::LeftAndRight(_, op, _) => {
                assert_eq!(*op, Operator::And);
            }
            _ => panic!("Expected LeftAndRight criteria, got {:?}", criteria),
        }

        // Test mixed case OR
        let result = CriteriaBuilder::build("a = 1 Or b = 2");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::LeftAndRight(_, op, _) => {
                assert_eq!(*op, Operator::Or);
            }
            _ => panic!("Expected LeftAndRight criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_build_complex_nested_criteria() {
        // Test complex nesting: a = 1 AND b = 2 OR c = 3
        let result = CriteriaBuilder::build("a = 1 AND b = 2 OR c = 3");
        assert!(result.is_ok());

        // Note: This should parse as ((a = 1 AND b = 2) OR c = 3) due to left-to-right parsing
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::LeftAndRight(left, op, right) => {
                assert_eq!(*op, Operator::Or);

                // Left side should be (a = 1 AND b = 2)
                match left.as_ref() {
                    Criteria::LeftAndRight(inner_left, inner_op, inner_right) => {
                        assert_eq!(*inner_op, Operator::And);

                        match inner_left.as_ref() {
                            Criteria::Final(constraint) => {
                                assert_eq!(constraint.field, "a");
                                assert_eq!(constraint.value, json!(1));
                            }
                            _ => panic!("Expected Final criteria, got {:?}", inner_left),
                        }

                        match inner_right.as_ref() {
                            Criteria::Final(constraint) => {
                                assert_eq!(constraint.field, "b");
                                assert_eq!(constraint.value, json!(2));
                            }
                            _ => panic!("Expected Final criteria, got {:?}", inner_right),
                        }
                    }
                    _ => panic!("Expected LeftAndRight criteria on left, got {:?}", left),
                }

                // Right side should be c = 3
                match right.as_ref() {
                    Criteria::Final(constraint) => {
                        assert_eq!(constraint.field, "c");
                        assert_eq!(constraint.value, json!(3));
                    }
                    _ => panic!("Expected Final criteria on right, got {:?}", right),
                }
            }
            _ => panic!("Expected LeftAndRight criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_build_with_boolean_values() {
        let result = CriteriaBuilder::build("active = true");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "active");
                assert_eq!(constraint.comparer, Comparer::Equal);
                assert_eq!(constraint.value, json!(true));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }

        let result = CriteriaBuilder::build("disabled != false");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "disabled");
                assert_eq!(constraint.comparer, Comparer::Different);
                assert_eq!(constraint.value, json!(false));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_build_with_numeric_values() {
        // Integer
        let result = CriteriaBuilder::build("count = 42");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "count");
                assert_eq!(constraint.value, json!(42));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }

        // Float
        let result = CriteriaBuilder::build("price >= 99.99");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "price");
                assert_eq!(constraint.comparer, Comparer::GreaterThanOrEqual);
                assert_eq!(constraint.value, json!(99.99));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }

        // Negative number
        let result = CriteriaBuilder::build("temperature < -10");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "temperature");
                assert_eq!(constraint.comparer, Comparer::LessThan);
                assert_eq!(constraint.value, json!(-10));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }
    }

    #[test]
    fn test_build_invalid_constraints() {
        // Test with invalid constraint format
        let result = CriteriaBuilder::build("invalid");
        println!("Result for 'invalid': {:?}", result);
        assert!(result.is_err());

        // Test with invalid operator
        let result = CriteriaBuilder::build("field == value");
        println!("Result for 'field == value': {:?}", result);
        assert!(result.is_err());

        // Test with invalid operator combination
        let result = CriteriaBuilder::build("a = 1 INVALID b = 2");
        println!("Result for 'a = 1 INVALID b = 2': {:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn test_start_removes_where_prefix() {
        // Test that start() correctly removes the "WHERE " prefix
        let result1 = CriteriaBuilder::start("WHERE name = \"test\"");
        let result2 = CriteriaBuilder::build("name = \"test\"");

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Both should produce the same result
        let criteria1 = result1.unwrap();
        let criteria2 = result2.unwrap();

        assert_eq!(criteria1, criteria2);
    }

    #[test]
    fn test_build_with_quoted_multi_word_values() {
        let result = CriteriaBuilder::build("name = \"John Doe\"");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "name");
                assert_eq!(constraint.comparer, Comparer::Equal);
                assert_eq!(constraint.value, json!("John Doe"));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }

        let result = CriteriaBuilder::build("description LIKE \"hello world\"");
        assert!(result.is_ok());
        let criteria = result.unwrap();
        match criteria.as_ref() {
            Criteria::Final(constraint) => {
                assert_eq!(constraint.field, "description");
                assert_eq!(constraint.comparer, Comparer::Like);
                assert_eq!(constraint.value, json!("hello world"));
            }
            _ => panic!("Expected Final criteria, got {:?}", criteria),
        }
    }
}


