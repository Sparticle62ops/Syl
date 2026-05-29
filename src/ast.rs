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
    // v1.4: Math Block binary operations
    BinaryOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    // v1.4: Unary negation
    UnaryNeg { value: Box<Expr> },
    // v1.4: Raylib intrinsics
    DeltaTime,
    MouseX,
    MouseY,
    GetDictKey { dict: String, key: Box<Expr> },
    JsonFromDict { dict: String },
    DictFromJson { json: Box<Expr> },
    CurrentTime,
    CurrentDate,
    // v2.5: Self reference for behaviors
    SelfField { field_name: String },
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Gt,
    Eq,
    Neq,
    Lte,
    Gte,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub value: Expr,
    pub body: Vec<Statement>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Statement {
    Assign { name: String, value: Expr },
    Create { var_type: String, name: String, value: Expr },
    Print { value: Expr },
    DefineAction { name: String, args: Vec<String>, body: Vec<Statement>, doc: Option<String> },
    CallAction { name: String, args: Vec<Expr> },
    Return { value: Expr },
    
    Write { data: Expr, path: Expr },
    Read { path: Expr, identifier: String },
    List { path: Expr, identifier: String },
    Move { path: Expr, destination: Expr },
    CreateFolder { path: Expr },
    
    Mutate { name: String, value: Expr },
    Copy { src: String, dst: String },
    Lend { src: String, dst: String },
    Give { src: String, dst: String },
    Discard { name: String },
    
    IfElse { condition_var: String, condition_val: Expr, then_branch: Vec<Statement>, else_branch: Option<Vec<Statement>> },
    Match { name: String, cases: Vec<MatchCase> },
    ForEach { item: String, collection: String, body: Vec<Statement> },
    Repeat { times: Expr, body: Vec<Statement> },
    WhileNot { condition_action: String, body: Vec<Statement> },
    While { condition: Expr, body: Vec<Statement> },

    IfEndsWith { filename: String, extension: String, body: Vec<Statement> },

    Import { filename: String, alias: String, body: Vec<Statement> },
    ExportAction { action: Box<Statement> },
    External { namespace: String, action: String, lib_path: String },
    
    Listen { port: Expr },
    RunBackground { action_call: Expr },
    Wait { action_call: Expr },

    CreateList { name: String, items: Vec<Expr> },
    AddToList { item: Expr, list_name: String },
    GetItem { identifier: String, index: Expr, list: String },
    SetItem { index: Expr, list: String, value: Expr },
    Increase { name: String, amount: Expr },
    
    Enforce { name: String, condition: String, crash_msg: String },

    IfKeyPressed { key: String, body: Vec<Statement> },
    DrawRect { x: Expr, y: Expr, w: Expr, h: Expr, color: Expr },
    DrawCircle { x: Expr, y: Expr, radius: Expr, color: Expr },
    
    Verify { left: Expr, right: Expr },
    
    DefineEntity { name: String, fields: Vec<(String, Expr)>, doc: Option<String> },
    CreateEntity { entity_type: String, name: String },
    SetField { field_name: String, entity_instance: String, value: Expr },
    Download { url: Expr, target: String },
    ListWords { source: Expr, identifier: String },
    
    IfExpr { condition: Expr, then_branch: Vec<Statement>, else_branch: Option<Vec<Statement>> },
    Execute { namespace: String, action: String, args: Vec<Expr> },

    // v1.7: Dictionaries and HTTP Server
    CreateDictionary { name: String },
    SetDictKey { dict: String, key: Expr, value: Expr },
    ListenHttp { port: Expr },
    HttpRequestRoute { path: Expr, body: Vec<Statement> },
    HttpReply { content: Expr },

    // v1.8: Data Exchange & Graceful Failure
    Attempt { action: Box<Statement> },
    IfFailed { body: Vec<Statement> },
    IfSucceeded { body: Vec<Statement> },

    // v1.9: Database & Time
    ConnectDB { path: Expr, identifier: String },
    ExecuteQuery { query: Expr, db_identifier: String, results_list: Option<String> },

    // v2.0 Terminal Control
    ClearTerminal,
    WaitForKeyPress { var: String },
    SleepMilliseconds { duration: Expr },
    // v2.1 Terminal Graphics
    PrintColored { text: Expr, color: String },
    DrawTerminalBox { x: Expr, y: Expr, w: Expr, h: Expr },
    MoveCursor { x: Expr, y: Expr },

    // v2.5 Entity Behaviors
    DefineBehavior {
        entity_name: String,
        action_name: String,
        args: Vec<String>,
        body: Vec<Statement>,
        doc: Option<String>,
    },
    TriggerBehavior {
        action_name: String,
        instance: String,
        args: Vec<Expr>,
    },
}


#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: String,
    pub target: String,
}
