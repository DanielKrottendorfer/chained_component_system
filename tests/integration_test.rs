use chained_component_system::chained_component_system;

#[derive(Debug, Default)]
pub struct Foo;
#[derive(Debug, Default)]
pub struct Goo;
#[derive(Debug, Default)]
pub struct Hoo;

use std::{
    sync::{Arc, Mutex},
};

chained_component_system!(
    components{
        foo: Foo,
        goo: Goo,
        hoo: Hoo,
        loo: Foo
    }

    entitys{
        Peon(foo, goo, hoo),
        Tree(foo, goo, loo)
    }

    global_systems{
        update_foo(foo,goo),
        update_goo(goo,hoo)
    }
);

#[test]
fn test_add() {
    let mut ecs = ECS::default();

    ecs.peon_soa.foo.push(Foo);
    ecs.peon_soa.goo.push(Goo);

    ecs.tree_soa.foo.push(Foo);
    ecs.tree_soa.goo.push(Goo);

    for a in ecs.update_foo() {
        println!("_____{:?}", a)
    }
}

#[test]
fn test_chain_zip() {
    let a0 = Arc::new(Mutex::new([0.0; 2]));
    let b0 = Arc::new(Mutex::new([0; 2]));
    let a1 = Arc::new(Mutex::new([1.0; 2]));
    let b1 = Arc::new(Mutex::new([1; 2]));

    let la0 = a0.lock();
    let la1 = a1.lock();

    let lb0 = b0.lock();
    let lb1 = b1.lock();

    let i = la0.iter().chain(la1.iter());
    let j = lb0.iter().chain(lb1.iter());
    let qw = i.zip(j);

    for k in qw {
        println!("{:?}", k);
    }
}
