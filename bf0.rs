
extern mod std;
use std::result;
use std::vec;
use std::os;
use std::io;

fn main() {

    let args = os::args();

    let mut mem = vec::from_elem(1024, 0u8);

    let mut program : ~[u8];

    match std::io::file_reader(&Path(args[1])) {
        result::Ok(f) => { program = f.read_whole_stream(); }
        result::Err(e) => {
            let err = fmt!("%s: %s: %s\n",args[0],args[1],e);
            std::io::stderr().write_str(err);
            return;
        }
    }

    let program_size = program.len();

    let mut ip = 0;
    let mut p = 0;

    while ip < program_size {

        match program[ip] as char {
          '<'  => { p -= 1; }
          '>'  => { p += 1; }
          '+'  => { mem[p] += 1; }
          '-'  => { mem[p] -= 1; }
          '.'  => { io::print(fmt!("%c",mem[p]as char)) }
          ','  => { let c = io::stdin().read_byte(); mem[p] = c as u8; }
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
