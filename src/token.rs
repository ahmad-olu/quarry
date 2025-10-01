use std::{collections::VecDeque, fmt};

use miette::{Error, LabeledSpan, SourceSpan};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'de> {
    pub origin: &'de str,
    pub offset: usize,
    pub kind: TokenKind<'de>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    String(&'de str),
    InterpolatedStart,
    InterpolatedEnd,
    Number(f64),
    Nil,
    True,
    False,

    // ======================
    // Structural tokens
    // ======================
    LeftBrace,
    RightBrace,
    Colon,
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

// impl fmt::Display for Token<'_> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match &self.kind {
//             TokenKind::String(literal) => write!(f, "String {:?} {:?}", self.origin, *literal),
//             TokenKind::InterpolatedString(literal) => {
//                 write!(f, "String {:?} {:?}", self.origin, *literal)
//             }
//             TokenKind::Number(literal) => write!(f, "Number {:?} {:?}", self.origin, literal),
//             _ => write!(f, "{:?} {:?}", self.kind, self.origin),
//         }
//     }
// }

pub struct Lexer<'de> {
    whole: &'de str,
    rest: &'de str,
    byte: usize,
    pending_string: VecDeque<Token<'de>>, // <- buffered tokens
}

impl<'de> Lexer<'de> {
    pub fn new(input: &'de str) -> Self {
        Self {
            whole: input,
            rest: input,
            byte: 0,
            pending_string: VecDeque::new(),
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

        //println!("==> {}  ====> {}", c_origin, self.byte);
        // Some(Ok(Token {
        //     offset: c_offset,
        //     origin: c_origin,
        //     kind: TokenKind::RightBrace,
        // }))

        enum StartsWith {
            String,
            // Number,
            Ident,
            Slash,
            Bang,
            Equal,
            // BangEqual,
            // EqualEqual,
            // LessEqual,
            // GreaterEqual,
            // Less,
            // Greater,
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
            '/' => StartsWith::Slash,
            '!' => StartsWith::Bang,
            '=' => StartsWith::Equal,
            '"' => StartsWith::String,
            'a'..='z' | 'A'..='Z' | '_' => StartsWith::Ident,
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
                    // println!("====> {:?}", a);
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
                    .find(|c| !matches!(c, 'a'..='z'|'A'..='Z' | '0'..='9'| '_'))
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
                if let Some(end_quote) = self.rest.find('"') {
                    todo!()
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
        }
    }
}
