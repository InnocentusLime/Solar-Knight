use super::{ TransitionRequest, GameState };

use egui_glium::EguiGlium;
use log::trace;
use glium::{ Frame, glutin };

use std::time::Duration;

use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;

pub struct StateData {}

impl StateData {
    pub fn init(_ctx : &mut GraphicsContext, _old : GameState) -> GameState {
        trace!("Testing mode entered");

        GameState::Testing(
            StateData {
            }
        )
    }

    /// The event processing procedure of the state.
    #[inline]
    pub fn process_event(&mut self, _ctx : &mut GraphicsContext, _input_tracker : &InputTracker, _event : &glutin::event::Event<'static, ()>) -> Option<TransitionRequest> {
        None 
    }

    /// The update routine of the state.
    /// This procedure is responsible for everything.
    #[inline]
    pub fn update(
        &mut self, 
        _ctx : &mut GraphicsContext, 
        _input_tracker : &InputTracker, 
        _dt : Duration,
        egui : &mut EguiGlium,
    ) -> Option<TransitionRequest> {
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
        None
    }

    #[inline]
    pub fn render(&self, _frame : &mut Frame, _ctx : &mut GraphicsContext, _targets : &mut RenderTargets, _input_tracker : &InputTracker) {
    }
}
