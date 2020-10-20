use glium::texture::texture2d::Texture2d;

use super::{ GameState, TransitionRequest }; 
use crate::graphics_init::{ RenderTargets, GraphicsContext, SCREEN_WIDTH };
use crate::loaders::texture_load_from_file; 
use crate::input_tracker::InputTracker;
use super::main_game;

pub struct StateData {
    pub background_texture : Texture2d,
    pub play_button_texture : Texture2d,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, old : GameState) -> GameState {
        GameState::MainMenu(
            StateData {
                background_texture : texture_load_from_file(&ctx.display, "textures/background.png").unwrap(), 
                play_button_texture : texture_load_from_file(&ctx.display, "textures/start_button.png").unwrap(),
            } 
        )
    }

    pub fn process_event(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, event : &glutin::event::Event<()>) -> Option<TransitionRequest> {
        match event {
            glutin::event::Event::WindowEvent {
                event : glutin::event::WindowEvent::MouseInput { 
                    state : glutin::event::ElementState::Released, 
                    button : glutin::event::MouseButton::Left, 
                    .. 
                },
                ..
            } => {
                let mouse_pos = input_tracker.mouse_position();
                if (-0.5f32 <= mouse_pos.x && mouse_pos.x <= 0.5f32) && (-0.5f32 <= mouse_pos.y && mouse_pos.y <= 0.5f32) { 
                    Some(Box::new(main_game::StateData::init)) 
                } else { None }
            },
            _ => None,
        }
    }

    pub fn update(&mut self, _ctx : &mut GraphicsContext, _input_tracker : &InputTracker) -> Option<TransitionRequest> {
        None
    }

    pub fn render(&self, ctx : &mut GraphicsContext, _ : &mut RenderTargets, _input_tracker : &InputTracker) {
        use glium::{ draw_parameters, index, Surface, Blend, uniform };
        use cgmath::{ One, Matrix4, vec2, vec3 };

        use crate::graphics_utils::draw_sprite;

        let mut frame = ctx.display.draw();
        frame.clear_color(1.0, 1.0, 1.0, 1.0);

        /* render code */
        let vp = ctx.proj_mat;

        draw_sprite(ctx, &mut frame, Matrix4::one(), &self.background_texture, Some(ctx.viewport()));
        draw_sprite(ctx, &mut frame, vp * Matrix4::from_nonuniform_scale(0.5f32, 0.5f32, 1.0f32), &self.play_button_texture, Some(ctx.viewport()));

        frame.finish().unwrap(); 
    }
}
