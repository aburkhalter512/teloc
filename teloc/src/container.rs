use crate::dependency::DependencyClone;
use crate::get_dependencies::GetDependencies;
use crate::service_provider::SelectContainer;
use crate::{Dependency, Resolver};
use frunk::HNil;
use once_cell::sync::OnceCell;
use std::marker::PhantomData;

/// Trait needed primary to working with `ConvertContainer`. Implement it for your container if you
/// wish that your container can be placed inside of `ConvertContainer`
pub trait ResolveContainer<'a, T, Deps> {
    fn resolve_container<F: Fn() -> Deps>(&'a self, deps: F) -> T;
}

#[derive(Debug)]
pub struct TransientContainer<T>(PhantomData<T>);
impl<T> TransientContainer<T> {
    pub fn new() -> Self {
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
impl<'this, 'cont, T, SP, Index, Deps, Infer>
    Resolver<'this, &'cont TransientContainer<T>, T, (Index, Deps, Infer)> for SP
where
    SP: SelectContainer<'this, &'cont TransientContainer<T>, Index>
        + GetDependencies<'this, Deps, Infer>,
    TransientContainer<T>: ResolveContainer<'cont, T, Deps>,
    T: Dependency<Deps> + 'cont,
{
    fn resolve(&'this self) -> T {
        TransientContainer::resolve_container(self.get(), || self.get_deps())
    }
}

pub struct TransientFactoryContainer<Deps, T, Body>(PhantomData<Deps>, PhantomData<T>, Body);
impl<Deps, T, Body: Fn(Deps) -> T> TransientFactoryContainer<Deps, T, Body> {
    pub fn new(body: Body) -> Self {
        Self(PhantomData, PhantomData, body)
    }
}
impl<'a, T, Deps, Body: Fn(Deps) -> T> ResolveContainer<'a, T, Deps>
    for TransientFactoryContainer<Deps, T, Body>
{
    fn resolve_container<F: Fn() -> Deps>(&'a self, get_deps: F) -> T {
        self.2(get_deps())
    }
}
impl<'this, 'cont, T, SP, Index, Deps, Body, Infer>
    Resolver<'this, &'cont TransientFactoryContainer<Deps, T, Body>, T, (Index, Deps, Infer)> for SP
where
    SP: SelectContainer<'this, &'cont TransientFactoryContainer<Deps, T, Body>, Index>
        + GetDependencies<'this, Deps, Infer>,
    TransientFactoryContainer<Deps, T, Body>: ResolveContainer<'cont, T, Deps>,
    T: 'cont,
    Body: 'cont,
    Deps: 'cont,
{
    fn resolve(&'this self) -> T {
        TransientFactoryContainer::resolve_container(self.get(), || self.get_deps())
    }
}

#[derive(Debug)]
pub struct SingletonContainer<T>(OnceCell<T>);
impl<T> SingletonContainer<T> {
    pub fn new() -> Self {
        Self(OnceCell::new())
    }

    pub fn get(&self) -> &OnceCell<T> {
        &self.0
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

impl<'this, 'cont, T, SP, Index, Deps, Infer>
    Resolver<'this, &'cont SingletonContainer<T>, T, (Index, Deps, Infer)> for SP
where
    SP: GetDependencies<'this, Deps, Infer>
        + SelectContainer<'this, &'cont SingletonContainer<T>, Index>,
    SingletonContainer<T>: ResolveContainer<'cont, &'cont T, Deps>,
    T: Dependency<Deps> + DependencyClone + 'cont,
    Deps: 'cont,
{
    fn resolve(&'this self) -> T {
        SingletonContainer::resolve_container(self.get(), || self.get_deps()).clone()
    }
}
impl<'this, 'cont, T, SP, Index, Deps, Infer>
    Resolver<'this, &'cont SingletonContainer<T>, &'cont T, (Index, Deps, Infer)> for SP
where
    SP: GetDependencies<'this, Deps, Infer>
        + SelectContainer<'this, &'cont SingletonContainer<T>, Index>,
    SingletonContainer<T>: ResolveContainer<'cont, &'cont T, Deps>,
    T: Dependency<Deps> + 'cont,
    Deps: 'cont,
{
    fn resolve(&'this self) -> &'cont T {
        SingletonContainer::resolve_container(self.get(), || self.get_deps())
    }
}

pub struct SingletonFactoryContainer<Deps, T, Body>(PhantomData<Deps>, OnceCell<T>, Body);
impl<Deps, T, Body: Fn(Deps) -> T> SingletonFactoryContainer<Deps, T, Body> {
    pub fn new(body: Body) -> Self {
        Self(PhantomData, OnceCell::new(), body)
    }
}
impl<'a, T, Deps, Body: Fn(Deps) -> T> ResolveContainer<'a, &'a T, Deps>
    for SingletonFactoryContainer<Deps, T, Body>
{
    fn resolve_container<F: Fn() -> Deps>(&'a self, get_deps: F) -> &'a T {
        self.1.get_or_init(|| self.2(get_deps()))
    }
}
impl<'this, 'cont, T, SP, Index, Deps, Body, Infer>
    Resolver<'this, &'cont SingletonFactoryContainer<Deps, T, Body>, T, (Index, Deps, Infer)> for SP
where
    SP: SelectContainer<'this, &'cont SingletonFactoryContainer<Deps, T, Body>, Index>
        + GetDependencies<'this, Deps, Infer>,
    SingletonFactoryContainer<Deps, T, Body>: ResolveContainer<'cont, &'cont T, Deps>,
    T: DependencyClone + 'cont,
    Body: 'cont,
    Deps: 'cont,
{
    fn resolve(&'this self) -> T {
        SingletonFactoryContainer::resolve_container(self.get(), || self.get_deps()).clone()
    }
}

impl<'this, 'cont, T, SP, Index, Deps, Body, Infer>
    Resolver<'this, &'cont SingletonFactoryContainer<Deps, T, Body>, &'cont T, (Index, Deps, Infer)>
    for SP
where
    SP: SelectContainer<'this, &'cont SingletonFactoryContainer<Deps, T, Body>, Index>
        + GetDependencies<'this, Deps, Infer>,
    SingletonFactoryContainer<Deps, T, Body>: ResolveContainer<'cont, &'cont T, Deps>,
    T: 'cont,
    Body: 'cont,
    Deps: 'cont,
{
    fn resolve(&'this self) -> &'cont T {
        SingletonFactoryContainer::resolve_container(self.get(), || self.get_deps())
    }
}

#[derive(Debug)]
pub struct InstanceContainer<T>(T);
impl<T> InstanceContainer<T> {
    pub fn new(instance: T) -> Self {
        Self(instance)
    }

    pub fn get(&self) -> &T {
        &self.0
    }
}
impl<'a, T> ResolveContainer<'a, &'a T, HNil> for InstanceContainer<T> {
    fn resolve_container<F: Fn() -> HNil>(&'a self, _: F) -> &'a T {
        &self.0
    }
}
impl<'this, 'cont, T, SP, Index> Resolver<'this, &'cont InstanceContainer<T>, T, Index> for SP
where
    SP: SelectContainer<'this, &'cont InstanceContainer<T>, Index>,
    InstanceContainer<T>: ResolveContainer<'cont, &'cont T, HNil>,
    T: DependencyClone + 'cont,
{
    fn resolve(&'this self) -> T {
        InstanceContainer::resolve_container(self.get(), || HNil).clone()
    }
}
impl<'this, 'cont, T, SP, Index> Resolver<'this, &'cont InstanceContainer<T>, &'cont T, Index>
    for SP
where
    SP: SelectContainer<'this, &'cont InstanceContainer<T>, Index>,
    InstanceContainer<T>: ResolveContainer<'cont, &'cont T, HNil>,
{
    fn resolve(&'this self) -> &'cont T {
        InstanceContainer::resolve_container(self.get(), || HNil)
    }
}

pub struct ConvertContainer<Cont, T, U>(Cont, PhantomData<(T, U)>);
impl<Cont, T, U> ConvertContainer<Cont, T, U> {
    pub fn new(cont: Cont) -> Self {
        Self(cont, PhantomData)
    }

    pub fn get(&self) -> &Cont {
        &self.0
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
impl<'this, 'cont, Cont, T, U, SP, Index, Deps, Infer>
    Resolver<'this, &'cont ConvertContainer<Cont, T, U>, U, (Index, Deps, Infer)> for SP
where
    SP: SelectContainer<'this, &'cont ConvertContainer<Cont, T, U>, Index>
        + GetDependencies<'this, Deps, Infer>,
    ConvertContainer<Cont, T, U>: ResolveContainer<'cont, U, Deps>,
    Cont: 'cont,
    T: Into<U> + 'cont,
    U: 'cont,
    Deps: 'cont,
{
    fn resolve(&'this self) -> U {
        ConvertContainer::resolve_container(self.get(), || self.get_deps())
    }
}
