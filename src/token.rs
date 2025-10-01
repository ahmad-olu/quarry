use std::fmt;

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
    InterpolatedString(&'de str),
    Number(f64),
    Null,
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
    Bang,         // !
    Equal,        // =
    BangEqual,    // !=
    EqualEqual,   // ==
    LessEqual,    // <=
    GreaterEqual, // >=
    Less,         // <
    Greater,      // >
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TokenKind::String(literal) => write!(f, "String {:?} {:?}", self.origin, *literal),
            TokenKind::InterpolatedString(literal) => {
                write!(f, "String {:?} {:?}", self.origin, *literal)
            }
            TokenKind::Number(literal) => write!(f, "Number {:?} {:?}", self.origin, literal),
            _ => write!(f, "{:?} {:?}", self.kind, self.origin),
        }
    }
}
