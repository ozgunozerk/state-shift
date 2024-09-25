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

#[type_slots(3)]
struct PlayerBuilder {
    race: Option<Race>,
    level: Option<u8>,
    skill_slots: Option<u8>,
    spell_slots: Option<u8>,
}

#[states(Initial, RaceSet, LevelSet, SkillSlotsSet, SpellSlotsSet)]
impl PlayerBuilder {
    #[require(Initial, B, C)] // an be called only at `Initial` state, and doesn't change the state
    fn new() -> Self {
        Self {
            race: None,
            level: None,
            skill_slots: None,
            spell_slots: None,
        }
    }

    #[require(Initial, B, C)] // can be called only at `Initial` state.
    #[switch_to(RaceSet, B, C)] // Transitions to `RaceSet` state
    fn set_race(self, race: Race) -> Self {
        Self {
            race: Some(race),
            ..self
        }
    }

    #[require(RaceSet, B, C)]
    #[switch_to(RaceSet, LevelSet, C)]
    fn set_level(self, level_modifier: u8) -> Self {
        let level = match self.race {
            Some(Race::Orc) => level_modifier + 2, // Orc's have +2 level advantage
            Some(Race::Human) => level_modifier,   // humans are weak
            None => unreachable!("type safety ensures that `race` is initialized"),
        };

        Self {
            level: Some(level),
            state2: PhantomData,
            ..self
        }
    }

    #[require(RaceSet, B, C)]
    #[switch_to(RaceSet, B, SkillSlotsSet)]
    fn set_skill_slots(self, skill_slot_modifier: u8) -> Self {
        let skill_slots = match self.race {
            Some(Race::Orc) => skill_slot_modifier,
            Some(Race::Human) => skill_slot_modifier + 1, // Human's have +1 skill slot advantage
            None => unreachable!("type safety ensures that `race` should be initialized"),
        };

        Self {
            skill_slots: Some(skill_slots),
            state3: PhantomData,
            ..self
        }
    }

    #[require(A, LevelSet, SkillSlotsSet)]
    #[switch_to(SpellSlotsSet, LevelSet, SkillSlotsSet)]
    fn set_spells(self, spell_slot_modifier: u8) -> Self {
        let level = self
            .level
            .expect("type safety ensures that `level` is initialized");
        let skill_slots = self
            .skill_slots
            .as_ref()
            .expect("type safety ensures that `skill_slots` is initialized");

        let spell_slots = level / 10 + skill_slots + spell_slot_modifier;

        Self {
            spell_slots: Some(spell_slots),
            state1: PhantomData,
            ..self
        }
    }

    /// doesn't require any state, so this is available at any state
    #[require(A, B, C)]
    fn say_hi(self) {
        println!("Hi!");

        Self
    }

    #[require(SpellSlotsSet, B, C)]
    fn build(self) -> Player {
        Player {
            race: self.race.expect("type safety ensures this is set"),
            level: self.level.expect("type safety ensures this is set"),
            skill_slots: self.skill_slots.expect("type safety ensures this is set"),
            spell_slots: self.spell_slots.expect("type safety ensures this is set"),
        }
    }
}
