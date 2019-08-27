use std::{fs, panic};
use wasm_pack::stamps;

fn run_test<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    before();
    let result = panic::catch_unwind(|| test());
    after();
    assert!(result.is_ok())
}

fn before() {
    remove_stamps_file()
}

fn after() {
    remove_stamps_file()
}

fn remove_stamps_file() {
    let stamps_file_path = stamps::get_stamps_file_path().unwrap();
    if stamps_file_path.exists() {
        fs::remove_file(stamps_file_path).unwrap();
    }
}

#[test]
#[should_panic]
#[serial]
fn load_stamp_from_non_existent_file() {
    run_test(|| {
        // ACT
        let json = stamps::read_stamps_file_to_json().unwrap();
        stamps::get_stamp_value("Foo", &json).unwrap();
    })
}

#[test]
#[serial]
fn load_stamp() {
    run_test(|| {
        // ARRANGE
        stamps::save_stamp_value("Foo", "Bar").unwrap();

        // ACT
        let json = stamps::read_stamps_file_to_json().unwrap();
        let stamp_value = stamps::get_stamp_value("Foo", &json).unwrap();

        // ASSERT
        assert_eq!(stamp_value, "Bar");
    })
}

#[test]
#[serial]
fn update_stamp() {
    run_test(|| {
        // ARRANGE
        stamps::save_stamp_value("Foo", "Bar").unwrap();

        // ACT
        stamps::save_stamp_value("Foo", "John").unwrap();

        // ASSERT
        let json = stamps::read_stamps_file_to_json().unwrap();
        let stamp_value = stamps::get_stamp_value("Foo", &json).unwrap();
        assert_eq!(stamp_value, "John");
    })
}
