//mod command;
//
//use super::n64::*;
//use super::cpu::opcode::Opcode::*;
//use super::cpu::instruction::*;
//use super::n64::mem_map::*;
//use super::n64::mem_map::Addr::*;
//
//use self::command::*;
//
//use std::io::*;

mod command;

use std::io::{stdin, stdout};
use std::io::prelude::*;
use std::borrow::Cow;

use super::n64::cpu::Instruction;
use super::n64::cpu::opcode::Opcode::*;
use super::n64::mem_map;
use super::n64::mem_map::Addr::*;
use super::n64::N64;
use self::command::Command;

pub struct Debugger {
    n64: N64,

    last_command: Option<Command>,
}

impl Debugger {
    pub fn new(n64: N64) -> Debugger {
        Debugger {
            n64: n64,

            last_command: None,
        }
    }

    pub fn run(&mut self) {
        loop {
            print!("r64> ");
            stdout().flush().unwrap();

            let command = match (read_stdin().parse(), self.last_command) {
                (Ok(Command::Repeat), Some(c)) => Ok(c),
                (Ok(Command::Repeat), None) => Err("No last command".into()),

                (Ok(c), _) => Ok(c),
                (Err(e), _) => Err(e),
            };

            match command {
                Ok(Command::Step(count)) => self.step(count),
                Ok(Command::Exit) => break,
                Ok(Command::Repeat) => unreachable!(),
                Err(ref e) => println!("{}", e),
            }


            self.last_command = command.ok();
        }
    }


    pub fn step(&mut self, count: usize) {
        for _ in 0..count {
            let current_pc = self.n64.cpu().current_pc_phys();
            let addr = mem_map::map_addr(current_pc as u32);
            let instr = Instruction(match addr {
                PifRom(offset) => self.n64.interconnect().pif().read_boot_rom(offset),
                _ => panic!("Debugger can't inspect address: {:?}", addr),
            });

            print!("{:018X}: ", current_pc);

            match instr.opcode() {
                Special => print!("{:?} (Special)", instr.special_op()),
                RegImm => print!("{:?} (RegImm)", instr.reg_imm_op()),
                _ => print!("{:?}", instr),
            }

            if self.n64.cpu().will_execute_from_delay_slot() {
                println!(" (DELAY)");
            } else {
                println!("");
            }

            self.n64.step();
        }
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