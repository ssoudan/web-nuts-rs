//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

// wasm_bindgen_test_configure!(run_in_worker);

#[wasm_bindgen_test]
fn pass() {
    // let trace = web_nuts::greet_node();

    // assert_eq!(trace, "");
}
