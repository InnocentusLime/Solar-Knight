use systems::teams::Team;
use systems::ship_transform::Transform;
use systems::ship_engine::Engines;
use systems::hp_system::HpInfo;
use systems::ship_gun::{ BulletSystem, Guns };
use systems::systems_core::{ MutationObserver, Observation, Storage, ComponentAccess, get_component_mut, get_component };
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

    pub fn process_event<Host, Observer>(
        &mut self, 
        storage : &mut Observation<Observer, Host>,
        _input_tracker : &InputTracker, 
        event : &glutin::event::Event<()>
    ) 
    where
        Host : Storage,
        Host::Object : ComponentAccess<Engines> + ComponentAccess<Guns> + ComponentAccess<HpInfo>,
        Observer : MutationObserver<Host>,
    {
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
                storage.mutate(0, |player, _| {
                    if get_component::<HpInfo, _>(player).is_alive() {
                        match virtual_keycode {
                            Some(event::VirtualKeyCode::E) => {
                                self.use_laser = !self.use_laser;
                                get_component_mut::<Guns, _>(player).guns.swap(0, 1);
                                if self.use_laser { println!("Laser arsenal on"); }
                                else { println!("Bullet + homing homies arsenal on"); }
                            },
                            Some(event::VirtualKeyCode::Space) => {
                                if self.dash_countdown.my_is_zero() {
                                    self.dash_countdown = Duration::from_secs(3);
                                    get_component_mut::<Engines, _>(player).engines[1].increase_speed()
                                }
                            },
                            _ => (),
                        }
                    }
                });
            },
            _ => ()
        }
    }
    
    pub fn update<Host, Observer>(
        &mut self, 
        storage : &mut Observation<Observer, Host>,
        input_tracker : &InputTracker, 
        bullet_sys : &mut BulletSystem, 
        dt : Duration
    ) 
    where
        Host : Storage,
        Host::Object : ComponentAccess<Engines> + ComponentAccess<HpInfo> + ComponentAccess<Guns> + ComponentAccess<Transform> + ComponentAccess<Team>,
        Observer : MutationObserver<Host>,
    {
        use glutin::event::{ VirtualKeyCode as Key, MouseButton };
        pub const VECTOR_NORMALIZATION_RANGE : f32 = 0.0001f32;

        storage
        .mutate(0, |player, _| {
            if get_component::<HpInfo, _>(player).is_alive() {
                let mouse_pos = input_tracker.mouse_position();
                if abs_diff_ne!(mouse_pos.magnitude(), 0.0f32, epsilon = VECTOR_NORMALIZATION_RANGE) {
                    get_component_mut::<Transform, _>(player).set_direction(mouse_pos.normalize());
                }
       
                let main_engine = get_component_mut::<Engines, _>(player).engines.get_mut(0).unwrap();
                if input_tracker.is_mouse_button_down(MouseButton::Right) {
                    main_engine.increase_speed()
                } else {
                    main_engine.decrease_speed()
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
        storage.mutate(0, |player, _| {
            get_component_mut::<Engines, _>(player).engines[1].decrease_speed()
        });
    }
}
