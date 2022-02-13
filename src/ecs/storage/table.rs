use std::{any::TypeId, alloc::Layout};
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::ptr::NonNull;
use std::hash::{Hash, Hasher};

use crate::ecs::component::{ComponentDescriptor, Component, ComponentId};
use crate::ecs::entity::Entity;
use crate::ecs::error::StorageFault;

use super::blobvec::BlobVec;


pub struct Column {
    component_id: ComponentId,
    column_data: BlobVec,
}

impl Column {
    const unsafe fn drop(ptr: *mut u8) {

    }

    #[inline]
    fn new(descriptor: &ComponentDescriptor) -> Column {
        Column::with_capacity(descriptor, 0)
    }

    #[inline]
    fn with_capacity(descriptor: &ComponentDescriptor, capacity: usize) -> Column {
        let layout = descriptor.layout.clone();
        Column {
            component_id: descriptor.id.clone(),
            column_data: BlobVec::new(layout, capacity, Self::drop),
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.column_data.capacity()
    }

    #[inline]
    pub fn item_capacity(&self) -> usize {
        self.column_data.item_capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.column_data.len()
    }

    #[inline]
    pub fn items_len(&self) -> usize {
        self.column_data.items_len()
    }

    #[inline]
    pub fn get_ptr(&self) -> NonNull<u8> {
        self.column_data.get_ptr()
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.column_data.reserve_exact(additional)
    }

    #[inline]
    pub fn push_uninit(&mut self) -> usize {
        self.column_data.push_uninit()
    }

    #[inline]
    pub unsafe fn init_unchecked(&mut self, index: usize, value: *mut u8) {
        self.column_data.init_unchecked(index, value)
    }

    #[inline]
    pub unsafe fn replace_unchecked(&mut self, index: usize, value: *mut u8) {
        self.column_data.replace_unchecked(index, value)
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> *mut u8 {
        self.column_data.get_unchecked(index)
    }

    pub unsafe fn swap_remove_and_forget_unchecked(&mut self, index: usize) -> *mut u8 {
        self.column_data.swap_remove_and_forget_unchecked(index)
    }

    pub unsafe fn swap_remove_and_drop_unchecked(&mut self, index: usize) {
        self.column_data.swap_remove_and_drop_unchecked(index)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.column_data.clear()
    }
}

/*#[derive(Clone)]
pub struct Entity {
    pub id: usize,
    pub has_mask: u64, // Components marked by bits (64 bits -> 64 Components)
    pub init_mask: u64,
}*/

pub struct TableMoveResult {
    pub moved_row: usize,
    pub swapped_entity: Option<Entity>,
}

#[derive(Default)]
pub struct Table {
    components: HashMap<ComponentId, Column>,
    entities: Vec<Entity>,
    //bitmasks: HashMap<TypeId, u64>,
}

impl Table {
    pub fn new() -> Table {
        Default::default()
    }

    pub fn with_capacity(row_capacity: usize, column_capacity: usize) -> Table {
        Table {
            components: Default::default(),
            entities: Vec::with_capacity(row_capacity),
        }
    }

    // X Add column - Component
    // X Add row - Entity
    // X Remove row: swap delete
    // X(get_column_mut) Init cell
    // Move row to another table
    // X(reserve_rows_exact) Batch add row

    pub fn add_column(&mut self, descriptor: &ComponentDescriptor) {
        self.components.entry(descriptor.id.clone()).or_insert_with(|| {
            Column::with_capacity(descriptor, self.entities.capacity())
        });
    }

    pub fn add_row(&mut self, entity: Entity) -> usize {
        for column in self.components.values_mut() {
            column.push_uninit();
        }
        self.entities.push(entity);
        self.entities.len() - 1
    }

    pub fn reserve_rows_exact(&mut self, additional: usize) {
        if self.entities.capacity() - self.entities.len() < additional {
            self.entities.reserve(additional);

            // use entities vector capacity as driving capacity for all related allocations
            // Vec::reserve may have reserved more space
            let new_capacity = self.entities.capacity();

            for column in self.components.values_mut() {
                column.reserve_exact(new_capacity - column.len());
            }
        }
    }

    pub fn get_column(&self, component_id: &ComponentId) -> Option<&Column> {
        self.components.get(component_id)
    }

    pub fn get_column_mut(&mut self, component_id: &ComponentId) -> Option<&mut Column> {
        self.components.get_mut(component_id)
    }

    pub unsafe fn swap_remove_and_drop_unchecked(&mut self, row: usize) -> Option<Entity> {
        for column in self.components.values_mut() {
            column.swap_remove_and_drop_unchecked(row);
        }

        let is_last = (row == self.entities.len() - 1);
        self.entities.swap_remove(row);        
        if is_last {
            None
        }
        else {
            Some(self.entities[row].clone())
        }
    }

    pub unsafe fn move_row_to_superset_unchecked(&mut self, row: usize, dst_table: &mut Table) -> TableMoveResult {
        debug_assert!(row < self.entities.len());

        let is_last = (row == self.entities.len() - 1);
        let moved_row = dst_table.add_row(self.entities.swap_remove(row));
        for column in self.components.values_mut() {
            let dst_column = dst_table.get_column_mut(&column.component_id).unwrap();
            let cell_data = column.swap_remove_and_forget_unchecked(row);
            dst_column.init_unchecked(moved_row, cell_data);
        }

        TableMoveResult {
            moved_row,
            swapped_entity: if is_last {
                None
            }
            else {
                Some(self.entities[row].clone())
            }
        }
    }

    pub unsafe fn move_row_forget_missing_unchecked(&mut self, row: usize, dst_table: &mut Table) -> TableMoveResult {
        debug_assert!(row < self.entities.len());

        let is_last = (row == self.entities.len() - 1);
        let moved_row = dst_table.add_row(self.entities.swap_remove(row));
        for column in self.components.values_mut() {
            let dst_column = dst_table.get_column_mut(&column.component_id);
            if let Some(dst_column) = dst_column {
                let cell_data = column.swap_remove_and_forget_unchecked(row);
                dst_column.init_unchecked(moved_row, cell_data);   
            }
            // if None => forget
        }

        TableMoveResult {
            moved_row,
            swapped_entity: if is_last {
                None
            }
            else {
                Some(self.entities[row].clone())
            }
        }
    }

    pub unsafe fn move_row_drop_missing_unchecked(&mut self, row: usize, dst_table: &mut Table) -> TableMoveResult {
        debug_assert!(row < self.entities.len());

        let is_last = (row == self.entities.len() - 1);
        let moved_row = dst_table.add_row(self.entities.swap_remove(row));
        for column in self.components.values_mut() {
            let dst_column = dst_table.get_column_mut(&column.component_id);
            match dst_column {
                Some(dst_column) => {
                    let cell_data = column.swap_remove_and_forget_unchecked(row);
                    dst_column.init_unchecked(moved_row, cell_data);   
                },
                None => {
                    column.swap_remove_and_drop_unchecked(row);
                }
            }
        }

        TableMoveResult {
            moved_row,
            swapped_entity: if is_last {
                None
            }
            else {
                Some(self.entities[row].clone())
            }
        }
    }

    pub unsafe fn remove_column(&mut self, component_id: &ComponentId) {
        self.get_column_mut(component_id).unwrap()
                .clear();
        self.components.remove(component_id);
    }

    // LEGACY

    /*pub fn register_component(&mut self, descriptor: &ComponentDescriptor) {
        let typeid = descriptor.typeid.clone();
        let capacity = self.entities.len();

        self.components.insert(typeid, Column::with_capacity(descriptor, capacity));
        
        let bitmask: u64 = 1 << (self.bitmasks.keys().len());
        self.bitmasks.insert(typeid, bitmask);
    }

    /*pub fn register_component<T: Component>(&mut self) {
        let id: usize = 0;
        let typeid = TypeId::of::<T>();
        let capacity = self.entities.len();

        self.components.insert(typeid, Column::with_capacity::<T>(id, capacity));
        
        let bitmask: u64 = 1 << (self.bitmasks.keys().len());
        self.bitmasks.insert(typeid, bitmask);
    }*/

    pub fn spawn_entity(&mut self) -> EntityMut {
        let entt_id = self.entities.len();
        let entity = Entity {
            id: entt_id,
            has_mask: 0,
            init_mask: 0,
        };

        self.entities.push(entity.clone());
        for (_, column) in &mut self.components {
            column.push_uninit();
        }

        EntityMut::new(self, entity)
    }

    pub fn contains_entity(&self, entt_id: usize) -> bool {
        entt_id < self.entities.len()
    }

    pub fn has_component<T: Component>(&self, entt_id: usize) -> Result<bool, StorageFault> {
        let typeid = TypeId::of::<T>();
        let bitmask = self.bitmasks.get(&typeid).ok_or(StorageFault::ComponentNotRegistered)?;

        if !self.contains_entity(entt_id) {
            return Err(StorageFault::EntityNotCreated);
        }
        
        Ok((self.entities[entt_id].has_mask & bitmask) == *bitmask)
    }

    pub fn is_init_component<T: Component>(&self, entt_id: usize) -> Result<bool, StorageFault> {
        self.has_component::<T>(entt_id)?;

        let typeid = TypeId::of::<T>();
        let bitmask = self.bitmasks.get(&typeid).ok_or(StorageFault::ComponentNotRegistered)?;

        Ok((self.entities[entt_id].init_mask & bitmask) == *bitmask)
    }

    fn check_request_integrity<T: Component>(&self, entt_id: usize) -> Result<(), StorageFault> {
        if !self.contains_entity(entt_id) {
            return Err(StorageFault::EntityNotCreated);
        }
        let hs = self.is_init_component::<T>(entt_id)?; // ? -> ComponentNotRegistered | EntityNotCreated | NoComponentOnEntity
        if !hs {
            return Err(StorageFault::ComponentUninitOnEntity);
        }

        // checks for
        // EntityNotCreated, ComponentNotRegistered, NoComponentOnEntity, ComponentUninitOnEntity
        Ok(())
    }

    pub fn get_component_of_entity<'a, T: Component>(&'a self, entt_id: usize) -> Result<&'a T, StorageFault> {
        self.check_request_integrity::<T>(entt_id)?;

        let typeid = TypeId::of::<T>();

        unsafe {
            Ok(&*self.components.get(&typeid).unwrap()
                    .get_unchecked(entt_id)
                    .cast::<T>())
        }
    }

    pub fn get_mut_component_of_entity<'a, T: Component>(&'a mut self, entt_id: usize) -> Result<&'a mut T, StorageFault> {
        self.check_request_integrity::<T>(entt_id)?;

        let typeid = TypeId::of::<T>();

        unsafe {
            Ok(&mut *self.components.get(&typeid).unwrap()
                    .get_unchecked(entt_id)
                    .cast::<T>())
        }
    }

    // Mark Entity to contain the Component
    // Does not initalize
    pub fn add_component_to_entity<T: Component>(&mut self, entt_id: usize) -> Result<(), StorageFault> {
        let ans = self.has_component::<T>(entt_id);
        match ans {
            Ok(false) => { // Component registered, entity exists, component not added to entity
                let typeid = TypeId::of::<T>();
                let bitmask = self.bitmasks.get(&typeid).unwrap();

                self.entities[entt_id].has_mask |= bitmask;

                Ok(())
            },
            Ok(true) => Ok(()), // Component already added to entity
            Err(err) => Err(err), // StorageFault::{EntityNotCreated, ComponentNotRegistered}
        }
    }

    // Initialize Component of Entity if Entity has the Component
    pub fn init_component_of_entity<T: Component>(&mut self, entt_id: usize, mut data: T) -> Result<(), StorageFault> {
        self.has_component::<T>(entt_id)?;
        
        let typeid = TypeId::of::<T>();
        let bitmask = self.bitmasks.get(&typeid).unwrap();
        let data_ptr: *mut u8 = std::ptr::addr_of_mut!(data).cast::<u8>();

        unsafe {
            self.components.get_mut(&typeid).unwrap()
                    .init_unchecked(entt_id, data_ptr);
        }

        self.entities[entt_id].init_mask |= bitmask;

        Ok(())
    }

    // Replace Component of Entity if Entity has the Component and it is initialized
    pub fn replace_component_of_entity<T: Component>(&mut self, entt_id: usize, mut data: T) -> Result<(), StorageFault> {
        self.check_request_integrity::<T>(entt_id)?;
        
        let typeid = TypeId::of::<T>();
        let data_ptr: *mut u8 = std::ptr::addr_of_mut!(data).cast::<u8>();

        // entt_id is valid index -> No (use after free, use uninit)
        // data_ptr is valid -> input as T
        unsafe {
            self.components.get_mut(&typeid).unwrap()
                    .replace_unchecked(entt_id, data_ptr);
        }

        Ok(())
    }

    pub fn delete_component_of_entity<T: Component>(&mut self, entt_id: usize) -> Result<(), StorageFault> {
        self.check_request_integrity::<T>(entt_id)?;

        let typeid = TypeId::of::<T>();
        let bitmask = self.bitmasks.get(&typeid).unwrap();
        let entity = self.entities.get_mut(entt_id).unwrap();
        
        entity.has_mask &= *bitmask ^ u64::MAX;
        entity.init_mask &= *bitmask ^ u64::MAX;

        Ok(())
    }*/
}

pub struct Tables {
    tables_vec: Vec<Table>,
    ids: HashMap<u64, usize>, // archetype hash -> table id
}

impl Tables {
    pub fn new() -> Self {
        Tables {
            tables_vec: vec![],
            ids: Default::default(),
        }
    }

    pub fn get_table<'a>(&'a self, table_id: usize) -> Option<&'a Table> {
        self.tables_vec.get(table_id)
    }

    pub fn get_table_mut<'a>(&'a mut self, table_id: usize) -> Option<&'a mut Table> {
        self.tables_vec.get_mut(table_id)
    }

    /*pub fn new_table<'a>(&'a mut self, components: &[ComponentDescriptor]) -> &'a mut Table {
        let component_ids: Vec<usize> = components.iter()
                .map(|cd| cd.id)
                .collect();

        let mut hasher = DefaultHasher::new();
        component_ids.hash(&mut hasher);
        let table_signature = hasher.finish();
        // TODO: what to do with permutations, assuming they produce different hash values

        let mut new_table = Table::new();
        for cd in components {
            new_table.register_component(&cd);
        }

        let index = self.tables_vec.len();
        self.ids.insert(table_signature, index);
        self.tables_vec.push(new_table);

        unsafe { self.tables_vec.get_unchecked_mut(index) }
    }*/
}

/*pub struct EntityMut<'a> {
    entity: Entity,
    table: &'a mut Table,
}

impl<'a> EntityMut<'a> {
    pub fn new(table: &mut Table, entity: Entity) -> EntityMut {
        EntityMut {
            entity,
            table,
        }
    }

    pub fn get_component<T: Component>(&self) -> Result<&T, StorageFault> {
        self.table.get_component_of_entity::<T>(self.entity.id)
    }

    pub fn get_mut_component<T: Component>(&mut self) -> Result<&mut T, StorageFault> {
        self.table.get_mut_component_of_entity::<T>(self.entity.id)
    }

    pub fn init_component<T: Component>(&mut self, data: T) -> Result<(), StorageFault> {
        self.table.init_component_of_entity::<T>(self.entity.id, data)
    }

    pub fn replace_component<T: Component>(&mut self, data: T) -> Result<(), StorageFault> {
        self.table.replace_component_of_entity::<T>(self.entity.id, data)
    }

    pub fn delete_component<T: Component>(&mut self) -> Result<(), StorageFault> {
        self.table.delete_component_of_entity::<T>(self.entity.id)
    }
}*/


#[cfg(test)]
mod tests {
    use super::{Table, ComponentDescriptor, Entity};


    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct Health(u64);
    
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct Mana(u64);

    #[test]
    fn register_and_get_component() {

    }

}