
fn mock_args(file: &str) -> Vec<String> {
  vec![
    String::from("foo"),
    String::from(file)
  ]
}

#[test]
fn it_parses_first_argument_as_file_that_exists() {
  let file = core::open_input(&mock_args("ressources/HelloWorld_no_passwd_v2.x.png"));
  assert!(!file.is_err(), "File was not found.")
}

#[test]
fn it_parses_first_argument_as_file_that_not_exists() {
  let file = core::open_input(&mock_args("random.txt"));
  assert!(file.is_err(), "File should not be found.")
}

#[test]
fn it_allows_to_not_provide_a_file_argument() {
  let args = vec![String::from("foo")];
  let file = core::open_input(&args);
  assert!(!file.is_err(), "default file should be used.")
}

#[test]
fn it_opens_a_png() {

  core::Steganogramm::new()
    .write_to("/tmp/out-test-image.png") 
    .hide();
}

#[test]
fn it_takes_all_3_values() {
  core::Steganogramm::new()
    .take_data_to_hide_from("foo.txt")
    .use_carrier_image("core/ressources/HelloWorld_no_passwd_v2.x.png")
    .write_to("/tmp/out-test-image.png");
}
