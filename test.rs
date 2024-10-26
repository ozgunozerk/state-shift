#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use state_shift::{states, type_state};
struct Player {
    race: Race,
    level: u8,
    skill_slots: u8,
}
#[automatically_derived]
impl ::core::fmt::Debug for Player {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "Player",
            "race",
            &self.race,
            "level",
            &self.level,
            "skill_slots",
            &&self.skill_slots,
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
struct PlayerBuilder<PlayerBuilderState1 = PlayerBuilderInitial>
where
    PlayerBuilderState1: SealerPlayerBuilder,
{
    race: Option<Race>,
    level: Option<u8>,
    skill_slots: Option<u8>,
    _state: (::std::marker::PhantomData<fn() -> PlayerBuilderState1>),
}
mod sealed_player_builder {
    pub trait Sealed {}
}
pub trait SealerPlayerBuilder: sealed_player_builder::Sealed {}
pub struct PlayerBuilderInitial;
pub struct PlayerBuilderRaceSet;
pub struct PlayerBuilderLevelSet;
pub struct PlayerBuilderSkillSlotsSet;
impl sealed_player_builder::Sealed for PlayerBuilderInitial {}
impl sealed_player_builder::Sealed for PlayerBuilderRaceSet {}
impl sealed_player_builder::Sealed for PlayerBuilderLevelSet {}
impl sealed_player_builder::Sealed for PlayerBuilderSkillSlotsSet {}
impl SealerPlayerBuilder for PlayerBuilderInitial {}
impl SealerPlayerBuilder for PlayerBuilderRaceSet {}
impl SealerPlayerBuilder for PlayerBuilderLevelSet {}
impl SealerPlayerBuilder for PlayerBuilderSkillSlotsSet {}
impl PlayerBuilder<PlayerBuilderInitial> {
    fn new() -> PlayerBuilder {
        PlayerBuilder {
            race: None,
            level: None,
            skill_slots: None,
            _state: ::std::marker::PhantomData,
        }
    }
}
impl PlayerBuilder<PlayerBuilderInitial> {
    fn set_race(self, race: Race) -> PlayerBuilder {
        PlayerBuilder {
            race: Some(race),
            level: self.level,
            skill_slots: self.skill_slots,
            _state: ::std::marker::PhantomData,
        }
    }
}
impl PlayerBuilder<PlayerBuilderRaceSet> {
    fn set_level(self, level_modifier: u8) -> PlayerBuilder {
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
            _state: ::std::marker::PhantomData,
        }
    }
}
impl PlayerBuilder<PlayerBuilderLevelSet> {
    fn set_skill_slots(self, skill_slot_modifier: u8) -> PlayerBuilder {
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
            _state: ::std::marker::PhantomData,
        }
    }
}
impl<A> PlayerBuilder<A>
where
    A: SealerPlayerBuilder,
{
    /// doesn't require any state, so this is available at any state
    fn say_hi(self) -> Self {
        {
            ::std::io::_print(format_args!("Hi!\n"));
        };
        self
    }
}
impl PlayerBuilder<PlayerBuilderSkillSlotsSet> {
    fn build(self) -> Player {
        Player {
            race: self.race.expect("type safety ensures this is set"),
            level: self.level.expect("type safety ensures this is set"),
            skill_slots: self.skill_slots.expect("type safety ensures this is set"),
        }
    }
}
impl PlayerBuilder {
    fn my_weird_method(&self) -> Self {
        use std::marker::PhantomData;
        Self {
            race: Some(Race::Human),
            level: self.level,
            skill_slots: self.skill_slots,
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
            source_file: "tests/simple_example.rs",
            start_line: 115usize,
            start_col: 8usize,
            end_line: 115usize,
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
        let player = PlayerBuilder::new()
            .set_race(Race::Human)
            .set_level(10)
            .set_skill_slots(10)
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
        match (&player.skill_slots, &11) {
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
            source_file: "tests/simple_example.rs",
            start_line: 129usize,
            start_col: 8usize,
            end_line: 129usize,
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
        match (&player.skill_slots, &another_player.skill_slots) {
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
