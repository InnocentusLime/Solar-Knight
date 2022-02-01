use bevy::prelude::*;

pub fn get_mouse_position(wnds : &Windows, camera_transform : &Transform) -> Option<Vec2> {
    let wnd = wnds.get_primary().unwrap();

    if let Some(pos) = wnd.cursor_position() {
        let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
        let p = pos - size / 2.0;
        let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);
        Some(pos_wld.truncate().truncate() / size.to_array()[1] * 2.0f32)
    } else { None }
}
