#![allow(unused)]
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::env;
use std::io::BufWriter;

fn main() -> io::Result<()> {
  let args: Vec<String> = env::args().collect();
  let mut f = core::open_input(&args)
    .expect("File cannot be opened.");

  match args.len() {
    4 => { 
      core::Steganogramm::new()
        .take_data_to_hide_from("foo.txt")
        .use_carrier_image("core/ressources/HelloWorld_no_passwd_v2.x.png")
        .write_to(&args[3])
        .hide();
    },
    _ => {}
  }

  loop {
    let mut buffer = [0; 10];
    let mut bytes_read = f.read(&mut buffer)
      .expect("Error during reading data");

    println!("File Content: {:?}", buffer);

    if bytes_read < buffer.len() {
      break
    }
  }

  Ok(())
}