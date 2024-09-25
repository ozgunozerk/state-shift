use type_state_macro::{require, switch_to};

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

// #[require(Initial, B, C)] // an be called only at `Initial` state, and doesn't change the state

#[require(Initial, B, C)]
#[switch_to(RaceSetMarker, B, C)] // Transitions to `RaceSet` state
fn new() -> PlayerBuilder {
    PlayerBuilder {
        race: None,
        level: None,
        skill_slots: None,
        spell_slots: None,
        state1: PhantomData,
        state2: PhantomData,
        state3: PhantomData,
    }
}

fn main() {
    let player = PlayerBuilder::<InitialMarker, InitialMarker, InitialMarker>::new();
}
