use std::time::Duration;

use super::{ GameState, TransitionRequest };
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;

use glium::{ glutin, Frame };
use glutin::{ event, event_loop::ControlFlow };
use egui_glium::EguiGlium;
use egui::{ epaint::ClippedShape, Widget, Sense, Id };

const SLEEP_FRAMES : u64 = 100;

pub struct StateData {
    // Some data for the booting screen
    frame_counter : u64,

    // Entrance controller
    debug_menu_open : bool,
    eg : EguiGlium,
    draw_req : Option<Vec<ClippedShape>>,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext) -> GameState {
        GameState::Booting(
            StateData {
                frame_counter : 0,
                
                debug_menu_open : false,
                eg : EguiGlium::new(&ctx.display),
                draw_req : None,
            } 
        )
    }

    pub fn process_event(&mut self, _ctx : &mut GraphicsContext, input_tracker : &InputTracker, event : &glutin::event::Event<'static, ()>) -> Option<TransitionRequest> {  
        use glutin::event;
        
        match event {
            event::Event::WindowEvent { event, window_id } => {
                // Avoid handing the real control flow to `egui` for now.
                // Waiting for the reply:
                // https://github.com/emilk/egui/issues/434
                let mut dummy = ControlFlow::Exit;
                self.eg.on_event(event.clone(), &mut dummy);
            },
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

    pub fn update(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, _dt : Duration) -> Option<TransitionRequest> {
        use super::{ main_menu, testing, main_game };
        use glutin::event;

        if self.frame_counter >= SLEEP_FRAMES {
            return Some(Box::new(main_menu::StateData::init));
        } else if !self.debug_menu_open { 
            self.frame_counter += 1;
        }
           
        let mut state_req : Option<TransitionRequest> = None;
        if self.debug_menu_open {
            let egui = &mut self.eg;

            egui.begin_frame(&ctx.display);
            egui::SidePanel::left("debug_menu", 350.0)
            .show(egui.ctx(), |ui| {
                ui.heading("Debug menu");
                if ui.button("Main menu").clicked() { state_req = Some(Box::new(main_menu::StateData::init)); }
                if ui.button("Main game").clicked() { state_req = Some(Box::new(main_game::StateData::init)); }
                if ui.button("Test room").clicked() { state_req = Some(Box::new(testing::StateData::init)); }
            });
            let (_, shapes) = egui.end_frame(&ctx.display);
            self.draw_req = Some(shapes);
        }

        state_req
    }

    pub fn render(&mut self, frame : &mut Frame, ctx : &mut GraphicsContext, _targets : &mut RenderTargets, _input_tracker : &InputTracker) {
        self.draw_req.take()
        .map(|x| self.eg.paint(&ctx.display, frame, x));
    }
}
