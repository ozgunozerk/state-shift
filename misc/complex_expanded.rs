/// Here you can find the `cargo expand` output of the `complex_example.rs` test file.
/// This is a rough `expansion`, and does not expand irrelevant parts of the code (e.g. `#[derive(Debug)]`, etc.)
///
/// This file serves the purpose of revealing what's happening behind the curtains.
use std::marker::PhantomData;
use type_state_macro::{require, states, switch_to};

#[derive(Debug)]
struct Player {
    race: Race,
    level: u8,
    skill_slots: u8,
    spell_slots: u8,
}

#[derive(Debug)]
enum Race {
    Orc,
    Human,
}

struct PlayerBuilder<State1, State2, State3> {
    race: Option<Race>,
    level: Option<u8>,
    skill_slots: Option<u8>,
    spell_slots: Option<u8>,
    state: (
        PhantomData<State1>,
        PhantomData<State2>,
        PhantomData<State3>,
    ),
}

mod sealed {
    pub trait Sealed {}
}

pub trait Initial: sealed::Sealed {}
pub trait RaceSet: sealed::Sealed {}
pub trait LevelSet: sealed::Sealed {}
pub trait SkillSlotsSet: sealed::Sealed {}
pub trait SpellSlotsSet: sealed::Sealed {}

struct InitialMarker;
struct RaceSetMarker;
struct LevelSetMarker;
struct SkillSlotsSetMarker;
struct SpellSlotsSetMarker;

impl sealed::Sealed for InitialMarker {}
impl sealed::Sealed for RaceSetMarker {}
impl sealed::Sealed for LevelSetMarker {}
impl sealed::Sealed for SkillSlotsSetMarker {}
impl sealed::Sealed for SpellSlotsSetMarker {}

impl Initial for InitialMarker {}
impl RaceSet for RaceSetMarker {}
impl LevelSet for LevelSetMarker {}
impl SkillSlotsSet for SkillSlotsSetMarker {}
impl SpellSlotsSet for SpellSlotsSetMarker {}

// put the constructors in a separate impl block
impl PlayerBuilder {
    fn new() -> Self {
        PlayerBuilder {
            race: None,
            level: None,
            skill_slots: None,
            spell_slots: None,
            state: (PhantomData, PhantomData, PhantomData),
        }
    }
}
impl<A, B, C> PlayerBuilder<A, B, C>
where
    A: Initial,
{
    fn set_race(self, race: Race) -> PlayerBuilder<RaceSetMarker, B, C> {
        {
            {
                PlayerBuilder {
                    race: Some(race),
                    level: self.level,
                    skill_slots: self.skill_slots,
                    spell_slots: self.spell_slots,
                    state: (PhantomData, PhantomData, PhantomData),
                }
            }
        }
    }
}
impl<A, B, C> PlayerBuilder<A, B, C>
where
    A: RaceSet,
{
    fn set_level(self, level_modifier: u8) -> PlayerBuilder<RaceSetMarker, LevelSetMarker, C> {
        {
            {
                let level = match self.race {
                    Some(Race::Orc) => level_modifier + 2,
                    Some(Race::Human) => level_modifier,
                    None => {
                        ::core::panicking::panic_fmt(format_args!(
                            "internal error: entered unreachable code: {0}",
                            format_args!("type safety ensures that `race` is initialized")
                        ));
                    }
                };
                PlayerBuilder {
                    race: self.race,
                    level: Some(level),
                    skill_slots: self.skill_slots,
                    spell_slots: self.spell_slots,
                    state: (PhantomData, PhantomData, PhantomData),
                }
            }
        }
    }
}
impl<A, B, C> PlayerBuilder<A, B, C>
where
    A: RaceSet,
{
    fn set_skill_slots(
        self,
        skill_slot_modifier: u8,
    ) -> PlayerBuilder<RaceSetMarker, B, SkillSlotsSetMarker> {
        {
            {
                let skill_slots = match self.race {
                    Some(Race::Orc) => skill_slot_modifier,
                    Some(Race::Human) => skill_slot_modifier + 1,
                    None => {
                        ::core::panicking::panic_fmt(format_args!(
                            "internal error: entered unreachable code: {0}",
                            format_args!("type safety ensures that `race` should be initialized")
                        ));
                    }
                };
                PlayerBuilder {
                    race: self.race,
                    level: self.level,
                    skill_slots: Some(skill_slots),
                    spell_slots: self.spell_slots,
                    state: (PhantomData, PhantomData, PhantomData),
                }
            }
        }
    }
}
impl<A, B, C> PlayerBuilder<A, B, C>
where
    B: LevelSet,
    C: SkillSlotsSet,
{
    fn set_spells(
        self,
        spell_slot_modifier: u8,
    ) -> PlayerBuilder<SpellSlotsSetMarker, LevelSetMarker, SkillSlotsSetMarker> {
        {
            {
                let level = self
                    .level
                    .expect("type safety ensures that `level` is initialized");
                let skill_slots = self
                    .skill_slots
                    .as_ref()
                    .expect("type safety ensures that `skill_slots` is initialized");
                let spell_slots = level / 10 + skill_slots + spell_slot_modifier;
                PlayerBuilder {
                    race: self.race,
                    level: self.level,
                    skill_slots: self.skill_slots,
                    spell_slots: Some(spell_slots),
                    state: (PhantomData, PhantomData, PhantomData),
                }
            }
        }
    }
}
impl<A, B, C> PlayerBuilder<A, B, C> {
    fn say_hi(self) -> Self {
        {
            {
                ::std::io::_print(format_args!("Hi!\n"));
            };
            self
        }
    }
}
impl<A, B, C> PlayerBuilder<A, B, C>
where
    A: SpellSlotsSet,
{
    fn build(self) -> Player {
        {
            Player {
                race: self.race.expect("type safety ensures this is set"),
                level: self.level.expect("type safety ensures this is set"),
                skill_slots: self.skill_slots.expect("type safety ensures this is set"),
                spell_slots: self.spell_slots.expect("type safety ensures this is set"),
            }
        }
    }
}
