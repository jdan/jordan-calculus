use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Pos {
    line: usize,
    character: usize,
}

#[derive(Debug, Clone, Copy)]
struct Span {
    start: Pos,
    end: Pos,
}

#[derive(Debug, Clone)]
enum TokKind {
    Var(String),
    Lambda,
    Dot,
    Plus,
    LParen,
    RParen,
}

#[derive(Debug, Clone)]
struct Tok {
    kind: TokKind,
    span: Span,
}

#[derive(Debug, Clone)]
enum Expr {
    Var { name: String, span: Span },
    Abs { param: String, param_span: Span, body: Box<Expr>, span: Span },
    App { left: Box<Expr>, right: Box<Expr>, span: Span },
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut output = io::stdout();
    let mut docs = HashMap::<String, String>::new();

    while let Some(msg) = read_message(&mut input)? {
        let Ok(v) = serde_json::from_str::<Value>(&msg) else { continue };
        let method = v.get("method").and_then(Value::as_str);
        match method {
            Some("initialize") => respond(&mut output, &v, json!({
                "capabilities": {
                    "textDocumentSync": 1,
                    "definitionProvider": true,
                    "positionEncoding": "utf-16"
                },
                "serverInfo": { "name": "jordancalculus-lsp" }
            }))?,
            Some("shutdown") => respond(&mut output, &v, Value::Null)?,
            Some("exit") => break,
            Some("textDocument/didOpen") => {
                if let Some(doc) = v.pointer("/params/textDocument") {
                    if let (Some(uri), Some(text)) = (doc.get("uri").and_then(Value::as_str), doc.get("text").and_then(Value::as_str)) {
                        docs.insert(uri.to_string(), text.to_string());
                    }
                }
            }
            Some("textDocument/didChange") => {
                let uri = v.pointer("/params/textDocument/uri").and_then(Value::as_str);
                let text = v.pointer("/params/contentChanges/0/text").and_then(Value::as_str);
                if let (Some(uri), Some(text)) = (uri, text) {
                    docs.insert(uri.to_string(), text.to_string());
                }
            }
            Some("textDocument/definition") => {
                let uri = v.pointer("/params/textDocument/uri").and_then(Value::as_str).unwrap_or_default();
                let pos = Pos {
                    line: v.pointer("/params/position/line").and_then(Value::as_u64).unwrap_or(0) as usize,
                    character: v.pointer("/params/position/character").and_then(Value::as_u64).unwrap_or(0) as usize,
                };
                let result = docs.get(uri).and_then(|text| definition_location(uri, text, pos));
                respond(&mut output, &v, result.unwrap_or(Value::Null))?;
            }
            _ => {
                if v.get("id").is_some() {
                    respond(&mut output, &v, Value::Null)?;
                }
            }
        }
    }
    Ok(())
}

fn read_message<R: BufRead>(input: &mut R) -> io::Result<Option<String>> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            content_length = value.trim().parse::<usize>().ok();
        }
    }
    let Some(len) = content_length else { return Ok(None) };
    let mut buf = vec![0; len];
    input.read_exact(&mut buf)?;
    Ok(Some(String::from_utf8_lossy(&buf).to_string()))
}

fn respond<W: Write>(output: &mut W, request: &Value, result: Value) -> io::Result<()> {
    let response = json!({ "jsonrpc": "2.0", "id": request.get("id").cloned().unwrap_or(Value::Null), "result": result });
    let body = response.to_string();
    write!(output, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    output.flush()
}

fn definition_location(uri: &str, text: &str, pos: Pos) -> Option<Value> {
    let (body, top_defs) = lsp_source_body_and_top_defs(text);
    if let Some((_, span)) = top_defs.iter().find(|(_, span)| contains_fuzzy(*span, pos)) {
        return Some(location(uri, *span));
    }
    if let Some(name) = variable_at_position(text, pos) {
        if let Some((_, span)) = top_defs.iter().rev().find(|(n, _)| n == &name) {
            return Some(location(uri, *span));
        }
    }
    let tokens = lex(&body).ok()?;
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.parse_expr().ok()?;
    let target = find_definition(&expr, pos, &mut Vec::new(), &top_defs)?;
    Some(location(uri, target))
}

fn location(uri: &str, target: Span) -> Value {
    json!([{
        "uri": uri,
        "range": {
            "start": { "line": target.start.line, "character": target.start.character },
            "end": { "line": target.end.line, "character": target.end.character }
        }
    }])
}

fn lsp_source_body_and_top_defs(input: &str) -> (String, Vec<(String, Span)>) {
    let mut body = Vec::new();
    let mut defs = Vec::new();
    for (line_no, line) in input.lines().enumerate() {
        if let Some((name, span)) = top_level_definition_name(line_no, line) {
            defs.push((name, span));
            body.push(String::new());
        } else {
            body.push(line.to_string());
        }
    }
    (body.join("\n"), defs)
}

fn top_level_definition_name(line_no: usize, line: &str) -> Option<(String, Span)> {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while chars.get(i).is_some_and(|c| c.is_whitespace()) { i += 1; }
    if !starts_with(&chars, i, "上げる") { return None; }
    i += "上げる".chars().count();
    while chars.get(i).is_some_and(|c| c.is_whitespace()) { i += 1; }
    let start = i;
    while chars.get(i).is_some_and(|c| *c != 'は') { i += 1; }
    let mut end = i;
    while end > start && chars[end - 1].is_whitespace() { end -= 1; }
    if start == end { return None; }
    let name: String = chars[start..end].iter().collect();
    if !name.chars().all(|c| is_katakana(c) && c != 'ッ') { return None; }
    Some((name, Span {
        start: Pos { line: line_no, character: start },
        end: Pos { line: line_no, character: end },
    }))
}

fn find_definition(expr: &Expr, pos: Pos, env: &mut Vec<(String, Span)>, top_defs: &[(String, Span)]) -> Option<Span> {
    match expr {
        Expr::Var { name, span } => {
            if contains_fuzzy(*span, pos) {
                env.iter()
                    .rev()
                    .find(|(n, _)| n == name)
                    .or_else(|| top_defs.iter().rev().find(|(n, _)| n == name))
                    .map(|(_, s)| *s)
            } else { None }
        }
        Expr::Abs { param, param_span, body, .. } => {
            if contains_fuzzy(*param_span, pos) {
                return Some(*param_span);
            }
            env.push((param.clone(), *param_span));
            let found = find_definition(body, pos, env, top_defs);
            env.pop();
            found
        }
        Expr::App { left, right, .. } => find_definition(left, pos, env, top_defs).or_else(|| find_definition(right, pos, env, top_defs)),
    }
}


fn variable_at_position(input: &str, pos: Pos) -> Option<String> {
    let line = input.lines().nth(pos.line)?;
    let chars: Vec<char> = line.chars().collect();
    for start in 0..chars.len() {
        if !is_variable_char(chars[start]) {
            continue;
        }
        let mut end = start + 1;
        while end < chars.len() && is_variable_char(chars[end]) {
            end += 1;
        }
        let span = Span {
            start: Pos { line: pos.line, character: start },
            end: Pos { line: pos.line, character: end },
        };
        if contains_fuzzy(span, pos) {
            return Some(chars[start..end].iter().collect());
        }
    }
    None
}

fn is_variable_char(c: char) -> bool {
    is_katakana(c) && c != 'ッ'
}

fn contains(span: Span, pos: Pos) -> bool {
    span.start <= pos && pos < span.end
}

fn contains_fuzzy(span: Span, pos: Pos) -> bool {
    if contains(span, pos) {
        return true;
    }
    // Neovim/LSP positions are UTF-16 code-unit offsets. This LSP currently
    // stores character offsets. For non-BMP this would need full conversion;
    // for JordanCalculus' BMP Katakana, the common mismatch is that clients
    // can report a position one codepoint before or after the token while the
    // cursor is visually on it. Accept adjacent positions on the same line so
    // `gd` works anywhere inside a multi-character variable like クミ.
    pos.line == span.start.line
        && pos.line == span.end.line
        && pos.character + 1 >= span.start.character
        && pos.character <= span.end.character
}

fn lex(input: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut line = 0;
    let mut col = 0;
    let mut at_line_start = true;
    while i < chars.len() {
        let c = chars[i];
        if c == 'え' && at_line_start {
            while i < chars.len() && chars[i] != '\n' && chars[i] != '\r' { bump(chars[i], &mut i, &mut line, &mut col); }
        } else if c.is_whitespace() {
            if c == '\n' || c == '\r' { at_line_start = true; }
            bump(c, &mut i, &mut line, &mut col);
        } else if c == 'え' {
            return Err("inline comment".into());
        } else if c == 'J' {
            let start = Pos { line, character: col };
            bump(c, &mut i, &mut line, &mut col);
            at_line_start = false;
            out.push(Tok { kind: TokKind::Lambda, span: Span { start, end: Pos { line, character: col } } });
        } else if c == 'ッ' {
            let start = Pos { line, character: col };
            bump(c, &mut i, &mut line, &mut col);
            at_line_start = false;
            out.push(Tok { kind: TokKind::Dot, span: Span { start, end: Pos { line, character: col } } });
        } else if c == '「' {
            let start = Pos { line, character: col };
            bump(c, &mut i, &mut line, &mut col);
            at_line_start = false;
            out.push(Tok { kind: TokKind::LParen, span: Span { start, end: Pos { line, character: col } } });
        } else if c == '」' {
            let start = Pos { line, character: col };
            bump(c, &mut i, &mut line, &mut col);
            at_line_start = false;
            out.push(Tok { kind: TokKind::RParen, span: Span { start, end: Pos { line, character: col } } });
        } else if starts_with(&chars, i, "足す") {
            let start = Pos { line, character: col };
            bump('足', &mut i, &mut line, &mut col);
            bump('す', &mut i, &mut line, &mut col);
            at_line_start = false;
            out.push(Tok { kind: TokKind::Plus, span: Span { start, end: Pos { line, character: col } } });
        } else if is_katakana(c) {
            let start = Pos { line, character: col };
            let mut name = String::new();
            while i < chars.len() && is_katakana(chars[i]) && chars[i] != 'ッ' {
                name.push(chars[i]);
                bump(chars[i], &mut i, &mut line, &mut col);
            }
            at_line_start = false;
            out.push(Tok { kind: TokKind::Var(name), span: Span { start, end: Pos { line, character: col } } });
        } else {
            return Err(format!("unexpected character {c:?}"));
        }
    }
    Ok(out)
}

fn bump(c: char, i: &mut usize, line: &mut usize, col: &mut usize) {
    *i += 1;
    if c == '\n' { *line += 1; *col = 0; } else { *col += 1; }
}

fn starts_with(chars: &[char], i: usize, s: &str) -> bool {
    s.chars().enumerate().all(|(off, ch)| chars.get(i + off) == Some(&ch))
}

fn is_katakana(c: char) -> bool {
    ('\u{30A0}'..='\u{30FF}').contains(&c) || ('\u{31F0}'..='\u{31FF}').contains(&c)
}

struct Parser { tokens: Vec<Tok>, pos: usize }

impl Parser {
    fn parse_expr(&mut self) -> Result<Expr, String> { self.parse_application() }

    fn parse_application(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_atom()?;
        while matches!(self.peek().map(|t| &t.kind), Some(TokKind::Plus)) {
            self.pos += 1;
            let rhs = self.parse_atom()?;
            let span = Span { start: expr_start(&lhs), end: expr_end(&rhs) };
            lhs = Expr::App { left: Box::new(lhs), right: Box::new(rhs), span };
        }
        Ok(lhs)
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            Some(Tok { kind: TokKind::Var(name), span }) => { self.pos += 1; Ok(Expr::Var { name, span }) }
            Some(Tok { kind: TokKind::Lambda, span: start_span }) => self.parse_abs(start_span.start),
            Some(Tok { kind: TokKind::LParen, .. }) => {
                self.pos += 1;
                let e = self.parse_expr()?;
                match self.peek().cloned() {
                    Some(Tok { kind: TokKind::RParen, .. }) => { self.pos += 1; Ok(e) }
                    _ => Err("expected closing 」".into())
                }
            }
            _ => Err("expected expression".into()),
        }
    }

    fn parse_abs(&mut self, start: Pos) -> Result<Expr, String> {
        self.pos += 1;
        let (param, param_span) = match self.peek().cloned() {
            Some(Tok { kind: TokKind::Var(name), span }) => { self.pos += 1; (name, span) }
            _ => return Err("expected variable after J".into()),
        };
        match self.peek().map(|t| &t.kind) {
            Some(TokKind::Dot) => self.pos += 1,
            _ => return Err("expected ッ".into()),
        }
        let body = self.parse_expr()?;
        let end = expr_end(&body);
        Ok(Expr::Abs { param, param_span, body: Box::new(body), span: Span { start, end } })
    }

    fn peek(&self) -> Option<&Tok> { self.tokens.get(self.pos) }
}

fn expr_start(expr: &Expr) -> Pos {
    match expr {
        Expr::Var { span, .. } | Expr::Abs { span, .. } | Expr::App { span, .. } => span.start,
    }
}

fn expr_end(expr: &Expr) -> Pos {
    match expr {
        Expr::Var { span, .. } | Expr::Abs { span, .. } | Expr::App { span, .. } => span.end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_nearest_binder() {
        let text = "Jアッ「Jイッア」";
        let loc = definition_location("file:///x.jc", text, Pos { line: 0, character: 7 }).unwrap();
        assert_eq!(loc[0]["range"]["start"]["character"], 1);
        assert_eq!(loc[0]["range"]["end"]["character"], 2);
    }

    #[test]
    fn finds_top_level_definition_macro() {
        let text = "上げる アイデンティティ は Jアッア\nアイデンティティ";
        let loc = definition_location("file:///x.jc", text, Pos { line: 1, character: 0 }).unwrap();
        assert_eq!(loc[0]["range"]["start"]["line"], 0);
        assert_eq!(loc[0]["range"]["start"]["character"], 4);
        assert_eq!(loc[0]["range"]["end"]["character"], 12);
    }

    #[test]
    fn finds_definition_from_inside_multi_character_variable() {
        let text = "上げる クミ は Jアッア\nクミ";
        let loc = definition_location("file:///x.jc", text, Pos { line: 1, character: 1 }).unwrap();
        assert_eq!(loc[0]["range"]["start"]["line"], 0);
        assert_eq!(loc[0]["range"]["start"]["character"], 4);
        assert_eq!(loc[0]["range"]["end"]["character"], 6);
    }

    #[test]
    fn finds_top_level_definition_from_definition_body() {
        let text = "上げる クミ は Jアッア\n上げる ステプ は 「クミ」足す「クミ」\nステプ";
        let loc = definition_location("file:///x.jc", text, Pos { line: 1, character: 13 }).unwrap();
        assert_eq!(loc[0]["range"]["start"]["line"], 0);
        assert_eq!(loc[0]["range"]["start"]["character"], 4);
        assert_eq!(loc[0]["range"]["end"]["character"], 6);
    }
}
