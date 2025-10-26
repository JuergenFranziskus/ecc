use crate::token::{
    At, Files, IntegerFormat, IntegerSuffix, IntegerToken, StringEncoding, Token, TokenKind,
};

pub struct Lexer<'a> {
    src: &'a str,
    index: usize,
    at: At,
    files: Files,
}
impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut files = Files::new();
        let dummy_file = files.get_file_id("<dummy file; this should never appear; lexer bug>");

        Self {
            src,
            index: 0,
            at: At::new(dummy_file, 1, 1),
            files,
        }
    }

    pub fn lex(mut self) -> (Vec<Token<'a>>, Files) {
        let mut tokens = Vec::new();

        while !self.is_eof() {
            let Some(token) = self.lex_next() else {
                continue;
            };
            tokens.push(token);
        }

        let eof_file = self.files.get_file_id("<EOF>");
        tokens.push(Token {
            at: At::new(eof_file, 1, 1),
            kind: TokenKind::Eof,
        });

        (tokens, self.files)
    }
    fn lex_next(&mut self) -> Option<Token<'a>> {
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
        } else {
            Some(self.lex_token())
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
        let file = self.files.get_file_id(file);
        self.at = At::new(file, line, 1);
    }
    fn lex_token(&mut self) -> Token<'a> {
        let at = self.at;

        for &(pattern, kind) in TOKEN_MAP {
            if self.matches(pattern) {
                let length = pattern.chars().count();
                self.advance(length);
                return Token { at, kind };
            }
        }

        if self.is_string_literal() {
            self.lex_string_literal()
        } else if self.matches("0x") || self.matches("0X") && self.peek(2).is_ascii_hexdigit() {
            self.lex_hexadecimal_literal()
        } else if self.matches("0b")
            || self.matches("0B") && self.peek(2) == '0'
            || self.peek(2) == '1'
        {
            self.lex_binary_literal()
        } else if self.matches("0") {
            self.lex_octal_literal()
        } else if self.cur().is_ascii_digit() {
            self.lex_decimal_literal()
        } else if self.cur().is_ascii_alphabetic() {
            self.lex_identifier()
        } else {
            self.next();
            Token {
                at,
                kind: TokenKind::Error,
            }
        }
    }
    fn lex_string_literal(&mut self) -> Token<'a> {
        let encoding = self.lex_string_encoding();

        let at = self.at;
        self.next();
        let start = self.index;
        loop {
            if self.matches("\\\"") {
                self.advance(2);
            } else if self.matches("\"") {
                break;
            } else {
                self.next();
            }
        }
        let end = self.index;
        self.next();

        let src = &self.src[start..end];

        Token {
            at,
            kind: TokenKind::String(src, encoding),
        }
    }
    fn lex_hexadecimal_literal(&mut self) -> Token<'a> {
        let at = self.at;
        self.advance(2);
        let start = self.index;
        while self.cur().is_ascii_hexdigit() || self.cur() == '\'' {
            self.next();
        }

        let end = self.index;
        let src = &self.src[start..end];

        let suffix = self.lex_integer_suffix();

        Token {
            at,
            kind: TokenKind::Integer(IntegerToken {
                source: src,
                format: IntegerFormat::Hexadecimal,
                suffix,
            }),
        }
    }
    fn lex_binary_literal(&mut self) -> Token<'a> {
        let at = self.at;
        self.advance(2);
        let start = self.index;
        while is_binary_digit(self.cur()) || self.cur() == '\'' {
            self.next();
        }

        let end = self.index;
        let src = &self.src[start..end];

        let suffix = self.lex_integer_suffix();

        Token {
            at,
            kind: TokenKind::Integer(IntegerToken {
                source: src,
                format: IntegerFormat::Binary,
                suffix,
            }),
        }
    }
    fn lex_octal_literal(&mut self) -> Token<'a> {
        let at = self.at;
        let start = self.index;
        while is_octal_digit(self.cur()) || self.cur() == '\'' {
            self.next();
        }

        let end = self.index;
        let src = &self.src[start..end];

        let suffix = self.lex_integer_suffix();

        Token {
            at,
            kind: TokenKind::Integer(IntegerToken {
                source: src,
                format: IntegerFormat::Octal,
                suffix,
            }),
        }
    }
    fn lex_decimal_literal(&mut self) -> Token<'a> {
        let at = self.at;
        let start = self.index;
        while self.cur().is_ascii_digit() || self.cur() == '\'' {
            self.next();
        }

        let end = self.index;
        let src = &self.src[start..end];

        let suffix = self.lex_integer_suffix();

        Token {
            at,
            kind: TokenKind::Integer(IntegerToken {
                source: src,
                format: IntegerFormat::Decimal,
                suffix,
            }),
        }
    }
    fn lex_identifier(&mut self) -> Token<'a> {
        let at = self.at;
        let start = self.index;
        while self.cur().is_ascii_alphanumeric() || self.cur() == '_' {
            self.next();
        }

        let end = self.index;
        let src = &self.src[start..end];

        Token {
            at,
            kind: TokenKind::Identifier(src),
        }
    }

    fn lex_integer_suffix(&mut self) -> Option<IntegerSuffix> {
        if self.matches("u") || self.matches("U") {
            self.next();
            Some(IntegerSuffix::Unsigned)
        } else if self.matches("ul")
            || self.matches("UL")
            || self.matches("uL")
            || self.matches("Ul")
        {
            self.advance(2);
            Some(IntegerSuffix::LongUnsigned)
        } else if self.matches("ull")
            || self.matches("ULL")
            || self.matches("uLL")
            || self.matches("Ull")
        {
            self.advance(3);
            Some(IntegerSuffix::LongLongUnsigned)
        } else if self.matches("uwb")
            || self.matches("UWB")
            || self.matches("uWB")
            || self.matches("Uwb")
        {
            self.advance(3);
            Some(IntegerSuffix::BitPreciseUnsigned)
        } else if self.matches("l") || self.matches("L") {
            self.next();
            Some(IntegerSuffix::Long)
        } else if self.matches("lu")
            || self.matches("LU")
            || self.matches("lU")
            || self.matches("Lu")
        {
            self.advance(2);
            Some(IntegerSuffix::LongUnsigned)
        } else if self.matches("ll") || self.matches("LL") {
            self.advance(2);
            Some(IntegerSuffix::LongLong)
        } else if self.matches("llu")
            || self.matches("LLU")
            || self.matches("llU")
            || self.matches("LLu")
        {
            self.advance(3);
            Some(IntegerSuffix::LongLongUnsigned)
        } else if self.matches("wb") || self.matches("WB") {
            self.advance(2);
            Some(IntegerSuffix::BitPrecise)
        } else if self.matches("wbu")
            || self.matches("WBU")
            || self.matches("wbU")
            || self.matches("WBu")
        {
            self.advance(3);
            Some(IntegerSuffix::BitPreciseUnsigned)
        } else {
            None
        }
    }
    fn is_string_literal(&self) -> bool {
        self.matches("\"")
            || self.matches("u8\"")
            || self.matches("u\"")
            || self.matches("U\"")
            || self.matches("L\"")
    }
    fn lex_string_encoding(&mut self) -> StringEncoding {
        if self.matches("u8") {
            StringEncoding::UTF8
        } else if self.matches("u") {
            StringEncoding::UTF16
        } else if self.matches("U") {
            StringEncoding::UTF32
        } else if self.matches("L") {
            StringEncoding::Wide
        } else {
            StringEncoding::None
        }
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

fn is_octal_digit(c: char) -> bool {
    matches!(c, '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7')
}
fn is_binary_digit(c: char) -> bool {
    matches!(c, '0' | '1')
}

static TOKEN_MAP: &[(&'static str, TokenKind)] = &[
    ("...", TokenKind::Ellipses),
    ("<<=", TokenKind::DoubleLessEqual),
    (">>=", TokenKind::DoubleGreaterEqual),
    ("->", TokenKind::ArrowLeft),
    ("++", TokenKind::DoublePlus),
    ("--", TokenKind::DoubleMinus),
    ("<<", TokenKind::DoubleLess),
    (">>", TokenKind::DoubleGreater),
    ("<=", TokenKind::LessEqual),
    (">=", TokenKind::GreaterEqual),
    ("==", TokenKind::DoubleEqual),
    ("!=", TokenKind::NotEqual),
    ("&&", TokenKind::DoubleAmpersand),
    ("||", TokenKind::DoubleBar),
    ("::", TokenKind::DoubleColon),
    ("*=", TokenKind::AsteriskEqual),
    ("/=", TokenKind::SlashEqual),
    ("%=", TokenKind::PercentEqual),
    ("+=", TokenKind::PlusEqual),
    ("-=", TokenKind::MinusEqual),
    ("&=", TokenKind::Ampersand),
    ("^=", TokenKind::CaretEqual),
    ("|=", TokenKind::BarEqual),
    ("[", TokenKind::OpenBracket),
    ("]", TokenKind::CloseBracket),
    ("(", TokenKind::OpenParenthesis),
    (")", TokenKind::CloseParenthesis),
    ("{", TokenKind::OpenBrace),
    ("}", TokenKind::CloseBrace),
    (".", TokenKind::Period),
    ("&", TokenKind::Ampersand),
    ("*", TokenKind::Asterisk),
    ("+", TokenKind::Plus),
    ("-", TokenKind::Minus),
    ("~", TokenKind::Tilde),
    ("!", TokenKind::Exclamation),
    ("/", TokenKind::Slash),
    ("%", TokenKind::Percent),
    ("<", TokenKind::Less),
    (">", TokenKind::Greater),
    ("^", TokenKind::Caret),
    ("|", TokenKind::Bar),
    ("?", TokenKind::Question),
    (":", TokenKind::Colon),
    (";", TokenKind::Semicolon),
    ("=", TokenKind::Equal),
    (",", TokenKind::Comma),
    ("alignas", TokenKind::Alignas),
    ("alignof", TokenKind::Alignof),
    ("auto", TokenKind::Auto),
    ("bool", TokenKind::Bool),
    ("break", TokenKind::Break),
    ("case", TokenKind::Case),
    ("char", TokenKind::Char),
    ("const", TokenKind::Const),
    ("constexpr", TokenKind::Constexpr),
    ("continue", TokenKind::Continue),
    ("default", TokenKind::Default),
    ("do", TokenKind::Do),
    ("double", TokenKind::Double),
    ("else", TokenKind::Else),
    ("enum", TokenKind::Enum),
    ("extern", TokenKind::Extern),
    ("false", TokenKind::False),
    ("float", TokenKind::Float),
    ("for", TokenKind::For),
    ("goto", TokenKind::Goto),
    ("if", TokenKind::If),
    ("inline", TokenKind::Inline),
    ("int", TokenKind::Int),
    ("long", TokenKind::Long),
    ("nullptr", TokenKind::Nullptr),
    ("register", TokenKind::Register),
    ("restrict", TokenKind::Restrict),
    ("return", TokenKind::Return),
    ("short", TokenKind::Short),
    ("signed", TokenKind::Signed),
    ("sizeof", TokenKind::Sizeof),
    ("static", TokenKind::Static),
    ("static_assert", TokenKind::StaticAssert),
    ("struct", TokenKind::Struct),
    ("switch", TokenKind::Switch),
    ("thread_local", TokenKind::ThreadLocal),
    ("true", TokenKind::True),
    ("typedef", TokenKind::Typedef),
    ("typeof", TokenKind::Typeof),
    ("typeof_unqual", TokenKind::TypeofUnqual),
    ("union", TokenKind::Union),
    ("unsigned", TokenKind::Unsigned),
    ("void", TokenKind::Void),
    ("volatile", TokenKind::Volatile),
    ("while", TokenKind::While),
    ("_Atomic", TokenKind::Atomic),
    ("_BitInt", TokenKind::BitInt),
    ("_Complex", TokenKind::Complex),
    ("_Decimal128", TokenKind::Decimal128),
    ("_Decimal32", TokenKind::Decimal32),
    ("_Decimal64", TokenKind::Decimal64),
    ("_Generic", TokenKind::Generic),
    ("_Imaginary", TokenKind::Imaginary),
    ("_Noreturn", TokenKind::Noreturn),
];
