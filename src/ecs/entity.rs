
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity {
    pub id: usize, // Which Row
    pub table_id: usize, // Which Table
}

impl Entity {
    #[inline]
    pub fn new(id: usize, table_id: usize) -> Self {
        Entity {
            id,
            table_id,
        }
    }
}

#[derive(Default)]
pub struct Entities {
    entity_vec: Vec<Entity>,   
}

impl Entities {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn add_entity(&mut self, id: usize, table_id: usize) -> usize {
        self.entity_vec.push(Entity::new(id, table_id));
        self.entity_vec.len() - 1
    }

    #[inline]
    pub fn get_entity(&self, index: usize) -> Option<&Entity> {
        self.entity_vec.get(index)
    }

    #[inline]
    pub fn get_entity_mut(&mut self, index: usize) -> Option<&mut Entity> {
        self.entity_vec.get_mut(index)
    }
}