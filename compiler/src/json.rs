//! Minimal JSON reader for patch documents (docs/compiler/SEMANTIC_PATCH_API.md).
//! The compiler has no dependencies, so this accepts only what patches need: objects, arrays,
//! strings, integers, booleans and null. Floats are rejected; surrogate pairs are rejected.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(items) => Some(items),
            _ => None,
        }
    }
}

pub fn parse(src: &str) -> Result<Json, String> {
    let mut p = Reader { chars: src.chars().collect(), pos: 0 };
    p.skip_ws();
    let value = p.value(0)?;
    p.skip_ws();
    if p.pos != p.chars.len() {
        return Err(format!("unexpected trailing content at character {}", p.pos));
    }
    Ok(value)
}

const MAX_JSON_DEPTH: usize = 32;

struct Reader {
    chars: Vec<char>,
    pos: usize,
}

impl Reader {
    fn skip_ws(&mut self) {
        while matches!(self.chars.get(self.pos), Some(' ' | '\t' | '\n' | '\r')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected '{c}' at character {}", self.pos))
        }
    }

    fn literal(&mut self, word: &str) -> Result<(), String> {
        for c in word.chars() {
            self.expect(c)?;
        }
        Ok(())
    }

    fn value(&mut self, depth: usize) -> Result<Json, String> {
        if depth > MAX_JSON_DEPTH {
            return Err("nesting exceeds 32 levels".into());
        }
        match self.peek() {
            Some('{') => self.object(depth),
            Some('[') => self.array(depth),
            Some('"') => self.string().map(Json::Str),
            Some('t') => self.literal("true").map(|_| Json::Bool(true)),
            Some('f') => self.literal("false").map(|_| Json::Bool(false)),
            Some('n') => self.literal("null").map(|_| Json::Null),
            Some(c) if c == '-' || c.is_ascii_digit() => self.integer(),
            Some(c) => Err(format!("unexpected character '{c}' at character {}", self.pos)),
            None => Err("unexpected end of input".into()),
        }
    }

    fn object(&mut self, depth: usize) -> Result<Json, String> {
        self.expect('{')?;
        let mut fields = Vec::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.pos += 1;
            return Ok(Json::Obj(fields));
        }
        loop {
            self.skip_ws();
            let key = self.string()?;
            if fields.iter().any(|(k, _)| *k == key) {
                return Err(format!("duplicate key '{key}'"));
            }
            self.skip_ws();
            self.expect(':')?;
            self.skip_ws();
            fields.push((key, self.value(depth + 1)?));
            self.skip_ws();
            match self.peek() {
                Some(',') => self.pos += 1,
                Some('}') => {
                    self.pos += 1;
                    return Ok(Json::Obj(fields));
                }
                _ => return Err(format!("expected ',' or '}}' at character {}", self.pos)),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<Json, String> {
        self.expect('[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            self.skip_ws();
            items.push(self.value(depth + 1)?);
            self.skip_ws();
            match self.peek() {
                Some(',') => self.pos += 1,
                Some(']') => {
                    self.pos += 1;
                    return Ok(Json::Arr(items));
                }
                _ => return Err(format!("expected ',' or ']' at character {}", self.pos)),
            }
        }
    }

    fn integer(&mut self) -> Result<Json, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        let digits_start = self.pos;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.pos == digits_start {
            return Err(format!("invalid number at character {start}"));
        }
        if matches!(self.peek(), Some('.' | 'e' | 'E')) {
            return Err(format!("floating-point numbers are not allowed at character {start}"));
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        text.parse::<i64>().map(Json::Int).map_err(|_| format!("integer out of range at character {start}"))
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err("unterminated string".into()),
                Some('"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some('\\') => {
                    self.pos += 1;
                    let esc = self.peek().ok_or("unterminated escape")?;
                    self.pos += 1;
                    match esc {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{8}'),
                        'f' => out.push('\u{c}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let mut code = 0u32;
                            for _ in 0..4 {
                                let d = self.peek().and_then(|c| c.to_digit(16)).ok_or("invalid \\u escape")?;
                                code = code * 16 + d;
                                self.pos += 1;
                            }
                            let c = char::from_u32(code).ok_or("\\u escape is a surrogate, not allowed")?;
                            out.push(c);
                        }
                        other => return Err(format!("invalid escape '\\{other}'")),
                    }
                }
                Some(c) if (c as u32) < 0x20 => return Err("control character in string".into()),
                Some(c) => {
                    out.push(c);
                    self.pos += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_patch_shaped_document() {
        let v = parse(r#"{"api":"aqelith.patch.v1","ops":[{"op":"rename","n":-3,"ok":true,"x":null}]}"#).unwrap();
        assert_eq!(v.get("api").and_then(Json::as_str), Some("aqelith.patch.v1"));
        assert_eq!(v.get("ops").and_then(Json::as_array).map(|a| a.len()), Some(1));
    }

    #[test]
    fn rejects_floats_duplicates_and_trailing_text() {
        assert!(parse("1.5").is_err());
        assert!(parse(r#"{"a":1,"a":2}"#).is_err());
        assert!(parse("{} x").is_err());
        assert!(parse(r#""\ud800""#).is_err());
    }
}
