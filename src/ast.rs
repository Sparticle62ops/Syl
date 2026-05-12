#[derive(Debug, Clone)]
pub enum Expr {
    StringLit(String),
    Number(i32),
    Identifier(String),
    Call { action: String, args: Vec<Expr> },
    GetField { field_name: String, entity_instance: String },
    Join { left: Box<Expr>, right: Box<Expr> },
    Sqrt { value: Box<Expr> },
    Pow { base: Box<Expr>, exponent: Box<Expr> },
    Random { min: Box<Expr>, max: Box<Expr> },
    FileSize { path: Box<Expr> },
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
    Write { data: Expr, path: Expr },
    Read { path: Expr, identifier: String },
    List { path: Expr, identifier: String },
    Move { path: Expr, destination: Expr },
    CreateFolder { path: Expr },
    External { namespace: String, action: String, lib_path: String },
    Execute { namespace: String, action: String, args: Vec<Expr> },
    CreateList { name: String, items: Vec<Expr> },
    AddToList { item: Expr, list_name: String },
    CallAction { name: String, args: Vec<Expr> },
    WhileNot { condition_action: String, body: Vec<Statement> },
    IfKeyPressed { key: String, body: Vec<Statement> },
    GetItem { identifier: String, index: Expr, list: String },
    SetItem { index: Expr, list: String, value: Expr },
    Increase { name: String, amount: Expr },
    Mutate { name: String, value: Expr },
    
    Copy { src: String, dst: String },
    Lend { src: String, dst: String },
    Give { src: String, dst: String },
    Discard { name: String },

    IfElse { condition_var: String, condition_val: Expr, then_branch: Vec<Statement>, else_branch: Option<Vec<Statement>> },
    IfEndsWith { filename: String, extension: String, body: Vec<Statement> },
    Match { name: String, cases: Vec<MatchCase> },
    ForEach { item: String, collection: String, body: Vec<Statement> },
    Repeat { times: Expr, body: Vec<Statement> },

    Import { filename: String, alias: String, body: Vec<Statement> },
    DefineAction { name: String, args: Vec<String>, body: Vec<Statement> },
    ExportAction { action: Box<Statement> },
    Return { value: Expr },

    Listen { port: Expr },
    RunBackground { action_call: Expr },
    Wait { action_call: Expr },

    DefineEntity { name: String, fields: Vec<(String, Expr)> },
    CreateEntity { entity_type: String, name: String },
    SetField { field_name: String, entity_instance: String, value: Expr },
    Download { url: Expr, target: String },
    ListWords { source: Expr, identifier: String },

    Enforce { name: String, condition: String, crash_msg: String },
}

#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: String,
    pub target: String,
}
