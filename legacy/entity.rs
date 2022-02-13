use std::{rc::Rc, cell::RefCell, any::{Any, TypeId}, collections::HashMap};

use super::{error::EcsFault, Component};


pub type EntityId = usize;
pub type EntityKey = u64;
pub type Entities = Vec<EntityKey>;


pub struct Entity {
    pub entt_id: EntityId,
}

pub type ComponentAccess = Rc<RefCell<dyn Component>>;
pub type ComponentTable = HashMap<TypeId, Vec<Option<ComponentAccess>>>;


#[derive(Default)]
pub struct EntityRegistry {
    components: ComponentTable,
    entities: Entities,
    cmp_bitmasks: HashMap<TypeId, u64>
}

impl EntityRegistry {
    pub fn new() -> Self {
        Default::default()
    }
    
    pub fn register_component<T: Component>(&mut self) {
        let typeid = TypeId::of::<T>();

        self.components.insert(typeid, Vec::new());
        
        let bitmask = 1 << (self.cmp_bitmasks.keys().len());
        self.cmp_bitmasks.insert(typeid, bitmask);
    }

    pub fn new_entity(&mut self) -> EntityPrototype {
        self.entities.push(0);
    
        self.components.iter_mut().for_each(|(_key, component)| {
            component.push(None);
        });
    
        let entt_id = self.entities.len() - 1;
        EntityPrototype::new(self, entt_id)
    }
    
    pub fn spawn(&mut self, entt_id: EntityId, mut components: HashMap<TypeId, ComponentAccess>) {
        components.drain().for_each(|(typeid, cmp_data)| {
            let bitmask = self.cmp_bitmasks.get(&typeid).unwrap();
    
            // component[entity] := data
            let cmp = self.components.get_mut(&typeid).unwrap();
            let cell = cmp.get_mut(entt_id).unwrap();
            *cell = Some(cmp_data);
    
            // entity << component
            self.entities[entt_id] |= *bitmask;
        })
    }

    pub fn get_component_of_entity<T: Component>(&self, entt_id: EntityId) -> Result<ComponentAccess, EcsFault> {
        self.everything_ok::<T>(entt_id)?;
        // -- entity & component exists --
        // -- entity has component --

        let typeid = TypeId::of::<T>();
        Ok(Rc::clone(&self.components.get(&typeid).unwrap()[entt_id].as_ref().unwrap()))
    }

    pub fn replace_component_of_entity<T: Component>(&mut self, entt_id: EntityId, data: T) -> Result<(), EcsFault> {  
        if !self.contains_entity(entt_id) {
            return Err(EcsFault::EntityNotCreated);
        }

        let typeid = data.type_id();
        let bitmask = self.cmp_bitmasks.get(&typeid).ok_or(EcsFault::ComponentNotRegistered)?;
    
        // -- entity & component exists --

        // component[entity] := data
        let cmp = self.components.get_mut(&typeid).unwrap();
        let cell = cmp.get_mut(entt_id).unwrap();
        *cell = Some(Rc::new(RefCell::new(data)));

        // entity << component
        self.entities[entt_id] |= *bitmask;
        
        Ok(())
    }

    pub fn delete_component_of_entity<T: Component>(&mut self, entt_id: EntityId) -> Result<(), EcsFault> {
        self.everything_ok::<T>(entt_id)?;
        // -- entity & component exists --
        // -- entity has component --

        let typeid = TypeId::of::<T>();
        let bitmask = self.cmp_bitmasks.get(&typeid).unwrap();

        let cmp = self.components.get_mut(&typeid).unwrap();
        let cell = cmp.get_mut(entt_id).unwrap();
        *cell = None;

        self.entities[entt_id] &= *bitmask ^ u64::MAX;

        Ok(())
    }

    pub fn get_bitmask<T: Component>(&self) -> Result<u64, EcsFault> {
        let typeid = TypeId::of::<T>();
        Ok(*self.cmp_bitmasks.get(&typeid).ok_or(EcsFault::ComponentNotRegistered)?)
    }

    pub fn get_entity(&self, entt_id: EntityId) -> Result<&EntityKey, EcsFault> {
        if !self.contains_entity(entt_id) {
           return Err(EcsFault::EntityNotCreated); 
        }

        Ok(&self.entities[entt_id])
    }

    pub fn has_component<T: Component>(&self, entt_id: EntityId) -> Result<bool, EcsFault> {
        let typeid = TypeId::of::<T>();
        let bitmask = self.cmp_bitmasks.get(&typeid).ok_or(EcsFault::ComponentNotRegistered)?;

        self.has_component_util(entt_id, *bitmask)
    }

    fn has_component_util(&self, entt_id: EntityId, bitmask: u64) -> Result<bool, EcsFault> {
        if !self.contains_entity(entt_id) {
            return Err(EcsFault::EntityNotCreated);
        }
        
        Ok((self.entities[entt_id] & bitmask) == bitmask)
    }

    pub fn contains_entity(&self, entt_id: EntityId) -> bool {
        0 <= entt_id && entt_id < self.entities.len()
    }

    fn everything_ok<T: Component>(&self, entt_id: EntityId) -> Result<(), EcsFault> {
        if !self.contains_entity(entt_id) {
            return Err(EcsFault::EntityNotCreated);
        }
        let hs = self.has_component::<T>(entt_id)?;
        if !hs {
            return Err(EcsFault::NoComponentOnEntity);
        }

        Ok(())
    }
}

pub struct EntityPrototype<'a> {
    entt_id: EntityId,
    building_registry: &'a mut EntityRegistry,
    components: Option<HashMap<TypeId, ComponentAccess>>,
}

impl<'a> EntityPrototype<'a> {
    pub fn new(registry: &'a mut EntityRegistry, entt_id: EntityId) -> Self {
        EntityPrototype {
            entt_id: entt_id,
            building_registry: registry,
            components: Some(Default::default())
        }
    }

    pub fn with_component<T: Component>(mut self, data: T) -> Self {
        self.components.as_mut().unwrap().insert(data.type_id(), Rc::new(RefCell::new(data)));

        self
    }

    pub fn spawn(mut self) -> EntityId {
        let entt_id = self.entt_id;

        assert!(matches!(self.components, Some(_)));
        self.building_registry.spawn(self.entt_id, self.components.take().unwrap());

        entt_id
    }
}


#[cfg(test)]
mod tests {
    use std::{rc::Rc, any::Any, borrow::Borrow};
    use core::cell::RefMut;

    use super::EntityRegistry;

    #[derive(Debug)]
    struct Health(u32);
    struct Position(u32, u32);

    #[test]
    fn spawn_entity() {
        let mut entity_registry = EntityRegistry::new();

        entity_registry.register_component::<Health>();
        entity_registry.register_component::<Position>();

        let entt_id = entity_registry.new_entity()
            .with_component(Health(5))
            .spawn();

        assert_eq!(entt_id, 0);
        assert!(entity_registry.has_component::<Health>(entt_id).unwrap());
        assert!(!entity_registry.has_component::<Position>(entt_id).unwrap());

        let health = entity_registry.get_component_of_entity::<Health>(entt_id);
        assert!(matches!(health, Ok(_)));
        let h = Rc::clone(&health.unwrap());
        let mut t = h.borrow_mut();
        /*let h = t.downcast_mut::<Health>().unwrap();
        assert_eq!(h.0, 5);
        h.0 = 10;
        assert_eq!(h.0, 10);
        println!("{:?}", h);*/

        let health = entity_registry.get_component_of_entity::<Health>(entt_id);
        assert!(matches!(health, Ok(_)));
    }

    #[test]
    fn add_delete_component() {
        let mut entity_registry = EntityRegistry::new();

        entity_registry.register_component::<Health>();
        entity_registry.register_component::<Position>();

        let entt_id = entity_registry.new_entity()
            .with_component(Health(5))
            .spawn();

        println!("entt(h): {}", entity_registry.get_entity(entt_id).unwrap());
        assert!(!entity_registry.has_component::<Position>(entt_id).unwrap());
    
        entity_registry.replace_component_of_entity(entt_id, Position(1, 2)).unwrap();

        println!("entt(h, p): {}", entity_registry.get_entity(entt_id).unwrap());
        assert!(entity_registry.has_component::<Position>(entt_id).unwrap());
    
        entity_registry.delete_component_of_entity::<Position>(entt_id).unwrap();
        
        println!("entt(h): {}", entity_registry.get_entity(entt_id).unwrap());
        assert!(!entity_registry.has_component::<Position>(entt_id).unwrap());
    }

    #[test]
    fn temp() {

    }

}