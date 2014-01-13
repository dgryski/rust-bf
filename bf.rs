
extern mod extra;
extern mod std;

use std::io;
use std::os;
use std::vec;
use std::io::File;

use extra::getopts::{optflag,getopts};

fn main() {

    let args = os::args();

    let program_name = args[0].clone();

    let opts = ~[
        optflag("C"),
    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };

    if matches.free.is_empty() || matches.free.len() != 1 {
            let err = format!("usage: {} [-C] filename.bf\n", program_name);
            io::stderr().write_str(err);
            return;
    }

    let input_file = matches.free[0].clone();

    let program : ~[u8] ;
    
    match load_program(input_file) {
        Some(p) => { program = p; }
        None => { fail!( ~"no program loaded" ) }
    }

    let offsets = calculate_offsets(program);

    if matches.opt_present("C") {
        output_c_code(program, offsets);
    } else {
        run_program(program, offsets);
    }
}

fn load_program(filename : &str) -> Option<~[u8]> {

    let mut f = File::open(&Path::new(filename));
    let mut program = f.read_to_end();

    program = program.iter().filter_map(|c| {
        let c = *c as char;
        if (c == '<' || c == '>' || c == '+' || c == '-' || c == '.' || c == ',' || c == '[' || c == ']') { Some(c as u8) } else { None }
    }).to_owned_vec();

    Some(program)
}

fn calculate_offsets(program : &[u8]) -> ~[uint] {

    let mut offsets = vec::from_elem(program.len(), 0 as uint);

    let mut prev = program[0];
    let mut count = 0u;

    // group like operations
    for (i, p) in program.iter().enumerate() {
        if *p == prev {
            count += 1;
        } else {
            let pc = prev as char;
            if pc == '<' || pc == '>' || pc == '+' || pc == '-' {
                offsets[i - count] = count;
            }
            count = 1;
            prev = *p;
        }
    }

    // precalculate jump locations
    for (i, p) in program.iter().enumerate() {
        if *p == '[' as u8 {
            let mut dst_ip = i;

            let mut seen_open = 0;
            loop  {
                dst_ip += 1;
                if seen_open == 0 && program[dst_ip] == ']' as u8 {
                    offsets[i] = dst_ip;
                    break ;
                }
                if program[dst_ip] == '[' as u8 { seen_open += 1; }
                if program[dst_ip] == ']' as u8 { seen_open -= 1; }
            }
        } else if *p == ']' as u8 {
            let mut dst_ip = i;

            let mut seen_close = 0;
            loop  {
                dst_ip -= 1;
                if seen_close == 0 && program[dst_ip] == '[' as u8 {
                    offsets[i] = dst_ip;
                    break ;
                }
                if program[dst_ip] == ']' as u8 { seen_close += 1; }
                if program[dst_ip] == '[' as u8 { seen_close -= 1; }
            }
        }
    }

    offsets
}


fn run_program(program : &[u8], offsets : &[uint]) {

    let mut ip = 0;
    let mut p = 0;

    let mut mem = vec::from_elem(1024, 0u8);
    let program_size = program.len();

    while ip < program_size {

        match program[ip] as char {
          '<' => { p -= offsets[ip]; ip += offsets[ip] - 1; }
          '>' => { p += offsets[ip]; ip += offsets[ip] - 1; }
          '+' => { mem[p] += offsets[ip] as u8; ip += offsets[ip] - 1;}
          '-' => { mem[p] -= offsets[ip] as u8; ip += offsets[ip] - 1; }
          '.' => { io::print(format!("{}",mem[p] as char)); }
          ',' => { let c = io::stdin().read_byte().unwrap(); mem[p] = c; }
          '[' => { if mem[p] == 0 { ip = offsets[ip]; } }
          ']' => { if mem[p] != 0 { ip = offsets[ip]; } }
          _ => { fail!( format!("unknown char in input: {}", program[ip] as char)); }
        }
        ip += 1;
    }
}

fn output_c_code(program : &[u8], offsets : &[uint]) {
    io::println("
#include <stdio.h>
unsigned char mem [1024];
int main() {
unsigned char *p = mem;
    ");

    let mut ip = 0;
    let program_size = program.len();

    while ip < program_size {
        match program[ip] as char {
          '<' => { io::println(format!("p -= {};", offsets[ip])); ip += offsets[ip] - 1; }
          '>' => { io::println(format!("p += {};", offsets[ip])); ip += offsets[ip] - 1; }
          '+' => { io::println(format!("*p += {};", offsets[ip])); ip += offsets[ip] - 1;}
          '-' => { io::println(format!("*p -= {};", offsets[ip])); ip += offsets[ip] - 1;}
          '.' => { io::println("putchar(*p);"); }
          ',' => { io::println("*p = (char)getchar();"); }
          '[' => { io::println("while(*p) {"); }
          ']' => { io::println("}"); }
          _ => { fail!( format!("unknown char in input: {}", program[ip] as char)); }
        }
        ip += 1;
    }

    io::println("}");
}
