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
        self.hp = get_component::<HpInfo, _>(ship).hp().to_string();
        self.mass = get_component::<PhysicsData, _>(ship).mass.to_string();
        self.pos_x = get_component::<Transform, _>(ship).pos.x.to_string();
        self.pos_y = get_component::<Transform, _>(ship).pos.y.to_string();
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
            captured_state.storage
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
        _event : &glutin::event::Event<'static, ()>,
        _captured_state : &mut main_game::StateData,
        _ctx : &mut GraphicsContext, 
        _input_tracker : &InputTracker, 
        _pointer_in_ui : bool,
        _look : &mut Point2<f32>,
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
        look : &mut Point2<f32>,
    ) {
        // Waiting for 1.53
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
                if let Some(ship) = captured_state.storage.get(*id) {
                    *look = get_component::<Transform, _>(ship).pos;
                }
            }
        });

        if old_id != self.id { 
            if let Some(ship) = captured_state.storage.get(self.id) {
                self.update_ship_strings(ship)
            } else {
                self.reset_ship_strings()
            }
        }
        
        let id = &mut self.id;

        // Printing data about the ship
        captured_state.storage
        .unlock_mutations(&mut captured_state.square_map)
        .mutate(*id, |ship, _| {
            // Editable scalar data
            egui::Separator::default().horizontal().ui(ui);
                
            input_box(ui, &mut self.mass, &mut get_component_mut::<PhysicsData, _>(ship).mass, "mass");
            input_box(ui, &mut self.hp, unsafe { get_component_mut::<HpInfo, _>(ship).hp_mut() }, "hp");
                

            // Editable non-scalar data
            egui::Separator::default().horizontal().ui(ui);
               
            let transform = get_component_mut::<Transform, _>(ship);
            input_box_2(ui, &mut self.pos_x, &mut self.pos_y, &mut transform.pos.x, &mut transform.pos.y, "pos");
            let dir = transform.direction();
            let mut ang = dir.angle(vec2(0.0f32, 1.0f32)).0;
            ui.horizontal(|ui| {
                ui.drag_angle(&mut ang);
                ui.heading("direction");
            });
            transform.set_direction_angle(ang);
                
            // Internal (or current uneditable) data
            egui::Separator::default().horizontal().ui(ui);
                
            let phys = get_component::<PhysicsData, _>(ship);
            ui.heading(format!("velocity : {}, {}", phys.velocity.x, phys.velocity.y));
            ui.heading(format!("force : {}, {}", phys.force.x, phys.force.y));
            ui.heading(format!("team : {:?}", *get_component::<Team, _>(ship)));
            ui.heading(format!("square_id : {:?}", get_component::<SquareMapNode, _>(ship).square_id()));
            ui.heading(format!("ai_routine : {:?}", get_component::<AiTag, _>(ship)));
                               
            // engines
            egui::Separator::default().horizontal().ui(ui);

            /*
            ship.engines.iter()
            .enumerate()
            .for_each(
                |(id, engine)| {
                    ui.heading(format!("engine{} : {:?}", id, engine));
                }
            );
            */
                        
            // guns
            egui::Separator::default().horizontal().ui(ui);

            /*
            ship.guns.iter()
            .enumerate()
            .for_each(
                |(id, gun)| {
                    ui.heading(format!("gun{} : {:?}", id, gun));
                }
            );
            */
        }).unwrap_or_else(|| { ui.heading("No ship at this cell"); });
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
