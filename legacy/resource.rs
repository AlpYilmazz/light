use std::{collections::HashMap, any::{TypeId, Any}, marker::PhantomData};
use core::ops::Deref;

use super::{system::{SystemParam, SystemParamFetch}, World, Resource};


#[derive(Default)]
pub struct ResourceRegistry {
    resources: HashMap<TypeId, Box<dyn Any>>
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn save<T: Any>(&mut self, data: T) {
        self.resources.insert(data.type_id(), Box::new(data));
    }

    pub fn get_ref<T: Any>(&self) -> Option<&T> {
        let typeid = TypeId::of::<T>();
        if let Some(data) = self.resources.get(&typeid) {
            return data.downcast_ref::<T>();
        }

        None
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let typeid = TypeId::of::<T>();
        if let Some(data) = self.resources.get_mut(&typeid) {
            return data.downcast_mut::<T>();
        }

        None
    }

    pub fn delete<T: Any>(&mut self) {
        self.resources.remove(&TypeId::of::<T>());
    }
}

pub struct Res<'w, T> {
    val: &'w T
}

impl<'w, T> Res<'w, T> {
    pub fn new(val: &'w T) -> Self {
        Res {
            val: val
        }
    }
}

impl<'w, T> Deref for Res<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val
    }
}

pub struct ResFetch<T> {
    marker: PhantomData<T>
}

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResFetch<T> {
    type Item = Res<'w, T>;

    fn get_param(world: &'w World) -> Self::Item {
        Res::<T>::new(world.get_resource_ref::<T>().unwrap())
    }
}

impl<'w, T: Resource> SystemParam for Res<'w, T> {
    type Fetch = ResFetch<T>;
}


#[cfg(test)]
mod tests {
    use crate::ecs::{World, system::{SystemParam, SystemParamFetch, IntoSystem, System}};

    use super::Res;

    pub struct GameOptions {
        pub fps: u32
    }

    fn print_fps(game_options: Res<GameOptions>) {
        println!("FPS: {}", game_options.fps);
    }

    #[test]
    fn deref_for_res() {
        let game_options = GameOptions { fps: 60 };
        let res = Res::new(&game_options);
        assert_eq!(res.fps, 60);
    }

    #[test]
    fn system_param_res() {
        let game_options = GameOptions { fps: 60 };
        let mut world = World::new();
        world.save_resource(game_options);

        let res = <<Res<GameOptions> as SystemParam>::Fetch as SystemParamFetch>::get_param(&world);
        assert_eq!(res.fps, 60);
    }

    #[test]
    fn system_param_function_res_param() {
        let game_options = GameOptions { fps: 60 };
        let mut world = World::new();
        world.save_resource(game_options);

        print_fps.system().run(&world, ());
    }

}