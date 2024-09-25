/// Here you can find the `cargo expand` output of the `complex_example.rs` test file.
/// This is a rough `expansion`, and does not expand irrelevant parts of the code (e.g. `#[derive(Debug)]`, etc.)
///
/// This file serves the purpose of revealing what's happening behind the curtains.
use std::marker::PhantomData;

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
    #[allow(unused)]
    Human,
}

struct PlayerBuilder<State1 = Initial, State2 = Initial, State3 = Initial>
where
    State1: TypeStateProtector,
    State2: TypeStateProtector,
    State3: TypeStateProtector,
{
    race: Option<Race>,
    level: Option<u8>,
    skill_slots: Option<u8>,
    spell_slots: Option<u8>,
    _state: (
        PhantomData<State1>,
        PhantomData<State2>,
        PhantomData<State3>,
    ),
}

mod sealed {
    pub trait Sealed {}
}

pub trait TypeStateProtector: sealed::Sealed {}

struct Initial;
struct RaceSet;
struct LevelSet;
struct SkillSlotsSet;
struct SpellSlotsSet;

impl sealed::Sealed for Initial {}
impl sealed::Sealed for RaceSet {}
impl sealed::Sealed for LevelSet {}
impl sealed::Sealed for SkillSlotsSet {}
impl sealed::Sealed for SpellSlotsSet {}

impl TypeStateProtector for Initial {}
impl TypeStateProtector for RaceSet {}
impl TypeStateProtector for LevelSet {}
impl TypeStateProtector for SkillSlotsSet {}
impl TypeStateProtector for SpellSlotsSet {}

// put the constructors in a separate impl block
impl PlayerBuilder {
    fn new() -> Self {
        PlayerBuilder {
            race: None,
            level: None,
            skill_slots: None,
            spell_slots: None,
            _state: (PhantomData, PhantomData, PhantomData),
        }
    }
}
impl<B, C> PlayerBuilder<Initial, B, C>
where
    B: TypeStateProtector,
    C: TypeStateProtector,
{
    fn set_race(self, race: Race) -> PlayerBuilder<RaceSet, B, C> {
        {
            {
                PlayerBuilder {
                    race: Some(race),
                    level: self.level,
                    skill_slots: self.skill_slots,
                    spell_slots: self.spell_slots,
                    _state: (PhantomData, PhantomData, PhantomData),
                }
            }
        }
    }
}
impl<B, C> PlayerBuilder<RaceSet, B, C>
where
    B: TypeStateProtector,
    C: TypeStateProtector,
{
    fn set_level(self, level_modifier: u8) -> PlayerBuilder<RaceSet, LevelSet, C> {
        {
            {
                let level = match self.race {
                    Some(Race::Orc) => level_modifier + 2,
                    Some(Race::Human) => level_modifier,
                    None => unreachable!("type safety ensures that `race` is initialized"),
                };
                PlayerBuilder {
                    race: self.race,
                    level: Some(level),
                    skill_slots: self.skill_slots,
                    spell_slots: self.spell_slots,
                    _state: (PhantomData, PhantomData, PhantomData),
                }
            }
        }
    }
}
impl<B, C> PlayerBuilder<RaceSet, B, C>
where
    B: TypeStateProtector,
    C: TypeStateProtector,
{
    fn set_skill_slots(self, skill_slot_modifier: u8) -> PlayerBuilder<RaceSet, B, SkillSlotsSet> {
        {
            {
                let skill_slots = match self.race {
                    Some(Race::Orc) => skill_slot_modifier,
                    Some(Race::Human) => skill_slot_modifier + 1,
                    None => unreachable!("type safety ensures that `race` should be initialized"),
                };
                PlayerBuilder {
                    race: self.race,
                    level: self.level,
                    skill_slots: Some(skill_slots),
                    spell_slots: self.spell_slots,
                    _state: (PhantomData, PhantomData, PhantomData),
                }
            }
        }
    }
}
impl<A> PlayerBuilder<A, LevelSet, SkillSlotsSet>
where
    A: TypeStateProtector,
{
    fn set_spells(
        self,
        spell_slot_modifier: u8,
    ) -> PlayerBuilder<SpellSlotsSet, LevelSet, SkillSlotsSet> {
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
                    _state: (PhantomData, PhantomData, PhantomData),
                }
            }
        }
    }
}
impl<A, B, C> PlayerBuilder<A, B, C>
where
    A: TypeStateProtector,
    B: TypeStateProtector,
    C: TypeStateProtector,
{
    fn say_hi(self) -> Self {
        println!("Hi!");

        self
    }
}
impl<B, C> PlayerBuilder<SpellSlotsSet, B, C>
where
    B: TypeStateProtector,
    C: TypeStateProtector,
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

fn main() {
    let player = PlayerBuilder::new()
        .set_race(Race::Orc)
        .set_level(10)
        .set_skill_slots(1)
        .set_spells(1)
        .say_hi()
        .build();

    println!("Race: {:?}", player.race);
    println!("Level: {}", player.level);
    println!("Skill slots: {}", player.skill_slots);
    println!("Spell slots: {}", player.spell_slots);
}
