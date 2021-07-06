use super::*;

pub struct ShipInspector {
    // Selection of ship
    id : usize,
    input : String,
    // Ship fields
    hp : String,
    mass : String,
    pos_x : String,
    pos_y : String,
}

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

fn input_box_2<T : FromStr + ToString>(ui : &mut Ui, buffer1 : &mut String, buffer2 : &mut String, target1 : &mut T, target2 : &mut T, header : &'static str) {
    ui.horizontal(
    |ui| {
        if 
            egui::TextEdit::singleline(buffer1)
            .desired_width(50.0f32)
            .ui(ui)
            .lost_focus() 
        {
            if let Ok(new_target) = buffer1.trim().parse() { *target1 = new_target; }
            else { *buffer1 = target1.to_string() }
        }
        
        if 
            egui::TextEdit::singleline(buffer2)
            .desired_width(50.0f32)
            .ui(ui)
            .lost_focus() 
        {
            if let Ok(new_target) = buffer2.trim().parse() { *target2 = new_target; }
            else { *buffer2 = target2.to_string() }
        }
        
        ui.heading(header);
    });
}

impl ShipInspector {
    // A special method for fetching ship's data and recording
    // them into the strings so the data displays correctly in
    // the textboxes
    fn update_ship_strings(&mut self, ship : &Ship) {
        self.hp = ship.core.hp().to_string();
        self.mass = ship.core.mass.to_string();
        self.pos_x = ship.core.pos.x.to_string();
        self.pos_y = ship.core.pos.y.to_string();
    }

    fn reset_ship_strings(&mut self) {
        self.hp = "".to_string();
        self.mass = "".to_string();
        self.pos_x = "".to_string();
        self.pos_y = "".to_string();
    }

    pub fn new(captured_state : &main_game::StateData) -> Box<dyn DebugState> {
        let mut res = 
            Box::new(
                ShipInspector {
                    id : 0,
                    input : "0".to_owned(),
                    hp : "".to_owned(),
                    mass : "".to_owned(),
                    pos_x : "".to_owned(),
                    pos_y : "".to_owned(),
                }
            )
        ;
        res.update_ship_strings(
            captured_state.battlefield
            .get(0).expect("Battlefield is empty")
        );
        res
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
        // Waiting for 1.54 now, I guess
        let old_id = self.id;
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

        if old_id != self.id { 
            if let Some(ship) = captured_state.battlefield.get(self.id) {
                self.update_ship_strings(ship)
            } else {
                self.reset_ship_strings()
            }
        }
        
        let id = &mut self.id;

        // Printing data about the ship
        match captured_state.battlefield.get_mut(*id) {
            None => { 
                ui.heading("No ship at this cell");
            },
            Some(ship) => {
                egui::Separator::default().horizontal().ui(ui);
                
                // TODO labels for the input_boxes
                let dir = ship.core.direction();
                input_box(ui, &mut self.mass, &mut ship.core.mass, "mass");
                input_box(ui, &mut self.hp, unsafe { ship.core.hp_mut() }, "hp");
                

                egui::Separator::default().horizontal().ui(ui);
                
                input_box_2(ui, &mut self.pos_x, &mut self.pos_y, &mut ship.core.pos.x, &mut ship.core.pos.y, "pos");
                let mut ang = dir.angle(vec2(0.0f32, 1.0f32)).0;
                ui.horizontal(|ui| {
                    ui.drag_angle(&mut ang);
                    ui.heading("direction");
                });
                ship.core.set_direction_angle(ang);
                
                egui::Separator::default().horizontal().ui(ui);
                
                ui.heading(format!("velocity : {}, {}", ship.core.velocity.x, ship.core.velocity.y));
                ui.heading(format!("force : {}, {}", ship.core.force.x, ship.core.force.y));
                ui.heading(format!("team : {:?}", ship.core.team()));
                                
                egui::Separator::default().horizontal().ui(ui);

                ship.engines.iter()
                .enumerate()
                .for_each(
                    |(id, engine)| {
                        ui.heading(format!("engine{} : {:?}", id, engine));
                    }
                );
                        
                egui::Separator::default().horizontal().ui(ui);

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
