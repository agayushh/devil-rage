//! Original stages. Same loop as Level Devil — doors of five, three worlds,
//! keys, a true door, two player — written for a terminal, not copied from Unept.

use crate::level::{LevelDef, Trap};
use crate::model::{CloneKind, Save, Step, KEY_TOTAL};

pub struct WorldInfo {
    pub name: &'static str,
    pub tag: &'static str,
    pub doors: &'static [&'static str],
}

pub static WORLDS: &[WorldInfo] = &[
    WorldInfo {
        name: "Level Devil",
        tag: "every floor is a rumor",
        doors: &["Floor", "Spikes", "Motion", "Liars", "Threshold"],
    },
    WorldInfo {
        name: "Level Devil-er",
        tag: "the rules walked out",
        doors: &["Burn", "Ballistics", "House Rules"],
    },
    WorldInfo {
        name: "Level Devil-est",
        tag: "bring someone to die with",
        doors: &["Company", "Glitch"],
    },
];

pub static CAMPAIGN: &[LevelDef] = &[
    LevelDef {
        world: 0,
        door: 0,
        stage: 0,
        name: "A Normal Floor",
        hint: "the middle is a rumor",
        key: None,
        map: " @                          D           \n##########MMMMM#########################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 0,
        stage: 1,
        name: "Please Walk Carefully",
        hint: "stopping is how the floor wins",
        key: None,
        map: " @                                  D   \n######CCCCCCCCCCCCCCCCCCCCCCCCCC########",
        traps: &[Trap::Delays { group: 0.22, solo: 0.50 }],
    },
    LevelDef {
        world: 0,
        door: 0,
        stage: 2,
        name: "Instant Regret",
        hint: "some floors never agreed to exist",
        key: None,
        map: " @                            D         \n##########FFFF##########################\n          ^^^^                          ",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 0,
        stage: 3,
        name: "The Doorstep",
        hint: "arrive in the air, or don't arrive",
        key: None,
        map: " @                D                     \n################MMMM####################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 0,
        stage: 4,
        name: "Two Lies",
        hint: "one falls later. one was never there",
        key: None,
        map: " @                                D     \n########FFFF########MMMMM###############\n        ^^^^        ^^^^^               ",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 1,
        stage: 0,
        name: "Say Hello",
        hint: "they introduce themselves late",
        key: None,
        map: " @              ,,,               D     \n########################################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 1,
        stage: 1,
        name: "Free Money",
        hint: "look up only if you like surprises",
        key: None,
        map: "        vvvvvvvvvvvvvv                  \n              o                         \n @                                  D   \n########################################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 1,
        stage: 2,
        name: "The Low Road",
        hint: "the short way spends you",
        key: Some(0),
        map: "          K                             \n    ############                        \n @      o                           D   \n########################################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 1,
        stage: 3,
        name: "Pendulum",
        hint: "wait until it looks away",
        key: None,
        map: " @           A                    D     \n##########    ##########################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 1,
        stage: 4,
        name: "Don't Stop",
        hint: "hesitation is edible",
        key: None,
        map: " @            ^         ^           D   \n########################################",
        traps: &[Trap::Chase { row: 0, speed: 4.0 }],
    },
    LevelDef {
        world: 0,
        door: 2,
        stage: 0,
        name: "Coming Through",
        hint: "it has somewhere to be. so do you",
        key: None,
        map: "                                        \n                                        \n @                                  D   \n########################################",
        traps: &[Trap::Wall { x: 22.0, y: 2.0, w: 2.0, h: 1.0, tx: 4.0, ty: 2.0, speed: 6.5, at: 6.0 }],
    },
    LevelDef {
        world: 0,
        door: 2,
        stage: 1,
        name: "The Lift",
        hint: "stand on it. do not outrun it",
        key: None,
        map: " @                                  D   \n########                      ##########",
        traps: &[Trap::Platform { x: 4.0, y: 1.0, w: 6.0, range: 22.0, speed: 0.9, vertical: false }],
    },
    LevelDef {
        world: 0,
        door: 2,
        stage: 2,
        name: "Boing",
        hint: "hold the direction you mean",
        key: None,
        map: " @                            D         \n#######B       #########################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 2,
        stage: 3,
        name: "Black Ice",
        hint: "enter slowly or donate yourself to the pit",
        key: None,
        map: " @                                  D   \n############~~~~~~~~~~~~    ############",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 2,
        stage: 4,
        name: "About Face",
        hint: "your first instinct is the pit",
        key: None,
        map: "      @                           D     \n    ##############    ##################",
        traps: &[Trap::Reverse { at: 0.0, sticky: true }],
    },
    LevelDef {
        world: 0,
        door: 3,
        stage: 0,
        name: "Not That One",
        hint: "the obvious exit pays rent to the spikes",
        key: Some(1),
        map: "                                        \n @                                  d   \n########    ############################\n                K               D       \n########################################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 3,
        stage: 1,
        name: "Middle Child",
        hint: "two of them are innocent",
        key: None,
        map: " @        *       o       *         D   \n########################################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 3,
        stage: 2,
        name: "Wrong Mouth",
        hint: "the low circle is a rumor with teeth",
        key: None,
        map: "    p                         q   D     \n  ^^^^^                     ############\n  @       p       q                     \n########################################\n                                        ",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 3,
        stage: 3,
        name: "Upside Down",
        hint: "after the line, the ceiling is the floor",
        key: Some(2),
        map: "########################################\n                      K           D     \n @                                      \n####################^^^^^^^^############",
        traps: &[Trap::Gravity { at: 8.0 }],
    },
    LevelDef {
        world: 0,
        door: 3,
        stage: 4,
        name: "Too Much",
        hint: "the tunnel hates ambition. the pit demands it",
        key: None,
        map: "################                        \n   vvvvvvvvvv                           \n @                                  D   \n##################       ###############",
        traps: &[Trap::HighJump { at: 14.0, scale: 1.58 }],
    },
    LevelDef {
        world: 0,
        door: 4,
        stage: 0,
        name: "Memory",
        hint: "you have met both of these",
        key: None,
        map: " @                  ,,              D   \n########MMMM############################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 4,
        stage: 1,
        name: "It Moved",
        hint: "let it leave, then go where it went",
        key: Some(3),
        map: "                      K                 \n              ##############            \n @                                D     \n########################################",
        traps: &[Trap::MoveDoor { at: 12.0, tx: 20.0, ty: 0.0, speed: 16.0 }],
    },
    LevelDef {
        world: 0,
        door: 4,
        stage: 2,
        name: "Spring Cleaning",
        hint: "the arc goes over the teeth",
        key: None,
        map: " @         A                      D     \n######B#      ##########################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 0,
        door: 4,
        stage: 3,
        name: "Left Foot Ice",
        hint: "press the wrong key, slowly",
        key: None,
        map: "      @                           D     \n    ######~~~~~~~~~~~~~~~~~~############",
        traps: &[Trap::Reverse { at: 0.0, sticky: true }],
    },
    LevelDef {
        world: 0,
        door: 4,
        stage: 4,
        name: "Threshold",
        hint: "the high door is a costume",
        key: Some(4),
        map: "                                    d   \n @      ,                   D K         \n##############MMMM######################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 1,
        door: 0,
        stage: 0,
        name: "Pack",
        hint: "hold jump. the ceiling is your friend",
        key: None,
        map: "########################################\n @                                  D   \n########                    ############",
        traps: &[Trap::Jetpack(4.0)],
    },
    LevelDef {
        world: 1,
        door: 0,
        stage: 1,
        name: "Tap",
        hint: "jump where the ceiling is bald",
        key: Some(5),
        map: "vvvvvvv         vvvv        vvvvvvvvvvvv\n                                        \n @                K                 D   \n##########    ########    ##############",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 1,
        door: 0,
        stage: 2,
        name: "Thirsty",
        hint: "the coin drinks the tank",
        key: None,
        map: "              ##############            \n @                                  D   \n############              ##############",
        traps: &[Trap::Jetpack(3.0), Trap::FuelDrain { x: 14.0, y: 0.0, w: 14.0, h: 2.0, rate: 6.0 }],
    },
    LevelDef {
        world: 1,
        door: 0,
        stage: 3,
        name: "Windows",
        hint: "move when the lane is empty",
        key: None,
        map: "                                        \n>@        ###         ###           D   \n########################################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 1,
        door: 0,
        stage: 4,
        name: "It Quits",
        hint: "land before the pack remembers it has a job",
        key: Some(6),
        map: "#######################                 \n @                  K               D   \n############################    ########",
        traps: &[Trap::Jetpack(3.5), Trap::JetpackOff { at: 14.0 }],
    },
    LevelDef {
        world: 1,
        door: 1,
        stage: 0,
        name: "Cover",
        hint: "the blocks are the only honest things here",
        key: None,
        map: "                                        \n> @     #         #         #       D   \n########################################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 1,
        door: 1,
        stage: 1,
        name: "Sideways",
        hint: "down is a direction with options",
        key: None,
        map: "                                        \n  @                               D   X \n################    ####################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 1,
        door: 1,
        stage: 2,
        name: "Closer",
        hint: "faster than the last one. still slower than you",
        key: Some(7),
        map: " @          ^         K   ^         D   \n########################################",
        traps: &[Trap::Chase { row: 0, speed: 5.4 }],
    },
    LevelDef {
        world: 1,
        door: 1,
        stage: 3,
        name: "Both",
        hint: "the floor and the bullet want the same thing",
        key: None,
        map: ">@                                  D   \n############CCCCCCCC####################",
        traps: &[Trap::Delays { group: 0.2, solo: 0.28 }],
    },
    LevelDef {
        world: 1,
        door: 1,
        stage: 4,
        name: "Leave",
        hint: "the circle is the exit. the gun is the lesson",
        key: None,
        map: "                                        \n>@  ### q                         q D   \n########################################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 1,
        door: 2,
        stage: 0,
        name: "One",
        hint: "the first hole is a rumor. the second is not",
        key: Some(8),
        map: " @                K                 D   \n##########GGGGGG##############    ######",
        traps: &[Trap::JumpLimit(1)],
    },
    LevelDef {
        world: 1,
        door: 2,
        stage: 1,
        name: "Optional Ground",
        hint: "tap. the ceiling is keeping score",
        key: None,
        map: "########################################\n                                        \n        vvvvvv      vvvvvvvv            \n @                                  D   \n########################################",
        traps: &[Trap::Infinite { at: 0.0 }],
    },
    LevelDef {
        world: 1,
        door: 2,
        stage: 2,
        name: "Manners",
        hint: "wide for the hole, small for the slot",
        key: None,
        map: "          ########                      \n @    +               -       # #   D   \n############# ##########################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 1,
        door: 2,
        stage: 3,
        name: "Promotion",
        hint: "the button is not a gift",
        key: None,
        map: "################                        \n                    ####################\n  @     t   n                       D   \n########################################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 1,
        door: 2,
        stage: 4,
        name: "House Rules",
        hint: "you know every one of these. they know you",
        key: Some(9),
        map: "                                        \n>@              K     o           D     \n########MMMM############################",
        traps: &[Trap::Delays { group: 0.18, solo: 0.4 }],
    },
    LevelDef {
        world: 2,
        door: 0,
        stage: 0,
        name: "Plus One",
        hint: "whatever you survive, they have to survive too",
        key: None,
        map: "           ^      @         ^     D     \n########################################",
        traps: &[Trap::Clone(CloneKind::Mirror)],
    },
    LevelDef {
        world: 2,
        door: 0,
        stage: 1,
        name: "Shared Appetite",
        hint: "jump the snack. your shadow has a spike",
        key: Some(10),
        map: "             ^    @       o   K   D     \n########################################",
        traps: &[Trap::Clone(CloneKind::Mirror)],
    },
    LevelDef {
        world: 2,
        door: 0,
        stage: 2,
        name: "Opposite Day",
        hint: "you go to the door. they go the other way",
        key: None,
        map: "          @   ^         D     2     ^   \n########################################",
        traps: &[Trap::Clone(CloneKind::Opposite)],
    },
    LevelDef {
        world: 2,
        door: 0,
        stage: 3,
        name: "Same Buttons",
        hint: "two floors, one cowardice",
        key: None,
        map: "                                        \n                                        \n 2                            E         \n##############  ########################\n                                        \n                                        \n @                            D         \n##############  ########################",
        traps: &[Trap::Clone(CloneKind::Shadow)],
    },
    LevelDef {
        world: 2,
        door: 0,
        stage: 4,
        name: "Both Doors",
        hint: "neither of you is allowed to be the hero alone",
        key: Some(11),
        map: "                                        \n 2                K               E     \n##############MMM#######################\n @                                D     \n##############MMM#######################",
        traps: &[Trap::Clone(CloneKind::Dual), Trap::Delays { group: 0.55, solo: 0.4 }],
    },
    LevelDef {
        world: 2,
        door: 1,
        stage: 0,
        name: "Blink",
        hint: "it is solid if you do not think about it",
        key: None,
        map: " @                                  D   \n###ffff#################################",
        traps: &[Trap::FlickerPeriod(1.35)],
    },
    LevelDef {
        world: 2,
        door: 1,
        stage: 1,
        name: "Unlisted",
        hint: "the gap filed paperwork",
        key: None,
        map: " @                                  D   \n########GGGGGGGGGGGGGGGGGGGGGG##########",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 2,
        door: 1,
        stage: 2,
        name: "Late Edit",
        hint: "the door relocates. the spikes do too",
        key: Some(12),
        map: "                                        \n @          ,,    K                 D   \n########################################",
        traps: &[Trap::MoveDoor { at: 8.0, tx: 28.0, ty: 1.0, speed: 14.0 }],
    },
    LevelDef {
        world: 2,
        door: 1,
        stage: 3,
        name: "Stack",
        hint: "nothing here is new. that is the problem",
        key: None,
        map: ">                                       \n  @                 o               D   \n########MMM#############################",
        traps: &[Trap::Delays { group: 0.55, solo: 0.42 }],
    },
    LevelDef {
        world: 2,
        door: 1,
        stage: 4,
        name: "Goodbye",
        hint: "the far door is the credits. the near door is the door",
        key: None,
        map: "                                        \n  @     D                           d   \n################CCCCCC##################",
        traps: &[Trap::Delays { group: 0.2, solo: 0.36 }, Trap::Taunt { at: 24.0, text: "almost" }],
    },
];

pub static SECRET: &[LevelDef] = &[
    LevelDef {
        world: 9,
        door: 0,
        stage: 0,
        name: "Read the Footer",
        hint: "the tip at the bottom is the trap",
        key: None,
        map: "    ^   @                         D     \n      ##################################",
        traps: &[Trap::Reverse { at: 0.0, sticky: true }, Trap::Lie],
    },
    LevelDef {
        world: 9,
        door: 0,
        stage: 1,
        name: "Greatest Hits",
        hint: "you have died to all of these",
        key: None,
        map: "                                        \n @                  o ,             D   \n##########MMM###########################",
        traps: &[Trap::Delays { group: 0.55, solo: 0.42 }],
    },
    LevelDef {
        world: 9,
        door: 0,
        stage: 2,
        name: "For Once",
        hint: "walk. it is only a door",
        key: None,
        map: "                          ######        \n  @                         D           \n########################################",
        traps: &[Trap::Peace, Trap::NoSkip],
    },
];

pub static VERSUS: &[LevelDef] = &[
    LevelDef {
        world: 8,
        door: 0,
        stage: 0,
        name: "Shared Floor",
        hint: "the bridge falls for both of you",
        key: None,
        map: "  @ 1                               D   \n##############MMMMMM####################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 8,
        door: 0,
        stage: 1,
        name: "Same Surprise",
        hint: "whoever runs first wakes them",
        key: None,
        map: "  @ 1           ,,    ,,            D   \n########################################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 8,
        door: 0,
        stage: 2,
        name: "Both Wrong",
        hint: "the pit is behind the instinct",
        key: None,
        map: "    @ 1                           D     \n   #####################################",
        traps: &[Trap::Reverse { at: 0.0, sticky: true }],
    },
    LevelDef {
        world: 8,
        door: 0,
        stage: 3,
        name: "One Saw",
        hint: "it does not take turns",
        key: None,
        map: "  @ 1             A                 D   \n##############          ################",
        traps: &[] as &'static [Trap],
    },
    LevelDef {
        world: 8,
        door: 0,
        stage: 4,
        name: "Betrayal",
        hint: "the button is not for you",
        key: None,
        map: "  @  1      !     o                 D   \n########################################",
        traps: &[] as &'static [Trap],
    },
];


pub fn find(world: u8, door: u8, stage: u8) -> Option<&'static LevelDef> {
    CAMPAIGN.iter().find(|l| l.world == world && l.door == door && l.stage == stage)
}

pub fn door_count(world: u8) -> usize {
    CAMPAIGN.iter().filter(|l| l.world == world).map(|l| l.door as usize).max().map(|d| d + 1).unwrap_or(0)
}

pub fn stage_count(world: u8, door: u8) -> usize {
    CAMPAIGN.iter().filter(|l| l.world == world && l.door == door).count()
}

pub fn bump(save: &mut Save) -> Step {
    if save.cleared || save.world >= 3 {
        save.cleared = true;
        save.world = 3;
        return Step::Done;
    }
    let stages = stage_count(save.world, save.door) as u8;
    if stages == 0 {
        save.cleared = true;
        return Step::Done;
    }
    save.stage += 1;
    if save.stage < stages {
        return Step::Stage;
    }
    save.stage = 0;
    let doors = door_count(save.world) as u8;
    save.door += 1;
    if save.door < doors {
        return Step::Door;
    }
    save.door = 0;
    save.world += 1;
    if save.world < 3 {
        return Step::World;
    }
    save.cleared = true;
    save.world = 3;
    save.door = 0;
    save.stage = 0;
    Step::Done
}

pub fn check_all() -> Result<(), String> {
    use crate::sim::Sim;
    for def in CAMPAIGN.iter().chain(SECRET.iter()) {
        Sim::boot(def, &[], false)?;
    }
    for def in VERSUS.iter() {
        Sim::boot(def, &[], true)?;
    }
    if CAMPAIGN.len() != 50 {
        return Err(format!("expected 50 campaign levels, got {}", CAMPAIGN.len()));
    }
    for w in 0..3 {
        let doors = door_count(w);
        if doors != WORLDS[w as usize].doors.len() {
            return Err(format!("world {w} door mismatch"));
        }
        for d in 0..doors {
            if stage_count(w as u8, d as u8) != 5 {
                return Err(format!("world {w} door {d} is not 5 stages"));
            }
        }
    }
    let mut keys: Vec<u8> = CAMPAIGN.iter().filter_map(|l| l.key).collect();
    keys.sort_unstable();
    if keys != (0..KEY_TOTAL).collect::<Vec<_>>() {
        return Err(format!("keys {keys:?}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::rehearse;

    #[test]
    fn structure() {
        check_all().unwrap();
    }

    #[test]
    fn bump_through_first_door() {
        let mut s = Save::default();
        assert_eq!(bump(&mut s), Step::Stage);
        assert_eq!((s.world, s.door, s.stage), (0, 0, 1));
        for _ in 0..3 {
            bump(&mut s);
        }
        assert_eq!(bump(&mut s), Step::Door);
        assert_eq!((s.world, s.door, s.stage), (0, 1, 0));
    }

    #[test]
    fn rehearse_opening() {
        rehearse(find(0, 0, 0).unwrap(), &[(1.05, 1, false), (0.75, 1, true), (1.6, 1, false)]).unwrap();
    }

    #[test]
    fn rehearse_keep_moving() {
        rehearse(find(0, 0, 1).unwrap(), &[(4.6, 1, false)]).unwrap();
    }

    #[test]
    fn rehearse_reverse() {
        rehearse(find(0, 2, 4).unwrap(), &[(1.15, -1, false), (0.7, -1, true), (1.6, -1, false)]).unwrap();
    }

    #[test]
    fn rehearse_ignore_the_coin() {
        rehearse(find(0, 1, 1).unwrap(), &[(4.6, 1, false)]).unwrap();
    }

    #[test]
    fn rehearse_gravity() {
        rehearse(find(0, 3, 3).unwrap(), &[(4.6, 1, false)]).unwrap();
    }

    #[test]
    fn rehearse_jetpack() {
        rehearse(find(1, 0, 0).unwrap(), &[(0.2, 1, false), (4.4, 1, true)]).unwrap();
    }

    #[test]
    fn every_stage_has_a_way_through() {
        let mut stuck = Vec::new();
        for def in CAMPAIGN.iter().chain(SECRET.iter()) {
            if !scripted_clear(def) {
                stuck.push(def.name);
            }
        }
        assert!(stuck.is_empty(), "no script cleared: {stuck:?}");
    }

    fn scripted_clear(def: &LevelDef) -> bool {
        let mut scripts: Vec<Vec<(f32, i8, bool)>> = vec![
            vec![(7.5, 1, false)],
            vec![(7.5, -1, false)],
            vec![(7.5, 1, true)],
            vec![(7.5, -1, true)],
            vec![(1.2, 0, false), (6.5, 1, false)],
            vec![(2.0, 0, false), (6.0, 1, false)],
            vec![(1.4, 0, false), (6.0, 1, true)],
            vec![(0.8, 1, false), (0.45, 1, true), (5.5, 1, false)],
            vec![(1.2, -1, false), (0.5, -1, true), (5.0, -1, false)],
            vec![(0.6, 1, false), (0.7, 1, true), (1.2, 1, false), (0.7, 1, true), (4.0, 1, false)],
            vec![(0.7, 1, false), (3.2, 0, false), (2.4, 1, false)],
            vec![(1.75, 1, false), (0.55, 1, true), (4.0, 1, false)],
            vec![(0.85, 1, false), (0.5, 1, true), (0.8, 1, false), (0.5, 1, true), (3.0, 1, false)],
            vec![(2.2, 1, true), (0.9, 1, false), (0.5, 1, true), (2.5, 1, false)],
            vec![(3.05, 1, false), (0.5, 1, true), (2.2, 1, false)],
            vec![(1.55, 1, false), (0.5, 1, true), (3.5, 1, false)],
            vec![(1.9, 1, false), (0.5, 1, true), (3.5, 1, false)],
            vec![(1.35, 1, false), (0.45, 1, true), (0.55, 1, false), (0.45, 1, true), (2.8, 1, false)],
            vec![(0.7, 1, false), (0.45, 1, true), (0.75, 1, false), (0.45, 1, true), (2.8, 1, false)],
            vec![(1.15, 1, false), (0.55, 1, true), (3.5, 1, false)],
            vec![(0.8, 0, false), (1.05, 1, false), (0.5, 1, true), (3.0, 1, false)],
            vec![(1.6, 0, false), (1.05, 1, false), (0.5, 1, true), (3.0, 1, false)],
            vec![(2.0, 1, true), (0.85, 1, false), (0.5, 1, true), (2.2, 1, false)],
        ];
        for dir in [1i8, -1] {
            let mut hop = Vec::new();
            for _ in 0..16 {
                hop.push((0.18, dir, true));
                hop.push((0.32, dir, false));
            }
            scripts.push(hop);
            let mut tap = Vec::new();
            for _ in 0..14 {
                tap.push((0.12, dir, true));
                tap.push((0.28, dir, false));
            }
            scripts.push(tap);
        }
        if scripts.iter().any(|s| rehearse(def, s).is_ok()) {
            return true;
        }
        for &wait in &[0.0f32, 0.7, 1.4, 2.2] {
            for &run in &[0.5f32, 1.0, 1.6, 2.3] {
                for &hold in &[0.4f32, 0.6] {
                    let one = [(wait, 0, false), (run, 1, false), (hold, 1, true), (4.0, 1, false)];
                    if rehearse(def, &one).is_ok() {
                        return true;
                    }
                    let two = [
                        (wait, 0, false),
                        (run, 1, false),
                        (hold, 1, true),
                        (0.6, 1, false),
                        (hold, 1, true),
                        (3.2, 1, false),
                    ];
                    if rehearse(def, &two).is_ok() {
                        return true;
                    }
                }
            }
        }
        false
    }

    #[test]
    fn fuzz_does_not_explode() {
        use crate::sim::{Sim, Sticks};
        for def in CAMPAIGN.iter().chain(SECRET.iter()) {
            let mut sim = Sim::boot(def, &[], false).unwrap();
            for f in 0..100 {
                let jump = f % 50 == 8;
                let dir = if f % 90 < 50 { 1 } else { -1 };
                sim.update(1.0 / 60.0, &Sticks::one(dir, jump, jump));
                assert!(sim.actors[0].x.is_finite(), "{}", def.name);
            }
        }
        for def in VERSUS {
            let mut sim = Sim::boot(def, &[], true).unwrap();
            sim.update(0.5, &Sticks::one(1, false, false));
            assert!(sim.actors.len() >= 2, "{}", def.name);
        }
    }
}
