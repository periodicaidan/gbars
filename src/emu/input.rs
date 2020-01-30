use piston::input::*;

use std::collections::HashMap;

enum GameBoyInputButtons {
    Up,
    Down,
    Left,
    Right,
    Start,
    Select,
    B,
    A,
    X,
    Y,
    LeftShoulder,
    RightShoulder
}

pub struct GameBoyKeymap {
    pub up: Button,
    pub down: Button,
    pub left: Button,
    pub right: Button,
    pub start: Button,
    pub select: Button,
    pub b: Button,
    pub a: Button,
    pub x: Button,
    pub y: Button,
    pub left_shoulder: Button,
    pub right_shoulder: Button
}

impl GameBoyKeymap {
    pub fn default() -> Self {
        GameBoyKeymap {
            up: Button::Keyboard(Key::Up),
            down: Button::Keyboard(Key::Down),
            left: Button::Keyboard(Key::Left),
            right: Button::Keyboard(Key::Right),
            start: Button::Keyboard(Key::Return),
            select: Button::Keyboard(Key::RShift),
            b: Button::Keyboard(Key::Z),
            a: Button::Keyboard(Key::X),
            x: Button::Keyboard(Key::S),
            y: Button::Keyboard(Key::A),
            left_shoulder: Button::Keyboard(Key::Q),
            right_shoulder: Button::Keyboard(Key::W)
        }
    }

    pub fn handle_keypress(&mut self, scancode: u32) -> Option<u8> {
        let key = Button::Keyboard(Key::from(scancode));

        match key {
            k if k == self.down => Some(0b0010_1000),
            k if k == self.up => Some(0b0010_0100),
            k if k == self.left => Some(0b0010_0010),
            k if k == self.right => Some(0b0010_0001),
            k if k == self.start => Some(0b0001_1000),
            k if k == self.select => Some(0b0001_0100),
            k if k == self.b => Some(0b0001_0010),
            k if k == self.a => Some(0b0001_0001),
            _ => None
        }
    }
}
