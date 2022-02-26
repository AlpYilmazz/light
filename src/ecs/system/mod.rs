use std::marker::PhantomData;

use super::World;

pub mod param;


pub trait System {
    type In;
    type Out;

    fn initialize(&mut self, world: &mut World);

    /// # Safety
    /// call `System::initialize` first
    unsafe fn run(&mut self, world: &World, input: Self::In) -> Self::Out;
}

pub trait IntoSystem<In, Out, Marker> {
    type Sys: System<In = In, Out = Out>;

    fn system(self) -> Self::Sys;
}

pub trait SystemParamState {
    fn init(world: &mut World) -> Self;
}

pub trait SystemParamFetch<'w, 's>: SystemParamState {
    type Item: SystemParam;
    unsafe fn get_param(state: &'s mut Self, world: &'w World) -> Self::Item;
}

pub trait SystemParam {
    type Fetch: for<'w, 's> SystemParamFetch<'w, 's>;
}

pub trait SystemParamFunction<In, Out, Param: SystemParam, Marker> {
    unsafe fn run(&mut self, world: &World, state: &mut <Param as SystemParam>::Fetch, input: In) -> Out;
}

pub struct FunctionSystem<In, Out, Param, Marker, F>
where
    Param: SystemParam,
    F: SystemParamFunction<In, Out, Param, Marker>
{
    sfunc: F,
    param_state: Option<<Param as SystemParam>::Fetch>,
    marker: PhantomData<fn() -> (In, Out, Marker)> // For it to own In, Out, Marker ???
    // The purpose of the generic Marker is to allow
    // having colliding trait implementations
    // Collision is solved by differentiating in terms of Marker
}

impl<In, Out, Param: SystemParam, Marker, F> System for FunctionSystem<In, Out, Param, Marker, F>
where
    Param: SystemParam,
    F: SystemParamFunction<In, Out, Param, Marker>
{
    type In = In;
    type Out = Out;

    fn initialize(&mut self, world: &mut World) {
        self.param_state = Some(<Param as SystemParam>::Fetch::init(world));
    }

    unsafe fn run(&mut self, world: &World, input: Self::In) -> Self::Out {
        println!("Hello, I am FunctionSystem");

        self.sfunc.run(world, self.param_state.as_mut().unwrap(), input)
    }
}

pub struct IsFunctionSystem;

impl<In, Out, Param, Marker, F> IntoSystem<In, Out, (IsFunctionSystem, Param, Marker)> for F
where
    Param: SystemParam,
    F: SystemParamFunction<In, Out, Param, Marker>
{
    type Sys = FunctionSystem<In, Out, Param, Marker, F>;
    
    fn system(self) -> Self::Sys {
        FunctionSystem {
            sfunc: self,
            param_state: None,
            marker: PhantomData
        }
    }
}

pub struct In<Inp> {
    pub data: Inp
}
pub struct InputMarker;

// Example Impl, Should implement this FnMut of varying number of inputs (SystemParam tuples)
impl<Out, Param: SystemParam, F> SystemParamFunction<(), Out, Param, ()> for F
where
    F: FnMut(Param) -> Out
        + FnMut(<<Param as SystemParam>::Fetch as SystemParamFetch>::Item) -> Out,
{
    unsafe fn run(&mut self, world: &World, state: &mut <Param as SystemParam>::Fetch, _input: ()) -> Out {
        let p 
                = <<Param as SystemParam>::Fetch as SystemParamFetch>::get_param(state, world);
        self(p)
    }
}

impl<Inp, Out, Param: SystemParam, F> SystemParamFunction<Inp, Out, Param, InputMarker> for F
where
    F: FnMut(In<Inp>, Param) -> Out
        + FnMut(In<Inp>, <<Param as SystemParam>::Fetch as SystemParamFetch>::Item) -> Out,
{
    unsafe fn run(&mut self, world: &World, state: &mut <Param as SystemParam>::Fetch, input: Inp) -> Out {
        let p 
                = <<Param as SystemParam>::Fetch as SystemParamFetch>::get_param(state, world);
        self(In{data: input}, p)
    }
}

/*impl<Out, Param, F> IntoSystem<(), Out, Param> for F
where
    Param: SystemParam,
    F: FnMut(&Param) -> Out
{
    type Sys = FunctionSystem<(), Out, Param, F>;

    fn system(self) -> Self::Sys {
        FunctionSystem {
            func: self,
            param: None,
            marker: PhantomData,
        }
    }
}*/


#[cfg(test)]
mod tests {
    use crate::ecs::World;

    use super::{System, SystemParam, IntoSystem, SystemParamFetch, In};

    struct FPS (u8);
    struct PlayerCount (u32);
    struct Name (String);
    struct Age (u32);
    struct Person {}
    struct Dead {}


    /*fn system_prototype(
        res_fps: Res<&FPS>,
        res_mut: ResMut<&PlayerCount>,
        entity1: EntityAccess,
        query: Query<(&Name, &mut Age), (With<&Person>, Without<&Dead>)>)
    {

    }*/

    /*pub struct u32Fetch;

    impl SystemParamFetch for u32Fetch {
        type Item = u32;

        fn get_param() -> Self::Item {
            345
        }
    }

    impl SystemParam for u32 {
        type Fetch = u32Fetch;
    }

    fn param_system(a: u32) {
        println!("No, I am Function, with SystemParam(s): ['{}']", a);
    }

    fn input_param_system(input: In<&str>, a: u32) {
        println!("No, I am Function, with In: '{}' and SystemParam(s): ['{}']", input.data, a);
    }

    #[test]
    fn temp1() {
        let world = World::new();
        param_system.system().run(&world, ());
        let input = "Hi, I am input";
        input_param_system.system().run(&world, input);
    }*/

}