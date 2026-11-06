//! Syntax tree for the v0 subset (ADR-0009). Spans are byte offsets.

/// Core effect set. Fixed in v0; unknown names are EFF002 (docs/security/EFFECT_SYSTEM.md).
pub const EFFECTS: &[&str] = &[
    "io",
    "fs",
    "net",
    "time",
    "rand",
    "env",
    "proc",
    "concurrency",
    "ui",
    "unsafe",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    I32,
    U32,
    Bool,
    /// Text of printable ASCII (S2). Values exist at run time only; no operations beyond equality.
    Str,
    Unit,
    /// A record type: an index into `Module::records`. Records are declared at module level (S2).
    Rec(u32),
}

impl Ty {
    pub fn from_name(s: &str) -> Option<Ty> {
        match s {
            "i32" => Some(Ty::I32),
            "u32" => Some(Ty::U32),
            "bool" => Some(Ty::Bool),
            "str" => Some(Ty::Str),
            "unit" => Some(Ty::Unit),
            _ => None,
        }
    }

    /// Name of a primitive type. A record type is named through `Module::records`, see `type_name`.
    pub fn name(self) -> &'static str {
        match self {
            Ty::I32 => "i32",
            Ty::U32 => "u32",
            Ty::Bool => "bool",
            Ty::Str => "str",
            Ty::Unit => "unit",
            Ty::Rec(_) => "record",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Eq,
}

impl BinOp {
    /// Higher binds tighter. `==` is 1, `+ -` is 2, `*` is 3.
    pub fn prec(self) -> u8 {
        match self {
            BinOp::Eq => 1,
            BinOp::Add | BinOp::Sub => 2,
            BinOp::Mul => 3,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Eq => "==",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub start: usize,
    pub end: usize,
    /// Depth of the tree rooted here; bounded by the parser (MAX_DEPTH).
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Int(String),
    Bool(bool),
    /// String literal, value without quotes. Printable ASCII, no quote or backslash (ADR-0015).
    Str(String),
    /// `Name { field: expr, ... }`: every declared field exactly once (S2 records).
    Record {
        name: String,
        name_start: usize,
        name_end: usize,
        fields: Vec<RecordField>,
    },
    /// `expr.field` (S2 records).
    Field {
        base: Box<Expr>,
        name: String,
        name_start: usize,
        name_end: usize,
    },
    Ident(String),
    Call {
        callee: String,
        args: Vec<Expr>,
    },
    Bin {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// Parenthesized expression. The formatter drops parentheses that precedence does not need.
    Group(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Let {
        name: String,
        mutable: bool,
        ty: Option<Ty>,
        value: Expr,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Ty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDef {
    pub name: String,
    pub name_start: usize,
    pub name_end: usize,
    pub params: Vec<Param>,
    pub ret: Ty,
    pub uses: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub records: Vec<RecordDef>,
    pub items: Vec<FnDef>,
    pub pages: Vec<Page>,
}

/// A record declaration (S2): named, typed fields, all required, no methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordDef {
    pub name: String,
    pub name_start: usize,
    pub name_end: usize,
    pub fields: Vec<(String, Ty)>,
}

/// One `name: expr` entry of a record literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordField {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub value: Expr,
}

impl Ty {
    /// Human name of a type in messages: primitives by name, records by their declared name.
    pub fn type_name(self, records: &[RecordDef]) -> String {
        match self {
            Ty::Rec(i) => records
                .get(i as usize)
                .map_or_else(|| "record".to_string(), |r| r.name.clone()),
            other => other.name().to_string(),
        }
    }
}

/// A string literal and its span. `value` excludes the quotes; the span includes them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    pub value: String,
    pub start: usize,
    pub end: usize,
}

/// One content statement of a page (ADR-0015). The set is closed. Target-neutral: no markup here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageItem {
    Title(Text),
    /// Level 1 or 2.
    Heading {
        level: u8,
        text: Text,
    },
    Para(Text),
    Link {
        label: Text,
        href: Text,
    },
}

/// A page: a name, a route path and content statements in source order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub name: String,
    pub name_start: usize,
    pub name_end: usize,
    pub route: Text,
    pub items: Vec<PageItem>,
}
