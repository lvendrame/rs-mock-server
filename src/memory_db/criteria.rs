use serde_json::{Map, Value};

use crate::memory_db::Constraint;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Operator {
    #[default]
    And,
    Or,
}

impl TryFrom<&str> for Operator {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "AND" => Ok(Operator::And),
            "OR" => Ok(Operator::Or),
            _ => Err(format!("Invalid operator: '{}'", value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Criteria {
    Final(Constraint),
    LeftAndRight(Box<Criteria>, Operator, Box<Criteria>),
    Parentheses(Box<Criteria>),
}

impl Criteria {
    pub fn into_boxed(self) -> Box<Self> {
        Box::new(self)
    }

    pub fn compare_item(&self, item: &Map<String, Value>) -> bool {
        match self {
            Criteria::Final(constraint) => constraint.compare_item(item),
            Criteria::LeftAndRight(criteria_a, operator, criteria_b) => {
                match operator {
                    Operator::And => criteria_a.compare_item(item) && criteria_b.compare_item(item),
                    Operator::Or => criteria_a.compare_item(item) || criteria_b.compare_item(item),
                }
            },
            Criteria::Parentheses(criteria) => criteria.compare_item(item),
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_criteria_compare_item_and_parentheses_or() {
        // WHERE A = 1 AND (B = 2 OR B = 3)
        let c_a = Criteria::Final(Constraint {
            field: "A".to_string(),
            comparer: Comparer::Equal,
            value: json!(1),
        });
        let c_b2 = Criteria::Final(Constraint {
            field: "B".to_string(),
            comparer: Comparer::Equal,
            value: json!(2),
        });
        let c_b3 = Criteria::Final(Constraint {
            field: "B".to_string(),
            comparer: Comparer::Equal,
            value: json!(3),
        });
        let or_b = Criteria::LeftAndRight(Box::new(c_b2), Operator::Or, Box::new(c_b3));
        let paren_or_b = Criteria::Parentheses(Box::new(or_b));
        let and_criteria = Criteria::LeftAndRight(Box::new(c_a), Operator::And, Box::new(paren_or_b));

        // Should match if A=1 AND (B=2 OR B=3)
        let mut item = Map::new();
        item.insert("A".to_string(), json!(1));
        item.insert("B".to_string(), json!(2));
        assert!(and_criteria.compare_item(&item));

        let mut item = Map::new();
        item.insert("A".to_string(), json!(1));
        item.insert("B".to_string(), json!(3));
        assert!(and_criteria.compare_item(&item));

        let mut item = Map::new();
        item.insert("A".to_string(), json!(1));
        item.insert("B".to_string(), json!(4));
        assert!(!and_criteria.compare_item(&item));

        let mut item = Map::new();
        item.insert("A".to_string(), json!(2));
        item.insert("B".to_string(), json!(2));
        assert!(!and_criteria.compare_item(&item));

        let mut item = Map::new();
        item.insert("A".to_string(), json!(2));
        item.insert("B".to_string(), json!(3));
        assert!(!and_criteria.compare_item(&item));
    }
    use super::*;
    use serde_json::{json, Map, Value};
    use crate::memory_db::{Constraint, Comparer};

    #[test]
    fn test_operator_try_from_valid() {
        assert_eq!(Operator::try_from("AND"), Ok(Operator::And));
        assert_eq!(Operator::try_from("and"), Ok(Operator::And));
        assert_eq!(Operator::try_from("Or"), Ok(Operator::Or));
        assert_eq!(Operator::try_from("OR"), Ok(Operator::Or));
    }

    #[test]
    fn test_operator_try_from_invalid() {
        assert!(Operator::try_from("XOR").is_err());
        assert!(Operator::try_from("").is_err());
        assert!(Operator::try_from("ANDOR").is_err());
    }

    fn make_item(field: &str, value: Value) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(field.to_string(), value);
        map
    }

    #[test]
    fn test_criteria_compare_item_final() {
        let constraint = Constraint {
            field: "age".to_string(),
            comparer: Comparer::Equal,
            value: json!(30),
        };
        let criteria = Criteria::Final(constraint);
        let item = make_item("age", json!(30));
        assert!(criteria.compare_item(&item));
        let item = make_item("age", json!(25));
        assert!(!criteria.compare_item(&item));
    }

    #[test]
    fn test_criteria_compare_item_left_and_right() {
        let c1 = Criteria::Final(Constraint {
            field: "a".to_string(),
            comparer: Comparer::Equal,
            value: json!(1),
        });
        let c2 = Criteria::Final(Constraint {
            field: "b".to_string(),
            comparer: Comparer::Equal,
            value: json!(2),
        });
        let and_criteria = Criteria::LeftAndRight(Box::new(c1.clone()), Operator::And, Box::new(c2.clone()));
        let or_criteria = Criteria::LeftAndRight(Box::new(c1.clone()), Operator::Or, Box::new(c2.clone()));

        let mut item = Map::new();
        item.insert("a".to_string(), json!(1));
        item.insert("b".to_string(), json!(2));
        assert!(and_criteria.compare_item(&item));
        assert!(or_criteria.compare_item(&item));

        let mut item = Map::new();
        item.insert("a".to_string(), json!(1));
        item.insert("b".to_string(), json!(3));
        assert!(!and_criteria.compare_item(&item));
        assert!(or_criteria.compare_item(&item));

        let mut item = Map::new();
        item.insert("a".to_string(), json!(0));
        item.insert("b".to_string(), json!(2));
        assert!(!and_criteria.compare_item(&item));
        assert!(or_criteria.compare_item(&item));

        let mut item = Map::new();
        item.insert("a".to_string(), json!(0));
        item.insert("b".to_string(), json!(3));
        assert!(!and_criteria.compare_item(&item));
        assert!(!or_criteria.compare_item(&item));
    }

    #[test]
    fn test_criteria_compare_item_parentheses() {
        let inner = Criteria::Final(Constraint {
            field: "x".to_string(),
            comparer: Comparer::Equal,
            value: json!(42),
        });
        let paren_criteria = Criteria::Parentheses(Box::new(inner));
        let item = make_item("x", json!(42));
        assert!(paren_criteria.compare_item(&item));
        let item = make_item("x", json!(0));
        assert!(!paren_criteria.compare_item(&item));
    }

    #[test]
    fn test_criteria_compare_item_nested_and_or() {
        // ((a = 1 AND b = 2) OR (c = 3 AND d = 4))
        let c_a = Criteria::Final(Constraint {
            field: "a".to_string(),
            comparer: Comparer::Equal,
            value: json!(1),
        });
        let c_b = Criteria::Final(Constraint {
            field: "b".to_string(),
            comparer: Comparer::Equal,
            value: json!(2),
        });
        let c_c = Criteria::Final(Constraint {
            field: "c".to_string(),
            comparer: Comparer::Equal,
            value: json!(3),
        });
        let c_d = Criteria::Final(Constraint {
            field: "d".to_string(),
            comparer: Comparer::Equal,
            value: json!(4),
        });

        let and1 = Criteria::LeftAndRight(Box::new(c_a.clone()), Operator::And, Box::new(c_b.clone()));
        let and2 = Criteria::LeftAndRight(Box::new(c_c.clone()), Operator::And, Box::new(c_d.clone()));
        let or_nested = Criteria::LeftAndRight(Box::new(and1.clone()), Operator::Or, Box::new(and2.clone()));

        // Should match if (a=1 AND b=2) OR (c=3 AND d=4)
        let mut item = Map::new();
        item.insert("a".to_string(), json!(1));
        item.insert("b".to_string(), json!(2));
        item.insert("c".to_string(), json!(0));
        item.insert("d".to_string(), json!(0));
        assert!(or_nested.compare_item(&item));

        let mut item = Map::new();
        item.insert("a".to_string(), json!(0));
        item.insert("b".to_string(), json!(0));
        item.insert("c".to_string(), json!(3));
        item.insert("d".to_string(), json!(4));
        assert!(or_nested.compare_item(&item));

        let mut item = Map::new();
        item.insert("a".to_string(), json!(1));
        item.insert("b".to_string(), json!(0));
        item.insert("c".to_string(), json!(3));
        item.insert("d".to_string(), json!(4));
        assert!(or_nested.compare_item(&item)); // c=3 AND d=4 matches

        let mut item = Map::new();
        item.insert("a".to_string(), json!(1));
        item.insert("b".to_string(), json!(2));
        item.insert("c".to_string(), json!(3));
        item.insert("d".to_string(), json!(0));
        assert!(or_nested.compare_item(&item)); // a=1 AND b=2 matches

        let mut item = Map::new();
        item.insert("a".to_string(), json!(0));
        item.insert("b".to_string(), json!(0));
        item.insert("c".to_string(), json!(0));
        item.insert("d".to_string(), json!(0));
        assert!(!or_nested.compare_item(&item)); // No match
    }

    #[test]
    fn test_criteria_compare_item_deep_parentheses() {
        // (((a = 1 AND b = 2) OR c = 3) AND (d = 4 OR e = 5))
        let c_a = Criteria::Final(Constraint {
            field: "a".to_string(),
            comparer: Comparer::Equal,
            value: json!(1),
        });
        let c_b = Criteria::Final(Constraint {
            field: "b".to_string(),
            comparer: Comparer::Equal,
            value: json!(2),
        });
        let c_c = Criteria::Final(Constraint {
            field: "c".to_string(),
            comparer: Comparer::Equal,
            value: json!(3),
        });
        let c_d = Criteria::Final(Constraint {
            field: "d".to_string(),
            comparer: Comparer::Equal,
            value: json!(4),
        });
        let c_e = Criteria::Final(Constraint {
            field: "e".to_string(),
            comparer: Comparer::Equal,
            value: json!(5),
        });

        let and_ab = Criteria::LeftAndRight(Box::new(c_a.clone()), Operator::And, Box::new(c_b.clone()));
        let or_ab_c = Criteria::LeftAndRight(Box::new(and_ab.clone()), Operator::Or, Box::new(c_c.clone()));
        let or_de = Criteria::LeftAndRight(Box::new(c_d.clone()), Operator::Or, Box::new(c_e.clone()));
        let top = Criteria::LeftAndRight(Box::new(or_ab_c.clone()), Operator::And, Box::new(or_de.clone()));

        // Should match if ((a=1 AND b=2) OR c=3) AND (d=4 OR e=5)
        let mut item = Map::new();
        item.insert("a".to_string(), json!(1));
        item.insert("b".to_string(), json!(2));
        item.insert("c".to_string(), json!(0));
        item.insert("d".to_string(), json!(4));
        item.insert("e".to_string(), json!(0));
        assert!(top.compare_item(&item));

        let mut item = Map::new();
        item.insert("a".to_string(), json!(0));
        item.insert("b".to_string(), json!(0));
        item.insert("c".to_string(), json!(3));
        item.insert("d".to_string(), json!(0));
        item.insert("e".to_string(), json!(5));
        assert!(top.compare_item(&item));

        let mut item = Map::new();
        item.insert("a".to_string(), json!(1));
        item.insert("b".to_string(), json!(2));
        item.insert("c".to_string(), json!(0));
        item.insert("d".to_string(), json!(0));
        item.insert("e".to_string(), json!(5));
        assert!(top.compare_item(&item));

        let mut item = Map::new();
        item.insert("a".to_string(), json!(0));
        item.insert("b".to_string(), json!(0));
        item.insert("c".to_string(), json!(3));
        item.insert("d".to_string(), json!(4));
        item.insert("e".to_string(), json!(0));
        assert!(top.compare_item(&item));

        let mut item = Map::new();
        item.insert("a".to_string(), json!(0));
        item.insert("b".to_string(), json!(0));
        item.insert("c".to_string(), json!(0));
        item.insert("d".to_string(), json!(0));
        item.insert("e".to_string(), json!(0));
        assert!(!top.compare_item(&item));
    }
}

