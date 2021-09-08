use super::*;
use systems::ship_attachment::AttachmentInfo;

fn input_box<T : FromStr + ToString>(ui : &mut Ui, buffer : &mut String, target : &mut T, header : &'static str) {
    ui.horizontal(
    |ui| {
        if 
            egui::TextEdit::singleline(buffer)
            .desired_width(50.0f32)
            .ui(ui)
            .lost_focus() 
        {
            if let Ok(new_target) = buffer.trim().parse() { *target = new_target; }
            else { *buffer = target.to_string() }
        }
        
        ui.heading(header);
    });
}

pub struct AttachmentEdit {
    // Actual info
    info : AttachmentInfo,
    ship_id : usize,
    // Strings
    parent_id_buff : String,
    ship_id_buff : String,
}

impl AttachmentEdit {
    pub fn new(_captured_state : &main_game::StateData) -> Box<dyn DebugState> {
        Box::new(
            AttachmentEdit {
                info : AttachmentInfo {
                    parent_id : 0,
                },
                ship_id : 0,
                parent_id_buff : "0".to_owned(),
                ship_id_buff : "0".to_owned(),
            }
        )
    }
}

impl DebugState for AttachmentEdit {

    fn name(&self) -> &'static str {
        "Add attachments"
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
        captured_state : &mut main_game::StateData,
        _ctx : &mut GraphicsContext, 
        _input_tracker : &InputTracker, 
        _dt : Duration,
        ui : &mut Ui,
        _pointer_in_ui : bool,
        _look : &mut Vector2<f32>,
    ) {
        input_box(ui, &mut self.ship_id_buff, &mut self.ship_id, "ship id");
        input_box(ui, &mut self.parent_id_buff, &mut self.info.parent_id, "parent id");
        if egui::Button::new("Add attachment").ui(ui).clicked() {
            match 
                (captured_state.storage.get(self.info.parent_id),
                captured_state.storage.get(self.ship_id))
            {
                (Some(_), Some(_)) => {
                    captured_state.attach_sys.add_attachment(self.ship_id, self.info);
                },
                (_, _) => {
                    // TODO maybe better error message
                    use log::error;
                    error!("Bad ship IDs");
                },
            }
        }
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
