use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use utils::{self, fixture};
use wasm_pack::{self, manifest};

#[test]
fn it_gets_the_crate_name_default_path() {
    let path = &PathBuf::from(".");
    assert!(manifest::get_crate_name(path).is_ok());
    assert_eq!(manifest::get_crate_name(path).unwrap(), "wasm-pack");
}

#[test]
fn it_gets_the_crate_name_provided_path() {
    let fixture = fixture::js_hello_world();
    assert!(manifest::get_crate_name(&fixture.path).is_ok());
    assert_eq!(
        manifest::get_crate_name(&fixture.path).unwrap(),
        "js-hello-world"
    );
}

#[test]
fn it_checks_has_cdylib_default_path() {
    let fixture = fixture::no_cdylib();
    // Ensure that there is a `Cargo.lock`.
    fixture.cargo_check();
    let step = wasm_pack::progressbar::Step::new(1);
    assert!(manifest::check_crate_config(&fixture.path, &step).is_err());
}

#[test]
fn it_checks_has_cdylib_provided_path() {
    let fixture = fixture::js_hello_world();
    // Ensure that there is a `Cargo.lock`.
    fixture.cargo_check();
    let step = wasm_pack::progressbar::Step::new(1);
    assert!(manifest::check_crate_config(&fixture.path, &step).is_ok());
}

#[test]
fn it_checks_has_cdylib_wrong_crate_type() {
    let fixture = fixture::bad_cargo_toml();
    let step = wasm_pack::progressbar::Step::new(1);
    assert!(manifest::check_crate_config(&fixture.path, &step).is_err());
}

#[test]
fn it_recognizes_a_map_during_depcheck() {
    let fixture = fixture::serde_feature();
    // Ensure that there is a `Cargo.lock`.
    fixture.cargo_check();
    let step = wasm_pack::progressbar::Step::new(1);
    assert!(manifest::check_crate_config(&fixture.path, &step).is_ok());
}

#[test]
fn it_creates_a_package_json_default_path() {
    let fixture = fixture::js_hello_world();
    let out_dir = fixture.path.join("pkg");
    let step = wasm_pack::progressbar::Step::new(1);
    wasm_pack::command::utils::create_pkg_dir(&out_dir, &step).unwrap();
    assert!(manifest::write_package_json(&fixture.path, &out_dir, &None, false, "", &step).is_ok());
    let package_json_path = &fixture.path.join("pkg").join("package.json");
    assert!(fs::metadata(package_json_path).is_ok());
    assert!(utils::manifest::read_package_json(&fixture.path, &out_dir).is_ok());
    let pkg = utils::manifest::read_package_json(&fixture.path, &out_dir).unwrap();
    assert_eq!(pkg.name, "js-hello-world");
    assert_eq!(pkg.repository.ty, "git");
    assert_eq!(
        pkg.repository.url,
        "https://github.com/rustwasm/wasm-pack.git"
    );
    assert_eq!(pkg.module, "js_hello_world.js");
    assert_eq!(pkg.types, "js_hello_world.d.ts");
    assert_eq!(pkg.side_effects, "false");

    let actual_files: HashSet<String> = pkg.files.into_iter().collect();
    let expected_files: HashSet<String> = [
        "js_hello_world_bg.wasm",
        "js_hello_world.d.ts",
        "js_hello_world.js",
    ]
        .iter()
        .map(|&s| String::from(s))
        .collect();
    assert_eq!(actual_files, expected_files);
}

#[test]
fn it_creates_a_package_json_provided_path() {
    let fixture = fixture::js_hello_world();
    let out_dir = fixture.path.join("pkg");
    let step = wasm_pack::progressbar::Step::new(1);
    wasm_pack::command::utils::create_pkg_dir(&out_dir, &step).unwrap();
    assert!(manifest::write_package_json(&fixture.path, &out_dir, &None, false, "", &step).is_ok());
    let package_json_path = &fixture.path.join("pkg").join("package.json");
    assert!(fs::metadata(package_json_path).is_ok());
    utils::manifest::read_package_json(&fixture.path, &out_dir).unwrap();
    let pkg = utils::manifest::read_package_json(&fixture.path, &out_dir).unwrap();
    assert_eq!(pkg.name, "js-hello-world");
    assert_eq!(pkg.module, "js_hello_world.js");

    let actual_files: HashSet<String> = pkg.files.into_iter().collect();
    let expected_files: HashSet<String> = [
        "js_hello_world_bg.wasm",
        "js_hello_world.d.ts",
        "js_hello_world.js",
    ]
        .iter()
        .map(|&s| String::from(s))
        .collect();
    assert_eq!(actual_files, expected_files);
}

#[test]
fn it_creates_a_package_json_provided_path_with_scope() {
    let fixture = fixture::js_hello_world();
    let out_dir = fixture.path.join("pkg");
    let step = wasm_pack::progressbar::Step::new(1);
    wasm_pack::command::utils::create_pkg_dir(&out_dir, &step).unwrap();
    assert!(
        manifest::write_package_json(
            &fixture.path,
            &out_dir,
            &Some("test".to_string()),
            false,
            "",
            &step
        ).is_ok()
    );
    let package_json_path = &fixture.path.join("pkg").join("package.json");
    assert!(fs::metadata(package_json_path).is_ok());
    assert!(utils::manifest::read_package_json(&fixture.path, &out_dir).is_ok());
    let pkg = utils::manifest::read_package_json(&fixture.path, &out_dir).unwrap();
    assert_eq!(pkg.name, "@test/js-hello-world");
    assert_eq!(pkg.module, "js_hello_world.js");

    let actual_files: HashSet<String> = pkg.files.into_iter().collect();
    let expected_files: HashSet<String> = [
        "js_hello_world_bg.wasm",
        "js_hello_world.d.ts",
        "js_hello_world.js",
    ]
        .iter()
        .map(|&s| String::from(s))
        .collect();
    assert_eq!(actual_files, expected_files);
}

#[test]
fn it_creates_a_pkg_json_with_correct_files_on_node() {
    let fixture = fixture::js_hello_world();
    let out_dir = fixture.path.join("pkg");
    let step = wasm_pack::progressbar::Step::new(1);
    wasm_pack::command::utils::create_pkg_dir(&out_dir, &step).unwrap();
    assert!(
        manifest::write_package_json(&fixture.path, &out_dir, &None, false, "nodejs", &step)
            .is_ok()
    );
    let package_json_path = &out_dir.join("package.json");
    assert!(fs::metadata(package_json_path).is_ok());
    utils::manifest::read_package_json(&fixture.path, &out_dir).unwrap();
    let pkg = utils::manifest::read_package_json(&fixture.path, &out_dir).unwrap();
    assert_eq!(pkg.name, "js-hello-world");
    assert_eq!(pkg.repository.ty, "git");
    assert_eq!(
        pkg.repository.url,
        "https://github.com/rustwasm/wasm-pack.git"
    );
    assert_eq!(pkg.main, "js_hello_world.js");
    assert_eq!(pkg.types, "js_hello_world.d.ts");

    let actual_files: HashSet<String> = pkg.files.into_iter().collect();
    let expected_files: HashSet<String> = [
        "js_hello_world_bg.wasm",
        "js_hello_world_bg.js",
        "js_hello_world.d.ts",
    ]
        .iter()
        .map(|&s| String::from(s))
        .collect();
    assert_eq!(actual_files, expected_files);
}

#[test]
fn it_creates_a_pkg_json_with_correct_files_on_nomodules() {
    let fixture = fixture::js_hello_world();
    let out_dir = fixture.path.join("pkg");
    let step = wasm_pack::progressbar::Step::new(1);
    wasm_pack::command::utils::create_pkg_dir(&out_dir, &step).unwrap();
    assert!(
        manifest::write_package_json(&fixture.path, &out_dir, &None, false, "no-modules", &step)
            .is_ok()
    );
    let package_json_path = &out_dir.join("package.json");
    assert!(fs::metadata(package_json_path).is_ok());
    utils::manifest::read_package_json(&fixture.path, &out_dir).unwrap();
    let pkg = utils::manifest::read_package_json(&fixture.path, &out_dir).unwrap();
    assert_eq!(pkg.name, "js-hello-world");
    assert_eq!(pkg.repository.ty, "git");
    assert_eq!(
        pkg.repository.url,
        "https://github.com/rustwasm/wasm-pack.git"
    );
    assert_eq!(pkg.browser, "js_hello_world.js");
    assert_eq!(pkg.types, "js_hello_world.d.ts");

    let actual_files: HashSet<String> = pkg.files.into_iter().collect();
    let expected_files: HashSet<String> = [
        "js_hello_world_bg.wasm",
        "js_hello_world.js",
        "js_hello_world.d.ts",
    ]
        .iter()
        .map(|&s| String::from(s))
        .collect();
    assert_eq!(actual_files, expected_files);
}

#[test]
fn it_creates_a_pkg_json_in_out_dir() {
    let fixture = fixture::js_hello_world();
    let out_dir = fixture.path.join("./custom/out");
    let step = wasm_pack::progressbar::Step::new(1);
    wasm_pack::command::utils::create_pkg_dir(&out_dir, &step).unwrap();
    assert!(manifest::write_package_json(&fixture.path, &out_dir, &None, false, "", &step).is_ok());

    let package_json_path = &fixture.path.join(&out_dir).join("package.json");
    assert!(fs::metadata(package_json_path).is_ok());
    assert!(utils::manifest::read_package_json(&fixture.path, &out_dir).is_ok());
}

#[test]
fn it_creates_a_package_json_with_correct_keys_when_types_are_skipped() {
    let fixture = fixture::js_hello_world();
    let out_dir = fixture.path.join("pkg");
    let step = wasm_pack::progressbar::Step::new(1);
    wasm_pack::command::utils::create_pkg_dir(&out_dir, &step).unwrap();
    assert!(manifest::write_package_json(&fixture.path, &out_dir, &None, true, "", &step).is_ok());
    let package_json_path = &out_dir.join("package.json");
    assert!(fs::metadata(package_json_path).is_ok());
    assert!(utils::manifest::read_package_json(&fixture.path, &out_dir).is_ok());
    let pkg = utils::manifest::read_package_json(&fixture.path, &out_dir).unwrap();
    assert_eq!(pkg.name, "js-hello-world");
    assert_eq!(pkg.repository.ty, "git");
    assert_eq!(
        pkg.repository.url,
        "https://github.com/rustwasm/wasm-pack.git"
    );
    assert_eq!(pkg.module, "js_hello_world.js");

    let actual_files: HashSet<String> = pkg.files.into_iter().collect();
    let expected_files: HashSet<String> = ["js_hello_world_bg.wasm", "js_hello_world.js"]
        .iter()
        .map(|&s| String::from(s))
        .collect();
    assert_eq!(actual_files, expected_files);
}

#[test]
fn it_errors_when_wasm_bindgen_is_not_declared() {
    let fixture = fixture::bad_cargo_toml();
    let step = wasm_pack::progressbar::Step::new(1);
    assert!(manifest::check_crate_config(&fixture.path, &step).is_err());
}

#[test]
fn it_does_not_error_when_wasm_bindgen_is_declared() {
    let fixture = fixture::js_hello_world();
    // Ensure that there is a `Cargo.lock`.
    fixture.cargo_check();
    let step = wasm_pack::progressbar::Step::new(1);
    assert!(manifest::check_crate_config(&fixture.path, &step).is_ok());
}
