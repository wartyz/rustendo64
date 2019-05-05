#![deny(trivial_casts, trivial_numeric_casts)]
extern crate byteorder;

extern crate num;

#[macro_use]
extern crate enum_primitive;

#[macro_use]
extern crate nom;

mod n64;
mod debugger;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use debugger::Debugger;
use n64::N64;


fn main() {
    let pif_file_name = env::args().nth(1).unwrap(); // pifdata.bin (BIOS)
    let rom_file_name = env::args().nth(2).unwrap(); // Nombre viene como argumento

    let pif = read_bin(pif_file_name); // Lee BIOS
    let rom = read_bin(rom_file_name); // Lee ROM

    let n64 = N64::new(pif, rom);
    let mut debugger = Debugger::new(n64);
    debugger.run();
}

/// Lee un fichero y devuelve un vector de u8
fn read_bin<P: AsRef<Path>>(path: P) -> Box<[u8]> {
    let mut file = File::open(path).unwrap(); // Abre fichero
    let mut file_buf = Vec::new();           // Crea un buffer de u8
    file.read_to_end(&mut file_buf).unwrap();          // Lee fichero en buffer
    file_buf.into_boxed_slice()   // Convierte un Vec en un slice de vectores Box
}