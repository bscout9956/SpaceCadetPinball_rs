use crate::maths;
use crate::maths::{Vector2, Vector2i, Vector3};
use std::sync::{LazyLock, Mutex};

#[derive(Default, Clone, Copy, PartialEq)]
struct VectorType4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl VectorType4 {
    fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Mat4RowMajor {
    row0: VectorType4,
    row1: VectorType4,
    row2: VectorType4,
    row3: VectorType4,
}

pub static MATRIX: LazyLock<Mutex<Mat4RowMajor>> =
    LazyLock::new(|| Mutex::new(Mat4RowMajor::default()));

pub static D: Mutex<f32> = Mutex::new(0.0);
pub static CENTER_X: Mutex<f32> = Mutex::new(0.0);

pub static CENTER_Y: Mutex<f32> = Mutex::new(0.0);

pub static Z_SCALER: Mutex<f32> = Mutex::new(0.0);
pub static Z_MIN: Mutex<f32> = Mutex::new(0.0);
pub static Z_MAX: Mutex<f32> = Mutex::new(0.0);

pub fn init(max4x3: [f32; 12], d: f32, cen_x: f32, cen_y: f32, zm: f32, zscaler: f32) {
    let mut matrix = *MATRIX.lock().unwrap();

    matrix.row0 = VectorType4::new(max4x3[0], max4x3[1], max4x3[2], max4x3[3]);
    matrix.row1 = VectorType4::new(max4x3[4], max4x3[5], max4x3[6], max4x3[7]);
    matrix.row2 = VectorType4::new(max4x3[8], max4x3[9], max4x3[10], max4x3[11]);
    matrix.row3 = VectorType4::new(0.0, 0.0, 0.0, 1.0);

    *MATRIX.lock().unwrap() = matrix;

    let mut d_val = D.lock().unwrap();
    let mut center_x = CENTER_X.lock().unwrap();
    let mut center_y = CENTER_Y.lock().unwrap();
    let mut z_min = Z_MIN.lock().unwrap();
    let mut z_scaler = Z_SCALER.lock().unwrap();
    let mut z_max = Z_MAX.lock().unwrap();

    *d_val = d;
    *center_x = cen_x;
    *center_y = cen_y;
    *z_min = zm;
    *z_scaler = zscaler;
    *z_max = (0xFFFF_FFFFu32 as f32) / *z_scaler + *z_min;
}

pub fn matrix_vector_multiply(mat: &Mat4RowMajor, vec: &Vector3) -> Vector3 {
    let mut dst_vec: Vector3 = Vector3::default();
    let x: f32 = vec.x;
    let y: f32 = vec.y;
    let z: f32 = vec.z;

    dst_vec.x = z * mat.row0.z + y * mat.row0.y + x * mat.row0.x + mat.row0.w;
    dst_vec.y = z * mat.row1.z + y * mat.row1.y + x * mat.row1.x + mat.row1.w;
    dst_vec.z = z * mat.row2.z + y * mat.row2.y + x * mat.row2.x + mat.row2.w;

    dst_vec
}

pub fn z_distance(vec: &Vector3) -> f32 {
    let matrix = *MATRIX.lock().unwrap();
    let proj_vec = matrix_vector_multiply(&matrix, vec);
    maths::magnitude(&proj_vec)
}

pub fn recenter(center_x: &f32, center_y: &f32) {
    let mut cx = CENTER_X.lock().unwrap();
    let mut cy = CENTER_Y.lock().unwrap();

    *cx = *center_x;
    *cy = *center_y;
}

pub(crate) fn x_form_to_2d(vec: &Vector2) -> Vector2i {
    let vec3 = Vector3 {
        x: vec.x,
        y: vec.y,
        z: 0.0f32,
    };
    x_form_to_2d_vec3(vec3)
}

fn x_form_to_2d_vec3(vec: Vector3) -> Vector2i {
    let matrix = MATRIX.lock().unwrap();
    let proj_vec = matrix_vector_multiply(&matrix, &vec);
    let proj_coef = if proj_vec.z == 0.0f32 {
        999_999.9_f32 // magic number?
    } else {
        let d_ = D.lock().unwrap();
        *d_ / proj_vec.z
    };

    let center_x = CENTER_X.lock().unwrap();
    let center_y = CENTER_Y.lock().unwrap();
    Vector2i {
        x: (proj_vec.x * proj_coef + *center_x) as i32,
        y: (proj_vec.y * proj_coef + *center_y) as i32,
    }
}
