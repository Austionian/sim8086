use sim8086::disassemble;
use std::{fs::File, io::Read};

fn main() {
    let args = std::env::args()
        .nth(1)
        .expect("please specify the file to read");

    let mut file = File::open(format!("./{args}")).expect("file not found");

    let mut buffer = Vec::new();

    // read in the file
    let _bytes = file.read_to_end(&mut buffer).expect("unable to read");

    println!("{}", disassemble(buffer));
}
