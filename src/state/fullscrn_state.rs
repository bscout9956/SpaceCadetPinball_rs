use crate::fullscrn::ResolutionInfo;

pub struct FullscrnState {
    pub scale_x: f32,
    pub scale_y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub resolution_array: [ResolutionInfo; 3],
    pub screen_mode: bool,
    pub display_changed: bool,
    pub resolution: i32,
}

impl FullscrnState {
    pub fn new() -> FullscrnState {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            resolution_array: [
                ResolutionInfo {
                    screen_width: 640,
                    screen_height: 480,
                    table_width: 600,
                    table_height: 416,
                    resolution_menu_id: 501,
                },
                ResolutionInfo {
                    screen_width: 800,
                    screen_height: 600,
                    table_width: 752,
                    table_height: 520,
                    resolution_menu_id: 502,
                },
                ResolutionInfo {
                    screen_width: 1024,
                    screen_height: 768,
                    table_width: 960,
                    table_height: 666,
                    resolution_menu_id: 503,
                },
            ],
            screen_mode: false,
            display_changed: false,
            resolution: 0,
        }
    }
}

impl Default for FullscrnState {
    fn default() -> Self {
        Self::new()
    }
}
