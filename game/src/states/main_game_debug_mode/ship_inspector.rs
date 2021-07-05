use super::*;

pub struct ShipInspector {
    // Selection of ship
    id : usize,
    input : String,
    // Ship fields
    hp : String,
    mass : String,
}

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

impl ShipInspector {
    pub fn new(captured_state : &main_game::StateData) -> Box<dyn DebugState> {
        Box::new(
            ShipInspector {
                id : 0,
                input : "".to_owned(),
                hp : "".to_owned(),
                mass : "".to_owned(),
            }
        )
    }
}

impl DebugState for ShipInspector {
    fn name(&self) -> &'static str {
        "Ship inspector"
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
        let input = &mut self.input;
        let id = &mut self.id;

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
            },
            Some(ship) => {
                // TODO labels for the input_boxes
                let dir = ship.core.direction();
                input_box(ui, &mut self.mass, &mut ship.core.mass, "mass");
                ui.heading(format!("pos : {}, {}", ship.core.pos.x, ship.core.pos.y));
                ui.heading(format!("direction : {}, {}", dir.x, dir.y));
                ui.heading(format!("force : {}, {}", ship.core.force.x, ship.core.force.y));
                input_box(ui, &mut self.hp, unsafe { ship.core.hp_mut() }, "hp");
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
