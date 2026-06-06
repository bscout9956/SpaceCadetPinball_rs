use crate::t_ball::TBall;
use crate::t_flipper_edge::TFlipperEdge;

#[derive(Copy, Clone, Debug, PartialOrd)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    fn from_vec3(vec3: Vector3) -> Self {
        Self {
            x: vec3.x,
            y: vec3.y,
        }
    }
}

impl PartialEq for Vector2 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn new_xy(x: f32, y: f32) -> Self {
        Self { x, y, z: 0.0 }
    }

    pub fn from_vec2(vec2: Vector2, z: f32) -> Self {
        Self {
            x: vec2.x,
            y: vec2.y,
            z,
        }
    }
}

pub struct Vector2i {
    pub x: i32,
    pub y: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
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

impl RectF {
    fn merge(&mut self, aabb: &RectF) {
        self.x_max = f32::max(self.x_max, aabb.x_max);
        self.y_max = f32::max(self.y_max, aabb.y_max);
        self.x_min = f32::min(self.x_min, aabb.x_min);
        self.y_min = f32::min(self.y_min, aabb.y_min);
    }
}

// Performs AABB merge, creating rect that is just large enough to contain both source rects.
pub fn enclosing_box(rect1: &RectangleType, rect2: &RectangleType, dest_rect: &mut RectangleType) {
    let mut x_pos = rect1.x_position;
    let mut width = rect1.width;

    if (rect2.x_position < rect1.x_position) {
        x_pos = rect2.x_position;
        width += rect1.x_position - rect2.x_position;
    }

    let mut y_pos = rect1.y_position;
    let mut height = rect1.height;

    if (rect2.y_position < rect1.y_position) {
        y_pos = rect2.y_position;
        height += rect1.y_position - rect2.y_position;
    }

    let mut x_end_2 = rect2.x_position + rect2.width;
    if x_end_2 > x_pos + width {
        width = x_end_2 - x_pos;
    }

    let mut y_end_2 = rect2.y_position + rect2.height;
    if y_end_2 > y_pos + height {
        height = y_end_2 - y_pos;
    }

    dest_rect.x_position = x_pos;
    dest_rect.y_position = y_pos;
    dest_rect.width = width;
    dest_rect.height = height;
}

pub fn rectangle_clip(
    rect1: &RectangleType,
    rect2: &RectangleType,
    dest: &mut RectangleType,
) -> bool {
    let x_end_2 = rect2.x_position + rect2.width;
    if rect2.x_position >= rect1.x_position + rect1.width || rect1.x_position >= x_end_2 {
        return false;
    }

    let y_end_2 = rect2.y_position + rect2.height;
    if rect2.y_position >= rect1.y_position + rect1.height || rect1.y_position >= y_end_2 {
        return false;
    }

    let mut x_pos = rect1.x_position;
    let mut width = rect1.width;
    if (rect1.x_position < rect2.x_position) {
        x_pos = rect2.x_position;
        width += rect1.x_position - rect2.x_position;
    }

    let mut y_pos = rect1.y_position;
    let mut height = rect1.height;
    if rect1.y_position < rect2.y_position {
        y_pos = rect2.y_position;
        height += rect1.y_position - rect2.y_position;
    }

    if x_pos + width > x_end_2 {
        width = x_end_2 - x_pos;
    }
    if y_pos + height > y_end_2 {
        height = y_end_2 - y_pos;
    }

    if width == 0 || height == 0 {
        return false;
    }

    dest.x_position = x_pos;
    dest.y_position = y_pos;
    dest.width = width;
    dest.height = height;

    true
}

// Returns the distance from ray origin to the first ray-circle intersection point.
pub fn ray_intersect_circle(ray: &RayType, circle: &CircleType) -> f32 {
    // O - ray origin
    // D - ray direction
    // C - circle center
    // R - circle radius
    // l, C - O, vector between O and C
    let l = vector_sub_vec2(&circle.center, &ray.origin);

    // Tca, l dot D, projection of l on D
    let tca = dot_product(&l, &ray.direction);
    if tca < 0.0 {
        return 1_000_000_000.0;
    }

    // l dot l, distance from ray origin to circle center
    let l_mag_sq = dot_product(&l, &l);

    // Thc^2 = rad^2 - d^2; d = sqrt(l dot l - Tca * Tca)
    let thc_sq = circle.radius_sq - l_mag_sq + tca * tca;

    // T0 = Tca - Thc, distance from origin to first intersection
    // If ray origin is inside the circle, then T0 is negative
    if l_mag_sq < circle.radius_sq {
        return tca - f32::sqrt(thc_sq);
    }

    // No intersection if ThcSq is negative, that is if d > rad
    if thc_sq < 0.0 {
        return 1_000_000_000.0;
    }

    let t0: f32 = tca - f32::sqrt(thc_sq);
    if t0 < 0.0 || t0 > ray.max_distance {
        return 1_000_000_000.0;
    }

    t0
}

pub fn normalize_2d(vec2: &mut Vector2) -> f32 {
    let mag: f32 = f32::sqrt(vec2.x * vec2.x + vec2.y * vec2.y);
    if mag != 0.0 {
        vec2.x /= mag;
        vec2.y /= mag;
    }
    mag
}

pub fn line_init(line: &mut LineType, x0: f32, y0: f32, x1: f32, y1: f32) {
    line.origin = Vector2 { x: x0, y: y0 };
    line.end = Vector2 { x: x1, y: y1 };
    line.direction.x = x1 - x0;
    line.direction.y = y1 - y0;
    normalize_2d(&mut line.direction);

    // Clockwise perpendicular to the line direction vector
    line.perpendicular = Vector2 {
        x: line.direction.y,
        y: -line.direction.x,
    };

    let mut line_start = x0;
    let mut line_end = x1;

    if f32::abs(line.direction.x) < 0.000_000_001 {
        line.direction.x = 0.0;
        line_start = y0;
        line_end = y1;
    }

    line.min_coord = f32::min(line_start, line_end);
    line.max_coord = f32::max(line_start, line_end);
}

// Returns the distance from ray origin to the ray-line segment intersection point.
// Stores ray-line intersection point in line.RayIntersect
pub fn ray_intersect_line(ray: &RayType, line: &mut LineType) -> f32 {
    // V1 vector between ray origin and line origin
    // V2 ray direction
    // V3 line perpendicular clockwise
    let mut v1 = vector_sub_vec2(&ray.origin, &line.origin);
    let mut v2 = line.direction;
    let v3 = Vector2 {
        x: -ray.direction.y,
        y: ray.direction.x,
    };

    // Project line on ray perpendicular, no intersection if ray is pointing away from the line
    let v2_dot_v3 = dot_product(&v2, &v3);
    if (v2_dot_v3 < 0.0) {
        // Distance to the intersect point: (V2 X V1) / (V2 dot V3)
        let distance = cross(&v2, &v1) / v2_dot_v3;
        if (distance >= -ray.min_distance && distance <= ray.max_distance) {
            line.ray_intersect.x = distance * ray.direction.x + ray.origin.x;
            line.ray_intersect.y = distance * ray.direction.y + ray.origin.y;

            // Check if intersection point is inside line segment
            let test_point = if line.direction.x != 0.0 {
                line.ray_intersect.x
            } else {
                line.ray_intersect.y
            };
            if (test_point >= line.min_coord && test_point <= line.max_coord) {
                return distance;
            }
        }
    }

    1_000_000_000.0
}

pub fn cross_mut(vec1: &Vector3, vec2: &Vector3, dst_vect: &mut Vector3) {
    dst_vect.x = vec2.z * vec1.y - vec2.y * vec1.z;
    dst_vect.y = vec2.x * vec1.z - vec1.x * vec2.z;
    dst_vect.z = vec1.x * vec2.y - vec2.x * vec1.y;
}

pub fn cross(vec1: &Vector2, vec2: &Vector2) -> f32 {
    vec1.x * vec2.y - vec1.y * vec2.x
}

pub fn magnitude(vec: &Vector3) -> f32 {
    let mut result;
    let mag_sq = vec.x * vec.x + vec.y * vec.y + vec.z * vec.z;
    if mag_sq == 0.0 {
        result = 0.0;
    } else {
        result = mag_sq.sqrt();
    }
    result
}

pub fn magnitude_sq(vec: &Vector2) -> f32 {
    vec.x * vec.x + vec.y * vec.y
}

pub fn magnitude_sq_i(vec: &Vector2i) -> i32 {
    vec.x * vec.x + vec.y * vec.y
}

pub fn vector_add(vec1_dst: &mut Vector2, vec2: &Vector2) {
    vec1_dst.x += vec2.x;
    vec1_dst.y += vec2.y;
}

pub fn vector_sub_vec2(vec1: &Vector2, vec2: &Vector2) -> Vector2 {
    Vector2 {
        x: vec1.x - vec2.x,
        y: vec1.y - vec2.y,
    }
}

pub fn vector_sub_vec3(vec1: &Vector3, vec2: &Vector3) -> Vector3 {
    Vector3 {
        x: vec1.x - vec2.x,
        y: vec1.y - vec2.y,
        z: vec1.z - vec2.z,
    }
}

pub fn vector_mul(vec1: &Vector2, val: f32) -> Vector2 {
    Vector2 {
        x: vec1.x * val,
        y: vec1.y * val,
    }
}

pub fn basic_collision(
    ball: &mut TBall,
    next_position: &Vector2,
    direction: &Vector2,
    elasticity: f32,
    smoothness: f32,
    threshold: f32,
    boost: f32,
) -> f32 {
    ball.position.x = next_position.x + direction.x * 0.0005;
    ball.position.y = next_position.y + direction.y * 0.0005;

    // Project ball direction on collision rebound direction
    // Unsure what to do with Z component, the original defines Vec2:Vec3
    // Rust doesn't support inheritance
    let mut rebound_proj = -dot_product(&direction, &Vector2::from_vec3(ball.direction));
    if rebound_proj < 0.0 {
        // Negative projection means no rebound, both direction vectors point the same way.
        rebound_proj = -rebound_proj;
    } else {
        let dx1: f32 = rebound_proj * direction.x;
        let dy1: f32 = rebound_proj * direction.y;
        ball.direction.x = (dx1 + ball.direction.x) * smoothness + dx1 * elasticity;
        ball.direction.y = (dy1 + ball.direction.y) * smoothness + dy1 * elasticity;
        // We're copying the ball to a mutable Vector2, so we mutate it and reassign back to the
        // original ball
        let ball_copy = &mut Vector2::from_vec3(ball.direction);
        normalize_2d(ball_copy);
        ball.direction = Vector3 {
            x: ball_copy.x,
            y: ball_copy.y,
            z: ball.direction.z,
        };
    }
    let rebound_speed = rebound_proj * ball.speed;
    ball.speed -= (1.0 - elasticity) * rebound_speed;

    if rebound_speed > threshold {
        // Change ball direction if rebound speed is above threshold
        ball.direction.x = ball.speed * ball.direction.x + direction.x * boost;
        ball.direction.y = ball.speed * ball.direction.y + direction.y * boost;
    }

    rebound_speed
}

pub fn distance_squared(vec1: &Vector2, vec2: &Vector2) -> f32 {
    let dx = vec1.x - vec2.x;
    let dy = vec1.y - vec2.y;
    dy * dy + dx * dx
}

pub fn dot_product(vec1: &Vector2, vec2: &Vector2) -> f32 {
    vec1.x * vec2.x + vec1.y * vec2.y
}

pub fn distance(vec1: &Vector2, vec2: &Vector2) -> f32 {
    f32::sqrt(distance_squared(&vec1, &vec2))
}

pub fn sin_cos(angle: f32) -> (f32, f32) {
    (angle.sin(), angle.cos())
}

pub fn rotate_pt(point: &mut Vector2, sin: f32, cos: f32, origin: &Vector2) {
    let x_offset = point.x - origin.x;
    let y_offset = point.y - origin.y;
    point.x = x_offset * cos - y_offset * sin + origin.x;
    point.y = x_offset * sin + y_offset * cos + origin.y;
}

// Return the distance from ray1 origin to the intersection point with the closest flipper feature.
// Sets ray2 origin to intersection point, direction to collision direction
pub fn distance_to_flipper(flipper: &mut TFlipperEdge, ray1: &RayType, ray2: &mut RayType) -> f32 {
    let mut distance: f32 = 1_000_000_000.0;
    let mut distance_type = FlipperIntersect::None;
    let mut new_distance = ray_intersect_line(ray1, &mut flipper.line_a);
    if new_distance < distance {
        distance = new_distance;
        distance_type = FlipperIntersect::LineA;
    }
    new_distance = ray_intersect_circle(ray1, &flipper.circle_base);
    if new_distance < distance {
        distance = new_distance;
        distance_type = FlipperIntersect::CircleBase;
    }
    new_distance = ray_intersect_circle(ray1, &flipper.circle_t1);
    if new_distance < distance {
        distance = new_distance;
        distance_type = FlipperIntersect::CircleT1;
    }
    new_distance = ray_intersect_line(ray1, &mut flipper.line_b);
    if new_distance < distance {
        distance = new_distance;
        distance_type = FlipperIntersect::LineB;
    }

    match distance_type {
        FlipperIntersect::LineA => {
            ray2.direction = flipper.line_a.perpendicular;
            ray2.origin = flipper.line_b.ray_intersect;
        }
        FlipperIntersect::LineB => {
            ray2.direction = flipper.line_b.perpendicular;
            ray2.origin = flipper.line_b.ray_intersect;
        }
        FlipperIntersect::CircleBase | FlipperIntersect::CircleT1 => {
            ray2.origin.x = distance * ray1.direction.x + ray1.origin.x;
            ray2.origin.y = distance * ray1.direction.y + ray1.origin.y;
            if distance_type == FlipperIntersect::CircleBase {
                ray2.direction = vector_sub_vec2(&ray2.origin, &flipper.circle_base.center);
            } else {
                ray2.direction = vector_sub_vec2(&ray2.origin, &flipper.circle_t1.center);
            }
            normalize_2d(&mut ray2.direction);
        }
        FlipperIntersect::None | _ => {}
    }

    distance
}

pub fn rotate_vector(vec: &mut Vector2, angle: f32) {
    let (s, c) = (angle.sin(), angle.cos());
    vec.x = c * vec.x - s * vec.y;
    vec.y = s * vec.x + c * vec.y;
    /* Error in the original, should be:
     * auto newX = c * vec.X - s * vec.Y;
     * vec.Y = s * vec.X + c * vec.Y;
     * vec.X = newX;
     */
    // Original code rotates the point on a figure eight curve.
    // Luckily, it is never used with angle always set to 0.
}

pub fn find_closest_edge(
    planes: &[RampPlaneType],
    plane_count: i32,
    wall: WallPointType,
    line_end: &mut Vector2,
    line_start: &mut Vector2,
) {
    let mut distance: f32 = 1_000_000_000.0;
    for plane in planes.iter().take(plane_count as usize) {
        let point_order: [&Vector2; 4] = [&plane.v1, &plane.v2, &plane.v3, &plane.v1];

        for pt_index in 0..3 {
            let point1 = point_order[pt_index];
            let point2 = point_order[pt_index + 1];

            let new_distance =
                self::distance(&wall.pt_0, point1) + self::distance(&wall.pt_1, point2);
            if new_distance < distance {
                distance = new_distance;
                *line_end = *point1;
                *line_start = *point2;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlipperIntersect {
    None = -1,
    LineA = 0,
    LineB = 1,
    CircleBase = 2,
    CircleT1 = 3,
}
