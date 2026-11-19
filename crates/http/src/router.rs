use std::collections::HashMap;
use std::sync::Arc;

use crate::request::{Method, Request};
use crate::response::{Response, StatusCode};

pub type Handler = Arc<dyn Fn(&Request) -> Response + Send + Sync>;

#[derive(Clone)]
enum Segment {
    Static(String),
    Param(String),
}

#[derive(Clone)]
struct Route {
    method: Method,
    segments: Vec<Segment>,
    handler: Handler,
}

#[derive(Clone, Default)]
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(
        mut self,
        method: Method,
        path: &str,
        handler: impl Fn(&Request) -> Response + Send + Sync + 'static,
    ) -> Self {
        self.routes.push(Route {
            method,
            segments: parse_pattern(path),
            handler: Arc::new(handler),
        });
        self
    }

    pub fn get(self, path: &str, h: impl Fn(&Request) -> Response + Send + Sync + 'static) -> Self {
        self.route(Method::Get, path, h)
    }
    pub fn post(self, path: &str, h: impl Fn(&Request) -> Response + Send + Sync + 'static) -> Self {
        self.route(Method::Post, path, h)
    }
    pub fn put(self, path: &str, h: impl Fn(&Request) -> Response + Send + Sync + 'static) -> Self {
        self.route(Method::Put, path, h)
    }
    pub fn patch(self, path: &str, h: impl Fn(&Request) -> Response + Send + Sync + 'static) -> Self {
        self.route(Method::Patch, path, h)
    }
    pub fn delete(self, path: &str, h: impl Fn(&Request) -> Response + Send + Sync + 'static) -> Self {
        self.route(Method::Delete, path, h)
    }

    pub fn nest(mut self, prefix: &str, other: Router) -> Self {
        let prefix = parse_pattern(prefix);
        for mut route in other.routes {
            let mut segments = prefix.clone();
            segments.append(&mut route.segments);
            route.segments = segments;
            self.routes.push(route);
        }
        self
    }

    pub fn merge(mut self, other: Router) -> Self {
        self.routes.extend(other.routes);
        self
    }

    pub fn handle(&self, mut req: Request) -> Response {
        let parts: Vec<&str> = req.path.split('/').filter(|s| !s.is_empty()).collect();
        let mut path_matched = false;

        for route in &self.routes {
            if let Some(params) = match_segments(&route.segments, &parts) {
                path_matched = true;
                if route.method == req.method {
                    req.params = params;
                    return (route.handler)(&req);
                }
            }
        }

        if path_matched {
            Response::error(
                StatusCode::METHOD_NOT_ALLOWED,
                format!("Cannot {} {}", req.method, req.path),
            )
        } else {
            Response::error(StatusCode::NOT_FOUND, format!("Cannot {} {}", req.method, req.path))
        }
    }
}

fn parse_pattern(path: &str) -> Vec<Segment> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| match s.strip_prefix(':') {
            Some(name) => Segment::Param(name.to_owned()),
            None => Segment::Static(s.to_owned()),
        })
        .collect()
}

fn match_segments(pattern: &[Segment], parts: &[&str]) -> Option<HashMap<String, String>> {
    if pattern.len() != parts.len() {
        return None;
    }
    let mut params = HashMap::new();
    for (seg, part) in pattern.iter().zip(parts) {
        match seg {
            Segment::Static(s) if s == part => {}
            Segment::Static(_) => return None,
            Segment::Param(name) => {
                params.insert(name.clone(), (*part).to_owned());
            }
        }
    }
    Some(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(method: Method, path: &str) -> Request {
        Request {
            method,
            path: path.into(),
            query: None,
            headers: vec![],
            body: vec![],
            params: HashMap::new(),
        }
    }

    fn router() -> Router {
        let todos = Router::new()
            .get("/", |_| Response::text(StatusCode::OK, "list"))
            .get("/:id", |r| Response::text(StatusCode::OK, r.param("id").unwrap().to_owned()));
        Router::new().nest("/todos", todos)
    }

    #[test]
    fn matches_static_and_params() {
        assert_eq!(router().handle(req(Method::Get, "/todos")).body, b"list");
        assert_eq!(router().handle(req(Method::Get, "/todos/42/")).body, b"42");
    }

    #[test]
    fn not_found_and_method_not_allowed() {
        assert_eq!(router().handle(req(Method::Get, "/nope")).status, StatusCode::NOT_FOUND);
        assert_eq!(
            router().handle(req(Method::Delete, "/todos")).status,
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
}
