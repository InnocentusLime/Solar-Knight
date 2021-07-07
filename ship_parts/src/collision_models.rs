use collision::declare_bodies;

// TODO laser ball is a circle!!!
// TODO `declare_bodies` could use a `box` variant
declare_bodies!(
    Player = (Mesh : (-0.1f32, 0.1f32), (0.1f32, 0.1f32), (0.1f32, -0.1f32), (-0.1f32, -0.1f32));
    EnemyTester = (Mesh : (-0.1f32, 0.1f32), (0.1f32, 0.1f32), (0.1f32, -0.1f32), (-0.1f32, -0.1f32));
    BulletTester = (Mesh : (-0.06f32, 0.09f32), (0.06f32, 0.09f32), (0.06f32, -0.09f32), (-0.06f32, -0.09f32));
    LaserBall = (Mesh : (-0.03f32, 0.03f32), (0.03f32, 0.03f32), (0.03f32, -0.03f32), (-0.03f32, -0.03f32));
    LaserBeam = (Mesh : (-0.03f32, sys_api::graphics_init::SCREEN_WIDTH / 1.5f32), (0.03f32, sys_api::graphics_init::SCREEN_WIDTH / 1.5f32), (0.03f32, 0.0f32), (-0.03f32, 0.0f32));
);

pub use bodies::*;
