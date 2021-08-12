use std::time::Duration;

use super::{ GameState, TransitionRequest };
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;

use egui_glium::EguiGlium;
use glium::{ glutin, Frame };

const SLEEP_FRAMES : u64 = 100;

pub struct StateData {
    // Some data for the booting screen
    frame_counter : u64,

    // Entrance controller
    debug_menu_open : bool,
}

impl StateData {
    pub fn init(_ctx : &mut GraphicsContext) -> GameState {
        GameState::Booting(
            StateData {
                frame_counter : 0,
                
                debug_menu_open : false,
            } 
        )
    }

    pub fn process_event(&mut self, _ctx : &mut GraphicsContext, input_tracker : &InputTracker, event : &glutin::event::Event<'static, ()>) -> Option<TransitionRequest> {  
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
                    Some(event::VirtualKeyCode::D) if input_tracker.is_key_down(event::VirtualKeyCode::LControl) => {
                        self.debug_menu_open = true;
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
        _ctx : &mut GraphicsContext, 
        _input_tracker : &InputTracker, 
        _dt : Duration,
        egui : &mut EguiGlium,
    ) -> Option<TransitionRequest> {
        use super::{ main_menu, testing, main_game };

        if self.frame_counter >= SLEEP_FRAMES {
            return Some(Box::new(main_menu::StateData::init));
        } else if !self.debug_menu_open { 
            self.frame_counter += 1;
        }
           
        let mut state_req : Option<TransitionRequest> = None;
        if self.debug_menu_open {
            egui::SidePanel::left("debug_menu", 350.0)
            .show(egui.ctx(), |ui| {
                ui.heading("Debug menu");
                if ui.button("Main menu").clicked() { state_req = Some(Box::new(main_menu::StateData::init)); }
                if ui.button("Main game").clicked() { state_req = Some(Box::new(main_game::StateData::init)); }
                if ui.button("Test room").clicked() { state_req = Some(Box::new(testing::StateData::init)); }
            });
        }

        state_req
    }

    pub fn render(&self, _frame : &mut Frame, _ctx : &mut GraphicsContext, _targets : &mut RenderTargets, _input_tracker : &InputTracker) {}
}
