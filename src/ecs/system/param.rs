use std::{ops::{Deref, DerefMut}, marker::PhantomData};

use crate::ecs::{query::{state::{Query, QueryState}, fetch::{FetchQuery, FetchState}, filter::{FilterQuery, FilterState}}, World, component::{Resource, ComponentId}, event::{Events, EventReader}};

use super::{SystemParam, SystemParamFetch, SystemParamState, System};


impl<'w, 's, Fe: 'static + FetchQuery, Fi: 'static + FilterQuery> SystemParam for Query<'w, 's, Fe, Fi> {
    type Fetch = QueryState<Fe, Fi>;
}

impl<Fe: 'static + FetchQuery, Fi: 'static + FilterQuery> SystemParamState for QueryState<Fe, Fi> {
    fn init(world: &mut World) -> Self {
        QueryState::new(world)
    }
}

impl<'w, 's, Fe: 'static + FetchQuery, Fi: 'static + FilterQuery> SystemParamFetch<'w, 's> for QueryState<Fe, Fi> {
    type Item = Query<'w, 's, Fe, Fi>;

    unsafe fn get_param(state: &'s mut Self, world: &'w World) -> Self::Item {
        Query::new(world, state)
    }
}

pub struct Res<'w, T: Resource> {
    val: &'w T
}

impl<'w, T: Resource> Deref for Res<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val
    }
}

impl<'w, T: Resource> SystemParam for Res<'w, T> {
    type Fetch = ResState<T>;
}

pub struct ResState<T: Resource> {
    resource_id: ComponentId,
    marker: PhantomData<T>,
}

impl<T: Resource> SystemParamState for ResState<T> {
    fn init(world: &mut World) -> Self {
        let id = world.add_resource::<T>();
        ResState {
            resource_id: id,
            marker: PhantomData,
        }
    }
}

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResState<T> {
    type Item = Res<'w, T>;

    unsafe fn get_param(state: &'s mut Self, world: &'w World) -> Self::Item {
        let val = &*world.get_resource_table().get_resource(&state.resource_id)
                        .unwrap_or_else(|| panic!("Resource is not stored"))
                        .cast::<T>();
        Res {
            val,
        }
    }
}

// ResMut
pub struct ResMut<'w, T: Resource> {
    val: &'w mut T
    // TODO: change tracking
}

impl<'w, T: Resource> Deref for ResMut<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val
    }
}

impl<'w, T: Resource> DerefMut for ResMut<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.val
    }
}

impl<'w, T: Resource> SystemParam for ResMut<'w, T> {
    type Fetch = ResMutState<T>;
}

pub struct ResMutState<T: Resource> {
    resource_id: ComponentId,
    marker: PhantomData<T>,
}

impl<T: Resource> SystemParamState for ResMutState<T> {
    fn init(world: &mut World) -> Self {
        let id = world.add_resource::<T>();
        ResMutState {
            resource_id: id,
            marker: PhantomData,
        }
    }
}

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResMutState<T> {
    type Item = ResMut<'w, T>;

    unsafe fn get_param(state: &'s mut Self, world: &'w World) -> Self::Item {
        let val = &mut *world.get_resource_table().get_resource_mut(&state.resource_id)
                        .unwrap_or_else(|| panic!("Resource is not stored"))
                        .cast::<T>();
        ResMut {
            val,
        }
    }
}

impl<'w, T: Resource> SystemParam for Option<Res<'w, T>> {
    type Fetch = OptionResState<T>;
}

pub struct OptionResState<T> {
    resource_id: ComponentId,
    marker: PhantomData<T>,
}

impl<T: Resource> SystemParamState for OptionResState<T> {
    fn init(world: &mut World) -> Self {
        let id = world.add_resource::<T>(); // TODO: should I add
        OptionResState {
            resource_id: id,
            marker: PhantomData,
        }
    }
}

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for OptionResState<T> {
    type Item = Option<Res<'w, T>>;

    unsafe fn get_param(state: &'s mut Self, world: &'w World) -> Self::Item {
        let val = &*world.get_resource_table().get_resource(&state.resource_id)?
                        .cast::<T>();
        Some(Res {
            val,
        })
    }
}

impl<'w, T: Resource> SystemParam for Option<ResMut<'w, T>> {
    type Fetch = OptionResMutState<T>;
}

pub struct OptionResMutState<T> {
    resource_id: ComponentId,
    marker: PhantomData<T>,
}

impl<T: Resource> SystemParamState for OptionResMutState<T> {
    fn init(world: &mut World) -> Self {
        let id = world.add_resource::<T>(); // TODO: should I add
        OptionResMutState {
            resource_id: id,
            marker: PhantomData,
        }
    }
}

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for OptionResMutState<T> {
    type Item = Option<ResMut<'w, T>>;

    unsafe fn get_param(state: &'s mut Self, world: &'w World) -> Self::Item {
        let val = &mut *world.get_resource_table().get_resource_mut(&state.resource_id)?
                        .cast::<T>();
        Some(ResMut {
            val,
        })
    }
}


pub struct Local<'a, T: Resource> {
    val: &'a mut T
}

impl<'a, T: Resource> Deref for Local<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val
    }
}

impl<'a, T: Resource> DerefMut for Local<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.val
    }
}

impl<'a, T: Resource + Default> SystemParam for Local<'a, T> {
    type Fetch = LocalState<T>;
}

pub struct LocalState<T: Resource>(T);

impl<T: Resource + Default> SystemParamState for LocalState<T> {
    fn init(world: &mut World) -> Self {
        LocalState(Default::default())
    }
}

impl<'w, 's, T: Resource + Default> SystemParamFetch<'w, 's> for LocalState<T> {
    type Item = Local<'s, T>;

    unsafe fn get_param(state: &'s mut Self, world: &'w World) -> Self::Item {
        Local {
            val: &mut state.0
        }
    }
}


// IMPORTANT NOTE:
// This is how you combine SystemParam structs to get a new valid SystemParam
// Can implement a derive macro based on this pattern
impl<'w, 's, T: Resource + Default> SystemParam for EventReader<'w, 's, T> {
    type Fetch = EventReaderState<T>;
}

pub struct EventReaderState<T: Resource> {
    events_state: ResState<Events<T>>,
    last_event_count_state: LocalState<(usize, PhantomData<T>)>,
}

impl<T: Resource + Default> SystemParamState for EventReaderState<T> {
    fn init(world: &mut World) -> Self {
        EventReaderState {
            events_state: ResState::init(world),
            last_event_count_state: LocalState::init(world),
        }
    }
}

impl<'w, 's, T: Resource + Default> SystemParamFetch<'w, 's> for EventReaderState<T> {
    type Item = EventReader<'w, 's, T>;

    unsafe fn get_param(state: &'s mut Self, world: &'w World) -> Self::Item {
        let events = ResState::get_param(&mut state.events_state, world);
        let last_event_count = LocalState::get_param(&mut state.last_event_count_state, world);
        EventReader::new(events, last_event_count)
    }
}


#[cfg(test)]
mod tests {
    
    use crate::ecs::component::{ComponentDescriptor, ComponentId};
    use crate::ecs::entity::Entity;
    use crate::ecs::query::state::Query;
    use crate::ecs::system::{FunctionSystem, IntoSystem, System, In};
    use crate::ecs::{Component, World};

    use super::{Res, Local};
    
    pub struct Transform {
        position: (f64, f64, f64),
        rotation: (f64, f64, f64),
        scale:    (f64, f64, f64),
    }
    pub struct Health(u64);
    pub struct Stamina(u64);
    pub struct Name(String);
    pub struct Dead;

    pub struct FPS(usize);

    fn query_test_func(inp: In<usize>, query: Query<&Health>) {
        println!("query system")
    }

    fn res_test_func(res: Res<FPS>) {
        println!("res system")
    }

    #[test]
    fn system_param_test() {
        let mut world = World::new();
        let mut components = [
            ComponentDescriptor::of::<Transform>(ComponentId(0)),
            ComponentDescriptor::of::<Health>(ComponentId(1)),
            ComponentDescriptor::of::<Stamina>(ComponentId(2)),
            ComponentDescriptor::of::<Name>(ComponentId(3)),
            ComponentDescriptor::of::<Dead>(ComponentId(4)),
        ];
        let table = world.get_tables_mut().new_table(&mut components);
        let row0 = table.add_row(Entity { id: 0, table_id: 0 });

        let mut system_query = query_test_func.system();
        let mut system_res = res_test_func.system();
        system_query.initialize(&mut world);
        system_res.initialize(&mut world);
        unsafe {
            system_query.run(&mut world, 0);
            system_res.run(&mut world, ());
        }
    }

    fn system_1(local: Local<usize>) {
        println!("system_1 local: {}", *local);
        *local.val += 1;
        println!("system_1 ++ local: {}", *local);
    }

    fn system_2(local: Local<usize>) {
        println!("system_2 local: {}", *local);
    }

    #[test]
    fn local_is_unique_per_system() {
        let mut world = World::new();
        let mut system_local_1 = system_1.system();
        let mut system_local_2 = system_2.system();
        system_local_1.initialize(&mut world);
        system_local_2.initialize(&mut world);
        unsafe {
            system_local_1.run(&mut world, ());
            system_local_2.run(&mut world, ());

            system_local_1.run(&mut world, ());
        }
    }

}
