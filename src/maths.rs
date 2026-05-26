use std::ops::Add;

pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl PartialEq for Vector2 {
    fn eq(&self, other: &Self) -> bool {
        return self.x == other.x && self.y == other.y;
    }

    fn ne(&self, other: &Self) -> bool {
        self.x != other.x || self.y != other.y
    }
}

pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    // TODO: We may want to eventually turn this into a Vec3 that is composed of a Vec2 and a Z axis
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn new_xy(x: f32, y: f32) -> Self {
        Self { x, y, z: 0.0 }
    }
}

pub struct Vector2i {
    pub x: i32,
    pub y: i32,
}

pub struct RectangleType {
    pub x_position: i32,
    pub y_position: i32,
    pub width: i32,
    pub height: i32,
}

pub struct CircleType {
    pub center: Vector2,
    pub radius_sq: f32,
}

pub struct RayType {
    pub origin: Vector2,
    pub direction: Vector2,
    pub max_distance: f32,
    pub min_distance: f32,
    pub collision_mask: i32,
}

pub struct LineType {
    pub perpendicular: Vector2,
    pub direction: Vector2,
    pub origin: Vector2,
    pub end: Vector2,
    pub min_coord: f32,
    pub max_coord: f32,
    pub ray_intersect: Vector2,
}

pub struct WallPointType {
    pub pt_0: Vector2,
    pub pt_1: Vector2,
}

pub struct RampPlaneType {
    pub ball_collision_offset: Vector3,
    pub v1: Vector2,
    pub v2: Vector2,
    pub v3: Vector2,
    pub gravity_angle_1: f32,
    pub gravity_angle_2: f32,
    pub field_force: Vector2,
}

pub struct RectF {
    pub x_max: f32,
    pub y_max: f32,
    pub x_min: f32,
    pub y_min: f32,
}

pub enum FlipperIntersect {
    None = -1,
    LineA = 0,
    LineB = 1,
    CircleBase = 2,
    CircleT1 = 3,
}
