//! Recursive-descent parser for the v0 subset (docs/language/GRAMMAR.md, ADR-0009).
//! Stops at the first syntax error; the checker reports many type errors.

use crate::ast::*;
use crate::diag::Diagnostic;
use crate::lexer::{lex, Tok, Token};

/// Maximum nesting of blocks, parentheses, calls and expression trees.
pub const MAX_DEPTH: usize = 128;

const RESERVED: &[&str] = &[
    "module", "fn", "page", "let", "mut", "uses", "true", "false",
];

pub fn parse(src: &str) -> Result<Module, Diagnostic> {
    let toks = lex(src)?;
    let mut p = Parser {
        src,
        toks,
        pos: 0,
        nest: 0,
    };
    p.module()
}

struct Parser<'a> {
    src: &'a str,
    toks: Vec<Token>,
    pos: usize,
    nest: usize,
}

fn atom(kind: ExprKind, start: usize, end: usize) -> Expr {
    Expr {
        kind,
        start,
        end,
        depth: 1,
    }
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.tok)
    }

    fn here(&self) -> (usize, usize) {
        match self.toks.get(self.pos) {
            Some(t) => (t.start, t.end),
            None => (self.src.len(), self.src.len()),
        }
    }

    fn err(&self, code: &'static str, msg: impl Into<String>) -> Diagnostic {
        let (s, e) = self.here();
        Diagnostic::new(self.src, code, msg, s, e)
    }

    fn is_sym(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Sym(x)) if *x == s)
    }

    fn is_kw(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Ident(x)) if x == s)
    }

    fn expect_sym(&mut self, s: &'static str) -> Result<(), Diagnostic> {
        if self.is_sym(s) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err("PAR001", format!("expected '{s}'")))
        }
    }

    fn expect_kw(&mut self, s: &'static str) -> Result<(), Diagnostic> {
        if self.is_kw(s) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err("PAR001", format!("expected '{s}'")))
        }
    }

    fn ident(&mut self) -> Result<(String, usize, usize), Diagnostic> {
        if let Some(Tok::Ident(name)) = self.peek() {
            if !RESERVED.contains(&name.as_str()) {
                let name = name.clone();
                let t = self.toks[self.pos].clone();
                self.pos += 1;
                return Ok((name, t.start, t.end));
            }
        }
        Err(self.err("PAR001", "expected identifier"))
    }

    fn enter(&mut self) -> Result<(), Diagnostic> {
        self.nest += 1;
        if self.nest > MAX_DEPTH {
            return Err(self.err("PAR002", "nesting too deep"));
        }
        Ok(())
    }

    fn leave(&mut self) {
        self.nest -= 1;
    }

    fn module(&mut self) -> Result<Module, Diagnostic> {
        self.expect_kw("module")?;
        let (name, _, _) = self.ident()?;
        let mut items = Vec::new();
        let mut pages = Vec::new();
        while self.pos < self.toks.len() {
            if self.is_kw("page") {
                pages.push(self.page_def()?);
            } else {
                items.push(self.fn_def()?);
            }
        }
        Ok(Module { name, items, pages })
    }

    /// A string literal where the grammar requires one.
    fn text(&mut self) -> Result<Text, Diagnostic> {
        if let Some(Tok::Str(value)) = self.peek() {
            let value = value.clone();
            let t = self.toks[self.pos].clone();
            self.pos += 1;
            return Ok(Text {
                value,
                start: t.start,
                end: t.end,
            });
        }
        Err(self.err("PAR001", "expected a string literal"))
    }

    /// `page <name> "<route>" { <statement>; ... }` (ADR-0015).
    fn page_def(&mut self) -> Result<Page, Diagnostic> {
        self.expect_kw("page")?;
        let (name, name_start, name_end) = self.ident()?;
        let route = self.text()?;
        self.expect_sym("{")?;
        let mut items = Vec::new();
        while !self.is_sym("}") {
            let (kw, ks, ke) = match self.peek() {
                Some(Tok::Ident(kw)) => {
                    let kw = kw.clone();
                    let t = self.toks[self.pos].clone();
                    self.pos += 1;
                    (kw, t.start, t.end)
                }
                _ => return Err(self.err("PAR001", "expected a page statement or '}'")),
            };
            let item = match kw.as_str() {
                "title" => PageItem::Title(self.text()?),
                "h1" => PageItem::Heading {
                    level: 1,
                    text: self.text()?,
                },
                "h2" => PageItem::Heading {
                    level: 2,
                    text: self.text()?,
                },
                "p" => PageItem::Para(self.text()?),
                "link" => {
                    let label = self.text()?;
                    let href = self.text()?;
                    PageItem::Link { label, href }
                }
                _ => {
                    return Err(Diagnostic::new(
                        self.src,
                        "PAR001",
                        format!("unknown page statement '{kw}'"),
                        ks,
                        ke,
                    ))
                }
            };
            self.expect_sym(";")?;
            items.push(item);
        }
        self.expect_sym("}")?;
        Ok(Page {
            name,
            name_start,
            name_end,
            route,
            items,
        })
    }

    fn fn_def(&mut self) -> Result<FnDef, Diagnostic> {
        self.expect_kw("fn")?;
        let (name, name_start, name_end) = self.ident()?;
        self.expect_sym("(")?;
        let mut params = Vec::new();
        if !self.is_sym(")") {
            loop {
                let (pname, _, _) = self.ident()?;
                self.expect_sym(":")?;
                let ty = self.ty()?;
                params.push(Param { name: pname, ty });
                if self.is_sym(",") {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        self.expect_sym(")")?;
        let ret = if self.is_sym("->") {
            self.pos += 1;
            self.ty()?
        } else {
            Ty::Unit
        };
        let mut uses = Vec::new();
        if self.is_kw("uses") {
            self.pos += 1;
            loop {
                let (eff, s, e) = self.ident()?;
                if !EFFECTS.contains(&eff.as_str()) {
                    return Err(Diagnostic::new(
                        self.src,
                        "EFF002",
                        format!("unknown effect '{eff}'"),
                        s,
                        e,
                    ));
                }
                uses.push(eff);
                if self.is_sym(",") {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        let body = self.block()?;
        Ok(FnDef {
            name,
            name_start,
            name_end,
            params,
            ret,
            uses,
            body,
        })
    }

    fn ty(&mut self) -> Result<Ty, Diagnostic> {
        let (name, s, e) = self.ident()?;
        Ty::from_name(&name).ok_or_else(|| {
            Diagnostic::new(self.src, "TYP004", format!("unknown type '{name}'"), s, e)
        })
    }

    fn block(&mut self) -> Result<Block, Diagnostic> {
        self.enter()?;
        self.expect_sym("{")?;
        let mut stmts = Vec::new();
        let mut tail = None;
        loop {
            if self.is_sym("}") {
                break;
            }
            if self.is_kw("let") {
                self.pos += 1;
                let mutable = if self.is_kw("mut") {
                    self.pos += 1;
                    true
                } else {
                    false
                };
                let (name, _, _) = self.ident()?;
                let ty = if self.is_sym(":") {
                    self.pos += 1;
                    Some(self.ty()?)
                } else {
                    None
                };
                self.expect_sym("=")?;
                let value = self.expr()?;
                self.expect_sym(";")?;
                stmts.push(Stmt::Let {
                    name,
                    mutable,
                    ty,
                    value,
                });
            } else {
                let e = self.expr()?;
                if self.is_sym(";") {
                    self.pos += 1;
                    stmts.push(Stmt::Expr(e));
                } else if self.is_sym("}") {
                    tail = Some(e);
                    break;
                } else {
                    return Err(self.err("PAR001", "expected ';' or '}' after expression"));
                }
            }
        }
        self.expect_sym("}")?;
        self.leave();
        Ok(Block { stmts, tail })
    }

    fn expr(&mut self) -> Result<Expr, Diagnostic> {
        self.binary(1)
    }

    fn peek_binop(&self) -> Option<BinOp> {
        match self.peek() {
            Some(Tok::Sym("+")) => Some(BinOp::Add),
            Some(Tok::Sym("-")) => Some(BinOp::Sub),
            Some(Tok::Sym("*")) => Some(BinOp::Mul),
            Some(Tok::Sym("==")) => Some(BinOp::Eq),
            _ => None,
        }
    }

    fn binary(&mut self, min: u8) -> Result<Expr, Diagnostic> {
        let mut lhs = self.primary()?;
        while let Some(op) = self.peek_binop() {
            if op.prec() < min {
                break;
            }
            if op == BinOp::Eq && matches!(lhs.kind, ExprKind::Bin { op: BinOp::Eq, .. }) {
                return Err(self.err(
                    "PAR005",
                    "comparison operators cannot be chained; add parentheses",
                ));
            }
            self.pos += 1;
            let rhs = self.binary(op.prec() + 1)?;
            let depth = 1 + lhs.depth.max(rhs.depth);
            if depth > MAX_DEPTH {
                return Err(self.err("PAR002", "expression nesting too deep"));
            }
            let (start, end) = (lhs.start, rhs.end);
            lhs = Expr {
                kind: ExprKind::Bin {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                start,
                end,
                depth,
            };
        }
        Ok(lhs)
    }

    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        let tok = match self.toks.get(self.pos) {
            Some(t) => t.clone(),
            None => return Err(self.err("PAR001", "expected expression")),
        };
        match &tok.tok {
            Tok::Int(text) => {
                self.pos += 1;
                Ok(atom(ExprKind::Int(text.clone()), tok.start, tok.end))
            }
            Tok::Ident(word) if word == "true" || word == "false" => {
                self.pos += 1;
                Ok(atom(ExprKind::Bool(word == "true"), tok.start, tok.end))
            }
            Tok::Ident(_) => {
                let (name, s, e) = self.ident()?;
                if !self.is_sym("(") {
                    return Ok(atom(ExprKind::Ident(name), s, e));
                }
                self.enter()?;
                self.pos += 1;
                let mut args = Vec::new();
                if !self.is_sym(")") {
                    loop {
                        args.push(self.expr()?);
                        if self.is_sym(",") {
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                }
                self.expect_sym(")")?;
                self.leave();
                let end = self.toks[self.pos - 1].end;
                let depth = 1 + args.iter().map(|a| a.depth).max().unwrap_or(0);
                if depth > MAX_DEPTH {
                    return Err(self.err("PAR002", "expression nesting too deep"));
                }
                Ok(Expr {
                    kind: ExprKind::Call { callee: name, args },
                    start: s,
                    end,
                    depth,
                })
            }
            Tok::Sym("(") => {
                self.enter()?;
                self.pos += 1;
                let inner = self.expr()?;
                self.expect_sym(")")?;
                self.leave();
                let depth = inner.depth + 1;
                if depth > MAX_DEPTH {
                    return Err(self.err("PAR002", "expression nesting too deep"));
                }
                let end = self.toks[self.pos - 1].end;
                Ok(Expr {
                    kind: ExprKind::Group(Box::new(inner)),
                    start: tok.start,
                    end,
                    depth,
                })
            }
            _ => Err(self.err("PAR001", "expected expression")),
        }
    }
}
