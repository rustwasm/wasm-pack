extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    fn alert(message: &str);
}

#[wasm_bindgen]
pub fn say_hello() {
    alert("Hello from wasm");
}
