use crate::options::InputTypes::Keyboard;
use dear_imgui_rs::sys::{ImGuiMouseCursor_COUNT, igGetFrameCount, igGetMainViewport};
use dear_imgui_rs::{BackendFlags, Io, Key, MouseButton};
use dear_imgui_rs::{Context, TextureId};
use num_traits::ToPrimitive;
use sdl2::keyboard::Keycode;
use sdl2::render::RenderTarget;
use sdl2::sys::SDL_EventType::{
    SDL_KEYDOWN, SDL_KEYUP, SDL_MOUSEBUTTONDOWN, SDL_MOUSEBUTTONUP, SDL_MOUSEMOTION,
    SDL_MOUSEWHEEL, SDL_TEXTINPUT, SDL_WINDOWEVENT,
};
use sdl2::sys::SDL_Keymod::{KMOD_ALT, KMOD_CTRL, KMOD_GUI, KMOD_SHIFT};
use sdl2::sys::SDL_SystemCursor::{
    SDL_SYSTEM_CURSOR_ARROW, SDL_SYSTEM_CURSOR_HAND, SDL_SYSTEM_CURSOR_IBEAM, SDL_SYSTEM_CURSOR_NO,
    SDL_SYSTEM_CURSOR_SIZEALL, SDL_SYSTEM_CURSOR_SIZENESW, SDL_SYSTEM_CURSOR_SIZENS,
    SDL_SYSTEM_CURSOR_SIZENWSE, SDL_SYSTEM_CURSOR_SIZEWE, SDL_SYSTEM_CURSOR_WAIT,
    SDL_SYSTEM_CURSOR_WAITARROW,
};
use sdl2::sys::SDL_WindowEventID::{
    SDL_WINDOWEVENT_ENTER, SDL_WINDOWEVENT_FOCUS_GAINED, SDL_WINDOWEVENT_LEAVE,
};
use sdl2::sys::{
    KeyCode, SDL_BUTTON_LEFT, SDL_BUTTON_MIDDLE, SDL_BUTTON_RIGHT, SDL_BUTTON_X1, SDL_BUTTON_X2,
    SDL_CreateRGBSurfaceFrom, SDL_CreateSystemCursor, SDL_CreateTextureFromSurface, SDL_Cursor,
    SDL_DestroyTexture, SDL_Event, SDL_FreeSurface, SDL_GetCurrentVideoDriver, SDL_GetVersion,
    SDL_GetWindowWMInfo, SDL_HINT_MOUSE_AUTO_CAPTURE, SDL_HINT_MOUSE_FOCUS_CLICKTHROUGH,
    SDL_Keycode, SDL_Keymod, SDL_Renderer, SDL_SYSWM_TYPE, SDL_SetHint, SDL_Surface, SDL_SysWMinfo,
    SDL_Texture, SDL_Window, SDL_bool, SDL_version,
};
use std::ffi::{CStr, CString, c_char, c_void};
use std::ops::{Add, Mul};
use std::ptr::{addr_of_mut, null_mut};
use std::sync::{LazyLock, Mutex};

pub static CURRENT_DEVICE: LazyLock<Mutex<Option<Device>>> = LazyLock::new(|| Mutex::new(None));

struct ClipRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

struct TriangleCacheItem {
    texture: *mut SDL_Texture,
    width: i32,
    height: i32,
}

impl Drop for TriangleCacheItem {
    fn drop(&mut self) {
        if !self.texture.is_null() {
            unsafe {
                SDL_DestroyTexture(self.texture);
            }
        }
    }
}

type Clip = ClipRect;

static UNIFORM_COLOR_TRIANGLE_CACHE_SIZE: usize = 512;
static GENERIC_TRIANGLE_CACHE_SIZE: usize = 64;

pub struct Device {
    pub renderer: *mut SDL_Renderer,
    pub cache_was_invalidated: bool,
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

type UniformColorTriangleKey = (u32, i32, i32, i32, i32, i32, i32);
type GenericTriangleVertexKey = (i32, i32, f64, f64, u32);
type GenericTriangleKey = (
    GenericTriangleVertexKey,
    GenericTriangleVertexKey,
    GenericTriangleVertexKey,
);
impl Device {
    pub fn new(renderer: *mut SDL_Renderer) -> Self {
        Self {
            renderer,
            cache_was_invalidated: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_u32(color: u32) -> Self {
        Self {
            r: ((color >> 0) & 0xff) as f32 / 255.0,
            g: ((color >> 8) & 0xff) as f32 / 255.0,
            b: ((color >> 16) & 0xff) as f32 / 255.0,
            a: ((color >> 24) & 0xff) as f32 / 255.0,
        }
    }

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_int(&self) -> u32 {
        ((self.r * 255.0) as u32 & 0xff)
            | (((self.g * 255.0) as u32 & 0xff) << 8)
            | (((self.b * 255.0) as u32 & 0xff) << 16)
            | (((self.a * 255.0) as u32 & 0xff) << 24)
    }

    pub fn use_as_draw_color<T: RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>) {
        canvas.set_draw_color(sdl2::pixels::Color::RGBA(
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ));
    }
}

impl Mul<Color> for Color {
    type Output = Self;

    fn mul(self, rhs: Color) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
            a: self.a * rhs.a,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

impl Add<Color> for Color {
    type Output = Self;

    fn add(self, rhs: Color) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        }
    }
}

struct Texture {
    surface: *mut SDL_Surface,
    source: *mut SDL_Texture,
}

impl Texture {
    pub unsafe fn sample(&self, u: f32, v: f32) -> Color {
        unsafe {
            let x: i32 = f32::round(u * ((*self.surface).w - 1) as f32 + 0.5) as i32;
            let y: i32 = f32::round(v * ((*self.surface).h - 1) as f32 + 0.5) as i32;

            let location = y * (*self.surface).w + x;
            assert!(location < (*self.surface).w * (*self.surface).h);
            let color_u32 = unsafe {
                let ptr = (*self.surface).pixels as *const u32;
                ptr.offset(location as isize).read()
            };

            Color::from_u32(color_u32)
        }
    }

    pub fn new(surface: *mut SDL_Surface, source: *mut SDL_Texture) -> Self {
        Self { surface, source }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            SDL_FreeSurface(self.surface);
            SDL_DestroyTexture(self.source);
        }
    }
}

pub fn initialize(
    context: &mut Context,
    renderer: *mut SDL_Renderer,
    window_width: i32,
    window_height: i32,
) {
    let mut io = context.io_mut();
    io.set_display_size([window_width as f32, window_height as f32]);
    let context_style = context.style_mut();
    context_style.set_window_rounding(0.0);
    context_style.set_anti_aliased_fill(false);
    context_style.set_anti_aliased_lines(false);
    context_style.set_child_rounding(0.0);
    context_style.set_popup_rounding(0.0);
    context_style.set_frame_rounding(0.0);
    context_style.set_scrollbar_rounding(0.0);
    context_style.set_grab_rounding(0.0);
    context_style.set_tab_rounding(0.0);

    let tex_data = context.fonts().get_tex_data();
    unsafe {
        let (pixels, width, height) = ((*tex_data).Pixels, (*tex_data).Width, (*tex_data).Height);
        let rmask = 0x0000_00ff;
        let gmask = 0x0000_ff00;
        let bmask = 0x00ff_0000;
        let amask = 0xff00_0000;

        let surface = SDL_CreateRGBSurfaceFrom(
            pixels.cast::<c_void>(),
            width,
            height,
            32,
            4 * width,
            rmask,
            gmask,
            bmask,
            amask,
        );

        let source = SDL_CreateTextureFromSurface(renderer, surface);
        let texture = Box::new(Texture::new(surface, source));
        let raw_addr = Box::into_raw(texture) as usize;
        let texture_id = TextureId::new(raw_addr as u64);
        context.fonts().set_texture_id(texture_id);
        if let Ok(mut guard) = CURRENT_DEVICE.lock() {
            *guard = Some(Device::new(renderer));
        }
    }
}

pub fn init_for_sdl_renderer(
    context: &mut Context,
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
) -> bool {
    unsafe { impl_sdl2_init(context, window, renderer) }
}

pub struct ImplSdl2Data {
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    time: u64,
    mouse_buttons_down: i32,
    cursor: [SDL_Cursor; ImGuiMouseCursor_COUNT as usize],
    pending_mouse_leave_frame: i32,
    clipboard_text_data: *mut c_char,
    mouse_can_use_global_state: bool,
}

unsafe fn impl_sdl2_init(
    context: &mut Context,
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
) -> bool {
    let mut io = context.io_mut();
    assert!(
        io.backend_platform_user_data().is_null(),
        "SDL_UserData is NULL"
    ); // VERIFY: Is this what we want?
    let mut mouse_can_use_global_state = false;

    let sdl_backend = unsafe { SDL_GetCurrentVideoDriver() };
    let sdl_backend_str = unsafe { CStr::from_ptr(sdl_backend).to_str().unwrap() };

    let global_mouse_whitelist = ["windows", "cocoa", "x11", "DIVE", "VMAN"];
    for mouse in global_mouse_whitelist {
        if sdl_backend_str.eq(mouse) {
            mouse_can_use_global_state = true;
        }
    }

    let cursors: [SDL_Cursor; ImGuiMouseCursor_COUNT as usize] = unsafe {
        [
            *SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_ARROW),
            *SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_IBEAM),
            *SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_SIZEALL),
            *SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_SIZENS),
            *SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_SIZEWE),
            *SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_SIZENESW),
            *SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_SIZENWSE),
            *SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_HAND),
            *SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_WAIT),
            *SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_WAITARROW),
            *SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_NO),
        ]
    };

    //new(ImNewWrapper(), ImGui::MemAlloc(sizeof(_TYPE))) _TYPE
    let bd = Box::new(ImplSdl2Data {
        window,
        renderer,
        time: 0,
        mouse_buttons_down: 0,
        cursor: cursors,
        pending_mouse_leave_frame: 0,
        clipboard_text_data: null_mut(),
        mouse_can_use_global_state,
    });

    let bd_ptr = Box::into_raw(bd);

    io.set_backend_platform_user_data(bd_ptr.cast::<c_void>());
    let current_flags = io.backend_flags();
    io.set_backend_flags(
        BackendFlags::HAS_MOUSE_CURSORS | current_flags | BackendFlags::HAS_SET_MOUSE_POS,
    );
    // TODO: Better check this properly
    //    context.set_clipboard_backend();

    #[cfg(target_os = "windows")]
    {
        let mut info: SDL_SysWMinfo = unsafe { std::mem::zeroed() };
        unsafe { SDL_GetVersion(&raw mut info.version) };
        let info_ptr = addr_of_mut!(info);

        if unsafe { SDL_GetWindowWMInfo(window, info_ptr) } == SDL_bool::SDL_TRUE {
            let viewport = unsafe { igGetMainViewport() };

            let win_info = unsafe { &*(info_ptr as *const SdlSysWminfoWindows) };
            let hwnd = win_info.window;

            unsafe {
                (*viewport).PlatformHandleRaw = hwnd;
            }
        }
    }
    unsafe {
        SDL_SetHint(
            SDL_HINT_MOUSE_FOCUS_CLICKTHROUGH.as_ptr().cast::<c_char>(),
            c"1".as_ptr(),
        );
        SDL_SetHint(
            SDL_HINT_MOUSE_AUTO_CAPTURE.as_ptr().cast::<c_char>(),
            c"0".as_ptr(),
        );
    }

    true
}

// This is just so we can handle MSWin Properly.
// I am not sure why I have to do that honestly, but it seems there's only bindings for Linux
// Which is strange cause SDL2 should support Windows??
#[repr(C)]
struct SdlSysWminfoWindows {
    pub version: SDL_version,
    pub subsystem: SDL_SYSWM_TYPE,
    pub window: *mut c_void,
    pub hdc: *mut c_void,
}

// You can read the io.WantCaptureMouse, io.WantCaptureKeyboard flags to tell if dear imgui wants to use your inputs.
// - When io.WantCaptureMouse is true, do not dispatch mouse input data to your main application, or clear/overwrite your copy of the mouse data.
// - When io.WantCaptureKeyboard is true, do not dispatch keyboard input data to your main application, or clear/overwrite your copy of the keyboard data.
// Generally you may always pass all inputs to dear imgui, and hide them from your application based on those two flags.
// If you have multiple SDL events and some of them are not meant to be used by dear imgui, you may need to filter events based on their windowID field.
pub(crate) fn impl_sdl2_process_event(context: &mut Context, event: *mut SDL_Event) -> bool {
    let io = context.io_mut();
    let bd_ptr = io.backend_platform_user_data();

    let mut bd: *mut ImplSdl2Data = std::ptr::null_mut();

    if bd_ptr.is_null() {
        return false;
    }

    bd = unsafe { &raw mut *bd_ptr.cast::<ImplSdl2Data>() };

    unsafe {
        if (*event).type_ == SDL_MOUSEMOTION as u32 {
            io.add_mouse_pos_event([(*event).motion.x as f32, (*event).motion.y as f32]);
            return true;
        }
        if (*event).type_ == SDL_MOUSEWHEEL as u32 {
            let wheel_x = if (*event).wheel.x > 0 {
                1.0f32
            } else if (*event).wheel.x < 0 {
                -1.0f32
            } else {
                0.0f32
            };

            let wheel_y = if (*event).wheel.y > 0 {
                1.0f32
            } else if (*event).wheel.y < 0 {
                -1.0f32
            } else {
                0.0f32
            };

            io.add_mouse_wheel_event([wheel_x, wheel_y]);
            return true;
        }
        if (*event).type_ == SDL_MOUSEBUTTONDOWN as u32
            || (*event).type_ == SDL_MOUSEBUTTONUP as u32
        {
            let (mouse_button, button_index) = match u32::from((*event).button.button) {
                SDL_BUTTON_LEFT => (MouseButton::Left, 0),
                SDL_BUTTON_RIGHT => (MouseButton::Right, 1),
                SDL_BUTTON_MIDDLE => (MouseButton::Middle, 2),
                SDL_BUTTON_X1 => (MouseButton::Extra1, 3),
                SDL_BUTTON_X2 => (MouseButton::Extra2, 4),
                _ => return false,
            };

            let is_down = (*event).type_ == SDL_MOUSEBUTTONDOWN as u32;

            io.add_mouse_button_event(mouse_button, (*event).type_ == SDL_MOUSEBUTTONDOWN as u32);

            if is_down {
                (*bd).mouse_buttons_down |= 1 << button_index;
            } else {
                (*bd).mouse_buttons_down &= !(1 << button_index);
            }

            return true;
        }

        if (*event).type_ == SDL_TEXTINPUT as u32 {
            let c_str = unsafe { CStr::from_ptr((*event).text.text.as_ptr()) };

            if let Ok(text) = c_str.to_str() {
                for c in text.chars() {
                    io.add_input_character(c);
                }
            }
            return true;
        }

        if (*event).type_ == SDL_KEYDOWN as u32 || (*event).type_ == SDL_KEYUP as u32 {
            impl_sdl2_update_key_modifiers(io, u32::from((*event).key.keysym.mod_));
            let key =
                impl_sdl2_keycode_to_imgui_key(Keycode::from_i32((*event).key.keysym.sym).unwrap());
            io.add_key_event(key, (*event).type_ == SDL_KEYDOWN as u32);
            return true;
        }

        if (*event).type_ == SDL_WINDOWEVENT as u32 {
            // - When capturing mouse, SDL will send a bunch of conflicting LEAVE/ENTER event on every mouse move, but the final ENTER tends to be right.
            // - However we won't get a correct LEAVE event for a captured window.
            // - In some cases, when detaching a window from main viewport SDL may send SDL_WINDOWEVENT_ENTER one frame too late,
            //   causing SDL_WINDOWEVENT_LEAVE on previous frame to interrupt drag operation by clear mouse position. This is why
            //   we delay process the SDL_WINDOWEVENT_LEAVE events by one frame. See issue #5012 for details.
            let window_event = (*event).window.event;
            if window_event == SDL_WINDOWEVENT_ENTER as u8 {
                (*bd).pending_mouse_leave_frame = 0;
            }
            if window_event == SDL_WINDOWEVENT_LEAVE as u8 {
                (*bd).pending_mouse_leave_frame += igGetFrameCount() + 1;
            }
            if window_event == SDL_WINDOWEVENT_FOCUS_GAINED as u8 {
                io.add_focus_event(true);
            } else if (*event).window.event == SDL_WINDOWEVENT_FOCUS_GAINED as u8 {
                io.add_focus_event(false);
            }
            return true;
        }
    }
    false
}

fn impl_sdl2_keycode_to_imgui_key(key: Keycode) -> Key {
    match key {
        Keycode::TAB => Key::Tab,
        Keycode::LEFT => Key::LeftArrow,
        Keycode::RIGHT => Key::RightArrow,
        Keycode::UP => Key::UpArrow,
        Keycode::DOWN => Key::DownArrow,
        Keycode::PAGEUP => Key::PageUp,
        Keycode::PAGEDOWN => Key::PageDown,
        Keycode::HOME => Key::Home,
        Keycode::END => Key::End,
        Keycode::INSERT => Key::Insert,
        Keycode::DELETE => Key::Delete,
        Keycode::BACKSPACE => Key::Backspace,
        Keycode::SPACE => Key::Space,
        Keycode::RETURN => Key::Enter,
        Keycode::ESCAPE => Key::Escape,
        Keycode::QUOTE => Key::Apostrophe,
        Keycode::COMMA => Key::Comma,
        Keycode::MINUS => Key::Minus,
        Keycode::PERIOD => Key::Period,
        Keycode::SLASH => Key::Slash,
        Keycode::SEMICOLON => Key::Semicolon,
        Keycode::EQUALS => Key::Equal,
        Keycode::LEFTBRACKET => Key::LeftBracket,
        Keycode::BACKSLASH => Key::Backslash,
        Keycode::RIGHTBRACKET => Key::RightBracket,
        Keycode::BACKQUOTE => Key::GraveAccent,
        Keycode::CAPSLOCK => Key::CapsLock,
        Keycode::SCROLLLOCK => Key::ScrollLock,
        Keycode::NUMLOCKCLEAR => Key::NumLock,
        Keycode::PRINTSCREEN => Key::PrintScreen,
        Keycode::PAUSE => Key::Pause,
        Keycode::KP_0 => Key::Keypad0,
        Keycode::KP_1 => Key::Keypad1,
        Keycode::KP_2 => Key::Keypad2,
        Keycode::KP_3 => Key::Keypad3,
        Keycode::KP_4 => Key::Keypad4,
        Keycode::KP_5 => Key::Keypad5,
        Keycode::KP_6 => Key::Keypad6,
        Keycode::KP_7 => Key::Keypad7,
        Keycode::KP_8 => Key::Keypad8,
        Keycode::KP_9 => Key::Keypad9,
        Keycode::KP_PERIOD => Key::KeypadDecimal,
        Keycode::KP_DIVIDE => Key::KeypadDivide,
        Keycode::KP_MULTIPLY => Key::KeypadMultiply,
        Keycode::KP_MINUS => Key::KeypadSubtract,
        Keycode::KP_PLUS => Key::KeypadAdd,
        Keycode::KP_ENTER => Key::KeypadEnter,
        Keycode::KP_EQUALS => Key::KeypadEqual,
        Keycode::LCTRL => Key::LeftCtrl,
        Keycode::LSHIFT => Key::LeftShift,
        Keycode::LALT => Key::LeftAlt,
        Keycode::LGUI => Key::LeftSuper,
        Keycode::RCTRL => Key::RightCtrl,
        Keycode::RSHIFT => Key::RightShift,
        Keycode::RALT => Key::RightAlt,
        Keycode::RGUI => Key::RightSuper,
        Keycode::APPLICATION => Key::Menu,
        Keycode::NUM_0 => Key::Key0,
        Keycode::NUM_1 => Key::Key1,
        Keycode::NUM_2 => Key::Key2,
        Keycode::NUM_3 => Key::Key3,
        Keycode::NUM_4 => Key::Key4,
        Keycode::NUM_5 => Key::Key5,
        Keycode::NUM_6 => Key::Key6,
        Keycode::NUM_7 => Key::Key7,
        Keycode::NUM_8 => Key::Key8,
        Keycode::NUM_9 => Key::Key9,
        Keycode::A => Key::A,
        Keycode::B => Key::B,
        Keycode::C => Key::C,
        Keycode::D => Key::D,
        Keycode::E => Key::E,
        Keycode::F => Key::F,
        Keycode::G => Key::G,
        Keycode::H => Key::H,
        Keycode::I => Key::I,
        Keycode::J => Key::J,
        Keycode::K => Key::K,
        Keycode::L => Key::L,
        Keycode::M => Key::M,
        Keycode::N => Key::N,
        Keycode::O => Key::O,
        Keycode::P => Key::P,
        Keycode::Q => Key::Q,
        Keycode::R => Key::R,
        Keycode::S => Key::S,
        Keycode::T => Key::T,
        Keycode::U => Key::U,
        Keycode::V => Key::V,
        Keycode::W => Key::W,
        Keycode::X => Key::X,
        Keycode::Y => Key::Y,
        Keycode::Z => Key::Z,
        Keycode::F1 => Key::F1,
        Keycode::F2 => Key::F2,
        Keycode::F3 => Key::F3,
        Keycode::F4 => Key::F4,
        Keycode::F5 => Key::F5,
        Keycode::F6 => Key::F6,
        Keycode::F7 => Key::F7,
        Keycode::F8 => Key::F8,
        Keycode::F9 => Key::F9,
        Keycode::F10 => Key::F10,
        Keycode::F11 => Key::F11,
        Keycode::F12 => Key::F12,
        _ => Key::None,
    }
}

pub fn impl_sdl2_update_key_modifiers(io: &mut Io, sdl_key_mods: u32) {
    io.add_key_event(Key::ModCtrl, (sdl_key_mods & KMOD_CTRL as u32) != 0);
    io.add_key_event(Key::ModShift, (sdl_key_mods & KMOD_SHIFT as u32) != 0);
    io.add_key_event(Key::ModAlt, (sdl_key_mods & KMOD_ALT as u32) != 0);
    io.add_key_event(Key::ModSuper, (sdl_key_mods & KMOD_GUI as u32) != 0);
}

pub fn impl_sdl2_get_backend_data(io: &mut Io) -> Option<&mut ImplSdl2Data> {
    let ptr = io.backend_platform_user_data();

    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &mut *ptr.cast::<ImplSdl2Data>() })
    }
}

pub(crate) fn render_new_frame() {
    todo!()
}

pub(crate) fn impl_sdl2_new_frame() {
    todo!()
}
