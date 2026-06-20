# How to Annotate Components with Auto-Binding Tags

This guide provides step-by-step instructions on how to register custom Rust, C++, or C game components with the Unify AutoML Discovery Registry. 

By annotating your source code with `@UnifyAutoBind` comments or `#[derive(AutoBind)]` macros, you make them discoverable to the automated optimization, testing, and Model Context Protocol (MCP) ecosystems without requiring manual config files.

---

## Method 1: Comment Annotations (Rust, C++, C)

Comment annotations are the most flexible method because they work across programming languages (Rust, C++, and C headers) and do not affect compilation. 

The Unify AutoML scanner searches for comments containing the `@UnifyAutoBind` token. The scanner extracts the name following the token.

### Syntax Rules
The scanner supports three styles of comment annotations:
1. **Parenthesis syntax**: `@UnifyAutoBind(ComponentName)`
2. **Colon syntax**: `@UnifyAutoBind: ComponentName`
3. **Space syntax**: `@UnifyAutoBind ComponentName`

### Example in C++ (`CombatSystem.h`)
Add a comment directly above your class or struct definition:

```cpp
#ifndef COMBAT_SYSTEM_H
#define COMBAT_SYSTEM_H

// @UnifyAutoBind: CombatSystem
class CombatSystem {
public:
    void resolve_damage(float attack, float defense);
};

#endif // COMBAT_SYSTEM_H
```

### Example in Rust (`quest.rs`)
Add a line comment anywhere in the source file:

```rust
// @UnifyAutoBind: QuestManager

pub struct QuestManager {
    pub active_quests: Vec<String>,
}
```

---

## Method 2: Macro Derivation (Rust Only)

If you are writing Rust, you can use the `AutoBind` derive macro. The scanner detects files containing the macro and automatically registers the component.

### Syntax Rules
The scanner looks for:
- `derive(AutoBind)`
- `derive(unify_automl::AutoBind)`

When the macro is detected, the component name defaults to the **file stem** (the name of the file without the `.rs` extension).

### Example
Add the derive macro to your struct or enum:

```rust
use unify_automl::AutoBind;

#[derive(Debug, Clone, AutoBind)]
pub struct HeroInventory {
    pub slots: Vec<String>,
    pub max_capacity: u32,
}
```

If this struct is defined inside a file named `hero_inventory.rs`, it will be registered under the name `hero_inventory`.

---

## Step-by-Step Validation Guide

Follow these steps to confirm that your newly annotated components are successfully parsed and registered:

### Step 1: Write the Code
Create a file named `dev_env/my_test_component.rs` with the following content:

```rust
// @UnifyAutoBind: CustomHeroSystem
pub struct CustomHeroSystem {
    pub health: u32,
    pub level: u32,
}
```

### Step 2: Run the Discovery Command
Run the `automl discover` command pointing to your development folder:

```bash
cargo run -p unify -- automl discover ./dev_env
```

### Step 3: Inspect the Output JSON
Verify that `CustomHeroSystem` is listed in the `components` array:

```json
{
  "success": true,
  "message": "Successfully discovered components in: ./dev_env",
  "data": {
    "components": [
      {
        "name": "CustomHeroSystem",
        "file_path": "./dev_env/my_test_component.rs",
        "language": "Rust",
        "binding_tag": "@UnifyAutoBind: CustomHeroSystem"
      }
    ],
    "workspace_games": [
      "Infinity Blade 4 MUD (ib4-mud)",
      "Gundam Nexus (nexus-session)"
    ]
  }
}
```

If the component does not appear:
1. Double check that the file extension is `.rs`, `.h`, `.cpp`, or `.hpp`.
2. Ensure the directory path passed to the command is correct.
3. Check that there are no typo errors in `@UnifyAutoBind` (it is case-sensitive).
