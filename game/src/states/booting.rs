use std::time::Duration;

use super::{ GameState, TransitionRequest };
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;

use glium::glutin;

const SLEEP_FRAMES : u64 = 100;

pub struct StateData {
    frame_counter : u64,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext) -> GameState {
        GameState::Booting(
            StateData {
                frame_counter : 0,
            } 
        )
    }

    pub fn process_event(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, event : &glutin::event::Event<()>) -> Option<TransitionRequest> { None }

    pub fn update(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, _ : Duration) -> Option<TransitionRequest> {
        use super::main_menu;

        if self.frame_counter >= SLEEP_FRAMES {
            Some(Box::new(main_menu::StateData::init))
        } else { self.frame_counter += 1; None }
    }

    pub fn render(&self, ctx : &mut GraphicsContext, _ : &mut RenderTargets, input_tracker : &InputTracker) {}
}
