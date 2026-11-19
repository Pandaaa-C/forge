use std::collections::HashMap;
use std::fmt;
use std::io::{self, BufRead, Read};

pub(crate) const MAX_HEAD_BYTES: usize = 8 * 1024;
pub(crate) const MAX_BODY_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl Method {
    pub fn parse(s: &str) -> Option<Method> {
        Some(match s {
            "GET" => Method::Get,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "PATCH" => Method::Patch,
            "DELETE" => Method::Delete,
            "HEAD" => Method::Head,
            "OPTIONS" => Method::Options,
            _ => return None,
        })
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub query: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub params: HashMap<String, String>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(String::as_str)
    }

    pub fn body_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }
}

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    Malformed(&'static str),
    TooLarge,
    Unsupported(&'static str),
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError::Io(e)
    }
}

pub(crate) fn parse<R: BufRead>(reader: &mut R) -> Result<Request, ParseError> {
    let mut head_budget = MAX_HEAD_BYTES;

    let line = read_line(reader, &mut head_budget)?;
    let mut parts = line.split(' ');
    let (Some(method), Some(target), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(ParseError::Malformed("bad request line"));
    };
    if !version.starts_with("HTTP/1.") {
        return Err(ParseError::Unsupported("only HTTP/1.x is supported"));
    }
    let method = Method::parse(method).ok_or(ParseError::Unsupported("unknown method"))?;
    if !target.starts_with('/') {
        return Err(ParseError::Malformed("path must start with /"));
    }
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_owned(), Some(q.to_owned())),
        None => (target.to_owned(), None),
    };

    let mut headers = Vec::new();
    loop {
        let line = read_line(reader, &mut head_budget)?;
        if line.is_empty() {
            break;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or(ParseError::Malformed("bad header line"))?;
        headers.push((name.trim().to_owned(), value.trim().to_owned()));
    }

    let mut req = Request {
        method,
        path,
        query,
        headers,
        body: Vec::new(),
        params: HashMap::new(),
    };
    
    if req.header("transfer-encoding").is_some() {
        return Err(ParseError::Unsupported("chunked bodies are not supported yet"));
    }
    if let Some(len) = req.header("content-length") {
        let len: usize = len
            .parse()
            .map_err(|_| ParseError::Malformed("bad content-length"))?;
        if len > MAX_BODY_BYTES {
            return Err(ParseError::TooLarge);
        }
        let mut body = vec![0; len];
        reader.read_exact(&mut body)?;
        req.body = body;
    }

    Ok(req)
}

fn read_line<R: BufRead>(reader: &mut R, budget: &mut usize) -> Result<String, ParseError> {
    let mut buf = Vec::new();
    let n = reader.by_ref().take(*budget as u64).read_until(b'\n', &mut buf)?;
    if n == 0 {
        return Err(ParseError::Malformed("unexpected end of request"));
    }
    if buf.last() != Some(&b'\n') {
        return Err(if n == *budget {
            ParseError::TooLarge
        } else {
            ParseError::Malformed("unexpected end of request")
        });
    }
    *budget -= n;
    buf.pop();
    if buf.last() == Some(&b'\r') {
        buf.pop();
    }
    String::from_utf8(buf).map_err(|_| ParseError::Malformed("header is not valid UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(raw: &str) -> Result<Request, ParseError> {
        parse(&mut raw.as_bytes())
    }

    #[test]
    fn parses_a_post_with_body() {
        let req = parse_str(
            "POST /todos?x=1 HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\nhello",
        )
        .unwrap();
        assert_eq!(req.method, Method::Post);
        assert_eq!(req.path, "/todos");
        assert_eq!(req.query.as_deref(), Some("x=1"));
        assert_eq!(req.header("host"), Some("localhost"));
        assert_eq!(req.body_str(), Some("hello"));
    }

    #[test]
    fn rejects_garbage() {
        assert!(matches!(parse_str("nonsense\r\n\r\n"), Err(ParseError::Malformed(_))));
    }

    #[test]
    fn rejects_huge_headers() {
        let raw = format!("GET / HTTP/1.1\r\nX: {}\r\n\r\n", "a".repeat(MAX_HEAD_BYTES));
        assert!(matches!(parse_str(&raw), Err(ParseError::TooLarge)));
    }
}
