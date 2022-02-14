
use std::collections::HashSet;

use super::component::ComponentId;

pub mod fetch;
pub mod filter;
pub mod state;

#[derive(Default)]
pub struct AccessState {
    read: HashSet<ComponentId>,
    write: HashSet<ComponentId>,
}

impl AccessState {
    pub fn empty() -> Self {
        Default::default()
    }

    pub fn add_read(&mut self, component_id: ComponentId) -> bool {
        self.read.insert(component_id)
    }

    pub fn add_write(&mut self, component_id: ComponentId) -> bool {
        self.write.insert(component_id)
    }

    pub fn has_read(&self, component_id: &ComponentId) -> bool {
        self.read.contains(component_id)
    }

    pub fn has_write(&self, component_id: &ComponentId) -> bool {
        self.write.contains(component_id)
    }

    pub fn has_any(&self, component_id: &ComponentId) -> bool {
        self.read.contains(component_id) || self.write.contains(component_id)
    }
}
