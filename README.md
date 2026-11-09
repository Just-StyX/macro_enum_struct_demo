# macro_enum_struct_demo
Make Structs fields public and implementation of TryFrom Trait for Structs and Enums using Procedural Macro

## Execution Steps
```bash
cargo run -q main_app
```

```text
--- Testing Named Struct with foreign types ---
Extracted struct fields -> first: 'Hello World', second: 42

--- Testing Arc integration + Deref Coercion ---
Direct dot access over Arc pointer -> second: 100

--- Testing Tuple Struct conversion ---
Tuple Struct positional verification -> .0: 'Tuple Element', .1: 3.140000104904175

--- Testing Enum Variant matching with unique Payloads ---
Enum constructed successfully -> Cat: Cat("Whiskers", 3)
Enum constructed successfully -> Dog: Dog { name: "Rover", age: 5 }
```