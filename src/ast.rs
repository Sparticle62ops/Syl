#[derive(Debug, Clone)]
pub enum Expr {
    StringLit(String),
    Number(i32),
    Identifier(String),
    Call { action: String, args: Vec<Expr> },
}

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub value: Expr,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assign { name: String, value: Expr },
    Create { var_type: String, name: String, value: Expr },
    Print { value: Expr },
    Mutate { name: String, value: Expr },
    
    Copy { src: String, dst: String },
    Lend { src: String, dst: String },
    Give { src: String, dst: String },
    Discard { name: String },

    IfElse { condition_var: String, condition_val: Expr, then_branch: Vec<Statement>, else_branch: Option<Vec<Statement>> },
    Match { name: String, cases: Vec<MatchCase> },
    ForEach { item: String, collection: String, body: Vec<Statement> },
    Repeat { times: Expr, body: Vec<Statement> },

    Import { filename: String, alias: String },
    DefineAction { name: String, args: Vec<String>, body: Vec<Statement> },
    ExportAction { action: Box<Statement> },
    Return { value: Expr },

    Listen { port: Expr },
    RunBackground { action_call: Expr },
    Wait { action_call: Expr },

    Enforce { name: String, condition: String, crash_msg: String },
}
