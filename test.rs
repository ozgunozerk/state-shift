#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod example {
    use state_shift::{state_impl, type_state};
    use std::marker::PhantomData;
    use std::mem::MaybeUninit;
    pub struct MyParentObject<'base> {
        #[allow(unused)]
        inner: MaybeUninit<*mut core::ffi::c_void>,
        _marker: PhantomData<&'base ()>,
    }
    impl<'base> MyParentObject<'base> {
        pub fn new() -> Self {
            MyParentObject {
                inner: MaybeUninit::zeroed(),
                _marker: PhantomData,
            }
        }
        pub fn method(&'base self) -> MethodBuilder {
            MethodBuilder::new()
        }
    }
    pub struct Method {}
    impl Method {
        pub fn start(self) -> Result<(), String> {
            Ok(())
        }
    }
    mod sealed_method_builder {
        pub trait Sealed {}
    }
    pub trait SealerMethodBuilder: sealed_method_builder::Sealed {}
    pub struct MethodBuilderUnset;
    pub struct MethodBuilderASet;
    pub struct MethodBuilderBSet;
    pub struct MethodBuilderAOrBSet;
    impl sealed_method_builder::Sealed for MethodBuilderUnset {}
    impl sealed_method_builder::Sealed for MethodBuilderASet {}
    impl sealed_method_builder::Sealed for MethodBuilderBSet {}
    impl sealed_method_builder::Sealed for MethodBuilderAOrBSet {}
    impl SealerMethodBuilder for MethodBuilderUnset {}
    impl SealerMethodBuilder for MethodBuilderASet {}
    impl SealerMethodBuilder for MethodBuilderBSet {}
    impl SealerMethodBuilder for MethodBuilderAOrBSet {}
    #[allow(clippy::type_complexity)]
    pub struct MethodBuilder<MethodBuilderState1 = MethodBuilderUnset>
    where
        MethodBuilderState1: SealerMethodBuilder,
    {
        #[allow(unused)]
        slot_a: Option<u8>,
        slot_b: Option<u8>,
        _state: (::std::marker::PhantomData<fn() -> MethodBuilderState1>),
    }
    impl MethodBuilder {}
    impl MethodBuilder<MethodBuilderUnset, MethodBuilderUnset, MethodBuilderUnset> {
        pub fn new() -> MethodBuilder {
            MethodBuilder {
                slot_a: None,
                slot_b: None,
                _state: (
                    ::std::marker::PhantomData,
                    ::std::marker::PhantomData,
                    ::std::marker::PhantomData,
                ),
            }
        }
    }
    impl<B, C> MethodBuilder<MethodBuilderUnset, B, C>
    where
        B: SealerMethodBuilder,
        C: SealerMethodBuilder,
    {
        pub fn set_slot_a(
            self,
            slot_a: u8,
        ) -> MethodBuilder<MethodBuilderASet, B, MethodBuilderAOrBSet> {
            MethodBuilder {
                slot_a: Some(slot_a),
                slot_b: self.slot_b,
                _state: (
                    ::std::marker::PhantomData,
                    ::std::marker::PhantomData,
                    ::std::marker::PhantomData,
                ),
            }
        }
    }
    impl<A, C> MethodBuilder<A, MethodBuilderBSet, C>
    where
        A: SealerMethodBuilder,
        C: SealerMethodBuilder,
    {
        pub fn set_slot_b(
            self,
            slot_b: u8,
        ) -> MethodBuilder<A, MethodBuilderBSet, MethodBuilderAOrBSet> {
            MethodBuilder {
                slot_a: self.slot_a,
                slot_b: Some(slot_b),
                _state: (
                    ::std::marker::PhantomData,
                    ::std::marker::PhantomData,
                    ::std::marker::PhantomData,
                ),
            }
        }
    }
    impl<A, B> MethodBuilder<A, B, MethodBuilderAOrBSet>
    where
        A: SealerMethodBuilder,
        B: SealerMethodBuilder,
    {
        pub fn build(self) -> Method {
            Method {}
        }
    }
    impl<A, B> MethodBuilder<A, B, MethodBuilderAOrBSet>
    where
        A: SealerMethodBuilder,
        B: SealerMethodBuilder,
    {
        pub fn start(self) -> Result<(), String> {
            Ok(())
        }
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "test_method_builder"]
#[doc(hidden)]
pub const test_method_builder: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_method_builder"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/visibility_example.rs",
        start_line: 86usize,
        start_col: 4usize,
        end_line: 86usize,
        end_col: 23usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_method_builder()),
    ),
};
fn test_method_builder() {
    let myparentobj = example::MyParentObject::new();
    let meth = myparentobj.method().set_slot_a(42).build();
    let res = meth.start();
    if !res.is_ok() {
        ::core::panicking::panic("assertion failed: res.is_ok()")
    }
}
#[rustc_main]
#[coverage(off)]
#[doc(hidden)]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&test_method_builder])
}
