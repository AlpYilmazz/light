use crate::ecs::{error::StorageFault, component::{ComponentDescriptor, Resource, ComponentId}, entity::Entity};

use super::table::Table;


pub struct ResourceTable {
    entity_id: usize,
    table: Table,
}

impl ResourceTable {
    pub const GLOBAL_ENTITY: Entity = Entity { id: 0, table_id: 0 };

    #[inline]
    pub fn new() -> ResourceTable {
        let mut table = Table::new();
        let row = table.add_row(Self::GLOBAL_ENTITY.clone());
        ResourceTable {
            entity_id: row,
            table,
        }
    }

    pub fn add_column(&mut self, descriptor: &ComponentDescriptor) {
        self.table.add_column(descriptor);
    }

    pub fn get_resource(&self, resource_id: &ComponentId) -> Option<*const u8> {
        unsafe {
            Some(self.table.get_column(resource_id)?
                .get_unchecked(self.entity_id))
        }
    }

    pub fn get_resource_mut(&self, resource_id: &ComponentId) -> Option<*mut u8> {
        unsafe {
            Some(self.table.get_column(resource_id)?
                .get_unchecked(self.entity_id))
        }
    }

    pub unsafe fn remove_column(&mut self, resource_id: &ComponentId) {
        self.table.remove_column(resource_id);
    }

    pub unsafe fn init_resource_unchecked(&mut self, resource_id: &ComponentId, data: *mut u8) {
        self.table.get_column_mut(resource_id).unwrap()
                .init_unchecked(self.entity_id, data);
    }

    pub unsafe fn remove_and_drop_unchecked(&mut self, resource_id: &ComponentId) {
        self.table.get_column_mut(resource_id).unwrap()
                .swap_remove_and_drop_unchecked(self.entity_id);
    }

    /*#[inline]
    pub fn register_resource<T: Resource>(&mut self, descriptor: ComponentDescriptor) {
        self.table.register_component(descriptor)
    }

    #[inline]
    pub fn get_resource<'a, T: Resource>(&'a self) -> Result<&'a T, StorageFault> {
        self.table.get_component_of_entity::<T>(self.resource_entity_id)
    }

    #[inline]
    pub fn get_mut_resource<'a, T: Resource>(&'a mut self) -> Result<&'a mut T, StorageFault> {
        self.table.get_mut_component_of_entity::<T>(self.resource_entity_id)
    }

    #[inline]
    pub fn init_resource<T: Resource>(&mut self, data: T) -> Result<(), StorageFault> {
        self.table.init_component_of_entity::<T>(self.resource_entity_id, data)
    }

    #[inline]
    pub fn replace_resource<T: Resource>(&mut self, data: T) -> Result<(), StorageFault> {
        self.table.replace_component_of_entity::<T>(self.resource_entity_id, data)
    }

    #[inline]
    pub fn delete_resource<T: Resource>(&mut self) -> Result<(), StorageFault> {
        self.table.delete_component_of_entity::<T>(self.resource_entity_id)
    }*/
}