use std::collections::HashSet;

use crate::{
    ast::*,
    token::{At, Token, TokenKind},
};

pub struct Parser<'a, 'b> {
    tokens: &'b [Token<'a>],
    index: usize,

    scopes: Vec<Scope<'a>>,
}
impl<'a, 'b> Parser<'a, 'b> {
    pub fn new(tokens: &'b [Token<'a>]) -> Self {
        Self {
            tokens,
            index: 0,

            scopes: Vec::new(),
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(Scope {
            typedefs: HashSet::new(),
        });
    }
    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn parse(mut self) -> Res<'a, Node<TranslationUnit<'a>>> {
        self.parse_translation_unit()
    }

    fn parse_primary_expression(&mut self) -> Res<'a, Node<PrimaryExpression<'a>>> {
        let at = self.cur().at;
        let node = match self.cur().kind {
            TokenKind::Identifier(name) => {
                self.next();
                PrimaryExpression::Identifier(name)
            }
            TokenKind::Integer(int) => {
                self.next();
                PrimaryExpression::Integer(int)
            }
            TokenKind::String(literal, encoding) => {
                self.next();
                PrimaryExpression::StringLiteral(StringLiteral(literal, encoding))
            }
            TokenKind::OpenParenthesis => {
                let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
                let inner = self.parse_expression()?;
                let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
                PrimaryExpression::Parenthesized {
                    open_parenthesis,
                    inner,
                    close_parenthesis,
                }
            }
            TokenKind::Generic => PrimaryExpression::Generic(self.parse_generic_selection()?),
            _ => {
                self.err_expected(Expected::PrimaryExpression)?;
                unreachable!()
            }
        };

        Ok(Node::new(at, node))
    }

    fn parse_generic_selection(&mut self) -> Res<'a, Node<GenericSelection<'a>>> {
        let generic_keyword = self.take(TokenKind::Generic)?;
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let controlling_expression = self.parse_assignment_expression()?;
        let comma = self.take(TokenKind::Comma)?;
        let association_list = self.parse_generic_assoc_list()?;
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

        Ok(Node::new(
            generic_keyword,
            GenericSelection {
                generic_keyword,
                open_parenthesis,
                controlling_expression,
                comma,
                association_list,
                close_parenthesis,
            },
        ))
    }
    fn parse_generic_assoc_list(&mut self) -> Res<'a, Node<GenericAssocList<'a>>> {
        self.parse_comma_list(
            Self::parse_generic_association,
            GenericAssocList::Leaf,
            |left, comma, right| GenericAssocList::Rec { left, comma, right },
        )
    }
    fn parse_generic_association(&mut self) -> Res<'a, Node<GenericAssociation<'a>>> {
        match self.cur().kind {
            TokenKind::Default => {
                let default_keyword = self.take(TokenKind::Default)?;
                let colon = self.take(TokenKind::Colon)?;
                let value = self.parse_assignment_expression()?;

                Ok(Node::new(
                    default_keyword,
                    GenericAssociation::Default {
                        default_keyword,
                        colon,
                        value,
                    },
                ))
            }
            _ => {
                let type_name = self.parse_type_name()?;
                let colon = self.take(TokenKind::Colon)?;
                let value = self.parse_assignment_expression()?;

                Ok(Node::new(
                    type_name.at(),
                    GenericAssociation::ForType {
                        type_name,
                        colon,
                        value,
                    },
                ))
            }
        }
    }

    fn parse_postfix_expression(&mut self) -> Res<'a, Node<PostfixExpression<'a>>> {
        let mut left = if let Ok(primary) = self.try_to(Self::parse_primary_expression) {
            Node::new(primary.at(), PostfixExpression::Primary(primary))
        } else if let Ok(compound) = self.try_to(Self::parse_compound_literal) {
            Node::new(compound.at(), PostfixExpression::CompoundLiteral(compound))
        } else {
            self.err_expected(Expected::PostfixExpressionLeaf)?;
            unreachable!()
        };

        loop {
            match self.cur().kind {
                TokenKind::OpenBracket => {
                    let open_bracket = self.take(TokenKind::OpenBracket)?;
                    let index = self.parse_expression()?;
                    let close_bracket = self.take(TokenKind::CloseBracket)?;
                    left = Node::new(
                        open_bracket,
                        PostfixExpression::Index {
                            left,
                            open_bracket,
                            index,
                            close_bracket,
                        },
                    );
                }
                TokenKind::OpenParenthesis => {
                    let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
                    let arguments = self.maybe(Self::parse_argument_expression_list);
                    let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
                    left = Node::new(
                        open_parenthesis,
                        PostfixExpression::Call {
                            left,
                            open_parenthesis,
                            arguments,
                            close_parenthesis,
                        },
                    );
                }
                TokenKind::Period => {
                    let period = self.take(TokenKind::Period)?;
                    let name = self.take_identifier()?;
                    left = Node::new(period, PostfixExpression::Member { left, period, name });
                }
                TokenKind::ArrowLeft => {
                    let arrow = self.take(TokenKind::ArrowLeft)?;
                    let name = self.take_identifier()?;
                    left = Node::new(
                        arrow,
                        PostfixExpression::MemberIndirect { left, arrow, name },
                    );
                }
                TokenKind::DoublePlus => {
                    let plus_plus = self.take(TokenKind::DoublePlus)?;
                    left = Node::new(
                        plus_plus,
                        PostfixExpression::PostIncrement { left, plus_plus },
                    );
                }
                TokenKind::DoubleMinus => {
                    let minus_minus = self.take(TokenKind::DoubleMinus)?;
                    left = Node::new(
                        minus_minus,
                        PostfixExpression::PostDecrement { left, minus_minus },
                    );
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_argument_expression_list(&mut self) -> Res<'a, Node<ArgumentExpressionList<'a>>> {
        self.parse_comma_list(
            Self::parse_assignment_expression,
            ArgumentExpressionList::Leaf,
            |left, comma, right| ArgumentExpressionList::Rec { left, comma, right },
        )
    }
    fn parse_compound_literal(&mut self) -> Res<'a, Node<CompoundLiteral<'a>>> {
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let storage_class = self.maybe(|p| Self::parse_storage_class_specifiers(p, &mut false));
        let type_name = self.parse_type_name()?;
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
        let initializer = self.parse_braced_initializer()?;
        Ok(Node::new(
            open_parenthesis,
            CompoundLiteral {
                open_parenthesis,
                storage_class,
                type_name,
                close_parenthesis,
                initializer,
            },
        ))
    }
    fn parse_storage_class_specifiers(
        &mut self,
        is_typedef: &mut bool,
    ) -> Res<'a, Node<StorageClassSpecifiers>> {
        self.parse_list(
            |p| Self::parse_storage_class_specifier(p, is_typedef),
            StorageClassSpecifiers::Leaf,
            StorageClassSpecifiers::Rec,
        )
    }

    fn parse_unary_expression(&mut self) -> Res<'a, Node<UnaryExpression<'a>>> {
        let at = self.cur().at;
        let node = match self.cur().kind {
            TokenKind::DoublePlus => {
                let plus_plus = self.take(TokenKind::DoublePlus)?;
                let right = self.parse_unary_expression()?;
                UnaryExpression::PreIncrement { plus_plus, right }
            }
            TokenKind::DoubleMinus => {
                let minus_minus = self.take(TokenKind::DoubleMinus)?;
                let right = self.parse_unary_expression()?;
                UnaryExpression::PreDecrement { minus_minus, right }
            }
            TokenKind::Ampersand
            | TokenKind::Asterisk
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Tilde
            | TokenKind::Exclamation => {
                let operator = match self.cur().kind {
                    TokenKind::Ampersand => UnaryOperator::AddressOf,
                    TokenKind::Asterisk => UnaryOperator::Dereference,
                    TokenKind::Plus => UnaryOperator::Positive,
                    TokenKind::Minus => UnaryOperator::Negative,
                    TokenKind::Tilde => UnaryOperator::BitNot,
                    TokenKind::Exclamation => UnaryOperator::LogicalNot,
                    _ => unreachable!(),
                };
                self.next();

                let operator = Node::new(at, operator);
                let right = self.parse_cast_expression()?;
                UnaryExpression::Operator(operator, right)
            }
            TokenKind::Sizeof => self.parse_sizeof_unary_expression()?,
            TokenKind::Alignof => {
                let alignof_keyword = self.take(TokenKind::Alignof)?;
                let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
                let type_name = self.parse_type_name()?;
                let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
                UnaryExpression::Alignof {
                    alignof_keyword,
                    open_parenthesis,
                    type_name,
                    close_parenthesis,
                }
            }
            _ => UnaryExpression::Postfix(self.parse_postfix_expression()?),
        };

        Ok(Node::new(at, node))
    }
    fn parse_sizeof_unary_expression(&mut self) -> Res<'a, UnaryExpression<'a>> {
        let sizeof_keyword = self.take(TokenKind::Sizeof)?;

        if let Ok(right) = self.try_to(Self::parse_unary_expression) {
            Ok(UnaryExpression::SizeofValue {
                sizeof_keyword,
                right,
            })
        } else {
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let type_name = self.parse_type_name()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

            Ok(UnaryExpression::SizeofType {
                sizeof_keyword,
                open_parenthesis,
                type_name,
                close_parenthesis,
            })
        }
    }

    fn parse_cast_expression(&mut self) -> Res<'a, Node<CastExpression<'a>>> {
        if let Ok(e) = self.try_to(Self::parse_actual_cast_expression) {
            Ok(e)
        } else {
            let e = self.parse_unary_expression()?;
            Ok(Node::new(e.at(), CastExpression::Unary(e)))
        }
    }
    fn parse_actual_cast_expression(&mut self) -> Res<'a, Node<CastExpression<'a>>> {
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let type_name = self.parse_type_name()?;
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
        let right = self.parse_cast_expression()?;

        Ok(Node::new(
            open_parenthesis,
            CastExpression::Cast {
                open_parenthesis,
                type_name,
                close_parenthesis,
                right,
            },
        ))
    }

    fn parse_multiplicative_expression(&mut self) -> Res<'a, Node<MultiplicativeExpression<'a>>> {
        let left = self.parse_cast_expression()?;
        let mut left = Node::new(left.at(), MultiplicativeExpression::Cast(left));

        loop {
            let operator = match self.cur().kind {
                TokenKind::Asterisk => |left, asterisk, right| MultiplicativeExpression::Multiply {
                    left,
                    asterisk,
                    right,
                },
                TokenKind::Slash => {
                    |left, slash, right| MultiplicativeExpression::Divide { left, slash, right }
                }
                TokenKind::Percent => |left, percent, right| MultiplicativeExpression::Modulo {
                    left,
                    percent,
                    right,
                },
                _ => break,
            };
            let op_at = self.cur().at;
            self.next();
            let right = self.parse_cast_expression()?;
            left = Node::new(left.at(), operator(left, op_at, right));
        }

        Ok(left)
    }
    fn parse_additive_expression(&mut self) -> Res<'a, Node<AdditiveExpression<'a>>> {
        let left = self.parse_multiplicative_expression()?;
        let mut left = Node::new(left.at(), AdditiveExpression::Multiplicative(left));

        loop {
            let operator = match self.cur().kind {
                TokenKind::Plus => {
                    |left, plus, right| AdditiveExpression::Add { left, plus, right }
                }
                TokenKind::Minus => {
                    |left, minus, right| AdditiveExpression::Subtract { left, minus, right }
                }
                _ => break,
            };
            let op_at = self.cur().at;
            self.next();
            let right = self.parse_multiplicative_expression()?;
            left = Node::new(left.at(), operator(left, op_at, right));
        }

        Ok(left)
    }
    fn parse_shift_expression(&mut self) -> Res<'a, Node<ShiftExpression<'a>>> {
        let left = self.parse_additive_expression()?;
        let mut left = Node::new(left.at(), ShiftExpression::Additive(left));

        loop {
            let operator = match self.cur().kind {
                TokenKind::DoubleLess => |left, double_less, right| ShiftExpression::Left {
                    left,
                    double_less,
                    right,
                },
                TokenKind::DoubleGreater => |left, double_greater, right| ShiftExpression::Right {
                    left,
                    double_greater,
                    right,
                },
                _ => break,
            };
            let op_at = self.cur().at;
            self.next();
            let right = self.parse_additive_expression()?;
            left = Node::new(left.at(), operator(left, op_at, right));
        }

        Ok(left)
    }
    fn parse_relational_expression(&mut self) -> Res<'a, Node<RelationalExpression<'a>>> {
        let left = self.parse_shift_expression()?;
        let mut left = Node::new(left.at(), RelationalExpression::Shift(left));

        loop {
            let operator = match self.cur().kind {
                TokenKind::Less => {
                    |left, less, right| RelationalExpression::Less { left, less, right }
                }
                TokenKind::Greater => |left, greater, right| RelationalExpression::Greater {
                    left,
                    greater,
                    right,
                },
                TokenKind::LessEqual => |left, less_equal, right| RelationalExpression::LessEqual {
                    left,
                    less_equal,
                    right,
                },
                TokenKind::GreaterEqual => {
                    |left, greater_equal, right| RelationalExpression::GreaterEqual {
                        left,
                        greater_equal,
                        right,
                    }
                }
                _ => break,
            };
            let op_at = self.cur().at;
            self.next();
            let right = self.parse_shift_expression()?;
            left = Node::new(left.at(), operator(left, op_at, right));
        }

        Ok(left)
    }
    fn parse_equality_expression(&mut self) -> Res<'a, Node<EqualityExpression<'a>>> {
        let left = self.parse_relational_expression()?;
        let mut left = Node::new(left.at(), EqualityExpression::Relational(left));

        loop {
            let operator = match self.cur().kind {
                TokenKind::DoubleEqual => {
                    |left, equal, right| EqualityExpression::Equal { left, equal, right }
                }
                TokenKind::NotEqual => |left, not_equal, right| EqualityExpression::NotEqual {
                    left,
                    not_equal,
                    right,
                },
                _ => break,
            };
            let op_at = self.cur().at;
            self.next();
            let right = self.parse_relational_expression()?;
            left = Node::new(left.at(), operator(left, op_at, right));
        }

        Ok(left)
    }
    fn parse_and_expression(&mut self) -> Res<'a, Node<AndExpression<'a>>> {
        let left = self.parse_equality_expression()?;
        let mut left = Node::new(left.at(), AndExpression::Equality(left));

        loop {
            let operator = match self.cur().kind {
                TokenKind::Ampersand => |left, ampersand, right| AndExpression::And {
                    left,
                    ampersand,
                    right,
                },
                _ => break,
            };
            let op_at = self.cur().at;
            self.next();
            let right = self.parse_equality_expression()?;
            left = Node::new(left.at(), operator(left, op_at, right));
        }

        Ok(left)
    }
    fn parse_exclusive_or_expression(&mut self) -> Res<'a, Node<ExclusiveOrExpression<'a>>> {
        let left = self.parse_and_expression()?;
        let mut left = Node::new(left.at(), ExclusiveOrExpression::And(left));

        loop {
            let operator = match self.cur().kind {
                TokenKind::Caret => {
                    |left, caret, right| ExclusiveOrExpression::ExclusiveOr { left, caret, right }
                }
                _ => break,
            };
            let op_at = self.cur().at;
            self.next();
            let right = self.parse_and_expression()?;
            left = Node::new(left.at(), operator(left, op_at, right));
        }

        Ok(left)
    }
    fn parse_inclusive_or_expression(&mut self) -> Res<'a, Node<InclusiveOrExpression<'a>>> {
        let left = self.parse_exclusive_or_expression()?;
        let mut left = Node::new(left.at(), InclusiveOrExpression::ExclusiveOr(left));

        loop {
            let operator = match self.cur().kind {
                TokenKind::Bar => {
                    |left, bar, right| InclusiveOrExpression::InclusiveOr { left, bar, right }
                }
                _ => break,
            };
            let op_at = self.cur().at;
            self.next();
            let right = self.parse_exclusive_or_expression()?;
            left = Node::new(left.at(), operator(left, op_at, right));
        }

        Ok(left)
    }
    fn parse_logical_and_expression(&mut self) -> Res<'a, Node<LogicalAndExpression<'a>>> {
        let left = self.parse_inclusive_or_expression()?;
        let mut left = Node::new(left.at(), LogicalAndExpression::InclusiveOr(left));

        loop {
            let operator = match self.cur().kind {
                TokenKind::DoubleAmpersand => {
                    |left, double_ampersand, right| LogicalAndExpression::LogicalAnd {
                        left,
                        double_ampersand,
                        right,
                    }
                }
                _ => break,
            };
            let op_at = self.cur().at;
            self.next();
            let right = self.parse_inclusive_or_expression()?;
            left = Node::new(left.at(), operator(left, op_at, right));
        }

        Ok(left)
    }
    fn parse_logical_or_expression(&mut self) -> Res<'a, Node<LogicalOrExpression<'a>>> {
        let left = self.parse_logical_and_expression()?;
        let mut left = Node::new(left.at(), LogicalOrExpression::LogicalAnd(left));

        loop {
            let operator = match self.cur().kind {
                TokenKind::DoubleBar => |left, double_bar, right| LogicalOrExpression::LogicalOr {
                    left,
                    double_bar,
                    right,
                },
                _ => break,
            };
            let op_at = self.cur().at;
            self.next();
            let right = self.parse_logical_and_expression()?;
            left = Node::new(left.at(), operator(left, op_at, right));
        }

        Ok(left)
    }
    fn parse_conditional_expression(&mut self) -> Res<'a, Node<ConditionalExpression<'a>>> {
        let left = self.parse_logical_or_expression()?;

        if self.is(TokenKind::Question) {
            let condition = left;
            let question = self.take(TokenKind::Question)?;
            let then_value = self.parse_expression()?;
            let colon = self.take(TokenKind::Colon)?;
            let else_value = self.parse_conditional_expression()?;

            Ok(Node::new(
                condition.at(),
                ConditionalExpression::Conditional {
                    condition,
                    question,
                    then_value,
                    colon,
                    else_value,
                },
            ))
        } else {
            let left = Node::new(left.at(), ConditionalExpression::LogicalOr(left));
            Ok(left)
        }
    }

    fn parse_assignment_expression(&mut self) -> Res<'a, Node<AssignmentExpression<'a>>> {
        if let Ok(e) = self.try_to(Self::parse_actual_assignment_expression) {
            Ok(e)
        } else {
            let left = self.parse_conditional_expression()?;
            Ok(Node::new(
                left.at(),
                AssignmentExpression::Conditional(left),
            ))
        }
    }
    fn parse_actual_assignment_expression(&mut self) -> Res<'a, Node<AssignmentExpression<'a>>> {
        let left = self.parse_unary_expression()?;
        let operator = match self.cur().kind {
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
                self.err_expected(Expected::AssignmentOperator)?;
                unreachable!()
            }
        };
        let operator = Node::new(self.cur().at, operator);
        self.next();

        let right = self.parse_assignment_expression()?;

        Ok(Node::new(
            left.at(),
            AssignmentExpression::Assignment {
                left,
                operator,
                right,
            },
        ))
    }

    fn parse_expression(&mut self) -> Res<'a, Node<Expression<'a>>> {
        self.parse_comma_list(
            Self::parse_assignment_expression,
            Expression::Assign,
            |left, comma, right| Expression::Comma { left, comma, right },
        )
    }

    fn parse_constant_expression(&mut self) -> Res<'a, Node<ConstantExpression<'a>>> {
        let inner = self.parse_conditional_expression()?;
        Ok(Node::new(inner.at(), ConstantExpression(inner)))
    }

    fn parse_declaration(&mut self) -> Res<'a, Node<Declaration<'a>>> {
        if let Ok(assert) = self.try_to(Self::parse_static_assert_declaration) {
            Ok(Node::new(assert.at(), Declaration::Assert(assert)))
        } else if let Ok(attribute) = self.try_to(Self::parse_attribute_declaration) {
            Ok(Node::new(attribute.at(), Declaration::Attribute(attribute)))
        } else if let Ok(attributes) = self.try_to(Self::parse_attribute_specifier_sequence) {
            let mut is_typedef = false;
            let specifiers = self.parse_declaration_specifiers(&mut is_typedef)?;
            let declarators = self.parse_init_declarator_list(is_typedef)?;
            let semicolon = self.take(TokenKind::Semicolon)?;

            Ok(Node::new(
                attributes.at(),
                Declaration::WithAttributes {
                    attributes,
                    specifiers,
                    declarators,
                    semicolon,
                },
            ))
        } else {
            let mut is_typedef = false;
            let specifiers = self.parse_declaration_specifiers(&mut is_typedef)?;
            let declarators = self.maybe(|p| Self::parse_init_declarator_list(p, is_typedef));
            let semicolon = self.take(TokenKind::Semicolon)?;
            Ok(Node::new(
                specifiers.at(),
                Declaration::Normal {
                    specifiers,
                    declarators,
                    semicolon,
                },
            ))
        }
    }

    fn parse_declaration_specifiers(
        &mut self,
        is_typedef: &mut bool,
    ) -> Res<'a, Node<DeclarationSpecifiers<'a>>> {
        let specifier = self.parse_declaration_specifier(is_typedef)?;
        if let Ok(cons) = self.try_to(|p| Self::parse_declaration_specifiers(p, is_typedef)) {
            Ok(Node::new(
                specifier.at(),
                DeclarationSpecifiers::Rec(specifier, cons),
            ))
        } else {
            let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
            Ok(Node::new(
                specifier.at(),
                DeclarationSpecifiers::Leaf(specifier, attributes),
            ))
        }
    }
    fn parse_declaration_specifier(
        &mut self,
        is_typedef: &mut bool,
    ) -> Res<'a, Node<DeclarationSpecifier<'a>>> {
        if let Ok(storage_class) =
            self.try_to(|p| Self::parse_storage_class_specifier(p, is_typedef))
        {
            Ok(Node::new(
                storage_class.at(),
                DeclarationSpecifier::StorageClass(storage_class),
            ))
        } else if let Ok(type_spec) = self.try_to(Self::parse_type_specifier_qualifier) {
            Ok(Node::new(
                type_spec.at(),
                DeclarationSpecifier::TypeSpecifier(type_spec),
            ))
        } else if let Ok(func) = self.try_to(Self::parse_function_specifier) {
            Ok(Node::new(
                func.at(),
                DeclarationSpecifier::FunctionSpecifier(func),
            ))
        } else {
            self.err_expected(Expected::DeclarationSpecifier)?;
            unreachable!()
        }
    }
    fn parse_init_declarator_list(
        &mut self,
        is_typedef: bool,
    ) -> Res<'a, Node<InitDeclaratorList<'a>>> {
        self.parse_comma_list(
            |p| Self::parse_init_declarator(p, is_typedef),
            InitDeclaratorList::Leaf,
            |left, comma, right| InitDeclaratorList::Rec { left, comma, right },
        )
    }
    fn parse_init_declarator(&mut self, is_typedef: bool) -> Res<'a, Node<InitDeclarator<'a>>> {
        let declarator = self.parse_declarator(is_typedef)?;
        if self.is(TokenKind::Equal) {
            let equal = self.take(TokenKind::Equal)?;
            let initializer = self.parse_initializer()?;

            Ok(Node::new(
                declarator.at(),
                InitDeclarator::Initializer {
                    declarator,
                    equal,
                    initializer,
                },
            ))
        } else {
            Ok(Node::new(
                declarator.at(),
                InitDeclarator::NoInitializer(declarator),
            ))
        }
    }
    fn parse_attribute_declaration(&mut self) -> Res<'a, Node<AttributeDeclaration<'a>>> {
        let attributes = self.parse_attribute_specifier_sequence()?;
        let semicolon = self.take(TokenKind::Semicolon)?;
        Ok(Node::new(
            attributes.at(),
            AttributeDeclaration {
                attributes,
                semicolon,
            },
        ))
    }
    fn parse_storage_class_specifier(
        &mut self,
        is_typedef: &mut bool,
    ) -> Res<'a, Node<StorageClassSpecifier>> {
        let specifier = match self.cur().kind {
            TokenKind::Auto => StorageClassSpecifier::Auto,
            TokenKind::Constexpr => StorageClassSpecifier::Constexpr,
            TokenKind::Extern => StorageClassSpecifier::Extern,
            TokenKind::Register => StorageClassSpecifier::Register,
            TokenKind::Static => StorageClassSpecifier::Static,
            TokenKind::ThreadLocal => StorageClassSpecifier::ThreadLocal,
            TokenKind::Typedef => StorageClassSpecifier::Typedef,
            _ => {
                self.err_expected(Expected::StorageClassSpecifier)?;
                unreachable!()
            }
        };
        let at = self.cur().at;
        self.next();

        if specifier == StorageClassSpecifier::Typedef {
            *is_typedef = true;
        }

        Ok(Node::new(at, specifier))
    }
    fn parse_type_specifier(&mut self) -> Res<'a, Node<TypeSpecifier<'a>>> {
        let at = self.cur().at;
        let specifier = match self.cur().kind {
            TokenKind::Void => {
                self.next();
                TypeSpecifier::Void
            }
            TokenKind::Char => {
                self.next();
                TypeSpecifier::Char
            }
            TokenKind::Short => {
                self.next();
                TypeSpecifier::Short
            }
            TokenKind::Int => {
                self.next();
                TypeSpecifier::Int
            }
            TokenKind::Long => {
                self.next();
                TypeSpecifier::Long
            }
            TokenKind::Float => {
                self.next();
                TypeSpecifier::Float
            }
            TokenKind::Double => {
                self.next();
                TypeSpecifier::Double
            }
            TokenKind::Signed => {
                self.next();
                TypeSpecifier::Signed
            }
            TokenKind::Unsigned => {
                self.next();
                TypeSpecifier::Unsigned
            }
            TokenKind::BitInt => {
                let bitint_keyword = self.take(TokenKind::BitInt)?;
                let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
                let width = self.parse_constant_expression()?;
                let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

                TypeSpecifier::BitInt {
                    bitint_keyword,
                    open_parenthesis,
                    width,
                    close_parenthesis,
                }
            }
            TokenKind::Bool => {
                self.next();
                TypeSpecifier::Bool
            }
            TokenKind::Complex => {
                self.next();
                TypeSpecifier::Complex
            }
            TokenKind::Decimal32 => {
                self.next();
                TypeSpecifier::Decimal32
            }
            TokenKind::Decimal64 => {
                self.next();
                TypeSpecifier::Decimal64
            }
            TokenKind::Decimal128 => {
                self.next();
                TypeSpecifier::Decimal128
            }
            _ => {
                if let Ok(atomic) = self.try_to(Self::parse_atomic_type_specifier) {
                    TypeSpecifier::Atomic(atomic)
                } else if let Ok(struct_or_union) =
                    self.try_to(Self::parse_struct_or_union_specifier)
                {
                    TypeSpecifier::StructOrUnion(struct_or_union)
                } else if let Ok(enum_spec) = self.try_to(Self::parse_enum_specifier) {
                    TypeSpecifier::Enum(enum_spec)
                } else if let Ok(typedef) = self.try_to(Self::parse_typedef_name) {
                    TypeSpecifier::TypedefName(typedef)
                } else if let Ok(typeof_spec) = self.try_to(Self::parse_typeof_specifier) {
                    TypeSpecifier::Typeof(typeof_spec)
                } else {
                    self.err_expected(Expected::TypeSpecifier)?;
                    unreachable!()
                }
            }
        };

        Ok(Node::new(at, specifier))
    }
    fn parse_struct_or_union_specifier(&mut self) -> Res<'a, Node<StructOrUnionSpecifier<'a>>> {
        let struct_or_union = self.parse_struct_or_union()?;
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);

        let parse_with_list = |parser: &mut Self| {
            let tag = parser.maybe(Self::take_identifier);
            let open_brace = parser.take(TokenKind::OpenBrace)?;
            let members = parser.parse_member_declaration_list()?;
            let close_brace = parser.take(TokenKind::CloseBrace)?;

            Ok((tag, open_brace, members, close_brace))
        };

        if let Ok((tag, open_brace, members, close_brace)) = self.try_to(parse_with_list) {
            Ok(Node::new(
                struct_or_union.at(),
                StructOrUnionSpecifier::WithMembers {
                    struct_or_union,
                    attributes,
                    tag,
                    open_brace,
                    members,
                    close_brace,
                },
            ))
        } else {
            let tag = self.take_identifier()?;
            Ok(Node::new(
                struct_or_union.at(),
                StructOrUnionSpecifier::WithoutMembers(struct_or_union, attributes, tag),
            ))
        }
    }
    fn parse_struct_or_union(&mut self) -> Res<'a, Node<StructOrUnion>> {
        let at = self.cur().at;
        let kind = match self.cur().kind {
            TokenKind::Struct => StructOrUnion::Struct,
            TokenKind::Union => StructOrUnion::Union,
            _ => {
                self.err_expected(Expected::StructOrUnion)?;
                unreachable!()
            }
        };

        Ok(Node::new(at, kind))
    }
    fn parse_member_declaration_list(&mut self) -> Res<'a, Node<MemberDeclarationList<'a>>> {
        self.parse_list(
            Self::parse_member_declaration,
            MemberDeclarationList::Leaf,
            MemberDeclarationList::Rec,
        )
    }
    fn parse_member_declaration(&mut self) -> Res<'a, Node<MemberDeclaration<'a>>> {
        if let Ok(assert) = self.try_to(Self::parse_static_assert_declaration) {
            Ok(Node::new(assert.at(), MemberDeclaration::Assert(assert)))
        } else {
            let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
            let specifiers = self.parse_specifier_qualifier_list()?;
            let declarators = self.maybe(Self::parse_member_declarator_list);
            let semicolon = self.take(TokenKind::Semicolon)?;

            Ok(Node::new(
                attributes
                    .as_ref()
                    .map(|a| a.at())
                    .unwrap_or(specifiers.at()),
                MemberDeclaration::Member {
                    attributes,
                    specifiers,
                    declarators,
                    semicolon,
                },
            ))
        }
    }
    fn parse_specifier_qualifier_list(&mut self) -> Res<'a, Node<SpecifierQualifierList<'a>>> {
        let specifier = self.parse_type_specifier_qualifier()?;

        if let Ok(cons) = self.try_to(Self::parse_specifier_qualifier_list) {
            Ok(Node::new(
                specifier.at(),
                SpecifierQualifierList::Rec(specifier, cons),
            ))
        } else {
            let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
            Ok(Node::new(
                specifier.at(),
                SpecifierQualifierList::Leaf(specifier, attributes),
            ))
        }
    }
    fn parse_type_specifier_qualifier(&mut self) -> Res<'a, Node<TypeSpecifierQualifier<'a>>> {
        if let Ok(specifier) = self.try_to(Self::parse_type_specifier) {
            Ok(Node::new(
                specifier.at(),
                TypeSpecifierQualifier::TypeSpecifier(specifier),
            ))
        } else if let Ok(qualifier) = self.try_to(Self::parse_type_qualifier) {
            Ok(Node::new(
                qualifier.at(),
                TypeSpecifierQualifier::TypeQualifier(qualifier),
            ))
        } else if let Ok(alignment) = self.try_to(Self::parse_alignment_specifier) {
            Ok(Node::new(
                alignment.at(),
                TypeSpecifierQualifier::AlignmentSpecifier(alignment),
            ))
        } else {
            self.err_expected(Expected::TypeSpecifierQualifier)?;
            unreachable!()
        }
    }
    fn parse_member_declarator_list(&mut self) -> Res<'a, Node<MemberDeclaratorList<'a>>> {
        self.parse_comma_list(
            Self::parse_member_declarator,
            MemberDeclaratorList::Leaf,
            |left, comma, right| MemberDeclaratorList::Rec { left, comma, right },
        )
    }
    fn parse_member_declarator(&mut self) -> Res<'a, Node<MemberDeclarator<'a>>> {
        let at = self.cur().at;

        let parse_with_colon = |parser: &mut Self| {
            let declarator = parser.maybe(|p| Self::parse_declarator(p, false));
            let colon = parser.take(TokenKind::Colon)?;
            let value = parser.parse_constant_expression()?;
            Ok((declarator, colon, value))
        };

        if let Ok((declarator, colon, width)) = self.try_to(parse_with_colon) {
            Ok(Node::new(
                at,
                MemberDeclarator::WithWidth {
                    declarator,
                    colon,
                    width,
                },
            ))
        } else {
            let declarator = self.parse_declarator(false)?;
            Ok(Node::new(at, MemberDeclarator::WithoutWidth(declarator)))
        }
    }
    fn parse_enum_specifier(&mut self) -> Res<'a, Node<EnumSpecifier<'a>>> {
        let enum_keyword = self.take(TokenKind::Enum)?;

        let parse_with_members = |parser: &mut Self| {
            let attributes = parser.maybe(Self::parse_attribute_specifier_sequence);
            let tag = parser.maybe(Self::take_identifier);
            let enum_type = parser.maybe(Self::parse_enum_type_specifier);
            let open_brace = parser.take(TokenKind::OpenBrace)?;
            let members = parser.parse_enumerator_list()?;
            let comma = parser.maybe(|p| p.take(TokenKind::Comma));
            let close_brace = parser.take(TokenKind::CloseBrace)?;
            Ok((
                attributes,
                tag,
                enum_type,
                open_brace,
                members,
                comma,
                close_brace,
            ))
        };

        if let Ok((attributes, tag, enum_type, open_brace, enumerators, final_comma, close_brace)) =
            self.try_to(parse_with_members)
        {
            Ok(Node::new(
                enum_keyword,
                EnumSpecifier::WithList {
                    enum_keyword,
                    attributes,
                    tag,
                    enum_type,
                    open_brace,
                    enumerators,
                    final_comma,
                    close_brace,
                },
            ))
        } else {
            let tag = self.take_identifier()?;
            let enum_type = self.maybe(Self::parse_enum_type_specifier);
            Ok(Node::new(
                enum_keyword,
                EnumSpecifier::WithoutList {
                    enum_keyword,
                    tag,
                    enum_type,
                },
            ))
        }
    }
    fn parse_enumerator_list(&mut self) -> Res<'a, Node<EnumeratorList<'a>>> {
        self.parse_comma_list(
            Self::parse_enumerator,
            EnumeratorList::Leaf,
            |left, comma, right| EnumeratorList::Rec { left, comma, right },
        )
    }
    fn parse_enumerator(&mut self) -> Res<'a, Node<Enumerator<'a>>> {
        let at = self.cur().at;
        let name = self.take_identifier()?;
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);

        if self.is(TokenKind::Equal) {
            let equal = self.take(TokenKind::Equal)?;
            let value = self.parse_constant_expression()?;
            Ok(Node::new(
                at,
                Enumerator::WithValue {
                    name,
                    attributes,
                    equal,
                    value,
                },
            ))
        } else {
            Ok(Node::new(at, Enumerator::WithoutValue(name, attributes)))
        }
    }
    fn parse_enum_type_specifier(&mut self) -> Res<'a, Node<EnumTypeSpecifier<'a>>> {
        let colon = self.take(TokenKind::Colon)?;
        let specifiers = self.parse_specifier_qualifier_list()?;
        Ok(Node::new(colon, EnumTypeSpecifier { colon, specifiers }))
    }
    fn parse_atomic_type_specifier(&mut self) -> Res<'a, Node<AtomicTypeSpecifier<'a>>> {
        let atomic_keyword = self.take(TokenKind::Atomic)?;
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let type_name = self.parse_type_name()?;
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

        Ok(Node::new(
            atomic_keyword,
            AtomicTypeSpecifier {
                atomic_keyword,
                open_parenthesis,
                type_name,
                close_parenthesis,
            },
        ))
    }
    fn parse_typeof_specifier(&mut self) -> Res<'a, Node<TypeofSpecifier<'a>>> {
        if self.is(TokenKind::Typeof) {
            let typeof_keyword = self.take(TokenKind::Typeof)?;
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let argument = self.parse_typeof_specifier_argument()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

            Ok(Node::new(
                typeof_keyword,
                TypeofSpecifier::Typeof {
                    typeof_keyword,
                    open_parenthesis,
                    argument,
                    close_parenthesis,
                },
            ))
        } else if self.is(TokenKind::TypeofUnqual) {
            let typeof_unqual_keyword = self.take(TokenKind::TypeofUnqual)?;
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let argument = self.parse_typeof_specifier_argument()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

            Ok(Node::new(
                typeof_unqual_keyword,
                TypeofSpecifier::TypeofUnqual {
                    typeof_unqual_keyword,
                    open_parenthesis,
                    argument,
                    close_parenthesis,
                },
            ))
        } else {
            self.err_expected(Expected::Typeof)?;
            unreachable!()
        }
    }
    fn parse_typeof_specifier_argument(&mut self) -> Res<'a, Node<TypeofSpecifierArgument<'a>>> {
        if let Ok(expr) = self.try_to(Self::parse_expression) {
            Ok(Node::new(
                expr.at(),
                TypeofSpecifierArgument::Expression(expr),
            ))
        } else {
            let type_name = self.parse_type_name()?;
            Ok(Node::new(
                type_name.at(),
                TypeofSpecifierArgument::Type(type_name),
            ))
        }
    }
    fn parse_type_qualifier(&mut self) -> Res<'a, Node<TypeQualifier>> {
        let kind = match self.cur().kind {
            TokenKind::Const => TypeQualifier::Const,
            TokenKind::Restrict => TypeQualifier::Restrict,
            TokenKind::Volatile => TypeQualifier::Volatile,
            TokenKind::Atomic => TypeQualifier::Atomic,
            _ => {
                self.err_expected(Expected::TypeQualifier)?;
                unreachable!()
            }
        };
        let at = self.cur().at;
        self.next();
        Ok(Node::new(at, kind))
    }
    fn parse_function_specifier(&mut self) -> Res<'a, Node<FunctionSpecifier>> {
        let kind = match self.cur().kind {
            TokenKind::Inline => FunctionSpecifier::Inline,
            TokenKind::Noreturn => FunctionSpecifier::NoReturn,
            _ => {
                self.err_expected(Expected::TypeQualifier)?;
                unreachable!()
            }
        };
        let at = self.cur().at;
        self.next();
        Ok(Node::new(at, kind))
    }
    fn parse_alignment_specifier(&mut self) -> Res<'a, Node<AlignmentSpecifier<'a>>> {
        let alignas_keyword = self.take(TokenKind::Alignas)?;
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;

        if let Ok(type_name) = self.try_to(Self::parse_type_name) {
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            Ok(Node::new(
                alignas_keyword,
                AlignmentSpecifier::AsType {
                    alignas_keyword,
                    open_parenthesis,
                    type_name,
                    close_parenthesis,
                },
            ))
        } else {
            let expression = self.parse_constant_expression()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            Ok(Node::new(
                alignas_keyword,
                AlignmentSpecifier::AsExpression {
                    alignas_keyword,
                    open_parenthesis,
                    expression,
                    close_parenthesis,
                },
            ))
        }
    }

    fn parse_declarator(&mut self, is_typedef: bool) -> Res<'a, Node<Declarator<'a>>> {
        let at = self.cur().at;
        let pointer = self.maybe(Self::parse_pointer);
        let direct = self.parse_direct_declarator(is_typedef)?;

        Ok(Node::new(at, Declarator(pointer, direct)))
    }
    fn parse_direct_declarator(&mut self, is_typedef: bool) -> Res<'a, Node<DirectDeclarator<'a>>> {
        let mut left = self.parse_direct_declarator_leaf(is_typedef)?;

        loop {
            match self.cur().kind {
                TokenKind::OpenBracket => left = self.parse_array_declarator(left)?,
                TokenKind::OpenParenthesis => left = self.parse_function_declarator(left)?,
                _ => break,
            }
        }

        Ok(left)
    }
    fn parse_direct_declarator_leaf(
        &mut self,
        is_typedef: bool,
    ) -> Res<'a, Node<DirectDeclarator<'a>>> {
        if self.is(TokenKind::OpenParenthesis) {
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let inner = self.parse_declarator(is_typedef)?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

            Ok(Node::new(
                open_parenthesis,
                DirectDeclarator::Parenthesized {
                    open_parenthesis,
                    inner,
                    close_parenthesis,
                },
            ))
        } else {
            let at = self.cur().at;
            let name = self.take_identifier()?;
            let attributes = self.maybe(Self::parse_attribute_specifier_sequence);

            if is_typedef {
                self.scopes.last_mut().unwrap().typedefs.insert(name);
            }

            Ok(Node::new(at, DirectDeclarator::Name(name, attributes)))
        }
    }
    fn parse_array_declarator(
        &mut self,
        left: Node<DirectDeclarator<'a>>,
    ) -> Res<'a, Node<DirectDeclarator<'a>>> {
        let at = left.at();
        let open_bracket = self.take(TokenKind::OpenBracket)?;
        let decl = if self.is(TokenKind::Static) {
            let static_keyword = self.take(TokenKind::Static)?;
            let qualifiers = self.maybe(Self::parse_type_qualifier_list);
            let length = self.parse_assignment_expression()?;
            let close_bracket = self.take(TokenKind::CloseBracket)?;
            ArrayDeclarator::StaticFirst {
                open_bracket,
                left,
                static_keyword,
                qualifiers,
                length,
                close_bracket,
            }
        } else {
            let qualifiers = self.maybe(Self::parse_type_qualifier_list);
            if qualifiers.is_some() && self.is(TokenKind::Static) {
                let qualifiers = qualifiers.unwrap();
                let static_keyword = self.take(TokenKind::Static)?;
                let length = self.parse_assignment_expression()?;
                let close_bracket = self.take(TokenKind::CloseBracket)?;

                ArrayDeclarator::StaticMid {
                    left,
                    open_bracket,
                    qualifiers,
                    static_keyword,
                    length,
                    close_bracket,
                }
            } else if self.is(TokenKind::Asterisk) {
                let asterisk = self.take(TokenKind::Asterisk)?;
                let close_bracket = self.take(TokenKind::CloseBracket)?;
                ArrayDeclarator::Variable {
                    left,
                    open_bracket,
                    qualifiers,
                    asterisk,
                    close_bracket,
                }
            } else {
                let length = self.maybe(Self::parse_assignment_expression);
                let close_bracket = self.take(TokenKind::CloseBracket)?;
                ArrayDeclarator::NoStatic {
                    left,
                    open_bracket,
                    qualifiers,
                    length,
                    close_bracket,
                }
            }
        };
        let array = Node::new(at, decl);
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);

        Ok(Node::new(
            array.at(),
            DirectDeclarator::Array(array, attributes),
        ))
    }
    fn parse_function_declarator(
        &mut self,
        left: Node<DirectDeclarator<'a>>,
    ) -> Res<'a, Node<DirectDeclarator<'a>>> {
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let parameters = self.maybe(Self::parse_parameter_type_list);
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
        let function = Node::new(
            left.at(),
            FunctionDeclarator {
                left,
                open_parenthesis,
                parameters,
                close_parenthesis,
            },
        );
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        Ok(Node::new(
            function.at(),
            DirectDeclarator::Function(function, attributes),
        ))
    }
    fn parse_pointer(&mut self) -> Res<'a, Node<Pointer<'a>>> {
        let asterisk = self.take(TokenKind::Asterisk)?;
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        let qualifiers = self.maybe(Self::parse_type_qualifier_list);

        if let Ok(outer) = self.try_to(Self::parse_pointer) {
            Ok(Node::new(
                asterisk,
                Pointer::Rec {
                    asterisk,
                    attributes,
                    qualifiers,
                    outer,
                },
            ))
        } else {
            Ok(Node::new(
                asterisk,
                Pointer::Leaf {
                    asterisk,
                    attributes,
                    qualifiers,
                },
            ))
        }
    }
    fn parse_type_qualifier_list(&mut self) -> Res<'a, Node<TypeQualifierList>> {
        self.parse_list(
            Self::parse_type_qualifier,
            TypeQualifierList::Leaf,
            TypeQualifierList::Rec,
        )
    }
    fn parse_parameter_type_list(&mut self) -> Res<'a, Node<ParameterTypeList<'a>>> {
        if self.is(TokenKind::Ellipses) {
            let ellipses = self.take(TokenKind::Ellipses)?;
            Ok(Node::new(ellipses, ParameterTypeList::Var { ellipses }))
        } else {
            let parameters = self.parse_parameter_list()?;
            if self.is(TokenKind::Comma) {
                let comma = self.take(TokenKind::Comma)?;
                let ellipses = self.take(TokenKind::Ellipses)?;
                Ok(Node::new(
                    parameters.at(),
                    ParameterTypeList::WithVar {
                        parameters,
                        comma,
                        ellipses,
                    },
                ))
            } else {
                Ok(Node::new(
                    parameters.at(),
                    ParameterTypeList::NoVar(parameters),
                ))
            }
        }
    }
    fn parse_parameter_list(&mut self) -> Res<'a, Node<ParameterList<'a>>> {
        self.parse_comma_list(
            Self::parse_parameter_declaration,
            ParameterList::Leaf,
            |left, comma, right| ParameterList::Rec { left, comma, right },
        )
    }
    fn parse_parameter_declaration(&mut self) -> Res<'a, Node<ParameterDeclaration<'a>>> {
        let at = self.cur().at;
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        let specifiers = self.parse_declaration_specifiers(&mut false)?;

        if let Ok(declarator) = self.try_to(|p| Self::parse_declarator(p, false)) {
            Ok(Node::new(
                at,
                ParameterDeclaration::Concrete(attributes, specifiers, declarator),
            ))
        } else {
            let declarator = self.maybe(Self::parse_abstract_declarator);
            Ok(Node::new(
                at,
                ParameterDeclaration::Abstract(attributes, specifiers, declarator),
            ))
        }
    }

    fn parse_type_name(&mut self) -> Res<'a, Node<TypeName<'a>>> {
        let specifiers = self.parse_specifier_qualifier_list()?;
        let declarator = self.maybe(Self::parse_abstract_declarator);

        Ok(Node::new(specifiers.at(), TypeName(specifiers, declarator)))
    }

    fn parse_abstract_declarator(&mut self) -> Res<'a, Node<AbstractDeclarator<'a>>> {
        if let Ok(pointer) = self.try_to(Self::parse_pointer) {
            if let Ok(direct) = self.try_to(Self::parse_direct_abstract_declarator) {
                Ok(Node::new(
                    pointer.at(),
                    AbstractDeclarator::Direct(Some(pointer), direct),
                ))
            } else {
                Ok(Node::new(
                    pointer.at(),
                    AbstractDeclarator::Pointer(pointer),
                ))
            }
        } else {
            let direct = self.parse_direct_abstract_declarator()?;
            Ok(Node::new(
                direct.at(),
                AbstractDeclarator::Direct(None, direct),
            ))
        }
    }
    fn parse_direct_abstract_declarator(&mut self) -> Res<'a, Node<DirectAbstractDeclarator<'a>>> {
        let mut left = if let Ok(left) = self.try_to(Self::parse_direct_abstract_declarator_leaf) {
            Some(left)
        } else {
            None
        };

        loop {
            match self.cur().kind {
                TokenKind::OpenBracket => left = Some(self.parse_array_abstract_declarator(left)?),
                TokenKind::OpenParenthesis => {
                    left = Some(self.parse_function_abstract_declarator(left)?)
                }
                _ => break,
            }
        }

        let Some(left) = left else {
            self.err_expected(Expected::AbstractDeclarator)?;
            unreachable!()
        };

        Ok(left)
    }
    fn parse_direct_abstract_declarator_leaf(
        &mut self,
    ) -> Res<'a, Node<DirectAbstractDeclarator<'a>>> {
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let inner = self.parse_abstract_declarator()?;
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;

        Ok(Node::new(
            open_parenthesis,
            DirectAbstractDeclarator::Parenthesized {
                open_parenthesis,
                inner,
                close_parenthesis,
            },
        ))
    }
    fn parse_array_abstract_declarator(
        &mut self,
        left: Option<Node<DirectAbstractDeclarator<'a>>>,
    ) -> Res<'a, Node<DirectAbstractDeclarator<'a>>> {
        let at = left.as_ref().map(|l| l.at()).unwrap_or(self.cur().at);
        let open_bracket = self.take(TokenKind::OpenBracket)?;
        let decl = if self.is(TokenKind::Static) {
            let static_keyword = self.take(TokenKind::Static)?;
            let qualifiers = self.maybe(Self::parse_type_qualifier_list);
            let length = self.parse_assignment_expression()?;
            let close_bracket = self.take(TokenKind::CloseBracket)?;
            ArrayAbstractDeclarator::StaticFirst {
                open_bracket,
                left,
                static_keyword,
                qualifiers,
                length,
                close_bracket,
            }
        } else if self.is(TokenKind::Asterisk) {
            let asterisk = self.take(TokenKind::Asterisk)?;
            let close_bracket = self.take(TokenKind::CloseBracket)?;
            ArrayAbstractDeclarator::Variable {
                left,
                open_bracket,
                asterisk,
                close_bracket,
            }
        } else {
            let qualifiers = self.maybe(Self::parse_type_qualifier_list);
            if qualifiers.is_some() && self.is(TokenKind::Static) {
                let qualifiers = qualifiers.unwrap();
                let static_keyword = self.take(TokenKind::Static)?;
                let length = self.parse_assignment_expression()?;
                let close_bracket = self.take(TokenKind::CloseBracket)?;

                ArrayAbstractDeclarator::StaticMid {
                    left,
                    open_bracket,
                    qualifiers,
                    static_keyword,
                    length,
                    close_bracket,
                }
            } else {
                let length = self.maybe(Self::parse_assignment_expression);
                let close_bracket = self.take(TokenKind::CloseBracket)?;
                ArrayAbstractDeclarator::NoStatic {
                    left,
                    open_bracket,
                    qualifiers,
                    length,
                    close_bracket,
                }
            }
        };
        let array = Node::new(at, decl);
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);

        Ok(Node::new(
            array.at(),
            DirectAbstractDeclarator::Array(array, attributes),
        ))
    }
    fn parse_function_abstract_declarator(
        &mut self,
        left: Option<Node<DirectAbstractDeclarator<'a>>>,
    ) -> Res<'a, Node<DirectAbstractDeclarator<'a>>> {
        let at = left.as_ref().map(|l| l.at()).unwrap_or(self.cur().at);
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let parameters = self.maybe(Self::parse_parameter_type_list);
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
        let function = Node::new(
            at,
            FunctionAbstractDeclarator {
                left,
                open_parenthesis,
                parameters,
                close_parenthesis,
            },
        );
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        Ok(Node::new(
            function.at(),
            DirectAbstractDeclarator::Function(function, attributes),
        ))
    }

    fn parse_typedef_name(&mut self) -> Res<'a, &'a str> {
        let TokenKind::Identifier(name) = self.cur().kind else {
            self.err_expected(Expected::TypedefName)?;
            unreachable!()
        };
        if !self.scopes.last().unwrap().typedefs.contains(name) {
            self.err_expected(Expected::TypedefName)?;
            unreachable!()
        }

        self.next();
        Ok(name)
    }

    fn parse_braced_initializer(&mut self) -> Res<'a, Node<BracedInitializer<'a>>> {
        let open_brace = self.take(TokenKind::OpenBrace)?;
        if self.is(TokenKind::CloseBrace) {
            let close_brace = self.take(TokenKind::CloseBrace)?;
            Ok(Node::new(
                open_brace,
                BracedInitializer::Empty {
                    open_brace,
                    close_brace,
                },
            ))
        } else {
            let initializers = self.parse_initializer_list()?;
            let final_comma = self.maybe(|p| p.take(TokenKind::Comma));
            let close_brace = self.take(TokenKind::CloseBrace)?;
            Ok(Node::new(
                open_brace,
                BracedInitializer::List {
                    open_brace,
                    initializers,
                    final_comma,
                    close_brace,
                },
            ))
        }
    }
    fn parse_initializer(&mut self) -> Res<'a, Node<Initializer<'a>>> {
        if let Ok(expr) = self.try_to(Self::parse_assignment_expression) {
            Ok(Node::new(expr.at(), Initializer::Expression(expr)))
        } else {
            let braced = self.parse_braced_initializer()?;
            Ok(Node::new(braced.at(), Initializer::Braced(braced)))
        }
    }
    fn parse_initializer_list(&mut self) -> Res<'a, Node<InitializerList<'a>>> {
        let at = self.cur().at;
        let designation = self.maybe(Self::parse_designation);
        let initializer = self.parse_initializer()?;
        let mut left = Node::new(at, InitializerList::Leaf(designation, initializer));

        while self.is(TokenKind::Comma) {
            let comma = self.take(TokenKind::Comma)?;
            let designation = self.maybe(Self::parse_designation);
            let initializer = self.parse_initializer()?;
            left = Node::new(
                left.at(),
                InitializerList::Rec {
                    left,
                    comma,
                    designation,
                    initializer,
                },
            )
        }

        Ok(left)
    }
    fn parse_designation(&mut self) -> Res<'a, Node<Designation<'a>>> {
        let designators = self.parse_designator_list()?;
        let equal = self.take(TokenKind::Equal)?;

        Ok(Node::new(
            designators.at(),
            Designation { designators, equal },
        ))
    }
    fn parse_designator_list(&mut self) -> Res<'a, Node<DesignatorList<'a>>> {
        self.parse_list(
            Self::parse_designator,
            DesignatorList::Leaf,
            DesignatorList::Rec,
        )
    }
    fn parse_designator(&mut self) -> Res<'a, Node<Designator<'a>>> {
        if self.is(TokenKind::OpenBracket) {
            let open_bracket = self.take(TokenKind::OpenBracket)?;
            let value = self.parse_constant_expression()?;
            let close_bracket = self.take(TokenKind::CloseBracket)?;

            Ok(Node::new(
                open_bracket,
                Designator::InBrackets {
                    open_bracket,
                    value,
                    close_bracket,
                },
            ))
        } else {
            let period = self.take(TokenKind::Period)?;
            let name = self.take_identifier()?;
            Ok(Node::new(period, Designator::AfterPeriod { period, name }))
        }
    }

    fn parse_static_assert_declaration(&mut self) -> Res<'a, Node<StaticAssertDeclaration<'a>>> {
        let static_assert_keyword = self.take(TokenKind::StaticAssert)?;
        let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
        let condition = self.parse_constant_expression()?;

        let message = if self.is(TokenKind::Comma) {
            let comma = self.take(TokenKind::Comma)?;
            let at = self.cur().at;
            let TokenKind::String(message, encoding) = self.cur().kind else {
                self.err_expected(Expected::StringLiteral)?;
                unreachable!()
            };
            let message = Node::new(at, StringLiteral(message, encoding));
            Some((comma, message))
        } else {
            None
        };
        let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
        let semicolon = self.take(TokenKind::Semicolon)?;

        Ok(Node::new(
            static_assert_keyword,
            StaticAssertDeclaration {
                static_assert_keyword,
                open_parenthesis,
                condition,
                message,
                close_parenthesis,
                semicolon,
            },
        ))
    }

    fn parse_attribute_specifier_sequence(
        &mut self,
    ) -> Res<'a, Node<AttributeSpecifierSequence<'a>>> {
        let left = self.parse_attribute_specifier()?;
        let mut left = Node::new(left.at(), AttributeSpecifierSequence(None, left));

        while let Ok(right) = self.try_to(Self::parse_attribute_specifier) {
            left = Node::new(left.at(), AttributeSpecifierSequence(Some(left), right));
        }

        Ok(left)
    }
    fn parse_attribute_specifier(&mut self) -> Res<'a, Node<AttributeSpecifier<'a>>> {
        let open_bracket_0 = self.take(TokenKind::OpenBracket)?;
        let open_bracket_1 = self.take(TokenKind::OpenBracket)?;
        let attributes = self.parse_attribute_list()?;
        let close_bracket_0 = self.take(TokenKind::CloseBracket)?;
        let close_bracket_1 = self.take(TokenKind::CloseBracket)?;

        Ok(Node::new(
            open_bracket_0,
            AttributeSpecifier {
                open_bracket_0,
                open_bracket_1,
                attributes,
                close_bracket_0,
                close_bracket_1,
            },
        ))
    }
    fn parse_attribute_list(&mut self) -> Res<'a, Node<AttributeList<'a>>> {
        todo!()
    }

    fn parse_statement(&mut self) -> Res<'a, Node<Statement<'a>>> {
        if let Ok(labeled) = self.try_to(Self::parse_labeled_statement) {
            Ok(Node::new(labeled.at(), Statement::Labeled(labeled)))
        } else {
            let unlabeled = self.parse_unlabeled_statement()?;
            Ok(Node::new(unlabeled.at(), Statement::Unlabeled(unlabeled)))
        }
    }
    fn parse_unlabeled_statement(&mut self) -> Res<'a, Node<UnlabeledStatement<'a>>> {
        if let Ok(expr_statement) = self.try_to(Self::parse_expression_statement) {
            Ok(Node::new(
                expr_statement.at(),
                UnlabeledStatement::Expression(expr_statement),
            ))
        } else {
            let at = self.cur().at;
            let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
            if let Ok(primary) = self.try_to(Self::parse_primary_block) {
                Ok(Node::new(
                    at,
                    UnlabeledStatement::Primary(attributes, primary),
                ))
            } else {
                let jump = self.parse_jump_statement()?;
                Ok(Node::new(at, UnlabeledStatement::Jump(attributes, jump)))
            }
        }
    }
    fn parse_primary_block(&mut self) -> Res<'a, Node<PrimaryBlock<'a>>> {
        if let Ok(compound) = self.try_to(Self::parse_compound_statement) {
            Ok(Node::new(compound.at(), PrimaryBlock::Compound(compound)))
        } else if let Ok(selection) = self.try_to(Self::parse_selection_statement) {
            Ok(Node::new(
                selection.at(),
                PrimaryBlock::Selection(selection),
            ))
        } else {
            let iteration = self.parse_iteration_statement()?;
            Ok(Node::new(
                iteration.at(),
                PrimaryBlock::Iteration(iteration),
            ))
        }
    }
    fn parse_secondary_block(&mut self) -> Res<'a, Node<SecondaryBlock<'a>>> {
        let inner = self.parse_statement()?;
        Ok(Node::new(inner.at(), SecondaryBlock(inner)))
    }
    fn parse_label(&mut self) -> Res<'a, Node<Label<'a>>> {
        let at = self.cur().at;
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);

        if self.is(TokenKind::Default) {
            let default_keyword = self.take(TokenKind::Default)?;
            let colon = self.take(TokenKind::Colon)?;
            Ok(Node::new(
                at,
                Label::Default {
                    attributes,
                    default_keyword,
                    colon,
                },
            ))
        } else if self.is(TokenKind::Case) {
            let case_keyword = self.take(TokenKind::Case)?;
            let value = self.parse_constant_expression()?;
            let colon = self.take(TokenKind::Colon)?;
            Ok(Node::new(
                at,
                Label::Case {
                    attributes,
                    case_keyword,
                    value,
                    colon,
                },
            ))
        } else {
            let name = self.take_identifier()?;
            let colon = self.take(TokenKind::Colon)?;
            Ok(Node::new(
                at,
                Label::Named {
                    attributes,
                    name,
                    colon,
                },
            ))
        }
    }
    fn parse_labeled_statement(&mut self) -> Res<'a, Node<LabeledStatement<'a>>> {
        let label = self.parse_label()?;
        let statement = self.parse_statement()?;
        Ok(Node::new(label.at(), LabeledStatement(label, statement)))
    }
    fn parse_compound_statement(&mut self) -> Res<'a, Node<CompoundStatement<'a>>> {
        let open_brace = self.take(TokenKind::OpenBrace)?;
        self.enter_scope();
        let items = self.maybe(Self::parse_block_item_list);
        self.leave_scope();
        let close_brace = self.take(TokenKind::CloseBrace)?;

        Ok(Node::new(
            open_brace,
            CompoundStatement {
                open_brace,
                items,
                close_brace,
            },
        ))
    }
    fn parse_block_item_list(&mut self) -> Res<'a, Node<BlockItemList<'a>>> {
        self.parse_list(
            Self::parse_block_item,
            BlockItemList::Leaf,
            BlockItemList::Rec,
        )
    }
    fn parse_block_item(&mut self) -> Res<'a, Node<BlockItem<'a>>> {
        if let Ok(declaration) = self.try_to(Self::parse_declaration) {
            Ok(Node::new(
                declaration.at(),
                BlockItem::Declaration(declaration),
            ))
        } else if let Ok(unlabeled) = self.try_to(Self::parse_unlabeled_statement) {
            Ok(Node::new(unlabeled.at(), BlockItem::Unlabeled(unlabeled)))
        } else {
            let label = self.parse_label()?;
            Ok(Node::new(label.at(), BlockItem::Label(label)))
        }
    }
    fn parse_expression_statement(&mut self) -> Res<'a, Node<ExpressionStatement<'a>>> {
        if let Ok(attributes) = self.try_to(Self::parse_attribute_specifier_sequence) {
            let expression = self.parse_expression()?;
            let semicolon = self.take(TokenKind::Semicolon)?;
            Ok(Node::new(
                attributes.at(),
                ExpressionStatement::WithAttributes {
                    attributes,
                    expression,
                    semicolon,
                },
            ))
        } else {
            let at = self.cur().at;
            let expression = self.maybe(Self::parse_expression);
            let semicolon = self.take(TokenKind::Semicolon)?;
            Ok(Node::new(
                at,
                ExpressionStatement::WithoutAttributes {
                    expression,
                    semicolon,
                },
            ))
        }
    }
    fn parse_selection_statement(&mut self) -> Res<'a, Node<SelectionStatement<'a>>> {
        if self.is(TokenKind::If) {
            let if_keyword = self.take(TokenKind::If)?;
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let condition = self.parse_expression()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            let then_body = self.parse_secondary_block()?;

            if self.is(TokenKind::Else) {
                let else_keyword = self.take(TokenKind::Else)?;
                let else_body = self.parse_secondary_block()?;
                Ok(Node::new(
                    if_keyword,
                    SelectionStatement::IfElse {
                        if_keyword,
                        open_parenthesis,
                        condition,
                        close_parenthesis,
                        then_body,
                        else_keyword,
                        else_body,
                    },
                ))
            } else {
                Ok(Node::new(
                    if_keyword,
                    SelectionStatement::If {
                        if_keyword,
                        open_parenthesis,
                        condition,
                        close_parenthesis,
                        then_body,
                    },
                ))
            }
        } else if self.is(TokenKind::Switch) {
            let switch_keyword = self.take(TokenKind::Switch)?;
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let selector = self.parse_expression()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            let body = self.parse_secondary_block()?;

            Ok(Node::new(
                switch_keyword,
                SelectionStatement::Switch {
                    switch_keyword,
                    open_parenthesis,
                    selector,
                    close_parenthesis,
                    body,
                },
            ))
        } else {
            self.err_expected(Expected::SelectionStatement)?;
            unreachable!()
        }
    }
    fn parse_iteration_statement(&mut self) -> Res<'a, Node<IterationStatement<'a>>> {
        if self.is(TokenKind::While) {
            let while_keyword = self.take(TokenKind::While)?;
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let condition = self.parse_expression()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            let body = self.parse_secondary_block()?;

            Ok(Node::new(
                while_keyword,
                IterationStatement::While {
                    while_keyword,
                    open_parenthesis,
                    condition,
                    close_parenthesis,
                    body,
                },
            ))
        } else if self.is(TokenKind::Do) {
            let do_keyword = self.take(TokenKind::Do)?;
            let body = self.parse_secondary_block()?;
            let while_keyword = self.take(TokenKind::While)?;
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;
            let condition = self.parse_expression()?;
            let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
            let semicolon = self.take(TokenKind::Semicolon)?;
            Ok(Node::new(
                do_keyword,
                IterationStatement::DoWhile {
                    do_keyword,
                    body,
                    while_keyword,
                    open_parenthesis,
                    condition,
                    close_parenthesis,
                    semicolon,
                },
            ))
        } else if self.is(TokenKind::For) {
            let for_keyword = self.take(TokenKind::For)?;
            let open_parenthesis = self.take(TokenKind::OpenParenthesis)?;

            if let Ok(initializer) = self.try_to(Self::parse_declaration) {
                let condition = self.maybe(Self::parse_expression);
                let counter = self.maybe(Self::parse_expression);
                let semicolon_1 = self.take(TokenKind::Semicolon)?;
                let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
                let body = self.parse_secondary_block()?;

                Ok(Node::new(
                    for_keyword,
                    IterationStatement::ForDeclaration {
                        for_keyword,
                        open_parenthesis,
                        initializer,
                        condition,
                        semicolon_1,
                        counter,
                        close_parenthesis,
                        body,
                    },
                ))
            } else {
                let initializer = self.maybe(Self::parse_expression);
                let semicolon_0 = self.take(TokenKind::Semicolon)?;
                let condition = self.maybe(Self::parse_expression);
                let counter = self.maybe(Self::parse_expression);
                let semicolon_1 = self.take(TokenKind::Semicolon)?;
                let close_parenthesis = self.take(TokenKind::CloseParenthesis)?;
                let body = self.parse_secondary_block()?;
                Ok(Node::new(
                    for_keyword,
                    IterationStatement::For {
                        for_keyword,
                        open_parenthesis,
                        initializer,
                        semicolon_0,
                        condition,
                        semicolon_1,
                        counter,
                        close_parenthesis,
                        body,
                    },
                ))
            }
        } else {
            self.err_expected(Expected::IterationStatement)?;
            unreachable!()
        }
    }
    fn parse_jump_statement(&mut self) -> Res<'a, Node<JumpStatement<'a>>> {
        if self.is(TokenKind::Goto) {
            let goto_keyword = self.take(TokenKind::Goto)?;
            let target = self.take_identifier()?;
            let semicolon = self.take(TokenKind::Semicolon)?;
            Ok(Node::new(
                goto_keyword,
                JumpStatement::Goto {
                    goto_keyword,
                    target,
                    semicolon,
                },
            ))
        } else if self.is(TokenKind::Continue) {
            let continue_keyword = self.take(TokenKind::Continue)?;
            let semicolon = self.take(TokenKind::Semicolon)?;
            Ok(Node::new(
                continue_keyword,
                JumpStatement::Continue {
                    continue_keyword,
                    semicolon,
                },
            ))
        } else if self.is(TokenKind::Break) {
            let break_keyword = self.take(TokenKind::Break)?;
            let semicolon = self.take(TokenKind::Semicolon)?;
            Ok(Node::new(
                break_keyword,
                JumpStatement::Break {
                    break_keyword,
                    semicolon,
                },
            ))
        } else if self.is(TokenKind::Return) {
            let return_keyword = self.take(TokenKind::Return)?;
            let value = self.maybe(Self::parse_expression);
            let semicolon = self.take(TokenKind::Semicolon)?;

            Ok(Node::new(
                return_keyword,
                JumpStatement::Return {
                    return_keyword,
                    value,
                    semicolon,
                },
            ))
        } else {
            self.err_expected(Expected::JumpStatement)?;
            unreachable!()
        }
    }

    fn parse_translation_unit(&mut self) -> Res<'a, Node<TranslationUnit<'a>>> {
        self.enter_scope();
        let ret = self.parse_list(
            Self::parse_external_declaration,
            TranslationUnit::Leaf,
            TranslationUnit::Rec,
        )?;
        self.leave_scope();
        Ok(ret)
    }
    fn parse_external_declaration(&mut self) -> Res<'a, Node<ExternalDeclaration<'a>>> {
        if let Ok(function) = self.try_to(Self::parse_function_definition) {
            Ok(Node::new(
                function.at(),
                ExternalDeclaration::FunctionDefinition(function),
            ))
        } else {
            let declaration = self.parse_declaration()?;
            Ok(Node::new(
                declaration.at(),
                ExternalDeclaration::Declaration(declaration),
            ))
        }
    }
    fn parse_function_definition(&mut self) -> Res<'a, Node<FunctionDefinition<'a>>> {
        let at = self.cur().at;
        let attributes = self.maybe(Self::parse_attribute_specifier_sequence);
        let specifiers = self.parse_declaration_specifiers(&mut false)?;
        let declarator = self.parse_declarator(false)?;
        let function_body = self.parse_function_body()?;
        Ok(Node::new(
            at,
            FunctionDefinition(attributes, specifiers, declarator, function_body),
        ))
    }
    fn parse_function_body(&mut self) -> Res<'a, Node<FunctionBody<'a>>> {
        let compound = self.parse_compound_statement()?;
        Ok(Node::new(compound.at(), FunctionBody(compound)))
    }

    fn try_to<T>(&mut self, mut to: impl FnMut(&mut Self) -> Res<'a, T>) -> Res<'a, T> {
        let index = self.index;
        let scopes = self.scopes.clone();

        match to(self) {
            Ok(t) => Ok(t),
            Err(err) => {
                self.index = index;
                self.scopes = scopes;
                Err(err)
            }
        }
    }
    fn maybe<T>(&mut self, parse: impl FnMut(&mut Self) -> Res<'a, T>) -> Option<T> {
        match self.try_to(parse) {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }
    fn parse_list<L, T>(
        &mut self,
        mut parse: impl FnMut(&mut Self) -> Res<'a, Node<T>>,
        leaf: impl Fn(Node<T>) -> L,
        rec: impl Fn(Node<L>, Node<T>) -> L,
    ) -> Res<'a, Node<L>> {
        let left = parse(self)?;
        let mut left = Node::new(left.at(), leaf(left));

        loop {
            let Ok(right) = self.try_to(&mut parse) else {
                break;
            };
            left = Node::new(left.at(), rec(left, right));
        }

        Ok(left)
    }
    fn parse_comma_list<L, T>(
        &mut self,
        parse: impl Fn(&mut Self) -> Res<'a, Node<T>>,
        leaf: impl Fn(Node<T>) -> L,
        rec: impl Fn(Node<L>, At, Node<T>) -> L,
    ) -> Res<'a, Node<L>> {
        let left = parse(self)?;
        let mut left = Node::new(left.at(), leaf(left));

        while self.is(TokenKind::Comma) {
            let comma = self.take(TokenKind::Comma)?;
            let right = parse(self)?;

            left = Node::new(left.at(), rec(left, comma, right));
        }

        Ok(left)
    }

    fn take(&mut self, kind: TokenKind<'a>) -> Res<'a, At> {
        if !self.is(kind) {
            self.err_expected(kind)?;
            unreachable!()
        }

        let at = self.cur().at;
        self.next();
        Ok(at)
    }
    fn take_identifier(&mut self) -> Res<'a, &'a str> {
        let TokenKind::Identifier(name) = self.cur().kind else {
            self.err_expected(Expected::Identifier)?;
            unreachable!()
        };
        self.next();
        Ok(name)
    }
    fn next(&mut self) {
        self.index += 1;
    }
    fn is(&self, kind: TokenKind) -> bool {
        if self.index >= self.tokens.len() {
            panic!("out-of-bounds check for token {kind:?}");
        }

        let cur = self.cur().kind;
        let is = cur == kind;
        // let compares = if is { "equal" } else { "non-equal" };
        // eprintln!("Token {cur:?} compares {compares} to {kind:?}");
        is
    }
    fn cur(&self) -> Token<'a> {
        self.peek(0)
    }
    fn peek(&self, offset: usize) -> Token<'a> {
        let i = self.index + offset;
        self.tokens[i]
    }

    fn err_expected(&self, expected: impl Into<Expected<'a>>) -> Res<'a, ()> {
        Err(ParseErr {
            at: self.cur(),
            kind: ParseErrKind::Expected(expected.into()),
        })
    }
}

#[derive(Clone, Debug)]
struct Scope<'a> {
    typedefs: HashSet<&'a str>,
}

type Res<'a, T> = Result<T, ParseErr<'a>>;

#[derive(Copy, Clone, Debug)]
pub struct ParseErr<'a> {
    pub at: Token<'a>,
    pub kind: ParseErrKind<'a>,
}

#[derive(Copy, Clone, Debug)]
pub enum ParseErrKind<'a> {
    Expected(Expected<'a>),
}

#[derive(Copy, Clone, Debug)]
pub enum Expected<'a> {
    Token(TokenKind<'a>),
    PrimaryExpression,
    PostfixExpressionLeaf,
    Identifier,
    AssignmentOperator,
    DeclarationSpecifier,
    StorageClassSpecifier,
    TypeSpecifier,
    StructOrUnion,
    Typeof,
    TypeSpecifierQualifier,
    TypeQualifier,
    AbstractDeclarator,
    StringLiteral,
    TypedefName,
    SelectionStatement,
    IterationStatement,
    JumpStatement,
}
impl<'a> From<TokenKind<'a>> for Expected<'a> {
    fn from(value: TokenKind<'a>) -> Self {
        Self::Token(value)
    }
}
