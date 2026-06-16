# SDK Architecture

The `rocket-sdk` is built upon a robust, compile-time enforced architecture. This document outlines the core patterns and abstractions used within the SDK, primarily focusing on the Zero-Cost Typestate Kernel, the `Machine<Law, Phase>` pattern, and the trait-based `Builder` API.

## Zero-Cost Typestate Kernel

The foundation of the `rocket-sdk` is the Zero-Cost Typestate Kernel. This pattern leverages Rust's type system to enforce valid state transitions at compile time without incurring runtime overhead. By representing states as distinct types, we ensure that invalid operations are caught early during compilation.

## Machine<Law, Phase>

The core abstraction of our Typestate Kernel is the `Machine<Law, Phase>` struct. 

```rust
pub struct Machine<L, P> {
    _law: std::marker::PhantomData<L>,
    pub phase: P,
}
```

### Concepts

1. **Law (`L`)**: A marker trait that represents the domain rules or semantic laws governing the machine's behavior. Different laws can dictate different validation rules or transition behaviors for the same underlying data.
2. **Phase (`P`)**: Represents the current operational state (typestate) of the machine. Typical phases include `Input`, `Validated`, and `Admitted`.

### State Transitions

Transitions between phases consume the machine, preventing the reuse of old states (avoiding invalid aliasing) and moving the data into a new phase.

```rust
// Example phases
pub struct Input { pub raw_data: String }
pub struct Validated { pub parsed_data: MyData }
pub struct Admitted { pub secure_data: MyData }

// Law definition
pub trait MyLaw {
    fn validate(input: &Input) -> Result<Validated, Error>;
}

// Transition implementation
impl<L: MyLaw> Machine<L, Input> {
    pub fn validate(self) -> Result<Machine<L, Validated>, Error> {
        let validated_phase = L::validate(&self.phase)?;
        Ok(Machine {
            _law: std::marker::PhantomData,
            phase: validated_phase,
        })
    }
}
```

This strict enforcement ensures that data must be validated before it can be admitted, and the Rust compiler guarantees this order of operations.

## Trait-based Builder API

To provide a developer-friendly interface over the Typestate Kernel, `rocket-sdk` employs a trait-based `Builder` API. This API abstracts the raw typestate transitions into an intuitive, fluent builder pattern.

```rust
pub trait Builder {
    type Output;
    fn build(self) -> Result<Self::Output, Error>;
}
```

The Builder API typically wraps the underlying `Machine` and provides semantic methods for configuration before finally calling `.build()`, which internally drives the machine through its required phases (e.g., from `Input` to `Admitted`).

### Example Usage

```rust
let instance = MyBuilder::new()
    .with_config("value")
    .build()?; // Triggers the typestate transitions safely
```

This approach hides the complexity of the Typestate Kernel while maintaining all its compile-time safety guarantees.
