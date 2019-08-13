use std::path::Path;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use png;

pub struct Steganogramm {
  encoder: Option<png::Encoder<std::io::BufWriter<std::fs::File>>>,
  decoder: Option<png::Decoder<std::fs::File>>,
  source: Option<std::fs::File>
}

impl Steganogramm {
  pub fn new() -> Steganogramm {
    Steganogramm { 
      encoder: None, 
      decoder: None,
      source: None
    }
  }

  pub fn use_carrier_image(&mut self, input_file: &str) -> &mut Steganogramm {
    self.decoder = Some(png::Decoder::new(File::open(input_file).unwrap()));
    self
  }

  pub fn write_to(&mut self, output_file: &str) -> &mut Steganogramm {
    let path = Path::new(output_file);
    let width: u32 = 512;
    let height: u32 = 512;

    self.encoder = Some(png::Encoder::new(
      BufWriter::new(File::create(path).unwrap()),
           width, height));
    self
  }

  pub fn take_data_to_hide_from(&mut self, input_file: &str) -> &mut Steganogramm {
    self.source = Some(File::open(input_file)
      .unwrap());
      // .expect("Source file was not readable.")
      
    self
  }

  pub fn hide(&mut self) -> &mut Steganogramm {
    // TODO try out 
    //   let fragment = req.url.take().unwrap_or_else(String::new);

    match self.encoder {
      None => {},
      Some(ref _enc) => {
        // TODO feat(core:hide) implement the basic functionality here
        
        // let data = [0x15; 512*512]; // An array containing a RGBA sequence. First pixel is red and second pixel is black.
        // enc.write_header().unwrap().write_image_data(&data).unwrap(); // Save
        // self.encoder = Some(*enc)
      }
    }

    self
  }
}

// TODO cleanup(unused)
pub fn open_input(args: &Vec<String>) -> io::Result<File> {
  let mut filename = "/dev/random";

  match args.len() {
    2 => filename = &args[1],
    _ => {}
  }

  File::open(filename)
}

// TODO cleanup(unused)
pub fn open_input_image(args: &Vec<String>) -> png::Decoder<File> {
  match args.len() {
    3 => return png::Decoder::new(File::open(&args[2]).unwrap()),
    _ => panic!("no image file given."),
  }
}