/// Here you can find the `cargo expand` output of the `simple_example.rs` test file.
/// This is a rough `expansion`, and does not expand irrelevant parts of the code (e.g. `#[derive(Debug)]`, etc.)
///
/// This file serves the purpose of revealing what's happening behind the curtains.
use std::marker::PhantomData;

#[derive(Debug)]
struct Player {
    race: Race,
    level: u8,
    skill_slots: u8,
}

#[derive(Debug, PartialEq)]
enum Race {
    Orc,
    #[allow(unused)]
    Human,
}

struct PlayerBuilder<State1 = Initial> {
    race: Option<Race>,
    level: Option<u8>,
    skill_slots: Option<u8>,
    #[allow(unused_parens)]
    _state: PhantomData<fn() -> State1>,
}

mod sealed {
    pub trait Sealed {}
}

pub trait TypeStateProtector: sealed::Sealed {}

struct Initial;
struct RaceSet;
struct LevelSet;
struct SkillSlotsSet;

impl sealed::Sealed for Initial {}
impl sealed::Sealed for RaceSet {}
impl sealed::Sealed for LevelSet {}
impl sealed::Sealed for SkillSlotsSet {}

impl TypeStateProtector for Initial {}
impl TypeStateProtector for RaceSet {}
impl TypeStateProtector for LevelSet {}
impl TypeStateProtector for SkillSlotsSet {}

impl PlayerBuilder<Initial> {
    fn new() -> Self {
        PlayerBuilder {
            race: None,
            level: None,
            skill_slots: None,
            _state: (PhantomData),
        }
    }
}

impl PlayerBuilder<Initial> {
    fn set_race(self, race: Race) -> PlayerBuilder<RaceSet> {
        PlayerBuilder {
            race: Some(race),
            level: self.level,
            skill_slots: self.skill_slots,
            _state: (PhantomData),
        }
    }
}
impl PlayerBuilder<RaceSet> {
    fn set_level(self, level_modifier: u8) -> PlayerBuilder<LevelSet> {
        let level = match self.race {
            Some(Race::Orc) => level_modifier + 2, // Orc's have +2 level advantage
            Some(Race::Human) => level_modifier,   // humans are weak
            None => unreachable!("type safety ensures that `race` is initialized"),
        };

        PlayerBuilder {
            race: self.race,
            level: Some(level),
            skill_slots: self.skill_slots,
            _state: (PhantomData),
        }
    }
}
impl PlayerBuilder<LevelSet> {
    fn set_skill_slots(self, skill_slot_modifier: u8) -> PlayerBuilder<SkillSlotsSet> {
        let skill_slots = match self.race {
            Some(Race::Orc) => skill_slot_modifier,
            Some(Race::Human) => skill_slot_modifier + 1, // Human's have +1 skill slot advantage
            None => unreachable!("type safety ensures that `race` should be initialized"),
        };

        PlayerBuilder {
            race: self.race,
            level: self.level,
            skill_slots: Some(skill_slots),
            _state: (PhantomData),
        }
    }
}
impl<A> PlayerBuilder<A>
where
    A: TypeStateProtector,
{
    fn say_hi(self) -> Self {
        println!("Hi!");

        self
    }
}
impl PlayerBuilder<SkillSlotsSet> {
    fn build(self) -> Player {
        Player {
            race: self.race.expect("type safety ensures this is set"),
            level: self.level.expect("type safety ensures this is set"),
            skill_slots: self.skill_slots.expect("type safety ensures this is set"),
        }
    }
}

fn main() {
    let player = PlayerBuilder::new()
        .set_race(Race::Orc)
        .set_level(1)
        .set_skill_slots(1)
        .say_hi()
        .build();

    println!("Race: {:?}", player.race);
    println!("Level: {}", player.level);
    println!("Skill slots: {}", player.skill_slots);
}
