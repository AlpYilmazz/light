use crate::ecs::World;

use super::{fetch::{FetchQuery, FetchState, Fetch}, filter::{FilterQuery, FilterState, Filter}};


/// Bundle of FetchQuery and FilterQuery
pub struct QueryState<Fe: FetchQuery, Fi: FilterQuery = ()> {
    fetch_state: <Fe as FetchQuery>::State,
    filter_state: <Fi as FilterQuery>::State,
}

impl<Fe: FetchQuery, Fi: FilterQuery> QueryState<Fe, Fi> {
    pub fn new(world: &mut World) -> Self {
        QueryState {
            fetch_state: <Fe as FetchQuery>::State::init(world),
            filter_state: <Fi as FilterQuery>::State::init(world),
        }
    } 
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

pub struct QueryIter<'w, 's, Fe: FetchQuery, Fi: FilterQuery> {
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
        let fetch = <Fe as FetchQuery>::Fetch::init(world);
        let filter = <Fi as FilterQuery>::Filter::init(world);

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