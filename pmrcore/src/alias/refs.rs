use std::marker::PhantomData;
use crate::alias::Alias;

pub struct AliasRef<'a, T> {
    pub(super) inner: Alias,
    pub(super) aliased: T,
    pub(super) phantom: PhantomData<&'a T>
}

pub struct AliasRefs<'a, T>(pub(super) Vec<AliasRef<'a, T>>);
