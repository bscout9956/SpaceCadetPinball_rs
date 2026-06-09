use crate::t_edge_segment::TEdgeSegment;

pub struct TDemo {
    pub active_flag: bool,
    flip_timer_timer_1: f32,
    flip_timer_timer_2: f32,
    unflip_timer_time_1: f32,
    unflip_timer_time_2: f32,
    flip_left_flag: bool,
    flip_right_flag: bool,
    flip_left_timer: i32,
    flip_right_timer: i32,
    plunger_flag: bool,
    restart_game_timer: i32,
    edge_1: Option<TEdgeSegment>,
    edge_2: Option<TEdgeSegment>,
    edge_3: Option<TEdgeSegment>,
}
