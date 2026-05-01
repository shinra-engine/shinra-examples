use gametok_abi::InputFrame;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    A,
    D,
    W,
    S,
    Left,
    Right,
    Up,
    Down,
    J,
    K,
    N,
    Esc,
    Q,
}

pub struct Keymap {
    held: HashSet<Key>,
    swipe_pending: bool,
    quit_pending: bool,
}

impl Default for Keymap {
    fn default() -> Self {
        Self::new()
    }
}

impl Keymap {
    pub fn new() -> Self {
        Self {
            held: HashSet::new(),
            swipe_pending: false,
            quit_pending: false,
        }
    }

    pub fn on_press(&mut self, k: Key) {
        match k {
            Key::N => self.swipe_pending = true,
            Key::Esc | Key::Q => self.quit_pending = true,
            _ => {
                self.held.insert(k);
            }
        }
    }

    pub fn on_release(&mut self, k: Key) {
        self.held.remove(&k);
    }

    /// Resolve held keys into an InputFrame the game can consume.
    pub fn frame(&self) -> InputFrame {
        let h = |k| self.held.contains(&k);
        let axis = |neg, pos| (h(pos) as i32 - h(neg) as i32) as f32;
        InputFrame {
            move_x: axis(Key::A, Key::D),
            move_z: axis(Key::W, Key::S),
            rot_yaw: axis(Key::Left, Key::Right),
            rot_pitch: axis(Key::Down, Key::Up),
            scale_delta: axis(Key::K, Key::J),
        }
    }

    /// Returns true once when 'n' has been pressed since the last call.
    pub fn take_swipe_next(&mut self) -> bool {
        std::mem::take(&mut self.swipe_pending)
    }

    pub fn take_quit(&mut self) -> bool {
        std::mem::take(&mut self.quit_pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_resolves_to_zero_when_neither_held() {
        let km = Keymap::new();
        let f = km.frame();
        assert_eq!(f.move_x, 0.0);
        assert_eq!(f.move_z, 0.0);
    }

    #[test]
    fn d_held_gives_positive_move_x() {
        let mut km = Keymap::new();
        km.on_press(Key::D);
        assert_eq!(km.frame().move_x, 1.0);
    }

    #[test]
    fn a_and_d_held_cancel() {
        let mut km = Keymap::new();
        km.on_press(Key::A);
        km.on_press(Key::D);
        assert_eq!(km.frame().move_x, 0.0);
    }

    #[test]
    fn n_is_one_shot() {
        let mut km = Keymap::new();
        km.on_press(Key::N);
        assert!(km.take_swipe_next());
        assert!(!km.take_swipe_next());
    }

    #[test]
    fn n_does_not_appear_in_held() {
        let mut km = Keymap::new();
        km.on_press(Key::N);
        let f = km.frame();
        assert_eq!(f.move_x, 0.0);
        assert_eq!(f.move_z, 0.0);
    }
}
