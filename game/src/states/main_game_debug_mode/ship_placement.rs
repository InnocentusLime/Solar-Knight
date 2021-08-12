use super::*;

pub struct ShipPlacement {
    placing : bool,
    placed_ship_info : usize,
    current_ship : Ship,
}

impl ShipPlacement {
    pub fn new(captured_state : &main_game::StateData) -> Box<dyn DebugState> {
        Box::new(
            ShipPlacement {
                placing : false,
                placed_ship_info : 0,
                current_ship : *captured_state.templates.get_ship(0).unwrap(),
            }
        )
    }
}

impl DebugState for ShipPlacement {
    fn name(&self) -> &'static str {
        "Ship placement"
    }
    
    fn process_event(
        &mut self,
        event : &glutin::event::Event<'static, ()>,
        captured_state : &mut main_game::StateData,
        _ctx : &mut GraphicsContext, 
        _input_tracker : &InputTracker, 
        pointer_in_ui : bool,
        _look : &mut Point2<f32>,
    ) {
        match event {
            event::Event::WindowEvent { event, .. } => {
                match event {
                    event::WindowEvent::MouseInput {
                        state,
                        button,
                        ..
                    } => {
                        if *button == event::MouseButton::Left && *state == event::ElementState::Pressed {
                            if !pointer_in_ui && self.placing {
                                captured_state.storage
                                .unlock_spawning(&mut captured_state.square_map)
                                .spawn(self.current_ship.clone());
                            }
                        }
                    },
                    _ => (),
                }
            },
            _ => (),
        }
    }

    fn update(
        &mut self,
        captured_state : &mut main_game::StateData,
        _ctx : &mut GraphicsContext, 
        input_tracker : &InputTracker, 
        _dt : Duration,
        ui : &mut Ui,
        _pointer_in_ui : bool,
        look : &mut Point2<f32>,
    ) {
        egui::ComboBox::from_label("Ship")
        .width(150.0)
        .show_index(
            ui, 
            &mut self.placed_ship_info,
            captured_state.templates.len(),
            |u| captured_state.templates.get_name(u).unwrap().to_owned()
        );
   
        if !self.placing {
            if egui::Button::new("place").ui(ui).clicked() { self.placing = true; }
        } else {
            if egui::Button::new("stop placing").ui(ui).clicked() { self.placing = false; }
        }

        if self.placing {
            self.current_ship = *captured_state.templates.get_ship(self.placed_ship_info).unwrap();
            get_component_mut::<Transform, _>(&mut self.current_ship).pos = *look + input_tracker.mouse_position();
        }
    }

    fn render(
        &self, 
        frame : &mut Frame, 
        captured_state : &main_game::StateData,
        ctx : &mut GraphicsContext, 
        targets : &mut RenderTargets, 
        _input_tracker : &InputTracker,
        pointer_in_ui : bool,
    ) {
        if self.placing && !pointer_in_ui {
            // render the to-place ship
            captured_state.render_sys.render_ship_debug(frame, ctx, targets, &self.current_ship);
        }
    }
}
