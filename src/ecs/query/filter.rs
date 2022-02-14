use crate::ecs::{World, storage::table::Table};

use super::AccessState;


pub trait FilterQuery {
    type State: FilterState;
    type Filter: for<'w, 's> Filter<'w, 's, State = Self::State>;
}

pub trait FilterState {
    fn init(world: &mut World) -> Self;
    fn update_access(&self, access_state: &mut AccessState);
    fn matches_table(&self, table: &Table) -> bool;
}

pub trait Filter<'w, 's> {
    type State: FilterState;

    fn init(world: &'w World) -> Self;
    fn set_table(&mut self, filter_state: &'s Self::State, table: &'w Table);
    fn matches(&self) -> bool;
}


impl FilterQuery for () {
    type State = UnitFilterState;
    type Filter = UnitFilter;
}

pub struct UnitFilterState;
impl FilterState for UnitFilterState {
    fn init(_world: &mut World) -> Self { Self }
    fn update_access(&self, _access_state: &mut AccessState) {}
    fn matches_table(&self, _table: &Table) -> bool { true }
}

pub struct UnitFilter;
impl<'w, 's> Filter<'w, 's> for UnitFilter {
    type State = UnitFilterState;
    fn init(_world: &'w World) -> Self { Self }
    fn set_table(&mut self, _filter_state: &'s Self::State, _table: &'w Table) {}
    fn matches(&self) -> bool { true }
}