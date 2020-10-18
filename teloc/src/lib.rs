use std::rc::Rc;
pub use teloc_macros::{container, Teloc};

pub trait Get<T> {
    fn get(&mut self) -> T;
}
pub trait GetRef<T> {
    fn get_ref(&self) -> &T;
}
pub trait GetClone<T: Clone> {
    fn get_clone(&self) -> T;
}

impl<C, T> GetRef<T> for ContainerWrapper<C>
where
    ContainerWrapper<C>: GetRef<Rc<T>>,
{
    fn get_ref(&self) -> &T {
        GetRef::<Rc<T>>::get_ref(self).as_ref()
    }
}

impl<C, T> GetClone<T> for ContainerWrapper<C>
where
    T: Clone,
    ContainerWrapper<C>: GetRef<T>,
{
    fn get_clone(&self) -> T {
        self.get_ref().clone()
    }
}
impl<C, T> GetClone<T> for ContainerWrapper<C>
where
    ContainerWrapper<C>: GetRef<Rc<T>>,
{
    fn get_clone(&self) -> T {
        self.get_ref().clone()
    }
}

pub struct ContainerWrapper<T>(pub T);
