
use gbemu::{
  bootrom,
  cartridge,
  peripherals,
  cpu,
  joypad,
};
use std::{
  env,
  io::Read,
  process::exit,
};

mod gameboy;
mod lcd;


fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    eprintln!("The file name argument is required.");
    exit(1);
  }

  let bootrom = bootrom::Bootrom::new();

  let mut gameboy = gameboy::GameBoy::new(bootrom);
  gameboy.run();
}

