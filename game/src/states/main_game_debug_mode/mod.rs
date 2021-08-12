use std::fmt;
use std::time::Duration;
use std::error::Error as StdError;

use super::{ TransitionRequest, GameState };

use egui_glium::EguiGlium;
use glium::{ Frame, glutin };
use glutin::event;
use egui::Widget;

use std::str::FromStr;

// TODO create a prelude
pub use egui::Ui;
pub use super::main_game;
pub use ship_parts::ship::Ship;
pub use sys_api::input_tracker::InputTracker;
pub use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
pub use systems::systems_core::{ Storage, ComponentAccess, get_component, get_component_mut };
pub use cgmath::{ EuclideanSpace, InnerSpace, Point2, point2, vec2 };
pub use systems::{
    ship_transform::Transform,
    ship_gun::{ Gun, Guns, BulletKind },
    ship_engine::{ Engine, Engines },
    physics::PhysicsData,
    hp_system::HpInfo,
    square_map::SquareMapNode,
    collision_check::CollisionInfo,
    teams::Team,
};
pub use ship_parts::ai_machine::AiTag;
pub use ship_parts::render::RenderInfo;

mod free_cam;
mod ship_placement;
mod ship_inspector;
mod attachment_edit;

#[derive(Debug)]
pub struct DebugModeEntranceError;

impl fmt::Display for DebugModeEntranceError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Can't enter the main game debug from a state that isn't `MainGame`")
    }
}

impl StdError for DebugModeEntranceError {}

pub trait DebugState {
    fn name(&self) -> &'static str;
    fn process_event(
        &mut self,
        event : &glutin::event::Event<'static, ()>,
        captured_state : &mut main_game::StateData,
        ctx : &mut GraphicsContext, 
        input_tracker : &InputTracker, 
        pointer_in_ui : bool,
        look : &mut Point2<f32>,
    );
    fn update(
        &mut self,
        captured_state : &mut main_game::StateData,
        ctx : &mut GraphicsContext, 
        input_tracker : &InputTracker, 
        dt : Duration,
        ui : &mut Ui,
        pointer_in_ui : bool,
        look : &mut Point2<f32>,
    );
    fn render(
        &self, 
        frame : &mut Frame, 
        captured_state : &main_game::StateData,
        ctx : &mut GraphicsContext, 
        targets : &mut RenderTargets, 
        input_tracker : &InputTracker,
        pointer_in_ui : bool,
    );
}

pub struct StateData {
    look : Point2<f32>,
    captured_state : main_game::StateData,
    // editor state
    state_id : usize,
    states : Vec<Box<dyn DebugState>>,
    // ui backend
    pointer_inside_panel : bool,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, old : GameState) -> GameState {
        match old {
            GameState::MainGame(captured_state) => { 
                GameState::MainGameDebugMode(
                    StateData {
                        // Basic init
                        look : <Point2<f32> as EuclideanSpace>::from_vec(-ctx.camera.disp.truncate()),
                        states : vec![
                            free_cam::FreeCam::new(&captured_state),
                            ship_placement::ShipPlacement::new(&captured_state),
                            ship_inspector::ShipInspector::new(&captured_state),
                            attachment_edit::AttachmentEdit::new(&captured_state),
                        ],
                        state_id : 0,
                        captured_state,
                        // ui backend init
                        pointer_inside_panel : true,
                    }
                )
            },
            _ => GameState::Failure(Box::new(DebugModeEntranceError)),
        }
    }

    /// The event processing procedure of the state.
    #[inline]
    pub fn process_event(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, event : &glutin::event::Event<'static, ()>) -> Option<TransitionRequest> {
        self.states[self.state_id].process_event(
            event,
            &mut self.captured_state,
            ctx,
            input_tracker,
            self.pointer_inside_panel,
            &mut self.look,
        );
        
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
            } => match virtual_keycode {
                Some(event::VirtualKeyCode::Escape) => return Some(Box::new(Self::unwrap)),
                _ => (),
            },
            _ => (),
        }

        None 
    }

    /// The update routine of the state.
    /// This procedure is responsible for everything.
    #[inline]
    pub fn update(
        &mut self, 
        ctx : &mut GraphicsContext, 
        input_tracker : &InputTracker, 
        dt : Duration,
        egui : &mut EguiGlium,
    ) -> Option<TransitionRequest> {
        use glutin::event::VirtualKeyCode as Key;
 
        let look = &mut self.look;
        let states = &mut self.states;
        let state_id = &mut self.state_id;
        let pointer_inside_ui = self.pointer_inside_panel;
        let captured_state = &mut self.captured_state;

        let mut quit = false;
        self.pointer_inside_panel =     
        {
            let rect =
                egui::SidePanel::left("debug_menu", 350.0)
                .show(egui.ctx(), |ui| {
                    ui.heading("Debug menu");
                    if ui.button("Quit").clicked() { quit = true; }

                    egui::ComboBox::from_label("Mode")
                    .width(150.0)
                    .selected_text(states[*state_id].name())
                    .show_ui(
                        ui,
                        |ui| {
                            states.iter()
                            .enumerate()
                            .for_each(
                                |(id, x)|
                                { ui.selectable_value(state_id, id, x.name()); }
                            )
                        }
                    );

                    egui::Separator::default()
                    .horizontal()
                    .ui(ui);

                    states[*state_id].update(
                        captured_state,
                        ctx,
                        input_tracker,
                        dt,
                        ui,
                        pointer_inside_ui,
                        look,
                    );
                }).response.rect
            ;
            egui.ctx().input().pointer
            .hover_pos()
            .map(|x| rect.contains(x))
            .unwrap_or(false)
        };

        if input_tracker.is_key_down(Key::W) { self.look += dt.as_secs_f32() * vec2(0.0f32, 1.0f32); }
        if input_tracker.is_key_down(Key::A) { self.look += dt.as_secs_f32() * vec2(-1.0f32, 0.0f32); }
        if input_tracker.is_key_down(Key::S) { self.look += dt.as_secs_f32() * vec2(0.0f32, -1.0f32); }
        if input_tracker.is_key_down(Key::D) { self.look += dt.as_secs_f32() * vec2(1.0f32, 0.0f32); }

        ctx.camera.disp = (-self.look.to_vec()).extend(0.0f32);

        if quit { Some(Box::new(Self::unwrap)) }
        else { None }
    }

    #[inline]
    pub fn render(&self, frame : &mut Frame, ctx : &mut GraphicsContext, targets : &mut RenderTargets, input_tracker : &InputTracker) {
        self.captured_state.render(frame, ctx, targets, input_tracker);
        self.states[self.state_id].render(
            frame, 
            &self.captured_state, 
            ctx, 
            targets, 
            input_tracker, 
            self.pointer_inside_panel
        );
    }

    pub fn unwrap(_ : &mut GraphicsContext, g : GameState) -> GameState {
        match g {
            GameState::MainGameDebugMode(x) => GameState::MainGame(x.captured_state),
            _ => panic!("Called `MainGameDebugMode::unwrap` for an alien state"),
        }
    }
}
