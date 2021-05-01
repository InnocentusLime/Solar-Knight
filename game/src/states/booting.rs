use std::time::Duration;

use super::{ GameState, TransitionRequest, testing };
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;

use glium::{ glutin, Frame };

const SLEEP_FRAMES : u64 = 100;

pub struct StateData {
    frame_counter : u64,
}

impl StateData {
    pub fn init(_ctx : &mut GraphicsContext) -> GameState {
        GameState::Booting(
            StateData {
                frame_counter : 0,
            } 
        )
    }

    pub fn process_event(&mut self, _ctx : &mut GraphicsContext, _input_tracker : &InputTracker, _event : &glutin::event::Event<()>) -> Option<TransitionRequest> { None }

    pub fn update(&mut self, _ctx : &mut GraphicsContext, input_tracker : &InputTracker, _dt : Duration) -> Option<TransitionRequest> {
        use super::main_menu;
        use glutin::event;

        if self.frame_counter >= SLEEP_FRAMES {
            return Some(Box::new(main_menu::StateData::init));
        } else { self.frame_counter += 1 }

        if input_tracker.is_key_down(event::VirtualKeyCode::T) {
            return Some(Box::new(testing::StateData::init))
        }

        None
    }

    pub fn render(&self, _frame : &mut Frame, _ctx : &mut GraphicsContext, _targets : &mut RenderTargets, _input_tracker : &InputTracker) {}
}
