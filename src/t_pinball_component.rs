use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

// TODO: The enums below may be incorrect, I'm not sure how to deal with conflicting
// enums yet, so I just split them into a bunch of other enums lol
pub enum MessageFlipper {
    TFlipperNull = 0,
    TFlipperExtend = 1,
    TFlipperRetract = 2,
}

pub enum MessageLight {
    TLightTurnOff = 0,
    TLightTurnOn = 1,
    TLightGetLightOnFlag = 2,
    TLightGetFlasherOnFlag = 3,
    TLightFlasherStart = 4,
    TLightApplyMultDelay = 5,
    TLightApplyDelay = 6,
    TLightFlasherStartTimed = 7,
    TLightTurnOffTimed = 8,
    TLightTurnOnTimed = 9,
    TLightSetOnStateBmpIndex = 11,
    TLightIncOnStateBmpIndex = 12,
    TLightDecOnStateBmpIndex = 13,
    TLightResetTimed = 14,
    TLightFlasherStartTimedThenStayOn = 15,
    TLightFlasherStartTimedThenStayOff = 16,
    TLightToggleValue = 17,
    TLightResetAndToggleValue = 18,
    TLightResetAndTurnOn = 19,
    TLightResetAndTurnOff = 20,
    TLightToggle = 21,
    TLightResetAndToggle = 22,
    TLightSetMessageField = 23,
    TLightFtTmpOverrideOn = -24,
    TLightFtTmpOverrideOff = -25,
    TLightFtResetOverride = -26,
}

pub enum MessageLightGroup {
    TLightGroupNull = 0,
    TLightGroupStepBackward = 24,
    TLightGroupStepForward = 25,
    TLightGroupAnimationBackward = 26,
    TLightGroupAnimationForward = 27,
    TLightGroupLightShowAnimation = 28,
    TLightGroupGameOverAnimation = 29,
    TLightGroupRandomAnimationSaturation = 30,
    TLightGroupRandomAnimationDesaturation = 31,
    TLightGroupOffsetAnimationForward = 32,
    TLightGroupOffsetAnimationBackward = 33,
    TLightGroupReset = 34,
    TLightGroupTurnOnAtIndex = 35,
    TLightGroupTurnOffAtIndex = 36,
    TLightGroupGetOnCount = 37,
    TLightGroupGetLightCount = 38,
    TLightGroupGetMessage2 = 39,
    TLightGroupGetAnimationFlag = 40,
    TLightGroupResetAndTurnOn = 41,
    TLightGroupResetAndTurnOff = 42,
    TLightGroupRestartNotifyTimer = 43,
    TLightGroupFlashWhenOn = 44,
    TLightGroupToggleSplitIndex = 45,
    TLightGroupStartFlasher = 46,
    TLightGroupCountdownEnded = 47,
}

pub enum MessageBumper {
    TBumperSetBmpIndex = 11,
    TBumperIncBmpIndex = 12,
    TBumperDecBmpIndex = 13,
}

pub enum MessageCodePublic {
    LeftFlipperInputPressed = 1000,
    LeftFlipperInputReleased = 1001,
    RightFlipperInputPressed = 1002,
    RightFlipperInputReleased = 1003,
    PlungerInputPressed = 1004,
    PlungerInputReleased = 1005,
    Pause = 1008,
    Resume = 1009,
    LooseFocus = 1010,
    SetTiltLock = 1011,
    ClearTiltLock = 1012,
    StartGamePlayer1 = 1013,
    NewGame = 1014,
    PlungerFeedBall = 1015,
    PlungerStartFeedTimer = 1016,
    PlungerLaunchBall = 1017,
    PlungerRelaunchBall = 1018,
    PlayerChanged = 1020,
    SwitchToNextPlayer = 1021,
    GameOver = 1022,
    Reset = 1024,
}

pub enum MessageControl {
    ControlBallCaptured = 57,
    ControlBallReleased = 58,
    ControlTimerExpired = 60,
    ControlNotifyTimerExpired = 61,
    ControlSpinnerLoopReset = 62,
    ControlCollision = 63,
    ControlEnableMultiplier = 64,
    ControlDisableMultiplier = 65,
    ControlMissionComplete = 66,
    ControlMissionStarted = 67,
}

pub enum MessageBlocker {
    TBlockerDisable = 51,
    TBlockerEnable = 52,
    TBlockerRestartTimeout = 59,
}

pub enum MessageTimer {
    TComponentGroupResetNotifyTimer = 48,

    TPopupTargetDisable = 49,
    TPopupTargetEnable = 50,

    TGateDisable = 53,
    TGateEnable = 54,

    TKickoutRestartTimer = 55,

    TSinkUnknown7 = 7,
    TSinkResetTimer = 56,
    TTimerResetTimer = 59,
}

pub enum MessageTarget {
    TSoloTargetDisable = 49,
    TSoloTargetEnable = 50,
}

#[repr(i32)]
pub enum MessageCode {
    MessageFlipper(MessageFlipper),
    MessageLight(MessageLight),
    MessageLightGroup(MessageLightGroup),
    MessageBumper(MessageBumper),
    MessageBlocker(MessageBlocker),
    MessageTarget(MessageTarget),
    MessageTimer(MessageTimer),
    MessageControl(MessageControl),
    MessageCodePublic(MessageCodePublic),
}

// TODO: Temporary
struct Control;
// TODO: Temporary
struct RenderSprite;
// TODO: Temporary
struct TPinballTable;
// TODO: Temporary
struct SpriteData;
// TODO: Temporary
struct Vector2i;
struct Vector2;

struct TPinballComponent {
    pub unused_base_flag: Rc<Cell<bool>>,
    pub active_flag: Rc<Cell<bool>>,
    pub message_field: i32,
    pub group_name: String,         // TODO: eh?
    pub component_control: Control, //TODO: Decide what this will be
    pub group_index: i32,
    pub render_sprite: RenderSprite, //TODO: Decide what this will be
    pub pinball_table: Weak<RefCell<TPinballTable>>,
    pub list_bitmap: Vec<SpriteData>, // TODO: Decide the internal struct

    visual_pos_norm_x: f32,
    visual_pos_norm_y: f32,
}

trait TPinballComponentBehavior {
    fn sprite_set(index: i32);
    fn sprite_set_ball(index: i32, pos: Vector2i, depth: f32);
    fn get_coordinates() -> Vector2;
    fn get_scoring(index: u32) -> i32;
    fn port_draw();
    fn message(code: MessageCode, value: f32) -> i32;
}

impl TPinballComponent {
    // TODO: Finish me
}

impl TPinballComponentBehavior for TPinballComponent {
    fn sprite_set(index: i32) {
        todo!()
    }

    fn sprite_set_ball(index: i32, pos: Vector2i, depth: f32) {
        todo!()
    }

    fn get_coordinates() -> Vector2 {
        todo!()
    }

    fn get_scoring(index: u32) -> i32 {
        todo!()
    }

    fn port_draw() {
        todo!()
    }

    fn message(code: MessageCode, value: f32) -> i32 {
        todo!()
    }
}
