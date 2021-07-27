use crate::PointerTarget;
use crate::storage::{ Ship, Storage };
use crate::core::Team;
use crate::earth::Earth;
use crate::gun::BulletSystem;
use sys_api::basic_graphics_data::SpriteData;
use sys_api::graphics_init::SpriteDataWriter;
use sys_api::graphics_init::{ RenderTargets, GraphicsContext };

use glium::Frame;
use cgmath::{ Point2, Matrix4, One, EuclideanSpace, InnerSpace, vec2 };
use glium::texture::texture2d::Texture2d;
use glium::uniforms::SamplerWrapFunction;
use serde::{ Serialize, Deserialize };

#[inline]
fn point_at(from : Point2<f32>, at : Point2<f32>) -> Option<Point2<f32>> {
    use sys_api::graphics_init::SCREEN_WIDTH;

    let v = at - from;
    let x = (v.x / v.y.abs()).clamp(-SCREEN_WIDTH, SCREEN_WIDTH);
    let y = (SCREEN_WIDTH * v.y / v.x.abs()).clamp(-1.0f32, 1.0f32);
    let pointer_v = vec2(x, y);

    if pointer_v.magnitude2() > v.magnitude2() { None }
    else { Some(<Point2<f32> as EuclideanSpace>::from_vec(pointer_v)) }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RenderInfo {
    pub enemy_base_texture : bool,
}

// TODO move the rendering stuff here
// TODO should `GraphicsContext` go there
pub struct RenderSystem {
    sun_texture : Texture2d,
    earth_texture : Texture2d,
    background_texture : Texture2d,
    ship_atlas_texture : Texture2d,
    ship_atlas_uv : pack::UvCoordTable,
    player_ship_texture : Texture2d,
    player_bullet_texture : Texture2d,
    pub pointer_target : Point2<f32>,
}

// pub render : fn(&Self, &mut SpriteDataWriter),

// TODO avoid repeating code in `render_ships` and `render_ships_debug`
impl RenderSystem {
    pub fn new(ctx : &mut GraphicsContext) -> Self {
        let sun_texture = loaders::load_texture_from_file(&ctx.display, "textures/sun.png").unwrap();
        let earth_texture = loaders::load_texture_from_file(&ctx.display, "textures/earth.png").unwrap();
        let background_texture = loaders::load_texture_from_file(&ctx.display, "textures/background_game.png").unwrap();
        let player_ship_texture = loaders::load_texture_from_file(&ctx.display, "textures/player_ship.png").unwrap();
        let player_bullet_texture = loaders::load_texture_from_file(&ctx.display, "textures/player_bullet.png").unwrap();
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
            sun_texture,
            player_bullet_texture,
            earth_texture,
            background_texture,
            player_ship_texture,
            ship_atlas_texture,
            ship_atlas_uv,
            pointer_target : Point2 { x : 0.0f32, y : 0.0f32 },
        }
    }

    fn fill_ship_buffer(
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
            self.fill_ship_buffer(ship, &mut writer)
        };

        draw_instanced_sprite(ctx, frame, &ctx.sprite_debug_buffer, vp, self.ship_atlas_texture.sampled(), Some(ctx.viewport()));
    }

    pub fn render_ships(
        &self, 
        frame : &mut Frame, 
        ctx : &mut GraphicsContext, 
        _targets : &mut RenderTargets,
        battlefield : &Storage
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
                    self.fill_ship_buffer(me, &mut writer)
                }
            );
        };

        draw_instanced_sprite(ctx, frame, &ctx.enemy_buffer, vp, self.ship_atlas_texture.sampled(), Some(ctx.viewport()));
    }

    pub fn render_background(
        &self, 
        frame : &mut Frame, 
        ctx : &mut GraphicsContext, 
        _targets : &mut RenderTargets,
    ) {
        use sys_api::graphics_utils::{ draw_sprite, draw_instanced_sprite };
        use sys_api::graphics_init::SCREEN_WIDTH;
        
        let cam = -ctx.camera.disp.truncate(); 
        let picker = vec2((0.2f32 * cam.x / SCREEN_WIDTH) % 1.0f32, (0.2f32 * cam.y) % 1.0f32);
        draw_sprite(
            ctx, frame, 
            Matrix4::one(),
            (picker.x, picker.y, 1.0f32, 1.0f32),
            self.background_texture.sampled().wrap_function(SamplerWrapFunction::Repeat), 
            Some(ctx.viewport())
        );
        let picker = vec2((0.05f32 * cam.x / SCREEN_WIDTH - 0.5f32) % 1.0f32, (0.05f32 * cam.y + 0.03f32) % 1.0f32);
        draw_sprite(
            ctx, frame, 
            Matrix4::one(),
            (picker.x, picker.y, 1.0f32, 1.0f32),
            self.background_texture.sampled().wrap_function(SamplerWrapFunction::Repeat), 
            Some(ctx.viewport())
        );
    }

    pub fn render_planets(
        &self, 
        frame : &mut Frame, 
        ctx : &mut GraphicsContext, 
        _targets : &mut RenderTargets,
        earth : &Earth,
    ) {
        use sys_api::graphics_utils::{ draw_sprite, draw_instanced_sprite };
        let vp = ctx.build_projection_view_matrix();
        
        draw_sprite(
            ctx, frame, 
            vp * earth.model_mat(), 
            (0.0f32, 0.0f32, 1.0f32, 1.0f32),
            self.earth_texture.sampled(), 
            Some(ctx.viewport())
        );
        draw_sprite(
            ctx, frame, 
            vp * Matrix4::from_nonuniform_scale(0.6f32, 0.6f32, 1.0f32), 
            (0.0f32, 0.0f32, 1.0f32, 1.0f32),
            self.sun_texture.sampled(), 
            Some(ctx.viewport())
        );
    }

    // TODO make it more universal. This might
    // end up moving `PointerTarget` back into `game`
    // TODO make the pointer depend on the look position?
    pub fn render_pointer(
        &self, 
        frame : &mut Frame, 
        ctx : &mut GraphicsContext, 
        _targets : &mut RenderTargets,
        storage : &Storage,
    ) {
        use sys_api::graphics_utils::{ draw_sprite, draw_instanced_sprite };
        
        let vp = ctx.build_projection_view_matrix();
        let pointer_target = self.pointer_target;

        let pointer = 
            storage
            .get(0)
            .and_then(|y| point_at(y.core.pos, pointer_target))
        ;

        if let Some(pointer) = pointer {
            let model_mat = 
                ctx.proj_mat * 
                Matrix4::from_translation(pointer.to_vec().extend(0.0f32)) * 
                Matrix4::from_nonuniform_scale(0.1f32, 0.1f32, 1.0f32)
            ;
            draw_sprite(
                ctx, frame, 
                model_mat, 
                (0.0f32, 0.0f32, 1.0f32, 1.0f32),
                self.player_ship_texture.sampled(), 
                Some(ctx.viewport())
            )
        }
    }

    pub fn render_bullets(
        &self, 
        frame : &mut Frame, 
        ctx : &mut GraphicsContext, 
        _targets : &mut RenderTargets,
        bullet_sys : &BulletSystem,
    ) {
        use sys_api::graphics_init::PLAYER_BULLET_LIMIT;
        use sys_api::graphics_utils::{ draw_sprite, draw_instanced_sprite };
        
        let vp = ctx.build_projection_view_matrix();
        
        // Orphaning technique
        // https://stackoverflow.com/questions/43036568/when-should-glinvalidatebufferdata-be-used
        // https://www.khronos.org/opengl/wiki/Buffer_Object_Streaming
        // https://community.khronos.org/t/vbos-strangely-slow/60109
        
        ctx.bullet_buffer.invalidate();
        
        let () = {
            let mut ptr = ctx.bullet_buffer.map_write();
            if ptr.len() < PLAYER_BULLET_LIMIT { panic!("Buffer too small"); }
            for i in 0..ptr.len() { 
                use sys_api::basic_graphics_data::ZEROED_SPRITE_DATA;
            
                ptr.set(i, ZEROED_SPRITE_DATA);
            }
            bullet_sys.iter()
            .enumerate()
            .for_each(|(i, x)| {
                let m = x.model_mat();
                //dbg!(i); dbg!(m);
                
                let dat =
                    SpriteData {
                        mat_col1 : m.x.into(),
                        mat_col2 : m.y.into(),
                        mat_col3 : m.z.into(),
                        mat_col4 : m.w.into(),
                        texture_bottom_left : [0.0f32, 0.0f32],
                        width_height : [1.0f32, 1.0f32],
                        color : [1.0f32, 1.0f32, 1.0f32, 1.0f32],
                    }
                ;
            
                ptr.set(i, dat);
            });
        };
        
        draw_instanced_sprite(ctx, frame, &ctx.bullet_buffer, vp, self.player_bullet_texture.sampled(), Some(ctx.viewport()));
    }
}
