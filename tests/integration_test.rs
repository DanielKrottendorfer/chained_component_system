use chained_component_system::chained_component_system;

use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug, Default, Clone)]
pub struct Foo(&'static str);
#[derive(Debug, Default, Clone)]
pub struct Goo(u32);
#[derive(Debug, Default, Clone)]
pub struct Hoo(f32);

chained_component_system!(
    components{
        foo: Foo,
        goo: Goo,
        hoo: Hoo,
        loo: Foo
    };

    entities{
        Peon(foo, goo),
        Tree(foo, goo, loo),
        Mage(loo, goo, hoo)
    };

    global_systems{
        FooGooSystem(foo, goo),
        FooGooLooSystem(foo, goo, loo),
        GooLooSystem(goo, loo),
        GooSystem(goo),
        FooLooSystem(foo,loo),
    };
);

#[test]
fn test_add() {
    let mut ecs = CHAINED_ECS::default();

    ecs.peon_soa.new_peon_soa(Foo("Peon"), Goo(11));
    ecs.tree_soa
        .new_tree_soa(Foo("Tree"), Goo(22), Foo("Loo Tree"));
    ecs.mage_soa
        .new_mage_soa(Foo("Loo Mage"), Goo(33), Hoo(0.0));

    let mut a = ecs.get_foo_goo_system_accessor();
    let mut b = ecs.get_goo_loo_system_accessor();
    let mut c = ecs.get_goo_system_accessor();
    let mut d = ecs.get_foo_goo_loo_system_accessor();
    let mut e = ecs.get_foo_loo_system_accessor();

    for i in a.iter() {
        println!("foo goo     ____{:?}", i);
    }
    for i in b.iter() {
        println!("    goo loo ____{:?}", i);
    }
    for i in c.iter() {
        println!("    goo     ____{:?}", i);
    }
    for i in d.iter() {
        println!("foo goo loo ____{:?}", i);
    }
    for i in e.iter() {
        println!("foo     loo ____{:?}", i);
    }
}
