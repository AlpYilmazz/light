
pub mod registry;

use std::sync::Mutex;

use glium::glutin::event::{VirtualKeyCode, ScanCode};
use lazy_static::lazy_static;

use self::registry::InputRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum KEY {
    Physical(ScanCode),
    Virtual(VirtualKeyCode)
}

lazy_static!{ 
    static ref KEYBOARD: Mutex<InputRegistry<KEY>> = Mutex::new(Default::default());
}


#[cfg(test)]
mod tests {

    use glium::glutin::event::VirtualKeyCode;

    use super::{KEY, KEYBOARD};

    #[test]
    fn press_repeat_release_forget_key() {
        let key = KEY::Virtual(VirtualKeyCode::A);
        let mut keyboard = KEYBOARD.lock().unwrap();
        
        keyboard.press(key);
        assert!(keyboard.is_pressed(key));
        assert!(!keyboard.is_repeated(key));
        assert!(!keyboard.is_released(key));

        keyboard.press(key);
        assert!(!keyboard.is_pressed(key));
        assert!(keyboard.is_repeated(key));
        assert!(!keyboard.is_released(key));

        keyboard.release(key);
        assert!(!keyboard.is_pressed(key));
        assert!(!keyboard.is_repeated(key));
        assert!(keyboard.is_released(key));

        keyboard.forget(key);
        assert!(!keyboard.is_pressed(key));
        assert!(!keyboard.is_repeated(key));
        assert!(!keyboard.is_released(key));
    }

}