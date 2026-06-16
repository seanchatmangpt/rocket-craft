# blueprint-macros

Procedural macros for writing UE4 Blueprint graphs with a Rust-like DSL.

The macros expand at compile time to `blueprint_core::BlueprintBuilder` call
chains and return a T3D-format `String`.

## Workspace setup

Add both crates to your `Cargo.toml`:

```toml
[dependencies]
blueprint-core = { path = "../blueprint-core" }
blueprint-macros = { path = "../blueprint-macros" }
```

## `blueprint!` macro

### Minimal example

```rust
use blueprint_macros::blueprint;

let t3d: String = blueprint! {
    name: "MyActor",
    parent: "Actor",

    on begin_play {
        print("Hello from Blueprint-RS!");
    }
};

println!("{}", t3d);
```

### Full syntax reference

```
blueprint! {
    // Required: Blueprint class name
    name: "BlueprintName",

    // Optional: parent class (default "Actor")
    parent: "ParentClass",

    // Variables  (optional, repeat as needed, separated by commas)
    var health: i32 = 100,
    var speed: f32 = 600.0,
    var is_alive: bool = true,
    var label: String = "Hero",
    var tag: Name,              // no default

    // Event handlers (optional, repeat as needed)
    on begin_play {
        // statements …
    }

    on end_play {
        // statements …
    }

    on tick(delta: float) {
        // statements …
    }

    on custom("MyCustomEvent") {
        // statements …
    }
}
```

### Variable types

| Syntax              | UE4 type       |
|---------------------|----------------|
| `i32`, `int`        | IntProperty    |
| `f32`, `float`      | FloatProperty  |
| `bool`, `boolean`   | BoolProperty   |
| `String`, `string`  | StrProperty    |
| `Name`, `name`      | NameProperty   |

### Statements inside event bodies

| Statement                                      | Expands to                             |
|------------------------------------------------|----------------------------------------|
| `print("message")`                             | K2Node_CallFunction → PrintString      |
| `print_string("message")`                      | same as `print`                        |
| `call func_name("arg1", "arg2")`               | K2Node_CallFunction                    |
| `set var_name = value`                         | K2Node_VariableSet                     |
| `get var_name`                                 | K2Node_VariableGet                     |
| `branch condition { true => …, false => … }`   | K2Node_IfThenElse (Branch)             |
| `for i in 0..10 { … }`                        | K2Node_MacroInstance (ForLoop)         |

### Complex example

```rust
use blueprint_macros::blueprint;

let t3d = blueprint! {
    name: "MyCharacter",
    parent: "Character",

    var health: i32 = 100,
    var speed: f32 = 600.0,
    var is_alive: bool = true,

    on begin_play {
        print("Game started!");
        call setup_character();
        set health = 100;
    }

    on tick(delta: float) {
        call update_movement("DeltaSeconds");
    }

    on custom("OnDamaged") {
        set health = 0;
        set is_alive = false;
        branch is_alive {
            true  => print("Still alive"),
            false => print("Dead"),
        }
    }
};
```

## `bp_node!` macro

Creates the T3D text for a single Blueprint graph node.

```rust
use blueprint_macros::bp_node;

let node: String = bp_node! {
    class: "K2Node_Event",
    name: "Event_BeginPlay",
    props: {
        EventReference: "ReceiveBeginPlay",
        bOverrideFunction: "True",
    }
};

println!("{}", node);
// Begin Object Class=/Script/BlueprintGraph.K2Node_Event
//    Name="Event_BeginPlay"
//    EventReference="ReceiveBeginPlay"
//    bOverrideFunction="True"
// End Object
```

### `bp_node!` fields

| Field     | Required | Description                                       |
|-----------|----------|---------------------------------------------------|
| `class`   | yes      | The short UE4 node class name (no namespace)      |
| `name`    | yes      | The `Name=` property value of the object          |
| `props`   | no       | Key-value pairs written as extra T3D properties   |

## How it works

Both macros are compiled as a `proc-macro` crate.  At compile time `syn` parses
the custom DSL grammar and `quote!` emits Rust code that calls
`blueprint_core::BlueprintBuilder` (or builds a raw T3D string for `bp_node!`).
No runtime parsing is required — the DSL is validated and translated entirely at
compile time.

### Generated code for `blueprint!`

```rust
// blueprint! { name: "Foo", parent: "Actor", on begin_play { print("Hi"); } }
// expands to roughly:
{
    ::blueprint_core::BlueprintBuilder::new("Foo", "Actor")
        .begin_play(|__ev| {
            __ev.print("Hi");
        })
        .to_t3d()
}
```
