use std::sync::Arc;
use make_public_macro::publics;

// 1. Testing a standard Struct (private field turns public, gets generic TryFrom)
#[derive(Debug, Clone)]
#[publics]
struct Example {
    first: String, // was implicitly private
    pub second: u32,
}

// 2. Testing an Unnamed Tuple Struct (elements turn public, gets generic TryFrom)
#[publics]
#[derive(Debug, Clone)]
struct ExampleTuple(String, f64);

// 3. Testing an Enum (all variant payloads receive precise From mappings)
#[allow(dead_code)]
#[derive(Debug, Clone)]
#[publics]
enum Pet {
    Cat(String, u32),
    Dog { name: String, age: u32 },
    Fish, // Unit variant ignored gracefully
}

fn main() {
    println!("--- Testing Named Struct with foreign types ---");
    // Coercion from (&str, i32) -> (String, u32) works seamlessly!
    let raw_tuple = ("Hello World", 42);
    let example = Example::try_from(raw_tuple).unwrap();
    
    // Testing visibility mutation: 'first' is accessible outside its crate container
    println!("Extracted struct fields -> first: '{}', second: {}\n", example.first, example.second);

    println!("--- Testing Arc integration + Deref Coercion ---");
    let shared_example = Arc::new(Example::try_from(("Arc String", 100)).unwrap());
    println!("Direct dot access over Arc pointer -> second: {}\n", shared_example.second);

    println!("--- Testing Tuple Struct conversion ---");
    // Coercion from (&str, f32) -> (String, f64)
    let tuple_example = ExampleTuple::try_from(("Tuple Element", 3.14f32)).unwrap();
    println!("Tuple Struct positional verification -> .0: '{}', .1: {}\n", tuple_example.0, tuple_example.1);

    println!("--- Testing Enum Variant matching with unique Payloads ---");
    // MODIFIED: Variants are now populated via their dedicated payload wrapper structs
    // to bypass the conflicting From layout limitations.
    let cat_pet = Pet::from(PetCatPayload("Whiskers".to_string(), 3u32));
    let dog_pet = Pet::from(PetDogPayload("Rover".to_string(), 5u32));

    println!("Enum constructed successfully -> Cat: {:?}", cat_pet);
    println!("Enum constructed successfully -> Dog: {:?}", dog_pet);
}
