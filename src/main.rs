
use std::env;
use std::vec;

use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;

extern crate getopts;
use getopts::Options;

fn main() {


    let args: Vec<String> = env::args().collect();
    let ref arg1 = args[1];

    let program_name = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("C", "", "generate C code");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.free.is_empty() || matches.free.len() != 1 {
        let err = format!("usage: {} [-C] filename.bf\n", program_name);
        io::stderr().write(err.as_bytes());
        return;
    }

    let input_file = matches.free[0].clone();

    let program: vec::Vec<u8>;

    match load_program(input_file) {
        Some(p) => program = p,
        None => panic!("no program loaded"),
    }

    let offsets = calculate_offsets(&program);

    if matches.opt_present("C") {
        output_c_code(program, offsets);
    } else {
        run_program(program, offsets);
    }
}

fn load_program(filename: String) -> Option<Vec<u8>> {

    let mut f = File::open(filename).expect("unable to open program");

    let mut program_str = String::new();

    f.read_to_string(&mut program_str).expect("unable to read program");

    let program = program_str.bytes()
        .filter_map(|c| {
            let c = c as char;
            if c == '<' || c == '>' || c == '+' || c == '-' || c == '.' || c == ',' || c == '[' ||
               c == ']' {
                Some(c as u8)
            } else {
                None
            }
        })
        .collect();

    Some(program)
}

fn calculate_offsets(program: &vec::Vec<u8>) -> Vec<usize> {

    let mut offsets = vec![0usize; program.len()];

    let mut prev = program[0];
    let mut count = 0;

    // group like operations
    for (i, p) in program.bytes().enumerate() {
        let p = p.unwrap();
        if p == prev {
            count += 1;
        } else {
            let pc = prev as char;
            if pc == '<' || pc == '>' || pc == '+' || pc == '-' {
                offsets[i - count] = count;
            }
            count = 1;
            prev = p;
        }
    }

    // precalculate jump locations
    for (i, p) in program.iter().enumerate() {
        if *p == '[' as u8 {
            let mut dst_ip = i;

            let mut seen_open = 0;
            loop {
                dst_ip += 1;
                if seen_open == 0 && program[dst_ip] == ']' as u8 {
                    offsets[i] = dst_ip;
                    break;
                }
                if program[dst_ip] == '[' as u8 {
                    seen_open += 1;
                }
                if program[dst_ip] == ']' as u8 {
                    seen_open -= 1;
                }
            }
        } else if *p == ']' as u8 {
            let mut dst_ip = i;

            let mut seen_close = 0;
            loop {
                dst_ip -= 1;
                if seen_close == 0 && program[dst_ip] == '[' as u8 {
                    offsets[i] = dst_ip;
                    break;
                }
                if program[dst_ip] == ']' as u8 {
                    seen_close += 1;
                }
                if program[dst_ip] == '[' as u8 {
                    seen_close -= 1;
                }
            }
        }
    }

    offsets
}


fn run_program(program: vec::Vec<u8>, offsets: vec::Vec<usize>) {

    let mut ip = 0;
    let mut p = 0;

    let mut mem = vec![0u8; 1024];
    let program_size = program.len();

    while ip < program_size {

        match program[ip] as char {
            '<' => {
                p -= offsets[ip];
                ip += offsets[ip] - 1;
            }
            '>' => {
                p += offsets[ip];
                ip += offsets[ip] - 1;
            }
            '+' => {
                mem[p] = mem[p].wrapping_add(offsets[ip] as u8);
                ip = ip.wrapping_add(offsets[ip] - 1)
            }
            '-' => {
                mem[p] = mem[p].wrapping_sub(offsets[ip] as u8);
                ip = ip.wrapping_add(offsets[ip] - 1)
            }
            '.' => {
                print!("{}", mem[p] as char);
            }
            ',' => {
                let mut b = vec![0; 1];
                let c = io::stdin().read(&mut b).unwrap();
                mem[p] = c as u8;
            }
            '[' => {
                if mem[p] == 0 {
                    ip = offsets[ip];
                }
            }
            ']' => {
                if mem[p] != 0 {
                    ip = offsets[ip];
                }
            }
            _ => {
                panic!(format!("unknown char in input: {}", program[ip] as char));
            }
        }
        ip += 1;
    }
}

fn indent(depth: usize) {
    for _ in 0..depth {
        print!("\t")
    }
}

fn output_c_code(program: vec::Vec<u8>, offsets: vec::Vec<usize>) {
    println!("
#include <stdio.h>
unsigned char mem [1024];
int main() {{
\tunsigned char *p = mem;
    ");

    let mut ip = 0usize;
    let program_size = program.len();

    let mut depth = 1;

    while ip < program_size {
        match program[ip] as char {
            '<' => {
                indent(depth);
                println!("p -= {};", offsets[ip]);
                ip += offsets[ip] - 1;
            }
            '>' => {
                indent(depth);
                println!("p += {};", offsets[ip]);
                ip += offsets[ip] - 1;
            }
            '+' => {
                indent(depth);
                println!("*p += {};", offsets[ip]);
                ip += offsets[ip] - 1;
            }
            '-' => {
                indent(depth);
                println!("*p -= {};", offsets[ip]);
                ip += offsets[ip] - 1;
            }
            '.' => {
                indent(depth);
                println!("putchar(*p);");
            }
            ',' => {
                indent(depth);
                println!("*p = (char)getchar();");
            }
            '[' => {
                indent(depth);
                depth = depth + 1;
                println!("while(*p) {{");
            }
            ']' => {
                depth = depth - 1;
                indent(depth);
                println!("}}");
            }
            _ => {
                panic!(format!("unknown char in input: {}", program[ip] as char));
            }
        }
        ip += 1;
    }

    println!("}}");
}
