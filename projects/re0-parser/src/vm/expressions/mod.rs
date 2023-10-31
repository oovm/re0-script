use super::*;

#[derive(Clone, Debug)]
pub enum Expression {
    Unary(UnaryExpression),
    Binary(BinaryExpression),
    Boolean(bool),
}
#[derive(Clone, Debug)]
pub struct UnaryExpression {
    /// The operator of the unary expression
    pub operator: Operator,
    /// The base of the unary expression
    pub base: Box<Expression>,
}
#[derive(Clone, Debug)]
pub struct BinaryExpression {
    /// The operator of the binary expression
    pub operator: Operator,
    /// Left hand side of the binary expression
    pub lhs: Box<Expression>,
    /// Right hand side of the binary expression
    pub rhs: Box<Expression>,
}
#[derive(Copy, Clone, Debug)]
pub enum Operator {
    /// `+`
    Add,
    /// `+=`
    AddAssign,
    /// `-`
    Sub,
    /// `-=`
    SubAssign,
    /// `*`
    Mul,
    /// `*=`
    MulAssign,
}

impl Expression {
    pub fn prefix(operator: Operator, base: Expression) -> Self {
        Self::Unary(UnaryExpression { operator, base: Box::new(base) })
    }
    pub fn infix(lhs: Expression, operator: Operator, rhs: Expression) -> Self {
        Self::Binary(BinaryExpression { operator, lhs: Box::new(lhs), rhs: Box::new(rhs) })
    }
    pub fn suffix(base: Expression, operator: Operator) -> Self {
        Self::Unary(UnaryExpression { operator, base: Box::new(base) })
    }
}
