use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct Container {
    services: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Container {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T: Send + Sync + 'static>(&mut self, value: T) {
        self.services.insert(TypeId::of::<T>(), Arc::new(value));
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.services
            .get(&TypeId::of::<T>())
            .cloned()
            .and_then(|s| s.downcast::<T>().ok())
    }

    pub fn resolve<T: Send + Sync + 'static>(&self) -> Arc<T> {
        self.get::<T>().unwrap_or_else(|| {
            panic!("forge-di: no provider registered for {}", std::any::type_name::<T>())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Greeter(&'static str);

    #[test]
    fn register_and_resolve() {
        let mut c = Container::new();
        c.register(Greeter("hi"));
        assert_eq!(c.resolve::<Greeter>().0, "hi");
        assert!(c.get::<String>().is_none());
    }
}
