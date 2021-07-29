use crate::storage::Ship;
use crate::gun::BulletSystem;
use crate::storage_traits::{ Observation, MutationObserver };
use crate::constants::VECTOR_NORMALIZATION_RANGE;

use std::time::Duration;
use std_ext::*;
use sys_api::input_tracker::InputTracker;

use glium::glutin;
use glutin::event;
use cgmath::{ InnerSpace, abs_diff_ne };

pub struct PlayerManager {
    // Quick bodge for the dash
    dash_countdown : Duration,
    use_laser : bool,
}

impl PlayerManager {
    pub fn new() -> PlayerManager {
        PlayerManager {
            use_laser : false,
            dash_countdown : <Duration as DurationExt>::my_zero(),
        }
    }

    pub fn process_event<Observer : MutationObserver>(
        &mut self, 
        storage : &mut Observation<Observer>,
        input_tracker : &InputTracker, 
        event : &glutin::event::Event<()>
    ) {
        match event {
            event::Event::DeviceEvent {
                event : event::DeviceEvent::Key (
                    event::KeyboardInput {
                        state : event::ElementState::Pressed,
                        virtual_keycode,
                        ..
                    }
                    
                ),
                ..
            } => {
                storage.mutate(0,
                    |player| {
                        if player.core.is_alive() {
                            match virtual_keycode {
                                Some(event::VirtualKeyCode::E) => {
                                    self.use_laser = !self.use_laser;
                                    player.guns.swap(0, 1);
                                    if self.use_laser { println!("Laser arsenal on"); }
                                    else { println!("Bullet + homing homies arsenal on"); }
                                },
                                Some(event::VirtualKeyCode::Space) => {
                                    if self.dash_countdown.my_is_zero() {
                                        self.dash_countdown = Duration::from_secs(3);
                                        player.engines[1].increase_speed()
                                    }
                                },
                                _ => (),
                            }
                        }
                    }
                );
            },
            _ => ()
        }
    }
    
    pub fn update<Observer : MutationObserver>(
        &mut self, 
        storage : &mut Observation<Observer>,
        input_tracker : &InputTracker, 
        bullet_sys : &mut BulletSystem, 
        dt : Duration
    ) {
        use glutin::event::{ VirtualKeyCode as Key, MouseButton };
        
        storage
        .mutate(0, |player| {
            if player.core.is_alive() {
                let mouse_pos = input_tracker.mouse_position();
                if abs_diff_ne!(mouse_pos.magnitude(), 0.0f32, epsilon = VECTOR_NORMALIZATION_RANGE) {
                    player.core.set_direction(mouse_pos.normalize());
                }
        
                if input_tracker.is_mouse_button_down(MouseButton::Right) {
                    player.engines[0].increase_speed()
                } else {
                    player.engines[0].decrease_speed()
                }
            }
        });
                
        if input_tracker.is_mouse_button_down(MouseButton::Left) {
            bullet_sys.shoot_from_gun(
                storage, 
                0, 
                0
            );
        }

        if input_tracker.is_key_down(Key::Q) && self.use_laser {
            bullet_sys.shoot_from_gun(
                storage, 
                0, 
                2
            );
        }
        
        if input_tracker.is_key_down(Key::Q) && !self.use_laser {
            bullet_sys.shoot_from_gun(
                storage, 
                0, 
                3
            );
        }
        self.dash_countdown = self.dash_countdown.saturating_sub(dt);
        storage.mutate(0, |player| {
            player.engines[1].decrease_speed()
        });
    }
}
