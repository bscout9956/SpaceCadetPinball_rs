use crate::{Duration, SdlRendererPtr, SdlWindowPtr, WelfordState};

#[allow(clippy::struct_excessive_bools)]
pub struct MainState {
    pub b_quit: bool,
    pub mouse_down: bool,
    pub has_focus: bool,
    pub return_value: i32,
    pub single_step: bool,
    pub target_frametime: Duration<1_000_000_000>,
    pub cursor_idle_counter: i64,
    pub fps_details: String,
    pub last_mouse_x: i32,
    pub last_mouse_y: i32,
    pub no_time_loss: bool,
    pub demo_active: bool,
    pub launch_ball_enabled: bool,
    pub high_scores_enabled: bool,
    pub gfr_display: Vec<f32>,
    pub prev_sdl_error: String,
    pub restart: bool,
    pub show_about_dialog: bool,
    pub about_tab_open: bool,
    pub show_imgui_demo: bool,
    pub show_sprite_viewer: bool,
    pub show_exit_popup: bool,
    pub renderer: Option<SdlRendererPtr>,
    pub update_to_frame_ratio: f64,
    pub disp_frame_rate: bool,
    pub disp_gr_history: bool,
    pub activated: bool,
    pub main_menu_height: i32,
    pub spin_threshold: Duration<1_000_000_000>,
    pub sleep_state: WelfordState,
    pub gfr_offset: u32,
    pub prev_sdl_error_count: u32,
    pub main_window: Option<SdlWindowPtr>,
    pub gfr_window: f32,
    pub full_tilt_tab_open: bool,
    pub idle_wait: i64
}

impl MainState {
    pub(crate) fn new() -> MainState {
        Self {
            b_quit: false,
            mouse_down: false,
            has_focus: true,
            return_value: 0,
            single_step: false,
            target_frametime: Duration(0),
            cursor_idle_counter: 0,
            fps_details: String::new(),
            last_mouse_x: 0,
            last_mouse_y: 0,
            no_time_loss: false,
            demo_active: false,
            launch_ball_enabled: true,
            high_scores_enabled: true,
            gfr_display: Vec::new(),
            prev_sdl_error: String::new(),
            restart: false,
            show_about_dialog: false,
            about_tab_open: false,
            show_imgui_demo: false,
            show_sprite_viewer: false,
            show_exit_popup: false,
            renderer: None,
            update_to_frame_ratio: 0.0,
            disp_frame_rate: false,
            disp_gr_history: false,
            activated: false,
            main_menu_height: 0,
            spin_threshold: Duration(0),
            sleep_state: WelfordState::new(),
            gfr_offset: 0,
            prev_sdl_error_count: 0,
            main_window: None,
            gfr_window: 5.0,
            full_tilt_tab_open: true,
            idle_wait: 0
        }
    }

    pub fn update_fps_details(&mut self, value: &str) {
        self.fps_details = value.to_string();
    }

    pub fn update_mouse_xy(&mut self, x: i32, y: i32) {
        self.last_mouse_x = x;
        self.last_mouse_y = y;
    }
}
