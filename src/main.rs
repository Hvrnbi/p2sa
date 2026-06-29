mod lib;

use clap::Parser;

#[derive(Parser)]
struct Args {
    src_path: String,
    output_path: String,
    output_width: u32,
    output_height: u32,
    nails_count: u16
}

fn main() {
    let args = Args::parse();

    lib::p2sa(args.src_path, args.output_path, [args.output_width, args.output_height], args.nails_count);
}
