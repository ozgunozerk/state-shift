#![allow(clippy::needless_lifetimes)]

use state_shift::{impl_state, type_state};

use core::fmt::Debug;

#[derive(Debug)]
struct Player<'a, 'b: 'a, T> {
    race: Race,
    level: u8,
    items: Vec<&'a T>,
    #[allow(unused)]
    passive_items: Vec<&'b T>,
}

#[derive(Debug, PartialEq)]
enum Race {
    #[allow(unused)]
    Orc,
    Human,
}

#[type_state(states = (Initial, RaceSet, LevelSet, ItemsSet), slots = (Initial))]
struct PlayerBuilder<'a, 'b: 'a, T>
where
    T: Debug,
{
    race: Option<Race>,
    level: Option<u8>,
    items: Option<Vec<&'a T>>,
    passive_items: Option<Vec<&'b T>>,
}

#[impl_state]
impl<'a, 'b, T> PlayerBuilder<'a, 'b, T>
where
    T: Debug,
{
    #[require(Initial)]
    fn new() -> PlayerBuilder<'a, 'b, T> {
        PlayerBuilder {
            race: None,
            level: None,
            items: None,
            passive_items: None,
        }
    }

    #[require(Initial)]
    #[switch_to(RaceSet)]
    fn set_race(self, race: Race) -> PlayerBuilder<'a, 'b, T> {
        PlayerBuilder {
            race: Some(race),
            level: self.level,
            items: self.items,
            passive_items: self.passive_items,
        }
    }

    #[require(RaceSet)]
    #[switch_to(LevelSet)]
    fn set_level(self, level_modifier: u8) -> PlayerBuilder<'a, 'b, T> {
        let level = match self.race {
            Some(Race::Orc) => level_modifier + 2,
            Some(Race::Human) => level_modifier,
            None => unreachable!("type safety ensures that `race` is initialized"),
        };

        PlayerBuilder {
            race: self.race,
            level: Some(level),
            items: self.items,
            passive_items: self.passive_items,
        }
    }

    #[require(LevelSet)]
    #[switch_to(ItemsSet)]
    fn set_items(self, items: Vec<&'a T>) -> PlayerBuilder<'a, 'b, T> {
        PlayerBuilder {
            race: self.race,
            level: self.level,
            items: Some(items),
            passive_items: self.passive_items,
        }
    }

    #[require(LevelSet)]
    #[switch_to(ItemsSet)]
    fn set_different_type_items<'c, 'd, Q>(self, items: Vec<&'c Q>) -> PlayerBuilder<'c, 'd, Q>
    where
        Q: Debug,
    {
        PlayerBuilder {
            race: self.race,
            level: self.level,
            items: Some(items),
            passive_items: None,
        }
    }

    #[require(LevelSet)]
    #[switch_to(ItemsSet)]
    fn set_items_might_fail(self, items: Vec<&'a T>) -> Option<PlayerBuilder<'a, 'b, T>> {
        if items.is_empty() {
            return None;
        }

        Some(PlayerBuilder {
            race: self.race,
            level: self.level,
            items: Some(items),
            passive_items: self.passive_items,
        })
    }

    #[require(A)]
    fn say_hi(self) -> Self {
        println!("Hi!");

        self
    }

    #[require(ItemsSet)]
    fn build(self) -> Player<'a, 'b, T> {
        Player {
            race: self.race.expect("type safety ensures this is set"),
            level: self.level.expect("type safety ensures this is set"),
            items: self.items.expect("type safety ensures this is set"),
            passive_items: vec![],
        }
    }
}

impl<'a, 'b, T> PlayerBuilder<'a, 'b, T>
where
    T: Debug,
{
    fn my_weird_method(&self) -> Self {
        use core::marker::PhantomData;

        Self {
            race: Some(Race::Human),
            level: self.level,
            items: self.items.clone(),
            passive_items: self.passive_items.clone(),
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
    fn different_type_items_works() {
        let items = vec![&"Sword", &"Shield"];
        let player = PlayerBuilder::<String>::new()
            .set_race(Race::Human)
            .set_level(10)
            .set_different_type_items(items)
            .say_hi()
            .build();

        assert_eq!(player.race, Race::Human);
        assert_eq!(player.level, 10);
        assert_eq!(player.items, vec![&"Sword", &"Shield"]);
    }

    #[test]
    fn set_items_might_fail_works() {
        let items = vec![&"Sword", &"Shield"];
        let player = PlayerBuilder::new()
            .set_race(Race::Human)
            .set_level(10)
            .set_items_might_fail(items);

        assert!(player.is_some());

        let items = vec![];
        let player = PlayerBuilder::<String>::new()
            .set_race(Race::Human)
            .set_level(10)
            .set_items_might_fail(items);

        assert!(player.is_none());
    }

    #[test]
    fn method_outside_of_macro_works() {
        let player: PlayerBuilder<'_, '_, &str> = PlayerBuilder::new();

        let another_player = PlayerBuilder::my_weird_method(&player);

        assert_eq!(player.level, another_player.level);
        assert_eq!(player.items, another_player.items);
    }
}
