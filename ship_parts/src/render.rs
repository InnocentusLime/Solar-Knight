use crate::storage_traits::Battlefield;
use crate::core::Team;
use crate::storage_traits::Ship;
use sys_api::basic_graphics_data::SpriteData;
use sys_api::graphics_init::SpriteDataWriter;
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };

use glium::Frame;
use glium::texture::texture2d::Texture2d;
use serde::{ Serialize, Deserialize };

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RenderInfo {
    pub enemy_base_texture : bool,
}

// TODO move the rendering stuff here
// TODO should `GraphicsContext` go there
pub struct RenderSystem {
    ship_atlas_texture : Texture2d,
    ship_atlas_uv : pack::UvCoordTable,
    player_ship_texture : Texture2d, 
}

// pub render : fn(&Self, &mut SpriteDataWriter),

// TODO avoid repeating code in `render_ships` and `render_ships_debug`
impl RenderSystem {
    pub fn new(ctx : &mut GraphicsContext) -> Self {
        let player_ship_texture = loaders::load_texture_from_file(&ctx.display, "textures/player_ship.png").unwrap();
        let (ship_atlas_texture, ship_atlas_uv) = 
            loaders::load_atlas_uv_from_file(
                &ctx.display,
                vec![
                    ("player".to_owned(), "textures/player_ship.png"),
                    ("enemy_base".to_owned(), "textures/enemy_ship_base.png"),
                ]
            )
            .unwrap()
        ;

        RenderSystem {
            player_ship_texture,
            ship_atlas_texture,
            ship_atlas_uv,
        }
    }

    fn fill_buffer(
        &self,
        me : &Ship,
        buff : &mut SpriteDataWriter,
    ) {
        let m = me.model_mat((0.1f32, 0.1f32));
        //dbg!(i); dbg!(m);
    
        let color : [f32; 4];
        if me.core.team() == Team::Hive && !me.render.enemy_base_texture { color = [1.0f32, 0.01f32, 0.01f32, 1.0f32] }
        else { color = [1.0f32; 4] }
           
        let cell = 
            if me.render.enemy_base_texture {
                self.ship_atlas_uv.entries["enemy_base"]
            } else {
                self.ship_atlas_uv.entries["player"]
            }
        ;

        let dat =
            SpriteData {
                mat_col1 : m.x.into(),
                mat_col2 : m.y.into(),
                mat_col3 : m.z.into(),
                mat_col4 : m.w.into(),
                texture_bottom_left : [cell.left, cell.bottom],
                width_height : [(cell.right - cell.left), (cell.top - cell.bottom)],
                color : color,
            }
        ;

        buff.put(dat);
    }

    pub fn render_ship_debug(
        &self, 
        frame : &mut Frame, 
        ctx : &mut GraphicsContext, 
        _targets : &mut RenderTargets,
        ship : &Ship,
    ) {
        use sys_api::graphics_init::SPRITE_DEBUG_LIMIT;
        use sys_api::graphics_utils::{ draw_sprite, draw_instanced_sprite };
        
        let vp = ctx.build_projection_view_matrix();
        
        ctx.sprite_debug_buffer.invalidate();
        let () = {
            let mut ptr = ctx.sprite_debug_buffer.map_write();
            let () = {
                if ptr.len() < SPRITE_DEBUG_LIMIT { panic!("Buffer too small"); }
                for i in 0..ptr.len() { 
                    use sys_api::basic_graphics_data::ZEROED_SPRITE_DATA;
                
                    ptr.set(i, ZEROED_SPRITE_DATA);
                }
            };
            let mut writer = SpriteDataWriter::new(ptr);
   
            // tester-render
            self.fill_buffer(ship, &mut writer)
        };

        draw_instanced_sprite(ctx, frame, &ctx.sprite_debug_buffer, vp, self.ship_atlas_texture.sampled(), Some(ctx.viewport()));
    }

    pub fn render_ships(
        &self, 
        frame : &mut Frame, 
        ctx : &mut GraphicsContext, 
        _targets : &mut RenderTargets,
        battlefield : &Battlefield
    ) {
        use sys_api::graphics_init::ENEMY_LIMIT;
        use sys_api::graphics_utils::{ draw_sprite, draw_instanced_sprite };
        
        let vp = ctx.build_projection_view_matrix();
        
        ctx.enemy_buffer.invalidate();
        let () = {
            let mut ptr = ctx.enemy_buffer.map_write();
            let () = {
                if ptr.len() < ENEMY_LIMIT { panic!("Buffer too small"); }
                for i in 0..ptr.len() { 
                    use sys_api::basic_graphics_data::ZEROED_SPRITE_DATA;
                
                    ptr.set(i, ZEROED_SPRITE_DATA);
                }
            };
            let mut writer = SpriteDataWriter::new(ptr);
   
            // tester-render
            battlefield.iter()
            .for_each(
                |me| {
                    self.fill_buffer(me, &mut writer)
                }
            );
        };

        draw_instanced_sprite(ctx, frame, &ctx.enemy_buffer, vp, self.ship_atlas_texture.sampled(), Some(ctx.viewport()));
    }
}
