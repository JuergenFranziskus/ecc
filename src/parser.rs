use std::collections::HashSet;

use super::ast::*;
use crate::token::{At, Token, TokenKind};

pub struct Parser<'a, 'b> {
    tokens: &'b [Token<'a>],
    index: usize,
    errors: Vec<ParseErr<'a>>,
    scopes: Vec<HashSet<&'a str>>,
}
impl<'a, 'b> Parser<'a, 'b> {
    pub fn new(tokens: &'b [Token<'a>]) -> Self {
        Self {
            tokens,
            index: 0,
            errors: Vec::new(),
            scopes: Vec::new(),
        }
    }

    pub fn parse(mut self) -> (Result<TranslationUnit<'a>, ()>, Vec<ParseErr<'a>>) {
        let ast = self.parse_translation_unit();
        (ast, self.errors)
    }

    fn parse_primary_expression(&mut self) -> Res<Expression<'a>> {
        let at = self.at();
        let kind = match self.kind() {
            TokenKind::Identifier(name) => {
                self.next();
                ExpressionKind::Identifier(name)
            }
            TokenKind::Integer(int) => {
                self.next();
                ExpressionKind::Integer(int)
            }
            TokenKind::String(literal, encoding) => {
                self.next();
                ExpressionKind::String(StringLiteral {
                    at,
                    literal,
                    encoding,
                })
            }
            TokenKind::OpenParenthesis => {
                let open_parenthesis = self.next();
                let inner = Box::new(self.parse_expression()?);
                let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
                ExpressionKind::Parenthesized {
                    open_parenthesis,
                    inner,
                    close_parenthesis,
                }
            }
            TokenKind::Generic => ExpressionKind::GenericSelection(self.parse_generic_selection()?),
            _ => {
                self.err(Expected::PrimaryExpression);
                return Err(());
            }
        };
        Ok(Expression { at, kind })
    }
    fn parse_generic_selection(&mut self) -> Res<GenericSelection<'a>> {
        let at = self.at();
        let generic_keyword = self.take(TokenKind::Generic)?;
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let controlling_expression = Box::new(self.parse_assignment_expression()?);
        let comma = self.take(TokenKind::Comma)?;
        let generic_assocs = self.parse_generic_assoc_list()?;
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

        Ok(GenericSelection {
            at,
            generic_keyword,
            open_parenthesis,
            controlling_expression,
            comma,
            generic_assocs,
            close_parenthesis,
        })
    }
    fn parse_generic_assoc_list(&mut self) -> Res<GenericAssocList<'a>> {
        self.comma_list(Self::parse_generic_association)
    }
    fn parse_generic_association(&mut self) -> Res<GenericAssociation<'a>> {
        let at = self.at();
        let kind = if self.is(TokenKind::Default) {
            GenericAssociationKind::Default {
                default_keyword: self.next(),
            }
        } else {
            GenericAssociationKind::ForType(self.parse_type_name()?)
        };
        let colon = self.take(TokenKind::Colon)?;
        let value = self.parse_assignment_expression()?;

        Ok(GenericAssociation {
            at,
            colon,
            kind,
            value,
        })
    }

    fn parse_postfix_expression(&mut self) -> Res<Expression<'a>> {
        let mut left = self.parse_postfix_expression_leaf()?;

        loop {
            let at = left.at;
            let kind = match self.kind() {
                TokenKind::OpenBracket => {
                    let open_bracket = self.next();
                    let index = Box::new(self.parse_expression()?);
                    let close_bracket = self.take(TokenKind::CloseBracket)?;
                    ExpressionKind::Index {
                        left: Box::new(left),
                        open_bracket,
                        index,
                        close_bracket,
                    }
                }
                TokenKind::OpenParenthesis => {
                    let open_parenthesis = self.next();
                    let arguments = self.maybe(Self::parse_argument_expression_list);
                    let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
                    ExpressionKind::Call {
                        left: Box::new(left),
                        open_parenthesis,
                        arguments,
                        close_parenthesis,
                    }
                }
                TokenKind::Period => {
                    let period = self.next();
                    let name = self.take_identifier()?;
                    ExpressionKind::Member {
                        left: Box::new(left),
                        period,
                        name,
                    }
                }
                TokenKind::ArrowLeft => {
                    let arrow = self.next();
                    let name = self.take_identifier()?;
                    ExpressionKind::MemberIndirect {
                        left: Box::new(left),
                        arrow,
                        name,
                    }
                }
                TokenKind::DoublePlus => {
                    let double_plus = self.next();
                    ExpressionKind::PostIncrement {
                        left: Box::new(left),
                        double_plus,
                    }
                }
                TokenKind::DoubleMinus => {
                    let double_minus = self.next();
                    ExpressionKind::PostDecrement {
                        left: Box::new(left),
                        double_minus,
                    }
                }
                _ => break,
            };
            left = Expression { at, kind };
        }

        Ok(left)
    }
    fn parse_postfix_expression_leaf(&mut self) -> Res<Expression<'a>> {
        self.one_of(
            [
                &mut Self::parse_compound_literal_expression,
                &mut Self::parse_primary_expression,
            ],
            Expected::PrimaryExpression,
        )
    }

    fn parse_argument_expression_list(&mut self) -> Res<ArgumentExpressionList<'a>> {
        self.comma_list(Self::parse_assignment_expression)
    }

    fn parse_compound_literal_expression(&mut self) -> Res<Expression<'a>> {
        let literal = self.parse_compound_literal()?;
        Ok(Expression {
            at: literal.at,
            kind: ExpressionKind::CompoundLiteral(literal),
        })
    }
    fn parse_compound_literal(&mut self) -> Res<CompoundLiteral<'a>> {
        let at = self.at();
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let storage_class = self.maybe(|p| Self::parse_storage_class_specifiers(p, &mut false));
        let type_name = self.parse_type_name()?;
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
        let initializer = self.parse_braced_initializer()?;
        Ok(CompoundLiteral {
            at,
            open_parenthesis,
            storage_class,
            type_name,
            close_parenthesis,
            initializer,
        })
    }
    fn parse_storage_class_specifiers(
        &mut self,
        is_typedef: &mut bool,
    ) -> Res<StorageClassSpecifiers> {
        self.list(|p| Self::parse_storage_class_specifier(p, is_typedef))
    }

    fn parse_unary_expression(&mut self) -> Res<Expression<'a>> {
        let at = self.at();
        let kind = match self.kind() {
            TokenKind::DoublePlus => {
                let double_plus = self.next();
                let right = self.parse_unary_expression()?;
                ExpressionKind::PreIncrement {
                    double_plus,
                    right: Box::new(right),
                }
            }
            TokenKind::DoubleMinus => {
                let double_minus = self.next();
                let right = self.parse_unary_expression()?;
                ExpressionKind::PreDecrement {
                    double_minus,
                    right: Box::new(right),
                }
            }
            TokenKind::Ampersand
            | TokenKind::Asterisk
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Tilde
            | TokenKind::Exclamation => {
                let operator = match self.kind() {
                    TokenKind::Ampersand => UnaryOperator::AddressOf,
                    TokenKind::Asterisk => UnaryOperator::Dereference,
                    TokenKind::Plus => UnaryOperator::Positive,
                    TokenKind::Minus => UnaryOperator::Negative,
                    TokenKind::Tilde => UnaryOperator::BitNot,
                    TokenKind::Exclamation => UnaryOperator::LogicalNot,
                    _ => unreachable!(),
                };
                self.next();
                let right = self.parse_cast_expression()?;
                ExpressionKind::Unary(operator, Box::new(right))
            }
            TokenKind::Sizeof => {
                let sizeof_keyword = self.next();
                let kind = if let Ok(expr) = self.try_to(Self::parse_unary_expression) {
                    SizeofKind::Expression(Box::new(expr))
                } else {
                    let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
                    let type_name = self.parse_type_name()?;
                    let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
                    SizeofKind::Type {
                        open_parenthesis,
                        type_name,
                        close_parenthesis,
                    }
                };

                ExpressionKind::Sizeof {
                    sizeof_keyword,
                    kind,
                }
            }
            TokenKind::Alignof => {
                let alignof_keyword = self.next();
                let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
                let type_name = self.parse_type_name()?;
                let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
                ExpressionKind::Alignof {
                    alignof_keyword,
                    open_parenthesis,
                    type_name,
                    close_parenthesis,
                }
            }
            _ => return self.parse_postfix_expression(),
        };

        Ok(Expression { at, kind })
    }
    fn parse_cast_expression(&mut self) -> Res<Expression<'a>> {
        if let Ok(e) = self.try_to(Self::parse_cast_expression_prime) {
            Ok(e)
        } else {
            self.parse_unary_expression()
        }
    }
    fn parse_cast_expression_prime(&mut self) -> Res<Expression<'a>> {
        let at = self.at();
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let type_name = self.parse_type_name()?;
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
        let right = self.parse_cast_expression()?;

        Ok(Expression {
            at,
            kind: ExpressionKind::Cast {
                open_parenthesis,
                type_name,
                close_parenthesis,
                right: Box::new(right),
            },
        })
    }

    fn parse_multiplicative_expression(&mut self) -> Res<Expression<'a>> {
        self.parse_binary_expression(
            Self::parse_cast_expression,
            &[
                (TokenKind::Asterisk, BinaryOperator::Multiply),
                (TokenKind::Slash, BinaryOperator::Divide),
                (TokenKind::Percent, BinaryOperator::Modulo),
            ],
        )
    }
    fn parse_additive_expression(&mut self) -> Res<Expression<'a>> {
        self.parse_binary_expression(
            Self::parse_multiplicative_expression,
            &[
                (TokenKind::Plus, BinaryOperator::Add),
                (TokenKind::Minus, BinaryOperator::Subtract),
            ],
        )
    }
    fn parse_shift_expression(&mut self) -> Res<Expression<'a>> {
        self.parse_binary_expression(
            Self::parse_additive_expression,
            &[
                (TokenKind::DoubleLess, BinaryOperator::ShiftLeft),
                (TokenKind::DoubleGreater, BinaryOperator::ShiftRight),
            ],
        )
    }
    fn parse_relational_expression(&mut self) -> Res<Expression<'a>> {
        self.parse_binary_expression(
            Self::parse_shift_expression,
            &[
                (TokenKind::Less, BinaryOperator::Less),
                (TokenKind::Greater, BinaryOperator::Greater),
                (TokenKind::LessEqual, BinaryOperator::LessEqual),
                (TokenKind::GreaterEqual, BinaryOperator::GreaterEqual),
            ],
        )
    }
    fn parse_equality_expression(&mut self) -> Res<Expression<'a>> {
        self.parse_binary_expression(
            Self::parse_relational_expression,
            &[
                (TokenKind::DoubleEqual, BinaryOperator::Equal),
                (TokenKind::NotEqual, BinaryOperator::NotEqual),
            ],
        )
    }
    fn parse_and_expression(&mut self) -> Res<Expression<'a>> {
        self.parse_binary_expression(
            Self::parse_equality_expression,
            &[(TokenKind::Ampersand, BinaryOperator::BitAnd)],
        )
    }
    fn parse_xor_expression(&mut self) -> Res<Expression<'a>> {
        self.parse_binary_expression(
            Self::parse_and_expression,
            &[(TokenKind::Caret, BinaryOperator::BitXor)],
        )
    }
    fn parse_or_expression(&mut self) -> Res<Expression<'a>> {
        self.parse_binary_expression(
            Self::parse_xor_expression,
            &[(TokenKind::Bar, BinaryOperator::BitOr)],
        )
    }
    fn parse_logical_and_expression(&mut self) -> Res<Expression<'a>> {
        self.parse_binary_expression(
            Self::parse_or_expression,
            &[(TokenKind::DoubleAmpersand, BinaryOperator::LogicalAnd)],
        )
    }
    fn parse_logical_or_expression(&mut self) -> Res<Expression<'a>> {
        self.parse_binary_expression(
            Self::parse_logical_and_expression,
            &[(TokenKind::DoubleBar, BinaryOperator::LogicalOr)],
        )
    }
    fn parse_conditional_expression(&mut self) -> Res<Expression<'a>> {
        let at = self.at();
        let left = self.parse_logical_or_expression()?;
        if !self.is(TokenKind::Question) {
            return Ok(left);
        }

        let condition = Box::new(left);
        let question = self.next();
        let then_value = Box::new(self.parse_expression()?);
        let colon = self.take(TokenKind::Colon)?;
        let else_value = Box::new(self.parse_conditional_expression()?);

        Ok(Expression {
            at,
            kind: ExpressionKind::Conditional {
                condition,
                question,
                then_value,
                colon,
                else_value,
            },
        })
    }
    fn parse_assignment_expression(&mut self) -> Res<Expression<'a>> {
        if let Ok(e) = self.try_to(Self::parse_assignment_expression_prime) {
            Ok(e)
        } else {
            self.parse_conditional_expression()
        }
    }
    fn parse_assignment_expression_prime(&mut self) -> Res<Expression<'a>> {
        let at = self.at();
        let left = Box::new(self.parse_unary_expression()?);
        let operator = match self.kind() {
            TokenKind::Equal => AssignmentOperator::Assign,
            TokenKind::AsteriskEqual => AssignmentOperator::Multiply,
            TokenKind::SlashEqual => AssignmentOperator::Divide,
            TokenKind::PercentEqual => AssignmentOperator::Modulo,
            TokenKind::PlusEqual => AssignmentOperator::Add,
            TokenKind::MinusEqual => AssignmentOperator::Subtract,
            TokenKind::DoubleLessEqual => AssignmentOperator::ShiftLeft,
            TokenKind::DoubleGreaterEqual => AssignmentOperator::ShiftRight,
            TokenKind::AmpersandEqual => AssignmentOperator::And,
            TokenKind::CaretEqual => AssignmentOperator::Xor,
            TokenKind::BarEqual => AssignmentOperator::Or,
            _ => {
                self.err(Expected::AssignmentOperator);
                return Err(());
            }
        };
        let operator_at = self.next();
        let right = Box::new(self.parse_assignment_expression()?);

        Ok(Expression {
            at,
            kind: ExpressionKind::Assign {
                left,
                operator: (operator_at, operator),
                right,
            },
        })
    }
    fn parse_expression(&mut self) -> Res<Expression<'a>> {
        let mut left = self.parse_assignment_expression()?;

        loop {
            if !self.is(TokenKind::Comma) {
                break;
            }
            let comma = self.next();
            let right = Box::new(self.parse_assignment_expression()?);
            left = Expression {
                at: left.at,
                kind: ExpressionKind::Comma {
                    left: Box::new(left),
                    comma,
                    right,
                },
            }
        }

        Ok(left)
    }
    fn parse_constant_expression(&mut self) -> Res<Expression<'a>> {
        self.parse_conditional_expression()
    }

    fn parse_declaration(&mut self) -> Res<Declaration<'a>> {
        let mut is_typedef = false;

        let at = self.at();
        let kind = if let Ok(assert) = self.try_to(Self::parse_static_assert_declaration) {
            DeclarationKind::Assert(assert)
        } else if let Ok(attribute) = self.try_to(Self::parse_attribute_declaration) {
            DeclarationKind::Attribute(attribute)
        } else if let Ok(attribute) = self.try_to(Self::parse_attribute_specifier_sequence) {
            let specifiers = self.parse_declaration_specifiers(&mut is_typedef)?;
            let init_declarators = self.parse_init_declarator_list(is_typedef)?;
            let semicolon = self.take(TokenKind::Semicolon)?;

            DeclarationKind::Normal {
                attributes: Some(attribute),
                specifiers,
                init_declarators: Some(init_declarators),
                semicolon,
            }
        } else {
            let specifiers = self.parse_declaration_specifiers(&mut is_typedef)?;
            let init_declarators = self.maybe(|p| Self::parse_init_declarator_list(p, is_typedef));
            let semicolon = self.take(TokenKind::Semicolon)?;
            DeclarationKind::Normal {
                attributes: None,
                specifiers,
                init_declarators,
                semicolon,
            }
        };

        Ok(Declaration { at, kind })
    }
    fn parse_declaration_specifiers(
        &mut self,
        is_typedef: &mut bool,
    ) -> Res<DeclarationSpecifiers<'a>> {
        let at = self.at();
        let specifier = self.parse_declaration_specifier(is_typedef)?;
        let kind =
            if let Ok(cons) = self.try_to(|p| Self::parse_declaration_specifiers(p, is_typedef)) {
                DeclarationSpecifiersKind::Cons(Box::new(cons))
            } else {
                let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
                DeclarationSpecifiersKind::Leaf(attributes)
            };

        Ok(DeclarationSpecifiers {
            at,
            specifier,
            kind,
        })
    }
    fn parse_declaration_specifier(
        &mut self,
        is_typedef: &mut bool,
    ) -> Res<DeclarationSpecifier<'a>> {
        self.one_of(
            [
                &mut |p| Ok(p.parse_storage_class_specifier(is_typedef)?.into()),
                &mut |p| Ok(p.parse_type_specifier_qualifier()?.into()),
                &mut |p| Ok(p.parse_function_specifier()?.into()),
            ],
            Expected::DeclarationSpecifier,
        )
    }
    fn parse_init_declarator_list(&mut self, is_typedef: bool) -> Res<InitDeclaratorList<'a>> {
        self.comma_list(|p| Self::parse_init_declarator(p, is_typedef))
    }
    fn parse_init_declarator(&mut self, is_typedef: bool) -> Res<InitDeclarator<'a>> {
        let at = self.at();
        let declarator = self.parse_declarator(is_typedef)?;
        let initializer = if self.is(TokenKind::Equal) {
            let equal = self.next();
            let initializer = self.parse_initializer()?;
            Some((equal, initializer))
        } else {
            None
        };

        Ok(InitDeclarator {
            at,
            declarator,
            initializer,
        })
    }
    fn parse_attribute_declaration(&mut self) -> Res<AttributeDeclaration<'a>> {
        let at = self.at();
        let attributes = self.parse_attribute_specifier_sequence()?;
        let semicolon = self.take(TokenKind::Semicolon)?;
        Ok(AttributeDeclaration {
            at,
            attributes,
            semicolon,
        })
    }
    fn parse_storage_class_specifier(
        &mut self,
        is_typedef: &mut bool,
    ) -> Res<StorageClassSpecifier> {
        let at = self.at();
        let kind = match self.kind() {
            TokenKind::Auto => StorageClassSpecifierKind::Auto,
            TokenKind::Constexpr => StorageClassSpecifierKind::Constexpr,
            TokenKind::Extern => StorageClassSpecifierKind::Extern,
            TokenKind::Register => StorageClassSpecifierKind::Register,
            TokenKind::Static => StorageClassSpecifierKind::Static,
            TokenKind::ThreadLocal => StorageClassSpecifierKind::ThreadLocal,
            TokenKind::Typedef => {
                *is_typedef = true;
                StorageClassSpecifierKind::Typedef
            }
            _ => {
                self.err(Expected::StorageClassSpecifier);
                return Err(());
            }
        };
        self.next();

        Ok(StorageClassSpecifier { at, kind })
    }
    fn parse_type_specifier(&mut self) -> Res<TypeSpecifier<'a>> {
        let at = self.at();
        let kind = match self.kind() {
            TokenKind::Void => {
                self.next();
                TypeSpecifierKind::Void
            }
            TokenKind::Char => {
                self.next();
                TypeSpecifierKind::Char
            }
            TokenKind::Short => {
                self.next();
                TypeSpecifierKind::Short
            }
            TokenKind::Int => {
                self.next();
                TypeSpecifierKind::Int
            }
            TokenKind::Long => {
                self.next();
                TypeSpecifierKind::Long
            }
            TokenKind::Float => {
                self.next();
                TypeSpecifierKind::Float
            }
            TokenKind::Double => {
                self.next();
                TypeSpecifierKind::Double
            }
            TokenKind::Signed => {
                self.next();
                TypeSpecifierKind::Signed
            }
            TokenKind::Unsigned => {
                self.next();
                TypeSpecifierKind::Unsigned
            }
            TokenKind::Bool => {
                self.next();
                TypeSpecifierKind::Bool
            }
            TokenKind::Complex => {
                self.next();
                TypeSpecifierKind::Complex
            }
            TokenKind::Decimal32 => {
                self.next();
                TypeSpecifierKind::Decimal32
            }
            TokenKind::Decimal64 => {
                self.next();
                TypeSpecifierKind::Decimal64
            }
            TokenKind::Decimal128 => {
                self.next();
                TypeSpecifierKind::Decimal128
            }
            TokenKind::Identifier(name) => {
                if !self.is_typedef_name(name) {
                    self.err(Expected::TypeSpecifier);
                    return Err(());
                }
                self.next();
                TypeSpecifierKind::TypedefName(name)
            }
            TokenKind::BitInt => {
                let bitint_keyword = self.next();
                let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
                let width = self.parse_constant_expression()?;
                let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
                TypeSpecifierKind::BitInt {
                    bitint_keyword,
                    open_parenthesis,
                    width,
                    close_parenthesis,
                }
            }
            _ => self.one_of(
                [
                    &mut |p| Ok(p.parse_atomic_type_specifier()?.into()),
                    &mut |p| Ok(p.parse_struct_or_union_specifier()?.into()),
                    &mut |p| Ok(p.parse_enum_specifier()?.into()),
                    &mut |p| Ok(p.parse_typeof_specifier()?.into()),
                ],
                Expected::TypeSpecifier,
            )?,
        };

        Ok(TypeSpecifier { at, kind })
    }
    fn parse_struct_or_union_specifier(&mut self) -> Res<StructOrUnionSpecifier<'a>> {
        let at = self.at();
        let struct_or_union = (at, self.parse_struct_or_union()?);
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        let tag = self.maybe(Self::take_identifier);
        let members = if self.is(TokenKind::OpenBrace) || tag.is_none() {
            let open_brace = self.take(TokenKind::OpenBrace)?;
            let members = self.parse_member_declaration_list()?;
            let close_brace = self.take(TokenKind::CloseBrace)?;
            Some((open_brace, members, close_brace))
        } else {
            None
        };

        Ok(StructOrUnionSpecifier {
            at,
            struct_or_union,
            attributes,
            tag,
            members,
        })
    }
    fn parse_struct_or_union(&mut self) -> Res<StructOrUnion> {
        let out = match self.kind() {
            TokenKind::Struct => StructOrUnion::Struct,
            TokenKind::Union => StructOrUnion::Union,
            _ => {
                self.err(Expected::StructOrUnion);
                return Err(());
            }
        };
        self.next();
        Ok(out)
    }
    fn parse_member_declaration_list(&mut self) -> Res<MemberDeclarationList<'a>> {
        self.list(Self::parse_member_declaration)
    }
    fn parse_member_declaration(&mut self) -> Res<MemberDeclaration<'a>> {
        if let Ok(assert) = self.try_to(Self::parse_static_assert_declaration) {
            Ok(MemberDeclaration {
                at: assert.at,
                kind: MemberDeclarationKind::Assert(assert),
            })
        } else {
            let at = self.at();
            let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
            let specifier_qualifiers = self.parse_specifier_qualifier_list()?;
            let member_declarators = self.maybe(Self::parse_member_declarator_list);
            let semicolon = self.take(TokenKind::Semicolon)?;
            Ok(MemberDeclaration {
                at,
                kind: MemberDeclarationKind::Member {
                    attributes,
                    specifier_qualifiers,
                    member_declarators,
                    semicolon,
                },
            })
        }
    }
    fn parse_specifier_qualifier_list(&mut self) -> Res<SpecifierQualifierList<'a>> {
        let at = self.at();
        let specifier_qualifier = Box::new(self.parse_type_specifier_qualifier()?);
        let kind = if let Ok(cons) = self.try_to(Self::parse_specifier_qualifier_list) {
            SpecifierQualifierListKind::Cons(Box::new(cons))
        } else {
            let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
            SpecifierQualifierListKind::Leaf(attributes)
        };

        Ok(SpecifierQualifierList {
            at,
            specifier_qualifier,
            kind,
        })
    }
    fn parse_type_specifier_qualifier(&mut self) -> Res<TypeSpecifierQualifier<'a>> {
        self.one_of(
            [
                &mut |p| Ok(p.parse_type_specifier()?.into()),
                &mut |p| Ok(p.parse_type_qualifier()?.into()),
                &mut |p| Ok(p.parse_alignment_specifier()?.into()),
            ],
            Expected::TypeSpecifierQualifier,
        )
    }
    fn parse_member_declarator_list(&mut self) -> Res<MemberDeclaratorList<'a>> {
        self.comma_list(Self::parse_member_declarator)
    }
    fn parse_member_declarator(&mut self) -> Res<MemberDeclarator<'a>> {
        if let Ok(m) = self.try_to(Self::parse_member_declarator_prime) {
            return Ok(m);
        }

        let declarator = self.parse_declarator(false)?;

        Ok(MemberDeclarator {
            at: declarator.at,
            declarator: Some(declarator),
            width: None,
        })
    }
    fn parse_member_declarator_prime(&mut self) -> Res<MemberDeclarator<'a>> {
        let at = self.at();
        let declarator = self.maybe(|p| Self::parse_declarator(p, false));
        let colon = self.take(TokenKind::Colon)?;
        let width = self.parse_constant_expression()?;

        Ok(MemberDeclarator {
            at,
            declarator,
            width: Some((colon, width)),
        })
    }
    fn parse_enum_specifier(&mut self) -> Res<EnumSpecifier<'a>> {
        let at = self.at();
        let enum_keyword = self.take(TokenKind::Enum)?;
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        let tag_at = self.cur();
        let tag = self.maybe(Self::take_identifier);
        let enum_type = self.maybe(Self::parse_enum_type_specifier);

        let enumerators = if self.is(TokenKind::OpenBrace) {
            let open_brace = self.next();
            let enumerators = self.parse_enumerator_list()?;
            let final_comma = self.maybe(|p| p.take(TokenKind::Comma));
            let close_brace = self.take(TokenKind::CloseBrace)?;

            Some((open_brace, enumerators, final_comma, close_brace))
        } else {
            if tag.is_none() {
                self.err_at(tag_at, Expected::Identifier);
            }
            None
        };

        Ok(EnumSpecifier {
            at,
            enum_keyword,
            attributes,
            tag,
            enum_type,
            enumerators,
        })
    }
    fn parse_enumerator_list(&mut self) -> Res<EnumeratorList<'a>> {
        self.comma_list(Self::parse_enumerator)
    }
    fn parse_enumerator(&mut self) -> Res<Enumerator<'a>> {
        let at = self.at();
        let name = self.take_identifier()?;
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        let value = if self.is(TokenKind::Equal) {
            let equal = self.next();
            let value = self.parse_constant_expression()?;
            Some((equal, value))
        } else {
            None
        };

        Ok(Enumerator {
            at,
            name,
            attributes,
            value,
        })
    }
    fn parse_enum_type_specifier(&mut self) -> Res<EnumTypeSpecifier<'a>> {
        let at = self.at();
        let colon = self.take(TokenKind::Colon)?;
        let specifier_qualifiers = self.parse_specifier_qualifier_list()?;

        Ok(EnumTypeSpecifier {
            at,
            colon,
            specifier_qualifiers,
        })
    }
    fn parse_atomic_type_specifier(&mut self) -> Res<AtomicTypeSpecifier<'a>> {
        let at = self.at();
        let atomic_keyword = self.take(TokenKind::Atomic)?;
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let type_name = self.parse_type_name()?;
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

        Ok(AtomicTypeSpecifier {
            at,
            atomic_keyword,
            open_parenthesis,
            type_name,
            close_parenthesis,
        })
    }
    fn parse_typeof_specifier(&mut self) -> Res<TypeofSpecifier<'a>> {
        let at = self.at();
        let (typeof_keyword, unqual) = if self.is(TokenKind::TypeofUnqual) {
            (self.next(), true)
        } else {
            (self.take(TokenKind::Typeof)?, false)
        };

        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let argument = self.parse_typeof_specifier_argument()?;
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

        Ok(TypeofSpecifier {
            at,
            typeof_keyword,
            unqual,
            open_parenthesis,
            argument,
            close_parenthesis,
        })
    }
    fn parse_typeof_specifier_argument(&mut self) -> Res<TypeofSpecifierArgument<'a>> {
        let at = self.at();
        let kind = self.one_of(
            [
                &mut |p| {
                    Ok(TypeofSpecifierArgumentKind::Expression(
                        p.parse_expression()?,
                    ))
                },
                &mut |p| Ok(TypeofSpecifierArgumentKind::Type(p.parse_type_name()?)),
            ],
            Expected::TypeofSpecifierArgument,
        )?;

        Ok(TypeofSpecifierArgument { at, kind })
    }
    fn parse_type_qualifier(&mut self) -> Res<TypeQualifier> {
        let at = self.at();
        let kind = match self.kind() {
            TokenKind::Const => TypeQualifierKind::Const,
            TokenKind::Restrict => TypeQualifierKind::Restrict,
            TokenKind::Volatile => TypeQualifierKind::Volatile,
            TokenKind::Atomic => TypeQualifierKind::Atomic,
            _ => {
                self.err(Expected::TypeQualifier);
                return Err(());
            }
        };

        Ok(TypeQualifier { at, kind })
    }
    fn parse_function_specifier(&mut self) -> Res<FunctionSpecifier> {
        let at = self.at();
        let kind = match self.kind() {
            TokenKind::Inline => FunctionSpecifierKind::Inline,
            TokenKind::Noreturn => FunctionSpecifierKind::NoReturn,
            _ => {
                self.err(Expected::TypeQualifier);
                return Err(());
            }
        };

        Ok(FunctionSpecifier { at, kind })
    }
    fn parse_alignment_specifier(&mut self) -> Res<AlignmentSpecifier<'a>> {
        let at = self.at();
        let alignas_keyword = self.take(TokenKind::Alignas)?;
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let kind = self.one_of(
            [
                &mut |p| {
                    Ok(AlignmentSpecifierKind::Expression(
                        p.parse_constant_expression()?,
                    ))
                },
                &mut |p| Ok(AlignmentSpecifierKind::Type(p.parse_type_name()?)),
            ],
            Expected::AlignasArgument,
        )?;

        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

        Ok(AlignmentSpecifier {
            at,
            alignas_keyword,
            open_parenthesis,
            kind,
            close_parenthesis,
        })
    }
    fn parse_declarator(&mut self, is_typedef: bool) -> Res<Declarator<'a>> {
        let at = self.at();
        let pointer = self.maybe(Self::parse_pointer);
        let direct = self.parse_direct_declarator(is_typedef)?;
        Ok(Declarator {
            at,
            pointer,
            direct,
        })
    }
    fn parse_direct_declarator(&mut self, is_typedef: bool) -> Res<DirectDeclarator<'a>> {
        let mut left = self.parse_direct_declarator_leaf(is_typedef)?;
        loop {
            match self.kind() {
                TokenKind::OpenBracket => left = self.parse_array_declarator(left)?,
                TokenKind::OpenParenthesis => left = self.parse_function_declarator(left)?,
                _ => break,
            }
        }

        Ok(left)
    }
    fn parse_direct_declarator_leaf(&mut self, is_typedef: bool) -> Res<DirectDeclarator<'a>> {
        let at = self.at();
        if self.is(TokenKind::OpenParenthesis) {
            let open_parenthesis = self.next();
            let inner = Box::new(self.parse_declarator(is_typedef)?);
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            Ok(DirectDeclarator {
                at,
                kind: DirectDeclaratorKind::Parenthesized {
                    open_parenthesis,
                    inner,
                    close_parenthesis,
                },
            })
        } else {
            let name = self.take_identifier()?;
            let attributes = self.maybe(Self::parse_attribute_specifier_sequence);

            if is_typedef {
                self.scopes.last_mut().unwrap().insert(name);
            }

            Ok(DirectDeclarator {
                at,
                kind: DirectDeclaratorKind::Name(name, attributes),
            })
        }
    }
    fn parse_array_declarator(&mut self, left: DirectDeclarator<'a>) -> Res<DirectDeclarator<'a>> {
        let at = self.at();
        let left = Box::new(left);
        let open_bracket = self.take(TokenKind::OpenBracket)?;
        let array = if self.is(TokenKind::Static) {
            let static_keyword = Some(self.next());
            let qualifiers = self.maybe(Self::parse_type_qualifier_list);
            let size = Some(self.parse_assignment_expression()?);
            let close_bracket = self.take(TokenKind::CloseBracket)?;
            ArrayDeclarator {
                at,
                left,
                open_bracket,
                qualifiers,
                kind: ArrayDeclaratorKind::Normal {
                    static_keyword,
                    size,
                },
                close_bracket,
            }
        } else {
            let qualifiers = self.maybe(Self::parse_type_qualifier_list);
            if qualifiers.is_some() && self.is(TokenKind::Static) {
                let static_keyword = Some(self.next());
                let size = Some(self.parse_assignment_expression()?);
                let close_bracket = self.take(TokenKind::CloseBracket)?;
                ArrayDeclarator {
                    at,
                    left,
                    open_bracket,
                    qualifiers,
                    kind: ArrayDeclaratorKind::Normal {
                        static_keyword,
                        size,
                    },
                    close_bracket,
                }
            } else if self.is(TokenKind::Asterisk) {
                let asterisk = self.next();
                let close_bracket = self.take(TokenKind::CloseBracket)?;
                ArrayDeclarator {
                    at,
                    left,
                    open_bracket,
                    qualifiers,
                    kind: ArrayDeclaratorKind::Var { asterisk },
                    close_bracket,
                }
            } else {
                let size = self.maybe(Self::parse_assignment_expression);
                let close_bracket = self.take(TokenKind::CloseBracket)?;
                ArrayDeclarator {
                    at,
                    left,
                    open_bracket,
                    qualifiers,
                    kind: ArrayDeclaratorKind::Normal {
                        static_keyword: None,
                        size,
                    },
                    close_bracket,
                }
            }
        };

        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        Ok(DirectDeclarator {
            at,
            kind: DirectDeclaratorKind::Array(array, attributes),
        })
    }
    fn parse_function_declarator(
        &mut self,
        left: DirectDeclarator<'a>,
    ) -> Res<DirectDeclarator<'a>> {
        let at = self.at();
        let left = Box::new(left);
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let parameters = self.maybe(Self::parse_parameter_type_list);
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);

        Ok(DirectDeclarator {
            at,
            kind: DirectDeclaratorKind::Function(
                FunctionDeclarator {
                    at,
                    left,
                    open_parenthesis,
                    parameters,
                    close_parenthesis,
                },
                attributes,
            ),
        })
    }
    fn parse_pointer(&mut self) -> Res<Pointer<'a>> {
        let at = self.at();
        let asterisk = self.take(TokenKind::Asterisk)?;
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        let qualifiers = self.maybe(Self::parse_type_qualifier_list);
        let right = self.maybe(Self::parse_pointer).map(Box::new);

        Ok(Pointer {
            at,
            asterisk,
            attributes,
            qualifiers,
            right,
        })
    }
    fn parse_type_qualifier_list(&mut self) -> Res<TypeQualifierList> {
        self.list(Self::parse_type_qualifier)
    }
    fn parse_parameter_type_list(&mut self) -> Res<ParameterTypeList<'a>> {
        let at = self.at();
        let parameters = self.maybe(Self::parse_parameter_list);
        let final_comma = if parameters.is_some() && self.is(TokenKind::Comma) {
            Some(self.next())
        } else {
            None
        };
        let ellipses = if parameters.is_none() || final_comma.is_some() {
            self.maybe(|p| p.take(TokenKind::Ellipses))
        } else {
            None
        };

        let parameters = parameters.map(|p| (p, final_comma));

        Ok(ParameterTypeList {
            at,
            parameters,
            ellipses,
        })
    }
    fn parse_parameter_list(&mut self) -> Res<ParameterList<'a>> {
        self.comma_list(Self::parse_parameter_declaration)
    }
    fn parse_parameter_declaration(&mut self) -> Res<ParameterDeclaration<'a>> {
        let at = self.at();
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        let specifiers = self.parse_declaration_specifiers(&mut false)?;
        let kind = self.one_of(
            [
                &mut |p| {
                    Ok(ParameterDeclarationKind::Concrete(
                        p.parse_declarator(false)?,
                    ))
                },
                &mut |p| {
                    Ok(ParameterDeclarationKind::Abstract(
                        p.maybe(Self::parse_abstract_declarator),
                    ))
                },
            ],
            Expected::ParameterDeclarationDeclarator,
        )?;

        Ok(ParameterDeclaration {
            at,
            attributes,
            specifiers,
            kind,
        })
    }
    fn parse_type_name(&mut self) -> Res<TypeName<'a>> {
        let at = self.at();
        let specifier_qualifiers = self.parse_specifier_qualifier_list()?;
        let declarator = self.maybe(Self::parse_abstract_declarator);

        Ok(TypeName {
            at,
            specifier_qualifiers,
            declarator,
        })
    }
    fn parse_abstract_declarator(&mut self) -> Res<AbstractDeclarator<'a>> {
        let at = self.at();
        let pointer = self.maybe(Self::parse_pointer);
        if pointer.is_none() {
            let direct = self.parse_direct_abstract_declarator()?;
            Ok(AbstractDeclarator {
                at,
                pointer,
                direct: Some(direct),
            })
        } else {
            let direct = self.maybe(Self::parse_direct_abstract_declarator);
            Ok(AbstractDeclarator {
                at,
                pointer,
                direct,
            })
        }
    }
    fn parse_direct_abstract_declarator(&mut self) -> Res<DirectAbstractDeclarator<'a>> {
        let mut left = self.maybe(|p| {
            let at = p.at();
            let open_parenthesis = p.take(TokenKind::OpenParenthesis)?;
            let inner = Box::new(p.parse_abstract_declarator()?);
            let close_parenthesis = p.take(TokenKind::CloseParenthesis)?;
            Ok(DirectAbstractDeclarator {
                at,
                kind: DirectAbstractDeclaratorKind::Parenthesized {
                    open_parenthesis,
                    inner,
                    close_parenthesis,
                },
            })
        });

        loop {
            match self.kind() {
                TokenKind::OpenBracket => left = Some(self.parse_array_abstract_declarator(left)?),
                TokenKind::OpenParenthesis => {
                    left = Some(self.parse_function_abstract_declarator(left)?)
                }
                _ => break,
            }
        }

        let Some(left) = left else {
            self.err(Expected::DirectAbstractDeclarator);
            return Err(());
        };

        Ok(left)
    }
    fn parse_array_abstract_declarator(
        &mut self,
        left: Option<DirectAbstractDeclarator<'a>>,
    ) -> Res<DirectAbstractDeclarator<'a>> {
        let left = left.map(Box::new);
        let at = self.at();
        let open_bracket = self.take(TokenKind::OpenBracket)?;
        let kind = if self.is(TokenKind::Static) {
            let static_keyword = Some(self.next());
            let qualifiers = self.maybe(Self::parse_type_qualifier_list);
            let size = Some(Box::new(self.parse_assignment_expression()?));
            ArrayAbstractDeclaratorKind::Normal {
                qualifiers,
                static_keyword,
                size,
            }
        } else if self.is(TokenKind::Asterisk) {
            ArrayAbstractDeclaratorKind::Var {
                asterisk: self.next(),
            }
        } else {
            let qualifiers = self.maybe(Self::parse_type_qualifier_list);
            if qualifiers.is_some() && self.is(TokenKind::Static) {
                let static_keyword = Some(self.take(TokenKind::Static)?);
                let size = Some(Box::new(self.parse_assignment_expression()?));
                ArrayAbstractDeclaratorKind::Normal {
                    qualifiers,
                    static_keyword,
                    size,
                }
            } else {
                let size = self.maybe(Self::parse_assignment_expression).map(Box::new);
                ArrayAbstractDeclaratorKind::Normal {
                    qualifiers,
                    static_keyword: None,
                    size,
                }
            }
        };

        let close_bracket = self.take(TokenKind::CloseBracket)?;

        let array = ArrayAbstractDeclarator {
            at,
            left,
            open_bracket,
            kind,
            close_bracket,
        };

        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);

        Ok(DirectAbstractDeclarator {
            at,
            kind: DirectAbstractDeclaratorKind::Array(array, attributes),
        })
    }
    fn parse_function_abstract_declarator(
        &mut self,
        left: Option<DirectAbstractDeclarator<'a>>,
    ) -> Res<DirectAbstractDeclarator<'a>> {
        let left = left.map(Box::new);
        let at = self.at();
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let parameters = self.maybe(Self::parse_parameter_type_list);
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

        let function = FunctionAbstractDeclarator {
            at,
            left,
            open_parenthesis,
            parameters,
            close_parenthesis,
        };
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);

        Ok(DirectAbstractDeclarator {
            at,
            kind: DirectAbstractDeclaratorKind::Function(function, attributes),
        })
    }
    fn parse_braced_initializer(&mut self) -> Res<BracedInitializer<'a>> {
        let at = self.at();
        let open_brace = self.take(TokenKind::OpenBrace)?;
        let initializers = if self.is(TokenKind::CloseBrace) {
            None
        } else {
            let initializer = self.parse_initializer_list()?;
            let final_comma = self.maybe(|p| p.take(TokenKind::Comma));
            Some((initializer, final_comma))
        };
        let close_brace = self.take(TokenKind::CloseBrace)?;

        Ok(BracedInitializer {
            at,
            open_brace,
            initializers,
            close_brace,
        })
    }
    fn parse_initializer(&mut self) -> Res<Initializer<'a>> {
        let at = self.at();
        let kind = self.one_of(
            [
                &mut |p| {
                    Ok(InitializerKind::Expression(
                        p.parse_assignment_expression()?,
                    ))
                },
                &mut |p| {
                    Ok(InitializerKind::Braced(Box::new(
                        p.parse_braced_initializer()?,
                    )))
                },
            ],
            Expected::Initializer,
        )?;

        Ok(Initializer { at, kind })
    }
    fn parse_initializer_list(&mut self) -> Res<InitializerList<'a>> {
        self.comma_list(|p| {
            let designation = p.maybe(Self::parse_designation);
            let initializer = p.parse_initializer()?;
            Ok((designation, initializer))
        })
    }
    fn parse_designation(&mut self) -> Res<Designation<'a>> {
        let at = self.at();
        let designators = self.parse_designator_list()?;
        let equal = self.take(TokenKind::Equal)?;

        Ok(Designation {
            at,
            designators,
            equal,
        })
    }
    fn parse_designator_list(&mut self) -> Res<DesignatorList<'a>> {
        self.list(Self::parse_designator)
    }
    fn parse_designator(&mut self) -> Res<Designator<'a>> {
        let at = self.at();
        let kind = if self.is(TokenKind::OpenBracket) {
            let open_bracket = self.next();
            let value = self.parse_constant_expression()?;
            let close_bracket = self.take(TokenKind::CloseBracket)?;
            DesignatorKind::InBrackets {
                open_bracket,
                value,
                close_bracket,
            }
        } else {
            let period = self.take(TokenKind::Period)?;
            let name = self.take_identifier()?;
            DesignatorKind::AfterPeriod { period, name }
        };

        Ok(Designator { at, kind })
    }
    fn parse_static_assert_declaration(&mut self) -> Res<StaticAssertDeclaration<'a>> {
        let at = self.at();
        let static_assert_keyword = self.take(TokenKind::StaticAssert)?;
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let condition = self.parse_constant_expression()?;
        let message = if self.is(TokenKind::Comma) {
            let comma = self.next();
            let string_at = self.at();
            let TokenKind::String(literal, encoding) = self.kind() else {
                self.err(Expected::StringLiteral);
                return Err(());
            };
            let string_literal = StringLiteral {
                at: string_at,
                literal,
                encoding,
            };
            Some((comma, string_literal))
        } else {
            None
        };
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
        let semicolon = self.take(TokenKind::Semicolon)?;

        Ok(StaticAssertDeclaration {
            at,
            static_assert_keyword,
            open_parenthesis,
            condition,
            message,
            close_parenthesis,
            semicolon,
        })
    }
    fn parse_attribute_specifier_sequence(&mut self) -> Res<AttributeSpecifierSequence<'a>> {
        let left = self.parse_attribute_specifier()?;
        let mut left = AttributeSpecifierSequence {
            at: left.at,
            left: None,
            specifier: left,
        };

        while let Ok(specifier) = self.try_to(Self::parse_attribute_specifier) {
            left = AttributeSpecifierSequence {
                at: left.at,
                left: Some(Box::new(left)),
                specifier,
            };
        }

        Ok(left)
    }
    fn parse_attribute_specifier(&mut self) -> Res<AttributeSpecifier<'a>> {
        let _at = self.at();
        let _open_bracket_0 = self.take(TokenKind::OpenBracket)?;
        let _open_bracket_1 = self.take(TokenKind::OpenBracket)?;
        todo!();
    }

    fn parse_statement(&mut self) -> Res<Statement<'a>> {
        let at = self.at();
        let kind = self.one_of(
            [
                &mut |p| Ok(StatementKind::Labeled(p.parse_labeled_statement()?)),
                &mut |p| Ok(StatementKind::Unlabeled(p.parse_unlabeled_statement()?)),
            ],
            Expected::Statement,
        )?;

        Ok(Statement { at, kind })
    }
    fn parse_unlabeled_statement(&mut self) -> Res<UnlabeledStatement<'a>> {
        let at = self.at();
        if let Ok(expr_statement) = self.try_to(Self::parse_expression_statement) {
            Ok(UnlabeledStatement {
                at,
                kind: UnlabeledStatementKind::Expression(expr_statement),
            })
        } else {
            let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
            let kind = if let Ok(primary) = self.try_to(Self::parse_primary_block) {
                UnlabeledStatementKind::Primary(attributes, primary)
            } else if let Ok(jump) = self.try_to(Self::parse_jump_statement) {
                UnlabeledStatementKind::Jump(attributes, jump)
            } else {
                self.err(Expected::UnlabeledStatement);
                return Err(());
            };
            Ok(UnlabeledStatement { at, kind })
        }
    }
    fn parse_primary_block(&mut self) -> Res<PrimaryBlock<'a>> {
        let at = self.at();
        let kind = self.one_of(
            [
                &mut |p| Ok(PrimaryBlockKind::Compound(p.parse_compound_statement()?)),
                &mut |p| Ok(PrimaryBlockKind::Selection(p.parse_selection_statement()?)),
                &mut |p| Ok(PrimaryBlockKind::Iteration(p.parse_iteration_statement()?)),
            ],
            Expected::PrimaryBlock,
        )?;

        Ok(PrimaryBlock { at, kind })
    }
    fn parse_secondary_block(&mut self) -> Res<SecondaryBlock<'a>> {
        let at = self.at();
        let statement = self.parse_statement()?;
        Ok(SecondaryBlock {
            at,
            statement: Box::new(statement),
        })
    }
    fn parse_label(&mut self) -> Res<Label<'a>> {
        let at = self.at();
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        let kind = if self.is(TokenKind::Default) {
            let default_keyword = self.next();
            LabelKind::Default { default_keyword }
        } else if self.is(TokenKind::Case) {
            let case_keyword = self.next();
            let value = self.parse_constant_expression()?;
            LabelKind::Case {
                case_keyword,
                value,
            }
        } else {
            let name = self.take_identifier()?;
            LabelKind::Name(name)
        };
        let colon = self.take(TokenKind::Colon)?;

        Ok(Label {
            at,
            attributes,
            kind,
            colon,
        })
    }
    fn parse_labeled_statement(&mut self) -> Res<LabeledStatement<'a>> {
        let at = self.at();
        let label = self.parse_label()?;
        let statement = Box::new(self.parse_statement()?);
        Ok(LabeledStatement {
            at,
            label,
            statement,
        })
    }
    fn parse_compound_statement(&mut self) -> Res<CompoundStatement<'a>> {
        let at = self.at();
        let open_brace = self.take(TokenKind::OpenBrace)?;

        self.scopes.push(HashSet::new());
        let items = self.maybe(Self::parse_block_item_list);
        self.scopes.pop();

        let close_brace = self.take(TokenKind::CloseBrace)?;

        Ok(CompoundStatement {
            at,
            open_brace,
            items,
            close_brace,
        })
    }
    fn parse_block_item_list(&mut self) -> Res<BlockItemList<'a>> {
        self.list(Self::parse_block_item)
    }
    fn parse_block_item(&mut self) -> Res<BlockItem<'a>> {
        let at = self.at();
        let kind = self.one_of(
            [
                &mut |p| Ok(BlockItemKind::Declaration(p.parse_declaration()?)),
                &mut |p| Ok(BlockItemKind::Unlabeled(p.parse_unlabeled_statement()?)),
                &mut |p| Ok(BlockItemKind::Label(p.parse_label()?)),
            ],
            Expected::BlockItem,
        )?;

        Ok(BlockItem { at, kind })
    }
    fn parse_expression_statement(&mut self) -> Res<ExpressionStatement<'a>> {
        let at = self.at();
        if let Ok(attributes) = self.try_to(Self::parse_attribute_specifier_sequence) {
            let expression = self.parse_expression()?;
            let semicolon = self.take(TokenKind::Semicolon)?;
            Ok(ExpressionStatement {
                at,
                attributes: Some(attributes),
                expression: Some(expression),
                semicolon,
            })
        } else {
            let expression = self.maybe(Self::parse_expression);
            let semicolon = self.take(TokenKind::Semicolon)?;
            Ok(ExpressionStatement {
                at,
                attributes: None,
                expression,
                semicolon,
            })
        }
    }
    fn parse_selection_statement(&mut self) -> Res<SelectionStatement<'a>> {
        let at = self.at();
        let kind = if self.is(TokenKind::If) {
            let if_keyword = self.next();
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let condition = self.parse_expression()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            let then_body = self.parse_secondary_block()?;

            let else_body = if self.is(TokenKind::Else) {
                let else_keyword = self.next();
                let else_body = self.parse_secondary_block()?;
                Some((else_keyword, else_body))
            } else {
                None
            };
            SelectionStatementKind::If {
                if_keyword,
                open_parenthesis,
                condition,
                close_parenthesis,
                then_body,
                else_body,
            }
        } else if self.is(TokenKind::Switch) {
            let switch_keyword = self.next();
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let controlling_expression = self.parse_expression()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            let body = self.parse_secondary_block()?;
            SelectionStatementKind::Switch {
                switch_keyword,
                open_parenthesis,
                controlling_expression,
                close_parenthesis,
                body,
            }
        } else {
            self.err(Expected::SelectionStatement);
            return Err(());
        };

        Ok(SelectionStatement { at, kind })
    }
    fn parse_iteration_statement(&mut self) -> Res<IterationStatement<'a>> {
        let at = self.at();
        let kind = if self.is(TokenKind::While) {
            let while_keyword = self.next();
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let condition = self.parse_expression()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            let body = self.parse_secondary_block()?;
            IterationStatementKind::While {
                while_keyword,
                open_parenthesis,
                condition,
                close_parenthesis,
                body,
            }
        } else if self.is(TokenKind::Do) {
            let do_keyword = self.next();
            let body = self.parse_secondary_block()?;
            let while_keyword = self.take(TokenKind::While)?;
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let condition = self.parse_expression()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            let semicolon = self.take(TokenKind::Semicolon)?;
            IterationStatementKind::DoWhile {
                do_keyword,
                body,
                while_keyword,
                open_parenthesis,
                condition,
                close_parenthesis,
                semicolon,
            }
        } else if self.is(TokenKind::For) {
            self.scopes.push(HashSet::new());
            let for_keyword = self.next();
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let initializer = self.parse_for_initializer()?;
            let condition = self.maybe(Self::parse_expression);
            let semicolon = self.take(TokenKind::Semicolon)?;
            let counter = self.maybe(Self::parse_expression);
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            let body = self.parse_secondary_block()?;
            self.scopes.pop();
            IterationStatementKind::For {
                for_keyword,
                open_parenthesis,
                initializer,
                condition,
                semicolon,
                counter,
                close_parenthesis,
                body,
            }
        } else {
            self.err(Expected::IterationStatement);
            return Err(());
        };

        Ok(IterationStatement { at, kind })
    }
    fn parse_for_initializer(&mut self) -> Res<ForInitializer<'a>> {
        if let Ok(declaration) = self.try_to(Self::parse_declaration) {
            Ok(ForInitializer::Declaration(declaration))
        } else {
            let expression = self.maybe(Self::parse_expression);
            let semicolon = self.take(TokenKind::Semicolon)?;
            Ok(ForInitializer::Expression(expression, semicolon))
        }
    }
    fn parse_jump_statement(&mut self) -> Res<JumpStatement<'a>> {
        let at = self.at();
        let kind = if self.is(TokenKind::Goto) {
            let goto_keyword = self.next();
            let target = self.take_identifier()?;
            JumpStatementKind::Goto {
                goto_keyword,
                target,
            }
        } else if self.is(TokenKind::Continue) {
            let continue_keyword = self.next();
            JumpStatementKind::Continue { continue_keyword }
        } else if self.is(TokenKind::Break) {
            let break_keyword = self.next();
            JumpStatementKind::Break { break_keyword }
        } else if self.is(TokenKind::Return) {
            let return_keyword = self.next();
            let value = self.maybe(Self::parse_expression);
            JumpStatementKind::Return {
                return_keyword,
                value,
            }
        } else {
            self.err(Expected::JumpStatement);
            return Err(());
        };
        let semicolon = self.take(TokenKind::Semicolon)?;

        Ok(JumpStatement {
            at,
            kind,
            semicolon,
        })
    }

    fn parse_translation_unit(&mut self) -> Res<TranslationUnit<'a>> {
        self.scopes.push(HashSet::new());
        let out = self.list(Self::parse_external_declaration);
        self.scopes.pop();
        out
    }
    fn parse_external_declaration(&mut self) -> Res<ExternalDeclaration<'a>> {
        let at = self.at();
        let kind = self.one_of(
            [
                &mut |p| {
                    Ok(ExternalDeclarationKind::Function(
                        p.parse_function_definition()?,
                    ))
                },
                &mut |p| Ok(ExternalDeclarationKind::Declaration(p.parse_declaration()?)),
            ],
            Expected::ExternalDeclaration,
        )?;
        Ok(ExternalDeclaration { at, kind })
    }
    fn parse_function_definition(&mut self) -> Res<FunctionDefinition<'a>> {
        let at = self.at();
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        let specifiers = self.parse_declaration_specifiers(&mut false)?;
        let declarator = self.parse_declarator(false)?;
        let body = self.parse_compound_statement()?;

        Ok(FunctionDefinition {
            at,
            attributes,
            specifiers,
            declarator,
            body,
        })
    }

    fn is_typedef_name(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }

        false
    }

    fn parse_binary_expression(
        &mut self,
        parse: fn(&mut Self) -> Res<Expression<'a>>,
        operations: &[(TokenKind<'a>, BinaryOperator)],
    ) -> Res<Expression<'a>> {
        let mut left = parse(self)?;
        'outer: loop {
            for &(token, operator) in operations {
                if self.is(token) {
                    let at = left.at;
                    let operator_at = self.next();
                    let right = parse(self)?;
                    let new_left = Box::new(left);
                    let right = Box::new(right);
                    let kind = ExpressionKind::Binary {
                        left: new_left,
                        operator: (operator_at, operator),
                        right,
                    };
                    left = Expression { at, kind };
                    continue 'outer;
                }
            }
            break;
        }

        Ok(left)
    }

    fn one_of<T, const N: usize>(
        &mut self,
        options: [&mut dyn FnMut(&mut Self) -> Res<T>; N],
        expected: Expected<'a>,
    ) -> Res<T> {
        for option in options {
            if let Ok(t) = self.try_to(|p| option(p)) {
                return Ok(t);
            }
        }

        self.err(expected);
        Err(())
    }
    fn list<T>(&mut self, mut parse: impl FnMut(&mut Self) -> Res<T>) -> Res<List<T>> {
        let at = self.at();
        let left = parse(self)?;
        let mut left = List {
            at,
            kind: ListKind::Leaf(Box::new(left)),
        };

        loop {
            if let Ok(right) = self.try_to(&mut parse) {
                left = List {
                    at: left.at,
                    kind: ListKind::Cons(Box::new(left), Box::new(right)),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }
    fn comma_list<T>(&mut self, mut parse: impl FnMut(&mut Self) -> Res<T>) -> Res<CommaList<T>> {
        let at = self.at();
        let left = parse(self)?;
        let mut left = CommaList {
            at,
            kind: CommaListKind::Leaf(Box::new(left)),
        };

        loop {
            if !self.is(TokenKind::Comma) {
                break;
            };
            let comma = self.next();
            let right = parse(self)?;
            left = CommaList {
                at: left.at,
                kind: CommaListKind::Cons {
                    left: Box::new(left),
                    comma,
                    right: Box::new(right),
                },
            };
        }

        Ok(left)
    }

    fn maybe<T>(&mut self, parse: impl FnMut(&mut Self) -> Res<T>) -> Option<T> {
        if let Ok(t) = self.try_to(parse) {
            Some(t)
        } else {
            None
        }
    }
    fn try_to<T>(&mut self, mut parse: impl FnMut(&mut Self) -> Res<T>) -> Res<T> {
        let index = self.index;
        let err_length = self.errors.len();
        let scopes_length = self.scopes.len();
        self.scopes.push(HashSet::new());

        match parse(self) {
            Ok(t) => {
                let top = self.scopes.pop().unwrap();
                debug_assert_eq!(scopes_length, self.scopes.len());
                self.scopes.last_mut().unwrap().extend(top);
                Ok(t)
            }
            Err(()) => {
                self.scopes.drain(scopes_length..);
                self.errors.drain(err_length..);
                self.index = index;
                Err(())
            }
        }
    }

    fn take_identifier(&mut self) -> Res<&'a str> {
        let TokenKind::Identifier(name) = self.kind() else {
            self.err(Expected::Identifier);
            return Err(());
        };
        self.next();
        Ok(name)
    }
    fn take(&mut self, kind: TokenKind<'a>) -> Res<At> {
        if !self.is(kind) {
            self.err(Expected::Token(kind));
            return Err(());
        }
        Ok(self.next())
    }
    fn next(&mut self) -> At {
        let at = self.at();
        self.index += 1;
        at
    }
    fn is(&self, kind: TokenKind) -> bool {
        self.kind() == kind
    }
    fn kind(&self) -> TokenKind<'a> {
        self.cur().kind
    }
    fn at(&self) -> At {
        self.cur().at
    }
    fn cur(&self) -> Token<'a> {
        self.tokens[self.index]
    }

    fn err(&mut self, expected: Expected<'a>) {
        let at = self.cur();
        self.err_at(at, expected);
    }
    fn err_at(&mut self, at: Token<'a>, expected: Expected<'a>) {
        self.errors.push(ParseErr { at, expected });
    }
}

type Res<T> = Result<T, ()>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ParseErr<'a> {
    pub at: Token<'a>,
    pub expected: Expected<'a>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Expected<'a> {
    Token(TokenKind<'a>),
    PrimaryExpression,
    Identifier,
    AssignmentOperator,
    DeclarationSpecifier,
    StorageClassSpecifier,
    TypeSpecifier,
    StructOrUnion,
    TypeSpecifierQualifier,
    TypeofSpecifierArgument,
    TypeQualifier,
    AlignasArgument,
    ParameterDeclarationDeclarator,
    DirectAbstractDeclarator,
    Initializer,
    StringLiteral,
    Statement,
    UnlabeledStatement,
    PrimaryBlock,
    BlockItem,
    SelectionStatement,
    IterationStatement,
    JumpStatement,
    ExternalDeclaration,
}
