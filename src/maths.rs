use std::ops::Add;

struct Vector2 {
    x: f32,
    y: f32,
}

impl PartialEq for Vector2 {
    fn eq(&self, other: &Self) -> bool {
        return self.x == other.x && self.y == other.y;
    }

    fn ne(&self, other: &Self) -> bool {
        return self.x != other.x || self.y != other.y;
    }
}

struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vector3 {
    // TODO: We may want to eventually turn this into a Vec3 that is composed of a Vec2 and a Z axis
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn new_xy(x: f32, y: f32) -> Self {
        Self { x, y, z: 0.0 }
    }
}

struct Vector2i {
    x: i32,
    y: i32,
}

struct RectangleType {
    x_position: i32,
    y_position: i32,
    width: i32,
    height: i32,
}

struct CircleType {
    center: Vector2,
    radius_sq: f32,
}

struct RayType {
    origin: Vector2,
    direction: Vector2,
    max_distance: f32,
    min_distance: f32,
    collision_mask: i32,
}

struct LineType {
    perpendicular: Vector2,
    direction: Vector2,
    origin: Vector2,
    end: Vector2,
    min_coord: f32,
    max_coord: f32,
    ray_intersect: Vector2,
}

struct WallPointType {
    pt_0: Vector2,
    pt_1: Vector2,
}

struct RampPlaneType {
    ball_collision_offset: Vector3,
    v1: Vector2,
    v2: Vector2,
    v3: Vector2,
    gravity_angle_1: f32,
    gravity_angle_2: f32,
    field_force: Vector2,
}

struct RectF {
    x_max: f32,
    y_max: f32,
    x_min: f32,
    y_min: f32,
}

enum FlipperIntersect {
    None = -1,
    LineA = 0,
    LineB = 1,
    CircleBase = 2,
    CircleT1 = 3,
}
