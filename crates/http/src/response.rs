use std::io::{self, Write};

use forge_json::{Json, ToJson};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusCode(u16);

impl StatusCode {
    pub const OK: Self = Self(200);
    pub const CREATED: Self = Self(201);
    pub const NO_CONTENT: Self = Self(204);
    pub const BAD_REQUEST: Self = Self(400);
    pub const UNAUTHORIZED: Self = Self(401);
    pub const FORBIDDEN: Self = Self(403);
    pub const NOT_FOUND: Self = Self(404);
    pub const METHOD_NOT_ALLOWED: Self = Self(405);
    pub const CONFLICT: Self = Self(409);
    pub const PAYLOAD_TOO_LARGE: Self = Self(413);
    pub const UNPROCESSABLE_ENTITY: Self = Self(422);
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);
    pub const NOT_IMPLEMENTED: Self = Self(501);

    pub fn new(code: u16) -> Option<Self> {
        (100..=599).contains(&code).then_some(Self(code))
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }

    pub fn reason(&self) -> &'static str {
        match self.0 {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            409 => "Conflict",
            413 => "Payload Too Large",
            418 => "I'm a teapot",
            422 => "Unprocessable Entity",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            _ => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        Self { status, headers: Vec::new(), body: Vec::new() }
    }

    pub fn text(status: StatusCode, body: impl Into<String>) -> Self {
        Self::new(status)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(body.into().into_bytes())
    }

    pub fn json<T: ToJson + ?Sized>(status: StatusCode, value: &T) -> Self {
        Self::new(status)
            .header("Content-Type", "application/json")
            .body(value.to_json().to_string().into_bytes())
    }

    pub fn error(status: StatusCode, message: impl Into<String>) -> Self {
        let body = Json::object([
            ("statusCode", status.as_u16().to_json()),
            ("message", Json::String(message.into())),
            ("error", status.reason().to_json()),
        ]);
        Self::json(status, &body)
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub(crate) fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        write!(w, "HTTP/1.1 {} {}\r\n", self.status.as_u16(), self.status.reason())?;
        for (name, value) in &self.headers {
            write!(w, "{name}: {value}\r\n")?;
        }
        write!(w, "Content-Length: {}\r\n", self.body.len())?;
        write!(w, "Connection: close\r\n\r\n")?;
        w.write_all(&self.body)
    }
}
