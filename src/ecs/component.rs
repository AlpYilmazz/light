use std::{any::TypeId, alloc::Layout, collections::HashMap};

use fixedbitset::FixedBitSet;


pub trait Resource: Send + Sync + 'static {}
impl<T> Resource for T where T: Send + Sync + 'static {}

pub trait Component: Send + Sync + 'static {}
impl<T> Component for T where T: Send + Sync + 'static {}


#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentId(usize);
impl ComponentId {
    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Clone)]
pub struct ComponentDescriptor {
    pub id: ComponentId,
    pub typeid: TypeId,
    pub layout: Layout,
    pub bitmask: FixedBitSet,
}

impl ComponentDescriptor {
    #[inline]
    pub fn of<T: Component>(component_id: ComponentId) -> ComponentDescriptor {
        let id = component_id.id();

        let block_size: usize = 32;
        
        let (mut blocks, rem) = (id / block_size, id % block_size);
        blocks += (rem > 0) as usize;
        let mut bitmask = FixedBitSet::with_capacity(blocks * block_size);
        bitmask.set(id, true);

        ComponentDescriptor {
            id: component_id,
            typeid: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            bitmask,
        }
    }

    #[inline]
    pub fn is_for<T: Component>(&self) -> bool {
        self.typeid == TypeId::of::<T>()
    }
}

/// Component/Resource Bookkeeper
/// Resources are Components of a specific Entity (the global entity)
#[derive(Default)]
pub struct Components {
    descriptors: Vec<ComponentDescriptor>,
    indices: HashMap<TypeId, usize>,
    resource_indices: HashMap<TypeId, usize>,
}

impl Components {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn add_component<T: Component>(&mut self) -> ComponentId {
        let typeid = TypeId::of::<T>();
        let index = self.descriptors.len();
        self.indices.entry(typeid).or_insert_with(|| {
            self.descriptors.push(ComponentDescriptor::of::<T>(ComponentId(index)));
            index
        });
        ComponentId(index)
    }

    #[inline]
    pub fn get_component<T: Component>(&self) -> Option<&ComponentDescriptor> {
        let typeid = TypeId::of::<T>();
        let index = self.indices.get(&typeid)?;
        self.descriptors.get(*index)
    }

    #[inline]
    pub fn add_resource<T: Resource>(&mut self) -> ComponentId {
        let typeid = TypeId::of::<T>();
        let index = self.descriptors.len();
        self.resource_indices.entry(typeid).or_insert_with(|| {
            self.descriptors.push(ComponentDescriptor::of::<T>(ComponentId(index)));
            index
        });
        ComponentId(index)
    }

    #[inline]
    pub fn get_resource<T: Resource>(&self) -> Option<&ComponentDescriptor> {
        let typeid = TypeId::of::<T>();
        let index = self.resource_indices.get(&typeid)?;
        self.descriptors.get(*index)
    }
}


#[cfg(test)]
mod tests {
    use std::ops::{BitAnd, BitOr};

    use fixedbitset::FixedBitSet;


    #[test]
    fn fixedbitset() {
        let mut b32 = FixedBitSet::with_capacity(32);
        let mut b64 = FixedBitSet::with_capacity(64);
        let mut b100 = FixedBitSet::with_capacity(100);
        
        b32.set(3, true);
        b64.set(40, true);

        let bor = &b32 | &b64;
        let band = &b32 & &b64;
        println!("b100:   {:?}", b100);
        println!("bor:    {:?}\n{}", bor, bor);
        println!("band:   {:?}\n{}", band, band);
    }

}