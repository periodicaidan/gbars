use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as GWindow;
use opengl_graphics::{OpenGL, GlGraphics};

use super::gb_types::*;

pub struct App {
    pub gl: GlGraphics,
    pub screen: ScreenBuffer
}

impl App {
    fn update(&mut self, args: &UpdateArgs) {
        // Update the screen and read the screen buffer from ROM
    }

    fn render(&mut self, args: &RenderArgs) {
        //
    }
}