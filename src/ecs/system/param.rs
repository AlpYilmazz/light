use crate::ecs::{query::{state::{Query, QueryState}, fetch::{FetchQuery, FetchState}, filter::{FilterQuery, FilterState}}, World};

use super::{SystemParam, SystemParamFetch, SystemParamState};


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

    fn get_param(state: &'s Self, world: &'w World) -> Self::Item {
        Query::new(world, state)
    }
}

use super::{FunctionSystem, IntoSystem, System};
use crate::ecs::Component;

pub struct Health(u64);

fn query_test_func(query: Query<&Health>) {

}

fn test() {
    let mut world = World::new();
    let mut system = query_test_func.system();
    system.initialize(&mut world);
    unsafe {
        system.run(&mut world, ());
    }
}