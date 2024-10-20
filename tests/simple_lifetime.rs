use state_shift::{states, switch_to, type_state};

#[derive(Debug)]
struct Player<'a, T> {
    race: Race,
    level: u8,
    items: Vec<&'a T>,
}

#[derive(Debug, PartialEq)]
enum Race {
    #[allow(unused)]
    Orc,
    Human,
}

#[type_state(state_slots = 1, default_state = Initial)]
struct PlayerBuilder<'a, T> {
    race: Option<Race>,
    level: Option<u8>,
    items: Option<Vec<&'a T>>,
}

#[states(Initial, RaceSet, LevelSet, ItemsSet)]
impl<'a, T> PlayerBuilder<'a, T> {
    #[require(Initial)]
    fn new() -> PlayerBuilder<'a, T> {
        PlayerBuilder {
            race: None,
            level: None,
            items: None,
        }
    }

    #[require(Initial)]
    #[switch_to(RaceSet)]
    fn set_race(self, race: Race) -> PlayerBuilder<'a, T> {
        PlayerBuilder {
            race: Some(race),
            level: self.level,
            items: self.items,
        }
    }

    #[require(RaceSet)]
    #[switch_to(LevelSet)]
    fn set_level(self, level_modifier: u8) -> PlayerBuilder<'a, T> {
        let level = match self.race {
            Some(Race::Orc) => level_modifier + 2,
            Some(Race::Human) => level_modifier,
            None => unreachable!("type safety ensures that `race` is initialized"),
        };

        PlayerBuilder {
            race: self.race,
            level: Some(level),
            items: self.items,
        }
    }

    #[require(LevelSet)]
    #[switch_to(ItemsSet)]
    fn set_items(self, items: Vec<&'a T>) -> PlayerBuilder<'a, T> {
        PlayerBuilder {
            race: self.race,
            level: self.level,
            items: Some(items),
        }
    }

    #[require(A)]
    fn say_hi(self) -> Self {
        println!("Hi!");

        self
    }

    #[require(ItemsSet)]
    fn build(self) -> Player<'a, T> {
        Player {
            race: self.race.expect("type safety ensures this is set"),
            level: self.level.expect("type safety ensures this is set"),
            items: self.items.expect("type safety ensures this is set"),
        }
    }
}

impl<'a, T> PlayerBuilder<'a, T> {
    fn my_weird_method(&self) -> Self {
        use std::marker::PhantomData;

        Self {
            race: Some(Race::Human),
            level: self.level,
            items: self.items.clone(),
            _state: (PhantomData),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_player_creation_works() {
        let items = vec![&"Sword", &"Shield"];
        let player = PlayerBuilder::new()
            .set_race(Race::Human)
            .set_level(10)
            .set_items(items)
            .say_hi()
            .build();

        assert_eq!(player.race, Race::Human);
        assert_eq!(player.level, 10);
        assert_eq!(player.items, vec![&"Sword", &"Shield"]);
    }

    #[test]
    fn method_outside_of_macro_works() {
        let player = PlayerBuilder::new();
        let another_player = PlayerBuilder::my_weird_method(&player);

        assert_eq!(player.level, another_player.level);
        assert_eq!(player.items, another_player.items);
    }
}
