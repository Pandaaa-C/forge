use std::sync::Arc;

use forge_core::{App, Module};
use forge_di::Container;
use forge_http::{Response, Router, StatusCode};
use forge_json::{Json, ToJson};
use forge_macros::injectable;

#[derive(Clone)]
struct Todo {
    id: u64,
    user_id: u64,
    title: String,
    done: bool,
}

impl ToJson for Todo {
    fn to_json(&self) -> Json {
        Json::object([
            ("id", self.id.to_json()),
            ("userId", self.user_id.to_json()),
            ("title", self.title.to_json()),
            ("done", self.done.to_json()),
        ])
    }
}

#[injectable]
struct TodoService {
    todos: Vec<Todo>,
}

impl TodoService {
    fn new() -> Self {
        Self {
            todos: vec![
                Todo {
                    id: 1,
                    user_id: 1,
                    title: "Build forge".into(),
                    done: false,
                },
                Todo {
                    id: 2,
                    user_id: 1,
                    title: "Write the HttpError type".into(),
                    done: false,
                },
            ],
        }
    }

    fn all(&self) -> &[Todo] {
        &self.todos
    }

    fn find(&self, id: u64) -> Option<&Todo> {
        self.todos.iter().find(|t| t.id == id)
    }

    fn for_user(&self, user_id: u64) -> Vec<Todo> {
        self.todos
            .iter()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect()
    }
}

struct TodoModule;

impl Module for TodoModule {
    fn providers(&self, c: &mut Container) {
        c.register(TodoService::new());
    }

    fn routes(&self, router: Router, c: &Container) -> Router {
        let list_svc: Arc<TodoService> = c.resolve();
        let find_svc = Arc::clone(&list_svc);

        let controller = Router::new()
            .get("/", move |_req| {
                Response::json(StatusCode::OK, list_svc.all())
            })
            .get("/:id", move |req| {
                let todo = req
                    .param("id")
                    .and_then(|id| id.parse::<u64>().ok())
                    .and_then(|id| find_svc.find(id));
                match todo {
                    Some(todo) => Response::json(StatusCode::OK, todo),
                    None => Response::error(StatusCode::NOT_FOUND, "Todo not found"),
                }
            });

        router.nest("/todos", controller)
    }
}

#[derive(Clone)]
struct User {
    id: u64,
    name: String,
}

impl ToJson for User {
    fn to_json(&self) -> Json {
        Json::object([("id", self.id.to_json()), ("name", self.name.to_json())])
    }
}

#[injectable]
struct UserService {
    users: Vec<User>,
    todos: Arc<TodoService>,
}

impl UserService {
    fn new(todos: Arc<TodoService>) -> Self {
        Self {
            users: vec![User {
                id: 1,
                name: "John Doe".to_string(),
            }],
            todos,
        }
    }

    fn all(&self) -> &[User] {
        &self.users
    }

    fn find(&self, id: u64) -> Option<&User> {
        self.users.iter().find(|user| user.id == id)
    }

    fn todos_of(&self, id: u64) -> Option<Vec<Todo>> {
        self.find(id).map(|user| self.todos.for_user(user.id))
    }
}

struct UserModule;

impl Module for UserModule {
    fn imports(&self) -> Vec<Box<dyn Module>> {
        vec![Box::new(TodoModule)]
    }

    fn providers(&self, c: &mut Container) {
        let todos = c.resolve::<TodoService>();
        c.register(UserService::new(todos));
    }

    fn routes(&self, router: Router, c: &Container) -> Router {
        let list_svc: Arc<UserService> = c.resolve();
        let find_svc = Arc::clone(&list_svc);
        let todos_svc = Arc::clone(&list_svc);

        let controller = Router::new()
            .get("/", move |_req| {
                Response::json(StatusCode::OK, list_svc.all())
            })
            .get("/:id", move |req| {
                let user = req
                    .param("id")
                    .and_then(|id| id.parse::<u64>().ok())
                    .and_then(|id| find_svc.find(id));
                match user {
                    Some(user) => Response::json(StatusCode::OK, user),
                    None => Response::error(StatusCode::NOT_FOUND, "User not found"),
                }
            })
            .get("/:id/todos", move |req| {
                let todos = req
                    .param("id")
                    .and_then(|id| id.parse::<u64>().ok())
                    .and_then(|id| todos_svc.todos_of(id));
                match todos {
                    Some(todos) => Response::json(StatusCode::OK, &todos),
                    None => Response::error(StatusCode::NOT_FOUND, "User not found"),
                }
            });

        router.nest("/users", controller)
    }
}

fn main() -> std::io::Result<()> {
    let port = forge_config::env_or("PORT", 3000u16);
    App::new().module(UserModule).listen(port)
}