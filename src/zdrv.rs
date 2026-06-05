use sdl2::sys::SDL_Texture;

#[derive(Clone)]
pub struct ZMapHeaderType {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub resolution: u32,
    pub z_map_data: Vec<u16>,
    pub texture: Option<SDL_Texture>,
}

impl ZMapHeaderType {
    pub fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            stride: 0,
            resolution: 0,
            z_map_data: vec![],
            texture: None,
        }
    }

    pub fn new(width: i32, height: i32, mut stride: i32) -> Self {
        if stride < 0 {
            stride = pad(width);
        }

        let size = (stride * height) as usize;

        Self {
            width,
            height,
            stride,
            resolution: 0,
            z_map_data: vec![0; size],
            texture: None,
        }
    }
}

pub fn fill(
    mut zmap: &mut ZMapHeaderType,
    width: i32,
    height: i32,
    x_off: i32,
    y_off: i32,
    fill_pattern: u16,
) {
    let mut dst_ptr = (zmap.stride * y_off + x_off) as usize;

    let mut y = height;
    while y > 0 {
        let mut x = width;
        while x > 0 {
            zmap.z_map_data[dst_ptr] = fill_pattern;
            dst_ptr += 1;
            x -= 1;
        }
        dst_ptr += (zmap.stride - width) as usize;
        y -= 1;
    }
}

pub fn pad(width: i32) -> i32 {
    let mut result = width;
    if (width & 3 != 0) {
        result = width - (width & 3) + 4;
    }
    result
}

pub fn flip_zmap_horizontally(zmap: &mut ZMapHeaderType) {
    if zmap.height <= 1 || zmap.width == 0 || zmap.z_map_data.is_empty() {
        return;
    }

    let mut dst_idx = 0;
    let mut src_idx = zmap.stride as usize * (zmap.height as usize - 1);

    let mut y = zmap.height as usize - 1;
    while y >= (zmap.height / 2) as usize {
        let mut x = 0;
        while x < (zmap.width as usize) {
            zmap.z_map_data.swap(dst_idx, src_idx);
            dst_idx += 1;
            src_idx += 1;
            x += 1
        }
        dst_idx += (zmap.stride - zmap.width) as usize;
        src_idx -= (zmap.stride + zmap.width) as usize;

        y -= 1;
    }
}
