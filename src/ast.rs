
#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Neg,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    None,
    True,
    False,
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Function(FunctionExpression),
    Call(CallExpression),
    Assignment(AssignmentExpression),
    Unary(UnaryExpression),
    Binary(BinaryExpression),
    Variable(String),
    Literal(Literal),
}

#[derive(Clone, Debug)]
pub struct Program {
    pub stmts: Vec<Box<Expression>>,
}

impl Program {
    pub fn new() -> Program {
        Program { stmts: Vec::new() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionExpression {
    pub name: String,
    pub args: String,
    pub exprs: Vec<Box<Expression>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallExpression {
    pub callable: Box<Expression>,
    pub args: Vec<Box<Expression>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssignmentExpression {
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpression {
    pub lhs: Box<Expression>,
    pub op: Operator,
    pub rhs: Box<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpression {
    pub op: Operator,
    pub expr: Box<Expression>,
}
