#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Field(String),
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),
    Call(String, Vec<Expression>),
}

#[derive(Debug, Clone)]
pub enum Command {
    Where(Expression),
    Limit(i64),
}

#[derive(Debug, Clone)]
pub struct Query {
    pub source: String,
    pub commands: Vec<Command>,
}
