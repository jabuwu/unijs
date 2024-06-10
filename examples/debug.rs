use tracing::{info, Level};
use unijs::Module;

#[tokasm::main]
async fn main() {
    unilog::init(Level::INFO, "");
    unijs::init();

    let js = r#"
        exports.values = function() {
            return [
                undefined,
                null,
                true,
                1234.0,
                "hello",
                [1,2,3],
                { foo: "bar" },
                () => {},
            ];
        }
    "#;

    let (mut scope, exports) = Module::load(&js);


    let values = exports.get(&mut scope, "values").into_function().unwrap();
    let array = values.call(&mut scope, &[]).into_array().unwrap();
    for i in 0..array.length(&mut scope) {
        info!("{:?}", array.get(&mut scope, i));
    }
}

