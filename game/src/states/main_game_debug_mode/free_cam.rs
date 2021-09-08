use super::*;

pub struct FreeCam;

impl FreeCam {
    pub fn new(_captured_state : &main_game::StateData) -> Box<dyn DebugState> {
        Box::new(FreeCam)
    }
}

impl DebugState for FreeCam {

    fn name(&self) -> &'static str {
        "Free camera"
    }
    
    fn process_event(
        &mut self,
        _event : &glutin::event::Event<'static, ()>,
        _captured_state : &mut main_game::StateData,
        _ctx : &mut GraphicsContext, 
        _input_tracker : &InputTracker, 
        _pointer_in_ui : bool,
        _look : &mut Vector2<f32>,
    ) {

    }

    fn update(
        &mut self,
        _captured_state : &mut main_game::StateData,
        _ctx : &mut GraphicsContext, 
        _input_tracker : &InputTracker, 
        _dt : Duration,
        _ui : &mut Ui,
        _pointer_in_ui : bool,
        _look : &mut Vector2<f32>,
    ) {

    }

    fn render(
        &self, 
        _frame : &mut Frame, 
        _captured_state : &main_game::StateData,
        _ctx : &mut GraphicsContext, 
        _targets : &mut RenderTargets, 
        _input_tracker : &InputTracker,
        _pointer_in_ui : bool,
    ) {

    }
}
