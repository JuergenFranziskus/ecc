use std::rc::Rc;

use crate::token::{At, IntegerToken, TokenKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node<T>(pub Rc<(At, T)>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Constant<'a> {
    Integer(IntegerToken<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrimaryExpression<'a> {
    Identifier(&'a str),
    Constant(Node<Constant<'a>>),
    StringLiteral(StringLiteral<'a>),
    InParenthesis(Node<Expression<'a>>),
    Generic(Node<GenericSelection<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringLiteral<'a>(pub &'a str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericSelection<'a>(
    pub Node<AssignmentExpression<'a>>,
    pub Node<GenericAssocList<'a>>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenericAssocList<'a> {
    Leaf(Node<GenericAssociation<'a>>),
    Rec(Node<GenericAssocList<'a>>, Node<GenericAssociation<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenericAssociation<'a> {
    ForType(Node<TypeName<'a>>, Node<AssignmentExpression<'a>>),
    Default(Node<AssignmentExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PostfixExpression<'a> {
    Leaf(Node<PrimaryExpression<'a>>),
    Index(Node<PostfixExpression<'a>>, Node<Expression<'a>>),
    Call(
        Node<PostfixExpression<'a>>,
        Option<Node<ArgumentExpressionList<'a>>>,
    ),
    Member(Node<PostfixExpression<'a>>, &'a str),
    MemberIndirect(Node<PostfixExpression<'a>>, &'a str),
    PostIncrement(Node<PostfixExpression<'a>>),
    PostDecrement(Node<PostfixExpression<'a>>),
    Compound(Node<CompoundLiteral<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgumentExpressionList<'a> {
    Leaf(Node<AssignmentExpression<'a>>),
    Rec(
        Node<ArgumentExpressionList<'a>>,
        Node<AssignmentExpression<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundLiteral<'a>(
    pub Option<Node<StorageClassSpecifiers>>,
    pub Node<TypeName<'a>>,
    pub Node<BracedInitializer<'a>>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageClassSpecifiers {
    Leaf(Node<StorageClassSpecifier>),
    Rec(Node<StorageClassSpecifiers>, Node<StorageClassSpecifier>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryExpression<'a> {
    Leaf(Node<PostfixExpression<'a>>),
    PreIncrement(Node<UnaryExpression<'a>>),
    PostIncrement(Node<UnaryExpression<'a>>),
    Unary(Node<UnaryOperator>, Node<CastExpression<'a>>),
    SizeofExpr(Node<UnaryExpression<'a>>),
    SizeofType(Node<TypeName<'a>>),
    Alignof(Node<TypeName<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    AddressOf,
    Dereference,
    Plus,
    Minus,
    BitwiseNot,
    LogicalNot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CastExpression<'a> {
    Leaf(Node<UnaryExpression<'a>>),
    Cast(Node<TypeName<'a>>, Node<CastExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MultiplicativeExpression<'a> {
    Leaf(Node<CastExpression<'a>>),
    Multiply(Node<MultiplicativeExpression<'a>>, Node<CastExpression<'a>>),
    Divide(Node<MultiplicativeExpression<'a>>, Node<CastExpression<'a>>),
    Modulo(Node<MultiplicativeExpression<'a>>, Node<CastExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AddativeExpression<'a> {
    Leaf(Node<MultiplicativeExpression<'a>>),
    Add(
        Node<AddativeExpression<'a>>,
        Node<MultiplicativeExpression<'a>>,
    ),
    Subtract(
        Node<AddativeExpression<'a>>,
        Node<MultiplicativeExpression<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShiftExpression<'a> {
    Leaf(Node<AddativeExpression<'a>>),
    Left(Node<ShiftExpression<'a>>, Node<AddativeExpression<'a>>),
    Right(Node<ShiftExpression<'a>>, Node<AddativeExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationalExpression<'a> {
    Leaf(Node<ShiftExpression<'a>>),
    Less(Node<RelationalExpression<'a>>, Node<ShiftExpression<'a>>),
    Greater(Node<RelationalExpression<'a>>, Node<ShiftExpression<'a>>),
    LessEqual(Node<RelationalExpression<'a>>, Node<ShiftExpression<'a>>),
    GreaterEqual(Node<RelationalExpression<'a>>, Node<ShiftExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EqualityExpression<'a> {
    Leaf(Node<RelationalExpression<'a>>),
    Equal(Node<EqualityExpression<'a>>, Node<RelationalExpression<'a>>),
    NotEqual(Node<EqualityExpression<'a>>, Node<RelationalExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AndExpression<'a> {
    Leaf(Node<EqualityExpression<'a>>),
    And(Node<AndExpression<'a>>, Node<EqualityExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExclusiveOrExpression<'a> {
    Leaf(Node<AndExpression<'a>>),
    ExclusiveOr(Node<ExclusiveOrExpression<'a>>, Node<AndExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InclusiveOrExpression<'a> {
    Leaf(Node<ExclusiveOrExpression<'a>>),
    InclusiveOr(
        Node<InclusiveOrExpression<'a>>,
        Node<ExclusiveOrExpression<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogicalAndExpression<'a> {
    Leaf(Node<InclusiveOrExpression<'a>>),
    And(
        Node<LogicalAndExpression<'a>>,
        Node<InclusiveOrExpression<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogicalOrExpression<'a> {
    Leaf(Node<LogicalAndExpression<'a>>),
    And(
        Node<LogicalOrExpression<'a>>,
        Node<LogicalAndExpression<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConditionalExpression<'a> {
    Leaf(Node<LogicalOrExpression<'a>>),
    Conditional(
        Node<LogicalOrExpression<'a>>,
        Node<Expression<'a>>,
        Node<ConditionalExpression<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssignmentExpression<'a> {
    Leaf(Node<ConditionalExpression<'a>>),
    Assign(
        Node<UnaryExpression<'a>>,
        Node<AssignmentOperator>,
        Node<AssignmentExpression<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssignmentOperator {
    Assign,
    MulAssign,
    DivAssign,
    ModAssign,
    AddAssign,
    SubAssign,
    ShiftLeftAssign,
    ShiftRightAssign,
    AndAssign,
    XorAssign,
    OrAssign,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression<'a> {
    Leaf(Node<AssignmentExpression<'a>>),
    Comma(Node<Expression<'a>>, Node<AssignmentExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstantExpression<'a>(pub Node<ConditionalExpression<'a>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declaration<'a> {
    Normal(
        Node<DeclarationSpecifiers<'a>>,
        Option<Node<InitDeclaratorList<'a>>>,
    ),
    Attributive(
        Node<AttributeSpecifierSequence<'a>>,
        Node<DeclarationSpecifiers<'a>>,
        Node<InitDeclaratorList<'a>>,
    ),
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
    Type(Node<TypeSpecifierQualifier<'a>>),
    Function(Node<FunctionSpecifier>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitDeclaratorList<'a> {
    Leaf(Node<InitDeclarator<'a>>),
    Rec(Node<InitDeclaratorList<'a>>, Node<InitDeclarator<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitDeclarator<'a> {
    NoInit(Node<Declarator<'a>>),
    Init(Node<Declarator<'a>>, Node<Initializer<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeDeclaration<'a>(pub Node<AttributeSpecifierSequence<'a>>);

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
    Unsigned,
    BitInt(Node<ConstantExpression<'a>>),
    Bool,
    Complex,
    Decimal32,
    Decimal64,
    Decimal128,
    Atomic(Node<AtomicTypeSpecifier<'a>>),
    StructUnion(Node<StructOrUnionSpecifier<'a>>),
    Enum(Node<EnumSpecifier<'a>>),
    TypedefName(Node<TypedefName<'a>>),
    Typeof(Node<TypeofSpecifier<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructOrUnionSpecifier<'a> {
    WithMembers(
        Node<StructOrUnion>,
        Option<Node<AttributeSpecifierSequence<'a>>>,
        Option<&'a str>,
        Node<MemberDeclarationList<'a>>,
    ),
    NoMembers(
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberDeclarationList<'a> {
    Leaf(Node<MemberDeclaration<'a>>),
    Rec(Node<MemberDeclarationList<'a>>, Node<MemberDeclaration<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberDeclaration<'a> {
    Member(
        Option<Node<AttributeSpecifierSequence<'a>>>,
        Node<SpecifierQualifierList<'a>>,
        Option<Node<MemberDeclaratorList<'a>>>,
    ),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberDeclaratorList<'a> {
    Leaf(Node<MemberDeclarator<'a>>),
    Rec(Node<MemberDeclaratorList<'a>>, Node<MemberDeclarator<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberDeclarator<'a> {
    Normal(Node<Declarator<'a>>),
    Weird(Option<Node<Declarator<'a>>>, Node<ConstantExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnumSpecifier<'a> {
    WithList(
        Option<Node<AttributeSpecifierSequence<'a>>>,
        Option<&'a str>,
        Option<Node<EnumTypeSpecifier<'a>>>,
        Node<EnumeratorList<'a>>,
    ),
    NoList(&'a str, Option<Node<EnumTypeSpecifier<'a>>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnumeratorList<'a> {
    Leaf(Node<Enumerator<'a>>),
    Rec(Node<EnumeratorList<'a>>, Node<Enumerator<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Enumerator<'a> {
    Implicit(&'a str, Option<Node<AttributeSpecifierSequence<'a>>>),
    Explicit(
        &'a str,
        Option<Node<AttributeSpecifierSequence<'a>>>,
        Node<ConstantExpression<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumTypeSpecifier<'a>(pub Node<SpecifierQualifierList<'a>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtomicTypeSpecifier<'a>(pub Node<TypeName<'a>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeofSpecifier<'a> {
    Normal(Node<TypeofSpecifierArgument<'a>>),
    Unqual(Node<TypeofSpecifierArgument<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeofSpecifierArgument<'a> {
    Expression(Node<Expression<'a>>),
    TypeName(Node<TypeName<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeQualifier {
    Const,
    Restrict,
    Volative,
    Atomic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionSpecifier {
    Inline,
    NoReturn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlignmentSpecifier<'a> {
    AsType(Node<TypeName<'a>>),
    AsExpr(Node<ConstantExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declarator<'a>(
    pub Option<Node<Pointer<'a>>>,
    pub Node<DirectDeclarator<'a>>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectDeclarator<'a> {
    Name(&'a str, Option<Node<AttributeSpecifierSequence<'a>>>),
    InParenthesis(Node<Declarator<'a>>),
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
    Normal(
        Node<DirectDeclarator<'a>>,
        Option<Node<TypeQualifierList>>,
        Option<Node<AssignmentExpression<'a>>>,
    ),
    StaticFirst(
        Node<DirectDeclarator<'a>>,
        Option<Node<TypeQualifierList>>,
        Node<AssignmentExpression<'a>>,
    ),
    StaticMid(
        Node<DirectDeclarator<'a>>,
        Node<TypeQualifierList>,
        Node<AssignmentExpression<'a>>,
    ),
    Var(Node<DirectDeclarator<'a>>, Option<Node<TypeQualifierList>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionDeclarator<'a>(
    pub Node<DirectDeclarator<'a>>,
    pub Option<Node<ParameterTypeList<'a>>>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pointer<'a> {
    Leaf(
        Option<Node<AttributeSpecifierSequence<'a>>>,
        Option<Node<TypeQualifierList>>,
    ),
    Rec(
        Option<Node<AttributeSpecifierSequence<'a>>>,
        Option<Node<TypeQualifierList>>,
        Node<Pointer<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeQualifierList {
    Leaf(Node<TypeQualifier>),
    Rec(Node<TypeQualifierList>, Node<TypeQualifier>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParameterTypeList<'a> {
    NoVar(Node<ParameterList<'a>>),
    WithVar(Node<ParameterList<'a>>),
    OnlyVar,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParameterList<'a> {
    Leaf(Node<ParameterDeclaration<'a>>),
    Rec(Node<ParameterList<'a>>, Node<ParameterDeclaration<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParameterDeclaration<'a> {
    Specific(
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
    Leaf(Node<Pointer<'a>>),
    Rec(
        Option<Node<Pointer<'a>>>,
        Node<DirectAbstractDeclarator<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectAbstractDeclarator<'a> {
    InParenthesis(Node<AbstractDeclarator<'a>>),
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
    Normal(
        Node<DirectAbstractDeclarator<'a>>,
        Option<Node<TypeQualifierList>>,
        Option<Node<AssignmentExpression<'a>>>,
    ),
    StaticFirst(
        Node<DirectAbstractDeclarator<'a>>,
        Option<Node<TypeQualifierList>>,
        Node<AssignmentExpression<'a>>,
    ),
    StaticMid(
        Node<DirectAbstractDeclarator<'a>>,
        Node<TypeQualifierList>,
        Node<AssignmentExpression<'a>>,
    ),
    Var(
        Node<DirectAbstractDeclarator<'a>>,
        Option<Node<TypeQualifierList>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionAbstractDeclarator<'a>(
    pub Node<DirectAbstractDeclarator<'a>>,
    pub Option<Node<ParameterTypeList<'a>>>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedefName<'a>(pub &'a str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BracedInitializer<'a> {
    Empty,
    Normal(Node<InitializerList<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Initializer<'a> {
    Assign(Node<AssignmentExpression<'a>>),
    Braced(Node<BracedInitializer<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitializerList<'a> {
    Leaf(Option<Node<Designation<'a>>>, Node<Initializer<'a>>),
    Rec(
        Node<InitializerList<'a>>,
        Option<Node<Designation<'a>>>,
        Node<Initializer<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Designation<'a>(pub Node<DesignatorList<'a>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesignatorList<'a> {
    Leaf(Node<Designator<'a>>),
    Rec(Node<DesignatorList<'a>>, Node<Designator<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Designator<'a> {
    InSquareBrackets(Node<ConstantExpression<'a>>),
    AfterPeriod(&'a str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StaticAssertDeclaration<'a> {
    WithMessage(Node<ConstantExpression<'a>>, Node<StringLiteral<'a>>),
    NoMessage(Node<ConstantExpression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeSpecifierSequence<'a>(
    pub Option<Node<AttributeSpecifierSequence<'a>>>,
    pub Node<AttributeSpecifier<'a>>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeSpecifier<'a>(pub Node<AttributeList<'a>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttributeList<'a> {
    Leaf(Option<Node<Attribute<'a>>>),
    Rec(Node<AttributeList<'a>>, Option<Node<Attribute<'a>>>),
}

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
pub struct AttributePrefixedToken<'a>(pub Node<AttributePrefix<'a>>, pub &'a str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributePrefix<'a>(pub &'a str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeArgumentClause<'a>(pub Option<Node<BalancedTokenSequence<'a>>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BalancedTokenSequence<'a> {
    Leaf(Node<BalancedToken<'a>>),
    Rec(Node<BalancedTokenSequence<'a>>, Node<BalancedToken<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BalancedToken<'a> {
    Leaf(TokenKind<'a>),
    InParenthesis(Option<Node<BalancedTokenSequence<'a>>>),
    InSquareBrackets(Option<Node<BalancedTokenSequence<'a>>>),
    InCurlyBraces(Option<Node<BalancedTokenSequence<'a>>>),
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
    Named(Option<Node<AttributeSpecifierSequence<'a>>>, &'a str),
    Case(
        Option<Node<AttributeSpecifierSequence<'a>>>,
        Node<ConstantExpression<'a>>,
    ),
    Default(Option<Node<AttributeSpecifierSequence<'a>>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabeledStatement<'a>(pub Node<Label<'a>>, pub Node<Statement<'a>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundStatement<'a>(pub Option<Node<BlockItemList<'a>>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockItemList<'a> {
    Leaf(Node<BlockItem<'a>>),
    Rec(Node<BlockItemList<'a>>, Node<BlockItem<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockItem<'a> {
    Declaration(Node<Declaration<'a>>),
    Unlabeled(Node<UnlabeledStatement<'a>>),
    Label(Node<Label<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionStatement<'a> {
    Normal(Option<Node<Expression<'a>>>),
    WithAttributes(Node<AttributeSpecifierSequence<'a>>, Node<Expression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionStatement<'a> {
    If(Node<Expression<'a>>, Node<SecondaryBlock<'a>>),
    IfElse(
        Node<Expression<'a>>,
        Node<SecondaryBlock<'a>>,
        Node<SecondaryBlock<'a>>,
    ),
    Switch(Node<Expression<'a>>, Node<SecondaryBlock<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IterationStatement<'a> {
    While(Node<Expression<'a>>, Node<SecondaryBlock<'a>>),
    Do(Node<SecondaryBlock<'a>>, Node<Expression<'a>>),
    For(
        Option<Node<Expression<'a>>>,
        Option<Node<Expression<'a>>>,
        Option<Node<Expression<'a>>>,
        Node<SecondaryBlock<'a>>,
    ),
    ForWeird(
        Node<Declaration<'a>>,
        Option<Node<Expression<'a>>>,
        Option<Node<Expression<'a>>>,
        Node<SecondaryBlock<'a>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JumpStatement<'a> {
    Goto(&'a str),
    Continue,
    Break,
    Return(Node<Expression<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TranslationUnit<'a> {
    Leaf(Node<ExternalDeclaration<'a>>),
    Rec(Node<TranslationUnit<'a>>, Node<ExternalDeclaration<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalDeclaration<'a> {
    Function(Node<FunctionDefinition<'a>>),
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
