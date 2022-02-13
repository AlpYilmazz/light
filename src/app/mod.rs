use std::collections::HashMap;

use crate::ecs::{World, system::System};


pub enum Stage {
    Startup,
    Update,
    Shutdown,
    Custom(&'static str)
}

pub struct App {
    world: World,
    stage_order: Vec<Stage>
}