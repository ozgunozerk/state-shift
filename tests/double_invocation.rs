use state_shift::{state_impl, type_state};

#[derive(Debug)]
struct Player {
    race: Race,
}

#[derive(Debug, PartialEq)]
enum Race {
    Orc,
    Human,
}

#[type_state(states = (Initial1, RaceSet1), slots = (Initial1))]
struct PlayerBuilder1 {
    race: Option<Race>,
}

#[state_impl]
impl PlayerBuilder1 {
    #[require(Initial1)] // require the default state for the constructor
    fn new() -> PlayerBuilder1 {
        PlayerBuilder1 { race: None }
    }

    #[require(Initial1)] // can be called only at `Initial` state.
    #[switch_to(RaceSet1)] // Transitions to `RaceSet` state
    fn set_race(self, race: Race) -> PlayerBuilder1 {
        PlayerBuilder1 { race: Some(race) }
    }

    #[require(RaceSet1)]
    fn build(self) -> Player {
        Player {
            race: self.race.expect("type safety ensures this is set"),
        }
    }
}

#[type_state(states = (Initial2, RaceSet2), slots = (Initial2))]
struct PlayerBuilder2 {
    race: Option<Race>,
}

#[state_impl]
impl PlayerBuilder2 {
    #[require(Initial2)] // require the default state for the constructor
    fn new() -> PlayerBuilder2 {
        PlayerBuilder2 { race: None }
    }

    #[require(Initial2)] // can be called only at `Initial` state.
    #[switch_to(RaceSet2)] // Transitions to `RaceSet` state
    fn set_race(self, race: Race) -> PlayerBuilder2 {
        PlayerBuilder2 { race: Some(race) }
    }

    #[require(RaceSet2)]
    fn build(self) -> Player {
        Player {
            race: self.race.expect("type safety ensures this is set"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_player_creation_works() {
        let player1 = PlayerBuilder1::new().set_race(Race::Human).build();
        let player2 = PlayerBuilder2::new().set_race(Race::Orc).build();

        assert_eq!(player1.race, Race::Human);
        assert_eq!(player2.race, Race::Orc);
    }
}
