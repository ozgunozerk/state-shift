/// Here you can find the `cargo expand` output of the `simple_example.rs` test file.
/// This is a rough `expansion`, and does not expand irrelevant parts of the code (e.g. `#[derive(Debug)]`, etc.)
///
/// This file serves the purpose of revealing what's happening behind the curtains.
use std::marker::PhantomData;

use type_state_macro::{require, states, switch_to, type_state};

#[derive(Debug)]
struct Player {
    race: Race,
    level: u8,
    skill_slots: u8,
}

#[derive(Debug, PartialEq)]
enum Race {
    #[allow(unused)]
    Orc,
    Human,
}

struct PlayerBuilder<State1> {
    race: Option<Race>,
    level: Option<u8>,
    skill_slots: Option<u8>,
    _state: (PhantomData<State1>),
}

pub trait Initial: sealed::Sealed {}
pub trait RaceSet: sealed::Sealed {}
pub trait LevelSet: sealed::Sealed {}
pub trait SkillSlotsSet: sealed::Sealed {}

struct InitialMarker;
struct RaceSetMarker;
struct LevelSetMarker;
struct SkillSlotsSetMarker;

impl sealed::Sealed for InitialMarker {}
impl sealed::Sealed for RaceSetMarker {}
impl sealed::Sealed for LevelSetMarker {}
impl sealed::Sealed for SkillSlotsSetMarker {}

impl Initial for InitialMarker {}
impl RaceSet for RaceSetMarker {}
impl LevelSet for LevelSetMarker {}
impl SkillSlotsSet for SkillSlotsSetMarker {}

// put the constructors in a separate impl block
impl PlayerBuilder {
    fn new() -> Self {
        PlayerBuilder {
            race: None,
            level: None,
            skill_slots: None,
            _state: (PhantomData),
        }
    }
}

impl<A> PlayerBuilder<A>
where
    A: Initial,
{
    fn set_race(self, race: Race) -> PlayerBuilder<RaceSetMarker> {
        PlayerBuilder {
            race: Some(race),
            level: self.level,
            skill_slots: self.skill_slots,
            _state: (PhantomData),
        }
    }
}
impl<A> PlayerBuilder<A>
where
    A: RaceSet,
{
    fn set_level(self, level_modifier: u8) -> PlayerBuilder<LevelSetMarker> {
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
impl<A> PlayerBuilder<A>
where
    A: RaceSet,
{
    fn set_skill_slots(self, skill_slot_modifier: u8) -> PlayerBuilder<SkillSlotsSetMarker> {
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
impl<A> PlayerBuilder<A> {
    fn say_hi(self) -> Self {
        println!("Hi!");

        self
    }
}
impl<A> PlayerBuilder<A>
where
    A: SkillSlotsSet,
{
    fn build(self) -> Player {
        Player {
            race: self.race.expect("type safety ensures this is set"),
            level: self.level.expect("type safety ensures this is set"),
            skill_slots: self.skill_slots.expect("type safety ensures this is set"),
        }
    }
}
