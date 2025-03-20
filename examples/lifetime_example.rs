#![allow(clippy::needless_lifetimes)]

/// Here you can find the `cargo expand` output of the `simple_lifetime.rs` test file.
/// This is a rough `expansion`, ignores some minor details for readability,
/// and does not expand irrelevant parts of the code (e.g. `#[derive(Debug)]`, etc.)
///
/// This file serves the purpose of revealing what's happening behind the curtains.
use core::marker::PhantomData;

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

#[allow(clippy::type_complexity)]
struct PlayerBuilder<'a, T, State1 = Initial>
where
    State1: TypeStateProtector,
{
    race: Option<Race>,
    level: Option<u8>,
    items: Option<Vec<&'a T>>,
    _state: PhantomData<fn() -> State1>,
}
mod sealed {
    pub trait Sealed {}
}
pub trait TypeStateProtector: sealed::Sealed {}
struct Initial;
struct RaceSet;
struct LevelSet;
struct ItemsSet;

impl sealed::Sealed for Initial {}
impl sealed::Sealed for RaceSet {}
impl sealed::Sealed for LevelSet {}
impl sealed::Sealed for ItemsSet {}

impl TypeStateProtector for Initial {}
impl TypeStateProtector for RaceSet {}
impl TypeStateProtector for LevelSet {}
impl TypeStateProtector for ItemsSet {}

impl<'a, T> PlayerBuilder<'a, T, Initial> {
    fn new() -> PlayerBuilder<'a, T> {
        PlayerBuilder {
            race: None,
            level: None,
            items: None,
            _state: PhantomData,
        }
    }
}

impl<'a, T> PlayerBuilder<'a, T, Initial> {
    fn set_race(self, race: Race) -> PlayerBuilder<'a, T, RaceSet> {
        {
            PlayerBuilder {
                race: Some(race),
                level: self.level,
                items: self.items,
                _state: PhantomData,
            }
        }
    }
}

impl<'a, T> PlayerBuilder<'a, T, RaceSet> {
    fn set_level(self, level_modifier: u8) -> PlayerBuilder<'a, T, LevelSet> {
        {
            let level = match self.race {
                Some(Race::Orc) => level_modifier + 2,
                Some(Race::Human) => level_modifier,
                None => unreachable!("type safety ensures that `race` is initialized"),
            };

            PlayerBuilder {
                race: self.race,
                level: Some(level),
                items: self.items,
                _state: PhantomData,
            }
        }
    }
}
impl<'a, T> PlayerBuilder<'a, T, LevelSet> {
    fn set_items(self, items: Vec<&'a T>) -> PlayerBuilder<'a, T, ItemsSet> {
        {
            PlayerBuilder {
                race: self.race,
                level: self.level,
                items: Some(items),
                _state: PhantomData,
            }
        }
    }
}

impl<'a, T, A> PlayerBuilder<'a, T, A>
where
    A: TypeStateProtector,
{
    fn say_hi(self) -> Self {
        println!("Hi!");

        self
    }
}
impl<'a, T> PlayerBuilder<'a, T, ItemsSet> {
    fn build(self) -> Player<'a, T> {
        Player {
            race: self.race.expect("type safety ensures this is set"),
            level: self.level.expect("type safety ensures this is set"),
            items: self.items.expect("type safety ensures this is set"),
        }
    }
}

fn main() {
    let player = PlayerBuilder::new()
        .set_race(Race::Human)
        .set_level(1)
        .set_items(vec![&"frostmourne"])
        .say_hi()
        .build();

    println!("Race: {:?}", player.race);
    println!("Level: {}", player.level);
    println!("Items: {:?}", player.items);
}
