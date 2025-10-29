use crate::token::{At, IntegerToken, StringEncoding, TokenKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct List<T> {
    pub at: At,
    pub kind: ListKind<T>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListKind<T> {
    Leaf(Box<T>),
    Cons(Box<List<T>>, Box<T>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommaList<T> {
    pub at: At,
    pub kind: CommaListKind<T>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommaListKind<T> {
    Leaf(Box<T>),
    Cons {
        left: Box<CommaList<T>>,
        comma: At,
        right: Box<T>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringLiteral<'a> {
    pub at: At,
    pub literal: &'a str,
    pub encoding: StringEncoding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression<'a> {
    pub at: At,
    pub kind: ExpressionKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionKind<'a> {
    Identifier(&'a str),
    Integer(IntegerToken<'a>),
    String(StringLiteral<'a>),
    Parenthesized {
        open_parenthesis: At,
        inner: Box<Expression<'a>>,
        close_parenthesis: At,
    },
    GenericSelection(GenericSelection<'a>),
    Index {
        left: Box<Expression<'a>>,
        open_bracket: At,
        index: Box<Expression<'a>>,
        close_bracket: At,
    },
    Call {
        left: Box<Expression<'a>>,
        open_parenthesis: At,
        arguments: Option<ArgumentExpressionList<'a>>,
        close_parenthesis: At,
    },
    Member {
        left: Box<Expression<'a>>,
        period: At,
        name: &'a str,
    },
    MemberIndirect {
        left: Box<Expression<'a>>,
        arrow: At,
        name: &'a str,
    },
    PostIncrement {
        left: Box<Expression<'a>>,
        double_plus: At,
    },
    PostDecrement {
        left: Box<Expression<'a>>,
        double_minus: At,
    },
    CompoundLiteral(CompoundLiteral<'a>),
    PreIncrement {
        double_plus: At,
        right: Box<Expression<'a>>,
    },
    PreDecrement {
        double_minus: At,
        right: Box<Expression<'a>>,
    },
    Unary(UnaryOperator, Box<Expression<'a>>),
    Sizeof {
        sizeof_keyword: At,
        kind: SizeofKind<'a>,
    },
    Alignof {
        alignof_keyword: At,
        open_parenthesis: At,
        type_name: TypeName<'a>,
        close_parenthesis: At,
    },
    Cast {
        open_parenthesis: At,
        type_name: TypeName<'a>,
        close_parenthesis: At,
        right: Box<Expression<'a>>,
    },
    Binary {
        left: Box<Expression<'a>>,
        operator: (At, BinaryOperator),
        right: Box<Expression<'a>>,
    },
    Conditional {
        condition: Box<Expression<'a>>,
        question: At,
        then_value: Box<Expression<'a>>,
        colon: At,
        else_value: Box<Expression<'a>>,
    },
    Assign {
        left: Box<Expression<'a>>,
        operator: (At, AssignmentOperator),
        right: Box<Expression<'a>>,
    },
    Comma {
        left: Box<Expression<'a>>,
        comma: At,
        right: Box<Expression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericSelection<'a> {
    pub at: At,
    pub generic_keyword: At,
    pub open_parenthesis: At,
    pub controlling_expression: Box<Expression<'a>>,
    pub comma: At,
    pub generic_assocs: GenericAssocList<'a>,
    pub close_parenthesis: At,
}

pub type GenericAssocList<'a> = CommaList<GenericAssociation<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericAssociation<'a> {
    pub at: At,
    pub colon: At,
    pub kind: GenericAssociationKind<'a>,
    pub value: Expression<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenericAssociationKind<'a> {
    Default { default_keyword: At },
    ForType(TypeName<'a>),
}

pub type ArgumentExpressionList<'a> = CommaList<Expression<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundLiteral<'a> {
    pub at: At,
    pub open_parenthesis: At,
    pub storage_class: Option<StorageClassSpecifiers>,
    pub type_name: TypeName<'a>,
    pub close_parenthesis: At,
    pub initializer: BracedInitializer<'a>,
}

pub type StorageClassSpecifiers = List<StorageClassSpecifier>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    AddressOf,
    Dereference,
    Positive,
    Negative,
    BitNot,
    LogicalNot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SizeofKind<'a> {
    Expression(Box<Expression<'a>>),
    Type {
        open_parenthesis: At,
        type_name: TypeName<'a>,
        close_parenthesis: At,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    ShiftLeft,
    ShiftRight,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
    BitAnd,
    BitOr,
    BitXor,
    LogicalAnd,
    LogicalOr,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AssignmentOperator {
    Assign,
    Multiply,
    Divide,
    Modulo,
    Add,
    Subtract,
    ShiftLeft,
    ShiftRight,
    And,
    Xor,
    Or,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration<'a> {
    pub at: At,
    pub kind: DeclarationKind<'a>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclarationKind<'a> {
    Normal {
        attributes: Option<AttributeSpecifierSequence<'a>>,
        specifiers: DeclarationSpecifiers<'a>,
        init_declarators: Option<InitDeclaratorList<'a>>,
        semicolon: At,
    },
    Assert(StaticAssertDeclaration<'a>),
    Attribute(AttributeDeclaration<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclarationSpecifiers<'a> {
    pub at: At,
    pub specifier: DeclarationSpecifier<'a>,
    pub kind: DeclarationSpecifiersKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclarationSpecifiersKind<'a> {
    Leaf(Option<AttributeSpecifierSequence<'a>>),
    Cons(Box<DeclarationSpecifiers<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclarationSpecifier<'a> {
    pub at: At,
    pub kind: DeclarationSpecifierKind<'a>,
}
impl<'a> From<TypeSpecifierQualifier<'a>> for DeclarationSpecifier<'a> {
    fn from(value: TypeSpecifierQualifier<'a>) -> Self {
        Self {
            at: value.at,
            kind: DeclarationSpecifierKind::Type(value),
        }
    }
}
impl<'a> From<StorageClassSpecifier> for DeclarationSpecifier<'a> {
    fn from(value: StorageClassSpecifier) -> Self {
        Self {
            at: value.at,
            kind: DeclarationSpecifierKind::StorageClass(value),
        }
    }
}
impl<'a> From<FunctionSpecifier> for DeclarationSpecifier<'a> {
    fn from(value: FunctionSpecifier) -> Self {
        Self {
            at: value.at,
            kind: DeclarationSpecifierKind::Function(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclarationSpecifierKind<'a> {
    StorageClass(StorageClassSpecifier),
    Type(TypeSpecifierQualifier<'a>),
    Function(FunctionSpecifier),
}

pub type InitDeclaratorList<'a> = CommaList<InitDeclarator<'a>>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitDeclarator<'a> {
    pub at: At,
    pub declarator: Declarator<'a>,
    pub initializer: Option<(At, Initializer<'a>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeDeclaration<'a> {
    pub at: At,
    pub attributes: AttributeSpecifierSequence<'a>,
    pub semicolon: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageClassSpecifier {
    pub at: At,
    pub kind: StorageClassSpecifierKind,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StorageClassSpecifierKind {
    Auto,
    Constexpr,
    Extern,
    Register,
    Static,
    ThreadLocal,
    Typedef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeSpecifier<'a> {
    pub at: At,
    pub kind: TypeSpecifierKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeSpecifierKind<'a> {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Signed,
    Unsigned,
    BitInt {
        bitint_keyword: At,
        open_parenthesis: At,
        width: Expression<'a>,
        close_parenthesis: At,
    },
    Bool,
    Complex,
    Decimal32,
    Decimal64,
    Decimal128,
    Atomic(AtomicTypeSpecifier<'a>),
    StructOrUnion(StructOrUnionSpecifier<'a>),
    Enum(EnumSpecifier<'a>),
    TypedefName(&'a str),
    Typeof(TypeofSpecifier<'a>),
}
impl<'a> From<AtomicTypeSpecifier<'a>> for TypeSpecifierKind<'a> {
    fn from(value: AtomicTypeSpecifier<'a>) -> Self {
        TypeSpecifierKind::Atomic(value)
    }
}
impl<'a> From<StructOrUnionSpecifier<'a>> for TypeSpecifierKind<'a> {
    fn from(value: StructOrUnionSpecifier<'a>) -> Self {
        TypeSpecifierKind::StructOrUnion(value)
    }
}
impl<'a> From<EnumSpecifier<'a>> for TypeSpecifierKind<'a> {
    fn from(value: EnumSpecifier<'a>) -> Self {
        TypeSpecifierKind::Enum(value)
    }
}
impl<'a> From<TypeofSpecifier<'a>> for TypeSpecifierKind<'a> {
    fn from(value: TypeofSpecifier<'a>) -> Self {
        TypeSpecifierKind::Typeof(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructOrUnionSpecifier<'a> {
    pub at: At,
    pub struct_or_union: (At, StructOrUnion),
    pub attributes: Option<AttributeSpecifierSequence<'a>>,
    pub tag: Option<&'a str>,
    pub members: Option<(At, MemberDeclarationList<'a>, At)>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StructOrUnion {
    Struct,
    Union,
}

pub type MemberDeclarationList<'a> = List<MemberDeclaration<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberDeclaration<'a> {
    pub at: At,
    pub kind: MemberDeclarationKind<'a>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberDeclarationKind<'a> {
    Member {
        attributes: Option<AttributeSpecifierSequence<'a>>,
        specifier_qualifiers: SpecifierQualifierList<'a>,
        member_declarators: Option<MemberDeclaratorList<'a>>,
        semicolon: At,
    },
    Assert(StaticAssertDeclaration<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecifierQualifierList<'a> {
    pub at: At,
    pub specifier_qualifier: Box<TypeSpecifierQualifier<'a>>,
    pub kind: SpecifierQualifierListKind<'a>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecifierQualifierListKind<'a> {
    Leaf(Option<AttributeSpecifierSequence<'a>>),
    Cons(Box<SpecifierQualifierList<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeSpecifierQualifier<'a> {
    pub at: At,
    pub kind: TypeSpecifierQualifierKind<'a>,
}
impl<'a> From<TypeSpecifier<'a>> for TypeSpecifierQualifier<'a> {
    fn from(value: TypeSpecifier<'a>) -> Self {
        Self {
            at: value.at,
            kind: TypeSpecifierQualifierKind::TypeSpecifier(value),
        }
    }
}
impl<'a> From<TypeQualifier> for TypeSpecifierQualifier<'a> {
    fn from(value: TypeQualifier) -> Self {
        Self {
            at: value.at,
            kind: TypeSpecifierQualifierKind::TypeQualifier(value),
        }
    }
}
impl<'a> From<AlignmentSpecifier<'a>> for TypeSpecifierQualifier<'a> {
    fn from(value: AlignmentSpecifier<'a>) -> Self {
        Self {
            at: value.at,
            kind: TypeSpecifierQualifierKind::Alignment(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeSpecifierQualifierKind<'a> {
    TypeSpecifier(TypeSpecifier<'a>),
    TypeQualifier(TypeQualifier),
    Alignment(AlignmentSpecifier<'a>),
}

pub type MemberDeclaratorList<'a> = CommaList<MemberDeclarator<'a>>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberDeclarator<'a> {
    pub at: At,
    pub declarator: Option<Declarator<'a>>,
    pub width: Option<(At, Expression<'a>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumSpecifier<'a> {
    pub at: At,
    pub enum_keyword: At,
    pub attributes: Option<AttributeSpecifierSequence<'a>>,
    pub tag: Option<&'a str>,
    pub enum_type: Option<EnumTypeSpecifier<'a>>,
    pub enumerators: Option<(At, EnumeratorList<'a>, Option<At>, At)>,
}

pub type EnumeratorList<'a> = CommaList<Enumerator<'a>>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Enumerator<'a> {
    pub at: At,
    pub name: &'a str,
    pub attributes: Option<AttributeSpecifierSequence<'a>>,
    pub value: Option<(At, Expression<'a>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumTypeSpecifier<'a> {
    pub at: At,
    pub colon: At,
    pub specifier_qualifiers: SpecifierQualifierList<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtomicTypeSpecifier<'a> {
    pub at: At,
    pub atomic_keyword: At,
    pub open_parenthesis: At,
    pub type_name: TypeName<'a>,
    pub close_parenthesis: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeofSpecifier<'a> {
    pub at: At,
    pub typeof_keyword: At,
    pub unqual: bool,
    pub open_parenthesis: At,
    pub argument: TypeofSpecifierArgument<'a>,
    pub close_parenthesis: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeofSpecifierArgument<'a> {
    pub at: At,
    pub kind: TypeofSpecifierArgumentKind<'a>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeofSpecifierArgumentKind<'a> {
    Expression(Expression<'a>),
    Type(TypeName<'a>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TypeQualifier {
    pub at: At,
    pub kind: TypeQualifierKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypeQualifierKind {
    Const,
    Restrict,
    Volatile,
    Atomic,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FunctionSpecifier {
    pub at: At,
    pub kind: FunctionSpecifierKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FunctionSpecifierKind {
    Inline,
    NoReturn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlignmentSpecifier<'a> {
    pub at: At,
    pub alignas_keyword: At,
    pub open_parenthesis: At,
    pub kind: AlignmentSpecifierKind<'a>,
    pub close_parenthesis: At,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlignmentSpecifierKind<'a> {
    Type(TypeName<'a>),
    Expression(Expression<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declarator<'a> {
    pub at: At,
    pub pointer: Option<Pointer<'a>>,
    pub direct: DirectDeclarator<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectDeclarator<'a> {
    pub at: At,
    pub kind: DirectDeclaratorKind<'a>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectDeclaratorKind<'a> {
    Name(&'a str, Option<AttributeSpecifierSequence<'a>>),
    Parenthesized {
        open_parenthesis: At,
        inner: Box<Declarator<'a>>,
        close_parenthesis: At,
    },
    Array(ArrayDeclarator<'a>, Option<AttributeSpecifierSequence<'a>>),
    Function(
        FunctionDeclarator<'a>,
        Option<AttributeSpecifierSequence<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrayDeclarator<'a> {
    pub at: At,
    pub left: Box<DirectDeclarator<'a>>,
    pub open_bracket: At,
    pub qualifiers: Option<TypeQualifierList>,
    pub kind: ArrayDeclaratorKind<'a>,
    pub close_bracket: At,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayDeclaratorKind<'a> {
    Normal {
        static_keyword: Option<At>,
        size: Option<Expression<'a>>,
    },
    Var {
        asterisk: At,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionDeclarator<'a> {
    pub at: At,
    pub left: Box<DirectDeclarator<'a>>,
    pub open_parenthesis: At,
    pub parameters: Option<ParameterTypeList<'a>>,
    pub close_parenthesis: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pointer<'a> {
    pub at: At,
    pub asterisk: At,
    pub attributes: Option<AttributeSpecifierSequence<'a>>,
    pub qualifiers: Option<TypeQualifierList>,
    pub right: Option<Box<Pointer<'a>>>,
}

pub type TypeQualifierList = List<TypeQualifier>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterTypeList<'a> {
    pub at: At,
    pub parameters: Option<(ParameterList<'a>, Option<At>)>,
    pub ellipses: Option<At>,
}

pub type ParameterList<'a> = CommaList<ParameterDeclaration<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterDeclaration<'a> {
    pub at: At,
    pub attributes: Option<AttributeSpecifierSequence<'a>>,
    pub specifiers: DeclarationSpecifiers<'a>,
    pub kind: ParameterDeclarationKind<'a>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParameterDeclarationKind<'a> {
    Concrete(Declarator<'a>),
    Abstract(Option<AbstractDeclarator<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeName<'a> {
    pub at: At,
    pub specifier_qualifiers: SpecifierQualifierList<'a>,
    pub declarator: Option<AbstractDeclarator<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbstractDeclarator<'a> {
    pub at: At,
    pub pointer: Option<Pointer<'a>>,
    pub direct: Option<DirectAbstractDeclarator<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectAbstractDeclarator<'a> {
    pub at: At,
    pub kind: DirectAbstractDeclaratorKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectAbstractDeclaratorKind<'a> {
    Parenthesized {
        open_parenthesis: At,
        inner: Box<AbstractDeclarator<'a>>,
        close_parenthesis: At,
    },
    Array(
        ArrayAbstractDeclarator<'a>,
        Option<AttributeSpecifierSequence<'a>>,
    ),
    Function(
        FunctionAbstractDeclarator<'a>,
        Option<AttributeSpecifierSequence<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrayAbstractDeclarator<'a> {
    pub at: At,
    pub left: Option<Box<DirectAbstractDeclarator<'a>>>,
    pub open_bracket: At,
    pub kind: ArrayAbstractDeclaratorKind<'a>,
    pub close_bracket: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayAbstractDeclaratorKind<'a> {
    Normal {
        qualifiers: Option<TypeQualifierList>,
        static_keyword: Option<At>,
        size: Option<Box<Expression<'a>>>,
    },
    Var {
        asterisk: At,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionAbstractDeclarator<'a> {
    pub at: At,
    pub left: Option<Box<DirectAbstractDeclarator<'a>>>,
    pub open_parenthesis: At,
    pub parameters: Option<ParameterTypeList<'a>>,
    pub close_parenthesis: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BracedInitializer<'a> {
    pub at: At,
    pub open_brace: At,
    pub initializers: Option<(InitializerList<'a>, Option<At>)>,
    pub close_brace: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Initializer<'a> {
    pub at: At,
    pub kind: InitializerKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitializerKind<'a> {
    Expression(Expression<'a>),
    Braced(Box<BracedInitializer<'a>>),
}

pub type InitializerList<'a> = CommaList<(Option<Designation<'a>>, Initializer<'a>)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Designation<'a> {
    pub at: At,
    pub designators: DesignatorList<'a>,
    pub equal: At,
}
pub type DesignatorList<'a> = List<Designator<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Designator<'a> {
    pub at: At,
    pub kind: DesignatorKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesignatorKind<'a> {
    InBrackets {
        open_bracket: At,
        value: Expression<'a>,
        close_bracket: At,
    },
    AfterPeriod {
        period: At,
        name: &'a str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticAssertDeclaration<'a> {
    pub at: At,
    pub static_assert_keyword: At,
    pub open_parenthesis: At,
    pub condition: Expression<'a>,
    pub message: Option<(At, StringLiteral<'a>)>,
    pub close_parenthesis: At,
    pub semicolon: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeSpecifierSequence<'a> {
    pub at: At,
    pub left: Option<Box<AttributeSpecifierSequence<'a>>>,
    pub specifier: AttributeSpecifier<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeSpecifier<'a> {
    pub at: At,
    pub open_bracket_0: At,
    pub open_bracket_1: At,
    pub attributes: AttributeList<'a>,
    pub close_bracket_0: At,
    pub close_bracket_1: At,
}

pub type AttributeList<'a> = CommaList<Option<Attribute<'a>>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute<'a> {
    pub at: At,
    pub token: AttributeToken<'a>,
    pub argument_clause: Option<AttributeArgumentClause<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeToken<'a> {
    pub at: At,
    pub prefix: Option<(&'a str, At)>,
    pub token: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeArgumentClause<'a> {
    pub at: At,
    pub open_parenthesis: At,
    pub tokens: Option<BalancedTokenSequence<'a>>,
    pub close_parenthesis: At,
}

pub type BalancedTokenSequence<'a> = List<BalancedToken<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BalancedToken<'a> {
    pub at: At,
    pub kind: BalancedTokenKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BalancedTokenKind<'a> {
    Parenthesized {
        open_parenthesis: At,
        inner: Option<BalancedTokenSequence<'a>>,
        close_parenthesis: At,
    },
    Bracketed {
        open_bracket: At,
        inner: Option<BalancedTokenSequence<'a>>,
        close_bracket: At,
    },
    Braced {
        open_brace: At,
        inner: Option<BalancedTokenSequence<'a>>,
        close_brace: At,
    },
    Token(TokenKind<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement<'a> {
    pub at: At,
    pub kind: StatementKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatementKind<'a> {
    Labeled(LabeledStatement<'a>),
    Unlabeled(UnlabeledStatement<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnlabeledStatement<'a> {
    pub at: At,
    pub kind: UnlabeledStatementKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnlabeledStatementKind<'a> {
    Expression(ExpressionStatement<'a>),
    Primary(Option<AttributeSpecifierSequence<'a>>, PrimaryBlock<'a>),
    Jump(Option<AttributeSpecifierSequence<'a>>, JumpStatement<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrimaryBlock<'a> {
    pub at: At,
    pub kind: PrimaryBlockKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrimaryBlockKind<'a> {
    Compound(CompoundStatement<'a>),
    Selection(SelectionStatement<'a>),
    Iteration(IterationStatement<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecondaryBlock<'a> {
    pub at: At,
    pub statement: Box<Statement<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label<'a> {
    pub at: At,
    pub attributes: Option<AttributeSpecifierSequence<'a>>,
    pub kind: LabelKind<'a>,
    pub colon: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LabelKind<'a> {
    Name(&'a str),
    Case {
        case_keyword: At,
        value: Expression<'a>,
    },
    Default {
        default_keyword: At,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabeledStatement<'a> {
    pub at: At,
    pub label: Label<'a>,
    pub statement: Box<Statement<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundStatement<'a> {
    pub at: At,
    pub open_brace: At,
    pub items: Option<BlockItemList<'a>>,
    pub close_brace: At,
}

pub type BlockItemList<'a> = List<BlockItem<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockItem<'a> {
    pub at: At,
    pub kind: BlockItemKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockItemKind<'a> {
    Declaration(Declaration<'a>),
    Unlabeled(UnlabeledStatement<'a>),
    Label(Label<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionStatement<'a> {
    pub at: At,
    pub attributes: Option<AttributeSpecifierSequence<'a>>,
    pub expression: Option<Expression<'a>>,
    pub semicolon: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionStatement<'a> {
    pub at: At,
    pub kind: SelectionStatementKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionStatementKind<'a> {
    If {
        if_keyword: At,
        open_parenthesis: At,
        condition: Expression<'a>,
        close_parenthesis: At,
        then_body: SecondaryBlock<'a>,
        else_body: Option<(At, SecondaryBlock<'a>)>,
    },
    Switch {
        switch_keyword: At,
        open_parenthesis: At,
        controlling_expression: Expression<'a>,
        close_parenthesis: At,
        body: SecondaryBlock<'a>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IterationStatement<'a> {
    pub at: At,
    pub kind: IterationStatementKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IterationStatementKind<'a> {
    While {
        while_keyword: At,
        open_parenthesis: At,
        condition: Expression<'a>,
        close_parenthesis: At,
        body: SecondaryBlock<'a>,
    },
    DoWhile {
        do_keyword: At,
        body: SecondaryBlock<'a>,
        while_keyword: At,
        open_parenthesis: At,
        condition: Expression<'a>,
        close_parenthesis: At,
        semicolon: At,
    },
    For {
        for_keyword: At,
        open_parenthesis: At,
        initializer: ForInitializer<'a>,
        condition: Option<Expression<'a>>,
        semicolon: At,
        counter: Option<Expression<'a>>,
        close_parenthesis: At,
        body: SecondaryBlock<'a>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JumpStatement<'a> {
    pub at: At,
    pub kind: JumpStatementKind<'a>,
    pub semicolon: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JumpStatementKind<'a> {
    Goto {
        goto_keyword: At,
        target: &'a str,
    },
    Continue {
        continue_keyword: At,
    },
    Break {
        break_keyword: At,
    },
    Return {
        return_keyword: At,
        value: Option<Expression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForInitializer<'a> {
    Expression(Option<Expression<'a>>, At),
    Declaration(Declaration<'a>),
}

pub type TranslationUnit<'a> = List<ExternalDeclaration<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalDeclaration<'a> {
    pub at: At,
    pub kind: ExternalDeclarationKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalDeclarationKind<'a> {
    Function(FunctionDefinition<'a>),
    Declaration(Declaration<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionDefinition<'a> {
    pub at: At,
    pub attributes: Option<AttributeSpecifierSequence<'a>>,
    pub specifiers: DeclarationSpecifiers<'a>,
    pub declarator: Declarator<'a>,
    pub body: CompoundStatement<'a>,
}
