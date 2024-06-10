use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use unijs::{Module, Value};

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
}

#[tokasm::main]
async fn main() {
    unilog::init(Level::INFO, "");
    unijs::init();

    let js = r#"
        exports.json = function(person) {
            person.name = "Alice";
            return person;
        }
    "#;
    let (mut scope, exports) = Module::load(&js);
    let json = exports.get(&mut scope, "json").into_function().unwrap();
    let person = Value::serialize(
        &mut scope,
        &Person {
            name: "Bob".to_owned(),
            age: 31,
        },
    )
    .unwrap();
    let result = json.call(&mut scope, &[person]);
    let person = result.deserialize::<Person>(&mut scope);
    info!("{:?}", person);
}
