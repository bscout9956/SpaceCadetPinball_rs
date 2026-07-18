use crate::t_blocker::TBlocker;
use crate::t_bumper::TBumper;
use crate::t_flipper::TFlipper;
use crate::t_gate::TGate;
use crate::t_light::TLight;
use crate::t_light_group::TLightGroup;
use crate::t_oneway::TOneWay;
use crate::t_plunger::TPlunger;
use crate::t_sound::TSound;
use crate::t_textbox::TTextBox;
use crate::t_wall::TWall;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

// This is the subset of the original control component registry whose concrete
// component types have been ported. Add the remaining original tags here as
// their Rust component types become available.
pub struct ComponentState {
    pub block_1: ComponentRef<TBlocker>,

    pub bump_1: ComponentRef<TBumper>,
    pub bump_2: ComponentRef<TBumper>,
    pub bump_3: ComponentRef<TBumper>,
    pub bump_4: ComponentRef<TBumper>,
    pub bump_5: ComponentRef<TBumper>,
    pub bump_6: ComponentRef<TBumper>,
    pub bump_7: ComponentRef<TBumper>,

    pub flip_1: ComponentRef<TFlipper>,
    pub flip_2: ComponentRef<TFlipper>,

    pub gate_1: ComponentRef<TGate>,
    pub gate_2: ComponentRef<TGate>,

    pub info_text_box: ComponentRef<TTextBox>,
    pub mission_text_box: ComponentRef<TTextBox>,

    pub plunger: ComponentRef<TPlunger>,

    pub rebo_1: ComponentRef<TWall>,
    pub rebo_2: ComponentRef<TWall>,
    pub rebo_3: ComponentRef<TWall>,
    pub rebo_4: ComponentRef<TWall>,

    pub lite_1: ComponentRef<TLight>,
    pub lite_2: ComponentRef<TLight>,
    pub lite_3: ComponentRef<TLight>,
    pub lite_4: ComponentRef<TLight>,
    pub lite_5: ComponentRef<TLight>,
    pub lite_6: ComponentRef<TLight>,
    pub lite_7: ComponentRef<TLight>,
    pub lite_8: ComponentRef<TLight>,
    pub lite_9: ComponentRef<TLight>,
    pub lite_10: ComponentRef<TLight>,
    pub lite_11: ComponentRef<TLight>,
    pub lite_12: ComponentRef<TLight>,
    pub lite_13: ComponentRef<TLight>,
    pub lite_16: ComponentRef<TLight>,
    pub lite_17: ComponentRef<TLight>,
    pub lite_18: ComponentRef<TLight>,
    pub lite_19: ComponentRef<TLight>,
    pub lite_20: ComponentRef<TLight>,
    pub lite_21: ComponentRef<TLight>,
    pub lite_22: ComponentRef<TLight>,
    pub lite_23: ComponentRef<TLight>,
    pub lite_24: ComponentRef<TLight>,
    pub lite_25: ComponentRef<TLight>,
    pub lite_26: ComponentRef<TLight>,
    pub lite_27: ComponentRef<TLight>,
    pub lite_28: ComponentRef<TLight>,
    pub lite_29: ComponentRef<TLight>,
    pub lite_30: ComponentRef<TLight>,
    pub lite_38: ComponentRef<TLight>,
    pub lite_39: ComponentRef<TLight>,
    pub lite_40: ComponentRef<TLight>,
    pub lite_54: ComponentRef<TLight>,
    pub lite_55: ComponentRef<TLight>,
    pub lite_56: ComponentRef<TLight>,
    pub lite_58: ComponentRef<TLight>,
    pub lite_59: ComponentRef<TLight>,
    pub lite_60: ComponentRef<TLight>,
    pub lite_61: ComponentRef<TLight>,
    pub lite_62: ComponentRef<TLight>,
    pub lite_67: ComponentRef<TLight>,
    pub lite_68: ComponentRef<TLight>,
    pub lite_69: ComponentRef<TLight>,
    pub lite_70: ComponentRef<TLight>,
    pub lite_71: ComponentRef<TLight>,
    pub lite_72: ComponentRef<TLight>,
    pub lite_77: ComponentRef<TLight>,
    pub lite_84: ComponentRef<TLight>,
    pub lite_85: ComponentRef<TLight>,
    pub lite_101: ComponentRef<TLight>,
    pub lite_102: ComponentRef<TLight>,
    pub lite_103: ComponentRef<TLight>,
    pub lite_104: ComponentRef<TLight>,
    pub lite_105: ComponentRef<TLight>,
    pub lite_106: ComponentRef<TLight>,
    pub lite_107: ComponentRef<TLight>,
    pub lite_108: ComponentRef<TLight>,
    pub lite_109: ComponentRef<TLight>,
    pub lite_110: ComponentRef<TLight>,
    pub lite_130: ComponentRef<TLight>,
    pub lite_131: ComponentRef<TLight>,
    pub lite_132: ComponentRef<TLight>,
    pub lite_133: ComponentRef<TLight>,
    pub lite_169: ComponentRef<TLight>,
    pub lite_170: ComponentRef<TLight>,
    pub lite_171: ComponentRef<TLight>,
    pub lite_195: ComponentRef<TLight>,
    pub lite_196: ComponentRef<TLight>,
    pub lite_198: ComponentRef<TLight>,
    pub lite_199: ComponentRef<TLight>,
    pub lite_200: ComponentRef<TLight>,
    pub lite_300: ComponentRef<TLight>,
    pub lite_301: ComponentRef<TLight>,
    pub lite_302: ComponentRef<TLight>,
    pub lite_303: ComponentRef<TLight>,
    pub lite_304: ComponentRef<TLight>,
    pub lite_305: ComponentRef<TLight>,
    pub lite_306: ComponentRef<TLight>,
    pub lite_307: ComponentRef<TLight>,
    pub lite_308: ComponentRef<TLight>,
    pub lite_309: ComponentRef<TLight>,
    pub lite_310: ComponentRef<TLight>,
    pub lite_311: ComponentRef<TLight>,
    pub lite_312: ComponentRef<TLight>,
    pub lite_313: ComponentRef<TLight>,
    pub lite_314: ComponentRef<TLight>,
    pub lite_315: ComponentRef<TLight>,
    pub lite_316: ComponentRef<TLight>,
    pub lite_317: ComponentRef<TLight>,
    pub lite_318: ComponentRef<TLight>,
    pub lite_319: ComponentRef<TLight>,
    pub lite_320: ComponentRef<TLight>,
    pub lite_321: ComponentRef<TLight>,
    pub lite_322: ComponentRef<TLight>,
    pub lite_roll_179: ComponentRef<TLight>,
    pub lite_roll_180: ComponentRef<TLight>,
    pub lite_roll_181: ComponentRef<TLight>,
    pub lite_roll_182: ComponentRef<TLight>,
    pub lite_roll_183: ComponentRef<TLight>,
    pub lite_roll_184: ComponentRef<TLight>,

    pub middle_circle: ComponentRef<TLightGroup>,
    pub left_chute_target_lights: ComponentRef<TLightGroup>,
    pub left_trek_lights: ComponentRef<TLightGroup>,
    pub goal_lights: ComponentRef<TLightGroup>,
    pub hyperspace_lights: ComponentRef<TLightGroup>,
    pub bumper_increment_lights: ComponentRef<TLightGroup>,
    pub bumper_solo_target_lights: ComponentRef<TLightGroup>,
    pub black_hole_sink_arrow_lights: ComponentRef<TLightGroup>,
    pub bumper_target_lights: ComponentRef<TLightGroup>,
    pub outer_circle: ComponentRef<TLightGroup>,
    pub right_trek_lights: ComponentRef<TLightGroup>,
    pub ramp_bumper_increment_lights: ComponentRef<TLightGroup>,
    pub ramp_target_lights: ComponentRef<TLightGroup>,
    pub skill_shot_lights: ComponentRef<TLightGroup>,
    pub top_circle_target_lights: ComponentRef<TLightGroup>,
    pub top_target_lights: ComponentRef<TLightGroup>,
    pub worm_hole_lights: ComponentRef<TLightGroup>,

    pub soundwave_3: ComponentRef<TSound>,
    pub soundwave_7: ComponentRef<TSound>,
    pub soundwave_8: ComponentRef<TSound>,
    pub soundwave_9: ComponentRef<TSound>,
    pub soundwave_10: ComponentRef<TSound>,
    pub soundwave_14_1: ComponentRef<TSound>,
    pub soundwave_14_2: ComponentRef<TSound>,
    pub soundwave_21: ComponentRef<TSound>,
    pub soundwave_23: ComponentRef<TSound>,
    pub soundwave_24: ComponentRef<TSound>,
    pub soundwave_25: ComponentRef<TSound>,
    pub soundwave_26: ComponentRef<TSound>,
    pub soundwave_27: ComponentRef<TSound>,
    pub soundwave_28: ComponentRef<TSound>,
    pub soundwave_30: ComponentRef<TSound>,
    pub soundwave_35_1: ComponentRef<TSound>,
    pub soundwave_35_2: ComponentRef<TSound>,
    pub soundwave_36_1: ComponentRef<TSound>,
    pub soundwave_36_2: ComponentRef<TSound>,
    pub soundwave_38: ComponentRef<TSound>,
    pub soundwave_39: ComponentRef<TSound>,
    pub soundwave_40: ComponentRef<TSound>,
    pub soundwave_41: ComponentRef<TSound>,
    pub soundwave_44: ComponentRef<TSound>,
    pub soundwave_45: ComponentRef<TSound>,
    pub soundwave_46: ComponentRef<TSound>,
    pub soundwave_47: ComponentRef<TSound>,
    pub soundwave_48: ComponentRef<TSound>,
    pub soundwave_49_d: ComponentRef<TSound>,
    pub soundwave_50_1: ComponentRef<TSound>,
    pub soundwave_50_2: ComponentRef<TSound>,
    pub soundwave_52: ComponentRef<TSound>,
    pub soundwave_59: ComponentRef<TSound>,
}

impl Default for ComponentState {
    fn default() -> Self {
        Self {
            block_1: ComponentRef::new("v_bloc1"),
            bump_1: ComponentRef::new("a_bump1"),
            bump_2: ComponentRef::new("a_bump2"),
            bump_3: ComponentRef::new("a_bump3"),
            bump_4: ComponentRef::new("a_bump4"),
            bump_5: ComponentRef::new("a_bump5"),
            bump_6: ComponentRef::new("a_bump6"),
            bump_7: ComponentRef::new("a_bump7"),
            flip_1: ComponentRef::new("a_flip1"),
            flip_2: ComponentRef::new("a_flip2"),
            gate_1: ComponentRef::new("v_gate1"),
            gate_2: ComponentRef::new("v_gate2"),
            info_text_box: ComponentRef::new("info_text_box"),
            mission_text_box: ComponentRef::new("mission_text_box"),
            plunger: ComponentRef::new("plunger"),
            rebo_1: ComponentRef::new("v_rebo1"),
            rebo_2: ComponentRef::new("v_rebo2"),
            rebo_3: ComponentRef::new("v_rebo3"),
            rebo_4: ComponentRef::new("v_rebo4"),
            lite_1: ComponentRef::new("lite1"),
            lite_2: ComponentRef::new("lite2"),
            lite_3: ComponentRef::new("lite3"),
            lite_4: ComponentRef::new("lite4"),
            lite_5: ComponentRef::new("lite5"),
            lite_6: ComponentRef::new("lite6"),
            lite_7: ComponentRef::new("lite7"),
            lite_8: ComponentRef::new("lite8"),
            lite_9: ComponentRef::new("lite9"),
            lite_10: ComponentRef::new("lite10"),
            lite_11: ComponentRef::new("lite11"),
            lite_12: ComponentRef::new("lite12"),
            lite_13: ComponentRef::new("lite13"),
            lite_16: ComponentRef::new("lite16"),
            lite_17: ComponentRef::new("lite17"),
            lite_18: ComponentRef::new("lite18"),
            lite_19: ComponentRef::new("lite19"),
            lite_20: ComponentRef::new("lite20"),
            lite_21: ComponentRef::new("lite21"),
            lite_22: ComponentRef::new("lite22"),
            lite_23: ComponentRef::new("lite23"),
            lite_24: ComponentRef::new("lite24"),
            lite_25: ComponentRef::new("lite25"),
            lite_26: ComponentRef::new("lite26"),
            lite_27: ComponentRef::new("lite27"),
            lite_28: ComponentRef::new("lite28"),
            lite_29: ComponentRef::new("lite29"),
            lite_30: ComponentRef::new("lite30"),
            lite_38: ComponentRef::new("lite38"),
            lite_39: ComponentRef::new("lite39"),
            lite_40: ComponentRef::new("lite40"),
            lite_54: ComponentRef::new("lite54"),
            lite_55: ComponentRef::new("lite55"),
            lite_56: ComponentRef::new("lite56"),
            lite_58: ComponentRef::new("lite58"),
            lite_59: ComponentRef::new("lite59"),
            lite_60: ComponentRef::new("lite60"),
            lite_61: ComponentRef::new("lite61"),
            lite_62: ComponentRef::new("lite62"),
            lite_67: ComponentRef::new("lite67"),
            lite_68: ComponentRef::new("lite68"),
            lite_69: ComponentRef::new("lite69"),
            lite_70: ComponentRef::new("lite70"),
            lite_71: ComponentRef::new("lite71"),
            lite_72: ComponentRef::new("lite72"),
            lite_77: ComponentRef::new("lite77"),
            lite_84: ComponentRef::new("lite84"),
            lite_85: ComponentRef::new("lite85"),
            lite_101: ComponentRef::new("lite101"),
            lite_102: ComponentRef::new("lite102"),
            lite_103: ComponentRef::new("lite103"),
            lite_104: ComponentRef::new("lite104"),
            lite_105: ComponentRef::new("lite105"),
            lite_106: ComponentRef::new("lite106"),
            lite_107: ComponentRef::new("lite107"),
            lite_108: ComponentRef::new("lite108"),
            lite_109: ComponentRef::new("lite109"),
            lite_110: ComponentRef::new("lite110"),
            lite_130: ComponentRef::new("lite130"),
            lite_131: ComponentRef::new("lite131"),
            lite_132: ComponentRef::new("lite132"),
            lite_133: ComponentRef::new("lite133"),
            lite_169: ComponentRef::new("lite169"),
            lite_170: ComponentRef::new("lite170"),
            lite_171: ComponentRef::new("lite171"),
            lite_195: ComponentRef::new("lite195"),
            lite_196: ComponentRef::new("lite196"),
            lite_198: ComponentRef::new("lite198"),
            lite_199: ComponentRef::new("lite199"),
            lite_200: ComponentRef::new("lite200"),
            lite_300: ComponentRef::new("lite300"),
            lite_301: ComponentRef::new("lite301"),
            lite_302: ComponentRef::new("lite302"),
            lite_303: ComponentRef::new("lite303"),
            lite_304: ComponentRef::new("lite304"),
            lite_305: ComponentRef::new("lite305"),
            lite_306: ComponentRef::new("lite306"),
            lite_307: ComponentRef::new("lite307"),
            lite_308: ComponentRef::new("lite308"),
            lite_309: ComponentRef::new("lite309"),
            lite_310: ComponentRef::new("lite310"),
            lite_311: ComponentRef::new("lite311"),
            lite_312: ComponentRef::new("lite312"),
            lite_313: ComponentRef::new("lite313"),
            lite_314: ComponentRef::new("lite314"),
            lite_315: ComponentRef::new("lite315"),
            lite_316: ComponentRef::new("lite316"),
            lite_317: ComponentRef::new("lite317"),
            lite_318: ComponentRef::new("lite318"),
            lite_319: ComponentRef::new("lite319"),
            lite_320: ComponentRef::new("lite320"),
            lite_321: ComponentRef::new("lite321"),
            lite_322: ComponentRef::new("lite322"),
            lite_roll_179: ComponentRef::new("literoll179"),
            lite_roll_180: ComponentRef::new("literoll180"),
            lite_roll_181: ComponentRef::new("literoll181"),
            lite_roll_182: ComponentRef::new("literoll182"),
            lite_roll_183: ComponentRef::new("literoll183"),
            lite_roll_184: ComponentRef::new("literoll184"),
            middle_circle: ComponentRef::new("middle_circle"),
            left_chute_target_lights: ComponentRef::new("lchute_tgt_lights"),
            left_trek_lights: ComponentRef::new("l_trek_lights"),
            goal_lights: ComponentRef::new("goal_lights"),
            hyperspace_lights: ComponentRef::new("hyperspace_lights"),
            bumper_increment_lights: ComponentRef::new("bmpr_inc_lights"),
            bumper_solo_target_lights: ComponentRef::new("bpr_solotgt_lights"),
            black_hole_sink_arrow_lights: ComponentRef::new("bsink_arrow_lights"),
            bumper_target_lights: ComponentRef::new("bumper_target_lights"),
            outer_circle: ComponentRef::new("outer_circle"),
            right_trek_lights: ComponentRef::new("r_trek_lights"),
            ramp_bumper_increment_lights: ComponentRef::new("ramp_bmpr_inc_lights"),
            ramp_target_lights: ComponentRef::new("ramp_tgt_lights"),
            skill_shot_lights: ComponentRef::new("skill_shot_lights"),
            top_circle_target_lights: ComponentRef::new("top_circle_tgt_lights"),
            top_target_lights: ComponentRef::new("top_target_lights"),
            worm_hole_lights: ComponentRef::new("worm_hole_lights"),
            soundwave_3: ComponentRef::new("soundwave3"),
            soundwave_7: ComponentRef::new("soundwave7"),
            soundwave_8: ComponentRef::new("soundwave8"),
            soundwave_9: ComponentRef::new("soundwave9"),
            soundwave_10: ComponentRef::new("soundwave10"),
            soundwave_14_1: ComponentRef::new("soundwave14"),
            soundwave_14_2: ComponentRef::new("soundwave14"),
            soundwave_21: ComponentRef::new("soundwave21"),
            soundwave_23: ComponentRef::new("soundwave23"),
            soundwave_24: ComponentRef::new("soundwave24"),
            soundwave_25: ComponentRef::new("soundwave25"),
            soundwave_26: ComponentRef::new("soundwave26"),
            soundwave_27: ComponentRef::new("soundwave27"),
            soundwave_28: ComponentRef::new("soundwave28"),
            soundwave_30: ComponentRef::new("soundwave30"),
            soundwave_35_1: ComponentRef::new("soundwave35"),
            soundwave_35_2: ComponentRef::new("soundwave35"),
            soundwave_36_1: ComponentRef::new("soundwave36"),
            soundwave_36_2: ComponentRef::new("soundwave36"),
            soundwave_38: ComponentRef::new("soundwave38"),
            soundwave_39: ComponentRef::new("soundwave39"),
            soundwave_40: ComponentRef::new("soundwave40"),
            soundwave_41: ComponentRef::new("soundwave41"),
            soundwave_44: ComponentRef::new("soundwave44"),
            soundwave_45: ComponentRef::new("soundwave45"),
            soundwave_46: ComponentRef::new("soundwave46"),
            soundwave_47: ComponentRef::new("soundwave47"),
            soundwave_48: ComponentRef::new("soundwave48"),
            soundwave_49_d: ComponentRef::new("soundwave49D"),
            soundwave_50_1: ComponentRef::new("soundwave50"),
            soundwave_50_2: ComponentRef::new("soundwave50"),
            soundwave_52: ComponentRef::new("soundwave52"),
            soundwave_59: ComponentRef::new("soundwave59"),
        }
    }
}

impl ComponentState {
    pub(crate) fn link_all(&mut self, table: &crate::t_pinball_table::TPinballTable) {
        crate::control::link_component(table, &mut self.block_1);
        crate::control::link_component(table, &mut self.bump_1);
        crate::control::link_component(table, &mut self.bump_2);
        crate::control::link_component(table, &mut self.bump_3);
        crate::control::link_component(table, &mut self.bump_4);
        crate::control::link_component(table, &mut self.bump_5);
        crate::control::link_component(table, &mut self.bump_6);
        crate::control::link_component(table, &mut self.bump_7);
        crate::control::link_component(table, &mut self.flip_1);
        crate::control::link_component(table, &mut self.flip_2);
        crate::control::link_component(table, &mut self.gate_1);
        crate::control::link_component(table, &mut self.gate_2);
        crate::control::link_component(table, &mut self.info_text_box);
        crate::control::link_component(table, &mut self.mission_text_box);
        crate::control::link_component(table, &mut self.plunger);
        crate::control::link_component(table, &mut self.rebo_1);
        crate::control::link_component(table, &mut self.rebo_2);
        crate::control::link_component(table, &mut self.rebo_3);
        crate::control::link_component(table, &mut self.rebo_4);
        crate::control::link_component(table, &mut self.lite_1);
        crate::control::link_component(table, &mut self.lite_2);
        crate::control::link_component(table, &mut self.lite_3);
        crate::control::link_component(table, &mut self.lite_4);
        crate::control::link_component(table, &mut self.lite_5);
        crate::control::link_component(table, &mut self.lite_6);
        crate::control::link_component(table, &mut self.lite_7);
        crate::control::link_component(table, &mut self.lite_8);
        crate::control::link_component(table, &mut self.lite_9);
        crate::control::link_component(table, &mut self.lite_10);
        crate::control::link_component(table, &mut self.lite_11);
        crate::control::link_component(table, &mut self.lite_12);
        crate::control::link_component(table, &mut self.lite_13);
        crate::control::link_component(table, &mut self.lite_16);
        crate::control::link_component(table, &mut self.lite_17);
        crate::control::link_component(table, &mut self.lite_18);
        crate::control::link_component(table, &mut self.lite_19);
        crate::control::link_component(table, &mut self.lite_20);
        crate::control::link_component(table, &mut self.lite_21);
        crate::control::link_component(table, &mut self.lite_22);
        crate::control::link_component(table, &mut self.lite_23);
        crate::control::link_component(table, &mut self.lite_24);
        crate::control::link_component(table, &mut self.lite_25);
        crate::control::link_component(table, &mut self.lite_26);
        crate::control::link_component(table, &mut self.lite_27);
        crate::control::link_component(table, &mut self.lite_28);
        crate::control::link_component(table, &mut self.lite_29);
        crate::control::link_component(table, &mut self.lite_30);
        crate::control::link_component(table, &mut self.lite_38);
        crate::control::link_component(table, &mut self.lite_39);
        crate::control::link_component(table, &mut self.lite_40);
        crate::control::link_component(table, &mut self.lite_54);
        crate::control::link_component(table, &mut self.lite_55);
        crate::control::link_component(table, &mut self.lite_56);
        crate::control::link_component(table, &mut self.lite_58);
        crate::control::link_component(table, &mut self.lite_59);
        crate::control::link_component(table, &mut self.lite_60);
        crate::control::link_component(table, &mut self.lite_61);
        crate::control::link_component(table, &mut self.lite_62);
        crate::control::link_component(table, &mut self.lite_67);
        crate::control::link_component(table, &mut self.lite_68);
        crate::control::link_component(table, &mut self.lite_69);
        crate::control::link_component(table, &mut self.lite_70);
        crate::control::link_component(table, &mut self.lite_71);
        crate::control::link_component(table, &mut self.lite_72);
        crate::control::link_component(table, &mut self.lite_77);
        crate::control::link_component(table, &mut self.lite_84);
        crate::control::link_component(table, &mut self.lite_85);
        crate::control::link_component(table, &mut self.lite_101);
        crate::control::link_component(table, &mut self.lite_102);
        crate::control::link_component(table, &mut self.lite_103);
        crate::control::link_component(table, &mut self.lite_104);
        crate::control::link_component(table, &mut self.lite_105);
        crate::control::link_component(table, &mut self.lite_106);
        crate::control::link_component(table, &mut self.lite_107);
        crate::control::link_component(table, &mut self.lite_108);
        crate::control::link_component(table, &mut self.lite_109);
        crate::control::link_component(table, &mut self.lite_110);
        crate::control::link_component(table, &mut self.lite_130);
        crate::control::link_component(table, &mut self.lite_131);
        crate::control::link_component(table, &mut self.lite_132);
        crate::control::link_component(table, &mut self.lite_133);
        crate::control::link_component(table, &mut self.lite_169);
        crate::control::link_component(table, &mut self.lite_170);
        crate::control::link_component(table, &mut self.lite_171);
        crate::control::link_component(table, &mut self.lite_195);
        crate::control::link_component(table, &mut self.lite_196);
        crate::control::link_component(table, &mut self.lite_198);
        crate::control::link_component(table, &mut self.lite_199);
        crate::control::link_component(table, &mut self.lite_200);
        crate::control::link_component(table, &mut self.lite_300);
        crate::control::link_component(table, &mut self.lite_301);
        crate::control::link_component(table, &mut self.lite_302);
        crate::control::link_component(table, &mut self.lite_303);
        crate::control::link_component(table, &mut self.lite_304);
        crate::control::link_component(table, &mut self.lite_305);
        crate::control::link_component(table, &mut self.lite_306);
        crate::control::link_component(table, &mut self.lite_307);
        crate::control::link_component(table, &mut self.lite_308);
        crate::control::link_component(table, &mut self.lite_309);
        crate::control::link_component(table, &mut self.lite_310);
        crate::control::link_component(table, &mut self.lite_311);
        crate::control::link_component(table, &mut self.lite_312);
        crate::control::link_component(table, &mut self.lite_313);
        crate::control::link_component(table, &mut self.lite_314);
        crate::control::link_component(table, &mut self.lite_315);
        crate::control::link_component(table, &mut self.lite_316);
        crate::control::link_component(table, &mut self.lite_317);
        crate::control::link_component(table, &mut self.lite_318);
        crate::control::link_component(table, &mut self.lite_319);
        crate::control::link_component(table, &mut self.lite_320);
        crate::control::link_component(table, &mut self.lite_321);
        crate::control::link_component(table, &mut self.lite_322);
        crate::control::link_component(table, &mut self.lite_roll_179);
        crate::control::link_component(table, &mut self.lite_roll_180);
        crate::control::link_component(table, &mut self.lite_roll_181);
        crate::control::link_component(table, &mut self.lite_roll_182);
        crate::control::link_component(table, &mut self.lite_roll_183);
        crate::control::link_component(table, &mut self.lite_roll_184);
        crate::control::link_component(table, &mut self.middle_circle);
        crate::control::link_component(table, &mut self.left_chute_target_lights);
        crate::control::link_component(table, &mut self.left_trek_lights);
        crate::control::link_component(table, &mut self.goal_lights);
        crate::control::link_component(table, &mut self.hyperspace_lights);
        crate::control::link_component(table, &mut self.bumper_increment_lights);
        crate::control::link_component(table, &mut self.bumper_solo_target_lights);
        crate::control::link_component(table, &mut self.black_hole_sink_arrow_lights);
        crate::control::link_component(table, &mut self.bumper_target_lights);
        crate::control::link_component(table, &mut self.outer_circle);
        crate::control::link_component(table, &mut self.right_trek_lights);
        crate::control::link_component(table, &mut self.ramp_bumper_increment_lights);
        crate::control::link_component(table, &mut self.ramp_target_lights);
        crate::control::link_component(table, &mut self.skill_shot_lights);
        crate::control::link_component(table, &mut self.top_circle_target_lights);
        crate::control::link_component(table, &mut self.top_target_lights);
        crate::control::link_component(table, &mut self.worm_hole_lights);
        crate::control::link_component(table, &mut self.soundwave_3);
        crate::control::link_component(table, &mut self.soundwave_7);
        crate::control::link_component(table, &mut self.soundwave_8);
        crate::control::link_component(table, &mut self.soundwave_9);
        crate::control::link_component(table, &mut self.soundwave_10);
        crate::control::link_component(table, &mut self.soundwave_14_1);
        crate::control::link_component(table, &mut self.soundwave_14_2);
        crate::control::link_component(table, &mut self.soundwave_21);
        crate::control::link_component(table, &mut self.soundwave_23);
        crate::control::link_component(table, &mut self.soundwave_24);
        crate::control::link_component(table, &mut self.soundwave_25);
        crate::control::link_component(table, &mut self.soundwave_26);
        crate::control::link_component(table, &mut self.soundwave_27);
        crate::control::link_component(table, &mut self.soundwave_28);
        crate::control::link_component(table, &mut self.soundwave_30);
        crate::control::link_component(table, &mut self.soundwave_35_1);
        crate::control::link_component(table, &mut self.soundwave_35_2);
        crate::control::link_component(table, &mut self.soundwave_36_1);
        crate::control::link_component(table, &mut self.soundwave_36_2);
        crate::control::link_component(table, &mut self.soundwave_38);
        crate::control::link_component(table, &mut self.soundwave_39);
        crate::control::link_component(table, &mut self.soundwave_40);
        crate::control::link_component(table, &mut self.soundwave_41);
        crate::control::link_component(table, &mut self.soundwave_44);
        crate::control::link_component(table, &mut self.soundwave_45);
        crate::control::link_component(table, &mut self.soundwave_46);
        crate::control::link_component(table, &mut self.soundwave_47);
        crate::control::link_component(table, &mut self.soundwave_48);
        crate::control::link_component(table, &mut self.soundwave_49_d);
        crate::control::link_component(table, &mut self.soundwave_50_1);
        crate::control::link_component(table, &mut self.soundwave_50_2);
        crate::control::link_component(table, &mut self.soundwave_52);
        crate::control::link_component(table, &mut self.soundwave_59);
    }
}

pub struct ComponentRef<T> {
    pub name: &'static str,
    pub component: Option<Weak<RefCell<T>>>,
}

impl<T> ComponentRef<T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            component: None,
        }
    }

    pub fn get(&self) -> Option<Rc<RefCell<T>>> {
        self.component.as_ref().and_then(Weak::upgrade)
    }

    pub fn set(&mut self, component: &Rc<RefCell<T>>) {
        self.component = Some(Rc::downgrade(component));
    }
}
