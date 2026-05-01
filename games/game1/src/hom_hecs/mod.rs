// hom_hecs/mod.rs — Homun-friendly free-function adapter over the hecs ECS.
// Inlined by homunc when a .hom does `use hom_hecs`.

use hecs::Component;
use std::cell::RefCell;

pub use hecs::{Entity, World};
pub use gametok_abi::InputFrame;

thread_local! {
    static WORLD: RefCell<World> = RefCell::new(World::new());
}

pub fn world_reset() {
    WORLD.with(|w| *w.borrow_mut() = World::new());
}

pub fn spawn1<A: Component>(a: A) -> Entity {
    WORLD.with(|w| w.borrow_mut().spawn((a,)))
}

pub fn spawn2<A: Component, B: Component>(a: A, b: B) -> Entity {
    WORLD.with(|w| w.borrow_mut().spawn((a, b)))
}

pub fn spawn3<A: Component, B: Component, C: Component>(a: A, b: B, c: C) -> Entity {
    WORLD.with(|w| w.borrow_mut().spawn((a, b, c)))
}

pub fn despawn(e: Entity) -> bool {
    WORLD.with(|w| w.borrow_mut().despawn(e).is_ok())
}

pub fn query1<A, F>(mut f: F)
where
    A: Component + Clone,
    F: FnMut(A),
{
    WORLD.with(|w| {
        let w = w.borrow();
        for (_, a) in w.query::<&A>().iter() {
            f(a.clone());
        }
    });
}

pub fn query1_mut<A, F>(mut f: F)
where
    A: Component + Clone,
    F: FnMut(A) -> A,
{
    WORLD.with(|w| {
        let mut w = w.borrow_mut();
        for (_, a) in w.query_mut::<&mut A>() {
            let new_a = f(a.clone());
            *a = new_a;
        }
    });
}

pub fn query2<A, B, F>(mut f: F)
where
    A: Component + Clone,
    B: Component + Clone,
    F: FnMut(A, B),
{
    WORLD.with(|w| {
        let w = w.borrow();
        for (_, (a, b)) in w.query::<(&A, &B)>().iter() {
            f(a.clone(), b.clone());
        }
    });
}

pub fn query2_mut<A, B, F>(mut f: F)
where
    A: Component + Clone,
    B: Component + Clone,
    F: FnMut(A, B) -> A,
{
    WORLD.with(|w| {
        let mut w = w.borrow_mut();
        for (_, (a, b)) in w.query_mut::<(&mut A, &B)>() {
            let new_a = f(a.clone(), b.clone());
            *a = new_a;
        }
    });
}

pub fn with1_mut<A: Component, F>(e: Entity, mut f: F) -> bool
where
    F: FnMut(&mut A),
{
    WORLD.with(|w| {
        let mut w = w.borrow_mut();
        for (eid, a) in w.query_mut::<&mut A>() {
            if eid == e {
                f(a);
                return true;
            }
        }
        false
    })
}

pub fn scale_step(scale: f32, delta: f32, dt: f32) -> f32 {
    if delta == 0.0 {
        scale
    } else {
        (scale * 1.5_f32.powf(delta * dt)).clamp(0.5, 50.0)
    }
}
