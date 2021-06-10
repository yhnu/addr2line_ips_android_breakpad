extern crate clap;
use clap::{App, Arg};
use regex::Regex;
use regex::RegexBuilder;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process;

use addr2line_breakpad::{lookup_address, parse_address, parse_breakpad_symbol_file, SymbolFile};

fn get_ips_offsets(ips: &Path, soname: &str) -> Vec<u64> {
    // let mut addresses = HashMap::new();
    let file = File::open(ips).unwrap();
    let mut reader = BufReader::new(file);
    let mut input = String::new();
    reader.read_to_string(&mut input).unwrap();
    let input = input.as_str();

    let re = r"^(?P<i>\d+)\s*(?P<so>[a-zA-Z.]*)\s*0x(?P<mem_address>[0-9a-f]*)\s*0x(?P<base>[0-9a-f]*)\s\+\s(?P<offset>[0-9]*)";
    let re: Regex = RegexBuilder::new(re).multi_line(true).build().unwrap();
    let mut offsets = vec![];
    for caps in re.captures_iter(input) {
        // println!("{:?}", caps);
        // println!("Movie: {:?}, Released: {:?}", &caps["size"], &caps["size2"]);
        if &caps["so"] == soname {
            let e = caps["offset"].parse::<u64>().unwrap();
            offsets.push(e);
        }
    }
    offsets
    //
    // let x = RE
    //     .captures(input)
    //     .and_then(|cap| cap.name("size").map(|login| login.as_str()));
    // println!("{}", x.unwrap());
    // let s = re.find(input.as_str())..as_str();
    // println!("{}", s);

    // let mut symbol_file = SymbolFile {
    //     files: HashMap::new(),
    //     functions: RangeMap::new(),
    //     lines: RangeMap::new(),
    //     public_symbols: BTreeMap::new(),
    // };
}

fn get_symed_line(symbol_file: &SymbolFile, address: &u64) -> String {
    if let Some(symbol) = lookup_address(&symbol_file, *address) {
        let source_file_name = if symbol.source_file_name.len() != 0 {
            symbol.source_file_name
        } else {
            String::from("??")
        };
        let source_file_number = if symbol.source_file_number != -1 {
            symbol.source_file_number.to_string()
        } else {
            String::from("?")
        };
        format!(
            "{} {}:{}",
            symbol.function_name, source_file_name, source_file_number,
        )
    } else {
        format!("Not found symbol for address({:#x}", address)
    }
}

fn parser_ips(ips: &Path, soname: &str, symfile: &SymbolFile) {
    let file = File::open(ips).unwrap();
    let reader = BufReader::new(file);

    let re = r"^(?P<i>\d+)\s*(?P<so>[a-zA-Z.]*)\s*0x(?P<mem_address>[0-9a-f]*)\s*0x(?P<base>[0-9a-f]*)\s\+\s(?P<offset>[0-9]*)$";
    let re: Regex = RegexBuilder::new(re).multi_line(true).build().unwrap();
    for line in reader.lines() {
        let line = line.unwrap();
        let line = line.as_str();
        let cap = re.captures(line);
        match cap {
            Some(cap) => {
                // let s:Vec<&str> = cap.iter().map(|e| { e.unwrap().as_str()}).collect();
                if &cap["so"] == soname {
                    let offset = &cap["offset"].parse::<u64>();
                    match offset {
                        Ok(e) => {
                            let symed_offset = get_symed_line(symfile, e);
                            let line = line.replace(&cap["offset"], symed_offset.as_str());
                            println!("{}", line);
                        }
                        _ => (),
                    }
                }
                // println!("{} {} {} {}", cap["i"], cap["so"], cap["mem_address"])
            }
            None => println!("{}", line),
        }
        // println!("{:?}", caps);
        // println!("Movie: {:?}, Released: {:?}", &caps["size"], &caps["size2"]);
        //     if &caps["so"] == soname {
        //         let e = caps["offset"].parse::<u64>().unwrap();
        //         offsets.push(e);
        //     }
        // }
    }
}
//
// https://chromium.googlesource.com/breakpad/breakpad/+/master/docs/symbol_files.md
fn main() {
    let matches = App::new("addr2line for ips Breakpad symbol file")
        .version("1.0")
        .author("yiluoyang <buutuud@gmail.com>")
        .arg(
            Arg::with_name("input")
                .help("input symbol file")
                .required(true),
        )
        .arg(
            Arg::with_name("ips")
                .help("ips file to lookup")
                .multiple(true)
                .required(true),
        )
        .get_matches();

    let input = matches.value_of("input").unwrap();
    let input = Path::new(input);
    if !input.exists() {
        println!("input file({}) is not exists", input.display());
        process::exit(-1);
    }

    let ips = matches.value_of("ips").unwrap();
    let ips = Path::new(ips);
    if !ips.exists() {
        println!("ips file({}) is not exists", input.display());
        process::exit(-1);
    }

    let symbol_file = parse_breakpad_symbol_file(input);
    parser_ips(ips, "UnityFramework", &symbol_file);
    process::exit(0);
}
