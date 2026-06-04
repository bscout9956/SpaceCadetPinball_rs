use crate::maths::Vector3;
use dear_imgui_rs::sys::Vector4;
use std::sync::{LazyLock, LockResult, Mutex};

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
struct Mat4RowMajor {
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

    (*matrix).row0 = VectorType4::new(max4x3[0], max4x3[1], max4x3[2], max4x3[3]);
    (*matrix).row1 = VectorType4::new(max4x3[4], max4x3[5], max4x3[6], max4x3[7]);
    (*matrix).row2 = VectorType4::new(max4x3[8], max4x3[9], max4x3[10], max4x3[11]);
    (*matrix).row3 = VectorType4::new(0.0, 0.0, 0.0, 1.0);

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
    *z_max = (0xffFFffFF as f32) / *z_scaler + *z_min;
}

pub fn matrix_vector_multiply(mat: &Mat4RowMajor, vec: &Vector3) -> Vector3 {
    todo!()
}
