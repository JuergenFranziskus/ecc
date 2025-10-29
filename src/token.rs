use std::ops::Index;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub at: At,
    pub kind: TokenKind<'a>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind<'a> {
    Identifier(&'a str),
    Integer(IntegerToken<'a>),
    String(&'a str, StringEncoding),

    OpenBracket,
    CloseBracket,
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    Period,
    ArrowLeft,
    DoublePlus,
    DoubleMinus,
    Ampersand,
    Asterisk,
    Plus,
    Minus,
    Tilde,
    Exclamation,
    Slash,
    Percent,
    DoubleLess,
    DoubleGreater,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    DoubleEqual,
    NotEqual,
    Caret,
    Bar,
    DoubleAmpersand,
    DoubleBar,
    Question,
    Colon,
    DoubleColon,
    Semicolon,
    Ellipses,
    Equal,
    AsteriskEqual,
    SlashEqual,
    PercentEqual,
    PlusEqual,
    MinusEqual,
    DoubleLessEqual,
    DoubleGreaterEqual,
    AmpersandEqual,
    CaretEqual,
    BarEqual,
    Comma,

    Alignas,
    Alignof,
    Auto,
    Bool,
    Break,
    Case,
    Char,
    Const,
    Constexpr,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extern,
    False,
    Float,
    For,
    Goto,
    If,
    Inline,
    Int,
    Long,
    Nullptr,
    Register,
    Restrict,
    Return,
    Short,
    Signed,
    Sizeof,
    Static,
    StaticAssert,
    Struct,
    Switch,
    ThreadLocal,
    True,
    Typedef,
    Typeof,
    TypeofUnqual,
    Union,
    Unsigned,
    Void,
    Volatile,
    While,
    Atomic,
    BitInt,
    Complex,
    Decimal128,
    Decimal32,
    Decimal64,
    Generic,
    Imaginary,
    Noreturn,

    Eof,
    Error,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IntegerToken<'a> {
    pub source: &'a str,
    pub format: IntegerFormat,
    pub suffix: Option<IntegerSuffix>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IntegerFormat {
    Decimal,
    Octal,
    Hexadecimal,
    Binary,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IntegerSuffix {
    Unsigned,
    Long,
    LongUnsigned,
    LongLong,
    LongLongUnsigned,
    BitPrecise,
    BitPreciseUnsigned,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StringEncoding {
    None,
    UTF8,
    UTF16,
    UTF32,
    Wide,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct At {
    pub file: usize,
    pub line: u32,
    pub column: u32,
}
impl At {
    pub fn new(file: usize, line: u32, column: u32) -> Self {
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

#[derive(Clone, Debug)]
pub struct Files {
    files: Vec<String>,
}
impl Files {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn get_file_id(&mut self, name: &str) -> usize {
        for (i, file) in self.files.iter().enumerate() {
            if file == name {
                return i;
            }
        }

        let i = self.files.len();
        self.files.push(name.to_string());
        i
    }
}
impl Index<usize> for Files {
    type Output = str;
    fn index(&self, index: usize) -> &Self::Output {
        &self.files[index]
    }
}
