use collision::declare_bodies;

declare_bodies!(
    Player = (Mesh : (-0.1f32, 0.1f32), (0.1f32, 0.1f32), (0.1f32, -0.1f32), (-0.1f32, -0.1f32));
    EnemyTester = (Mesh : (-0.1f32, 0.1f32), (0.1f32, 0.1f32), (0.1f32, -0.1f32), (-0.1f32, -0.1f32));
    BulletTester = (Mesh : (-0.06f32, 0.09f32), (0.06f32, 0.09f32), (0.06f32, -0.09f32), (-0.06f32, -0.09f32));
);

pub use bodies::*;
