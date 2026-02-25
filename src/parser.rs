use std::fmt::Display;

use tracing::{info, instrument};

use crate::ast::*;
use crate::ast::{Attribute, AttributeArg};
use crate::lexer::{Lexer, Token};

pub enum Either<A, B> {
    Left(A),
    Right(B),
}

pub struct Parser {
    code: String,
    tokens: Vec<Token>,
    position: usize,
    current_token: Token,
}

#[derive(Debug)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} at {}", self.message, self.span)
    }
}

impl ParseError {
    fn new(parser: &Parser, message: String) -> Self {
        let mut line = 0;
        let mut column = 0;
        for c in parser.code.chars().take(parser.position) {
            if c == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        ParseError {
            message,
            span: Span { line, column },
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new(input: &str) -> Self {
        info!("====================================");
        info!("LEXING...");
        info!("====================================");
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let current_token = tokens.get(0).cloned().unwrap_or(Token::Eof);
        Parser {
            code: input.to_string(),
            tokens,
            position: 0,
            current_token,
        }
    }

    #[instrument(skip(self))]
    fn advance(&mut self) {
        self.position += 1;
        self.current_token = self
            .tokens
            .get(self.position)
            .cloned()
            .unwrap_or(Token::Eof);
    }

    #[instrument(skip(self))]
    fn peek(&self, offset: usize) -> Token {
        self.tokens
            .get(self.position + offset)
            .cloned()
            .unwrap_or(Token::Eof)
    }

    #[instrument(skip(self))]
    fn expect(&mut self, expected: Token) -> ParseResult<()> {
        if self.current_token == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(
                self,
                format!("Expected {:?}, got {:?}", expected, self.current_token),
            ))
        }
    }

    fn skip_newlines(&mut self) {
        while self.current_token == Token::Newline {
            self.advance();
        }
    }

    #[instrument(skip(self))]
    fn skip_terminators(&mut self) {
        while self.current_token == Token::Newline
            || self.current_token == Token::Semicolon
            || self.current_token == Token::Comma
        {
            self.advance();
        }
    }

    #[instrument(skip(self))]
    pub fn parse(&mut self) -> ParseResult<Program> {
        self.skip_newlines();
        let statements = self.parse_statements()?;
        Ok(Program::Statements(statements))
    }

    #[instrument(skip(self))]
    fn parse_statements(&mut self) -> ParseResult<Vec<Statement>> {
        let mut statements = Vec::new();

        loop {
            self.skip_newlines();

            match &self.current_token {
                Token::Else | Token::Eof | Token::RightBrace => break,
                _ => {}
            }

            let stmt = self.parse_statement()?;
            statements.push(stmt);

            self.skip_terminators();
        }

        Ok(statements)
    }

    #[instrument(skip(self))]
    fn parse_fields(&mut self) -> ParseResult<Vec<Field>> {
        let mut fields = Vec::new();

        loop {
            self.skip_newlines();

            match &self.current_token {
                Token::Else | Token::Eof | Token::RightBrace => break,
                _ => {}
            }

            let stmt = self.parse_field()?;
            fields.push(stmt);

            self.skip_terminators();
        }

        Ok(fields)
    }

    #[instrument(skip(self))]
    fn parse_receivers(&mut self) -> ParseResult<Vec<Expression>> {
        self.expect(Token::To)?;
        let mut receivers = Vec::new();

        if self.current_token == Token::LeftBracket {
            loop {
                match &self.current_token {
                    Token::Else | Token::Eof | Token::RightBracket => break,
                    _ => {}
                }

                if let Token::Identifier(name) = &self.current_token {
                    receivers.push(Expression::String(name.clone()));
                }

                self.skip_terminators();
            }
        } else if let Token::Identifier(name) = &self.current_token {
            receivers.push(Expression::String(name.clone()));
        } else {
            return Err(ParseError::new(
                self,
                format!(
                    "Expected identifier or left bracket, got {:?}",
                    self.current_token
                ),
            ));
        }

        Ok(receivers)
    }

    #[instrument(skip(self))]
    fn parse_attributes(&mut self) -> ParseResult<Vec<Attribute>> {
        let mut attributes = Vec::new();

        while self.current_token == Token::Hash {
            self.advance();
            self.expect(Token::LeftBracket)?;

            let name = if let Token::Identifier(n) = &self.current_token {
                let name = n.clone();
                self.advance();
                name
            } else {
                return Err(ParseError::new(
                    self,
                    format!("Expected attribute name, got {:?}", self.current_token),
                ));
            };

            let args = if self.current_token == Token::LeftParen {
                self.advance();
                let args = self.parse_attribute_args()?;
                self.expect(Token::RightParen)?;
                args
            } else {
                Vec::new()
            };

            self.expect(Token::RightBracket)?;
            self.skip_newlines();

            attributes.push(Attribute { name, args });
        }

        Ok(attributes)
    }

    #[instrument(skip(self))]
    fn parse_attribute_args(&mut self) -> ParseResult<Vec<AttributeArg>> {
        let mut args = Vec::new();

        if self.current_token == Token::RightParen {
            return Ok(args);
        }

        loop {
            let arg = self.parse_attribute_arg()?;
            args.push(arg);

            if self.current_token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }

        Ok(args)
    }

    #[instrument(skip(self))]
    fn parse_attribute_arg(&mut self) -> ParseResult<AttributeArg> {
        match &self.current_token.clone() {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();

                if self.current_token == Token::Assign {
                    self.advance();
                    let value = self.parse_attribute_arg()?;
                    Ok(AttributeArg::KeyValue(name, Box::new(value)))
                } else {
                    Ok(AttributeArg::Identifier(name))
                }
            }
            Token::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(AttributeArg::String(s))
            }
            Token::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(AttributeArg::Integer(n))
            }
            Token::True => {
                self.advance();
                Ok(AttributeArg::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(AttributeArg::Boolean(false))
            }
            _ => Err(ParseError::new(
                self,
                format!("Expected attribute argument, got {:?}", self.current_token),
            )),
        }
    }

    #[instrument(skip(self))]
    fn parse_statement(&mut self) -> ParseResult<Statement> {
        self.skip_newlines();

        let attributes = if self.current_token == Token::Hash {
            self.parse_attributes()?
        } else {
            Vec::new()
        };

        match &self.current_token {
            Token::Import => self.parse_import_statement(),
            Token::If => {
                if !attributes.is_empty() {
                    return Ok(Statement::Attributes(attributes));
                }
                self.parse_if_statement()
            }
            Token::For => self.parse_for_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Break => {
                self.advance();
                Ok(Statement::Break)
            }
            Token::Continue => {
                self.advance();
                Ok(Statement::Continue)
            }
            Token::Defer => self.parse_defer_statement(),
            Token::Identifier(_) => {
                if !attributes.is_empty() {
                    return Ok(Statement::Attributes(attributes));
                }
                // Check for :: which means declaration (proc or struct)
                if self.peek(1) == Token::DoubleColon {
                    self.parse_declaration(attributes)
                } else if self.peek(1) == Token::ColonAssign {
                    self.parse_colon_assignment()
                } else if self.peek(1) == Token::Assign {
                    self.parse_assignment()
                } else {
                    let expr = self.parse_expression()?;
                    Ok(Statement::Expression(expr))
                }
            }
            _ => {
                if !attributes.is_empty() {
                    return Ok(Statement::Attributes(attributes));
                }
                let expr = self.parse_expression()?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    #[instrument(skip(self))]
    fn parse_field(&mut self) -> ParseResult<Field> {
        self.skip_newlines();

        let attributes = if self.current_token == Token::Hash {
            self.parse_attributes()?
        } else {
            Vec::new()
        };

        let name = if let Token::Identifier(n) = &self.current_token {
            let name = n.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::new(
                self,
                format!("Expected field name, got {:?}", self.current_token),
            ));
        };

        self.expect(Token::Colon)?;

        let typ = if let Token::Identifier(t) = &self.current_token {
            let typ = t.clone();
            self.advance();
            typ
        } else {
            return Err(ParseError::new(
                self,
                format!("Expected field type, got {:?}", self.current_token),
            ));
        };

        // Skip optional comma
        if self.current_token == Token::Comma {
            self.advance();
        }

        Ok(Field {
            name,
            typ,
            attributes,
        })
    }

    #[instrument(skip(self))]
    fn parse_declaration(&mut self, attributes: Vec<Attribute>) -> ParseResult<Statement> {
        // name :: proc(...) or name :: struct {...}
        let name = if let Token::Identifier(n) = &self.current_token {
            let name = n.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::new(
                self,
                format!("Expected identifier, got {:?}", self.current_token),
            ));
        };

        self.expect(Token::DoubleColon)?;

        match &self.current_token {
            Token::Proc => self.parse_proc_def(name, attributes),
            Token::Struct => self.parse_struct_def(name, attributes),
            _ => Err(ParseError::new(
                self,
                format!(
                    "Expected 'proc' or 'struct' after '::', got {:?}",
                    self.current_token
                ),
            )),
        }
    }

    #[instrument(skip(self))]
    fn parse_proc_def(
        &mut self,
        name: String,
        attributes: Vec<Attribute>,
    ) -> ParseResult<Statement> {
        self.expect(Token::Proc)?;

        let params = if self.current_token == Token::LeftParen {
            self.advance();
            let params = self.parse_parameter_list()?;
            self.expect(Token::RightParen)?;
            params
        } else {
            Vec::new()
        };

        // Parse optional return type: -> type
        let return_type = if self.current_token == Token::Arrow {
            self.advance();
            if let Token::Identifier(t) = &self.current_token {
                let typ = t.clone();
                self.advance();
                Some(typ)
            } else {
                return Err(ParseError::new(
                    self,
                    format!(
                        "Expected return type after '->', got {:?}",
                        self.current_token
                    ),
                ));
            }
        } else {
            None
        };

        self.skip_newlines();
        self.expect(Token::LeftBrace)?;
        self.skip_newlines();

        let body = self.parse_statements()?;

        self.expect(Token::RightBrace)?;

        Ok(Statement::ProcDef(ProcDef {
            name,
            params,
            return_type,
            body,
            attributes,
        }))
    }

    #[instrument(skip(self))]
    fn parse_handler_def(
        &mut self,
        name: String,
        attributes: Vec<Attribute>,
    ) -> ParseResult<HandlerDef> {
        self.expect(Token::Handler)?;

        let params = if self.current_token == Token::LeftParen {
            self.advance();
            let params = self.parse_parameter_list()?;
            self.expect(Token::RightParen)?;
            params
        } else {
            Vec::new()
        };

        self.skip_newlines();
        self.expect(Token::LeftBrace)?;
        self.skip_newlines();

        let body = self.parse_statements()?;

        self.expect(Token::RightBrace)?;

        Ok(HandlerDef {
            name,
            params,
            body,
            attributes,
        })
    }

    #[instrument(skip(self))]
    fn parse_parameter_list(&mut self) -> ParseResult<Vec<Parameter>> {
        let mut params = Vec::new();

        if let Token::Identifier(name) = &self.current_token {
            let param_name = name.clone();
            self.advance();

            // Parse optional type annotation: name: type
            let param_type = if self.current_token == Token::Colon {
                self.advance();
                if let Token::Identifier(t) = &self.current_token {
                    let typ = t.clone();
                    self.advance();
                    Some(typ)
                } else {
                    return Err(ParseError::new(
                        self,
                        format!("Expected type after ':', got {:?}", self.current_token),
                    ));
                }
            } else {
                None
            };

            params.push(Parameter {
                name: param_name,
                typ: param_type,
            });

            while self.current_token == Token::Comma {
                self.advance();
                if let Token::Identifier(name) = &self.current_token {
                    let param_name = name.clone();
                    self.advance();

                    let param_type = if self.current_token == Token::Colon {
                        self.advance();
                        if let Token::Identifier(t) = &self.current_token {
                            let typ = t.clone();
                            self.advance();
                            Some(typ)
                        } else {
                            return Err(ParseError::new(
                                self,
                                format!("Expected type after ':', got {:?}", self.current_token),
                            ));
                        }
                    } else {
                        None
                    };

                    params.push(Parameter {
                        name: param_name,
                        typ: param_type,
                    });
                } else {
                    return Err(ParseError::new(
                        self,
                        format!("Expected parameter name, got {:?}", self.current_token),
                    ));
                }
            }
        }

        Ok(params)
    }

    #[instrument(skip(self))]
    fn parse_struct_def(
        &mut self,
        name: String,
        attributes: Vec<Attribute>,
    ) -> ParseResult<Statement> {
        self.expect(Token::Struct)?;
        self.skip_newlines();

        self.expect(Token::LeftBrace)?;
        self.skip_newlines();

        let fields = self.parse_fields()?;

        self.expect(Token::RightBrace)?;

        Ok(Statement::StructDef(StructDef {
            name,
            fields,
            attributes,
        }))
    }

    #[instrument(skip(self))]
    fn parse_if_statement(&mut self) -> ParseResult<Statement> {
        self.expect(Token::If)?;
        self.skip_newlines();

        let condition = self.parse_expression()?;
        self.skip_newlines();

        self.expect(Token::LeftBrace)?;
        self.skip_newlines();

        let then_block = self.parse_statements()?;

        self.expect(Token::RightBrace)?;
        self.skip_newlines();

        let elsif_blocks = Vec::new();
        let else_block = if self.current_token == Token::Else {
            self.advance();
            self.skip_newlines();

            self.expect(Token::LeftBrace)?;
            self.skip_newlines();

            let else_body = self.parse_statements()?;

            self.expect(Token::RightBrace)?;

            Some(else_body)
        } else {
            None
        };

        Ok(Statement::If(IfStatement {
            condition,
            then_block,
            elsif_blocks,
            else_block,
        }))
    }

    #[instrument(skip(self))]
    fn parse_import_statement(&mut self) -> ParseResult<Statement> {
        self.expect(Token::Import)?;

        let path = if let Token::String(s) = &self.current_token {
            let path = s.clone();
            self.advance();
            path
        } else {
            return Err(ParseError::new(
                self,
                format!("Expected string, got {:?}", self.current_token),
            ));
        };

        Ok(Statement::Import(path))
    }

    #[instrument(skip(self))]
    fn parse_colon_assignment(&mut self) -> ParseResult<Statement> {
        let name = if let Token::Identifier(n) = &self.current_token {
            let name = n.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::new(
                self,
                format!("Expected identifier, got {:?}", self.current_token),
            ));
        };

        self.expect(Token::ColonAssign)?;
        let value = self.parse_expression()?;

        Ok(Statement::ColonAssignment(name, value))
    }

    #[instrument(skip(self))]
    fn parse_defer_statement(&mut self) -> ParseResult<Statement> {
        self.expect(Token::Defer)?;
        let stmt = self.parse_statement()?;
        Ok(Statement::Defer(Box::new(stmt)))
    }

    #[instrument(skip(self))]
    fn parse_assignment(&mut self) -> ParseResult<Statement> {
        let name = if let Token::Identifier(n) = &self.current_token {
            let name = n.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::new(
                self,
                format!("Expected identifier, got {:?}", self.current_token),
            ));
        };

        self.expect(Token::Assign)?;
        let expr = self.parse_expression()?;

        Ok(Statement::Assignment(name, expr))
    }

    #[instrument(skip(self))]
    fn parse_for_statement(&mut self) -> ParseResult<Statement> {
        self.expect(Token::For)?;
        self.skip_newlines();

        let variable = if let Token::Identifier(name) = &self.current_token {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::new(
                self,
                format!(
                    "Expected variable name in for loop, got {:?}",
                    self.current_token
                ),
            ));
        };

        self.expect(Token::In)?;
        self.skip_newlines();

        let iterable = self.parse_expression()?;
        self.skip_newlines();

        self.expect(Token::LeftBrace)?;
        self.skip_newlines();

        let body = self.parse_statements()?;

        self.expect(Token::RightBrace)?;

        Ok(Statement::For(ForStatement {
            variable,
            iterable,
            body,
        }))
    }

    #[instrument(skip(self))]
    fn parse_return_statement(&mut self) -> ParseResult<Statement> {
        self.expect(Token::Return)?;

        let expr = match &self.current_token {
            Token::Newline | Token::Semicolon | Token::Eof | Token::RightBrace => None,
            _ => Some(self.parse_expression()?),
        };

        Ok(Statement::Return(expr))
    }

    #[instrument(skip(self))]
    fn parse_message(&mut self) -> ParseResult<Message> {
        let name = if let Token::Identifier(name) = &self.current_token {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::new(
                self,
                format!("Expected identifier after, got {:?}", self.current_token),
            ));
        };

        let mut args = Vec::new();
        if self.current_token == Token::LeftParen {
            self.advance();
            if self.current_token != Token::RightParen {
                args.push(self.parse_expression()?);
                while self.current_token == Token::Comma {
                    self.advance();
                    args.push(self.parse_expression()?);
                }
            }
        }

        self.expect(Token::RightParen)?;

        Ok(Message { name, args })
    }

    #[instrument(skip(self))]
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_logical_or()
    }

    #[instrument(skip(self))]
    fn parse_logical_or(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_logical_and()?;

        while self.current_token == Token::Or {
            self.advance();
            let right = self.parse_logical_and()?;
            left = Expression::Binary(BinaryOp::Or, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    #[instrument(skip(self))]
    fn parse_logical_and(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_bitwise_or()?;

        while self.current_token == Token::And {
            self.advance();
            let right = self.parse_bitwise_or()?;
            left = Expression::Binary(BinaryOp::And, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    #[instrument(skip(self))]
    fn parse_bitwise_or(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_bitwise_xor()?;

        while self.current_token == Token::BitwiseOr || self.current_token == Token::Pipe {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            left = Expression::Binary(BinaryOp::BitwiseOr, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    #[instrument(skip(self))]
    fn parse_bitwise_xor(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_bitwise_and()?;

        while self.current_token == Token::BitwiseXor {
            self.advance();
            let right = self.parse_bitwise_and()?;
            left = Expression::Binary(BinaryOp::BitwiseXor, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    #[instrument(skip(self))]
    fn parse_bitwise_and(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_equality()?;

        while self.current_token == Token::BitwiseAnd {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::Binary(BinaryOp::BitwiseAnd, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    #[instrument(skip(self))]
    fn parse_equality(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_comparison()?;

        loop {
            let op = match &self.current_token {
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                _ => break,
            };

            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    #[instrument(skip(self))]
    fn parse_comparison(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_shift()?;

        loop {
            let op = match &self.current_token {
                Token::LessThan => BinaryOp::LessThan,
                Token::LessEqual => BinaryOp::LessThanOrEqual,
                Token::GreaterThan => BinaryOp::GreaterThan,
                Token::GreaterEqual => BinaryOp::GreaterThanOrEqual,
                Token::Spaceship => BinaryOp::Spaceship,
                _ => break,
            };

            self.advance();
            let right = self.parse_shift()?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    #[instrument(skip(self))]
    fn parse_shift(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_range()?;

        loop {
            let op = match &self.current_token {
                Token::LeftShift => BinaryOp::LeftShift,
                Token::RightShift => BinaryOp::RightShift,
                _ => break,
            };

            self.advance();
            let right = self.parse_range()?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    #[instrument(skip(self))]
    fn parse_range(&mut self) -> ParseResult<Expression> {
        let left = self.parse_addition()?;

        match &self.current_token {
            Token::Range => {
                self.advance();
                let right = self.parse_addition()?;
                Ok(Expression::Range {
                    start: Box::new(left),
                    end: Box::new(right),
                    exclusive: false,
                })
            }
            Token::ExclusiveRange => {
                self.advance();
                let right = self.parse_addition()?;
                Ok(Expression::Range {
                    start: Box::new(left),
                    end: Box::new(right),
                    exclusive: true,
                })
            }
            _ => Ok(left),
        }
    }

    #[instrument(skip(self))]
    fn parse_addition(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_multiplication()?;

        loop {
            let op = match &self.current_token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                _ => break,
            };

            self.advance();
            let right = self.parse_multiplication()?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    #[instrument(skip(self))]
    fn parse_multiplication(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_power()?;

        loop {
            let op = match &self.current_token {
                Token::Star => BinaryOp::Multiply,
                Token::Slash => BinaryOp::Divide,
                Token::Percent => BinaryOp::Modulo,
                _ => break,
            };

            self.advance();
            let right = self.parse_power()?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    #[instrument(skip(self))]
    fn parse_power(&mut self) -> ParseResult<Expression> {
        let left = self.parse_unary()?;

        if self.current_token == Token::Power {
            self.advance();
            let right = self.parse_power()?;
            Ok(Expression::Binary(
                BinaryOp::Power,
                Box::new(left),
                Box::new(right),
            ))
        } else {
            Ok(left)
        }
    }

    #[instrument(skip(self))]
    fn parse_unary(&mut self) -> ParseResult<Expression> {
        match &self.current_token {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary(UnaryOp::Negate, Box::new(expr)))
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary(UnaryOp::Not, Box::new(expr)))
            }
            Token::BitwiseNot => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary(UnaryOp::BitwiseNot, Box::new(expr)))
            }
            _ => self.parse_postfix(),
        }
    }

    #[instrument(skip(self))]
    fn parse_postfix(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            match &self.current_token {
                Token::Dot => {
                    self.advance();
                    let method_name = if let Token::Identifier(name) = &self.current_token {
                        let name = name.clone();
                        self.advance();
                        name
                    } else {
                        return Err(ParseError::new(
                            self,
                            format!(
                                "Expected method name after '.', got {:?}",
                                self.current_token
                            ),
                        ));
                    };

                    let args = if self.current_token == Token::LeftParen {
                        self.advance();
                        let args = self.parse_argument_list()?;
                        self.expect(Token::RightParen)?;
                        args
                    } else {
                        Vec::new()
                    };

                    expr = Expression::Call {
                        method: method_name,
                        args,
                    };
                }
                Token::LeftBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(Token::RightBracket)?;
                    expr = Expression::Index {
                        expr: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    #[instrument(skip(self))]
    fn parse_primary(&mut self) -> ParseResult<Expression> {
        match &self.current_token.clone() {
            Token::Integer(n) => {
                let value = *n;
                self.advance();
                Ok(Expression::Integer(value))
            }
            Token::Float(n) => {
                let value = *n;
                self.advance();
                Ok(Expression::Float(value))
            }
            Token::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expression::String(value))
            }
            Token::InterpolatedString(parts) => {
                let parts_clone = parts.clone();
                self.advance();
                self.parse_interpolated_string(parts_clone)
            }
            Token::Symbol(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expression::Symbol(value))
            }
            Token::True => {
                self.advance();
                Ok(Expression::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(Expression::Boolean(false))
            }
            Token::Nil => {
                self.advance();
                Ok(Expression::Nil)
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();

                if self.current_token == Token::DoubleColon {
                    let mut namespace_parts = vec![name];

                    while self.current_token == Token::DoubleColon {
                        self.advance();

                        if let Token::Identifier(part) = &self.current_token {
                            namespace_parts.push(part.clone());
                            self.advance();
                        } else {
                            return Err(ParseError::new(
                                self,
                                format!(
                                    "Expected identifier after '::', got {:?}",
                                    self.current_token
                                ),
                            ));
                        }
                    }

                    if self.current_token == Token::LeftParen {
                        self.advance();
                        let args = self.parse_argument_list()?;
                        self.expect(Token::RightParen)?;

                        let last_part = namespace_parts.pop().unwrap();
                        return Ok(Expression::Call {
                            method: last_part,
                            args,
                        });
                    }

                    return Ok(Expression::NamespaceAccess(namespace_parts));
                }

                if self.current_token == Token::LeftParen {
                    self.advance();
                    let args = self.parse_argument_list()?;
                    self.expect(Token::RightParen)?;
                    Ok(Expression::Call { method: name, args })
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBracket => {
                self.advance();
                let elements = self.parse_argument_list()?;
                self.expect(Token::RightBracket)?;
                Ok(Expression::Array(elements))
            }
            Token::LeftBrace => {
                self.advance();
                let pairs = self.parse_hash_pairs()?;
                self.expect(Token::RightBrace)?;
                Ok(Expression::Hash(pairs))
            }

            _ => Err(ParseError::new(
                self,
                format!("Unexpected token in expression: {:?}", self.current_token),
            )),
        }
    }

    #[instrument(skip(self))]
    fn parse_argument_list(&mut self) -> ParseResult<Vec<Expression>> {
        let mut args = Vec::new();

        match &self.current_token {
            Token::RightParen | Token::RightBracket => return Ok(args),
            _ => {}
        }

        args.push(self.parse_expression()?);

        while self.current_token == Token::Comma {
            self.advance();
            args.push(self.parse_expression()?);
        }

        Ok(args)
    }

    #[instrument(skip(self))]
    fn parse_hash_pairs(&mut self) -> ParseResult<Vec<(Expression, Expression)>> {
        let mut pairs = Vec::new();

        if self.current_token == Token::RightBrace {
            return Ok(pairs);
        }

        loop {
            let key = self.parse_expression()?;
            self.expect(Token::Arrow)?;
            let value = self.parse_expression()?;
            pairs.push((key, value));

            if self.current_token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }

        Ok(pairs)
    }

    #[instrument(skip(self))]
    fn parse_interpolated_string(
        &mut self,
        parts: Vec<crate::lexer::InterpolationPart>,
    ) -> ParseResult<Expression> {
        use crate::ast::InterpolationPart as AstPart;
        use crate::lexer::InterpolationPart as LexerPart;

        let mut ast_parts = Vec::new();

        for part in parts {
            match part {
                LexerPart::Literal(s) => {
                    ast_parts.push(AstPart::Literal(s));
                }
                LexerPart::Expression(tokens) => {
                    let expr = self.parse_interpolation_tokens(tokens)?;
                    ast_parts.push(AstPart::Expression(Box::new(expr)));
                }
            }
        }

        Ok(Expression::StringInterpolation(ast_parts))
    }

    #[instrument(skip(self))]
    fn parse_interpolation_tokens(&mut self, tokens: Vec<Token>) -> ParseResult<Expression> {
        if tokens.is_empty() {
            return Err(ParseError::new(
                self,
                "Empty interpolation expression".to_string(),
            ));
        }

        let saved_tokens = std::mem::replace(&mut self.tokens, tokens);
        let saved_position = self.position;
        let saved_current = self.current_token.clone();

        self.position = 0;
        self.current_token = self.tokens.get(0).cloned().unwrap_or(Token::Eof);

        let result = self.parse_expression();

        self.tokens = saved_tokens;
        self.position = saved_position;
        self.current_token = saved_current;

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let mut parser = Parser::new("42");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::Expression(Expression::Integer(n)) => assert_eq!(*n, 42),
                    _ => panic!("Expected integer expression"),
                }
            }
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let mut parser = Parser::new("2 + 3 * 4");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
            }
        }
    }

    #[test]
    fn test_parse_assignment() {
        let mut parser = Parser::new("x = 42");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::Assignment(name, Expression::Integer(n)) => {
                        assert_eq!(name, "x");
                        assert_eq!(*n, 42);
                    }
                    _ => panic!("Expected assignment statement"),
                }
            }
        }
    }

    #[test]
    fn test_parse_method_def() {
        let mut parser = Parser::new("add :: proc(a: int, b: int) { a + b }");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::ProcDef(proc_def) => {
                        assert_eq!(proc_def.name, "add");
                        assert_eq!(proc_def.params.len(), 2);
                        assert_eq!(proc_def.params[0].name, "a");
                        assert_eq!(proc_def.params[1].name, "b");
                    }
                    _ => panic!("Expected proc definition"),
                }
            }
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let mut parser = Parser::new("if x > 0 { println(x) }");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::If(if_stmt) => {
                        assert_eq!(if_stmt.then_block.len(), 1);
                        assert_eq!(if_stmt.elsif_blocks.len(), 0);
                        assert!(if_stmt.else_block.is_none());
                    }
                    _ => panic!("Expected if statement"),
                }
            }
        }
    }

    #[test]
    fn test_parse_array() {
        let mut parser = Parser::new("[1, 2, 3]");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::Expression(Expression::Array(elements)) => {
                        assert_eq!(elements.len(), 3);
                    }
                    _ => panic!("Expected array expression"),
                }
            }
        }
    }

    #[test]
    fn test_parse_hash() {
        let mut parser = Parser::new("{:name => \"John\", :age => 30}");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::Expression(Expression::Hash(pairs)) => {
                        assert_eq!(pairs.len(), 2);
                    }
                    _ => panic!("Expected hash expression"),
                }
            }
        }
    }

    #[test]
    fn test_parse_method_call() {
        let mut parser = Parser::new("array.length()");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::Expression(Expression::Call { method, .. }) => {
                        assert_eq!(method, "length");
                    }
                    _ => panic!("Expected method call expression"),
                }
            }
        }
    }

    #[test]
    fn test_parse_attributes_on_method() {
        let mut parser = Parser::new("#[inline]\n#[test]\ndef foo()\n  42\nend");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::ProcDef(proc_def) => {
                        assert_eq!(proc_def.name, "foo");
                        assert_eq!(proc_def.attributes.len(), 2);
                        assert_eq!(proc_def.attributes[0].name, "inline");
                        assert_eq!(proc_def.attributes[1].name, "private");
                    }
                    _ => panic!("Expected proc definition with attributes"),
                }
            }
        }
    }

    #[test]
    fn test_parse_attributes_with_args() {
        let mut parser = Parser::new("#[deprecated(since=\"1.0\")]\nold_fn :: proc() { }");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::ProcDef(proc_def) => {
                        assert_eq!(proc_def.name, "old_fn");
                        assert_eq!(proc_def.attributes.len(), 1);
                        assert_eq!(proc_def.attributes[0].name, "deprecated");
                        assert_eq!(proc_def.attributes[0].args.len(), 1);
                        match &proc_def.attributes[0].args[0] {
                            AttributeArg::KeyValue(key, value) => {
                                assert_eq!(key, "since");
                                match &**value {
                                    AttributeArg::String(s) => assert_eq!(s, "1.0"),
                                    _ => panic!("Expected string value"),
                                }
                            }
                            _ => panic!("Expected key-value pair"),
                        }
                    }
                    _ => panic!("Expected proc definition"),
                }
            }
        }
    }

    #[test]
    fn test_parse_attributes_on_struct() {
        let mut parser = Parser::new("#[derive(Debug)]\nFoo :: struct { }");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::StructDef(struct_def) => {
                        assert_eq!(struct_def.name, "Foo");
                        assert_eq!(struct_def.attributes.len(), 1);
                        assert_eq!(struct_def.attributes[0].name, "derive");
                        assert_eq!(struct_def.attributes[0].args.len(), 1);
                    }
                    _ => panic!("Expected struct definition"),
                }
            }
        }
    }

    #[test]
    fn test_parse_import() {
        let code = r#"import "math""#;
        let mut parser = Parser::new(code);
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::Import(path) => {
                        assert_eq!(path, "math");
                    }
                    _ => panic!("Expected import statement"),
                }
            }
        }
    }

    #[test]
    fn test_parse_namespace_access() {
        let mut parser = Parser::new("Module::Class");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::Expression(Expression::NamespaceAccess(parts)) => {
                        assert_eq!(parts.len(), 2);
                        assert_eq!(parts[0], "Module");
                        assert_eq!(parts[1], "Class");
                    }
                    _ => panic!("Expected namespace access expression"),
                }
            }
        }
    }

    #[test]
    fn test_parse_nested_namespace() {
        let mut parser = Parser::new("App::Models::User");
        let result = parser.parse().unwrap();
        match result {
            Program::Statements(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    Statement::Expression(Expression::NamespaceAccess(parts)) => {
                        assert_eq!(parts.len(), 3);
                        assert_eq!(parts[0], "App");
                        assert_eq!(parts[1], "Models");
                        assert_eq!(parts[2], "User");
                    }
                    _ => panic!("Expected nested namespace access"),
                }
            }
        }
    }
}
