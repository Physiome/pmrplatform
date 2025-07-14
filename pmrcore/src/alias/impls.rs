use std::{
    ops::{Deref, DerefMut},
    marker::PhantomData,
};
use crate::{
    alias::{
        Alias,
        Aliases,
        AliasEntries,
        AliasEntry,
        AliasRef,
        AliasRefs,
        traits,
    },
};

impl Alias {
    pub(crate) fn bind<'a, T>(
        self,
        aliased: T,
    ) -> AliasRef<'a, T> {
        AliasRef {
            inner: self,
            aliased,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> traits::Alias<'a, T> for Alias {
    fn kind(&self) -> &str {
        self.kind.as_str()
    }
    fn kind_id(&self) -> i64 {
        self.kind_id
    }
    fn alias(&self) -> &str {
        self.alias.as_str()
    }
    fn created_ts(&self) -> i64 {
        self.created_ts
    }
    fn aliased(&'a self) -> Option<&'a T> {
        None
    }
}

impl<'a, T> traits::Alias<'a, T> for AliasRef<'a, T> {
    fn kind(&self) -> &str {
        self.inner.kind.as_str()
    }
    fn kind_id(&self) -> i64 {
        self.inner.kind_id
    }
    fn alias(&self) -> &str {
        self.inner.alias.as_str()
    }
    fn created_ts(&self) -> i64 {
        self.inner.created_ts
    }
    fn aliased(&'a self) -> Option<&'a T> {
        Some(&self.aliased)
    }
}

impl From<Vec<Alias>> for Aliases {
    fn from(args: Vec<Alias>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[Alias; N]> for Aliases {
    fn from(args: [Alias; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for Aliases {
    type Target = Vec<Alias>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Aliases {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Aliases {
    type Item = Alias;
    type IntoIter = std::vec::IntoIter<Alias>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> From<Vec<AliasRef<'a, T>>> for AliasRefs<'a, T> {
    fn from(args: Vec<AliasRef<'a, T>>) -> Self {
        Self(args)
    }
}

impl<'a, T, const N: usize> From<[AliasRef<'a, T>; N]> for AliasRefs<'a, T> {
    fn from(args: [AliasRef<'a, T>; N]) -> Self {
        Self(args.into())
    }
}

impl<'a, T> Deref for AliasRefs<'a, T> {
    type Target = Vec<AliasRef<'a, T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> IntoIterator for AliasRefs<'a, T> {
    type Item = AliasRef<'a, T>;
    type IntoIter = std::vec::IntoIter<AliasRef<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> AliasEntries<T> {
    pub fn kind(&self) -> &str {
        self.kind.as_str()
    }
}

impl<'a, T> Deref for AliasEntries<T> {
    type Target = Vec<AliasEntry<T>>;

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl<T> IntoIterator for AliasEntries<T> {
    type Item = AliasEntry<T>;
    type IntoIter = std::vec::IntoIter<AliasEntry<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<T> AliasEntry<T> {
    pub fn map<U, F>(self, f: F) -> AliasEntry<U>
    where
        F: FnOnce(T) -> U,
    {
        AliasEntry {
            alias: self.alias,
            entity: f(self.entity),
        }
    }
}
