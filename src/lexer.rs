use miette::{Error, LabeledSpan};

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'de> {
    pub origin: &'de str,
    pub offset: usize,
    pub kind: TokenKind<'de>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'de> {
    // ======================
    // Keywords
    // ======================
    Let,
    Import,
    Json,
    Multipart,
    Raw,
    Form,
    Save,
    Assert,
    Matches,
    In,
    Contains,
    Test,
    Graphql,
    Ws, // websocket

    // ======================
    // HTTP Methods
    // ======================
    Get,
    Post,
    Put,
    Delete,

    // ======================
    // Identifiers & Names
    // ======================
    Name,  // request name: e.g. `name: getUsers`
    Ident, // generic identifier (variables, headers, etc.)

    // ======================
    // Literals
    // ======================
    TemplateLiteral {
        content: &'de str,        // the full content including interpolations
        parts: Vec<TemplatePart>, // parsed parts
    },
    Double(f64),
    Integer(i64),
    Nil,
    True,
    False,

    // ======================
    // Structural tokens
    // ======================
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
    Newline,
    Eof, // end-of-file

    // ======================
    // Operators
    // ======================
    Bang,      // !
    Equal,     // =
    BangEqual, // !=
    EqualEqual, // ==
               // LessEqual,    // <=
               // GreaterEqual, // >=
               // Less,         // <
               // Greater,      // >
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Literal(String),
    Interpolation(String),
}

pub struct Lexer<'de> {
    whole: &'de str,
    rest: &'de str,
    byte: usize,
}

impl<'de> Lexer<'de> {
    pub fn new(input: &'de str) -> Self {
        Self {
            whole: input,
            rest: input,
            byte: 0,
        }
    }
}

impl<'de> Iterator for Lexer<'de> {
    type Item = Result<Token<'de>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.rest.chars();
        let c = chars.next()?;
        let c_offset = self.byte; //c_at
        let c_origin = &self.rest[..c.len_utf8()]; //c_str
        let c_onwards = self.rest;

        // println!("==> {}", c_onwards);

        self.rest = chars.as_str();
        self.byte += c.len_utf8();

        enum StartsWith {
            String,
            Number,
            Ident,
            Slash,
            Bang,
            Equal,
        }

        let structural_token_return =
            move |kind: TokenKind<'de>| -> Option<Result<Token<'de>, Error>> {
                Some(Ok(Token {
                    kind,
                    offset: c_offset,
                    origin: c_origin,
                }))
            };

        let starts_with = match c {
            '{' => return structural_token_return(TokenKind::LeftBrace),
            '}' => return structural_token_return(TokenKind::RightBrace),
            ':' => return structural_token_return(TokenKind::Colon),
            ',' => return structural_token_return(TokenKind::Comma),
            '/' => StartsWith::Slash,
            '!' => StartsWith::Bang,
            '=' => StartsWith::Equal,
            // '_'=>
            '`' | '"' | '\'' => StartsWith::String,
            'a'..='z' | 'A'..='Z' => StartsWith::Ident,
            '0'..='9' => StartsWith::Number,
            '-' => {
                if matches!(&c_onwards[1..2].chars().nth(0)?, '0'..='9') {
                    let first_non_digit = c_onwards[1..]
                        .find(|c| !matches!(c, '.' | '0'..='9'))
                        .map(|i| i + 1) // add back the starting offset (1)
                        .unwrap_or_else(|| c_onwards.len());
                    return self.lex_number_literal(c_onwards, c_offset, first_non_digit);
                }

                return Some(Err(miette::miette! {
                    labels = vec![
                        LabeledSpan::at(self.byte - c.len_utf8()..self.byte, "this character"),
                    ],
                    "token `{c}` can't standalone",
                }
                .with_source_code(self.whole.to_string())));
            }
            c if c == '\n' => {
                if self.rest.starts_with('\n') {
                    return self.next();
                }

                return structural_token_return(TokenKind::Newline);
            }
            c if c.is_whitespace() => return self.next(),
            c => {
                return Some(Err(miette::miette! {
                    labels = vec![
                        LabeledSpan::at(self.byte - c.len_utf8()..self.byte, "this character"),
                    ],
                    "Unexpected token `{c}` in input",
                }
                .with_source_code(self.whole.to_string())));
            }
        };

        match starts_with {
            StartsWith::Bang => {
                //if let Some('=') = self.rest.chars().next() {}
                //if matches!(self.rest.chars().next(), Some('=')) {}

                if self.rest.starts_with('=') {
                    self.byte += 1;
                    self.rest = &self.rest[1..];

                    return Some(Ok(Token {
                        origin: &c_onwards[..2],
                        offset: c_offset,
                        kind: TokenKind::BangEqual,
                    }));
                }

                Some(Ok(Token {
                    origin: c_origin,
                    offset: c_offset,
                    kind: TokenKind::Bang,
                }))
            }
            StartsWith::Equal => {
                if self.rest.starts_with('=') {
                    self.byte += 1;
                    self.rest = &self.rest[1..];

                    return Some(Ok(Token {
                        origin: &c_onwards[..2],
                        offset: c_offset,
                        kind: TokenKind::EqualEqual,
                    }));
                }

                Some(Ok(Token {
                    origin: c_origin,
                    offset: c_offset,
                    kind: TokenKind::Equal,
                }))
            }
            StartsWith::Slash => {
                if self.rest.starts_with("/") {
                    let line_end = self.rest.find('\n').unwrap_or_else(|| self.rest.len());
                    self.byte = line_end;
                    self.rest = &self.rest[line_end..];
                    return self.next();
                }
                return self.next();
            }
            StartsWith::Ident => {
                let first_non_ident = c_onwards
                    .find(|c| !matches!(c, 'a'..='z'|'A'..='Z' | '0'..='9'))
                    .unwrap_or_else(|| c_onwards.len());
                let literal = &c_onwards[..first_non_ident];
                let extra_bytes = literal.len() - c.len_utf8();
                self.byte += extra_bytes;
                self.rest = &self.rest[extra_bytes..];

                let kind = match literal {
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    "nil" => TokenKind::Nil,
                    "let" => TokenKind::Let,
                    "import" => TokenKind::Import,
                    "json" => TokenKind::Json,
                    "multipart" => TokenKind::Multipart,
                    "raw" => TokenKind::Raw,
                    "form" => TokenKind::Form,
                    "save" => TokenKind::Save,
                    "assert" => TokenKind::Assert,
                    "matches" => TokenKind::Matches,
                    "in" => TokenKind::In,
                    "contains" => TokenKind::Contains,
                    "test" => TokenKind::Test,
                    "get" => TokenKind::Get,
                    "post" => TokenKind::Post,
                    "put" => TokenKind::Put,
                    "delete" => TokenKind::Delete,
                    //Todo: add other request method
                    "name" => TokenKind::Name,
                    "graphql" => TokenKind::Graphql,
                    "ws" => TokenKind::Ws,
                    _ => TokenKind::Ident,
                };

                return Some(Ok(Token {
                    origin: literal,
                    offset: c_offset,
                    kind: kind,
                }));
            }
            StartsWith::String => {
                if c == '`' {
                    return self.lex_template_literal(c_offset, '`');
                } else if c == '"' {
                    return self.lex_template_literal(c_offset, '"');
                } else if c == '\'' {
                    return self.lex_template_literal(c_offset, '\'');
                } else {
                    let err = Some(Err(miette::miette! {
                        labels = vec![
                            LabeledSpan::new_with_span(
                                Some("string termination error".to_string()),
                                (self.byte - c.len_utf8()..self.whole.len()),
                            )
                        ],
                        "expected terminator `\"`",
                    }
                    .with_source_code(self.whole.to_string())));

                    self.byte += self.rest.len();
                    self.rest = &self.rest[self.rest.len()..];

                    return err;
                }
            }
            StartsWith::Number => {
                let first_non_digit = c_onwards
                    .find(|c| !matches!(c, '.' | '0'..='9'))
                    .unwrap_or_else(|| c_onwards.len());
                return self.lex_number_literal(c_onwards, c_offset, first_non_digit);
            }
        }
    }
}

impl<'de> Lexer<'de> {
    fn lex_template_literal(
        &mut self,
        start_offset: usize,
        terminating_char: char,
    ) -> Option<Result<Token<'de>, Error>> {
        let start = self.byte;
        let mut parts = Vec::new();
        let mut current_literal = String::new();

        loop {
            if self.rest.is_empty() {
                return Some(Err(miette::miette! {
                    labels = vec![
                        LabeledSpan::new_with_span(
                            Some("template literal termination error".to_string()),
                            start_offset..self.whole.len() ,
                        )
                    ],
                    "expected terminator `{}`",
                    terminating_char
                }
                .with_source_code(self.whole.to_string())));
            }

            let mut chars = self.rest.chars();
            let ch = chars.next().unwrap();

            match ch {
                c if c == terminating_char => {
                    self.rest = chars.as_str();
                    self.byte += ch.len_utf8();

                    // Add remaining literal if any
                    if !current_literal.is_empty() {
                        parts.push(TemplatePart::Literal(current_literal));
                    }

                    let content = &self.whole[start..self.byte - 1]; // exclude the closing backtick
                    return Some(Ok(Token {
                        origin: &self.whole[start_offset..self.byte],
                        offset: start_offset,
                        kind: TokenKind::TemplateLiteral { content, parts },
                    }));
                }
                '$' if chars.as_str().starts_with('{') => {
                    self.rest = chars.as_str();
                    self.byte += ch.len_utf8();

                    self.rest = &self.rest[1..];
                    self.byte += 1;

                    if !current_literal.is_empty() {
                        parts.push(TemplatePart::Literal(current_literal.clone()));
                        current_literal.clear();
                    }

                    match self.extract_interpolation() {
                        Ok(expr) => {
                            parts.push(TemplatePart::Interpolation(expr.to_string()));
                        }
                        Err(e) => return Some(Err(e)),
                    }
                }
                _ => {
                    current_literal.push(ch);
                    self.rest = chars.as_str();
                    self.byte += ch.len_utf8();
                }
            }
        }
    }

    fn extract_interpolation(&mut self) -> Result<&'de str, miette::Report> {
        let start = self.byte;
        let mut brace_depth = 1;

        while brace_depth > 0 {
            if self.rest.is_empty() {
                return Err(miette::miette! {
                    labels = vec![
                        LabeledSpan::new_with_span(
                            Some("unclosed interpolation".to_string()),
                            start - 2..self.whole.len() , // -2 to include ${
                        )
                    ],
                    r"expected closing `}}` for interpolation",
                }
                .with_source_code(self.whole.to_string()));
            }

            let mut chars = self.rest.chars();
            let ch = chars.next().unwrap();

            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }

            self.rest = chars.as_str();
            self.byte += ch.len_utf8();
        }

        // Extract the expression (excluding the closing brace)
        let expr = &self.whole[start..self.byte - 1];
        Ok(expr)
    }

    fn lex_number_literal(
        &mut self,
        c_onwards: &'de str,
        c_offset: usize,
        first_non_digit: usize,
    ) -> Option<Result<Token<'de>, Error>> {
        let literal = &c_onwards[..first_non_digit];
        let is_float = literal.contains('.');

        let kind = if is_float {
            match literal.parse::<f64>() {
                Ok(val) => TokenKind::Double(val),
                Err(e) => {
                    return Some(Err(miette::miette! {
                        labels = vec![
                            LabeledSpan::at(
                                self.byte - literal.len()..self.byte,
                                "this numeric literal"
                            ),
                        ],
                        "{e}",
                    }
                    .with_source_code(self.whole.to_string())));
                }
            }
        } else {
            match literal.parse::<i64>() {
                Ok(val) => TokenKind::Integer(val),
                Err(e) => {
                    return Some(Err(miette::miette! {
                        labels = vec![
                            LabeledSpan::at(
                                self.byte - literal.len()..self.byte,
                                "this numeric literal"
                            ),
                        ],
                        "{e}",
                    }
                    .with_source_code(self.whole.to_string())));
                }
            }
        };

        self.byte += literal.len();
        self.rest = &self.rest[literal.len()..];

        Some(Ok(Token {
            origin: c_onwards,
            offset: c_offset,
            kind,
        }))
    }
}
