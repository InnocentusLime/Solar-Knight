use std::time::Duration;

use egui_glium::EguiGlium;
use cgmath::{ EuclideanSpace, Point2 };
use glium::{ glutin, Frame };

use systems::systems_core::{ Storage, get_component };
use ship_parts::player::PlayerManager;
use ship_parts::PointerTarget;
use ship_parts::earth::Earth;
use systems::ship_transform::Transform;
use systems::physics::PhysicsSystem;
use systems::collision_check::CollisionSystem;
use systems::square_map::SquareMap;
use ship_parts::ai_machine::AiMachine;
use ship_parts::render::RenderSystem;
use systems::ship_gun::BulletSystem;
use systems::ship_engine::EngineSystem;
use systems::hp_system::HpSystem;
use ship_parts::ship::{ ShipStorage, TemplateTable };
use systems::ship_attachment::AttachmentSystem;
use systems::teams::Team;
use systems::hp_system::HpInfo;
use super::{ GameState, TransitionRequest, main_menu, main_game_debug_mode };
use std_ext::*;
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;

const SPAWN_RATE : Duration = Duration::from_secs(3);
 
pub struct StateData {
    timer : Duration,
    pointer_target : PointerTarget,

    pub templates : TemplateTable,

    pub hp_sys : HpSystem,
    pub engine_sys : EngineSystem,
    pub storage : ShipStorage,
    pub player : PlayerManager,
    pub bullet_sys : BulletSystem,
    pub attach_sys : AttachmentSystem,
    pub render_sys : RenderSystem,
    pub ai_machine : AiMachine,
    pub square_map : SquareMap,
    pub phys_sys : PhysicsSystem,
    pub collision_sys : CollisionSystem,
    pub earth : Earth,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, _old : GameState) -> GameState {
        let templates = TemplateTable::new();
        let mut storage = ShipStorage::new();

        let mut square_map = SquareMap::new();
        templates.spawn_template(0, &mut storage.unlock_spawning(&mut square_map));
                
        let me =
            StateData {
                timer : SPAWN_RATE,
                earth : Earth::new(),

                pointer_target : PointerTarget::None,
        
                templates,

                hp_sys : HpSystem::new(),
                engine_sys : EngineSystem::new(),
                bullet_sys : BulletSystem::new(),
                attach_sys : AttachmentSystem::new(),
                render_sys : RenderSystem::new(ctx),
                ai_machine : AiMachine::new(),
                collision_sys : CollisionSystem::new(),
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
                .mutate(0, |player, _| {
                    if get_component::<HpInfo, _>(player).is_alive() {
                        match virtual_keycode {
                            Some(event::VirtualKeyCode::Key1) => *pointer_target = PointerTarget::None,
                            Some(event::VirtualKeyCode::Key2) => *pointer_target = PointerTarget::Sun,
                            Some(event::VirtualKeyCode::Key3) => *pointer_target = PointerTarget::Earth,
                            _ => (),
                        }
                    }
                });
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
        if let Some(player) = self.storage.get(0) {
            //assert!(player.core.team() == Team::Earth);
            if !get_component::<HpInfo, _>(player).is_alive() { 
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
        deletion_lock.retain(|x| *get_component::<Team, _>(x) == Team::Earth || get_component::<HpInfo, _>(x).hp() > 0);

        let mut mutation_lock = self.storage.unlock_mutations(&mut self.square_map);
        // Systems
        self.earth.update(dt);
        self.engine_sys.update(&mut mutation_lock);
        self.phys_sys.update(&mut mutation_lock, dt);
        self.bullet_sys.update_guns(&mut mutation_lock, dt);
        self.bullet_sys.update_bullets(&mut mutation_lock, &self.collision_sys, dt);
        self.attach_sys.update(&mut mutation_lock);
        self.ai_machine.update(&mut mutation_lock, &self.earth, &mut self.bullet_sys, dt);
        self.hp_sys.update(&mut mutation_lock);
        // Plugins
        self.player.update(&mut mutation_lock, input_tracker, &mut self.bullet_sys, dt);
        
        if let Some(player) = self.storage.get(0) {
            ctx.camera.disp = (-get_component::<Transform, _>(player).pos.to_vec()).extend(0.0f32);
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
            self.render_sys.render_pointer(frame, ctx, targets);
        }
    }
}

