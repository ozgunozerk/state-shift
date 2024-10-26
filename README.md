# With and Without State-Shift:

state-shift let's you convert your structs and methods into type-state version, without the ugly code.

If `type-state-pattern` didn't sound familiar, scroll to [What the hell is even Type-State-Pattern?](#what-the-hell-is-even-type-state-pattern)

Say, you want to build a player, and some fields need to be set before the others. In short, a classic type-state-pattern example...

> [!WARNING]
> Below are the comparison codes with and without state-shift. If you don't like reading huge chunks of code like me, scroll a bit down to see the [chunk by chunk comparison](#lets-break-it-down)

## Full Code Comparison:

> [!CAUTION]
> A simple Type-State `PlayerBuilder` example WITHOUT state-shift:

```rust
use std::marker::PhantomData;

struct PlayerBuilder<State1 = Initial> {
    race: Option<Race>,
    level: Option<u8>,
    skill_slots: Option<u8>,
    _state: PhantomData<fn() -> State1>,
}

mod sealed {
    pub trait Sealed {}
}

pub trait TypeStateProtector: sealed::Sealed {}

struct Initial;
struct RaceSet;
struct LevelSet;
struct SkillSlotsSet;

impl sealed::Sealed for Initial {}
impl sealed::Sealed for RaceSet {}
impl sealed::Sealed for LevelSet {}
impl sealed::Sealed for SkillSlotsSet {}

impl TypeStateProtector for Initial {}
impl TypeStateProtector for RaceSet {}
impl TypeStateProtector for LevelSet {}
impl TypeStateProtector for SkillSlotsSet {}


impl PlayerBuilder<Initial> {
    fn new() -> Self {
        PlayerBuilder {
            race: None,
            level: None,
            skill_slots: None,
            _state: (PhantomData),
        }
    }


    fn set_race(self, race: Race) -> PlayerBuilder<RaceSet> {
        PlayerBuilder {
            race: Some(race),
            level: self.level,
            skill_slots: self.skill_slots,
            _state: (PhantomData),
        }
    }
}

impl PlayerBuilder<RaceSet> {
    fn set_level(self, level_modifier: u8) -> PlayerBuilder<LevelSet> {
        let level = match self.race {
            Some(Race::Orc) => level_modifier + 2, // Orc's have +2 level advantage
            Some(Race::Human) => level_modifier,   // humans are weak
            None => unreachable!("type safety ensures that `race` is initialized"),
        };

        PlayerBuilder {
            race: self.race,
            level: Some(level),
            skill_slots: self.skill_slots,
            _state: (PhantomData),
        }
    }
}

impl PlayerBuilder<LevelSet> {
    fn set_skill_slots(self, skill_slot_modifier: u8) -> PlayerBuilder<SkillSlotsSet> {
        let skill_slots = match self.race {
            Some(Race::Orc) => skill_slot_modifier,
            Some(Race::Human) => skill_slot_modifier + 1, // Human's have +1 skill slot advantage
            None => unreachable!("type safety ensures that `race` should be initialized"),
        };

        PlayerBuilder {
            race: self.race,
            level: self.level,
            skill_slots: Some(skill_slots),
            _state: (PhantomData),
        }
    }
}

impl<A> PlayerBuilder<A>
where
    A: TypeStateProtector,
{
    fn say_hi(self) -> Self {
        println!("Hi!");
        self
    }
}

impl PlayerBuilder<SkillSlotsSet> {
    fn build(self) -> Player {
        Player {
            race: self.race.expect("type safety ensures this is set"),
            level: self.level.expect("type safety ensures this is set"),
            skill_slots: self.skill_slots.expect("type safety ensures this is set"),
        }
    }
}

#[derive(Debug)]
struct Player {
    race: Race,
    level: u8,
    skill_slots: u8,
}

#[derive(Debug, PartialEq)]
enum Race {
    Orc,
    #[allow(unused)]
    Human,
}

fn main() {
    let player = PlayerBuilder::new()
        .set_race(Race::Orc)
        .set_level(1)
        .set_skill_slots(1)
        .say_hi()
        .build();
    println!("Race: {:?}", player.race);
    println!("Level: {}", player.level);
    println!("Skill slots: {}", player.skill_slots);
}
```

<br>
<br>

---

<br>
<br>

> [!TIP]
> A simple Type-State `PlayerBuilder` example WITH state-shift:

```rust
use state_shift::{states, switch_to, type_state};


#[type_state(state_slots = 1, default_state = Initial)]
struct PlayerBuilder {
    race: Option<Race>,
    level: Option<u8>,
    skill_slots: Option<u8>,
}

#[states(Initial, RaceSet, LevelSet, SkillSlotsSet)]
impl PlayerBuilder {
    #[require(Initial)] // require the default state for the constructor
    fn new() -> PlayerBuilder {
        PlayerBuilder {
            race: None,
            level: None,
            skill_slots: None,
        }
    }

    #[require(Initial)] // can be called only at `Initial` state.
    #[switch_to(RaceSet)] // Transitions to `RaceSet` state
    fn set_race(self, race: Race) -> PlayerBuilder {
        PlayerBuilder {
            race: Some(race),
            level: self.level,
            skill_slots: self.skill_slots,
        }
    }

    #[require(RaceSet)]
    #[switch_to(LevelSet)]
    fn set_level(self, level_modifier: u8) -> PlayerBuilder {
        let level = match self.race {
            Some(Race::Orc) => level_modifier + 2, // Orc's have +2 level advantage
            Some(Race::Human) => level_modifier,   // humans are weak
            None => unreachable!("type safety ensures that `race` is initialized"),
        };

        PlayerBuilder {
            race: self.race,
            level: Some(level),
            skill_slots: self.skill_slots,
        }
    }

    #[require(LevelSet)]
    #[switch_to(SkillSlotsSet)]
    fn set_skill_slots(self, skill_slot_modifier: u8) -> PlayerBuilder {
        let skill_slots = match self.race {
            Some(Race::Orc) => skill_slot_modifier,
            Some(Race::Human) => skill_slot_modifier + 1, // Human's have +1 skill slot advantage
            None => unreachable!("type safety ensures that `race` should be initialized"),
        };

        PlayerBuilder {
            race: self.race,
            level: self.level,
            skill_slots: Some(skill_slots),
        }
    }

    /// doesn't require any state, so this is available at any state
    #[require(A)]
    fn say_hi(self) -> Self {
        println!("Hi!");
        self
    }

    #[require(SkillSlotsSet)]
    fn build(self) -> Player {
        Player {
            race: self.race.expect("type safety ensures this is set"),
            level: self.level.expect("type safety ensures this is set"),
            skill_slots: self.skill_slots.expect("type safety ensures this is set"),
        }
    }
}

#[derive(Debug)]
struct Player {
    race: Race,
    level: u8,
    skill_slots: u8,
}

#[derive(Debug, PartialEq)]
enum Race {
    #[allow(unused)]
    Orc,
    Human,
}

fn main() {
    let player = PlayerBuilder::new()
        .set_race(Race::Orc)
        .set_level(1)
        .set_skill_slots(1)
        .say_hi()
        .build();
    println!("Race: {:?}", player.race);
    println!("Level: {}", player.level);
    println!("Skill slots: {}", player.skill_slots);
}
```

## Let's break it down:

Consuming huge chunks of code may be overwhelming, so let's break it down.

> [!NOTE]
> Also, let's assume that you want to track multiple states simultaneously for your struct


### 1. Hiding the ugly and unreadable boilerplate code required for your structs:
<br/>

- without this library, you probably have to write something like this (BAD):
    ```rust
    struct PlayerBuilder<State1 = Initial, State2 = Initial, State3 = Initial>
    where
        State1: TypeStateProtector,
        State2: TypeStateProtector,
        State3: TypeStateProtector,
    {
        race: Option<Race>,
        level: Option<u8>,
        skill_slots: Option<u8>,
        spell_slots: Option<u8>,
        _state: (
            PhantomData<State1>,
            PhantomData<State2>,
            PhantomData<State3>,
        ),
    }
    ```

> [!CAUTION]
> The above code might suck the enjoyment out of writing Rust code.

<br/>
<br/>

- with this library, you can write this (GOOD):
    ```rust
    #[type_state(state_slots = 3, default_state = Initial)]
    struct PlayerBuilder {
        race: Option<Race>,
        level: Option<u8>,
        skill_slots: Option<u8>,
        spell_slots: Option<u8>,
    }
    ```

> [!TIP]
> Mmmhh! Much better, right?

<br/>

### 2. Hiding the  ugly and unreadable boilerplate code required for your impl blocks:

<br/>

- without this library, you probably have to write something like this (BAD):

    ```rust
    impl<B, C> PlayerBuilder<Initial, B, C>
    where
        B: TypeStateProtector,
        C: TypeStateProtector,
    {
        fn set_race(self, race: Race) -> PlayerBuilder<RaceSet, B, C> {
            {
                {
                    PlayerBuilder {
                        race: Some(race),
                        level: self.level,
                        skill_slots: self.skill_slots,
                        spell_slots: self.spell_slots,
                        _state: (PhantomData, PhantomData, PhantomData),
                    }
                }
            }
        }
    }
    ```

> [!CAUTION]
> It's not immediately obvious what's going on here, which state is required, to which state it's transitioning into, etc.

<br/>
<br/>

- with this library, you can write this (GOOD):
    ```rust
    #[require(Initial, B, C)]
    #[switch_to(RaceSet, B, C)]
    fn set_race(self, race: Race) -> PlayerBuilder {
        PlayerBuilder {
            race: Some(race),
            level: self.level,
            skill_slots: self.skill_slots,
            spell_slots: self.spell_slots,
        }
    }
    ```

> [!TIP]
> Immediately signals:
>
> - which state is required.
>
> - to which state it's transitioning into.
>
> No weird generics and intermediate unit structs that hurting your brain.

<br/>
<br/>

### 3. Hiding the ugly and unreadable boilerplate code required for intermediate traits and structs:

<br/>

- without this library, in order to ensure the type-safety, you have to write traits and unit structs (BAD):
    ```rust
    mod sealed {
        pub trait Sealed {}
    }

    pub trait TypeStateProtector: sealed::Sealed {}

    pub struct Initial;
    pub struct RaceSet;
    pub struct LevelSet;
    pub struct SkillSlotsSet;
    pub struct SpellSlotsSet;

    impl sealed::Sealed for Initial {}
    impl sealed::Sealed for RaceSet {}
    impl sealed::Sealed for LevelSet {}
    impl sealed::Sealed for SkillSlotsSet {}
    impl sealed::Sealed for SpellSlotsSet {}

    impl TypeStateProtector for Initial {}
    impl TypeStateProtector for RaceSet {}
    impl TypeStateProtector for LevelSet {}
    impl TypeStateProtector for SkillSlotsSet {}
    impl TypeStateProtector for SpellSlotsSet {}
    ```

> [!CAUTION]
> EWWWW

<br/>
<br/>


- with this library, you can write this (GOOD):
    ```rust
    #[states(Initial, RaceSet, LevelSet, SkillSlotsSet, SpellSlotsSet)]
    impl PlayerBuilder {
        // ...methods redacted...
    }
    ```

> [!TIP]
> The necessary states that we want to use, cannot be more clear!

<br/>

# Tell me more

I love type-state pattern's promises:

- compile time checks

- better/safer auto completion suggestions by your IDE

- no additional runtime costs

However, I agree that in order to utilize type-state pattern, the code has to become quite ugly. We are talking about less readable and maintainable code, just because of this.

Although I'm a fan, I agree usually it's not a good idea to use type-state pattern.

And THAT, my friends, bothered me...

So I wrote `state-shift`.

TL;DR -> it lets you convert your structs and methods into type-state version, without the ugly code. So, best of both worlds!

If you don't appreciate all the boilerplate code required by Type-State-Pattern that makes the DevX worse, but you still like the idea of type-safety provided by it, this library is for you. `state-shift` lets you write your code as if type-state-pattern was not there, yet grants you the benefits of type-safety.


# What the hell is even Type-State-Pattern?

Here is a great blog post that explains it, I heard that the author is a good person: https://cryptical.xyz/rust/type-state-pattern

TL;DR -> instead of relying on runtime checks, Type-State-Pattern uses type-safety to enforce specific methods are only callable at specific states at compile time.

For example, you cannot call `fight()` method on a `Player` struct when it is in `Dead` state. You normally accomplish this by introducing boolean flags and runtime checks. With Type-State-Pattern, you achieve this without any runtime checks, purely by the type-safety provided by Rust primitives.

This is good, due to:
- better DevX (users of the library won't be even able to call this invalid methods)
- less runtime bugs
- less runtime checks -> more performant code
- zero-cost abstractions for this type checks (no additional performance cost of doing this)

<br/>

## Why you should use care about type-state?

### 1. State-Focused Methods

Let’s say you have a `Player` struct with methods like:

- `die()`
- `resurrect()`

As a reasonable person, you probably don’t want someone to call `die()` on a player who’s already `Dead`.

> [!TIP]
> People cannot die twice!

With this library, you can ensure that your methods respect the logical state transitions, preventing awkward situations like trying to `player.die().die()`;

This library lets you have above mentioned type-safe methods, *WITHOUT*:
- duplicating your structs (one for `Dead` state and one for `Alive` state)
- writing runtime checks
- hurting the performance of your code
- making your code horrible to look at due to infamous Type-State-Pattern

In short, the users of this library won't be able to call:

> [!CAUTION]
> ```rust
> let player = PlayerBuilder::new().die().die(); // ❌ Invalid!
> ```
> The good thing is, after calling the first `die()` method, the second `die()` **won't be even suggested** by your IDE via autocomplete.
>
> And even if you insist to type it anyway, it will be a compile-time error!


### 2. Field/Method Order & Dependencies

Imagine you have a `PlayerBuilder` struct designed to construct a `Player`. Some fields need to be set before others because of logical dependencies. For instance, the `race` field must be specified before the `level` field, as the race affects how we calculate the player's starting level.

> [!CAUTION]
>  So, we don't want the below code:
>```rust
>let player = PlayerBuilder::new().level(10) // ❌ Invalid!
>```

> [!TIP]
>  We want the below code:
>```rust
>let player = PlayerBuilder::new().race(Race::Human).level(10) // ✅
>```

The gist of it is, some fields of the `PlayerBuilder` are depending on other fields. So we want to force the users of this library to set these fields in order by making invalid orders completely unrepresentable at compile time. Even rust-analyzer won't suggest the invalid methods as auto-completion! How wonderful is that!

<br/>
<br/>

# Additional benefits of using this Library

### 1. You get type-safety for your methods, with concise and clear syntax.
The macros do all the heavy lifting for you. You just need to write your code as if type-state-pattern was not there, yet grants you the benefits of type-safety.

<br/>


### 2. Sealed-traits

this library also uses sealed-traits to ensure even more safety! And again, you don't need to worry about anything. Sealed-traits basically ensure that the user cannot implement these trait themselves. So, your structs are super-safe!

<br/>


### 3. Clear documentation

I tried to document nearly everything. If you are curios on what the macros do under the hood, even those macros are documented! Just check the inline documentation and I'm sure you will understand what's going on in a blink of an eye!

<br/>

### 4. Suggestions and contributions are welcome!

I'm a quite friendly guy. Don't hesitate to open an issue or a pull request if you have any suggestions or if you want to contribute! Just keep in mind that everyone contributing to here (including myself) are doing it voluntarily. So, always be respectful and appreciate other's time and effort.


<br/>
<br/>

# Advanced & Helpful Tips

Remember, this library is just hiding the ugly type-state-pattern boilerplate code under the hood. This means, your code still have to obey some rules.

Most of the issues arise from when we are returning the `Self` type. The compiler doesn't like the `Self` keyword in type-state-pattern, because we are actually not returning the `Self`, but a different type. For example, it could be that our method is accepting `Player<Alive>` but we are returning `Player<Dead>`.

And you know how Rust compiler is. It is very strict about types!


## Rules

### 1. If your method is switching states (most probably it does), avoid using `Self` in the return position of the method's signature:

> [!CAUTION]
>
> ```rust
> fn my_method(self) -> Self { // `-> Self` ❌
>    // redacted body
> }

> [!TIP]
> ```rust
> fn my_method(self) -> PlayerBuilder { // `-> ConcreteName` ✅
>    // redacted body
> }


### 2. Similarly, also avoid using `Self` in the method's body:

> [!CAUTION]
>
> ```rust
> fn my_method(self) -> PlayerBuilder {
>
>    Self {  // `Self {}` ❌
>       race: Race::human
>       level: self.level
>    }
> }

> [!TIP]
> ```rust
> fn my_method(self) -> PlayerBuilder {
>
>    PlayerBuilder {  // `PlayerBuilder {}` ✅
>       race: Race::human
>       level: self.level
>    }
> }

### 3. `self` is ok to use, but there is one exception:

> [!CAUTION]
>
> ```rust
> fn my_method(self) -> PlayerBuilder {
>
>    PlayerBuilder {
>       race: Race::human
>       ..self  // `..self` ❌
>    }
> }

> [!NOTE]
> actually having `..self` is not supported by the Rust compiler in this context, YET.
>
> So hoping it will become stable in the future and we won't have to worry about it.

### 4. These macros appends a hidden `_state` field to your struct to make it compatible with type-state-pattern. If you want to opt-out of the macros for god knows why, keep in mind that you need to provide the hidden `_state` field for your methods.

> [!WARNING]
> ```rust
> impl PlayerBuilder {
>     fn my_weird_method(&self) -> Self {
>         Self {
>             race: Some(Race::Human),
>             level: self.level,
>             skill_slots: self.skill_slots,
>            _state: (::std::marker::PhantomData), // Don't forget this!
>         }
>     }
> }

> [!IMPORTANT]
> You only need to worry about `_state` field if you want to opt-out of the macros! So, keep using the macros, and keep yourself stress free 🥂

## Tips

### 1. Tracking multiple states

This feature was both my favorite to implement and the most brain-melting (design-wise and implementation-wise).

**The problem:**

Imagine you have three fields for your struct: `a`, `b`, and `c`. You want `c` to be set only after both `a` and `b` are set. Not just one of them—both.

How do you accomplish this with type-state-pattern? This is a problem because the default design pattern allows you to have a single state to track.

One workaround is to have multiple states for the combinations of `a` and `b`. For example, you can have the following states:
- `a_set_but_b_not_set`
- `b_set_but_a_not_set`
- `a_set_and_b_set`.

This is not a good solution due to 2 reasons:
- it is fucking ugly
- you need to duplicate your methods and give them different names, because you cannot have multiple methods with the same name. If this didn't make sense, take a look at the expanded codes, and you will see why we need to have the same method on different `impl` blocks. The compiler of course doesn't like that. The only workaround to have the same function body on different `impl` blocks, is to have different names for these methods. Same methods, but different names? No more explanation needed on why this is bad.

**The Solution:**

Multiple state slots. By allowing multiple state slots, you can track each state separately, and they won't override each other. You can see this in action in the `tests/complex_example.rs`. It showcases how this is done, and when can it be useful. Now, the macro for our struct should make more sense:

```rust
#[type_state(state_slots = 3, default_state = Initial)]
struct PlayerBuilder {
    race: Option<Race>,
    level: Option<u8>,
    skill_slots: Option<u8>,
    spell_slots: Option<u8>,
}
```

### 2. How do I pass the player to a function (no method), does it require extra type annotations to specify the state?

Say you have this:

```rust
fn player_builder_logger(player_builder: PlayerBuilder) {
    println!("PlayerBuilder's level: {:?}", player_builder.level);
}
```

You can pass the `player_builder` without any type-annotation, but then it would expect the states to be equal to the default ones, in this case: `PlayerBuilder<Initial>`.

If you want to pass another state, I think you have to explicitly tell the code:
```rust
fn player_builder_logger(player_builder: PlayerBuilder<LevelSet>) {
    println!("PlayerBuilder's level: {:?}", player_builder.level);
}
```

Then you can call it like this:
```rust
fn main() {
        let player = PlayerBuilder::new().set_race(Race::Human).set_level(4);
        player_builder_logger(player);
}
```

### 3. Will the generics, lifetimes, and visibility of my methods and structs be preserved?
- yes
- yes
- yes
- yes
- yes

### 4. Can I use `async` or `const` methods?
- YES!

### 5. Can I use `Result<MyStruct>` or `Option<MyStruct>` or similar complex types in my methods?
- you can use them in the return type!
- you can use them in the body!
- basically, yes!

Happy coding!
