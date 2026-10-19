#[derive(Debug, PartialEq)]
pub enum Token {
    Plus,
    Minus,
    Star,
    Div,
    Mod,
    Equal,
    GreaterThan,
    LessThan,
    NotEqual,
    EqualTo,
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Bang,
    Dot,
    Semicolon,
    Number(i64),
    Float(f64),
    Bool(bool),
    Ident(String),
    Function,
    StringLiteral(String),
    Var,
    If,
    Else,
    While,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    BitwiseAnd,
    BitwiseOr,
    Break,
    Continue,
    Return,
    EOF,
}

// TODO we want to incr line count with ever new line but nested loops are eating them

pub fn scan(source: &str) -> Result<Vec<Token>, &str> {
    let mut vec: Vec<Token> = Vec::new();
    let mut iter = source.chars().peekable();
    while let Some(ch) = iter.next() {
        match ch {
            ',' => vec.push(Token::Comma),
            '(' => vec.push(Token::LeftParen),
            ')' => vec.push(Token::RightParen),
            '{' => vec.push(Token::LeftBrace),
            '}' => vec.push(Token::RightBrace),
            '+' => vec.push(Token::Plus),
            '-' => vec.push(Token::Minus),
            '*' => vec.push(Token::Star),
            '/' => {
                if matches!(iter.peek(), Some('/')) {
                    iter.next();
                    while let Some(c) = iter.next() {
                        if c == '\n' {
                            break;
                        }
                    }
                } else if matches!(iter.peek(), Some('*')) {
                    iter.next();
                    let mut closed = false;
                    while let Some(c) = iter.next() {
                        if c == '*' && matches!(iter.peek(), Some('/')) {
                            iter.next();
                            closed = true;
                            break;
                        }
                    }
                    if !closed {
                        return Err("Unterminated multi-line comment");
                    }
                } else {
                    vec.push(Token::Div);
                }
            }
            '%' => vec.push(Token::Mod),
            '!' => {
                if matches!(iter.peek(), Some('=')) {
                    iter.next();
                    vec.push(Token::NotEqual);
                } else {
                    vec.push(Token::LogicalNot);
                }
            }
            '=' => {
                if matches!(iter.peek(), Some('=')) {
                    iter.next();
                    vec.push(Token::EqualTo);
                } else {
                    vec.push(Token::Equal); // This should be Assignment token not equal
                }
            }
            '>' => {
                if matches!(iter.peek(), Some('=')) {
                    iter.next();
                    vec.push(Token::GreaterThanOrEqualTo);
                } else {
                    vec.push(Token::GreaterThan);
                }
            }
            '<' => {
                if matches!(iter.peek(), Some('=')) {
                    iter.next();
                    vec.push(Token::LessThanOrEqualTo);
                } else {
                    vec.push(Token::LessThan);
                }
            }
            '&' => {
                if matches!(iter.peek(), Some('&')) {
                    iter.next();
                    vec.push(Token::LogicalAnd);
                } else {
                    vec.push(Token::BitwiseAnd);
                }
            }
            '|' => {
                if matches!(iter.peek(), Some('|')) {
                    iter.next();
                    vec.push(Token::LogicalOr);
                } else {
                    vec.push(Token::BitwiseOr);
                }
            }
            ';' => vec.push(Token::Semicolon),
            // '\n' => {
            // TODO automatic semi colon insertion
            // }
            c if c.is_whitespace() => {}
            c if c.is_ascii_digit() => {
                let mut number = String::from(c);
                let mut has_dot = false;
                let mut needs_digit = false;
                while let Some(next_num) = iter.peek() {
                    match next_num {
                        '0'..='9' => {
                            number.push(*next_num);
                            needs_digit = false;
                            iter.next();
                        }
                        '.' => {
                            if has_dot || needs_digit {
                                return Err("Invalid numeric literal");
                            }
                            has_dot = true;
                            needs_digit = true;
                            number.push(*next_num);
                            iter.next();
                        }
                        '_' => {
                            if needs_digit {
                                return Err("Invalid numeric literal");
                            }
                            needs_digit = true;
                            iter.next(); // consume and ignore 
                        }
                        _ => break,
                    }
                }

                if needs_digit {
                    return Err("Invalid numeric literal");
                }

                if has_dot {
                    let value = number.parse::<f64>().map_err(|_| "Invalid float literal")?;
                    vec.push(Token::Float(value));
                } else {
                    let value = number
                        .parse::<i64>()
                        .map_err(|_| "Number literal overflow")?;
                    vec.push(Token::Number(value));
                }
            }
            c if c.is_alphabetic() => {
                let mut ident: String = String::from(ch);
                while let Some(next_ch) = iter.peek() {
                    if next_ch.is_alphanumeric() || *next_ch == '_' {
                        ident.push(*next_ch);
                        iter.next();
                    } else {
                        break;
                    }
                }

                let target: Token = match ident.as_str() {
                    "var" => Token::Var,
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    "fun" => Token::Function,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "break" => Token::Break,
                    "return" => Token::Return,
                    "continue" => Token::Continue,
                    "and" => Token::LogicalAnd,
                    "or" => Token::LogicalOr,
                    "not" => Token::LogicalNot,
                    ident => Token::Ident(ident.to_string()),
                };
                vec.push(target);
            }
            '"' => {
                // theres probably a better way to do this. scan and capture n at once
                let mut str_lit = String::new();
                let mut closed = false;
                while let Some(next_char) = iter.next() {
                    if next_char == '"' {
                        closed = true;
                        break;
                    } else {
                        str_lit.push(next_char);
                    }
                }

                if !closed {
                    return Err("Unterminated string literal");
                }

                vec.push(Token::StringLiteral(str_lit));
            }

            // TODO, store line, start, end offsets with the error and tokens for reporting
            _ => return Err("Unexpected character"),
        }
    }
    vec.push(Token::EOF);
    Ok(vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal() {
        assert_eq!(scan("="), Ok(vec![Token::Equal, Token::EOF]))
    }
    #[test]
    fn test_plus() {
        assert_eq!(scan("+"), Ok(vec![Token::Plus, Token::EOF]))
    }
    #[test]
    fn test_star() {
        assert_eq!(scan("*"), Ok(vec![Token::Star, Token::EOF]))
    }
    #[test]
    fn test_div() {
        assert_eq!(scan("/"), Ok(vec![Token::Div, Token::EOF]))
    }
    #[test]
    fn test_mod() {
        assert_eq!(scan("%"), Ok(vec![Token::Mod, Token::EOF]))
    }
    #[test]
    fn test_not_equal() {
        assert_eq!(scan("!="), Ok(vec![Token::NotEqual, Token::EOF]))
    }
    #[test]
    fn test_equal_to() {
        assert_eq!(scan("=="), Ok(vec![Token::EqualTo, Token::EOF]))
    }
    #[test]
    fn test_semicolon() {
        assert_eq!(scan(";"), Ok(vec![Token::Semicolon, Token::EOF]))
    }
    #[test]
    fn test_comma() {
        assert_eq!(scan(","), Ok(vec![Token::Comma, Token::EOF]))
    }

    #[test]
    fn test_bang() {
        assert_eq!(scan("!"), Ok(vec![Token::LogicalNot, Token::EOF]))
    }
    #[test]
    fn test_not() {
        assert_eq!(scan("not"), Ok(vec![Token::LogicalNot, Token::EOF]))
    }

    #[test]
    fn test_greater_than() {
        assert_eq!(scan(">"), Ok(vec![Token::GreaterThan, Token::EOF]))
    }
    #[test]
    fn test_less_than() {
        assert_eq!(scan("<"), Ok(vec![Token::LessThan, Token::EOF]))
    }
    #[test]
    fn test_left_paren() {
        assert_eq!(scan("("), Ok(vec![Token::LeftParen, Token::EOF]))
    }
    #[test]
    fn test_right_paren() {
        assert_eq!(scan(")"), Ok(vec![Token::RightParen, Token::EOF]))
    }
    #[test]
    fn test_left_brace() {
        assert_eq!(scan("{"), Ok(vec![Token::LeftBrace, Token::EOF]))
    }
    #[test]
    fn test_right_brace() {
        assert_eq!(scan("}"), Ok(vec![Token::RightBrace, Token::EOF]))
    }
    #[test]
    fn test_greater_than_or_equal_to() {
        assert_eq!(
            scan(">="),
            Ok(vec![Token::GreaterThanOrEqualTo, Token::EOF])
        )
    }
    #[test]
    fn test_less_than_or_equal_to() {
        assert_eq!(scan("<="), Ok(vec![Token::LessThanOrEqualTo, Token::EOF]))
    }

    #[test]
    fn test_if() {
        assert_eq!(scan("if"), Ok(vec![Token::If, Token::EOF]))
    }

    #[test]
    fn test_else() {
        assert_eq!(scan("else"), Ok(vec![Token::Else, Token::EOF]))
    }

    #[test]
    fn test_logical_and() {
        assert_eq!(scan("&&"), Ok(vec![Token::LogicalAnd, Token::EOF]))
    }

    #[test]
    fn test_logical_or() {
        assert_eq!(scan("||"), Ok(vec![Token::LogicalOr, Token::EOF]))
    }

    #[test]
    fn test_bitwise_and() {
        assert_eq!(scan("&"), Ok(vec![Token::BitwiseAnd, Token::EOF]))
    }

    #[test]
    fn test_bitwise_or() {
        assert_eq!(scan("|"), Ok(vec![Token::BitwiseOr, Token::EOF]))
    }

    #[test]
    fn test_int_number() {
        assert_eq!(scan("1"), Ok(vec![Token::Number(1), Token::EOF]));
        assert_eq!(scan("12"), Ok(vec![Token::Number(12), Token::EOF]));
        assert_eq!(scan("123"), Ok(vec![Token::Number(123), Token::EOF]));
    }

    #[test]
    fn test_number_overflow() {
        assert_eq!(
            scan("189789123749237437248972389473892479237489237943"),
            Err("Number literal overflow")
        );
    }

    #[test]
    fn test_number_separator() {
        assert_eq!(
            scan("1_000_000"),
            Ok(vec![Token::Number(1000000), Token::EOF])
        );
        assert_eq!(
            scan("1_000.50"),
            Ok(vec![Token::Float(1000.50), Token::EOF])
        );
    }

    #[test]
    fn test_invalid_numbers() {
        assert_eq!(scan("10_"), Err("Invalid numeric literal"));
        assert_eq!(scan("1__000"), Err("Invalid numeric literal"));
        assert_eq!(scan("1._0"), Err("Invalid numeric literal"));
        assert_eq!(scan(".05"), Err("Unexpected character"));
        assert_eq!(scan("1."), Err("Invalid numeric literal"));
        assert_eq!(scan("1.1.1"), Err("Invalid numeric literal"));
        assert_eq!(scan("1..3"), Err("Invalid numeric literal"));
    }

    #[test]
    fn test_float_number() {
        assert_eq!(scan("3.14"), Ok(vec![Token::Float(3.14), Token::EOF]));
    }

    #[test]
    fn test_bool_true() {
        assert_eq!(scan("true"), Ok(vec![Token::Bool(true), Token::EOF]));
    }
    #[test]
    fn test_bool_false() {
        assert_eq!(scan("false"), Ok(vec![Token::Bool(false), Token::EOF]));
    }

    #[test]
    fn test_identifier() {
        assert_eq!(
            scan("c"),
            Ok(vec![Token::Ident("c".to_string()), Token::EOF])
        );
        assert_eq!(
            scan("result"),
            Ok(vec![Token::Ident("result".to_string()), Token::EOF])
        );
        assert_eq!(
            scan("snake_case"),
            Ok(vec![Token::Ident("snake_case".to_string()), Token::EOF])
        );
        assert_eq!(
            scan("password1234"),
            Ok(vec![Token::Ident("password1234".to_string()), Token::EOF])
        );
    }

    #[test]
    fn test_keyword_boundary() {
        assert_eq!(
            scan("varx"),
            Ok(vec![Token::Ident("varx".to_string()), Token::EOF])
        );
    }

    #[test]
    fn test_var() {
        assert_eq!(scan("var"), Ok(vec![Token::Var, Token::EOF]))
    }

    #[test]
    fn test_break() {
        assert_eq!(scan("break"), Ok(vec![Token::Break, Token::EOF]))
    }

    #[test]
    fn test_continue() {
        assert_eq!(scan("continue"), Ok(vec![Token::Continue, Token::EOF]))
    }

    #[test]
    fn test_return() {
        assert_eq!(scan("return"), Ok(vec![Token::Return, Token::EOF]))
    }

    #[test]
    fn test_single_line_comment() {
        assert_eq!(scan("// comment here"), Ok(vec![Token::EOF]))
    }

    #[test]
    fn test_end_of_line_comment() {
        assert_eq!(
            scan(
                "var x; // comment here
                var y;"
            ),
            Ok(vec![
                Token::Var,
                Token::Ident("x".to_string()),
                Token::Semicolon,
                Token::Var,
                Token::Ident("y".to_string()),
                Token::Semicolon,
                Token::EOF
            ])
        )
    }

    #[test]
    fn test_multi_line_comment() {
        assert_eq!(
            scan(
                "var x;
                /* comment here
                var y;
                */"
            ),
            Ok(vec![
                Token::Var,
                Token::Ident("x".to_string()),
                Token::Semicolon,
                Token::EOF
            ])
        )
    }

    #[test]
    fn test_inline_multi_line_comment() {
        assert_eq!(
            scan("var x /* comment here */;"),
            Ok(vec![
                Token::Var,
                Token::Ident("x".to_string()),
                Token::Semicolon,
                Token::EOF
            ])
        )
    }

    #[test]
    fn test_unterminated_multiline_comment() {
        assert_eq!(scan("/* hello..."), Err("Unterminated multi-line comment"));
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(
            scan("\"! var!\""),
            Ok(vec![Token::StringLiteral("! var!".to_string()), Token::EOF])
        );
    }

    #[test]
    fn test_empty_string_literal() {
        assert_eq!(
            scan("\"\""),
            Ok(vec![Token::StringLiteral("".to_string()), Token::EOF])
        );
    }

    #[test]
    fn test_unterminated_string_literal() {
        assert_eq!(scan("\"hello..."), Err("Unterminated string literal"));
    }

    #[test]
    fn test_empty_source() {
        assert_eq!(scan(""), Ok(vec![Token::EOF]));
        assert_eq!(scan("        "), Ok(vec![Token::EOF]));
    }

    #[test]
    fn test_assignment() {
        assert_eq!(
            scan("var x = 10;"),
            Ok(vec![
                Token::Var,
                Token::Ident("x".to_string()),
                Token::Equal,
                Token::Number(10),
                Token::Semicolon,
                Token::EOF
            ])
        );

        assert_eq!(
            scan("var ok = true;"),
            Ok(vec![
                Token::Var,
                Token::Ident("ok".to_string()),
                Token::Equal,
                Token::Bool(true),
                Token::Semicolon,
                Token::EOF
            ])
        );
    }

    #[test]
    fn test_assignment_with_string_literals() {
        assert_eq!(
            scan("var name = \"Mitch\";"),
            Ok(vec![
                Token::Var,
                Token::Ident("name".to_string()),
                Token::Equal,
                Token::StringLiteral("Mitch".to_string()),
                Token::Semicolon,
                Token::EOF
            ])
        );
    }

    #[test]
    fn test_expression() {
        assert_eq!(
            scan("x = x + 3 * 4;"),
            Ok(vec![
                Token::Ident("x".to_string()),
                Token::Equal,
                Token::Ident("x".to_string()),
                Token::Plus,
                Token::Number(3),
                Token::Star,
                Token::Number(4),
                Token::Semicolon,
                Token::EOF
            ])
        );
    }

    #[test]
    fn test_unexpected_character() {
        assert_eq!(scan("@"), Err("Unexpected character"));
    }
}
