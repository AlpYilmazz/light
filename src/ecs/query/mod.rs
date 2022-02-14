
use std::{collections::HashSet, marker::PhantomData, ptr::NonNull};

use super::{World, component::{Component, ComponentId}, storage::table::Table};

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

pub trait FetchQuery {
    type State: FetchState;
    type Fetch: for<'w, 's> Fetch<'w, 's, State = Self::State>;
}

pub trait FetchState: /*Send + Sync + */Sized {
    fn new(world: &mut World) -> Self;
    fn update_access(&self, access_state: &mut AccessState);
    fn matches_table(&self, table: &Table) -> bool;
}

pub trait Fetch<'w, 's>: Sized {
    type Item;
    type State: FetchState;

    fn new(world: &'w World) -> Self;
    unsafe fn set_table(&mut self, fetch_state: &'s Self::State, table: &'w Table);
    unsafe fn fetch_item_from_table(&mut self, row: usize) -> Self::Item;
}

pub trait FilterQuery {
    type State: FilterState;
    type Filter: for<'w, 's> Filter<'w, 's, State = Self::State>;
}

pub trait FilterState {
    fn new(world: &mut World) -> Self;
    fn update_access(&self, access_state: &mut AccessState);
    fn matches_table(&self, table: &Table) -> bool;
}

pub trait Filter<'w, 's> {
    type State: FilterState;

    fn new(world: &'w World) -> Self;
    fn set_table(&mut self, filter_state: &'s Self::State, table: &'w Table);
    fn matches(&self) -> bool;
}

impl<T: Component> FetchQuery for &T {
    type State = RefFetchState<T>;
    type Fetch = RefFetch<T>;
}

pub struct RefFetchState<T> {
    component_id: ComponentId,
    marker: PhantomData<T>,
}

impl<T: Component> FetchState for RefFetchState<T> {
    fn new(world: &mut World) -> Self {
        let component_id = world.add_component::<T>();
        RefFetchState {
            component_id,
            marker: PhantomData,
        }
    }

    fn update_access(&self, access_state: &mut AccessState) {
        if access_state.has_write(&self.component_id) {
            panic!("Access conflict");
        }
        access_state.add_read(self.component_id.clone());
    }

    fn matches_table(&self, table: &Table) -> bool {
        table.has_column(&self.component_id)
    }
}

pub struct RefFetch<T> {
    table_column: NonNull<T>,
}

impl<'w, 's, T: Component> Fetch<'w, 's> for RefFetch<T> {
    type Item = &'w T;

    type State = RefFetchState<T>;

    fn new(world: &'w World) -> Self {
        RefFetch {
            table_column: NonNull::dangling(),
        }
    }

    unsafe fn set_table(&mut self, fetch_state: &'s Self::State, table: &'w Table) {
        self.table_column = table.get_column(&fetch_state.component_id).unwrap()
                                .get_ptr().cast::<T>();
    }

    /// # Safety
    /// - `self.table_column` should not be dangling
    /// - call `set_table` method before calling this method
    /// - row should be valid address for `self.table_column`
    unsafe fn fetch_item_from_table(&mut self, row: usize) -> Self::Item {
        &*self.table_column.as_ptr().add(row)
    }
}

impl FilterQuery for () {
    type State = UnitFilterState;
    type Filter = UnitFilter;
}

pub struct UnitFilterState;
impl FilterState for UnitFilterState {
    fn new(_world: &mut World) -> Self { Self }
    fn update_access(&self, _access_state: &mut AccessState) {}
    fn matches_table(&self, _table: &Table) -> bool { true }
}

pub struct UnitFilter;
impl<'w, 's> Filter<'w, 's> for UnitFilter {
    type State = UnitFilterState;
    fn new(_world: &'w World) -> Self { Self }
    fn set_table(&mut self, _filter_state: &'s Self::State, _table: &'w Table) {}
    fn matches(&self) -> bool { true }
}


/// Bundle of FetchQuery and FilterQuery
pub struct QueryState<Fe: FetchQuery, Fi: FilterQuery = ()> {
    fetch_state: <Fe as FetchQuery>::State,
    filter_state: <Fi as FilterQuery>::State,
}

/// Actual SystemParam Query
pub struct Query<'w, 's, Fe: FetchQuery, Fi: FilterQuery = ()> {
    world: &'w World,
    query_state: &'s QueryState<Fe, Fi>,
}

// query: Query<(&Name, &Age), With<&Person>>

impl<'w, 's, Fe: FetchQuery, Fi: FilterQuery> Query<'w, 's, Fe, Fi> {
    pub fn new(world: &'w World, query_state: &'s QueryState<Fe, Fi>) -> Self {
        Query {
            world,
            query_state,
        }
    }

    pub fn iter(&self) -> QueryIter<'w, 's, Fe, Fi> {
        QueryIter::new(self.world, self.query_state)
    }
}

pub struct QueryIter<'w, 's, Fe, Fi>
where
    Fe: FetchQuery,
    Fi: FilterQuery,
{
    world: &'w World,
    query_state: &'s QueryState<Fe, Fi>,
    fetch: <Fe as FetchQuery>::Fetch,
    filter: <Fi as FilterQuery>::Filter,
    matched_table_ids: Vec<usize>, // TODO: implement something
    current_table_index: usize,
    current_row: usize,
    current_table_len: usize,
}

impl<'w, 's, Fe: FetchQuery, Fi: FilterQuery> QueryIter<'w, 's, Fe, Fi> {
    pub fn new(world: &'w World, query_state: &'s QueryState<Fe, Fi>) -> Self {
        let fetch = <Fe as FetchQuery>::Fetch::new(world);
        let filter = <Fi as FilterQuery>::Filter::new(world);

        QueryIter {
            world,
            query_state,
            fetch,
            filter,
            matched_table_ids: Vec::new(),
            current_table_index: 0,
            current_row: 0,
            current_table_len: 0,
        }
    }
}

impl<'w, 's, Fe: FetchQuery, Fi: FilterQuery> Iterator for QueryIter<'w, 's, Fe, Fi> {
    type Item = <<Fe as FetchQuery>::Fetch as Fetch<'w, 's>>::Item;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        unsafe {
            loop {
                // enters the if in the first iteration
                if self.current_row >= self.current_table_len {
                    let table_id = self.matched_table_ids.get(self.current_table_index)?;
                    let table = self.world.get_tables().get_table(*table_id)?;

                    self.current_table_index += 1;

                    // I dont think this ever happens
                    // or at least should prevent it from happening
                    // by not having it in self.matched_table_ids
                    if table.is_empty() {
                        continue;
                    }
                    
                    self.fetch.set_table(&self.query_state.fetch_state, table);
                    self.filter.set_table(&self.query_state.filter_state, table);

                    self.current_row = 0;
                    self.current_table_len = table.len();
                }

                if !self.filter.matches() {
                    println!("Filter does not match");
                    self.current_row += 1;
                    continue;
                }

                let item = self.fetch.fetch_item_from_table(self.current_row);
                self.current_row += 1;

                return Some(item);
            }
        }
    }
}