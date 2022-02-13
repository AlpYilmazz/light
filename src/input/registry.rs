
use core::hash::Hash;
use std::collections::HashSet;

pub struct InputRegistry<K>
where
    K: Copy + Eq + Hash
{
    pressed: HashSet<K>,
    repeated: HashSet<K>,
    released: HashSet<K>
}

impl<K> Default for InputRegistry<K>
where
    K: Copy + Eq + Hash
{
    fn default() -> Self {
        InputRegistry {
            pressed: Default::default(),
            repeated: Default::default(),
            released: Default::default() 
        }
    }
}

impl<K> InputRegistry<K>
where
    K: Copy + Eq + Hash
{

    pub fn press(&mut self, input: K) {
        if self.pressed.contains(&input) {
            self.repeat(input);
        }
        else {
            self.pressed.insert(input);
        }
    }

    pub fn repeat(&mut self, input: K) {
        self.pressed.remove(&input);
        self.repeated.insert(input);
    }

    pub fn release(&mut self, input: K) {
        self.pressed.remove(&input);
        self.repeated.remove(&input);
        self.released.insert(input);
    }

    pub fn clear(&mut self, input: K) {
        self.pressed.remove(&input);
        self.released.remove(&input);
    }

    pub fn forget(&mut self, input: K) {
        self.pressed.remove(&input);
        self.repeated.remove(&input);
        self.released.remove(&input);
    }

    pub fn is_pressed(&self, input: K) -> bool {
        self.pressed.contains(&input)
    }

    pub fn is_repeated(&self, input: K) -> bool {
        self.repeated.contains(&input)
    }

    pub fn is_released(&self, input: K) -> bool {
        self.released.contains(&input)
    }

}