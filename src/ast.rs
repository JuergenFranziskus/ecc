use std::rc::Rc;

use crate::token::{At, IntegerToken, StringEncoding, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node<T>(pub Rc<(At, T)>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum List<T> {
    Leaf(Node<T>),
    Rec(Node<List<T>>, Node<T>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommaList<T> {
    Leaf(Node<T>),
    Rec {
        left: Node<List<T>>,
        comma: At,
        right: Node<T>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringLiteral<'a>(pub &'a str, pub StringEncoding);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrimaryExpression<'a> {
    Identifier(&'a str),
    Integer(IntegerToken<'a>),
    StringLiteral(StringLiteral<'a>),
    Parenthesized {
        open_parenthesis: At,
        inner: Node<Expression<'a>>,
        close_parenthesis: At,
    },
    Generic(GenericSelection<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericSelection<'a> {
    pub generic_keyword: At,
    pub open_parenthesis: At,
    pub controlling_expression: Node<AssignmentExpression<'a>>,
    pub comma: At,
    pub association_list: Node<GenericAssocList<'a>>,
}

pub type GenericAssocList<'a> = List<GenericAssociation<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenericAssociation<'a> {
    ForType {
        type_name: Node<TypeName<'a>>,
        colon: At,
        value: Node<AssignmentExpression<'a>>,
    },
    Default {
        default_keyword: At,
        colon: At,
        value: Node<AssignmentExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PostfixExpression<'a> {
    Primary(Node<PrimaryExpression<'a>>),
    Index {
        left: Node<PostfixExpression<'a>>,
        open_bracket: At,
        index: Node<Expression<'a>>,
        close_bracket: At,
    },
    Call {
        left: Node<PostfixExpression<'a>>,
        open_parenthesis: At,
        arguments: Option<Node<ArgumentExpressionList<'a>>>,
        close_parenthesis: At,
    },
    Member {
        left: Node<PostfixExpression<'a>>,
        period: At,
        name: &'a str,
    },
    MemberIndirect {
        left: Node<PostfixExpression<'a>>,
        arrow: At,
        name: &'a str,
    },
    PostIncrement {
        left: Node<PostfixExpression<'a>>,
        plus_plus: At,
    },
    PostDecrement {
        left: Node<PostfixExpression<'a>>,
        minus_minus: At,
    },
    CompoundLiteral(Node<CompoundLiteral<'a>>),
}

pub type ArgumentExpressionList<'a> = CommaList<AssignmentExpression<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundLiteral<'a> {
    pub open_parenthesis: At,
    pub storage_class: Option<Node<StorageClassSpecifiers>>,
    pub type_name: Node<TypeName<'a>>,
    pub close_parenthesis: At,
    pub initializer: Node<BracedInitializer<'a>>,
}

pub type StorageClassSpecifiers = List<StorageClassSpecifier>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryExpression<'a> {
    Postfix(Node<PostfixExpression<'a>>),
    PreIncrement {
        plus_plus: At,
        right: Node<UnaryExpression<'a>>,
    },
    PreDecrement {
        minus_minus: At,
        right: Node<UnaryExpression<'a>>,
    },
    Operator(Node<UnaryOperator>, Node<CastExpression<'a>>),
    SizeofValue {
        sizeof_keyword: At,
        right: Node<UnaryExpression<'a>>,
    },
    SizeofType {
        sizeof_keyword: At,
        open_parenthesis: At,
        type_name: Node<TypeName<'a>>,
        close_parenthesis: At,
    },
    Alignof {
        alignof_keyword: At,
        open_parenthesis: At,
        type_name: Node<TypeName<'a>>,
        close_parenthesis: At,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    AddressOf,
    Dereference,
    Positive,
    Negative,
    BitNot,
    LogicalNot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CastExpression<'a> {
    Unary(Node<UnaryExpression<'a>>),
    Cast {
        open_parenthesis: At,
        type_name: Node<TypeName<'a>>,
        close_parenthesis: At,
        right: Node<CastExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MultiplicativeExpression<'a> {
    Cast(Node<CastExpression<'a>>),
    Multiply {
        left: Node<MultiplicativeExpression<'a>>,
        asterisk: At,
        right: Node<CastExpression<'a>>,
    },
    Divide {
        left: Node<MultiplicativeExpression<'a>>,
        slash: At,
        right: Node<CastExpression<'a>>,
    },
    Modulo {
        left: Node<MultiplicativeExpression<'a>>,
        percent: At,
        right: Node<CastExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdditiveExpression<'a> {
    Multiplicative(Node<MultiplicativeExpression<'a>>),
    Add {
        left: Node<AdditiveExpression<'a>>,
        plus: At,
        right: Node<MultiplicativeExpression<'a>>,
    },
    Subtract {
        left: Node<AdditiveExpression<'a>>,
        minus: At,
        right: Node<MultiplicativeExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShiftExpression<'a> {
    Additive(Node<AdditiveExpression<'a>>),
    Left {
        left: Node<ShiftExpression<'a>>,
        double_less: At,
        right: Node<AdditiveExpression<'a>>,
    },
    Right {
        left: Node<ShiftExpression<'a>>,
        double_greater: At,
        right: Node<AdditiveExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationalExpression<'a> {
    Shift(Node<ShiftExpression<'a>>),
    Less {
        left: Node<RelationalExpression<'a>>,
        less: At,
        right: Node<ShiftExpression<'a>>,
    },
    Greater {
        left: Node<RelationalExpression<'a>>,
        greater: At,
        right: Node<ShiftExpression<'a>>,
    },
    LessEqual {
        left: Node<RelationalExpression<'a>>,
        less_equal: At,
        right: Node<ShiftExpression<'a>>,
    },
    GreaterEqual {
        left: Node<RelationalExpression<'a>>,
        greater_equal: At,
        right: Node<ShiftExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EqualityExpression<'a> {
    Relational(Node<RelationalExpression<'a>>),
    Equal {
        left: Node<EqualityExpression<'a>>,
        equal: At,
        right: Node<RelationalExpression<'a>>,
    },
    NotEqual {
        left: Node<EqualityExpression<'a>>,
        not_equal: At,
        right: Node<RelationalExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AndExpression<'a> {
    Equality(Node<EqualityExpression<'a>>),
    And {
        left: Node<AndExpression<'a>>,
        ampersand: At,
        right: Node<EqualityExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExclusiveOrExpression<'a> {
    And(Node<AndExpression<'a>>),
    ExclusiveOr {
        left: Node<ExclusiveOrExpression<'a>>,
        caret: At,
        right: Node<AndExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InclusiveOrExpression<'a> {
    ExclusiveOr(Node<ExclusiveOrExpression<'a>>),
    InclusiveOr {
        left: Node<InclusiveOrExpression<'a>>,
        bar: At,
        right: Node<ExclusiveOrExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogicalAndExpression<'a> {
    InclusiveOr(Node<InclusiveOrExpression<'a>>),
    LogicalAnd {
        left: Node<LogicalAndExpression<'a>>,
        double_ampersand: At,
        right: Node<InclusiveOrExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogicalOrExpression<'a> {
    LogicalAnd(Node<LogicalAndExpression<'a>>),
    LogicalOr {
        left: Node<LogicalOrExpression<'a>>,
        double_bar: At,
        right: Node<LogicalAndExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConditionalExpression<'a> {
    LogicalOr(Node<LogicalOrExpression<'a>>),
    Conditional {
        condition: Node<LogicalOrExpression<'a>>,
        question_mark: At,
        then_value: Node<Expression<'a>>,
        colon: At,
        else_value: Node<ConditionalExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssignmentExpression<'a> {
    Conditional(Node<ConditionalExpression<'a>>),
    Assignment {
        left: Node<UnaryExpression<'a>>,
        operator: Node<AssignmentOperator>,
        right: Node<AssignmentExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
pub enum Expression<'a> {
    Assign(Node<AssignmentExpression<'a>>),
    Comma {
        left: Node<Expression<'a>>,
        comma: At,
        right: Node<AssignmentExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstantExpression<'a>(pub Node<ConditionalExpression<'a>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declaration<'a> {
    Normal {
        specifiers: Node<DeclarationSpecifiers<'a>>,
        declarators: Option<Node<InitDeclaratorList<'a>>>,
        semicolon: At,
    },
    WithAttributes {
        attributes: Node<AttributeSpecifierSequence<'a>>,
        specifiers: Node<DeclarationSpecifiers<'a>>,
        declarators: Node<InitDeclaratorList<'a>>,
        semicolon: At,
    },
    Assert(Node<StaticAssertDeclaration<'a>>),
    Attribute(Node<AttributeDeclaration<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclarationSpecifiers<'a> {
    Leaf(
        Node<DeclarationSpecifier<'a>>,
        Option<Node<AttributeSpecifierSequence<'a>>>,
    ),
    Rec(
        Node<DeclarationSpecifier<'a>>,
        Node<DeclarationSpecifiers<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclarationSpecifier<'a> {
    StorageClass(Node<StorageClassSpecifier>),
    TypeSpecifier(Node<TypeSpecifierQualifier<'a>>),
    FunctionSpecifier(Node<FunctionSpecifier>),
}

pub type InitDeclaratorList<'a> = CommaList<InitDeclarator<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitDeclarator<'a> {
    NoInitializer(Node<Declarator<'a>>),
    Initializer {
        declarator: Node<Declarator<'a>>,
        equal: At,
        initializer: Node<Initializer<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeDeclaration<'a> {
    pub attributes: Node<AttributeSpecifierSequence<'a>>,
    pub semicolon: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageClassSpecifier {
    Auto,
    Constexpr,
    Extern,
    Register,
    Static,
    ThreadLocal,
    Typedef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeSpecifier<'a> {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Singed,
    Unsigned,
    BitInt {
        bitint_keyword: At,
        open_parenthesis: At,
        width: Node<ConstantExpression<'a>>,
        close_parenthesis: At,
    },
    Bool,
    Complex,
    Decimal32,
    Decimal64,
    Decimal128,
    Atomic(Node<AtomicTypeSpecifier<'a>>),
    StructOrUnion(Node<StructOrUnionSpecifier<'a>>),
    Enum(Node<EnumSpecifier<'a>>),
    TypedefName(&'a str),
    Typeof(Node<TypeofSpecifier<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructOrUnionSpecifier<'a> {
    WithMembers {
        struct_or_union: Node<StructOrUnion>,
        attributes: Option<Node<AttributeSpecifierSequence<'a>>>,
        tag: Option<&'a str>,
        open_brace: At,
        members: Node<MemberDeclarationList<'a>>,
        close_brace: At,
    },
    WithoutMembers(
        Node<StructOrUnion>,
        Option<Node<AttributeSpecifierSequence<'a>>>,
        &'a str,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructOrUnion {
    Struct,
    Union,
}

pub type MemberDeclarationList<'a> = List<MemberDeclaration<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberDeclaration<'a> {
    Member {
        attributes: Option<Node<AttributeSpecifierSequence<'a>>>,
        specifiers: Node<SpecifierQualifierList<'a>>,
        declarators: Option<Node<MemberDeclaratorList<'a>>>,
        semicolon: At,
    },
    Assert(Node<StaticAssertDeclaration<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecifierQualifierList<'a> {
    Leaf(
        Node<TypeSpecifierQualifier<'a>>,
        Option<Node<AttributeSpecifierSequence<'a>>>,
    ),
    Rec(
        Node<TypeSpecifierQualifier<'a>>,
        Node<SpecifierQualifierList<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeSpecifierQualifier<'a> {
    TypeSpecifier(Node<TypeSpecifier<'a>>),
    TypeQualifier(Node<TypeQualifier>),
    AlignmentSpecifier(Node<AlignmentSpecifier<'a>>),
}

pub type MemberDeclaratorList<'a> = CommaList<MemberDeclarator<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberDeclarator<'a> {
    WithoutWidth(Node<Declarator<'a>>),
    WithWidth {
        declarator: Option<Node<Declarator<'a>>>,
        colon: At,
        width: Node<ConstantExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnumSpecifier<'a> {
    WithList {
        enum_keyword: At,
        attributes: Node<AttributeSpecifierSequence<'a>>,
        tag: &'a str,
        enum_type: Option<Node<EnumTypeSpecifier<'a>>>,
        open_brace: At,
        enumerators: Node<EnumeratorList<'a>>,
        final_comma: Option<At>,
        close_brace: At,
    },
    WithoutList {
        enum_keyword: At,
        tag: &'a str,
        enum_type: Option<Node<EnumTypeSpecifier<'a>>>,
    },
}

pub type EnumeratorList<'a> = CommaList<Enumerator<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Enumerator<'a> {
    WithoutValue(&'a str, Option<Node<AttributeSpecifierSequence<'a>>>),
    WithValue {
        name: &'a str,
        attributes: Option<Node<AttributeSpecifierSequence<'a>>>,
        equal: At,
        value: Node<ConstantExpression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumTypeSpecifier<'a> {
    pub colon: At,
    pub specifiers: Node<SpecifierQualifierList<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtomicTypeSpecifier<'a> {
    pub atomic_keyword: At,
    pub open_parenthesis: At,
    pub type_name: Node<TypeName<'a>>,
    pub close_parenthesis: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeofSpecifier<'a> {
    Typeof {
        typeof_keyword: At,
        open_parenthesis: At,
        argument: Node<TypeofSpecifierArgument<'a>>,
        close_parenthesis: At,
    },
    TypeofUnqual {
        typeof_unqual_keyword: At,
        open_parenthesis: At,
        argument: Node<TypeofSpecifierArgument<'a>>,
        close_parenthesis: At,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeofSpecifierArgument<'a> {
    Expression(Node<Expression<'a>>),
    Type(Node<TypeName<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeQualifier {
    Const,
    Restrict,
    Volatile,
    Atomic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionSpecifier {
    Inline,
    NoReturn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlignmentSpecifier<'a> {
    AsType {
        alignas_keyword: At,
        open_parenthesis: At,
        type_name: Node<TypeName<'a>>,
        close_parenthesis: At,
    },
    AsExpression {
        alignas_keyword: At,
        open_parenthesis: At,
        expression: Node<ConstantExpression<'a>>,
        close_parenthesis: At,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declarator<'a>(
    pub Option<Node<Pointer<'a>>>,
    pub Node<DirectDeclarator<'a>>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectDeclarator<'a> {
    Name(&'a str, Option<Node<AttributeSpecifierSequence<'a>>>),
    Parenthesized {
        open_parenthesis: At,
        inner: Node<Declarator<'a>>,
        close_parenthesis: At,
    },
    Array(
        Node<ArrayDeclarator<'a>>,
        Option<Node<AttributeSpecifierSequence<'a>>>,
    ),
    Function(
        Node<FunctionDeclarator<'a>>,
        Option<Node<AttributeSpecifierSequence<'a>>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayDeclarator<'a> {
    NoStatic {
        left: Node<DirectDeclarator<'a>>,
        open_bracket: At,
        qualifiers: Option<Node<TypeQualifierList>>,
        length: Option<Node<AssignmentExpression<'a>>>,
        close_bracket: At,
    },
    StaticFirst {
        left: Node<DirectDeclarator<'a>>,
        open_bracket: At,
        static_keyword: At,
        qualifiers: Option<Node<TypeQualifierList>>,
        length: Node<AssignmentExpression<'a>>,
        close_bracket: At,
    },
    StaticMid {
        left: Node<DirectDeclarator<'a>>,
        open_bracket: At,
        qualifiers: Node<TypeQualifierList>,
        static_keyword: At,
        length: Node<AssignmentExpression<'a>>,
        close_bracket: At,
    },
    Variable {
        left: Node<DirectDeclarator<'a>>,
        open_bracket: At,
        qualifiers: Option<Node<TypeQualifierList>>,
        asterisk: At,
        close_bracket: At,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionDeclarator<'a> {
    pub left: Node<DirectDeclarator<'a>>,
    pub open_paren: At,
    pub parameters: Option<Node<ParameterTypeList<'a>>>,
    pub close_paren: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pointer<'a> {
    Leaf {
        asterisk: At,
        attributes: Option<Node<AttributeSpecifierSequence<'a>>>,
        qualifiers: Option<Node<TypeQualifierList>>,
    },
    Rec {
        asterisk: At,
        attributes: Option<Node<AttributeSpecifierSequence<'a>>>,
        qualifiers: Option<Node<TypeQualifierList>>,
        outer: Node<Pointer<'a>>,
    },
}

pub type TypeQualifierList = List<TypeQualifier>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParameterTypeList<'a> {
    NoVar(Node<ParameterList<'a>>),
    WithVar {
        parameters: Node<ParameterList<'a>>,
        comma: At,
        ellipses: At,
    },
    Var {
        ellipses: At,
    },
}

pub type ParameterList<'a> = CommaList<ParameterDeclaration<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParameterDeclaration<'a> {
    Concrete(
        Option<Node<AttributeSpecifierSequence<'a>>>,
        Node<DeclarationSpecifiers<'a>>,
        Node<Declarator<'a>>,
    ),
    Abstract(
        Option<Node<AttributeSpecifierSequence<'a>>>,
        Node<DeclarationSpecifiers<'a>>,
        Option<Node<AbstractDeclarator<'a>>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeName<'a>(
    pub Node<SpecifierQualifierList<'a>>,
    pub Option<Node<AbstractDeclarator<'a>>>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbstractDeclarator<'a> {
    Pointer(Node<Pointer<'a>>),
    Direct(
        Option<Node<Pointer<'a>>>,
        Node<DirectAbstractDeclarator<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectAbstractDeclarator<'a> {
    Parenthesized {
        open_parenthesis: At,
        inner: Node<AbstractDeclarator<'a>>,
        close_parenthesis: At,
    },
    Array(
        Node<ArrayAbstractDeclarator<'a>>,
        Option<Node<AttributeSpecifierSequence<'a>>>,
    ),
    Function(
        Node<FunctionAbstractDeclarator<'a>>,
        Option<Node<AttributeSpecifierSequence<'a>>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayAbstractDeclarator<'a> {
    NoStatic {
        left: Option<Node<DirectDeclarator<'a>>>,
        open_bracket: At,
        qualifiers: Option<Node<TypeQualifierList>>,
        length: Option<Node<AssignmentExpression<'a>>>,
        close_bracket: At,
    },
    StaticFirst {
        left: Option<Node<DirectDeclarator<'a>>>,
        open_bracket: At,
        static_keyword: At,
        qualifiers: Option<Node<TypeQualifierList>>,
        length: Node<AssignmentExpression<'a>>,
        close_bracket: At,
    },
    StaticMid {
        left: Option<Node<DirectDeclarator<'a>>>,
        open_bracket: At,
        qualifiers: Node<TypeQualifierList>,
        static_keyword: At,
        length: Node<AssignmentExpression<'a>>,
        close_bracket: At,
    },
    Variable {
        left: Option<Node<DirectDeclarator<'a>>>,
        open_bracket: At,
        asterisk: At,
        close_bracket: At,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionAbstractDeclarator<'a> {
    pub left: Option<Node<DirectAbstractDeclarator<'a>>>,
    pub open_paren: At,
    pub parameters: Option<Node<ParameterTypeList<'a>>>,
    pub close_paren: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BracedInitializer<'a> {
    Empty {
        open_brace: At,
        close_brace: At,
    },
    List {
        open_brace: At,
        initializers: Node<InitializerList<'a>>,
        final_comma: Option<At>,
        close_brace: At,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Initializer<'a> {
    Expression(Node<AssignmentExpression<'a>>),
    Braced(Node<BracedInitializer<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitializerListEntry<'a>(pub Option<Node<Designation<'a>>>, pub Node<Initializer<'a>>);
pub type InitializerList<'a> = CommaList<InitializerListEntry<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Designation<'a> {
    pub designators: Node<DesignatorList<'a>>,
    pub equal: At,
}

pub type DesignatorList<'a> = List<Designator<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Designator<'a> {
    InBrackets {
        open_bracket: At,
        value: Node<ConstantExpression<'a>>,
        close_bracket: At,
    },
    AfterPeriod {
        period: At,
        name: &'a str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticAssertDeclaration<'a> {
    pub static_assert_keyword: At,
    pub open_parenthesis: At,
    pub condition: Node<ConstantExpression<'a>>,
    pub message: Option<(At, Node<StringLiteral<'a>>)>,
    pub close_parenthesis: At,
    pub semicolon: At,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeSpecifierSequence<'a>(
    pub Option<Node<AttributeSpecifierSequence<'a>>>,
    pub Node<AttributeSpecifier<'a>>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeSpecifier<'a> {
    pub open_bracket_0: At,
    pub open_bracket_1: At,
    pub attributes: Node<AttributeList<'a>>,
    pub close_bracket_0: At,
    pub close_bracket_1: At,
}

pub type AttributeList<'a> = CommaList<Option<Attribute<'a>>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute<'a>(
    pub Node<AttributeToken<'a>>,
    pub Option<Node<AttributeArgumentClause<'a>>>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttributeToken<'a> {
    Standard(Node<StandardAttribute<'a>>),
    Prefixed(Node<AttributePrefixedToken<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StandardAttribute<'a>(pub &'a str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributePrefixedToken<'a> {
    pub prefix: Node<AttributePrefix<'a>>,
    pub double_colon: At,
    pub token: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributePrefix<'a>(pub &'a str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeArgumentClause<'a> {
    pub open_parenthesis: At,
    pub tokens: Option<Node<BalancedTokenSequence<'a>>>,
    pub close_parenthesis: At,
}

pub type BalancedTokenSequence<'a> = List<BalancedToken<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BalancedToken<'a> {
    Parenthesized {
        open_parenthesis: At,
        tokens: Option<Node<BalancedTokenSequence<'a>>>,
        close_parenthesis: At,
    },
    Bracketed {
        open_bracket: At,
        tokens: Option<Node<BalancedTokenSequence<'a>>>,
        close_bracket: At,
    },
    Braced {
        open_brace: At,
        tokens: Option<Node<BalancedTokenSequence<'a>>>,
        close_brace: At,
    },
    Token(Token<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Statement<'a> {
    Labeled(Node<LabeledStatement<'a>>),
    Unlabeled(Node<UnlabeledStatement<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnlabeledStatement<'a> {
    Expression(Node<ExpressionStatement<'a>>),
    Primary(
        Option<Node<AttributeSpecifierSequence<'a>>>,
        Node<PrimaryBlock<'a>>,
    ),
    Jump(
        Option<Node<AttributeSpecifierSequence<'a>>>,
        Node<JumpStatement<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrimaryBlock<'a> {
    Compound(Node<CompoundStatement<'a>>),
    Selection(Node<SelectionStatement<'a>>),
    Iteration(Node<IterationStatement<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecondaryBlock<'a>(pub Node<Statement<'a>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Label<'a> {
    Named {
        attributes: Option<Node<AttributeSpecifierSequence<'a>>>,
        name: &'a str,
        colon: At,
    },
    Case {
        attributes: Option<Node<AttributeSpecifierSequence<'a>>>,
        case_keyword: At,
        value: Node<ConstantExpression<'a>>,
        colon: At,
    },
    Default {
        attributes: Option<Node<AttributeSpecifierSequence<'a>>>,
        default_keyword: At,
        colon: At,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabeledStatement<'a>(pub Node<Label<'a>>, pub Node<Statement<'a>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundStatement<'a> {
    pub open_brace: At,
    pub items: Option<Node<BlockItemList<'a>>>,
    pub close_brace: At,
}

pub type BlockItemList<'a> = List<BlockItem<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockItem<'a> {
    Declaration(Node<Declaration<'a>>),
    Unlabeled(Node<UnlabeledStatement<'a>>),
    Label(Node<Label<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionStatement<'a> {
    WithoutAttributes {
        expression: Option<Node<Expression<'a>>>,
        semicolon: At,
    },
    WithAttributes {
        attributes: Node<AttributeSpecifierSequence<'a>>,
        expression: Node<Expression<'a>>,
        semicolon: At,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionStatement<'a> {
    If {
        if_keyword: At,
        open_parenthesis: At,
        condition: Node<Expression<'a>>,
        close_parenthesis: At,
        then_body: Node<SecondaryBlock<'a>>,
    },
    IfElse {
        if_keyword: At,
        open_parenthesis: At,
        condition: Node<Expression<'a>>,
        close_parenthesis: At,
        then_body: Node<SecondaryBlock<'a>>,
        else_keyword: At,
        else_body: Node<SecondaryBlock<'a>>,
    },
    Switch {
        switch_keyword: At,
        open_parenthesis: At,
        selector: Node<Expression<'a>>,
        close_parenthesis: At,
        body: Node<Expression<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IterationStatement<'a> {
    While {
        while_keyword: At,
        open_parenthesis: At,
        condition: Node<Expression<'a>>,
        close_parenthesis: At,
        body: Node<SecondaryBlock<'a>>,
    },
    DoWhile {
        do_keyword: At,
        body: Node<SecondaryBlock<'a>>,
        while_keyword: At,
        open_parenthesis: At,
        condition: Node<Expression<'a>>,
        close_parenthesis: At,
        semicolon: At,
    },
    For {
        for_keyword: At,
        open_parenthesis: At,
        initializer: Option<Node<Expression<'a>>>,
        semicolon_0: At,
        condition: Option<Node<Expression<'a>>>,
        semicolon_1: At,
        counter: Option<Node<Expression<'a>>>,
        close_parenthesis: At,
        body: Node<SecondaryBlock<'a>>,
    },
    ForDeclaration {
        for_keyword: At,
        open_parenthesis: At,
        initializer: Node<Declaration<'a>>,
        condition: Option<Node<Expression<'a>>>,
        semicolon_1: At,
        counter: Option<Node<Expression<'a>>>,
        close_parenthesis: At,
        body: Node<SecondaryBlock<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JumpStatement<'a> {
    Goto {
        goto_keyword: At,
        target: &'a str,
        semicolon: At,
    },
    Continue {
        continue_keyword: At,
        semicolon: At,
    },
    Break {
        break_keyword: At,
        semicolon: At,
    },
    Return {
        return_keyword: At,
        value: Option<Node<Expression<'a>>>,
        semicolon: At,
    },
}

pub type TranslationUnit<'a> = List<ExternalDeclaration<'a>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalDeclaration<'a> {
    FunctionDefinition(Node<FunctionDefinition<'a>>),
    Declaration(Node<Declaration<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionDefinition<'a>(
    pub Option<Node<AttributeSpecifierSequence<'a>>>,
    pub Node<DeclarationSpecifiers<'a>>,
    pub Node<Declarator<'a>>,
    pub Node<FunctionBody<'a>>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionBody<'a>(pub Node<CompoundStatement<'a>>);
