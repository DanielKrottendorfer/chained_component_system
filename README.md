# chained_component_system      

`chained_component_system` is an attempt to create an ECS-Style component system, where all functionalities are generated at compile time. <br>
Components, Entities and Systems are defined in a procedural macro. This macro creates a SOA-struct for each Entity. Each System chains the SOA components of every Entity with a fitting signature together. 
These Chained structures can be accessed through Accessors. Accessors are `Send + Sync` so they can be shared/handed off to different threads. <br> 
The contents of each Accessors are contained in an RwLock so depending on the definition in the System, Components will be locked to `Read` or to `Write`.

<br>

```rust

chained_component_system!(
    components{
        foo: Foo,
        goo: Goo,
        hoo: Hoo,
        loo: Foo,
    };

    entities{
        Peon(foo, goo),
        NoCont(foo,goo,hoo,loo),
        Tree(loo, goo, foo),
        Mage(loo, goo, hoo),
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
```
<br>

To manage the state of a given SOA-Element generational keys are used.
Deleted Entities are not dropped immediately external resources may need to be handled separately.

<br>

```rust
pub struct Key {
    index: usize,
    generation: u32,
    entity_type: EntityType,
}

pub enum EntityState {
    Free { next_free: usize },
    Occupied,
}
```
