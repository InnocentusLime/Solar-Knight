use super::{ TransitionRequest, GameState, main_game };

use glium::{ Frame, Surface, glutin };
use glutin::event;
use log::trace;

use std::time::Duration;

use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;

pub struct StateData {

}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, old : GameState) -> GameState {
        trace!("Testing mode entered");

        GameState::Testing(
            StateData {}
        )
    }

    /// The event processing procedure of the state.
    #[inline]
    pub fn process_event(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, event : &glutin::event::Event<()>) -> Option<TransitionRequest> {
        None 
    }

    /// The update routine of the state.
    /// This procedure is responsible for everything.
    #[inline]
    pub fn update(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, dt : Duration) -> Option<TransitionRequest> {
        None
    }

    #[inline]
    pub fn render(&mut self, frame : &mut Frame, ctx : &mut GraphicsContext, targets : &mut RenderTargets, input_tracker : &InputTracker) {
    }
}
