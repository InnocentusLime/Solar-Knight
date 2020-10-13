use cgmath::{ EuclideanSpace, Rad, Angle, Vector2, Point2, vec2 };
use glium::texture::texture2d::Texture2d;
use glutin::event::VirtualKeyCode;

use super::{ GameState, TransitionRequest };
use crate::graphics_init::{ RenderTargets, GraphicsContext };
use crate::input_tracker::InputTracker;
use crate::loaders::texture_load_from_file;
use crate::containers::ConsistentLinearChunk;

mod enemies;
mod player;

use player::*;

pub struct StateData {
    player_ship_texture : Texture2d,
    sun_texture : Texture2d,
    earth_texture : Texture2d,
    basic_enemy_ship_texture : Texture2d,
    background_texture : Texture2d,
    player_bullet_texture : Texture2d,

    player : Player,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, old : GameState) -> GameState {
        use crate::graphics_init::ENEMY_BULLET_LIMIT;

        let sun_texture = texture_load_from_file(&ctx.display, "textures/sun.png").unwrap();
        let player_ship_texture = texture_load_from_file(&ctx.display, "textures/player_ship.png").unwrap();
        let earth_texture = texture_load_from_file(&ctx.display, "textures/earth_texture.png").unwrap();
        let basic_enemy_ship_texture = texture_load_from_file(&ctx.display, "textures/basic_enemy_ship.png").unwrap();
        let player_bullet_texture = texture_load_from_file(&ctx.display, "textures/player_bullet.png").unwrap();

        match old {
            GameState::MainMenu(dat) => {
                GameState::MainGame(
                    StateData {
                        player : Player::new(),
                        player_ship_texture,
                        sun_texture,
                        background_texture : dat.background_texture,
                        earth_texture,
                        basic_enemy_ship_texture,
                        player_bullet_texture,
                    }
                )
            },
            _ => {
                GameState::MainGame(
                    StateData {
                        player : Player::new(),
                        player_ship_texture,
                        sun_texture,
                        background_texture : texture_load_from_file(&ctx.display, "textures/background.png").unwrap(),
                        earth_texture,
                        basic_enemy_ship_texture,
                        player_bullet_texture,
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
                        virtual_keycode : Some(event::VirtualKeyCode::Q),
                        ..
                    }
                    
                ),
                ..
            } => {
                //self.player.shoot();
            },
            _ => (),
        }

        None 
    }

    pub fn update(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker) -> Option<TransitionRequest> {
        use cgmath::{ vec2, dot };
        use cgmath::{ Transform, Angle, InnerSpace, Matrix4 };

        use std::ops::{ Add, Sub };

        use crate::collision::*;

        let player_ang = vec2(0.0f32, 1.0f32).angle(input_tracker.mouse_position());
        self.player.update(input_tracker.is_key_down(VirtualKeyCode::Space), player_ang);
        
        ctx.camera.disp = (-self.player.pos.to_vec()).extend(0.0f32);

         
        if input_tracker.is_key_down(VirtualKeyCode::Q) {
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

        draw_sprite(ctx, &mut frame, Matrix4::one(), &self.background_texture, (1.0f32, 1.0f32), Some(ctx.viewport()));
        draw_sprite(ctx, &mut frame, vp, &self.sun_texture, (0.4f32, 0.4f32), Some(ctx.viewport()));
        draw_sprite(ctx, &mut frame, vp * self.player.model_mat(), &self.player_ship_texture, (0.1f32, 0.1f32), Some(ctx.viewport()));

        // Orphaning technique
        // https://stackoverflow.com/questions/43036568/when-should-glinvalidatebufferdata-be-used
        // https://www.khronos.org/opengl/wiki/Buffer_Object_Streaming
        // https://community.khronos.org/t/vbos-strangely-slow/60109
        ctx.player_bullet_buffer.invalidate();
        self.player.fill_bullet_buffer(&mut ctx.player_bullet_buffer);
        draw_instanced_sprite(ctx, &mut frame, &ctx.player_bullet_buffer, vp, &self.player_bullet_texture, (0.06f32, 0.09f32), Some(ctx.viewport()));

        frame.finish().unwrap();
    }
}

