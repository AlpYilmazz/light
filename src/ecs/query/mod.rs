
use super::World;


pub trait Fetch<'w> {
    type Item;

    fn new() -> Self;
    fn fetch_item(&mut self) -> Self::Item;
}
pub trait Filter {

    fn new() -> Self;
    fn get_filter(&self) -> u64;
}


impl<'w, F0, F1> Fetch<'w> for (F0, F1)
where
    F0: Fetch<'w>,
    F1: Fetch<'w>
{
    type Item = (F0::Item, F1::Item);
    
    fn new() -> Self {
        (<F0 as Fetch>::new(), <F1 as Fetch>::new())
    }

    fn fetch_item(&mut self) -> <Self as Fetch<'w>>::Item {
        let (f0, f1) = self;
        (f0.fetch_item(), f1.fetch_item())
    }
}

pub struct Query<'w, Fe: Fetch<'w>, Fi: Filter> {
    world: &'w World,
    fetch: Fe,
    filter: Fi,
}

// query: Query<(&Name, &Age), With<&Person>>

impl<'w, Fe: Fetch<'w>, Fi: Filter> Query<'w, Fe, Fi> {
    pub fn new(world: &'w World) -> Self {
        Query {
            world: world,
            fetch: <Fe as Fetch>::new(),
            filter: <Fi as Filter>:: new(),
        }
    }

    pub fn iter(&self) -> QueryIter<'w, Fe, Fi> {
        QueryIter::new(self.world)
    }
}

pub struct QueryIter<'w, Fe: Fetch<'w>, Fi: Filter> {
    world: &'w World,
    fetch: Fe,
    filter: Fi,
}

impl<'w, Fe: Fetch<'w>, Fi: Filter> QueryIter<'w, Fe, Fi> {
    pub fn new(world: &'w World) -> Self {
        QueryIter {
            world: world,
            fetch: <Fe as Fetch>::new(),
            filter: <Fi as Filter>:: new(),
        }
    }
}

impl<'w, Fe, Fi> Iterator for QueryIter<'w, Fe, Fi>
where
    Fe: Fetch<'w>,
    Fi: Filter
{
    type Item = <Fe as Fetch<'w>>::Item;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let filter_mask = self.filter.get_filter();
        Some(self.fetch.fetch_item())
    }
}