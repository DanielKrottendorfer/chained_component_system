use cgmath::Array;
use chained_component_system::chained_component_system;

#[derive(Debug, Default)]
pub struct Foo(&'static str);
#[derive(Debug, Default)]
pub struct Goo(u32);
#[derive(Debug, Default)]
pub struct Hoo(f32);

use std::borrow::BorrowMut;
use std::iter::Chain;

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

    for i in ecs.get_foo_goo_system_chunk_iterator() {
        println!("foo goo     ____{:?}", i);
    }
    for i in ecs.get_goo_loo_system_chunk_iterator() {
        println!("    goo loo ____{:?}", i);
    }
    for i in ecs.get_goo_system_chunk_iterator() {
        println!("    goo     ____{:?}", i);
    }
    for i in ecs.get_foo_goo_loo_system_chunk_iterator() {
        println!("foo goo loo ____{:?}", i);
    }
    for i in ecs.get_foo_loo_system_chunk_iterator() {
        println!("foo     loo ____{:?}", i);
    }
}

use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

pub struct FooSOA {
    a0: Mutex<Vec<Option<f64>>>,
    a1: Mutex<Vec<i64>>,
    a2: Mutex<Vec<u64>>,
}

pub struct ChunkIterator<'a> {
    a0: MutexGuard<'a, Vec<Option<f64>>>,
    a1: MutexGuard<'a, Vec<i64>>,
    a2: MutexGuard<'a, Vec<u64>>,
    index: usize,
}

impl<'a> IntoIterator for &'a FooSOA {
    type Item = (&'a mut f64, &'a mut i64, &'a mut u64);

    type IntoIter = ChunkIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ChunkIterator {
            index: 0,
            a0: self.a0.lock().unwrap(),
            a1: self.a1.lock().unwrap(),
            a2: self.a2.lock().unwrap(),
        }
    }
}

pub fn unwrap_try_lock<T, U>(tl: Result<MutexGuard<T>, U>) -> Option<MutexGuard<T>> {
    match tl {
        Ok(t) => Some(t),
        Err(_) => None,
    }
}

impl FooSOA {
    pub fn try_into_iter(&self) -> Option<ChunkIterator> {
        Some(ChunkIterator {
            index: 0,
            a0: unwrap_try_lock(self.a0.try_lock())?,
            a1: unwrap_try_lock(self.a1.try_lock())?,
            a2: unwrap_try_lock(self.a2.try_lock())?,
        })
    }
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = (&'a mut f64, &'a mut i64, &'a mut u64);

    fn next<'b>(&mut self) -> Option<Self::Item> {
        let t = if self.a0.len() > self.index {
            let a = self.a0.as_mut_ptr();
            let b = self.a1.as_mut_ptr();
            let c = self.a2.as_mut_ptr();
            let z ={
                unsafe {
                    Some((
                        (*a.add(self.index)).as_mut()?,
                        &mut *b.add(self.index),
                        &mut *c.add(self.index),
                    ))
                }
            };
            z
        } else {
            None
        };
        self.index += 1;
        t
    }
}

#[test]
fn chunk_test() {
    let a0: Mutex<Vec<Option<f64>>> =
        Mutex::new(vec![0.0; 5].iter().map(|x| Some(x.clone())).collect());
    let a1 = Mutex::new(vec![1; 5]);
    let a2 = Mutex::new(vec![2; 5]);

    let c = Arc::new(FooSOA { a0, a1, a2 });

    let a0: Mutex<Vec<Option<f64>>> =
        Mutex::new(vec![10.0; 5].iter().map(|x| Some(x.clone())).collect());
    let a1 = Mutex::new(vec![11; 5]);
    let a2 = Mutex::new(vec![12; 5]);

    let d = Arc::new(FooSOA { a0, a1, a2 });

    let ct = c.clone();
    let ct2 = d.clone();

    let t1 = thread::spawn(move || {
        if let Some(ct) = ct.try_into_iter() {
            if let Some(ct2) = ct2.try_into_iter() {
                for i in ct.into_iter().chain(ct2.into_iter()) {
                    println!("{:?}", i);
                }
            } else {
                println!("failed to lock ctt2");
            }
        } else {
            println!("failed to lock ctt");
        }
    });

    let ctt = c.clone();
    let ctt2 = d.clone();

    let t2 = thread::spawn(move || {
        if let Some(ct) = ctt.try_into_iter() {
            if let Some(ct2) = ctt2.try_into_iter() {
                for i in ct.into_iter().chain(ct2.into_iter()) {
                    println!("{:?}", i);
                }
            } else {
                println!("failed to lock ctt2");
            }
        } else {
            println!("failed to lock ctt");
        }
    });

    t1.join();
    t2.join();
}
