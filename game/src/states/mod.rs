use std::time::Duration;
use std::error::Error as StdError;

use egui_glium::EguiGlium;
use glium::{ glutin, Frame };

use sys_api::graphics_init::{ RenderTargets, GraphicsContext };
use sys_api::input_tracker::InputTracker;

/// The core data of the game.
/// Most of it is kept on stack. Everything that can't
/// be kept on stack, must be stored in a smart pointer
/// and kept as a field in this struct. 
///
/// ## The whole game is a state machine 
///
/// Event processing and upate procedures are allowed to
/// return new copies of `GameState` to indicate a state
/// transition.
///
/// The engine handles the state transitions the following
/// way:
/// * If a state transition request is encoutered during input
///   processing, the engine will remember the request, ignoring
///   all the future input and skipping the update routine, 
///   the soon-to-be-dropped state will be allowed to draw one last 
///   frame
/// * If a state transition was requested during the update routine
///   and a transition wasn't requested before, one last frame will
///   be drawn before applying the transition
///
/// ## Implementation details / Engineering guidelines
///
/// The gameloop is working the following way
/// 1. event processing
/// 2. update
/// 3. render
/// 4. transition handling
///
/// Event processing is limited in time to ensure that the game updates
/// in constant time, so it's not guaranteed that the engine will fully
/// empty the event queue before calling `update`.
///
/// As mentioned, we consider it a good choice to have the game updating
/// at constant time (+- milliseconds). It simplifies the implementation
/// of most things in general and rules out possible game design challenges
/// when the assumption that the game updated at constant speed.
/// As for the game data, ost of it is kept on stack. Everything that can't
/// be kept on stack, must be stored in a smart pointer
/// and kept as a field in this struct. 
/// 
/// ## The whole game is a state machine 
/// 
/// Event processing and upate procedures are allowed to
/// return new copies of `GameState` to indicate a state
/// transition.
///
/// The engine handles the state transitions the following
/// way:
/// * If a state transition request is encoutered during input
///   processing, the engine will remember the request, ignoring
///   all the future input and skipping the update routine, 
///   the soon-to-be-dropped state will be allowed to draw one last 
///   frame
/// * If a state transition was requested during the update routine 
///   and a transition wasn't requested before, one last frame will
///   be drawn before applying the transition
///
/// ## Implementation details / Engineering guidelines
///
/// The gameloop is working the following way
/// 1. event processing
/// 2. update
/// 3. render
/// 4. transition handling
///
/// Event processing is limited in time to ensure that the game updates
/// in constant time, so it's not guaranteed that the engine will fully
/// empty the event queue before calling `update`.
///
/// The rendering routine *MUST NOT MODIFY THE STATE*. It must be *READ ONLY*.
/// That means that all updates, which *SEEM* like they belong to rendering
/// (i.e. animations) must be handled by the `update` routine.
///
/// During the state transitioning from `A` to `B`, `A` gets `consumed`, so
/// `B` can reuse some textures and other data. That means that it is 
/// *VERY INLIKELY* that `A`'s destructor will be called. That means that
/// *NONE* of the states can have fields which can cause a memory leak if
/// the state's destructor wasn't called
///
/// Since states "ask" for a transition through the return-code, one *MUST NOT*
/// do a state transition inside the `update` or `process_event` function.
///
/// Despite the engine having code for handling the cases where multiple state
/// transitions can be requested one *SHOULD NOT* construct code which creates
/// more than one transition request.
pub enum GameState {
    /// This state indicates that the engine should quit.
    /// Make sure that all code which causes the game to shutdown
    /// makes the engine reach that state
    Quit,
    /// This state is used to indicate that
    /// the user moved the state and didn't bring
    /// it back
    Empty,
    /// This state is used to mark a failure. Creating
    /// it is cheap, because it doesn't require any textures.
    /// This state attempts to log the error and then goes to
    /// `Quitting` state.
    Failure(Box<dyn StdError>),
    /// This is the first state that the engine picks. Can be
    /// used for some pre-game initialization.
    Booting(booting::StateData),
    MainMenu(main_menu::StateData),
    /// The main game
    MainGame(main_game::StateData),
    /// The debug mode for main game.
    /// It plugs right into the main game state allowing the on-fly
    /// editing of the data.
    MainGameDebugMode(main_game_debug_mode::StateData),
    /// The state used purely for testing of different features like
    /// gui or `sys_api`
    Testing(testing::StateData),
}

impl GameState {
    #[inline]
    pub fn boot_state(ctx : &mut GraphicsContext) -> Self {
        booting::StateData::init(ctx)
    }

    #[inline]
    pub fn failure_state_request(x : Box<dyn StdError>) -> TransitionRequest {
        Box::new(|_, _| GameState::Failure(x))
    }

    /// The event processing procedure of the state.
    #[inline]
    pub fn process_event(
        &mut self, 
        ctx : &mut GraphicsContext, 
        input_tracker : &InputTracker, 
        event : &glutin::event::Event<'static, ()>,
    ) -> Option<TransitionRequest> {
        match self {
            GameState::Booting(x) => x.process_event(ctx, input_tracker, event),
            GameState::MainMenu(x) => x.process_event(ctx, input_tracker, event),
            GameState::MainGame(x) => x.process_event(ctx, input_tracker, event),
            GameState::MainGameDebugMode(x) => x.process_event(ctx, input_tracker, event),
            GameState::Testing(x) => x.process_event(ctx, input_tracker, event),
            _ => None,
        }
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
        match self {
            GameState::Booting(x) => x.update(ctx, input_tracker, dt, egui),
            GameState::MainMenu(x) => x.update(ctx, input_tracker, dt, egui),
            GameState::MainGame(x) => x.update(ctx, input_tracker, dt, egui),
            GameState::MainGameDebugMode(x) => x.update(ctx, input_tracker, dt, egui),
            GameState::Testing(x) => x.update(ctx, input_tracker, dt, egui),
            _ => None,
        }
    }

    #[inline]
    pub fn render(
        &self, 
        frame : &mut Frame,
        ctx : &mut GraphicsContext, 
        targets : &mut RenderTargets, 
        input_tracker : &InputTracker
    ) {
        match self {
            GameState::Booting(x) => x.render(frame, ctx, targets, input_tracker),
            GameState::MainMenu(x) => x.render(frame, ctx, targets, input_tracker),
            GameState::MainGame(x) => x.render(frame, ctx, targets, input_tracker),
            GameState::MainGameDebugMode(x) => x.render(frame, ctx, targets, input_tracker),
            GameState::Testing(x) => x.render(frame, ctx, targets, input_tracker),
            _ => (),
        }
    }
}

/// The type alias for transition request to speed up the code production.
/// A transition request is pretty much an initialization procedure for a state.
pub type TransitionRequest = Box<dyn FnOnce(&mut GraphicsContext, GameState) -> GameState>;

/// The fps at which the game updates
pub const TICKS_PER_SECOND : u32 = 60;
pub const FRAMES_PER_SECOND : u32 = 60;

/// List of modules which are present
pub mod booting;
pub mod main_menu;
pub mod main_game;
pub mod main_game_debug_mode;
pub mod testing;
