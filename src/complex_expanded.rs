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
    Human,
}

// private module to seal traits
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

struct PlayerBuilder<State1, State2, State3> {
    race: Option<Race>,
    level: Option<u8>,
    skill_slots: Option<u8>,
    spell_slots: Option<u8>,
    state1: PhantomData<State1>,
    state2: PhantomData<State2>,
    state3: PhantomData<State3>,
}

impl<A, B, C> PlayerBuilder<A, B, C>
where
    A: Initial,
{
    fn new() -> PlayerBuilder<A, B, C> {
        PlayerBuilder {
            state1: PhantomData,
            state2: PhantomData,
            state3: PhantomData,
            race: None,
            level: None,
            skill_slots: None,
            spell_slots: None,
        }
    }

    fn set_race(self, race: Race) -> PlayerBuilder<RaceSetMarker, B, C> {
        PlayerBuilder {
            state1: PhantomData,
            state2: PhantomData,
            state3: PhantomData,
            race: Some(race),
            level: self.level,
            skill_slots: self.skill_slots,
            spell_slots: self.spell_slots,
        }
    }
}

impl<A, B, C> PlayerBuilder<A, B, C>
where
    A: RaceSet,
{
    fn print_race(&self) {
        println!(
            "{:?}",
            self.race
                .as_ref()
                .expect("type safety ensures that `race` is set")
        );
    }

    fn set_level(self, level_modifier: u8) -> PlayerBuilder<A, LevelSetMarker, C> {
        let level = match self.race {
            Some(Race::Orc) => level_modifier + 2, // Orc's have +2 level advantage
            Some(Race::Human) => level_modifier,   // humans are weak
            None => unreachable!("type safety ensures that `race` is initialized"),
        };

        PlayerBuilder {
            state1: PhantomData,
            state2: PhantomData,
            state3: PhantomData,
            race: self.race,
            level: Some(level),
            skill_slots: self.skill_slots,
            spell_slots: self.spell_slots,
        }
    }

    fn set_skill_slots(self, skill_slot_modifier: u8) -> PlayerBuilder<A, B, SkillSlotsSetMarker> {
        let skill_slots = match self.race {
            Some(Race::Orc) => skill_slot_modifier,
            Some(Race::Human) => skill_slot_modifier + 1, // Human's have +1 skill slot advantage
            None => unreachable!("type safety ensures that `race` should be initialized"),
        };

        PlayerBuilder {
            state1: PhantomData,
            state2: PhantomData,
            state3: PhantomData,
            race: self.race,
            level: self.level,
            skill_slots: Some(skill_slots),
            spell_slots: self.spell_slots,
        }
    }
}

impl<A, B, C> PlayerBuilder<A, B, C>
where
    B: LevelSet,
    C: SkillSlotsSet,
{
    fn set_spells(self, spell_slot_modifier: u8) -> PlayerBuilder<SpellSlotsSetMarker, B, C> {
        let level = self
            .level
            .expect("type safety ensures that `level` is initialized");
        let skill_slots = self
            .skill_slots
            .as_ref()
            .expect("type safety ensures that `skill_slots` is initialized");

        let spell_slots = level / 10 + skill_slots + spell_slot_modifier;

        PlayerBuilder {
            state1: PhantomData,
            state2: PhantomData,
            state3: PhantomData,
            race: self.race,
            level: self.level,
            skill_slots: self.skill_slots,
            spell_slots: Some(spell_slots),
        }
    }
}

impl<A, B, C> PlayerBuilder<A, B, C> {
    fn say_hi(self) -> Self {
        println!("Hi!");

        PlayerBuilder {
            state1: PhantomData,
            state2: PhantomData,
            state3: PhantomData,
            race: self.race,
            level: self.level,
            skill_slots: self.skill_slots,
            spell_slots: self.spell_slots,
        }
    }
}

impl<A, B, C> PlayerBuilder<A, B, C>
where
    A: SpellSlotsSet,
{
    fn build(self) -> Player {
        Player {
            race: self.race.expect("type safety ensures this is set"),
            level: self.level.expect("type safety ensures this is set"),
            skill_slots: self.skill_slots.expect("type safety ensures this is set"),
            spell_slots: self.spell_slots.expect("type safety ensures this is set"),
        }
    }
}

fn main() {
    let player = PlayerBuilder::<InitialMarker, InitialMarker, InitialMarker>::new()
        .set_race(Race::Orc)
        .set_level(1)
        .set_skill_slots(1)
        .set_spells(1)
        .build();

    println!("{:?}", player);
}
