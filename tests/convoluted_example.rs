mod example {
    use std::marker::PhantomData;
    use std::mem::MaybeUninit;

    use state_shift::{states, switch_to, type_state};

    pub struct MyParentObject<'base> {
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

        // ...

        pub fn method(&'base self) -> MethodBuilder {
            MethodBuilder::new()
        }
    }

    struct Method {}

    impl Method {
        fn start(self) -> Result<(), String> {
            Ok(())
        }
    }

    #[type_state(state_slots = 3, default_state = Unset)]
    pub struct MethodBuilder {
        slot_a: Option<u8>,
        slot_b: Option<u8>,
    }

    impl MethodBuilder {}

    #[states(Unset, ASet, BSet, AOrBSet)]
    impl MethodBuilder {
        #[require(Unset, Unset, Unset)] // require the default state for the constructor
        pub fn new() -> MethodBuilder {
            MethodBuilder {
                slot_a: None,
                slot_b: None,
            }
        }

        #[require(Unset, B, C)]
        #[switch_to(ASet, B, AOrBSet)]
        pub fn set_slot_a(self, slot_a: u8) -> MethodBuilder {
            MethodBuilder {
                slot_a: Some(slot_a),
                slot_b: self.slot_b,
            }
        }

        #[require(A, BSet, C)]
        #[switch_to(A, BSet, AOrBSet)]
        pub fn set_slot_b(self, slot_b: u8) -> MethodBuilder {
            MethodBuilder {
                slot_a: self.slot_a,
                slot_b: Some(slot_b),
            }
        }

        #[require(A, B, AOrBSet)]
        pub fn build(self) -> Method {
            Method {}
        }

        #[require(A, B, AOrBSet)]
        pub fn start(self) -> Result<(), String> {
            Ok(())
        }
    }
}

fn main() {
    let myparentobj = example::MyParentObject::new();

    // using the clean builder pattern within the parent
    let meth = myparentobj.method().set_slot_a(42).build();
    let res = meth.start();

    // using the builder pattern internally but starting the method immediately
    // No need for the separate Method type
    let res2 = myparentobj.method().set_slot_a(42).start();

    assert!(res == res2)
}
