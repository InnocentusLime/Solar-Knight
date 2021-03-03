use glium::glutin;
use glutin::window::{ WindowId, Window };
use glutin::event::{ Event, WindowEvent, VirtualKeyCode, ElementState, MouseButton, KeyboardInput, DeviceEvent };
use glutin::dpi::{ PhysicalSize, PhysicalPosition };
use cgmath::Vector2;

use crate::graphics_init::SCREEN_WIDTH;

pub struct InputTracker {
    window_size : PhysicalSize<u32>,
    mouse_pos_in_window : (f32, f32),
    key_map : [bool; 500],
    mouse_button_map : [bool; 259],
    id : WindowId,
}

impl InputTracker {
    #[inline]
    fn mouse_button_to_id(x : MouseButton) -> usize {
        match x {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Other(x) => 3 + x as usize,
        }
    }

    pub fn new(window : &Window) -> Self {
        InputTracker {
            window_size : window.inner_size(),
            mouse_pos_in_window : (0.0f32, 0.0f32),
            key_map : [false; 500],
            mouse_button_map : [false; 259],
            id : window.id(),
        }
    }

    pub fn process_event(&mut self, event : &Event<()>) {
        match event {
            Event::WindowEvent { event, window_id } if *window_id == self.id => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse_pos_in_window = 
                    (
                        (position.x as f32 / self.window_size.width as f32 - 0.5f32) * 2.0f32 * SCREEN_WIDTH, 
                        -(position.y as f32 / self.window_size.height as f32 - 0.5f32) * 2.0f32
                    )
                },
                WindowEvent::MouseInput {
                    state,
                    button,
                    ..
                } => self.mouse_button_map[InputTracker::mouse_button_to_id(*button)] = (*state == ElementState::Pressed),
                _ => (),
            },
            Event::DeviceEvent {
                event,
                ..
            } => match event {
                DeviceEvent::Key (
                    KeyboardInput {
                        state,
                        virtual_keycode : Some(code),
                        ..
                    }
                ) => self.key_map[*code as usize] = (*state == ElementState::Pressed),
                _ => (),
            },
            _ => (),
        }
    }

    #[inline]
    pub fn mouse_position(&self) -> Vector2<f32> {
        use cgmath::vec2;

        vec2(self.mouse_pos_in_window.0, self.mouse_pos_in_window.1)
    }

    #[inline]
    pub fn is_key_down(&self, key : VirtualKeyCode) -> bool {
        self.key_map[key as usize]
    }

    #[inline]
    pub fn is_mouse_button_down(&self, button : MouseButton) -> bool {
        self.mouse_button_map[InputTracker::mouse_button_to_id(button)]
    }
}
