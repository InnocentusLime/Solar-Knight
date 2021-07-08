use crate::storage_traits::Battlefield;
use crate::core::Team;
use crate::storage_traits::Ship;
use sys_api::basic_graphics_data::SpriteData;
use sys_api::graphics_init::SpriteDataWriter;
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };

use glium::Frame;
use glium::texture::texture2d::Texture2d;

use crate::loaders::texture_load_from_file;

#[derive(Clone, Copy)]
pub struct RenderInfo {

}

// TODO move the rendering stuff here
// TODO should `GraphicsContext` go there
pub struct RenderSystem {
    player_ship_texture : Texture2d, 
}

// pub render : fn(&Self, &mut SpriteDataWriter),

impl RenderSystem {
    pub fn new(ctx : &mut GraphicsContext) -> Self {
        let player_ship_texture = texture_load_from_file(&ctx.display, "textures/player_ship.png").unwrap();

        RenderSystem {
            player_ship_texture,
        }
    }

    fn fill_buffer(
        me : &Ship,
        buff : &mut SpriteDataWriter,
    ) {
        let m = me.model_mat((0.1f32, 0.1f32));
        //dbg!(i); dbg!(m);
    
        let color : [f32; 4];
        if me.core.team() == Team::Hive { color = [1.0f32, 0.01f32, 0.01f32, 1.0f32] }
        else { color = [1.0f32; 4] }
            
        let dat =
            SpriteData {
                mat_col1 : m.x.into(),
                mat_col2 : m.y.into(),
                mat_col3 : m.z.into(),
                mat_col4 : m.w.into(),
                texture_bottom_left : [0.0f32, 0.0f32],
                width_height : [1.0f32, 1.0f32],
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
            Self::fill_buffer(ship, &mut writer)
        };

        draw_instanced_sprite(ctx, frame, &ctx.sprite_debug_buffer, vp, self.player_ship_texture.sampled(), Some(ctx.viewport()));
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
                    Self::fill_buffer(me, &mut writer)
                }
            );
        };

        draw_instanced_sprite(ctx, frame, &ctx.enemy_buffer, vp, self.player_ship_texture.sampled(), Some(ctx.viewport()));
    }
}
