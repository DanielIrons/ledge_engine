use crate::{
    ecs::component::Component,
    ecs::{Fetch, FetchMut},
    ecs::entity::Entities,
    ecs::entity::Entity,
    ecs::join::Joinable,
    ecs::layeredbitmap::LayeredBitMap,
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut}
};

use vector_map::VecMap;

// Stores the bitset (used for joining) and the physical storage for the component.
pub struct TrackedStorage<C: Component> {
    pub bitset: LayeredBitMap,
    pub inner: C::Storage,
}

impl<C: Component> TrackedStorage<C> {
    pub fn new(storage: C::Storage) -> Self {
        Self {
            bitset: LayeredBitMap::new(),
            inner: storage,
        }
    }

    pub fn insert(&mut self, index: usize, value: C) {
        self.inner.insert(index, value);
        self.bitset.insert(index);
    }
}

// Simple vec wrapper, added because in the future there will be other storage types as well.
pub struct VecStorage<T> {
    pub inner: VecMap<usize, T>,
}

impl<T: Default> Default for VecStorage<T> {
    fn default() -> Self {
        Self{
            inner: Default::default()
        }
    }
}

// Found this online helps with defaults for some of the types.
pub trait TryDefault: Sized {
    fn try_default() -> Result<Self, String>;

    fn unwrap_default() -> Self {
        match Self::try_default() {
            Ok(x) => x,
            Err(e) => panic!("Failed to create a default value for storage ({:?})", e),
        }
    }
}

impl<T> TryDefault for T
where
    T: Default,
{
    fn try_default() -> Result<Self, String> {
        Ok(T::default())
    }
}

pub trait DynamicStorage<T>: TryDefault {
    fn insert(&mut self, index: usize, value: T);
    fn get(&self, id: usize) -> &T;
}

impl<T: Default> DynamicStorage<T> for VecStorage<T> {
    fn insert(&mut self, index: usize, value: T) {
        self.inner.insert(index, value);
        // if index < self.inner.len() {
        //     println!("Inserting in dyn storage entity with id: {}", index);
        //     self.inner.insert(index, value); // TODO this seems like a bad way to do it.
        // } else if index == 0 && self.inner.len() == 0 {
        //     println!("Inserting in dyn storage when 0 entity with id: {}", index);
        //     self.inner.insert(index, value); // TODO this seems like a bad way to do it.
        // }
    }

    fn get(&self, id: usize) -> &T {
        self.inner.get(&id).unwrap()
    }
} 

impl<C: Component> Default for TrackedStorage<C>
where
    C::Storage: Default
{
    fn default() -> Self {
        Self {
            bitset: Default::default(),
            inner: Default::default(),
        }
    }
}

// Currently only used for component storage reads. TODO impl
pub struct Storage<'a, T, D> {
    pub data: D,
    pub entities: Entities<'a>,
    pub phantom: PhantomData<T>,
}

impl<'a, T, D> Storage<'a, T, D>
where 
    D: DerefMut<Target = TrackedStorage<T>>,
    T: Component,
    {
    pub fn insert(&mut self, entity: Entity, value: T) {
        // println!("Inserting in storage entity with id: {}", entity.id());
        self.data.insert(entity.id(), value);
    }

    pub fn get(&self, e: Entity) -> Option<&T> {
        Some(self.data.inner.get(e.id()))
        // if self.data.bitset.contains(e.id()) && self.entities.is_alive(e) {
        //     Some(self.data.inner.get(e.id()))
        // } else {
        //     None
        // }
    }
}

impl<'a, 'b, T, D> Joinable for &'a Storage<'b, T, D>
where
    T: Component,
    D: Deref<Target = TrackedStorage<T>> 
{
    type Value = &'a T::Storage;
    type Type = &'a T;

    fn get_values(&self) -> Self::Value {
        // println!("{:?}", self.data.inner);
        &self.data.inner
    }

    fn get_keys(&self) -> Vec<usize> {
        println!("{:?}", LayeredBitMap::join(&self.data.bitset, &self.data.bitset));
        LayeredBitMap::join(&self.data.bitset, &self.data.bitset)
    }

    fn get(value: &mut Self::Value, index: usize) -> Self::Type {
        value.get(index)
    }
}

// impl<'a, 'b, T, D> Joinable for &'a mut Storage<'b, T, D>
// where
//     T: Component,
//     D: Deref<Target = TrackedStorage<T>> 
// {
//     type Value = &'a mut T::Storage;
//     type Type = &'a T;

//     fn get_values(&self) -> Self::Value {
//         &self.data.inner
//     }

//     fn get_keys(&self) -> Vec<usize> {
//         LayeredBitMap::join(&self.data.bitset, &self.data.bitset)
//     }

//     fn get(value: &mut Self::Value, index: usize) -> Self::Type {
//         value.get(index)
//     }
// }

// pub trait SystemStorage {}

pub type ReadStorage<'a, T> = Storage<'a, T, Fetch<'a, TrackedStorage<T>>>;

pub type WriteStorage<'a, T> = Storage<'a, T, FetchMut<'a, TrackedStorage<T>>>;

impl<'a, 'b, T, D, A, B> Joinable for (&'a Storage<'b, T, D>, &'a Storage<'b, A, B>)
where
    A: Component,
    B: Deref<Target = TrackedStorage<A>>,
    T: Component,
    D: Deref<Target = TrackedStorage<T>> 
{
    type Value = (&'a T::Storage, &'a A::Storage);
    type Type = (&'a T, &'a A);

    fn get_values(&self) -> Self::Value {
        (&self.0.data.inner, &self.1.data.inner)
    }

    fn get_keys(&self) -> Vec<usize> {
        LayeredBitMap::join(&self.0.data.bitset, &self.1.data.bitset)
    }

    fn get(value: &mut Self::Value, index: usize) -> Self::Type {
        (value.0.get(index), value.1.get(index))
    }
}