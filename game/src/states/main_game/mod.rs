use std::time::Duration;

use log::{ trace, info, warn };
use cgmath::{ EuclideanSpace, One, Rad, Angle, Vector2, Point2, vec2, point2 };
use glium::texture::texture2d::Texture2d;
use glutin::event::{ VirtualKeyCode, MouseButton };

use sys_api::basic_graphics_data::SpriteData;
use ship_parts::gun::{ BulletSystem, TESTER_BULLET_SIZE };
use super::{ GameState, TransitionRequest };
use std_ext::collections::memory_chunk::MemoryChunk;
use std_ext::*;
use sys_api::graphics_init::{ RenderTargets, GraphicsContext, ENEMY_LIMIT, PLAYER_BULLET_LIMIT };
use sys_api::input_tracker::InputTracker;
use crate::loaders::texture_load_from_file;

mod ships;
mod earth;
//mod enemies;
//mod player;

use earth::*;
use ships::Battlefield;
//use player::*;
//use enemies::{ Hive, Enemy, tester::Tester };

const SPAWN_RATE : Duration = Duration::from_secs(3);

/*
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PointerTarget {
    None,
    Sun,
    Earth,
}

struct DasherTraceData {
    direction : Vector2<f32>,
    position : Point2<f32>,
}

impl DasherTraceData {
    fn new() -> Self {
        DasherTraceData {
            direction : vec2(0.0f32, 0.0f32),
            position : point2(0.0f32, 0.0f32),
        }
    }

    fn update(&mut self, player : &Player, dash_direction : Vector2<f32>) {
        self.position = player.pos();
        self.direction = -(dash_direction + (player.get_speed() as f32 / Player::MAX_SPEED as f32) * player.direction());
    }
}
*/

pub struct StateData {
    player_ship_texture : Texture2d,
    sun_texture : Texture2d,
    earth_texture : Texture2d,
    basic_enemy_ship_texture : Texture2d,
    background_texture : Texture2d,
    player_bullet_texture : Texture2d,
    player_dash_trace_texture : Texture2d,

    earth : Earth,
    battlefield : Battlefield,

    timer : Duration,
    //pointer_target : PointerTarget,
    //dasher_trace_data : DasherTraceData,
    bullet_sys : BulletSystem,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, old : GameState) -> GameState {
        use sys_api::graphics_init::ENEMY_BULLET_LIMIT;

        let sun_texture = texture_load_from_file(&ctx.display, "textures/sun.png").unwrap();
        let player_ship_texture = texture_load_from_file(&ctx.display, "textures/player_ship.png").unwrap();
        let earth_texture = texture_load_from_file(&ctx.display, "textures/earth.png").unwrap();
        let basic_enemy_ship_texture = texture_load_from_file(&ctx.display, "textures/basic_enemy_ship.png").unwrap();
        let player_bullet_texture = texture_load_from_file(&ctx.display, "textures/player_bullet.png").unwrap();
        let player_dash_trace_texture = texture_load_from_file(&ctx.display, "textures/player_dash_trace.png").unwrap();

        let background_texture;

        match old {
            GameState::MainMenu(dat) => {
                background_texture = dat.background_texture;
            },
            _ => {
                background_texture = texture_load_from_file(&ctx.display, "textures/background.png").unwrap();
            },
        }

        let mut battlefield = Battlefield::new();

        use ships::Ship;
        use ship_parts::collision_models::model_indices;
        use ship_parts::core::Team;
        battlefield.spawn(Ship::player_ship(Team::Earth, point2(0.0f32, 0.0f32), vec2(0.0f32, 1.0f32)));
                
        GameState::MainGame(
            StateData {
                earth : Earth::new(),
                //hive : Hive::new(),
                //player : Player::new(),

                player_ship_texture,
                sun_texture,
                background_texture,
                earth_texture,
                basic_enemy_ship_texture,
                player_bullet_texture,
                player_dash_trace_texture,

                timer : SPAWN_RATE,
                //pointer_target : PointerTarget::None,
                //dasher_trace_data : DasherTraceData::new(),
                bullet_sys : BulletSystem::new(),
                battlefield,
            }
        )
    }

    pub fn process_event(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, event : &glutin::event::Event<()>) -> Option<TransitionRequest> { 
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
                if let Some(player) = self.battlefield.player() {
                    // Feel free to go crazy to implement controls
                    // for other layouts.
                    use ships::{ ShipLayout, PlayerShip };
                    match &mut player.layout {
                        ShipLayout::PlayerShip(sh) => {
                            match virtual_keycode {
                                Some(event::VirtualKeyCode::W) => sh.increase_speed(),
                                Some(event::VirtualKeyCode::S) => sh.decrease_speed(),
                            /*
                                Some(event::VirtualKeyCode::D) => {
                                    self.player.dash_right()
                                    .map_or((), |x| self.dasher_trace_data.update(&self.player, x))
                                    // update the dash data
                                },
                                Some(event::VirtualKeyCode::A) => {
                                    self.player.dash_left()
                                    .map_or((), |x| self.dasher_trace_data.update(&self.player, x))
                                    // update the dash data
                                },
                                Some(event::VirtualKeyCode::Key1) => self.pointer_target = PointerTarget::None,
                                Some(event::VirtualKeyCode::Key2) => self.pointer_target = PointerTarget::Sun,
                                Some(event::VirtualKeyCode::Key3) => self.pointer_target = PointerTarget::Earth,
                            */
                                _ => (),
                            }
                        },     
                        _ => (), // Unsupported layout
                    }
                }
            },
            _ => (),
        }

        None 
    }

    pub fn update(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, dt : Duration) -> Option<TransitionRequest> {
        use cgmath::{ vec2, dot };
        use cgmath::{ Transform, Angle, InnerSpace, Matrix4 };

        use std::ops::{ Add, Sub };

        self.earth.update(dt);

        /*
        if self.timer.my_is_zero() {
            self.timer = SPAWN_RATE;
            //self.hive.spawn(Enemy::Tester(Tester::new()));
        } else if self.hive.alive_count() < ENEMY_LIMIT { 
            self.timer = self.timer.my_saturating_sub(dt);
        }
        */

        if let Some(player) = self.battlefield.player() {
            player.core.direction = input_tracker.mouse_position().normalize();
            ctx.camera.disp = (-player.core.pos.to_vec()).extend(0.0f32);

            use ships::{ ShipLayout, PlayerShip };
            match &mut player.layout {
                ShipLayout::PlayerShip(x) => {        
                    if input_tracker.is_mouse_button_down(MouseButton::Left) {
                        x.gun.shoot(&player.core)
                        .map_or((), |x| self.bullet_sys.spawn(x));
                    }
                },
                _ => warn!("Can't shoot with current player layout"),
            }

        } else { panic!("No player!!"); }

        self.battlefield.update(dt);
        self.bullet_sys.update(&mut self.battlefield, dt);
        self.battlefield.think(&mut self.bullet_sys);
         
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

    pub fn render(&self, ctx : &mut GraphicsContext, targets : &mut RenderTargets, _input_tracker : &InputTracker) {
        use glium::{ draw_parameters, index, Surface, Blend, Rect, uniform };
        use cgmath::{ Transform, Matrix4, vec2, vec3 };

        use sys_api::graphics_utils::{ draw_sprite, draw_instanced_sprite };
        use sys_api::graphics_init::{ ASPECT_RATIO, ENEMY_BULLET_LIMIT };

        let mut frame = ctx.display.draw();
        frame.clear_color(1.0, 0.0, 0.0, 1.0);

        let vp = ctx.build_projection_view_matrix();

        draw_sprite(ctx, &mut frame, Matrix4::one(), &self.background_texture, Some(ctx.viewport()));
        draw_sprite(ctx, &mut frame, vp * self.earth.model_mat(), &self.earth_texture, Some(ctx.viewport()));
        draw_sprite(ctx, &mut frame, vp * Matrix4::from_nonuniform_scale(0.6f32, 0.6f32, 1.0f32), &self.sun_texture, Some(ctx.viewport()));
        //draw_sprite(ctx, &mut frame, vp * self.player.model_mat(), &self.player_ship_texture, Some(ctx.viewport()));

        // Orphaning technique
        // https://stackoverflow.com/questions/43036568/when-should-glinvalidatebufferdata-be-used
        // https://www.khronos.org/opengl/wiki/Buffer_Object_Streaming
        // https://community.khronos.org/t/vbos-strangely-slow/60109
        
        ctx.bullet_buffer.invalidate();
        self.bullet_sys.fill_buffer(&mut ctx.bullet_buffer);
        draw_instanced_sprite(ctx, &mut frame, &ctx.bullet_buffer, vp, &self.player_bullet_texture, Some(ctx.viewport()));

        ctx.enemy_buffer.invalidate();
        self.battlefield.fill_buffer(&mut ctx.enemy_buffer);
        draw_instanced_sprite(ctx, &mut frame, &ctx.enemy_buffer, vp, &self.basic_enemy_ship_texture, Some(ctx.viewport()));

        /*
        let pointer_target = 
            match self.pointer_target {
                PointerTarget::None => None,
                PointerTarget::Sun => Some(Point2 { x : 0.0f32, y : 0.0f32 }),
                PointerTarget::Earth => Some(self.earth.pos()),
            }
        ;

        let pointer = pointer_target.and_then(|x| self.player.point_at(x));

        match pointer {
            Some(pointer) => {
                let model_mat = ctx.proj_mat * Matrix4::from_translation(pointer.to_vec().extend(0.0f32)) * Matrix4::from_nonuniform_scale(0.1f32, 0.1f32, 1.0f32);
                draw_sprite(ctx, &mut frame, model_mat, &self.basic_enemy_ship_texture, Some(ctx.viewport()))
            },
            None => (),
        }
        */

        /*
        const PLAYER_DASH_TRACE_SPEED : f32 = 0.6f32;
        // If we are doing the dash (player returns the dash parameter),
        // draw the trace
        self.player.dash_trace_param()
        .map_or(
            (),
            |t| {
                let k = 2.0f32 * t;
                let trace_direction = self.dasher_trace_data.direction;
                let trace_pos = self.dasher_trace_data.position + (0.1f32 * (1.0f32 - t)) * trace_direction;
                let model_mat = 
                Matrix4::from_translation(trace_pos.to_vec().extend(0.0f32)) * 
                Matrix4::new(
                    trace_direction.x, trace_direction.y, 0.0f32, 0.0f32,
                    -trace_direction.y, trace_direction.x, 0.0f32, 0.0f32,
                    0.0f32, 0.0f32, 1.0f32, 0.0f32,
                    0.0f32, 0.0f32, 0.0f32, 1.0f32,
                ) * 
                Matrix4::from_nonuniform_scale(k * 0.04f32, (k / 2.0f32 * 0.4f32 + 0.6f32) * 0.125f32, 1.0f32); 
                draw_sprite(ctx, &mut frame, vp * model_mat, &self.player_dash_trace_texture, Some(ctx.viewport()))
            }
        );
        */
        
        frame.finish().unwrap();
    }
}

