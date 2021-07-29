use std::time::Duration;

use egui_glium::EguiGlium;
use cgmath::abs_diff_ne;
use cgmath::{ EuclideanSpace, InnerSpace, One, Point2, vec2, point2 };
use glium::{ glutin, Frame };
use glium::texture::texture2d::Texture2d;
use glium::uniforms::SamplerWrapFunction;
use glutin::event::{ MouseButton };

use ship_parts::player::PlayerManager;
use ship_parts::PointerTarget;
use ship_parts::earth::Earth;
use ship_parts::physics::PhysicsSystem;
use ship_parts::square_map::SquareMap;
use ship_parts::ai_machine::AiMachine;
use ship_parts::render::RenderSystem;
use ship_parts::constants::VECTOR_NORMALIZATION_RANGE;
use ship_parts::{ BulletSystem, Team, Storage };
use ship_parts::attachment::AttachmentSystem;
use super::{ GameState, TransitionRequest, main_menu, main_game_debug_mode };
use std_ext::*;
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;
use loaders::load_texture_from_file;

const SPAWN_RATE : Duration = Duration::from_secs(3);
 
pub struct StateData {
    timer : Duration,
    pointer_target : PointerTarget,

    pub storage : Storage,
    pub player : PlayerManager,
    pub bullet_sys : BulletSystem,
    pub attach_sys : AttachmentSystem,
    pub render_sys : RenderSystem,
    pub ai_machine : AiMachine,
    pub square_map : SquareMap,
    pub phys_sys : PhysicsSystem,
    pub earth : Earth,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, _old : GameState) -> GameState {
        let mut storage = Storage::new();

        let mut square_map = SquareMap::new();
        storage
        .unlock_spawning(&mut square_map)
        .spawn_template(0);
                
        let mut me =
            StateData {
                timer : SPAWN_RATE,
                earth : Earth::new(),

                pointer_target : PointerTarget::None,
                bullet_sys : BulletSystem::new(),
                attach_sys : AttachmentSystem::new(),
                render_sys : RenderSystem::new(ctx),
                ai_machine : AiMachine::new(),
                square_map,
                phys_sys : PhysicsSystem::new(),
                storage,
                player : PlayerManager::new(),
            }
        ;
        GameState::MainGame(me)
    }

    pub fn process_event(&mut self, _ctx : &mut GraphicsContext, input_tracker : &InputTracker, event : &glutin::event::Event<()>) -> Option<TransitionRequest> { 
        use glutin::event;

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
                if let Some(event::VirtualKeyCode::D) = virtual_keycode {
                    if input_tracker.is_key_down(event::VirtualKeyCode::LControl) {
                        return Some(Box::new(main_game_debug_mode::StateData::init));
                    }
                }

                let pointer_target = &mut self.pointer_target;
                self.storage.unlock_mutations(&mut self.square_map)
                .mutate(0,
                    |player| {
                        if player.core.is_alive() {
                            match virtual_keycode {
                                Some(event::VirtualKeyCode::Key1) => *pointer_target = PointerTarget::None,
                                Some(event::VirtualKeyCode::Key2) => *pointer_target = PointerTarget::Sun,
                                Some(event::VirtualKeyCode::Key3) => *pointer_target = PointerTarget::Earth,
                                _ => (),
                            }
                        }
                    }
                );
            },
            _ => (),
        }

        let mut lock = self.storage.unlock_mutations(&mut self.square_map);
        self.player.process_event(&mut lock, input_tracker, event);
        
        None 
    }

    pub fn update(
        &mut self, 
        ctx : &mut GraphicsContext, 
        input_tracker : &InputTracker, 
        dt : Duration,
        _egui : &mut EguiGlium,
    ) -> Option<TransitionRequest> {
        use glutin::event::{ VirtualKeyCode as Key };

        if let Some(player) = self.storage.get(0) {
            assert!(player.core.team() == Team::Earth);
            if !player.core.is_alive() { 
                println!("You have died!");
                return Some(Box::new(main_menu::StateData::init)); 
            }
        } else { panic!("No player!"); }

        if self.timer.my_is_zero() {
            //const SPAWN_DISTANCE : f32 = 5.0f32;

            self.timer = SPAWN_RATE;
            /*
            let n = rand::random::<u16>() % 40;
            let u = (std::f32::consts::TAU / 40.0f32) * (n as f32);
            let (s, c) = u.sin_cos();
            let (x, y) = (c * SPAWN_DISTANCE, s * SPAWN_DISTANCE);
            //dbg!((x, y));
            self.battlefield.spawn(ship_parts::enemy_brute(Team::Hive, point2(x, y), vec2(0.0f32, 1.0f32)));
            */
        } else { // TODO introduce enemy limit
            self.timer = self.timer.my_saturating_sub(dt);
        }

        let mut deletion_lock = self.storage.unlock_deletion(&mut self.square_map, &mut self.attach_sys, &mut self.bullet_sys);
        deletion_lock.filter(|x| x.core.team() != Team::Earth && x.core.hp() == 0);

        let mut mutation_lock = self.storage.unlock_mutations(&mut self.square_map);

        self.earth.update(dt);
        self.phys_sys.update(&mut mutation_lock, dt);
        self.bullet_sys.update(&mut mutation_lock, dt);
        self.attach_sys.update(&mut mutation_lock);
        self.ai_machine.update(&mut mutation_lock, &self.earth, &mut self.bullet_sys, dt);

        self.player.update(&mut mutation_lock, input_tracker, &mut self.bullet_sys, dt);
        
        if let Some(player) = self.storage.get(0) {
            ctx.camera.disp = (-player.core.pos.to_vec()).extend(0.0f32);
        } else { panic!("No player!!"); }
            
        match self.pointer_target {
            PointerTarget::None => (),
            PointerTarget::Sun => self.render_sys.pointer_target = Point2 { x : 0.0f32, y : 0.0f32 },
            PointerTarget::Earth => self.render_sys.pointer_target = self.earth.pos(),
        }
 
        None
    }

    pub fn render(&self, frame : &mut Frame, ctx : &mut GraphicsContext, targets : &mut RenderTargets, _input_tracker : &InputTracker) {
        // Drawing paralaxed background
        self.render_sys.render_background(frame, ctx, targets);
        // Planets 
        self.render_sys.render_planets(frame, ctx, targets, &self.earth);
        // Bullets
        self.render_sys.render_bullets(frame, ctx, targets, &self.bullet_sys);
        // Ships
        self.render_sys.render_ships(frame, ctx, targets, &self.storage);

        if self.pointer_target != PointerTarget::None {
            self.render_sys.render_pointer(frame, ctx, targets, &self.storage);
        }
    }

    pub fn player_pos(&self) -> Point2<f32> {
        match self.storage.get(0) {
            Some(p) => p.core.pos,
            None => point2(0.0f32, 0.0f32),
        }
    }
}

