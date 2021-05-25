use super::{ TransitionRequest, GameState, main_game };

use glium::{ Frame, Surface, glutin };
use glutin::{ event, event_loop::ControlFlow };
use log::trace;
use egui_glium::EguiGlium;
use egui::epaint::ClippedShape;

use std::time::Duration;

use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;

pub struct StateData {
    eg : EguiGlium,
    draw_req : Option<Vec<ClippedShape>>,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, old : GameState) -> GameState {
        trace!("Testing mode entered");

        GameState::Testing(
            StateData {
                eg : EguiGlium::new(&ctx.display),
                draw_req : None,
            }
        )
    }

    /// The event processing procedure of the state.
    #[inline]
    pub fn process_event(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, event : &glutin::event::Event<'static, ()>) -> Option<TransitionRequest> {
        match event {
            event::Event::WindowEvent { event, .. } => {
                // Avoid handing the real control flow to `egui` for now.
                // Waiting for the reply:
                // https://github.com/emilk/egui/issues/434
                let mut dummy = ControlFlow::Exit;
                self.eg.on_event(event.clone(), &mut dummy);
            },
            _ => (),
        }

        None 
    }

    /// The update routine of the state.
    /// This procedure is responsible for everything.
    #[inline]
    pub fn update(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, dt : Duration) -> Option<TransitionRequest> {
        let egui = &mut self.eg;
        egui.begin_frame(&ctx.display);

        egui::SidePanel::left("my_side_panel", 300.0).show(egui.ctx(), |ui| {
            ui.heading("Hello World!");
            if ui.button("Quit").clicked() { println!("Quiter."); }

            egui::ComboBox::from_label("Version")
            .width(150.0)
            .selected_text("foo")
            .show_ui(ui, |ui| {
                egui::CollapsingHeader::new("Dev")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("contents");
                });
            });
        });

        let (_, shapes) = egui.end_frame(&ctx.display);

        self.draw_req = Some(shapes);

        None
    }

    #[inline]
    pub fn render(&mut self, frame : &mut Frame, ctx : &mut GraphicsContext, targets : &mut RenderTargets, input_tracker : &InputTracker) {
        self.draw_req.take()
        .map(|x| self.eg.paint(&ctx.display, frame, x));
    }
}
