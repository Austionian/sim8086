use clap::Parser;
use sim8086::disassemble;
use std::{fs::File, io::Read};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// The path to the binary file to read in
    #[arg(short, long)]
    file: String,
    /// Whether to execute the instructions
    #[arg(
        short,
        long,
        default_missing_value("true"),
        default_value("false"),
        num_args(0..=1),
        require_equals(false)
    )]
    exec: bool,

    /// Whether to dump the memory
    #[arg(
        short,
        long,
        default_missing_value("true"),
        default_value("false"),
        num_args(0..=1),
        require_equals(false)
    )]
    dump: bool,
}

fn main() {
    let args = Args::parse();
    let is_executing = args.exec;
    let is_dumping = args.dump;

    let mut file = File::open(format!("./{}", args.file)).expect("file not found");

    let mut buffer = Vec::new();

    // read in the file
    let _bytes = file.read_to_end(&mut buffer).expect("unable to read");

    println!("{}", disassemble(buffer, is_executing, is_dumping));
}
