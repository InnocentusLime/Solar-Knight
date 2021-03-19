use std::time::Duration;

use cgmath::{ EuclideanSpace, InnerSpace, One, Point2, vec2, point2 };
use glium::glutin;
use glium::texture::texture2d::Texture2d;
use glium::uniforms::SamplerWrapFunction;
use glutin::event::{ MouseButton };

use ship_parts::{ BulletSystem, Team, PlayerShip, Battlefield };
use super::{ GameState, TransitionRequest, main_menu };
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
    player_ship_texture : Texture2d,
    sun_texture : Texture2d,
    earth_texture : Texture2d,
    basic_enemy_ship_texture : Texture2d,
    background_texture : Texture2d,
    player_bullet_texture : Texture2d,
    //player_dash_trace_texture : Texture2d,

    battlefield : Battlefield,

    timer : Duration,
    pointer_target : PointerTarget,
    bullet_sys : BulletSystem,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, _old : GameState) -> GameState {
        let sun_texture = texture_load_from_file(&ctx.display, "textures/sun.png").unwrap();
        let player_ship_texture = texture_load_from_file(&ctx.display, "textures/player_ship.png").unwrap();
        let earth_texture = texture_load_from_file(&ctx.display, "textures/earth.png").unwrap();
        let basic_enemy_ship_texture = texture_load_from_file(&ctx.display, "textures/basic_enemy_ship.png").unwrap();
        let player_bullet_texture = texture_load_from_file(&ctx.display, "textures/player_bullet.png").unwrap();
        //let player_dash_trace_texture = texture_load_from_file(&ctx.display, "textures/player_dash_trace.png").unwrap();
        let background_texture = texture_load_from_file(&ctx.display, "textures/background_game.png").unwrap();

        let mut battlefield = Battlefield::new();

        battlefield.spawn(ship_parts::player_ship(Team::Earth, point2(0.0f32, 0.0f32), vec2(0.0f32, 1.0f32)));
                
        let mut me =
            StateData {
                //hive : Hive::new(),
                //player : Player::new(),

                player_ship_texture,
                sun_texture,
                background_texture,
                earth_texture,
                basic_enemy_ship_texture,
                player_bullet_texture,
                //player_dash_trace_texture,

                timer : SPAWN_RATE,
                pointer_target : PointerTarget::None,
                bullet_sys : BulletSystem::new(),
                battlefield,
            }
        ;
        me.load_params();
        GameState::MainGame(me)
    }

    fn load_params(&mut self) {
        use ship_parts::storage_traits::FRICTION_KOEFF;

        let (friction, player_mass) = load_environment_params();

        unsafe { FRICTION_KOEFF = friction; }
        match self.battlefield.get_mut_downcasted::<PlayerShip>(0) {
            Some(player) => player.core.mass = player_mass,
            _ => (),
        }
    }

    pub fn process_event(&mut self, _ctx : &mut GraphicsContext, _input_tracker : &InputTracker, event : &glutin::event::Event<()>) -> Option<TransitionRequest> { 
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
                match self.battlefield.get_mut_downcasted::<PlayerShip>(0) {
                    Some(player) if player.core.is_alive() => {
                        match virtual_keycode {
                            Some(event::VirtualKeyCode::U) => self.load_params(),
                            Some(event::VirtualKeyCode::W) => player.increase_speed(),
                            Some(event::VirtualKeyCode::S) => player.decrease_speed(),
                            Some(event::VirtualKeyCode::Key1) => self.pointer_target = PointerTarget::None,
                            Some(event::VirtualKeyCode::Key2) => self.pointer_target = PointerTarget::Sun,
                            Some(event::VirtualKeyCode::Key3) => self.pointer_target = PointerTarget::Earth,
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

    pub fn update(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, dt : Duration) -> Option<TransitionRequest> {
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

        match self.battlefield.get_mut_downcasted::<PlayerShip>(0) {
            Some(player) if player.core.is_alive() => {
                //dbg!(player.core.velocity);
                player.core.direction = input_tracker.mouse_position().normalize();
        
                if input_tracker.is_mouse_button_down(MouseButton::Left) {
                    player.layout.gun.shoot(&player.core)
                    .map_or((), |x| self.bullet_sys.spawn(x));
                }
            },
            _ => (),
        }

        self.battlefield.update(dt);
        self.bullet_sys.update(&mut self.battlefield, dt);
        self.battlefield.think(&mut self.bullet_sys, dt);
       
        if let Some(player) = self.battlefield.get_downcasted::<PlayerShip>(0) {
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

    pub fn render(&self, ctx : &mut GraphicsContext, _targets : &mut RenderTargets, _input_tracker : &InputTracker) {
        use glium::Surface;
        use cgmath::Matrix4;

        use sys_api::graphics_utils::{ draw_sprite, draw_instanced_sprite };

        let mut frame = ctx.display.draw();
        frame.clear_color(0.0, 0.0, 0.0, 1.0);

        let vp = ctx.build_projection_view_matrix();

        // Drawing paralaxed background
        //use sys_api::graphics_init::ASPECT_RATIO;
        let cam = -ctx.camera.disp.truncate(); 
        let picker = vec2((0.2f32 * cam.x) % 1.0f32, (0.2f32 * cam.y) % 1.0f32);
        draw_sprite(
            ctx, &mut frame, 
            Matrix4::one(),
//            (picker.x / SCREEN_WIDTH, picker.y, 1.0f32, 1.0f32),
            (picker.x, picker.y, 1.0f32, 1.0f32),
            self.background_texture.sampled().wrap_function(SamplerWrapFunction::Repeat), 
            Some(ctx.viewport())
        );
        let picker = vec2((0.05f32 * cam.x - 0.5f32) % 1.0f32, (0.05f32 * cam.y + 0.03f32) % 1.0f32);
        draw_sprite(
            ctx, &mut frame, 
            Matrix4::one(),
//            (picker.x / SCREEN_WIDTH, picker.y, 1.0f32, 1.0f32),
            (picker.x, picker.y, 1.0f32, 1.0f32),
            self.background_texture.sampled().wrap_function(SamplerWrapFunction::Repeat), 
            Some(ctx.viewport())
        );
        // Planets 
        draw_sprite(
            ctx, &mut frame, 
            vp * self.battlefield.earth.model_mat(), 
            (0.0f32, 0.0f32, 1.0f32, 1.0f32),
            self.earth_texture.sampled(), 
            Some(ctx.viewport())
        );
        draw_sprite(
            ctx, &mut frame, 
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
        draw_instanced_sprite(ctx, &mut frame, &ctx.bullet_buffer, vp, self.player_bullet_texture.sampled(), Some(ctx.viewport()));

        ctx.enemy_buffer.invalidate();
        self.battlefield.fill_buffer(&mut ctx.enemy_buffer);
        draw_instanced_sprite(ctx, &mut frame, &ctx.enemy_buffer, vp, self.player_ship_texture.sampled(), Some(ctx.viewport()));

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
                .get_downcasted::<PlayerShip>(0)
                .and_then(|y| point_at(y.core.pos, x))
            )
        ;

        match pointer {
            Some(pointer) => {
                let model_mat = ctx.proj_mat * Matrix4::from_translation(pointer.to_vec().extend(0.0f32)) * Matrix4::from_nonuniform_scale(0.1f32, 0.1f32, 1.0f32);
                draw_sprite(
                    ctx, &mut frame, 
                    model_mat, 
                    (0.0f32, 0.0f32, 1.0f32, 1.0f32),
                    self.basic_enemy_ship_texture.sampled(), 
                    Some(ctx.viewport())
                )
            },
            None => (),
        }

        frame.finish().unwrap();
    }
}

