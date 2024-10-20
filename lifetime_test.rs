#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use state_shift::{states, switch_to, type_state};
struct Player<'a, T> {
    race: Race,
    level: u8,
    items: Vec<&'a T>,
}
#[automatically_derived]
impl<'a, T: ::core::fmt::Debug> ::core::fmt::Debug for Player<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "Player",
            "race",
            &self.race,
            "level",
            &self.level,
            "items",
            &&self.items,
        )
    }
}
enum Race {
    #[allow(unused)]
    Orc,
    Human,
}
#[automatically_derived]
impl ::core::fmt::Debug for Race {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                Race::Orc => "Orc",
                Race::Human => "Human",
            },
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Race {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Race {
    #[inline]
    fn eq(&self, other: &Race) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
#[allow(clippy::type_complexity)]
struct PlayerBuilder<'a, T, State1 = Initial>
where
    State1: TypeStateProtector,
{
    race: Option<Race>,
    level: Option<u8>,
    items: Option<Vec<&'a T>>,
    _state: (::std::marker::PhantomData<fn() -> State1>),
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
    fn new() -> Self {
        Self {
            race: None,
            level: None,
            items: None,
        }
    }
}
impl<'a, T> PlayerBuilder<'a, T, Initial> {
    fn set_race(self, race: Race) -> Self<RaceSet> {
        {
            Self {
                race: Some(race),
                level: self.level,
                items: self.items,
            }
        }
    }
}
impl<'a, T> PlayerBuilder<'a, T, RaceSet> {
    fn set_level(self, level_modifier: u8) -> Self<LevelSet> {
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
            Self {
                race: self.race,
                level: Some(level),
                items: self.items,
            }
        }
    }
}
impl<'a, T> PlayerBuilder<'a, T, LevelSet> {
    fn set_items(self, items: Vec<&'a T>) -> Self<ItemsSet> {
        {
            Self {
                race: self.race,
                level: self.level,
                items: Some(items),
            }
        }
    }
}
impl<'a, T, A> PlayerBuilder<'a, T, A>
where
    A: TypeStateProtector,
{
    fn say_hi(self) -> Self {
        {
            ::std::io::_print(format_args!("Hi!\n"));
        };
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
    extern crate test;
    #[cfg(test)]
    #[rustc_test_marker = "tests::simple_player_creation_works"]
    #[doc(hidden)]
    pub const simple_player_creation_works: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::simple_player_creation_works"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/simple_lifetime.rs",
            start_line: 106usize,
            start_col: 8usize,
            end_line: 106usize,
            end_col: 36usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(simple_player_creation_works()),
        ),
    };
    fn simple_player_creation_works() {
        let items = <[_]>::into_vec(
            #[rustc_box]
            ::alloc::boxed::Box::new([&"Sword", &"Shield"]),
        );
        let player = PlayerBuilder::new()
            .set_race(Race::Human)
            .set_level(10)
            .set_items(items)
            .say_hi()
            .build();
        match (&player.race, &Race::Human) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        match (&player.level, &10) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        match (
            &player.items,
            &<[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([&"Sword", &"Shield"]),
            ),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    extern crate test;
    #[cfg(test)]
    #[rustc_test_marker = "tests::method_outside_of_macro_works"]
    #[doc(hidden)]
    pub const method_outside_of_macro_works: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::method_outside_of_macro_works"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            source_file: "tests/simple_lifetime.rs",
            start_line: 121usize,
            start_col: 8usize,
            end_line: 121usize,
            end_col: 37usize,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result(method_outside_of_macro_works()),
        ),
    };
    fn method_outside_of_macro_works() {
        let player = PlayerBuilder::new();
        let another_player = PlayerBuilder::my_weird_method(&player);
        match (&player.level, &another_player.level) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        match (&player.items, &another_player.items) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
}
#[rustc_main]
#[coverage(off)]
#[doc(hidden)]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[
        &method_outside_of_macro_works,
        &simple_player_creation_works,
    ])
}
