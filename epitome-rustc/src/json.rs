//! Just enough JSON for talking to the proc-macro server: a value type, a
//! reader and a writer.

use std::fmt::Write;

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    /// Members in the order written.
    Object(Vec<(String, Json)>),
}

impl Json {
    /// The member `key` of an object.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(m) => m.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Json::Number(n) if *n >= 0.0 && *n <= u32::MAX as f64 => Some(*n as u32),
            _ => None,
        }
    }

    /// An array of numbers, as `u32`s.
    pub fn as_u32s(&self) -> Option<Vec<u32>> {
        self.as_array()?.iter().map(Json::as_u32).collect()
    }

    /// The value as JSON text, on one line.
    pub fn write(&self, out: &mut String) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    let _ = write!(out, "{}", *n as i64);
                } else {
                    let _ = write!(out, "{n}");
                }
            }
            Json::String(s) => write_string(s, out),
            Json::Array(a) => {
                out.push('[');
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    v.write(out);
                }
                out.push(']');
            }
            Json::Object(m) => {
                out.push('{');
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_string(k, out);
                    out.push(':');
                    v.write(out);
                }
                out.push('}');
            }
        }
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        self.write(&mut s);
        s
    }

    /// Reads one value from `text`.
    pub fn parse(text: &str) -> Result<Json, String> {
        let mut r = Reader { b: text.as_bytes(), i: 0 };
        let v = r.value()?;
        r.ws();
        if r.i != r.b.len() {
            return Err(format!("unexpected text at byte {}", r.i));
        }
        Ok(v)
    }
}

/// An object from members.
pub fn obj(members: Vec<(&str, Json)>) -> Json {
    Json::Object(members.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

/// An array of numbers.
pub fn nums(v: &[u32]) -> Json {
    Json::Array(v.iter().map(|&n| Json::Number(n as f64)).collect())
}

fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

struct Reader<'t> {
    b: &'t [u8],
    i: usize,
}

impl<'t> Reader<'t> {
    fn ws(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.ws();
        if self.b.get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.b.get(self.i) {
            Some(b'{') => {
                self.i += 1;
                let mut m = Vec::new();
                if self.eat(b'}') {
                    return Ok(Json::Object(m));
                }
                loop {
                    self.ws();
                    let k = self.string()?;
                    if !self.eat(b':') {
                        return Err("expected `:`".into());
                    }
                    m.push((k, self.value()?));
                    if self.eat(b'}') {
                        return Ok(Json::Object(m));
                    }
                    if !self.eat(b',') {
                        return Err("expected `,` or `}`".into());
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut a = Vec::new();
                if self.eat(b']') {
                    return Ok(Json::Array(a));
                }
                loop {
                    a.push(self.value()?);
                    if self.eat(b']') {
                        return Ok(Json::Array(a));
                    }
                    if !self.eat(b',') {
                        return Err("expected `,` or `]`".into());
                    }
                }
            }
            Some(b'"') => Ok(Json::String(self.string()?)),
            Some(b't') if self.b[self.i..].starts_with(b"true") => {
                self.i += 4;
                Ok(Json::Bool(true))
            }
            Some(b'f') if self.b[self.i..].starts_with(b"false") => {
                self.i += 5;
                Ok(Json::Bool(false))
            }
            Some(b'n') if self.b[self.i..].starts_with(b"null") => {
                self.i += 4;
                Ok(Json::Null)
            }
            Some(_) => {
                let s = self.i;
                while self.i < self.b.len() && matches!(self.b[self.i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
                {
                    self.i += 1;
                }
                let t = std::str::from_utf8(&self.b[s..self.i]).unwrap_or("");
                t.parse().map(Json::Number).map_err(|_| format!("bad JSON value at byte {s}"))
            }
            None => Err("unexpected end of JSON".into()),
        }
    }

    fn string(&mut self) -> Result<String, String> {
        if self.b.get(self.i) != Some(&b'"') {
            return Err(format!("expected a string at byte {}", self.i));
        }
        self.i += 1;
        let mut out = Vec::new();
        while let Some(&c) = self.b.get(self.i) {
            self.i += 1;
            match c {
                b'"' => return String::from_utf8(out).map_err(|e| e.to_string()),
                b'\\' => {
                    let e = *self.b.get(self.i).ok_or("bad escape")?;
                    self.i += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'u' => {
                            let hex = std::str::from_utf8(self.b.get(self.i..self.i + 4).ok_or("bad \\u")?)
                                .map_err(|e| e.to_string())?;
                            let mut code = u32::from_str_radix(hex, 16).map_err(|e| e.to_string())?;
                            self.i += 4;
                            if (0xD800..0xDC00).contains(&code) && self.b[self.i..].starts_with(b"\\u") {
                                let lo = std::str::from_utf8(&self.b[self.i + 2..self.i + 6]).unwrap_or("0");
                                let lo = u32::from_str_radix(lo, 16).unwrap_or(0);
                                code = 0x10000 + ((code - 0xD800) << 10) + (lo - 0xDC00);
                                self.i += 6;
                            }
                            let c = char::from_u32(code).unwrap_or('\u{fffd}');
                            let mut buf = [0; 4];
                            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                        }
                        other => out.push(other),
                    }
                }
                c => out.push(c),
            }
        }
        Err("unterminated string".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let text = r#"{"a":[1,2,{"b":"x\"y\né"}],"c":null,"d":true,"e":-1.5}"#;
        let v = Json::parse(text).unwrap();
        assert_eq!(v.get("a").unwrap().as_array().unwrap()[1], Json::Number(2.0));
        assert_eq!(v.get("a").unwrap().as_array().unwrap()[2].get("b").unwrap().as_str(), Some("x\"y\né"));
        assert_eq!(Json::parse(&v.to_text()).unwrap(), v);
    }
}
