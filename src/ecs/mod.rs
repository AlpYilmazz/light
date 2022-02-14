
use self::{storage::{resource_table::ResourceTable, table::Tables}, entity::Entities, component::{Components, ComponentId, Component}};


pub mod error;
pub mod entity;
pub mod component;
pub mod system;
pub mod query;
pub mod storage;


pub struct World {
    entities: Entities,
    components: Components,
    resources: ResourceTable,
    tables: Tables,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Entities::new(),
            components: Components::new(),
            resources: ResourceTable::new(),
            tables: Tables::new(),
        }
    }

    pub fn add_component<T: Component>(&mut self) -> ComponentId {
        self.components.add_component::<T>()
    }

    pub fn get_resource_table<'a>(&'a self) -> &'a ResourceTable {
        &self.resources
    }

    pub fn get_resource_table_mut<'a>(&'a mut self) -> &'a mut ResourceTable {
        &mut self.resources
    }

    pub fn get_tables<'a>(&'a self) -> &'a Tables {
        &self.tables
    }

    pub fn get_tables_mut<'a>(&'a mut self) -> &'a mut Tables {
        &mut self.tables
    }

}


#[cfg(test)]
mod tests {
    
    #[test]
    fn bitwise() {
        println!("{}", 1 << 4);
        assert_eq!(16, 1 << 4);
    }

}