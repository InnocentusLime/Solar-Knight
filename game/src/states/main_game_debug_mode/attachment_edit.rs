use super::*;
use ship_parts::attachment::AttachmentInfo;

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
    pub fn new(captured_state : &main_game::StateData) -> Box<dyn DebugState> {
        Box::new(
            AttachmentEdit {
                info : AttachmentInfo {
                    parent_id : 0,
                    parent_uid : 0,
                    my_uid : 0,
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
        input_box(ui, &mut self.ship_id_buff, &mut self.ship_id, "ship id");
        input_box(ui, &mut self.parent_id_buff, &mut self.info.parent_id, "parent id");
        if egui::Button::new("Add attachment").ui(ui).clicked() {
            match 
                (captured_state.battlefield.get(self.info.parent_id),
                captured_state.battlefield.get(self.ship_id))
            {
                (Some(parent), Some(ship)) => {
                    self.info.parent_uid = parent.core.uid();
                    self.info.my_uid = ship.core.uid();
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
        frame : &mut Frame, 
        captured_state : &main_game::StateData,
        ctx : &mut GraphicsContext, 
        targets : &mut RenderTargets, 
        input_tracker : &InputTracker,
        pointer_in_ui : bool,
    ) {
    }
}
