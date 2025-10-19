use crate::{
    ast::*,
    token::{Token, TokenKind},
};

pub struct Parser<'a, 'b> {
    tokens: &'b [Token<'a>],
    index: usize,

    errors: Vec<ParseErr<'a>>,
}
impl<'a, 'b> Parser<'a, 'b> {
    pub fn new(tokens: &'b [Token<'a>]) -> Self {
        Self {
            tokens,
            index: 0,

            errors: Vec::new(),
        }
    }

    fn take(&mut self, kind: TokenKind<'a>) -> Res<()> {
        if self.is(kind) {
            self.err_expected(kind);
            return Err(());
        }

        self.next();
        Ok(())
    }
    fn next(&mut self) {
        self.index += 1;
    }
    fn is(&self, kind: TokenKind) -> bool {
        self.cur().kind == kind
    }
    fn cur(&self) -> Token<'a> {
        self.peek(0)
    }
    fn peek(&self, offset: usize) -> Token<'a> {
        let i = self.index + offset;
        self.tokens[i]
    }

    fn err_expected(&mut self, kind: TokenKind<'a>) {
        self.errors.push(ParseErr {
            at: self.cur(),
            kind: ParseErrKind::Expected(kind),
        })
    }
    fn err_expected_identifier(&mut self) {
        self.errors.push(ParseErr {
            at: self.cur(),
            kind: ParseErrKind::ExpectedIdentifier,
        })
    }
}

type Res<T> = Result<T, ()>;

pub struct ParseErr<'a> {
    pub at: Token<'a>,
    pub kind: ParseErrKind<'a>,
}

pub enum ParseErrKind<'a> {
    Expected(TokenKind<'a>),
    ExpectedIdentifier,
}
