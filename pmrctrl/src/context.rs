use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

#[derive(Clone, Default)]
pub struct Context {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Context {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert<T>(&mut self, val: T) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.map
            .insert(TypeId::of::<T>(), Arc::new(val))
            .and_then(|v| (v as Arc<dyn Any + Send + Sync>).downcast().ok())
    }

    pub fn get<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|v| (v.clone() as Arc<dyn Any + Send + Sync>).downcast().ok())
    }

    pub fn clear(&mut self) {
        self.map.clear()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        #[derive(Debug, PartialEq)]
        struct Source(String);

        #[derive(Debug, PartialEq)]
        struct Dest(String);

        let mut ctx = Context::new();
        ctx.insert(Source(String::from("src")));
        ctx.insert(Dest(String::from("dst")));

        assert_eq!(ctx.get(), Some(Arc::new(Source(String::from("src")))));
        assert_eq!(ctx.get(), Some(Arc::new(Dest(String::from("dst")))));

        let mut ctx2 = ctx.clone();

        assert_eq!(ctx.get(), Some(Arc::new(Source(String::from("src")))));
        assert_eq!(ctx2.get(), Some(Arc::new(Source(String::from("src")))));
        assert_eq!(ctx.get(), Some(Arc::new(Dest(String::from("dst")))));
        assert_eq!(ctx2.get(), Some(Arc::new(Dest(String::from("dst")))));

        ctx.insert(Source(String::from("full source")));
        ctx2.insert(Dest(String::from("full dest")));

        assert_eq!(ctx.get(), Some(Arc::new(Source(String::from("full source")))));
        assert_eq!(ctx2.get(), Some(Arc::new(Source(String::from("src")))));
        assert_eq!(ctx.get(), Some(Arc::new(Dest(String::from("dst")))));
        assert_eq!(ctx2.get(), Some(Arc::new(Dest(String::from("full dest")))));

        ctx.clear();
        assert_eq!(ctx.get::<Source>(), None);
        assert_eq!(ctx.get::<Dest>(), None);
        assert_eq!(ctx2.get(), Some(Arc::new(Source(String::from("src")))));
        assert_eq!(ctx2.get(), Some(Arc::new(Dest(String::from("full dest")))));
    }
}
