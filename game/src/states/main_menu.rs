use std::time::Duration;

use egui_glium::EguiGlium;
use glium::{ glutin, Frame };
use glium::texture::texture2d::Texture2d;

use super::{ GameState, TransitionRequest }; 
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use loaders::load_texture_from_file; 
use sys_api::input_tracker::InputTracker;
use super::main_game;

pub struct StateData {
    pub background_texture : Texture2d,
    pub play_button_texture : Texture2d,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, _old : GameState) -> GameState {
        GameState::MainMenu(
            StateData {
                background_texture : load_texture_from_file(&ctx.display, "textures/background.png").unwrap(), 
                play_button_texture : load_texture_from_file(&ctx.display, "textures/start_button.png").unwrap(),
            } 
        )
    }

    pub fn process_event(&mut self, _ctx : &mut GraphicsContext, input_tracker : &InputTracker, event : &glutin::event::Event<()>) -> Option<TransitionRequest> {
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

    pub fn update(
        &mut self, 
        _ctx : &mut GraphicsContext, 
        _input_tracker : &InputTracker, 
        _dt : Duration,
        _egui : &mut EguiGlium,
    ) -> Option<TransitionRequest> {
        None
    }

    pub fn render(&self, frame : &mut Frame, ctx : &mut GraphicsContext, _ : &mut RenderTargets, _input_tracker : &InputTracker) {
        use glium::Surface;
        use cgmath::{ One, Matrix4 };

        use sys_api::graphics_utils::draw_sprite;
        /* render code */
        let vp = ctx.proj_mat;

        draw_sprite(
            ctx, frame, 
            Matrix4::one(), 
            (0.0f32, 0.0f32, 1.0f32, 1.0f32),
            self.background_texture.sampled(), 
            Some(ctx.viewport())
        );
        draw_sprite(
            ctx, frame, 
            vp * Matrix4::from_nonuniform_scale(0.5f32, 0.5f32, 1.0f32), 
            (0.0f32, 0.0f32, 1.0f32, 1.0f32),
            self.play_button_texture.sampled(), 
            Some(ctx.viewport())
        );
    }
}
