use itertools::PeekNth;

use super::Lexer;

pub type TokenStream = PeekNth<Lexer>;

#[derive(Debug)]
pub enum Program {
    Program(FunctionDefinition),
}

#[derive(Debug)]
pub enum FunctionDefinition {
    Function { name: String, body: Block },
}

#[derive(Debug)]
pub enum BlockItem {
    S(Statement),
    D(Declaration),
}

#[derive(Debug)]
pub enum Block {
    Block(Vec<BlockItem>),
}

#[derive(Debug)]
pub enum Statement {
    Return(Expression),
    Expression(Expression),
    If {
        condition: Expression,
        then_statement: Box<Statement>,
        else_statement: Option<Box<Statement>>,
    },
    Compound(Block),
    Goto(String),
    Label {
        label: String,
        body: Box<Statement>,
    },
    Break(Option<String>),
    Continue(Option<String>),
    While {
        condition: Expression,
        body: Box<Statement>,
        label: Option<String>,
    },
    DoWhile {
        condition: Expression,
        body: Box<Statement>,
        label: Option<String>,
    },
    For {
        init: ForInit,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Box<Statement>,
        label: Option<String>,
    },
    Switch {
        condition: Expression,
        body: Box<Statement>,
        label: Option<String>,
        case_expressions: Vec<i32>, //label values (when we fold, fill this after we fold)
        default: bool,
    },
    Case {
        condition: Expression,
        body: Box<Statement>,
        label: Option<String>,
    },
    Default {
        body: Box<Statement>,
        label: Option<String>,
    },
    Null,
}

#[derive(Debug)]
pub enum ForInit {
    InitDecl(Declaration),
    InitExp(Option<Expression>),
}

#[derive(Debug)]
pub enum Declaration {
    Declaration {
        name: String,
        init: Option<Expression>,
    },
}

#[derive(Debug)]
pub enum Expression {
    IntConstant(i32),
    Unary {
        unary_operator: UnaryOperator,
        expression: Box<Expression>,
    },
    Binary {
        binary_operator: BinaryOperator,
        left_expression: Box<Expression>,
        right_expression: Box<Expression>,
    },
    Var(String),
    Assignment {
        left_expression: Box<Expression>,
        right_expression: Box<Expression>,
        operator: Option<BinaryOperator>,
    },
    Postfix {
        postfix_operator: PostfixOperator,
        expression: Box<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        true_case: Box<Expression>,
        false_case: Box<Expression>,
    },
}

#[derive(Debug)]
pub enum PostfixOperator {
    Increment,
    Decrement,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
    Increment,
    Decrement,
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    Leq,
    GreaterThan,
    Geq,
    Assigmnent,
    CompoundAssignment(Box<BinaryOperator>),
    Ternary,
}
