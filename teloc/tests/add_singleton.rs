use frunk::{HCons, HNil};
use teloc::{Dependency, DependencyClone, Resolver, ServiceProvider};

struct ConstService {
    data: i32,
    data2: u8,
}
impl ConstService {
    pub fn init(data: i32, data2: u8) -> Self {
        ConstService { data, data2 }
    }
}

#[derive(Dependency)]
struct ControllerA<'a> {
    service: &'a ConstService,
}

#[derive(Dependency, Clone)]
struct ControllerWithDependencyClone<'a> {
    service: &'a ConstService,
}

impl<'a> DependencyClone for ControllerWithDependencyClone<'a> {}

#[test]
fn test_resolve_singleton_by_reference() {
    let sp = ServiceProvider::new()
        .add_singleton::<ControllerA>()
        .add_instance(ConstService {
            data: 10i32,
            data2: 8u8,
        });

    let controllerA: &ControllerA = sp.resolve();
    assert_eq!(controllerA.service.data, 10i32);
}

#[test]
fn test_resolve_singleton_by_dependency_clone() {
    let sp = ServiceProvider::new()
        .add_singleton::<ControllerWithDependencyClone>()
        .add_instance(ConstService {
            data: 10i32,
            data2: 8u8,
        });

    let controller: ControllerWithDependencyClone = sp.resolve();
    assert_eq!(controller.service.data, 10i32);
}

#[test]
fn test_resolve_singleton_factory_by_reference() {
    let sp = ServiceProvider::new()
        .add_singleton_factory(|deps: HCons<&ConstService, HNil>| {
            let (service, _rest) = deps.pluck();
            ControllerA { service }
        })
        .add_instance(ConstService {
            data: 10i32,
            data2: 8u8,
        });

    let controllerA: &ControllerA = sp.resolve();
    assert_eq!(controllerA.service.data, 10i32);
}

#[test]
fn test_resolve_singleton_factory_by_dependency_clone() {
    let sp = ServiceProvider::new()
        .add_singleton_factory(|deps: HCons<&ConstService, HNil>| {
            let (service, _rest) = deps.pluck();
            ControllerWithDependencyClone { service }
        })
        .add_instance(ConstService {
            data: 10i32,
            data2: 8u8,
        });

    let controller: ControllerWithDependencyClone = sp.resolve();
    assert_eq!(controller.service.data, 10i32);
}
