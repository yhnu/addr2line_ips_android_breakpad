extern crate clap;
use clap::{App, Arg};
use std::path::Path;
use std::process;

use addr2line_breakpad::{parse_address, parse_breakpad_symbol_file, lookup_address};

// https://chromium.googlesource.com/breakpad/breakpad/+/master/docs/symbol_files.md
fn main() {
    let matches = App::new("addr2line for Breakpad symbol file")
        .version("1.0")
        .author("liudingsan <lds2012@gmail.com>")
        .arg(Arg::with_name("input").help("input symbol file").required(true))
        .arg(Arg::with_name("address").help("address to lookup").multiple(true).required(true))
        .get_matches();

    let input = matches.value_of("input").unwrap();
    let input = Path::new(input);
    if !input.exists() {
        println!("input file({}) is not exists", input.display());
        process::exit(-1);
    }

    let addresses: Vec<u64> = matches.values_of("address").unwrap().map(|addr| parse_address(addr).unwrap()).collect();

    let symbol_file = parse_breakpad_symbol_file(input);

    for address in addresses {
        if let Some(symbol) = lookup_address(&symbol_file, address) {
            let source_file_name = if symbol.source_file_name.len() != 0 { symbol.source_file_name } else { String::from("??") };
            let source_file_number = if symbol.source_file_number != -1 { symbol.source_file_number.to_string() } else { String::from("?") };
            println!(
                "{:#x} {} {}:{}",
                address, symbol.function_name, source_file_name, source_file_number
            );
        } else {
            println!("Not found symbol for address({:#x}", address);
        }
    }
}
