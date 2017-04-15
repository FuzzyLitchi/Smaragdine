use lexer::Tokenizer;
use lexer::token::{Token, TokenType};

macro_rules! token {
    ($tokenizer:expr, $token_type:ident, $accum:expr) => {{
        token!($tokenizer , TokenType::$token_type, $accum)
    }};
    ($tokenizer:expr, $token_type:expr, $accum:expr) => {{
        let tokenizer = $tokenizer as &$crate::lexer::Tokenizer;
        let token_type = $token_type as $crate::lexer::token::TokenType;
        Some(Token::new(token_type, tokenizer.last_position(), $accum))
    }};
}

/// Matcher.
pub trait Matcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token>;
}

/// A matcher that only matches white-space.
pub struct WhitespaceMatcher {}

impl Matcher for WhitespaceMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        let mut found = false;
        while !tokenizer.end() && tokenizer.peek().unwrap().is_whitespace() {
            found = true;
            tokenizer.next();
        }
        if found {
            token!(tokenizer, Whitespace, String::new())
        } else {
            None
        }
    }
}

/// A matcher that matches base-10 integer literals.
pub struct IntLiteralMatcher {}

impl Matcher for IntLiteralMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        let mut accum = String::new();
        let base = match tokenizer.peek().unwrap() {
            &'0' => {
                match tokenizer.peek_n(1) {
                    Some(chr) => {
                        match chr {
                            &'x' => 16, // base 16 (hexadecimal)
                            &'b' => 2, // base 2 (binary)
                            _ => 10, // base 10 (decimal)
                        }
                    }
                    _ => 10, // base 10 (decimal)
                }
            }
            _ => 10, // base 10 (decimal)
        };
        if base != 10 {
            tokenizer.advance(2); // skip prefix
        }
        while !tokenizer.end() && tokenizer.peek().unwrap().is_digit(base) {
            accum.push(tokenizer.next().unwrap());
        }
        if !accum.is_empty() {
            // Produce token as base-10 string
            let literal: String = match u64::from_str_radix(accum.as_str(), base) {
                Ok(result) => result.to_string(),
                Err(error) => panic!("Unable to parse integer literal: {}", error)
            };
            token!(tokenizer, IntLiteral, literal)
        } else {
            None
        }
    }
}

/// A matcher that matches string literals.
pub struct StringLiteralMatcher {}

impl Matcher for StringLiteralMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        let delimeter  = match tokenizer.peek().unwrap() {
            &'"'  => Some('"'),
            &'\'' => Some('\''),
            _ => return None,
        };
        tokenizer.advance(1); // Skips the opening delimeter
        let mut string       = String::new();
        let mut found_escape = false;
        loop {
            if tokenizer.end() {
                break
            }
            match delimeter.unwrap() {
                '\''  => {
                    if tokenizer.peek().unwrap() == &'\'' {
                        break
                    }
                    string.push(tokenizer.next().unwrap())
                },
                _ => {
                    if found_escape {
                        string.push(
                            match tokenizer.next().unwrap() {
                                c @ '\\' | c @ '"' => c,
                                'n' => '\n',
                                'r' => '\r',
                                't' => '\t',
                                s => panic!("Invalid character escape: {}", s),
                            }
                        );
                        found_escape = false
                    } else {
                        match tokenizer.peek().unwrap() {
                            &'\\' => {
                                tokenizer.next();
                                found_escape = true
                            },
                            &'"' => break,
                            _ => string.push(tokenizer.next().unwrap()),
                        }
                    }
                }
            }
        }
        tokenizer.advance(1); // Skips the closing delimeter

        if string.len() == 1 {
            token!(tokenizer, CharLiteral, string)
        } else {
            token!(tokenizer, StringLiteral, string)
        }
    }
}

/// A matcher that matches constant elements
/// of the specified token type.
pub struct ConstantMatcher {
    token_type: TokenType,
    constants: Vec<String>,
}

impl ConstantMatcher {
    pub fn new(token_type: TokenType, constants: Vec<String>) -> Self {
        ConstantMatcher {
            token_type: token_type,
            constants: constants,
        }
    }
}

impl Matcher for ConstantMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        for constant in self.constants.clone() {
            let dat = tokenizer.clone().take(constant.len());
            if dat.size_hint().1.unwrap() != constant.len() {
                return None;
            }
            if dat.collect::<String>() == constant {
                tokenizer.advance(constant.len());
                return token!(tokenizer, self.token_type.clone(), constant)
            }
        }
        None
    }
}

/// A matcher that matches identifiers.
pub struct IdentifierMatcher {}

impl Matcher for IdentifierMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        let mut identifier = String::new();
        let curr = tokenizer.next().unwrap();
        if curr.is_alphabetic() || curr == '_' {
            identifier.push(curr)
        } else {
            return None;
        }
        while !tokenizer.end() {
            let current = *tokenizer.peek().unwrap();
            if !current.is_whitespace() && ("_?!".contains(current) || current.is_alphanumeric()) {
                identifier.push(tokenizer.next().unwrap());
            } else {
                break;
            }
        }
        if !identifier.is_empty() {
            token!(tokenizer, Identifier, identifier)
        } else {
            None
        }
    }
}