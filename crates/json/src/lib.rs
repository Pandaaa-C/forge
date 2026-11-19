use std::fmt::{self, Write};

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    pub fn object<I, K>(fields: I) -> Json
    where
        I: IntoIterator<Item = (K, Json)>,
        K: Into<String>,
    {
        Json::Object(fields.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Json::Null => f.write_str("null"),
            Json::Bool(b) => write!(f, "{b}"),
            Json::Number(n) => write_number(f, *n),
            Json::String(s) => write_string(f, s),
            Json::Array(items) => {
                f.write_char('[')?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_char(',')?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_char(']')
            }
            Json::Object(fields) => {
                f.write_char('{')?;
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        f.write_char(',')?;
                    }
                    write_string(f, key)?;
                    write!(f, ":{value}")?;
                }
                f.write_char('}')
            }
        }
    }
}

fn write_number(f: &mut fmt::Formatter<'_>, n: f64) -> fmt::Result {
    if !n.is_finite() {
        f.write_str("null")
    } else if n.fract() == 0.0 && n.abs() < 1e15 {
        write!(f, "{}", n as i64)
    } else {
        write!(f, "{n}")
    }
}

fn write_string(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    f.write_char('"')?;
    for c in s.chars() {
        match c {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
            c => f.write_char(c)?,
        }
    }
    f.write_char('"')
}

pub trait ToJson {
    fn to_json(&self) -> Json;
}

impl ToJson for Json {
    fn to_json(&self) -> Json {
        self.clone()
    }
}
impl ToJson for bool {
    fn to_json(&self) -> Json {
        Json::Bool(*self)
    }
}

impl ToJson for str {
    fn to_json(&self) -> Json {
        Json::String(self.to_owned())
    }
}

impl ToJson for String {
    fn to_json(&self) -> Json {
        Json::String(self.clone())
    }
}

macro_rules! number_to_json {
    ($($t:ty),*) => {$(
        impl ToJson for $t {
            fn to_json(&self) -> Json {
                Json::Number(*self as f64)
            }
        }
    )*};
}
number_to_json!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

impl<T: ToJson + ?Sized> ToJson for &T {
    fn to_json(&self) -> Json {
        (**self).to_json()
    }
}

impl<T: ToJson> ToJson for Option<T> {
    fn to_json(&self) -> Json {
        match self {
            Some(v) => v.to_json(),
            None => Json::Null,
        }
    }
}

impl<T: ToJson> ToJson for [T] {
    fn to_json(&self) -> Json {
        Json::Array(self.iter().map(ToJson::to_json).collect())
    }
}

impl<T: ToJson> ToJson for Vec<T> {
    fn to_json(&self) -> Json {
        self.as_slice().to_json()
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while matches!(
            self.peek(),
            Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')
        ) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, b: u8) -> Result<(), String> {
        if self.peek() == Some(b) {
            self.pos += 1;

            Ok(())
        } else {
            Err(format!("Expected '{}' at {}", b as char, self.pos))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.skip_whitespace();

        match self.peek() {
            Some(b'n') => self.literal("null", Json::Null),
            Some(b't') => self.literal("true", Json::Bool(true)),
            Some(b'f') => self.literal("false", Json::Bool(false)),
            Some(b'"') => self.string().map(Json::String),
            Some(b'[') => self.array(),
            Some(b'{') => self.object(),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(format!("unexpected input at {}", self.pos)),
        }
    }
    fn literal(&mut self, word: &str, value: Json) -> Result<Json, String> {
        if self.bytes[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(value)
        } else {
            Err(format!("unexpected input at {}", self.pos))
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.pos;
        while matches!(
            self.peek(),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.pos += 1;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos]).unwrap();
        text.parse()
            .map(Json::Number)
            .map_err(|_| format!("bad number at {start}"))
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = Vec::new();
        loop {
            match self.peek() {
                None => return Err("unterminated string".into()),
                Some(b'"') => {
                    self.pos += 1;
                    break;
                }
                Some(b'\\') => {
                    self.pos += 1;
                    let c = match self.peek() {
                        Some(b'n') => b'\n',
                        Some(b't') => b'\t',
                        Some(b'r') => b'\r',
                        Some(b'"') => b'"',
                        Some(b'\\') => b'\\',
                        Some(b'/') => b'/',
                        _ => return Err(format!("bad escape at {}", self.pos)),
                    };
                    out.push(c);
                    self.pos += 1;
                }
                Some(b) => {
                    out.push(b);
                    self.pos += 1;
                }
            }
        }
        String::from_utf8(out).map_err(|_| "invalid utf-8".into())
    }

    fn array(&mut self) -> Result<Json, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Json::Array(items));
                }
                _ => return Err(format!("expected ',' or ']' at {}", self.pos)),
            }
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.expect(b'{')?;
        let mut fields = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Json::Object(fields));
        }
        loop {
            self.skip_whitespace();
            let key = self.string()?;
            self.skip_whitespace();
            self.expect(b':')?;
            let value = self.value()?;
            fields.push((key, value));
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Json::Object(fields));
                }
                _ => return Err(format!("expected ',' or '}}' at {}", self.pos)),
            }
        }
    }
}

pub fn parse(text: &str) -> Result<Json, String> {
    let mut p = Parser { bytes: text.as_bytes(), pos: 0 };
    let v = p.value()?;
    p.skip_whitespace();
    if p.pos != p.bytes.len() {
        return Err(format!("Trailing characters at {}", p.pos));
    }

    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_nested_values() {
        let v = Json::object([
            ("id", 7.to_json()),
            ("price", 1.5.to_json()),
            ("tags", vec!["a", "b"].to_json()),
            ("note", None::<String>.to_json()),
        ]);
        assert_eq!(
            v.to_string(),
            r#"{"id":7,"price":1.5,"tags":["a","b"],"note":null}"#
        );
    }

    #[test]
    fn escapes_strings() {
        let v = "say \"hi\"\n\\ \u{1}".to_json();
        assert_eq!(v.to_string(), r#""say \"hi\"\n\\ \u0001""#);
    }

    #[test]
    fn non_finite_numbers_become_null() {
        assert_eq!(f64::NAN.to_json().to_string(), "null");
    }
}
