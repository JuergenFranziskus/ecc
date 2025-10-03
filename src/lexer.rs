use crate::token::{SrcLoc, Token, TokenKind};

pub struct Lexer<'a> {
    src: &'a str,
    index: usize,
    at: SrcLoc<'a>,
}
impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            index: 0,
            at: SrcLoc::new("<dummy file; this should never appear; lexer bug>", 1, 1),
        }
    }

    pub fn lex(mut self) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();

        while !self.is_eof() {
            let Some(token) = self.lex_next() else {
                continue;
            };
            tokens.push(token);
        }

        tokens.push(Token {
            at: SrcLoc::new("<EOF>", 1, 1),
            kind: TokenKind::Eof,
        });

        tokens
    }
    fn lex_next(&mut self) -> Option<Token<'a>> {
        let at = self.at;

        if self.cur() == '\n' {
            self.next();
            self.at.next_line();
            None
        } else if self.cur().is_whitespace() {
            self.next();
            None
        } else if self.cur() == '#' {
            self.process_line_directive();
            None
        } else if self.cur() == '(' {
            self.next();
            Some(Token {
                at,
                kind: TokenKind::OpenParen,
            })
        } else if self.cur() == ')' {
            self.next();
            Some(Token {
                at,
                kind: TokenKind::CloseParen,
            })
        } else if self.cur() == '{' {
            self.next();
            Some(Token {
                at,
                kind: TokenKind::OpenCurly,
            })
        } else if self.cur() == '}' {
            self.next();
            Some(Token {
                at,
                kind: TokenKind::CloseCurly,
            })
        } else if self.cur() == ';' {
            self.next();
            Some(Token {
                at,
                kind: TokenKind::Semicolon,
            })
        } else if self.matches("int") {
            self.advance(3);
            Some(Token {
                at,
                kind: TokenKind::Int,
            })
        } else if self.matches("void") {
            self.advance(4);
            Some(Token {
                at,
                kind: TokenKind::Void,
            })
        } else if self.matches("return") {
            self.advance(6);
            Some(Token {
                at,
                kind: TokenKind::Return,
            })
        } else if self.cur() == '0' {
            let start = self.index;
            while self.cur().is_ascii_digit() || self.cur() == '\'' {
                self.next();
            }
            let end = self.index;
            let literal = &self.src[start..end];
            Some(Token {
                at,
                kind: TokenKind::Octal(literal),
            })
        } else if self.cur().is_ascii_alphabetic() || self.cur() == '_' {
            let start = self.index;
            while self.cur().is_ascii_alphanumeric() || self.cur() == '_' {
                self.next();
            }
            let end = self.index;
            let identifier = &self.src[start..end];
            Some(Token {
                at,
                kind: TokenKind::Identifier(identifier),
            })
        } else {
            self.next();
            Some(Token {
                at,
                kind: TokenKind::Error,
            })
        }
    }
    fn process_line_directive(&mut self) {
        self.take('#');
        self.take(' ');
        let line = self.src[self.index..].split_whitespace().next().unwrap();
        self.index += line.len();
        self.take(' ');
        self.take('"');
        let file = self.src[self.index..].split('"').next().unwrap();
        self.index += file.len();
        self.take('"');
        let rest_line = self.src[self.index..].split('\n').next().unwrap();
        self.index += rest_line.len();
        self.take('\n');

        let line: u32 = line.parse().unwrap();
        self.at = SrcLoc::new(file, line, 1);
    }

    fn matches(&self, pattern: &str) -> bool {
        self.src[self.index..].starts_with(pattern)
    }
    fn take(&mut self, c: char) {
        assert_eq!(self.cur(), c);
        self.next();
    }
    fn next(&mut self) {
        self.advance(1);
    }
    fn advance(&mut self, by: usize) {
        let bytes: usize = self.src[self.index..]
            .chars()
            .take(by)
            .map(|c| c.len_utf8())
            .sum();
        self.index += bytes;
        self.at.next_column(by as u32);
    }
    fn cur(&self) -> char {
        self.peek(0)
    }
    fn peek(&self, offset: usize) -> char {
        self.src[self.index..].chars().skip(offset).next().unwrap()
    }
    fn is_eof(&self) -> bool {
        self.index >= self.src.len()
    }
}
