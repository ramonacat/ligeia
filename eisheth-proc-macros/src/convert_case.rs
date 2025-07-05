pub fn pascal_to_snake<TIter: ToString>(input: &str, make_case: impl Fn(char) -> TIter) -> String {
    let mut output = String::new();

    for c in input.chars() {
        if c.is_uppercase() && !output.is_empty() {
            output += "_";
        }

        output.push_str(&(make_case)(c).to_string());
    }

    output
}

pub fn pascal_to_upper_snake(input: &str) -> String {
    pascal_to_snake(input, char::to_uppercase)
}

pub fn pascal_to_lower_snake(input: &str) -> String {
    pascal_to_snake(input, char::to_lowercase)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_pascal_to_lower_snake() {
        let cases = [("", ""), ("Test", "test"), ("TestTwo", "test_two")];

        for (input, expected) in cases {
            assert_eq!(expected, pascal_to_lower_snake(input));
        }
    }

    #[test]
    pub fn test_pascal_to_upper_snake() {
        let cases = [("", ""), ("Test", "TEST"), ("TestTwo", "TEST_TWO")];

        for (input, expected) in cases {
            assert_eq!(expected, pascal_to_upper_snake(input));
        }
    }
}
