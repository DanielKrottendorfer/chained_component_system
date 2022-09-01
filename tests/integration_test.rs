use chained_component_system::chained_component_system;

use std::{sync::*, thread, time::Duration};

pub mod structs;

use structs::*;

chained_component_system!(
    components{
        foo: Foo,
        goo: Goo,
        hoo: Hoo,
        loo: Foo,
        v: (usize,String)
    };

    entities{
        Peon(foo, goo),
        NoCont(foo,goo,hoo,loo),
        Tree(loo, goo, foo),
        Mage(loo, goo, hoo),
        Ve(foo,v)
    };

    global_systems{ 
        FooSystem(foo, KEY),
        GooSystem(goo),
        LooSystem(loo),
        GooLooSystem(goo, mut loo),
        FooLooSystem(foo, loo),
        FooGooSystem(foo, goo, KEY),
        FooGooLooSystem(foo, goo, loo, KEY),
    };
);

#[test]
fn test_add() {
    let mut ecs = CHAINED_ECS::new();

    let a = ecs.get_foo_system_accessor();
    let mut b = ecs.get_goo_loo_system_accessor();
    let c = ecs.get_goo_system_accessor();
    let d = ecs.get_foo_goo_loo_system_accessor();
    let d2 = ecs.get_foo_goo_loo_system_accessor();
    let e = ecs.get_foo_loo_system_accessor();

    ecs.add_peon_soa(Foo("Foo Peons"), Goo(11));
    ecs.add_peon_soa(Foo("Foo Peons2"), Goo(22));
    ecs.add_tree_soa(Foo("Loo Tree"), Goo(23), Foo("Foo Tree"));
    ecs.add_mage_soa(Foo("Loo Mage"), Goo(33), Hoo(0.0));

    let ta = thread::spawn(move || {
        let mut foo_key = Vec::new();
        for i in a.lock().iter() {
            thread::sleep(Duration::from_millis(100));
            println!("1 foo         ____{:?}", i);

            foo_key.push(i.1);
        }
        foo_key
    });

    let tb = thread::spawn(move || {
        for i in b.lock().iter() {
            thread::sleep(Duration::from_millis(100));
            println!("2     goo loo ____{:?}", i);
        }
    });

    let te = thread::spawn(move || {
        for i in e.lock().iter() {
            thread::sleep(Duration::from_millis(100));
            println!("3 foo     loo ____{:?}", i);
        }
    });

    let tc = thread::spawn(move || {
        for i in c.lock().iter() {
            thread::sleep(Duration::from_millis(100));
            println!("4     goo     ____{:?}", i);
        }
    });

    let td = thread::spawn(move || {
        let d_lock = d.lock();
        for i in d_lock.iter() {
            thread::sleep(Duration::from_millis(100));
            println!("5 foo goo loo ____{:?}", i);
        }
    });

    let keys = ta.join().unwrap();
    tb.join().unwrap();
    tc.join().unwrap();
    td.join().unwrap();
    te.join().unwrap();

    let d_lock = d2.lock();

    for k in keys {
        println!("{:?}", k);
        let a = d_lock.get(k);
        println!("{:?}", a);
    }
}

#[test]
fn static_t() {
    let mut ecs = CHAINED_ECS::new();
    ecs.add_peon_soa(Foo("Foo 1"), Goo(1));
    ecs.add_peon_soa(Foo("Foo 2"), Goo(2));
    ecs.add_peon_soa(Foo("Foo 3"), Goo(3));
    ecs.add_peon_soa(Foo("Foo 4"), Goo(4));

    let mut peon = ecs.get_peon();

    let to_delete: Vec<Key> = peon
        .lock()
        .iter()
        .filter_map(|x| {
            if x.1 .0 % 2 == 0 {
                Some(x.2.clone())
            } else {
                None
            }
        })
        .collect();

    for d in to_delete.iter() {
        if ecs.delete(d).is_none() {
            panic!("delete did not work");
        };
    }

    ecs.add_peon_soa(Foo("Foo 13"), Goo(13));
    ecs.add_peon_soa(Foo("Foo 14"), Goo(14));
    ecs.add_peon_soa(Foo("Foo 15"), Goo(15));

    let mut out = String::new();
    for peon in peon.lock().iter() {
        out = format!("{} {}", out, peon.1 .0);
    }

    assert_eq!(out, " 1 14 3 13 15");
}

#[test]
fn static_t2() {
    let mut ecs = CHAINED_ECS::new();
    ecs.add_peon_soa(Foo("Foo 13"), Goo(13));
    ecs.add_peon_soa(Foo("Foo 14"), Goo(14));
    ecs.add_peon_soa(Foo("Foo 15"), Goo(15));

    let b = ecs.get_foo_goo_system_accessor();
    let a = b.lock();
    for i in a.iter() {
        for y in a.iter() {
            println!("{:?} {:?}", i, y);
        }
    }
}
