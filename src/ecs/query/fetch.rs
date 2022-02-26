use std::{marker::PhantomData, ptr::NonNull};

use crate::ecs::{World, storage::table::Table, component::{Component, ComponentId}};

use super::AccessState;


pub trait FetchQuery {
    type State: FetchState;
    type Fetch: for<'w, 's> Fetch<'w, 's, State = Self::State>;
}

pub trait FetchState: /*Send + Sync + */Sized {
    fn init(world: &mut World) -> Self;
    fn update_access(&self, access_state: &mut AccessState);
    fn matches_table(&self, table: &Table) -> bool;
}

pub trait Fetch<'w, 's>: Sized {
    type Item;
    type State: FetchState;

    fn init(world: &'w World) -> Self;
    unsafe fn set_table(&mut self, fetch_state: &'s Self::State, table: &'w Table);
    unsafe fn fetch_item_from_table(&mut self, row: usize) -> Self::Item;
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
    fn init(world: &mut World) -> Self {
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

    fn init(world: &'w World) -> Self {
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


impl<T: Component> FetchQuery for &mut T {
    type State = RefMutFetchState<T>;
    type Fetch = RefMutFetch<T>;
}

pub struct RefMutFetchState<T> {
    component_id: ComponentId,
    marker: PhantomData<T>,
}

impl<T: Component> FetchState for RefMutFetchState<T> {
    fn init(world: &mut World) -> Self {
        let component_id = world.add_component::<T>();
        RefMutFetchState {
            component_id,
            marker: PhantomData,
        }
    }

    fn update_access(&self, access_state: &mut AccessState) {
        if access_state.has_any(&self.component_id) {
            panic!("Access conflict");
        }
        access_state.add_write(self.component_id.clone());
    }

    fn matches_table(&self, table: &Table) -> bool {
        table.has_column(&self.component_id)
    }
}

pub struct RefMutFetch<T> {
    table_column: NonNull<T>,
}

impl<'w, 's, T: Component> Fetch<'w, 's> for RefMutFetch<T> {
    type Item = &'w mut T;
    type State = RefMutFetchState<T>;

    fn init(world: &'w World) -> Self {
        RefMutFetch {
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
        &mut *self.table_column.as_ptr().add(row)
    }
}