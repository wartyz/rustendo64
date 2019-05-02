mod command;

use super::n64::*;
use self::command::*;

use std::io::*;

pub struct Debugger {
    n64: N64
}

impl Debugger {
    pub fn new(n64: N64) -> Debugger {
        Debugger {
            n64: n64
        }
    }

    pub fn run(&mut self) {
        loop {
            //self.n64.run_instruction();
            print!("r64> ");
            stdout().flush().unwrap();

            let command = read_stdin().parse();

            match command {
                Ok(Command::Step) => self.step(),
                Ok(Command::Exit) => break,
                Err(_) => println!("Invalid input")
            }
        }
    }

    pub fn step(&mut self) {
        print!("{:018X}: ", self.n64.cpu().reg_pc());

//        match instr.opcode() {
//            Special => print!("Special: {:?}", instr.special_op()),
//            RegImm => print!("RegImm: {:?}", instr.reg_imm_op()),
//            _ => print!("{:?}", instr)
//        }
//        match delay_slot {
//            DelaySlot::Yes => println!(" (DELAY)"),
//            _ => println!("")
//        };
        // Print next PC/instruction

        self.n64.step();
    }
}

fn read_stdin() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().into()
}

//enum DelaySlot {
//    Yes,
//    No,
//}

// TODO: Move elsewhere
//fn print_instr(instr: Instruction, pc: u64, delay_slot: DelaySlot) {
//    print!("reg_pc {:018X}: ", pc);
//    match instr.opcode() {
//        Special => print!("Special: {:?}", instr.special_op()),
//        RegImm => print!("RegImm: {:?}", instr.reg_imm_op()),
//        _ => print!("{:?}", instr)
//    }
//    match delay_slot {
//        DelaySlot::Yes => println!(" (DELAY)"),
//        _ => println!("")
//    };
//}