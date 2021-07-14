use std::time::Duration;

use egui_glium::EguiGlium;
use cgmath::abs_diff_ne;
use cgmath::{ EuclideanSpace, InnerSpace, One, Point2, vec2, point2 };
use glium::{ glutin, Frame };
use glium::texture::texture2d::Texture2d;
use glium::uniforms::SamplerWrapFunction;
use glutin::event::{ MouseButton };

use ship_aprts::square_map::SquareMap;
use ship_parts::ai_machine::AiMachine;
use ship_parts::render::RenderSystem;
use ship_parts::constants::VECTOR_NORMALIZATION_RANGE;
use ship_parts::{ BulletSystem, Team, Battlefield };
use ship_parts::attachment::AttachmentSystem;
use super::{ GameState, TransitionRequest, main_menu, main_game_debug_mode };
use std_ext::*;
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;
use crate::loaders::texture_load_from_file;

const SPAWN_RATE : Duration = Duration::from_secs(3);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PointerTarget {
    None,
    Sun,
    Earth,
}
    
#[inline]
pub fn point_at(from : Point2<f32>, at : Point2<f32>) -> Option<Point2<f32>> {
    use sys_api::graphics_init::SCREEN_WIDTH;

    let v = at - from;
    let x = (v.x / v.y.abs()).clamp(-SCREEN_WIDTH, SCREEN_WIDTH);
    let y = (SCREEN_WIDTH * v.y / v.x.abs()).clamp(-1.0f32, 1.0f32);
    let pointer_v = vec2(x, y);

    if pointer_v.magnitude2() > v.magnitude2() { None }
    else { Some(<Point2<f32> as EuclideanSpace>::from_vec(pointer_v)) }
}

fn load_environment_params() -> (f32, f32) {
    use std::fs::File;
    use std::io::{ BufReader, BufRead };

    let f = File::open("params.txt").expect("Params file not found");
    let mut f = BufReader::new(f);

    let mut line = String::new();
    f.read_line(&mut line).expect("Failed to read friction expression");
    let friction = line.trim().parse().expect("Failed to parse player mass expression");
    line.clear();
    f.read_line(&mut line).expect("Failed to read player mass expression");
    let mass = line.trim().parse().expect("Failed to parse player mass expression");

    (friction, mass)
}

pub struct StateData {
    pub sun_texture : Texture2d,
    pub earth_texture : Texture2d,
    pub basic_enemy_ship_texture : Texture2d,
    pub background_texture : Texture2d,
    pub player_bullet_texture : Texture2d,
    //player_dash_trace_texture : Texture2d,

    pub battlefield : Battlefield,

    timer : Duration,
    pointer_target : PointerTarget,
    bullet_sys : BulletSystem,
    pub attach_sys : AttachmentSystem,
    pub render_sys : RenderSystem,
    pub ai_machine : AiMachine,
    pub square_map : SquareMap,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, _old : GameState) -> GameState {
        let sun_texture = texture_load_from_file(&ctx.display, "textures/sun.png").unwrap();
        let earth_texture = texture_load_from_file(&ctx.display, "textures/earth.png").unwrap();
        let basic_enemy_ship_texture = texture_load_from_file(&ctx.display, "textures/basic_enemy_ship.png").unwrap();
        let player_bullet_texture = texture_load_from_file(&ctx.display, "textures/player_bullet.png").unwrap();
        //let player_dash_trace_texture = texture_load_from_file(&ctx.display, "textures/player_dash_trace.png").unwrap();
        let background_texture = texture_load_from_file(&ctx.display, "textures/background_game.png").unwrap();

        let mut battlefield = Battlefield::new();

        battlefield.spawn_template(0);
                
        let mut me =
            StateData {
                sun_texture,
                background_texture,
                earth_texture,
                basic_enemy_ship_texture,
                player_bullet_texture,
                //player_dash_trace_texture,

                timer : SPAWN_RATE,
                pointer_target : PointerTarget::None,
                bullet_sys : BulletSystem::new(),
                attach_sys : AttachmentSystem::new(),
                render_sys : RenderSystem::new(ctx),
                ai_machine : AiMachine::new(),
                square_map : SquareMap::new(),
                battlefield,
            }
        ;
        me.load_params();
        GameState::MainGame(me)
    }

    fn load_params(&mut self) {
        use ship_parts::storage_traits::FRICTION_KOEFF;

        let (friction, player_mass) = load_environment_params();

        /*
        unsafe { FRICTION_KOEFF = friction; }
        match self.battlefield.get_mut_downcasted::<PlayerShip>(0) {
            Some(player) => player.core.mass = player_mass,
            _ => (),
        }
        */
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
                match self.battlefield.get_mut(0) {
                    Some(player) if player.core.is_alive() => {
                        match virtual_keycode {
                            Some(event::VirtualKeyCode::U) => self.load_params(),
                            Some(event::VirtualKeyCode::Key1) => self.pointer_target = PointerTarget::None,
                            Some(event::VirtualKeyCode::Key2) => self.pointer_target = PointerTarget::Sun,
                            Some(event::VirtualKeyCode::Key3) => self.pointer_target = PointerTarget::Earth,
                            Some(event::VirtualKeyCode::D) if input_tracker.is_key_down(event::VirtualKeyCode::LControl) => 
                                return Some(Box::new(main_game_debug_mode::StateData::init))
                            ,
                            _ => (),
                        }
                    },
                    _ => (),
                }
            },
            _ => (),
        }

        None 
    }

    pub fn update(
        &mut self, 
        ctx : &mut GraphicsContext, 
        input_tracker : &InputTracker, 
        dt : Duration,
        _egui : &mut EguiGlium,
    ) -> Option<TransitionRequest> {
        if let Some(player) = self.battlefield.get(0) {
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

        match self.battlefield.get_mut(0) {
            Some(player) if player.core.is_alive() => {
                let mouse_pos = input_tracker.mouse_position();
                if abs_diff_ne!(mouse_pos.magnitude(), 0.0f32, epsilon = VECTOR_NORMALIZATION_RANGE) {
                    player.core.set_direction(mouse_pos.normalize());
                }
        
                if input_tracker.is_mouse_button_down(MouseButton::Right) {
                    player.engines[0].increase_speed()
                } else {
                    player.engines[0].decrease_speed()
                }
            },
            _ => (),
        }
                
        if input_tracker.is_mouse_button_down(MouseButton::Left) {
            self.bullet_sys.shoot_from_gun(&mut self.battlefield, 0, 0);
        }


        self.battlefield.update(dt);
        self.bullet_sys.update(&mut self.battlefield, dt);
        self.attach_sys.update(&mut self.battlefield);
        self.ai_machine.update(&mut self.battlefield, &mut self.bullet_sys, dt);
       
        if let Some(player) = self.battlefield.get(0) {
            ctx.camera.disp = (-player.core.pos.to_vec()).extend(0.0f32);
        } else { panic!("No player!!"); }
         
        /*
        let sun_box = mesh_of_sprite(Matrix4::one(), vec2(0.4f32, 0.4f32));
        let player_box = 
            mesh_of_sprite(
                Matrix4::from_translation(self.player_position.extend(0.0f32)) * Matrix4::from_angle_z(self.player_rotation),
                vec2(0.1f32, 0.1f32)
            )
        ;
        */

        None
    }

    pub fn render(&self, frame : &mut Frame, ctx : &mut GraphicsContext, targets : &mut RenderTargets, _input_tracker : &InputTracker) {
        use glium::Surface;
        use cgmath::Matrix4;

        use sys_api::graphics_utils::{ draw_sprite, draw_instanced_sprite };

        let vp = ctx.build_projection_view_matrix();

        // Drawing paralaxed background
        use sys_api::graphics_init::SCREEN_WIDTH;
        let cam = -ctx.camera.disp.truncate(); 
        let picker = vec2((0.2f32 * cam.x / SCREEN_WIDTH) % 1.0f32, (0.2f32 * cam.y) % 1.0f32);
        draw_sprite(
            ctx, frame, 
            Matrix4::one(),
            (picker.x, picker.y, 1.0f32, 1.0f32),
            self.background_texture.sampled().wrap_function(SamplerWrapFunction::Repeat), 
            Some(ctx.viewport())
        );
        let picker = vec2((0.05f32 * cam.x / SCREEN_WIDTH - 0.5f32) % 1.0f32, (0.05f32 * cam.y + 0.03f32) % 1.0f32);
        draw_sprite(
            ctx, frame, 
            Matrix4::one(),
            (picker.x, picker.y, 1.0f32, 1.0f32),
            self.background_texture.sampled().wrap_function(SamplerWrapFunction::Repeat), 
            Some(ctx.viewport())
        );
        // Planets 
        draw_sprite(
            ctx, frame, 
            vp * self.battlefield.earth.model_mat(), 
            (0.0f32, 0.0f32, 1.0f32, 1.0f32),
            self.earth_texture.sampled(), 
            Some(ctx.viewport())
        );
        draw_sprite(
            ctx, frame, 
            vp * Matrix4::from_nonuniform_scale(0.6f32, 0.6f32, 1.0f32), 
            (0.0f32, 0.0f32, 1.0f32, 1.0f32),
            self.sun_texture.sampled(), 
            Some(ctx.viewport())
        );

        // Orphaning technique
        // https://stackoverflow.com/questions/43036568/when-should-glinvalidatebufferdata-be-used
        // https://www.khronos.org/opengl/wiki/Buffer_Object_Streaming
        // https://community.khronos.org/t/vbos-strangely-slow/60109
        
        ctx.bullet_buffer.invalidate();
        self.bullet_sys.fill_buffer(&mut ctx.bullet_buffer);
        draw_instanced_sprite(ctx, frame, &ctx.bullet_buffer, vp, self.player_bullet_texture.sampled(), Some(ctx.viewport()));

        self.render_sys
        .render_ships(
            frame,
            ctx,
            targets,
            &self.battlefield
        );

        let pointer_target = 
            match self.pointer_target {
                PointerTarget::None => None,
                PointerTarget::Sun => Some(Point2 { x : 0.0f32, y : 0.0f32 }),
                PointerTarget::Earth => Some(self.battlefield.earth.pos()),
            }
        ;

        let pointer = 
            pointer_target
            .and_then(
                |x| 
                self.battlefield
                .get(0)
                .and_then(|y| point_at(y.core.pos, x))
            )
        ;

        match pointer {
            Some(pointer) => {
                let model_mat = ctx.proj_mat * Matrix4::from_translation(pointer.to_vec().extend(0.0f32)) * Matrix4::from_nonuniform_scale(0.1f32, 0.1f32, 1.0f32);
                draw_sprite(
                    ctx, frame, 
                    model_mat, 
                    (0.0f32, 0.0f32, 1.0f32, 1.0f32),
                    self.basic_enemy_ship_texture.sampled(), 
                    Some(ctx.viewport())
                )
            },
            None => (),
        }
    }

    pub fn player_pos(&self) -> Point2<f32> {
        match self.battlefield.get(0) {
            Some(p) => p.core.pos,
            None => point2(0.0f32, 0.0f32),
        }
    }
}

