use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Var(String),
    Abs { param: String, body: Box<Expr> },
    App(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Var(String),
    Lambda, // J
    Dot,    // ッ
    Plus,   // 足す
    LParen, // 「
    RParen, // 」
}

pub fn parse(input: &str) -> Result<Expr, String> {
    let tokens = lex(input)?;
    let mut p = Parser { tokens, pos: 0 };
    let expr = p.parse_expr()?;
    if p.pos != p.tokens.len() {
        return Err(format!("unexpected token {:?} at token {}", p.tokens[p.pos], p.pos));
    }
    Ok(expr)
}

fn lex(input: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c == 'J' {
            out.push(Tok::Lambda);
            i += 1;
        } else if c == 'ッ' {
            out.push(Tok::Dot);
            i += 1;
        } else if c == '「' {
            out.push(Tok::LParen);
            i += 1;
        } else if c == '」' {
            out.push(Tok::RParen);
            i += 1;
        } else if starts_with(&chars, i, "足す") {
            out.push(Tok::Plus);
            i += 2;
        } else if is_katakana(c) {
            let start = i;
            i += 1;
            while i < chars.len() && is_katakana(chars[i]) && chars[i] != 'ッ' {
                i += 1;
            }
            out.push(Tok::Var(chars[start..i].iter().collect()));
        } else {
            return Err(format!("unexpected character {c:?} at character {i}"));
        }
    }
    Ok(out)
}

fn starts_with(chars: &[char], i: usize, s: &str) -> bool {
    s.chars().enumerate().all(|(off, ch)| chars.get(i + off) == Some(&ch))
}

fn is_katakana(c: char) -> bool {
    ('\u{30A0}'..='\u{30FF}').contains(&c) || ('\u{31F0}'..='\u{31FF}').contains(&c)
}

struct Parser {
    tokens: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_application()
    }

    fn parse_application(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_atom()?;
        while self.eat(&Tok::Plus) {
            let rhs = self.parse_atom()?;
            lhs = Expr::App(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Tok::Var(v)) => {
                let v = v.clone();
                self.pos += 1;
                Ok(Expr::Var(v))
            }
            Some(Tok::Lambda) => self.parse_abs(),
            Some(Tok::LParen) => {
                self.pos += 1;
                let e = self.parse_expr()?;
                if !self.eat(&Tok::RParen) {
                    return Err("expected closing 」".into());
                }
                Ok(e)
            }
            other => Err(format!("expected expression at token {}, got {:?}", self.pos, other)),
        }
    }

    fn parse_abs(&mut self) -> Result<Expr, String> {
        self.expect(&Tok::Lambda)?;
        let param = match self.peek() {
            Some(Tok::Var(v)) => {
                let v = v.clone();
                self.pos += 1;
                v
            }
            other => return Err(format!("expected Katakana variable after J, got {:?}", other)),
        };
        self.expect(&Tok::Dot)?;
        let body = self.parse_expr()?;
        Ok(Expr::Abs { param, body: Box::new(body) })
    }

    fn peek(&self) -> Option<&Tok> { self.tokens.get(self.pos) }

    fn eat(&mut self, tok: &Tok) -> bool {
        if self.peek() == Some(tok) {
            self.pos += 1;
            true
        } else { false }
    }

    fn expect(&mut self, tok: &Tok) -> Result<(), String> {
        if self.eat(tok) { Ok(()) } else { Err(format!("expected {:?} at token {}", tok, self.pos)) }
    }
}

#[derive(Debug, Clone)]
struct Lambda<'a> {
    id: usize,
    param_id: i32,
    body: &'a Expr,
    expr_ptr: *const Expr,
}

pub fn compile_wat(expr: &Expr) -> String {
    let mut symbols = BTreeMap::<String, i32>::new();
    collect_symbols(expr, &mut symbols);
    let mut lambdas = Vec::new();
    collect_lambdas(expr, &symbols, &mut lambdas);
    let lambda_ids: HashMap<*const Expr, usize> = lambdas.iter().map(|l| (l.expr_ptr, l.id)).collect();

    let mut wat = String::new();
    wat.push_str("(module\n");
    wat.push_str(r#"  (memory (export "memory") 1)
  (global $heap (mut i32) (i32.const 16))

  (func $alloc4 (param $a i32) (param $b i32) (param $c i32) (param $d i32) (result i32)
    (local $p i32)
    global.get $heap
    local.set $p
    global.get $heap
    i32.const 16
    i32.add
    global.set $heap
    local.get $p
    local.get $a
    i32.store
    local.get $p
    i32.const 4
    i32.add
    local.get $b
    i32.store
    local.get $p
    i32.const 8
    i32.add
    local.get $c
    i32.store
    local.get $p
    i32.const 12
    i32.add
    local.get $d
    i32.store
    local.get $p)

  (func $closure (param $env i32) (param $code i32) (result i32)
    i32.const 0
    local.get $env
    local.get $code
    i32.const 0
    call $alloc4)

  (func $bind (param $env i32) (param $var i32) (param $val i32) (result i32)
    i32.const 1
    local.get $var
    local.get $val
    local.get $env
    call $alloc4)

  (func $lookup (param $env i32) (param $var i32) (result i32)
    (block $missing
      (loop $again
        local.get $env
        i32.eqz
        br_if $missing
        local.get $env
        i32.const 4
        i32.add
        i32.load
        local.get $var
        i32.eq
        if
          local.get $env
          i32.const 8
          i32.add
          i32.load
          return
        end
        local.get $env
        i32.const 12
        i32.add
        i32.load
        local.set $env
        br $again))
    unreachable)

"#);

    for lam in &lambdas {
        wat.push_str(&format!("  (func $lambda_{} (param $env i32) (param $arg i32) (result i32)\n    (local $newenv i32)\n", lam.id));
        wat.push_str(&format!("    local.get $env\n    i32.const {}\n    local.get $arg\n    call $bind\n    local.set $newenv\n", lam.param_id));
        emit_expr(lam.body, &symbols, &lambda_ids, Some("newenv"), &mut wat, 4);
        wat.push_str("  )\n\n");
    }

    wat.push_str("  (func $apply (param $f i32) (param $arg i32) (result i32)\n");
    for lam in &lambdas {
        wat.push_str(&format!("    local.get $f\n    i32.const 8\n    i32.add\n    i32.load\n    i32.const {}\n    i32.eq\n    if\n      local.get $f\n      i32.const 4\n      i32.add\n      i32.load\n      local.get $arg\n      call $lambda_{}\n      return\n    end\n", lam.id, lam.id));
    }
    wat.push_str("    unreachable)\n\n");

    wat.push_str("  (func (export \"main\") (result i32)\n");
    emit_expr(expr, &symbols, &lambda_ids, None, &mut wat, 2);
    wat.push_str("  )\n)");
    wat
}

fn collect_symbols(expr: &Expr, symbols: &mut BTreeMap<String, i32>) {
    match expr {
        Expr::Var(v) => { intern(symbols, v); }
        Expr::Abs { param, body } => { intern(symbols, param); collect_symbols(body, symbols); }
        Expr::App(a, b) => { collect_symbols(a, symbols); collect_symbols(b, symbols); }
    }
}

fn intern(symbols: &mut BTreeMap<String, i32>, v: &str) -> i32 {
    if let Some(id) = symbols.get(v) { *id } else {
        let id = symbols.len() as i32 + 1;
        symbols.insert(v.to_string(), id);
        id
    }
}

fn collect_lambdas<'a>(expr: &'a Expr, symbols: &BTreeMap<String, i32>, out: &mut Vec<Lambda<'a>>) {
    match expr {
        Expr::Var(_) => {}
        Expr::Abs { param, body } => {
            let id = out.len();
            out.push(Lambda { id, param_id: symbols[param], body, expr_ptr: expr as *const Expr });
            collect_lambdas(body, symbols, out);
        }
        Expr::App(a, b) => { collect_lambdas(a, symbols, out); collect_lambdas(b, symbols, out); }
    }
}

fn emit_expr(expr: &Expr, symbols: &BTreeMap<String, i32>, lambda_ids: &HashMap<*const Expr, usize>, env: Option<&str>, wat: &mut String, indent: usize) {
    let pad = " ".repeat(indent);
    let push_env = |wat: &mut String| {
        if let Some(local) = env {
            wat.push_str(&format!("{pad}local.get ${local}\n"));
        } else {
            wat.push_str(&format!("{pad}i32.const 0\n"));
        }
    };

    match expr {
        Expr::Var(v) => {
            push_env(wat);
            wat.push_str(&format!("{pad}i32.const {}\n{pad}call $lookup\n", symbols[v]));
        }
        Expr::Abs { .. } => {
            let id = lambda_ids[&(expr as *const Expr)];
            push_env(wat);
            wat.push_str(&format!("{pad}i32.const {id}\n{pad}call $closure\n"));
        }
        Expr::App(a, b) => {
            emit_expr(a, symbols, lambda_ids, env, wat, indent);
            emit_expr(b, symbols, lambda_ids, env, wat, indent);
            wat.push_str(&format!("{pad}call $apply\n"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_identity() {
        assert_eq!(parse("Jアッア").unwrap(), Expr::Abs { param: "ア".into(), body: Box::new(Expr::Var("ア".into())) });
    }

    #[test]
    fn parses_application_and_parens() {
        assert_eq!(parse("「Jアッア」足すイ").unwrap(), Expr::App(Box::new(Expr::Abs { param: "ア".into(), body: Box::new(Expr::Var("ア".into())) }), Box::new(Expr::Var("イ".into()))));
    }
}
