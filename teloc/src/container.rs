use crate::dependency::DependencyClone;
use crate::get_dependencies::GetDependencies;
use crate::{Dependency, Resolver};
use frunk::hlist::Selector;
use frunk::HNil;
use once_cell::sync::OnceCell;
use std::marker::PhantomData;

/// Init is a trait used in [`ServiceProvider`] for create an empty version of `Container`. If you
/// create your own version of container and you want that it can work with other container like
/// [`ConvertContainer`], you must implement this trait.
///
/// [`ServiceProvider`]: teloc::ServiceProvider
/// [`ConvertContainer`]: teloc::container::ConvertContainer
pub trait Container {
    type Data;
    fn init(data: Self::Data) -> Self;
}

/// Trait needed primary to working with `ConvertContainer`. Implement it for your container if you
/// wish that your container can be placed inside of `ConvertContainer`
pub trait ResolveContainer<'a, T, Deps> {
    fn resolve_container<F: Fn() -> Deps>(&'a self, deps: F) -> T;
}

#[derive(Debug)]
pub struct TransientContainer<T>(PhantomData<T>);
impl<T> Container for TransientContainer<T> {
    type Data = ();

    fn init(_: ()) -> Self {
        Self(PhantomData)
    }
}
impl<'a, T, Deps> ResolveContainer<'a, T, Deps> for TransientContainer<T>
where
    T: Dependency<Deps>,
{
    fn resolve_container<F: Fn() -> Deps>(&'a self, get_deps: F) -> T {
        T::init(get_deps())
    }
}
impl<'a, T, SP, Index, Deps, Infer> Resolver<'a, TransientContainer<T>, T, (Index, Deps, Infer)>
    for SP
where
    SP: Selector<TransientContainer<T>, Index> + GetDependencies<'a, Deps, Infer>,
    T: Dependency<Deps> + 'a,
    TransientContainer<T>: ResolveContainer<'a, T, Deps>,
{
    fn resolve(&'a self) -> T {
        TransientContainer::resolve_container(self.get(), || self.get_deps())
    }
}

#[derive(Debug)]
pub struct SingletonContainer<T>(OnceCell<T>);
impl<T> Container for SingletonContainer<T> {
    type Data = ();

    fn init(_: ()) -> Self {
        Self(OnceCell::new())
    }
}
impl<'a, T, Deps> ResolveContainer<'a, &'a T, Deps> for SingletonContainer<T>
where
    T: Dependency<Deps> + 'a,
{
    fn resolve_container<F: Fn() -> Deps>(&'a self, get_deps: F) -> &'a T {
        self.get().get_or_init(|| T::init(get_deps()))
    }
}

impl<'a, T, SP, Index, Deps, Infer> Resolver<'a, SingletonContainer<T>, T, (Index, Deps, Infer)>
    for SP
where
    SingletonContainer<T>: ResolveContainer<'a, &'a T, Deps>,
    T: Dependency<Deps> + DependencyClone + 'a,
    Deps: 'a,
    SP: GetDependencies<'a, Deps, Infer> + Selector<SingletonContainer<T>, Index>,
{
    fn resolve(&'a self) -> T {
        SingletonContainer::resolve_container(self.get(), || self.get_deps()).clone()
    }
}
impl<'a, T, SP, Index, Deps, Infer> Resolver<'a, SingletonContainer<T>, &'a T, (Index, Deps, Infer)>
    for SP
where
    SingletonContainer<T>: ResolveContainer<'a, &'a T, Deps>,
    T: Dependency<Deps> + 'a,
    Deps: 'a,
    SP: GetDependencies<'a, Deps, Infer> + Selector<SingletonContainer<T>, Index>,
{
    fn resolve(&'a self) -> &'a T {
        SingletonContainer::resolve_container(self.get(), || self.get_deps())
    }
}
impl<T> SingletonContainer<T> {
    #[inline]
    pub fn get(&self) -> &OnceCell<T> {
        &self.0
    }
}

#[derive(Debug)]
pub struct InstanceContainer<T>(T);
impl<T> Container for InstanceContainer<T> {
    type Data = T;

    fn init(instance: T) -> Self {
        Self(instance)
    }
}
impl<'a, T> ResolveContainer<'a, &'a T, HNil> for InstanceContainer<T> {
    fn resolve_container<F: Fn() -> HNil>(&'a self, _: F) -> &'a T {
        &self.0
    }
}
impl<'a, T, SP, Index> Resolver<'a, InstanceContainer<T>, T, Index> for SP
where
    T: DependencyClone + 'a,
    SP: Selector<InstanceContainer<T>, Index>,
    InstanceContainer<T>: ResolveContainer<'a, &'a T, HNil>,
{
    fn resolve(&'a self) -> T {
        InstanceContainer::resolve_container(self.get(), || HNil).clone()
    }
}
impl<'a, T, SP, Index> Resolver<'a, InstanceContainer<T>, &'a T, Index> for SP
where
    SP: Selector<InstanceContainer<T>, Index>,
    InstanceContainer<T>: ResolveContainer<'a, &'a T, HNil>,
{
    fn resolve(&'a self) -> &'a T {
        InstanceContainer::resolve_container(self.get(), || HNil)
    }
}
impl<T> InstanceContainer<T> {
    #[inline]
    pub fn get(&self) -> &T {
        &self.0
    }
}

pub struct ConvertContainer<Cont, T, U>(Cont, PhantomData<(T, U)>);
impl<Cont, T, U> Container for ConvertContainer<Cont, T, U>
where
    Cont: Container,
{
    type Data = Cont::Data;

    fn init(data: Self::Data) -> Self {
        Self(Cont::init(data), PhantomData)
    }
}
impl<'a, Cont, T, U, Deps> ResolveContainer<'a, U, Deps> for ConvertContainer<Cont, T, U>
where
    Cont: ResolveContainer<'a, T, Deps>,
    T: Into<U>,
{
    fn resolve_container<F: Fn() -> Deps>(&'a self, deps: F) -> U {
        Cont::resolve_container(&self.0, deps).into()
    }
}
impl<'a, Cont, T, U, SP, Index, Deps, Infer>
    Resolver<'a, ConvertContainer<Cont, T, U>, U, (Index, Deps, Infer)> for SP
where
    U: 'a,
    Deps: 'a,
    Cont: 'a,
    T: Into<U> + 'a,
    ConvertContainer<Cont, T, U>: ResolveContainer<'a, U, Deps>,
    SP: Selector<ConvertContainer<Cont, T, U>, Index> + GetDependencies<'a, Deps, Infer>,
{
    fn resolve(&'a self) -> U {
        ConvertContainer::resolve_container(self.get(), || self.get_deps())
    }
}
impl<Cont, ContT, T> ConvertContainer<Cont, ContT, T> {
    #[inline]
    pub fn get(&self) -> &Cont {
        &self.0
    }
}
