
extern mod std;
use io::{ReaderUtil,WriterUtil};

fn main() {

     let args = os::args();

    if vec::len(args) != 2 {
            let err = fmt!("usage: %s filename.bf\n",args[0]);
            io::stderr().write_str(err);
            return;
    }

    let program : ~[u8] ;
    
    match load_program(args[1]) {
        Some(p) => { program = copy p; }
        None => { fail ~"no program loaded" }
    }

    let offsets = calculate_offsets(program);

    run_program(program, offsets);
//  output_c_code(program, offsets);

}

fn load_program(filename : &str) -> Option<~[u8]> {

    let mut program : ~[u8];

    match io::file_reader(&Path(filename)) {
        result::Ok(f) => { program = f.read_whole_stream(); }
        result::Err(e) => {
            let err = fmt!("%s: %s\n",filename,e);
            io::stderr().write_str(err);
            return None;
        }
    }

    program = vec::filter(program, |c| {
        let c = *c as char; c == '<' || c == '>' || c == '+' || c == '-' || c == '.' || c == ',' || c == '[' || c == ']'
    });

    Some(program)
}

fn calculate_offsets(program : &[u8]) -> ~[uint] {

    let mut offsets = vec::from_elem(vec::len(program), 0);

    let mut prev = program[0];
    let mut count = 0u;

    // group like operations
    for program.eachi |i, p| {
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
    for program.eachi |i, p| {
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

    move offsets
}


fn run_program(program : &[u8], offsets : &[uint]) {

    let mut ip = 0;
    let mut p = 0;

    let mut mem = vec::from_elem(1024, 0u8);
    let program_size = vec::len(program);

    while ip >= 0 && ip < program_size {

        match program[ip] {
          '<' as u8 => { p -= offsets[ip]; ip += offsets[ip] - 1; }
          '>' as u8 => { p += offsets[ip]; ip += offsets[ip] - 1; }
          '+' as u8 => { mem[p] += offsets[ip] as u8; ip += offsets[ip] - 1;}
          '-' as u8 => { mem[p] -= offsets[ip] as u8; ip += offsets[ip] - 1; }
          '.' as u8 => { io::print(fmt!("%c",mem[p]as char)); }
          ',' as u8 => { let c = io::stdin().read_byte(); mem[p] = c as u8; }
          '[' as u8 => { if mem[p] == 0 { ip = offsets[ip]; } }
          ']' as u8 => { if mem[p] != 0 { ip = offsets[ip]; } }
          _ => { fail fmt!("unknow char in input: %c", program[ip] as char); }
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
    let program_size = vec::len(program);

    while (ip < program_size) {
        match program[ip] {
          '<' as u8 => { io::println(fmt!("p -= %u;", offsets[ip])); ip += offsets[ip] - 1; }
          '>' as u8 => { io::println(fmt!("p += %u;", offsets[ip])); ip += offsets[ip] - 1; }
          '+' as u8 => { io::println(fmt!("*p += %u;", offsets[ip])); ip += offsets[ip] - 1;}
          '-' as u8 => { io::println(fmt!("*p -= %u;", offsets[ip])); ip += offsets[ip] - 1;}
          '.' as u8 => { io::println("putchar(*p);"); }
          ',' as u8 => { io::println("*p = (char)getchar();"); }
          '[' as u8 => { io::println("while(*p) {"); }
          ']' as u8 => { io::println("}"); }
          _ => { fail fmt!("unknow char in input: %c", program[ip] as char); }
        }
        ip += 1;
    }

    io::println("}");
}
