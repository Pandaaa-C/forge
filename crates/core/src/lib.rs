use std::any::TypeId;
use std::collections::HashSet;
use std::io::empty;
use forge_di::Container;
use forge_http::Router;

pub trait Module: 'static {
    fn imports(&self) -> Vec<Box<dyn Module>> {
        Vec::new()
    }

    fn providers(&self, _container: &mut Container) {}

    fn routes(&self, router: Router, _container: &Container) -> Router {
        router
    }

    fn id(&self) ->  TypeId{
        TypeId::of::<Self>()
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub struct App {
    container: Container,
    router: Router,
    registered: HashSet<TypeId>,
    stack: Vec<(TypeId, &'static str)>,
}

impl App {
    pub fn new() -> Self {
        Self {
            container: Container::new(),
            router: Router::new(),
            registered: HashSet::new(),
            stack: Vec::new(),
        }
    }

    pub fn module(mut self, module: impl Module) -> Self {
        self.register(&module);
        self
    }

    fn register(&mut self, module: &dyn Module) {
        let id = module.id();

        if self.registered.contains(&id) {
            return;
        }

        if self.stack.iter().any(|(seen, _)| *seen == id) {
            let chain: Vec<&str> = self.stack.iter().map(|(_, name)| *name).collect();
            panic!("forge: circular import: {} -> {}", chain.join(" -> "), module.name());
        }

        self.stack.push((id, module.name()));

        for imports in module.imports() {
            self.register(imports.as_ref());
        }

        self.stack.pop();

        module.providers(&mut self.container);
        self.router = module.routes(std::mem::take(&mut self.router), &self.container);
        self.registered.insert(id);
    }

    pub fn listen(self, port: u16) -> std::io::Result<()> {
        forge_http::serve(self.router, port)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
