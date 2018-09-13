extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn fail() {
    assert_eq!(1, 2);
}
