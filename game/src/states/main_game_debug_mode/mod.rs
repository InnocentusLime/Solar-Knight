use std::fmt;
use std::time::Duration;
use std::error::Error as StdError;

use super::{ TransitionRequest, GameState, main_game };

use egui_glium::EguiGlium;
use glium::{ Frame, Surface, glutin };
use lazy_static::lazy_static;
use glutin::{ event, event_loop::ControlFlow };
use cgmath::{ EuclideanSpace, Point2, point2, vec2 };
use egui::{ epaint::ClippedShape, Widget, Sense, Id, Ui };

use ship_parts::Ship;
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;

use std::str::FromStr;

fn input_box<T : FromStr + ToString>(ui : &mut Ui, buffer : &mut String, target : &mut T, header : &'static str) {
    if 
        egui::TextEdit::singleline(buffer)
        .hint_text(header)
        .desired_width(50.0f32)
        .ui(ui)
        .lost_focus() 
    {
        if let Ok(new_target) = buffer.trim().parse() { *target = new_target; }
        else { *buffer = target.to_string() }
    }
}

#[derive(Debug)]
pub struct DebugModeEntranceError;

impl fmt::Display for DebugModeEntranceError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Can't enter the main game debug from a state that isn't `MainGame`")
    }
}

impl StdError for DebugModeEntranceError {}

lazy_static! {
    static ref TEMPLATE_TABLE : [fn() -> Ship; 2] =
        [
            ship_parts::player_ship,
            ship_parts::turret_ship,
        ]
    ;
}

#[derive(Clone)]
enum DebugState {
    FreeCam,
    PlacingShip {
        placing : bool,
        placed_ship_info : usize,
        current_ship : Ship,
    },
    InspectingShip {
        // Selection of ship
        id : usize,
        input : String,
        // Ship fields
        hp : String,
        mass : String,
    },
}

impl PartialEq for DebugState {
    fn eq(&self, other : &Self) -> bool {
        use std::mem;

        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl DebugState {
    fn free_cam_display_str() -> &'static str { "Free camera" }

    fn free_cam() -> Self { Self::FreeCam }
    
    fn placing_ship_display_str() -> &'static str { "Placing ship" }

    fn placing_ship() -> Self {
        Self::PlacingShip {
            placing : false,
            placed_ship_info : 0,
            current_ship : (TEMPLATE_TABLE[0])(),
        }
    }

    fn inspecting_ship() -> Self {
        Self::InspectingShip {
            // Ship selection
            id : 0,
            input : 0.to_string(),
            // Ship fields
            hp : String::new(),
            mass : String::new(),
        }
    }

    fn inspecting_ship_display_str() -> &'static str { "Inspecting ship" }
}

impl fmt::Display for DebugState {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        match self {
            DebugState::FreeCam => write!(f, "{}", Self::free_cam_display_str()),
            DebugState::PlacingShip { .. } => write!(f, "{}", Self::placing_ship_display_str()),
            DebugState::InspectingShip { .. } => write!(f, "{}", Self::inspecting_ship_display_str()),
        }
    }
}

pub struct StateData {
    look : Point2<f32>,
    captured_state : main_game::StateData,
    // editor state
    state : DebugState,
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
                        look : captured_state.player_pos(),
                        captured_state,
                        state : DebugState::FreeCam,
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
        use glutin::event;
        
        match event {
            event::Event::WindowEvent { event, window_id } => {
                match event {
                    event::WindowEvent::MouseInput {
                        state,
                        button,
                        ..
                    } => {
                        if *button == event::MouseButton::Left && *state == event::ElementState::Pressed {
                            match &self.state {
                                DebugState::PlacingShip { current_ship, placing, .. } if !self.pointer_inside_panel && *placing => {
                                    self.captured_state.battlefield.spawn(current_ship.clone());
                                },
                                _ => (),
                            }
                        }
                    },
                    _ => (),
                }
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
            } => match virtual_keycode {
                Some(event::VirtualKeyCode::Escape) => return Some(Box::new(Self::unwrap)),
                _ => (),
            },
            _ => (),
            // Place on mouse lbp
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
        use sys_api::graphics_init::SCREEN_WIDTH;

        let mv = input_tracker.mouse_position();
        // place the ship according to mouse pos and camera pos
        
        let look = &mut self.look;
        let state = &mut self.state;
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
                    .selected_text(format!("{}", state))
                    .show_ui(
                        ui,
                        |ui| {
                            ui.selectable_value(state, DebugState::free_cam(), DebugState::free_cam_display_str());
                            ui.selectable_value(state, DebugState::placing_ship(), DebugState::placing_ship_display_str());
                            ui.selectable_value(state, DebugState::inspecting_ship(), DebugState::inspecting_ship_display_str());
                        }
                    );

                    egui::Separator::default()
                    .horizontal()
                    .ui(ui);

                    match state {
                        DebugState::FreeCam => (),
                        DebugState::PlacingShip { current_ship, placed_ship_info, placing } => {
                            egui::ComboBox::from_label("Ship")
                            .width(150.0)
                            .show_index(
                                ui, 
                                placed_ship_info,
                                TEMPLATE_TABLE.len(),
                                |u| u.to_string()
                            );
    
                            if !*placing {
                                if egui::Button::new("place").ui(ui).clicked() { *placing = true; }
                            } else {
                                if egui::Button::new("stop placing").ui(ui).clicked() { *placing = false; }
                            }
                        },
                        DebugState::InspectingShip { id, input, hp, mass } => {

                            ui.horizontal(
                            |ui| {
                                // Text box for choosing the ship
                                if 
                                    egui::TextEdit::singleline(input)
                                    .hint_text("ship id")
                                    .desired_width(50.0f32)
                                    .ui(ui)
                                    .lost_focus() 
                                {
                                    if let Ok(new_id) = input.trim().parse() { *id = new_id; }
                                    else { *input = id.to_string() }
                                }

                                if egui::Button::new("Jump").ui(ui).clicked() {
                                    if let Some(ship) = captured_state.battlefield.get(*id) {
                                        *look = ship.core.pos;
                                    }
                                }
                            });

                            // Printing data about the ship
                            match captured_state.battlefield.get_mut(*id) {
                                None => { 
                                    ui.heading("No ship at this cell");
                                }
                                Some(ship) => {
                                    // TODO labels for the input_boxes
                                    let dir = ship.core.direction();
                                    input_box(ui, mass, &mut ship.core.mass, "mass");
                                    ui.heading(format!("pos : {}, {}", ship.core.pos.x, ship.core.pos.y));
                                    ui.heading(format!("direction : {}, {}", dir.x, dir.y));
                                    ui.heading(format!("force : {}, {}", ship.core.force.x, ship.core.force.y));
                                    input_box(ui, hp, unsafe { ship.core.hp_mut() }, "hp");
                                    ui.heading(format!("team : {:?}", ship.core.team()));
                                
                                    egui::Separator::default()
                                    .horizontal()
                                    .ui(ui);

                                    ship.engines.iter()
                                    .enumerate()
                                    .for_each(
                                        |(id, engine)| {
                                            ui.heading(format!("engine{} : {:?}", id, engine));
                                        }
                                    );
                        
                                    egui::Separator::default()
                                    .horizontal()
                                    .ui(ui);

                                    ship.guns.iter()
                                    .enumerate()
                                    .for_each(
                                        |(id, gun)| {
                                            ui.heading(format!("gun{} : {:?}", id, gun));
                                        }
                                    );
                                },
                            }
                        },
                    }
                }).response.rect
            ;
            egui.ctx().input().pointer
            .hover_pos()
            .map(|x| rect.contains(x))
            .unwrap_or(false)
        };
       
        match &mut self.state {
            DebugState::PlacingShip { current_ship, placed_ship_info, .. } if !self.pointer_inside_panel => {
                *current_ship = (TEMPLATE_TABLE[*placed_ship_info])();
                current_ship.core.pos = self.look + mv;
            },
            _ => (),
        }

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
        
        let vp = ctx.build_projection_view_matrix();
        match &self.state {
            DebugState::PlacingShip { current_ship, placing, .. } => {
                use sys_api::graphics_init::SpriteDataWriter;
                use sys_api::graphics_utils::draw_instanced_sprite;

                ctx.sprite_debug_buffer.invalidate();

                let () = {
                    //use sys_api::graphics_init::{ ENEMY_LIMIT };
                    let mut ptr = ctx.sprite_debug_buffer.map_write();
                    //if ptr.len() < ENEMY_LIMIT { panic!("Buffer too small"); }
                    for i in 0..ptr.len() { 
                        use sys_api::basic_graphics_data::ZEROED_SPRITE_DATA;            
                        ptr.set(i, ZEROED_SPRITE_DATA);
                    }

                    let mut writer = SpriteDataWriter::new(ptr);
                    (current_ship.render)(current_ship, &mut writer);
                };

                if *placing && !self.pointer_inside_panel {
                    // render the to-place ship
                    draw_instanced_sprite(ctx, frame, &ctx.sprite_debug_buffer, vp, self.captured_state.player_ship_texture.sampled(), Some(ctx.viewport()));
                }
            }
            _ => (),
        }
    }

    pub fn unwrap(_ : &mut GraphicsContext, g : GameState) -> GameState {
        match g {
            GameState::MainGameDebugMode(x) => GameState::MainGame(x.captured_state),
            _ => panic!("Called `MainGameDebugMode::unwrap` for an alien state"),
        }
    }
}
