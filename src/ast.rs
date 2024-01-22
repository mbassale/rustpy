#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Not,
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
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
    Empty,
    Block(BlockExpression),
    Function(FunctionExpression),
    Call(CallExpression),
    If(IfExpression),
    While(WhileExpression),
    Continue,
    Break,
    Return(ReturnExpression),
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
pub struct BlockExpression {
    pub exprs: Vec<Box<Expression>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionExpression {
    pub name: String,
    pub args: Vec<String>,
    pub body: BlockExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallExpression {
    pub callable: Box<Expression>,
    pub args: Vec<Box<Expression>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub then_branch: Box<Expression>,
    pub elif_branches: Vec<ElifExpression>,
    pub else_branch: Box<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElifExpression {
    pub condition: Box<Expression>,
    pub then_branch: Box<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WhileExpression {
    pub condition: Box<Expression>,
    pub body: Box<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReturnExpression {
    pub expr: Box<Expression>,
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
