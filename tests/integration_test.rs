use chained_component_system::chained_component_system;

#[derive(Debug, Default)]
pub struct Foo(&'static str);
#[derive(Debug, Default)]
pub struct Goo(u32);
#[derive(Debug, Default)]
pub struct Hoo(f32);

use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};


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
        Mage(foo, goo, hoo)
    };

    global_systems{
        FooSystem(foo, goo),
        GooSystem(foo, goo, loo),
    };
);

#[test]
fn test_add() {
    let mut ecs = CHAINED_ECS::default();

    println!("{:?}", ecs);

    ecs.peon_soa.new_peon_soa(Foo("Peon"), Goo(11));
    ecs.tree_soa.new_tree_soa(Foo("Tree"), Goo(22), Foo("Loo1"));
    ecs.mage_soa.new_mage_soa(Foo("Mage"), Goo(11), Hoo(2.0));

    for i in ecs.get_foo_system_chunk_iterator() {
        println!("get foo ____{:?}", i);
    }
    for i in ecs.get_goo_system_chunk_iterator() {
        println!("get goo ____{:?}", i);
    }
}

mod chunk {

    pub struct Chunk {
        a0: Vec<f64>,
        a1: Vec<f64>,
        a2: Vec<f64>,
    }

    pub struct ChunkIterator<'a> {
        a0: &'a mut Vec<f64>,
        a1: &'a mut Vec<f64>,
        a2: &'a mut Vec<f64>,
        index: usize,
    }

    impl Chunk {
        pub fn get_chunk_iter(&mut self) -> ChunkIterator<'_> {
            ChunkIterator {
                index: 0,
                a0: &mut self.a0,
                a1: &mut self.a1,
                a2: &mut self.a2,
            }
        }
    }

    impl<'a> IntoIterator for &'a mut Chunk {
        type Item = (&'a mut f64, &'a mut f64, &'a mut f64);

        type IntoIter = ChunkIterator<'a>;

        fn into_iter(self) -> Self::IntoIter {
            ChunkIterator {
                index: 0,
                a0: &mut self.a0,
                a1: &mut self.a1,
                a2: &mut self.a2,
            }
        }
    }

    impl<'a> Iterator for ChunkIterator<'a> {
        type Item = (&'a mut f64, &'a mut f64, &'a mut f64);

        fn next<'b>(&mut self) -> Option<Self::Item> {
            let t = if self.a0.len() > self.index {
                let a = self.a0.as_mut_ptr();
                let b = self.a1.as_mut_ptr();
                let c = self.a2.as_mut_ptr();
                unsafe {
                    Some((
                        &mut *a.add(self.index),
                        &mut *b.add(self.index),
                        &mut *c.add(self.index),
                    ))
                }
            } else {
                None
            };
            self.index += 1;
            t
        }
    }
}

pub struct Chunk {
    a0: Mutex<Vec<f64>>,
    a1: Mutex<Vec<f64>>,
    a2: Mutex<Vec<f64>>,
}

pub struct ChunkIterator<'a> {
    a0: MutexGuard<'a, Vec<f64>>,
    a1: MutexGuard<'a, Vec<f64>>,
    a2: MutexGuard<'a, Vec<f64>>,
    index: usize,
}

impl<'a> IntoIterator for &'a Chunk {
    type Item = (&'a mut f64, &'a mut f64, &'a mut f64);

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

impl Chunk {
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
    type Item = (&'a mut f64, &'a mut f64, &'a mut f64);

    fn next<'b>(&mut self) -> Option<Self::Item> {
        let t = if self.a0.len() > self.index {
            let a = self.a0.as_mut_ptr();
            let b = self.a1.as_mut_ptr();
            let c = self.a2.as_mut_ptr();
            unsafe {
                Some((
                    &mut *a.add(self.index),
                    &mut *b.add(self.index),
                    &mut *c.add(self.index),
                ))
            }
        } else {
            None
        };
        self.index += 1;
        t
    }
}

#[test]
fn chunk_test() {
    let a0 = Mutex::new(vec![0.0; 5]);
    let a1 = Mutex::new(vec![1.0; 5]);
    let a2 = Mutex::new(vec![2.0; 5]);

    let c = Arc::new(Chunk { a0, a1, a2 });

    let a02 = Mutex::new(vec![10.0; 5]);
    let a12 = Mutex::new(vec![11.0; 5]);
    let a22 = Mutex::new(vec![12.0; 5]);

    let c2 = Arc::new(Chunk {
        a0: a02,
        a1: a12,
        a2: a22,
    });

    let ct = c.clone();
    let ct2 = c2.clone();

    let t1 = thread::spawn(move || {
        if let Some(ct) = ct.try_into_iter() {
            if let Some(ct2) = ct2.try_into_iter() {
                for i in ct.into_iter().chain(ct2.into_iter()) {
                    thread::sleep(Duration::from_secs(1));
                    println!("{:?}", i);
                }
            }
        }
    });

    let ctt = c.clone();
    let ctt2 = c2.clone();

    let t2 = thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        if let Some(ct) = ctt.try_into_iter() {
            if let Some(ct2) = ctt2.try_into_iter() {
                for i in ct.into_iter().chain(ct2.into_iter()) {
                    thread::sleep(Duration::from_secs(1));
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

#[test]
fn test_chain_zip() {
    let a0 = Arc::new(Mutex::new([0.0; 5]));
    let a1 = Arc::new(Mutex::new([1.0; 5]));
    let a2 = Arc::new(Mutex::new([2.0; 5]));

    let a0l = a0.lock().unwrap();
    let a1l = a1.lock().unwrap();
    let a2l = a2.lock().unwrap();

    let qw = a0l.iter().zip(a1l.iter()).zip(a2l.iter());

    for k in qw {
        println!("{:?}", k);
    }
}
