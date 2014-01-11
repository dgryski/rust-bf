
extern mod std;
use std::vec;
use std::os;
use std::io;

use std::io::File;

fn main() {

    let args = os::args();

    let mut mem = vec::from_elem(1024, 0u8);


    let mut f = File::open(&Path::new(args[1]));
    let program = f.read_to_end();
    let program_size = program.len();

    let mut ip = 0;
    let mut p = 0;

    while ip < program_size {

        match program[ip] as char {
          '<'  => { p -= 1; }
          '>'  => { p += 1; }
          '+'  => { mem[p] += 1; }
          '-'  => { mem[p] -= 1; }
          '.'  => { io::print(format!("{}", mem[p] as char)) }
          ','  => { let c = io::stdin().read_byte().unwrap(); mem[p] = c as u8; }
          '['  => {
            if mem[p] == 0 {
                let mut seen_open = 0;
                loop  {
                    ip += 1;
                    if seen_open == 0 && program[ip] == ']' as u8 { break ; }
                    if program[ip] == '[' as u8 { seen_open += 1; }
                    if program[ip] == ']' as u8 { seen_open -= 1; }
                }
            }
          }
          ']' => {
            if mem[p] != 0 {
                let mut seen_close = 0;
                loop  {
                    ip -= 1;
                    if seen_close == 0 && program[ip] == '[' as u8 { break ; }
                    if program[ip] == ']' as u8 { seen_close += 1; }
                    if program[ip] == '[' as u8 { seen_close -= 1; }
                }
            }
          }
          _ => {/* ignore unknown chars */ }
        }
        ip += 1;
    }
}
