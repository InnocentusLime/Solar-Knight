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
                current_ship : captured_state.battlefield.template_table[0].prefab,
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
        ctx : &mut GraphicsContext, 
        input_tracker : &InputTracker, 
        pointer_in_ui : bool,
        look : &mut Point2<f32>,
    ) {
        match event {
            event::Event::WindowEvent { event, window_id } => {
                match event {
                    event::WindowEvent::MouseInput {
                        state,
                        button,
                        ..
                    } => {
                        if *button == event::MouseButton::Left && *state == event::ElementState::Pressed {
                            if !pointer_in_ui && self.placing {
                                captured_state.battlefield.spawn(self.current_ship.clone());
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
        ctx : &mut GraphicsContext, 
        input_tracker : &InputTracker, 
        dt : Duration,
        ui : &mut Ui,
        pointer_in_ui : bool,
        look : &mut Point2<f32>,
    ) {
        egui::ComboBox::from_label("Ship")
        .width(150.0)
        .show_index(
            ui, 
            &mut self.placed_ship_info,
            captured_state.battlefield.template_table.len(),
            |u| captured_state.battlefield.template_table[u].name.to_owned()
        );
   
        if !self.placing {
            if egui::Button::new("place").ui(ui).clicked() { self.placing = true; }
        } else {
            if egui::Button::new("stop placing").ui(ui).clicked() { self.placing = false; }
        }

        if self.placing {
            self.current_ship = captured_state.battlefield.template_table[self.placed_ship_info].prefab;
            self.current_ship.core.pos = *look + input_tracker.mouse_position();
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
        use sys_api::graphics_init::SpriteDataWriter;
        use sys_api::graphics_utils::draw_instanced_sprite;
        
        let vp = ctx.build_projection_view_matrix();

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
            (self.current_ship.render)(&self.current_ship, &mut writer);
        };

        if self.placing && !pointer_in_ui {
            // render the to-place ship
            draw_instanced_sprite(ctx, frame, &ctx.sprite_debug_buffer, vp, captured_state.player_ship_texture.sampled(), Some(ctx.viewport()));
        }
    }
}
