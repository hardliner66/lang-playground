use std::fmt;

use tracing::instrument;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    InterpolatedString(Vec<InterpolationPart>),
    Symbol(String),
    True,
    False,
    Nil,

    // Identifiers and Keywords
    Identifier(String),
    Proc,
    Struct,
    Import,
    If,
    Else,
    For,
    In,
    Break,
    Continue,
    Return,
    Defer,
    When,
    Case,
    Where,
    Distinct,
    Using,
    Cast,
    Transmute,
    AutoCast,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Power,          // **
    Equal,          // ==
    NotEqual,       // !=
    LessThan,       // <
    LessEqual,      // <=
    GreaterThan,    // >
    GreaterEqual,   // >=
    Spaceship,      // <=>
    And,            // &&
    Or,             // ||
    Not,            // !
    BitwiseAnd,     // &
    BitwiseOr,      // |
    BitwiseXor,     // ^
    BitwiseNot,     // ~
    LeftShift,      // <<
    RightShift,     // >>
    Range,          // ..
    ExclusiveRange, // ...
    Arrow,          // ->
    ColonAssign,    // :=
    FatArrow,       // =>

    // Assignment
    Assign, // =

    // Delimiters
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Colon,
    DoubleColon,
    Semicolon,
    Question,
    Pipe,
    Hash,
    Newline,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationPart {
    Literal(String),
    Expression(Vec<Token>),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "Integer({})", n),
            Token::Float(n) => write!(f, "Float({})", n),
            Token::String(s) => write!(f, "String(\"{}\")", s),
            Token::Symbol(s) => write!(f, "Symbol(:{})", s),
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        Lexer {
            input: chars,
            position: 0,
            current_char,
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        };
    }

    fn peek(&self, offset: usize) -> Option<char> {
        let pos = self.position + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    #[instrument(skip(self))]
    fn skip_comment(&mut self) {
        // Skip single-line comments starting with //
        if self.current_char == Some('/') && self.peek(1) == Some('/') {
            while self.current_char.is_some() && self.current_char != Some('\n') {
                self.advance();
            }
        }
    }

    #[instrument(ret, skip(self))]
    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && self.peek(1).map_or(false, |c| c.is_ascii_digit()) {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else if ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token::Float(num_str.parse().unwrap())
        } else {
            Token::Integer(num_str.parse().unwrap())
        }
    }

    #[instrument(ret, skip(self))]
    fn read_string(&mut self, quote: char) -> Token {
        if quote == '"' {
            return self.read_interpolated_string();
        }

        let mut string = String::new();
        self.advance();

        while let Some(ch) = self.current_char {
            if ch == quote {
                self.advance();
                break;
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char {
                    let escaped_char = match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '\'' => '\'',
                        '"' => '"',
                        _ => escaped,
                    };
                    string.push(escaped_char);
                    self.advance();
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Token::String(string)
    }

    #[instrument(ret, skip(self))]
    fn read_interpolated_string(&mut self) -> Token {
        let mut parts = Vec::new();
        let mut current_literal = String::new();
        self.advance();

        while let Some(ch) = self.current_char {
            if ch == '"' {
                if !current_literal.is_empty() {
                    parts.push(InterpolationPart::Literal(current_literal));
                }
                self.advance();
                break;
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char {
                    let escaped_char = match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '@' => '@',
                        _ => escaped,
                    };
                    current_literal.push(escaped_char);
                    self.advance();
                }
            } else if ch == '#' && self.peek(1) == Some('{') {
                if !current_literal.is_empty() {
                    parts.push(InterpolationPart::Literal(current_literal.clone()));
                    current_literal.clear();
                }

                self.advance();
                self.advance();

                let expr_tokens = self.read_interpolation_expression();
                parts.push(InterpolationPart::Expression(expr_tokens));
            } else {
                current_literal.push(ch);
                self.advance();
            }
        }

        if parts.is_empty() {
            Token::String(String::new())
        } else if parts.len() == 1 {
            if let InterpolationPart::Literal(s) = &parts[0] {
                Token::String(s.clone())
            } else {
                Token::InterpolatedString(parts)
            }
        } else {
            Token::InterpolatedString(parts)
        }
    }

    #[instrument(ret, skip(self))]
    fn read_interpolation_expression(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut brace_depth = 1;

        while let Some(ch) = self.current_char {
            if ch == '}' {
                brace_depth -= 1;
                if brace_depth == 0 {
                    self.advance();
                    break;
                }
            } else if ch == '{' {
                brace_depth += 1;
            }

            if ch == ' ' || ch == '\t' {
                self.advance();
                continue;
            }

            let token = self.next_token_for_interpolation();
            if token != Token::Eof {
                tokens.push(token);
            }
        }

        tokens
    }

    #[instrument(ret, skip(self))]
    fn next_token_for_interpolation(&mut self) -> Token {
        match self.current_char {
            None => Token::Eof,
            Some('}') => Token::Eof,
            Some('+') => {
                self.advance();
                Token::Plus
            }
            Some('-') => {
                self.advance();
                Token::Minus
            }
            Some('*') => {
                self.advance();
                if self.current_char == Some('*') {
                    self.advance();
                    Token::Power
                } else {
                    Token::Star
                }
            }
            Some('/') => {
                self.advance();
                Token::Slash
            }
            Some('%') => {
                self.advance();
                Token::Percent
            }
            Some('(') => {
                self.advance();
                Token::LeftParen
            }
            Some(')') => {
                self.advance();
                Token::RightParen
            }
            Some('[') => {
                self.advance();
                Token::LeftBracket
            }
            Some(']') => {
                self.advance();
                Token::RightBracket
            }
            Some('.') => {
                self.advance();
                Token::Dot
            }
            Some(',') => {
                self.advance();
                Token::Comma
            }
            Some('"') => self.read_string('"'),
            Some('\'') => self.read_string('\''),
            Some(ch) if ch.is_ascii_digit() => self.read_number(),
            Some(ch) if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
            Some('@') => {
                self.advance();
                let mut ident = String::from("@");
                while let Some(ch) = self.current_char {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
                Token::Identifier(ident)
            }
            Some(_) => {
                self.advance();
                Token::Eof
            }
        }
    }

    #[instrument(ret, skip(self))]
    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' || ch == '?' || ch == '!' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        match ident.as_str() {
            "proc" => Token::Proc,
            "struct" => Token::Struct,
            "import" => Token::Import,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "in" => Token::In,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "return" => Token::Return,
            "defer" => Token::Defer,
            "when" => Token::When,
            "case" => Token::Case,
            "where" => Token::Where,
            "distinct" => Token::Distinct,
            "using" => Token::Using,
            "cast" => Token::Cast,
            "transmute" => Token::Transmute,
            "auto_cast" => Token::AutoCast,
            "true" => Token::True,
            "false" => Token::False,
            "nil" => Token::Nil,
            _ => Token::Identifier(ident),
        }
    }

    #[instrument(ret, skip(self))]

    pub fn next_token(&mut self) -> Token {
        loop {
            self.skip_whitespace();

            if self.current_char == Some('@') {
                if self.peek(1) == Some('[') {
                    break;
                } else {
                    self.skip_comment();
                    continue;
                }
            }

            break;
        }

        match self.current_char {
            None => Token::Eof,
            Some('\n') => {
                self.advance();
                Token::Newline
            }
            Some(';') => {
                self.advance();
                Token::Semicolon
            }
            Some('+') => {
                self.advance();
                Token::Plus
            }
            Some('*') => {
                self.advance();
                if self.current_char == Some('*') {
                    self.advance();
                    Token::Power
                } else {
                    Token::Star
                }
            }

            Some('/') => {
                // Check for comment
                if self.peek(1) == Some('/') {
                    self.skip_comment();
                    self.next_token()
                } else {
                    self.advance();
                    Token::Slash
                }
            }
            Some('%') => {
                self.advance();
                Token::Percent
            }
            Some('=') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::Equal
                } else if self.current_char == Some('>') {
                    self.advance();
                    Token::FatArrow
                } else {
                    Token::Assign
                }
            }
            Some('!') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::NotEqual
                } else {
                    Token::Not
                }
            }
            Some('<') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    if self.current_char == Some('>') {
                        self.advance();
                        Token::Spaceship
                    } else {
                        Token::LessEqual
                    }
                } else if self.current_char == Some('<') {
                    self.advance();
                    Token::LeftShift
                } else {
                    Token::LessThan
                }
            }
            Some('>') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::GreaterEqual
                } else if self.current_char == Some('>') {
                    self.advance();
                    Token::RightShift
                } else {
                    Token::GreaterThan
                }
            }
            Some('&') => {
                self.advance();
                if self.current_char == Some('&') {
                    self.advance();
                    Token::And
                } else {
                    Token::BitwiseAnd
                }
            }
            Some('|') => {
                self.advance();
                if self.current_char == Some('|') {
                    self.advance();
                    Token::Or
                } else {
                    Token::Pipe
                }
            }
            Some('^') => {
                self.advance();
                Token::BitwiseXor
            }
            Some('~') => {
                self.advance();
                Token::BitwiseNot
            }
            Some('@') => {
                if self.peek(1) == Some('[') {
                    self.advance();
                    Token::Hash
                } else {
                    self.skip_comment();
                    self.next_token()
                }
            }
            Some('.') => {
                self.advance();
                if self.current_char == Some('.') {
                    self.advance();
                    if self.current_char == Some('.') {
                        self.advance();
                        Token::ExclusiveRange
                    } else {
                        Token::Range
                    }
                } else {
                    Token::Dot
                }
            }
            Some('(') => {
                self.advance();
                Token::LeftParen
            }
            Some(')') => {
                self.advance();
                Token::RightParen
            }
            Some('[') => {
                self.advance();
                Token::LeftBracket
            }
            Some(']') => {
                self.advance();
                Token::RightBracket
            }
            Some('{') => {
                self.advance();
                Token::LeftBrace
            }
            Some('}') => {
                self.advance();
                Token::RightBrace
            }
            Some(',') => {
                self.advance();
                Token::Comma
            }
            Some(':') => {
                self.advance();
                if self.current_char == Some(':') {
                    self.advance();
                    Token::DoubleColon
                } else if self.current_char == Some('=') {
                    self.advance();
                    Token::ColonAssign
                } else if self
                    .current_char
                    .map_or(false, |c| c.is_alphabetic() || c == '_')
                {
                    // Read symbol (e.g., :symbol)
                    let mut symbol = String::new();
                    while let Some(ch) = self.current_char {
                        if ch.is_alphanumeric() || ch == '_' {
                            symbol.push(ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    Token::Symbol(symbol)
                } else {
                    Token::Colon
                }
            }
            Some('-') => {
                self.advance();
                if self.current_char == Some('>') {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            Some('?') => {
                self.advance();
                Token::Question
            }
            Some('"') => self.read_string('"'),
            Some('\'') => self.read_string('\''),
            Some(ch) if ch.is_ascii_digit() => self.read_number(),
            Some(ch) if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
            Some(ch) => {
                panic!("Unexpected character: '{}'", ch);
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14 1_000_000");
        assert_eq!(lexer.next_token(), Token::Integer(42));
        assert_eq!(lexer.next_token(), Token::Float(3.14));
        assert_eq!(lexer.next_token(), Token::Integer(1000000));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new(r#""hello" 'world'"#);
        assert_eq!(lexer.next_token(), Token::String("hello".to_string()));
        assert_eq!(lexer.next_token(), Token::String("world".to_string()));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("proc struct import if else for");
        assert_eq!(lexer.next_token(), Token::Proc);
        assert_eq!(lexer.next_token(), Token::Struct);
        assert_eq!(lexer.next_token(), Token::Import);
        assert_eq!(lexer.next_token(), Token::If);
        assert_eq!(lexer.next_token(), Token::Else);
        assert_eq!(lexer.next_token(), Token::For);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / ** == != < <= > >= <=> && ||");
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Minus);
        assert_eq!(lexer.next_token(), Token::Star);
        assert_eq!(lexer.next_token(), Token::Slash);
        assert_eq!(lexer.next_token(), Token::Power);
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::NotEqual);
        assert_eq!(lexer.next_token(), Token::LessThan);
        assert_eq!(lexer.next_token(), Token::LessEqual);
        assert_eq!(lexer.next_token(), Token::GreaterThan);
        assert_eq!(lexer.next_token(), Token::GreaterEqual);
        assert_eq!(lexer.next_token(), Token::Spaceship);
        assert_eq!(lexer.next_token(), Token::And);
        assert_eq!(lexer.next_token(), Token::Or);
    }

    #[test]
    fn test_symbols() {
        let mut lexer = Lexer::new(":name :value");
        assert_eq!(lexer.next_token(), Token::Symbol("name".to_string()));
        assert_eq!(lexer.next_token(), Token::Symbol("value".to_string()));
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("foo bar_baz empty? save!");
        assert_eq!(lexer.next_token(), Token::Identifier("foo".to_string()));
        assert_eq!(lexer.next_token(), Token::Identifier("bar_baz".to_string()));
        assert_eq!(lexer.next_token(), Token::Identifier("empty?".to_string()));
        assert_eq!(lexer.next_token(), Token::Identifier("save!".to_string()));
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("x # this is a comment\ny");
        assert_eq!(lexer.next_token(), Token::Identifier("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Newline);
        assert_eq!(lexer.next_token(), Token::Identifier("y".to_string()));
    }

    #[test]
    fn test_string_interpolation() {
        let mut lexer = Lexer::new(r#""Hello #{name}!""#);
        let token = lexer.next_token();
        match token {
            Token::InterpolatedString(parts) => {
                assert_eq!(parts.len(), 3);
                assert!(matches!(&parts[0], InterpolationPart::Literal(s) if s == "Hello "));
                assert!(matches!(&parts[1], InterpolationPart::Expression(_)));
                assert!(matches!(&parts[2], InterpolationPart::Literal(s) if s == "!"));
            }
            _ => panic!("Expected InterpolatedString, got {:?}", token),
        }
    }

    #[test]
    fn test_string_interpolation_complex() {
        let mut lexer = Lexer::new(r#""Result: #{x + y * 2}""#);
        let token = lexer.next_token();
        match token {
            Token::InterpolatedString(parts) => {
                assert_eq!(parts.len(), 2);
                assert!(matches!(&parts[0], InterpolationPart::Literal(s) if s == "Result: "));
                if let InterpolationPart::Expression(tokens) = &parts[1] {
                    assert!(tokens.len() >= 3);
                }
            }
            _ => panic!("Expected InterpolatedString, got {:?}", token),
        }
    }

    #[test]
    fn test_no_interpolation_single_quote() {
        let mut lexer = Lexer::new(r#"'Hello #{name}!'"#);
        assert_eq!(
            lexer.next_token(),
            Token::String("Hello #{name}!".to_string())
        );
    }
}
