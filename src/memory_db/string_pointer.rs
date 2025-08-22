#[derive(Debug, Default, Clone, PartialEq)]
pub struct StringPointer {
    pub start_position: usize,
    pub end_position: usize,
    pub current_position: usize,
    pub start_char: char,
    pub end_char: char,
}

impl StringPointer {

    pub fn new(start_char: char, end_char: char) -> Self {
        Self {
            current_position: 0,
            start_char,
            end_char,
            ..Default::default()
        }
    }

    pub fn next(&mut self, value: &str) -> bool {
        let chars: Vec<char> = value.chars().collect();

        if self.current_position >= chars.len() {
            return false;
        }

        self.start_position = self.current_position;
        while self.current_position < chars.len() &&
            chars[self.current_position] != self.start_char &&
            chars[self.current_position] != self.end_char {
            self.current_position += 1;
        }
        self.end_position = self.current_position;

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pointer_new() {
        let pointer = StringPointer::new('(', ')');
        assert_eq!(pointer.start_char, '(');
        assert_eq!(pointer.end_char, ')');
        assert_eq!(pointer.current_position, 0);
        assert_eq!(pointer.start_position, 0);
        assert_eq!(pointer.end_position, 0);
    }

    #[test]
    fn test_next_with_no_special_chars() {
        let mut pointer = StringPointer::new('(', ')');
        let text = "hello world";

        let result = pointer.next(text);
        assert!(result);
        assert_eq!(pointer.start_position, 0);
        assert_eq!(pointer.end_position, 11); // Should go to end of string
        assert_eq!(pointer.current_position, 11);
    }

    #[test]
    fn test_next_with_start_char() {
        let mut pointer = StringPointer::new('(', ')');
        let text = "hello(world";

        let result = pointer.next(text);
        assert!(result);
        assert_eq!(pointer.start_position, 0);
        assert_eq!(pointer.end_position, 5); // Should stop at '('
        assert_eq!(pointer.current_position, 5);
    }

    #[test]
    fn test_next_with_end_char() {
        let mut pointer = StringPointer::new('(', ')');
        let text = "hello)world";

        let result = pointer.next(text);
        assert!(result);
        assert_eq!(pointer.start_position, 0);
        assert_eq!(pointer.end_position, 5); // Should stop at ')'
        assert_eq!(pointer.current_position, 5);
    }

    #[test]
    fn test_next_with_both_chars() {
        let mut pointer = StringPointer::new('(', ')');
        let text = "hello(world)test";

        let result = pointer.next(text);
        assert!(result);
        assert_eq!(pointer.start_position, 0);
        assert_eq!(pointer.end_position, 5); // Should stop at first special char '('
        assert_eq!(pointer.current_position, 5);
    }

    #[test]
    fn test_next_starting_with_special_char() {
        let mut pointer = StringPointer::new('(', ')');
        let text = "(hello world)";

        let result = pointer.next(text);
        assert!(result);
        assert_eq!(pointer.start_position, 0);
        assert_eq!(pointer.end_position, 0); // Should stop immediately at '('
        assert_eq!(pointer.current_position, 0);
    }

    #[test]
    fn test_next_empty_string() {
        let mut pointer = StringPointer::new('(', ')');
        let text = "";

        let result = pointer.next(text);
        assert!(!result); // Should return false for empty string
        assert_eq!(pointer.start_position, 0);
        assert_eq!(pointer.end_position, 0);
        assert_eq!(pointer.current_position, 0);
    }

    #[test]
    fn test_multiple_next_calls() {
        let mut pointer = StringPointer::new('(', ')');
        let text = "a(b)c(d)e";

        // First call: should go from start to first '('
        let result1 = pointer.next(text);
        assert!(result1);
        assert_eq!(pointer.start_position, 0);
        assert_eq!(pointer.end_position, 1); // Stops at '('
        assert_eq!(pointer.current_position, 1);

        // Move past the '(' for next call
        pointer.current_position += 1;

        // Second call: should go from after '(' to ')'
        let result2 = pointer.next(text);
        assert!(result2);
        assert_eq!(pointer.start_position, 2);
        assert_eq!(pointer.end_position, 3); // Stops at ')'
        assert_eq!(pointer.current_position, 3);
    }

    #[test]
    fn test_next_beyond_string_length() {
        let mut pointer = StringPointer::new('(', ')');
        let text = "hello";

        // First call should work
        let result1 = pointer.next(text);
        assert!(result1);
        assert_eq!(pointer.current_position, 5);

        // Second call should return false since we're at end
        let result2 = pointer.next(text);
        assert!(!result2);
    }

    #[test]
    fn test_next_with_different_chars() {
        let mut pointer = StringPointer::new('[', ']');
        let text = "hello[world]test";

        let result = pointer.next(text);
        assert!(result);
        assert_eq!(pointer.start_position, 0);
        assert_eq!(pointer.end_position, 5); // Should stop at '['
        assert_eq!(pointer.current_position, 5);
    }

    #[test]
    fn test_string_pointer_for_criteria_parsing() {
        // Test case similar to what criteria builder might use
        let mut pointer = StringPointer::new('(', ')');
        let text = "a = 1 AND (b = 2 OR c = 3)";

        let result = pointer.next(text);
        assert!(result);
        assert_eq!(pointer.start_position, 0);
        assert_eq!(pointer.end_position, 10); // Should stop at '('
        assert_eq!(pointer.current_position, 10);

        // Extract the part before parentheses
        let before_paren = &text[pointer.start_position..pointer.end_position];
        assert_eq!(before_paren, "a = 1 AND ");
    }

    #[test]
    fn test_consecutive_special_chars() {
        let mut pointer = StringPointer::new('(', ')');
        let text = "()";

        let result = pointer.next(text);
        assert!(result);
        assert_eq!(pointer.start_position, 0);
        assert_eq!(pointer.end_position, 0); // Should stop at first '('
        assert_eq!(pointer.current_position, 0);
    }
}
