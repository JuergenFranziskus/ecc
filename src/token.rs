#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub at: SrcLoc<'a>,
    pub kind: TokenKind<'a>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind<'a> {
    CloseCurly,
    CloseParen,
    Eof,
    Error,
    Identifier(&'a str),
    Int,
    Octal(&'a str),
    OpenCurly,
    OpenParen,
    Return,
    Semicolon,
    Void,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SrcLoc<'a> {
    pub column: u32,
    pub file: &'a str,
    pub line: u32,
}
impl<'a> SrcLoc<'a> {
    pub fn new(file: &'a str, line: u32, column: u32) -> Self {
        Self { file, line, column }
    }

    pub fn next_column(&mut self, by: u32) {
        self.column += by;
    }
    pub fn next_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }
}
