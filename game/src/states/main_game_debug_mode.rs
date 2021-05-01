use std::fmt;
use std::time::Duration;
use std::error::Error as StdError;

use super::{ TransitionRequest, GameState, main_game };

use log::trace;
use glium::{ Frame, Surface, glutin };
use lazy_static::lazy_static;
use glutin::event;
use cgmath::{ EuclideanSpace, Point2, point2, vec2 };

//use rags::Context as RagsContext;
use ship_parts::Ship;
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;

//use crate::rags_impl::RagsBackend;

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

enum DebugState {
    FreeCam,
    PlacingShip {
        placed_ship_info : usize,
        current_ship : Ship,
    },
}

pub struct StateData {
//    rctx : RagsContext,
    look : Point2<f32>,
    captured_state : main_game::StateData,
    // editor state
    state : DebugState,
}

impl StateData {
    pub fn init(ctx : &mut GraphicsContext, old : GameState) -> GameState {
        trace!("Debug mode entered");

        match old {
            GameState::MainGame(captured_state) => { 
                GameState::MainGameDebugMode(
                    StateData {
//                        rctx : RagsContext::new(ctx.display.get_framebuffer_dimensions()),
                        look : captured_state.player_pos(),
                        captured_state,
                        state : DebugState::FreeCam,
                    }
                )
            },
            _ => GameState::Failure(Box::new(DebugModeEntranceError)),
        }
    }

    /// The event processing procedure of the state.
    #[inline]
    pub fn process_event(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, event : &glutin::event::Event<()>) -> Option<TransitionRequest> {
        use glutin::event;
        
        match event {
            event::Event::WindowEvent { event, window_id } => match event {
                event::WindowEvent::MouseInput {
                    state,
                    button,
                    ..
                } => {
                    if *button == event::MouseButton::Left {
                        /*
                        if *state == event::ElementState::Pressed { self.rctx.input.press_mouse(); }
                        else { self.rctx.input.release_mouse(); }
                        */
                        match &self.state {
                            DebugState::PlacingShip { current_ship, .. } => {
                                trace!("Ship spawned");
                                self.captured_state.battlefield.spawn(current_ship.clone());
                            },
                            _ => (),
                        }
                    }
                },
                _ => (),
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
                Some(event::VirtualKeyCode::Escape) => {
                    trace!("Debug mode terminated");
                    return Some(Box::new(Self::unwrap))
                },
                // Key for entering ship placement
                Some(event::VirtualKeyCode::P) => {
                    trace!("Entered ship placement");
                    self.state = DebugState::PlacingShip { placed_ship_info : 0, current_ship : (TEMPLATE_TABLE[0])() };
                },
                // Key for scrolling through ships
                Some(event::VirtualKeyCode::N) => {
                    match &mut self.state {
                        DebugState::PlacingShip { placed_ship_info, current_ship } => {
                            *placed_ship_info = (*placed_ship_info + 1) % TEMPLATE_TABLE.len();
                            *current_ship = (TEMPLATE_TABLE[*placed_ship_info])();
                            trace!("Selected ship ID {}", placed_ship_info);
                        }
                        _ => (),
                    }
                },
                // Key for entering free-cam
                Some(event::VirtualKeyCode::C) => {
                    trace!("Entered free cam");
                    self.state = DebugState::FreeCam;
                },
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
    pub fn update(&mut self, ctx : &mut GraphicsContext, input_tracker : &InputTracker, dt : Duration) -> Option<TransitionRequest> {
        use glutin::event::VirtualKeyCode as Key;
        use sys_api::graphics_init::SCREEN_WIDTH;

        let mv = input_tracker.mouse_position();
        
        match &mut self.state {
            DebugState::PlacingShip { current_ship, .. } => {
                current_ship.core.pos = self.look + mv;
            },
            _ => (),
        }
        /*
        let (width, height) = self.rctx.dimensions();
        self.rctx.input.place_mouse(
            (
                ((mv.x / (2.0f32 * SCREEN_WIDTH) + 0.5f32) * (width as f32)) as u32, 
                (-(mv.y / 2.0f32 - 0.5f32) * (height as f32)) as u32
            )
        );
        self.rctx.update();
        self.rctx.clear_commands();
        self.rctx.draw_box((0, 0), (100, 100), (0, 0, 120, 100));
        */

        if input_tracker.is_key_down(Key::W) { self.look += dt.as_secs_f32() * vec2(0.0f32, 1.0f32); }
        if input_tracker.is_key_down(Key::A) { self.look += dt.as_secs_f32() * vec2(-1.0f32, 0.0f32); }
        if input_tracker.is_key_down(Key::S) { self.look += dt.as_secs_f32() * vec2(0.0f32, -1.0f32); }
        if input_tracker.is_key_down(Key::D) { self.look += dt.as_secs_f32() * vec2(1.0f32, 0.0f32); }

        ctx.camera.disp = (-self.look.to_vec()).extend(0.0f32);

        // place the ship according to mouse pos and camera pos

        None
    }

    #[inline]
    pub fn render(&mut self, frame : &mut Frame, ctx : &mut GraphicsContext, targets : &mut RenderTargets, input_tracker : &InputTracker) {
        self.captured_state.render(frame, ctx, targets, input_tracker);
        
        let vp = ctx.build_projection_view_matrix();
        match &self.state {
            DebugState::PlacingShip { current_ship, .. } => {
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

                draw_instanced_sprite(ctx, frame, &ctx.sprite_debug_buffer, vp, self.captured_state.player_ship_texture.sampled(), Some(ctx.viewport()));
            }
            _ => (),
        }

        // render the to-place ship

        /*
        self.rctx.done(&mut RagsBackend::new(&mut frame, ctx));
        */
    }

    pub fn unwrap(_ : &mut GraphicsContext, g : GameState) -> GameState {
        match g {
            GameState::MainGameDebugMode(x) => GameState::MainGame(x.captured_state),
            _ => panic!("Called `MainGameDebugMode::unwrap` for an alien state"),
        }
    }
}
