use super::*;

pub struct FreeCam;

impl FreeCam {
    pub fn new(captured_state : &main_game::StateData) -> Box<dyn DebugState> {
        Box::new(FreeCam)
    }
}

impl DebugState for FreeCam {

    fn name(&self) -> &'static str {
        "Free camera"
    }
    
    fn process_event(
        &mut self,
        event : &glutin::event::Event<'static, ()>,
        captured_state : &mut main_game::StateData,
        ctx : &mut GraphicsContext, 
        input_tracker : &InputTracker, 
        pointer_in_ui : bool,
        look : &mut Point2<f32>,
    ) {

    }

    fn update(
        &mut self,
        captured_state : &mut main_game::StateData,
        ctx : &mut GraphicsContext, 
        input_tracker : &InputTracker, 
        dt : Duration,
        ui : &mut Ui,
        pointer_in_ui : bool,
        look : &mut Point2<f32>,
    ) {

    }

    fn render(
        &self, 
        frame : &mut Frame, 
        captured_state : &main_game::StateData,
        ctx : &mut GraphicsContext, 
        targets : &mut RenderTargets, 
        input_tracker : &InputTracker,
        pointer_in_ui : bool,
    ) {

    }
}
