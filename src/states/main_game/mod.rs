use std::time::Duration;

use cgmath::{ EuclideanSpace, Rad, Angle, Vector2, Point2, vec2 };
use glium::texture::texture2d::Texture2d;
use glutin::event::{ VirtualKeyCode, MouseButton };

use super::{ GameState, TransitionRequest };
use crate::duration_ext::*;
use crate::graphics_init::{ RenderTargets, GraphicsContext, ENEMY_LIMIT };
use crate::input_tracker::InputTracker;
use crate::loaders::texture_load_from_file;

mod earth;
mod enemies;
mod player;

use earth::*;
use player::*;
use enemies::{ Hive, Enemy, tester::Tester };

const SPAWN_RATE : Duration = Duration::from_secs(3);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PointerTarget {
    None,
    Sun,
    Earth,
}

pub struct StateData {
    player_ship_texture : Texture2d,
    sun_texture : Texture2d,
    earth_texture : Texture2d,
    basic_enemy_ship_texture : Texture2d,
    background_texture : Texture2d,
    player_bullet_texture : Texture2d,
    player_dash_trace_texture : Texture2d,

    earth : Earth,
    player : Player,
    hive : Hive,

    timer : Duration,
    pointer_target : PointerTarget,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, old : GameState) -> GameState {
        use crate::graphics_init::ENEMY_BULLET_LIMIT;

        let sun_texture = texture_load_from_file(&ctx.display, "textures/sun.png").unwrap();
        let player_ship_texture = texture_load_from_file(&ctx.display, "textures/player_ship.png").unwrap();
        let earth_texture = texture_load_from_file(&ctx.display, "textures/earth.png").unwrap();
        let basic_enemy_ship_texture = texture_load_from_file(&ctx.display, "textures/basic_enemy_ship.png").unwrap();
        let player_bullet_texture = texture_load_from_file(&ctx.display, "textures/player_bullet.png").unwrap();
        let player_dash_trace_texture = texture_load_from_file(&ctx.display, "textures/player_dash_trace.png").unwrap();

        match old {
            GameState::MainMenu(dat) => {
                GameState::MainGame(
                    StateData {
                        earth : Earth::new(),
                        hive : Hive::new(),
                        player : Player::new(),

                        player_ship_texture,
                        sun_texture,
                        background_texture : dat.background_texture,
                        earth_texture,
                        basic_enemy_ship_texture,
                        player_bullet_texture,
                        player_dash_trace_texture,

                        timer : SPAWN_RATE,
                        pointer_target : PointerTarget::None,
                    }
                )
            },
            _ => {
                GameState::MainGame(
                    StateData {
                        earth : Earth::new(),
                        hive : Hive::new(),
                        player : Player::new(),

                        player_ship_texture,
                        sun_texture,
                        background_texture : texture_load_from_file(&ctx.display, "textures/background.png").unwrap(),
                        earth_texture,
                        basic_enemy_ship_texture,
                        player_bullet_texture,
                        player_dash_trace_texture,

                        timer : SPAWN_RATE,
                        pointer_target : PointerTarget::None,
                    }
                )
            },
        }
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
                match virtual_keycode {
                    Some(event::VirtualKeyCode::W) => self.player.increase_speed(),
                    Some(event::VirtualKeyCode::S) => self.player.decrease_speed(),
                    Some(event::VirtualKeyCode::D) => self.player.dash_right(),
                    Some(event::VirtualKeyCode::A) => self.player.dash_left(),
                    Some(event::VirtualKeyCode::Key1) => self.pointer_target = PointerTarget::None,
                    Some(event::VirtualKeyCode::Key2) => self.pointer_target = PointerTarget::Sun,
                    Some(event::VirtualKeyCode::Key3) => self.pointer_target = PointerTarget::Earth,
                    _ => (),
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

        use crate::collision::*;

        self.earth.update(dt);

        if self.timer.my_is_zero() {
            self.timer = SPAWN_RATE;
            //self.hive.spawn(Enemy::Tester(Tester::new()));
        } else if self.hive.alive_count() < ENEMY_LIMIT { 
            self.timer = self.timer.my_saturating_sub(dt);
        }

        self.player.update(input_tracker.mouse_position(), dt);
        self.hive.update(&self.player, dt);
        self.player.update_bullets(&mut self.hive, dt);
        
        ctx.camera.disp = (-self.player.pos().to_vec()).extend(0.0f32);

         
        if input_tracker.is_mouse_button_down(MouseButton::Left) {
                self.player.shoot();
        }
        

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

        use crate::graphics_utils::{ draw_sprite, draw_instanced_sprite };
        use crate::graphics_init::{ ASPECT_RATIO, ENEMY_BULLET_LIMIT };

        let mut frame = ctx.display.draw();
        frame.clear_color(1.0, 0.0, 0.0, 1.0);

        let vp = ctx.build_projection_view_matrix();

        draw_sprite(ctx, &mut frame, Matrix4::one(), &self.background_texture, Some(ctx.viewport()));
        draw_sprite(ctx, &mut frame, vp * self.earth.model_mat(), &self.earth_texture, Some(ctx.viewport()));
        draw_sprite(ctx, &mut frame, vp * Matrix4::from_nonuniform_scale(0.6f32, 0.6f32, 1.0f32), &self.sun_texture, Some(ctx.viewport()));
        draw_sprite(ctx, &mut frame, vp * self.player.model_mat(), &self.player_ship_texture, Some(ctx.viewport()));

        // Orphaning technique
        // https://stackoverflow.com/questions/43036568/when-should-glinvalidatebufferdata-be-used
        // https://www.khronos.org/opengl/wiki/Buffer_Object_Streaming
        // https://community.khronos.org/t/vbos-strangely-slow/60109
        ctx.player_bullet_buffer.invalidate();
        self.player.fill_bullet_buffer(&mut ctx.player_bullet_buffer);
        draw_instanced_sprite(ctx, &mut frame, &ctx.player_bullet_buffer, vp, &self.player_bullet_texture, Some(ctx.viewport()));

        ctx.enemy_buffer.invalidate();
        self.hive.fill_enemy_buffer(&mut ctx.enemy_buffer);
        draw_instanced_sprite(ctx, &mut frame, &ctx.enemy_buffer, vp, &self.basic_enemy_ship_texture, Some(ctx.viewport()));

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

        match self.player.dash_trace_model_mat() {
            Some(mat) => draw_sprite(ctx, &mut frame, vp * mat, &self.player_dash_trace_texture, Some(ctx.viewport())),
            None => (),
        }

/*
        for enemy in self.hive.enemies.iter() {
            match enemy {
                Enemy::Brute(_) => (),
                Enemy::Tester(enemy) => draw_sprite(ctx, &mut frame, vp * enemy.model_mat(), &self.basic_enemy_ship_texture, (0.1f32, 0.1f32), Some(ctx.viewport())),
            }
        }
*/

        frame.finish().unwrap();
    }
}

