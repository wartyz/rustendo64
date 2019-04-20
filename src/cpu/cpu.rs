use std::fmt;
use super::cp0;
use super::opcode::Opcode::*;
use super::super::interconnect;
use super::instruction::Instruction;
//use std::intrinsics::init;

const NUM_GPR: usize = 32; // 32 registros

#[derive(Debug)]
pub struct Cpu {
    // Arreglo de registros de proposito general
    reg_gpr: [u64; NUM_GPR],
    // Arreglo de registros de coma flotante
    reg_fpr: [f64; NUM_GPR],
    // Contador de programa
    reg_pc: u64,
    // Para resultados de palabra doble??
    reg_hi: u64,
    // Para resultados de palabra doble??
    reg_lo: u64,

    reg_llbit: bool,
    //TODO: Enum type
    //Registro de Implementacion/Revision FCR0
    reg_fcr0: u32,
    // Registro de Control/Status FCR31
    reg_fcr31: u32,

    cp0: cp0::Cp0,

    interconnect: interconnect::Interconnect,
}

//// Para presentar adecuadamente los registros, cambio el macro try! por ?
//impl fmt::Debug for Cpu {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        const REGS_PER_LINE: usize = 2;
//        const REG_NAMES: [&'static str; NUM_GPR] = [
//            "r0", "at", "v0", "v1", "a0", "a1", "a2", "a3",
//            "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7",
//            "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7",
//            "t8", "t9", "k0", "k1", "gp", "sp", "s8", "ra",
//        ];
//
//        write!(f, "\nCPU General Purpose Registers:")?;
//        for reg_num in 0..NUM_GPR {
//            if (reg_num % REGS_PER_LINE) == 0 {
//                writeln!(f, "")?;
//            }
//            write!(f,
//                   "{reg_name}/gpr{num:02}: {value:#018X} ",
//                   num = reg_num,
//                   reg_name = REG_NAMES[reg_num],
//                   value = self.reg_gpr[reg_num],
//            )?;
//        }
//
//        write!(f, "\n\nCPU Floating Point Registers:")?;
//        for reg_num in 0..NUM_GPR {
//            if (reg_num % REGS_PER_LINE) == 0 {
//                writeln!(f, "")?;
//            }
//            write!(f,
//                   "fpr{num:02}: {value:21} ",
//                   num = reg_num,
//                   value = self.reg_fpr[reg_num], )?;
//        }
//
//        writeln!(f, "\n\nCPU Special Registers:")?;
//        writeln!(f,
//                 "\
//            reg_pc: {:#018X}\n\
//            reg_hi: {:#018X}\n\
//            reg_lo: {:#018X}\n\
//            reg_llbit: {}\n\
//            reg_fcr0:  {:#010X}\n\
//            reg_fcr31: {:#010X}\n\
//            ",
//                 self.reg_pc,
//                 self.reg_hi,
//                 self.reg_lo,
//                 self.reg_llbit,
//                 self.reg_fcr0,
//                 self.reg_fcr31
//        )?;
//
//        writeln!(f, "{:#?}", self.cp0)?;
//        writeln!(f, "{:#?}", self.interconnect)
//    }
//}

// CPU
impl Cpu {
    pub fn new(interconnect: interconnect::Interconnect) -> Cpu {
        Cpu {
            reg_gpr: [0; NUM_GPR],   // Registros de proposito general
            reg_fpr: [0.0; NUM_GPR], // Registros de coma flotante
            reg_pc: 0,               // Contador de programa
            reg_hi: 0,               // Registros de multiplicar y dividie
            reg_lo: 0,
            reg_llbit: false,        // Registro Load/Link
            reg_fcr0: 0,             // Registro Implementacion/revision coma flotante
            reg_fcr31: 0,            // Registro control/status coma flotante
            cp0: cp0::Cp0::default(),

            interconnect: interconnect,
        }
    }

    pub fn power_on_reset(&mut self) {
        self.cp0.power_on_reset();

        self.reg_pc = 0xffff_ffff_bfc0_0000; // TODO: move to const
    }

    // TODO: Different interface
    pub fn run(&mut self) {
        loop {
            self.run_instruction();
        }
    }

    pub fn run_instruction(&mut self) {
        let instr = Instruction(self.read_word(self.reg_pc));

//        //TODO: Check endian
//        let opcode =  // Opcode 6 bits altos
//        let rs =  // Registro source
//        let instr.rt() =  // Registro destino (target)
//        let imm =  // Valor inmediato

        // instr.rt() => reg source, rd => reg destino, instr.rt() => reg destino/fuente
        // Tipo R-> opcode  rs   instr.rt()    rd   shamt funct
        //          XXXXXX XXXXX XXXXX         XXXXX XXXXX XXXXXX
        // Tipo I-> opcode  rs    instr.rt()    immediate(imm)
        //          XXXXXX XXXXX XXXXX          XXXXXXXXXXXXXXXX
        // Tipo J-> opcode        address
        //          XXXXXX XXXXXXXXXXXXXXXXXXXXXXXXXX

        match instr.opcode() {
            Andi => {
                // Tipo I, andi
                let res = self.read_reg_gpr(instr.rs() as usize) & (instr.imm() as u64);
                self.write_reg_gpr(instr.rt() as usize, res);
            }
            Ori => {
                // Tipo I, ori -> instr.imm() OR Registro que dice instr.rs() ->registro que dice instr.rt()
                let res = self.read_reg_gpr(instr.rs() as usize) | (instr.imm() as u64);
                self.write_reg_gpr(instr.rt() as usize, res);
            }
            Lui => {
                // Tipo I, lui, Load Upper Immediate en registro que dice instr.rt() (numero de registro)
                let value = ((instr.imm() << 16) as i32) as u64;
                self.write_reg_gpr(instr.rt() as usize, value);
            }
            Mtc0 => {
                // mtc0: Ejemplo mtc0 t1, $12 _> carga lo que hay en t1 al registro 12 de C0
                let data = self.read_reg_gpr(instr.rt() as usize);
                self.cp0.write_reg(instr.rd(), data);
            }
            Beql => {
                // beql (wrapping_shl hace <<)
                if self.read_reg_gpr(instr.rs() as usize) == self.read_reg_gpr(instr.rt() as usize) {
                    //let offset = imm;
                    let sign_extended_offset =
                        ((instr.offset() as i16) as u64).wrapping_shl(2);
                    self.reg_pc = self.reg_pc.wrapping_add(sign_extended_offset);

                    // TODO: Split into its own function
                    // TODO: Don't do delay slot this way :)
                    self.run_instruction();
                }
            }
            Lw => {
                // Tipo I, lw // instr.rs() = base añadida al offset => dirección virtual hacia reg rt
                let base = instr.rs();
                //let offset = instr.imm();

                let sign_extended_offset = (instr.offset() as i16) as u64;
                let virt_addr =
                    self.read_reg_gpr(base as usize).wrapping_add(sign_extended_offset);
                let mem = (self.read_word(virt_addr) as i32) as u64;
                self.write_reg_gpr(instr.rt() as usize, mem);
            }

            //_ => panic!("Unrecognized instruction: {:#x}", instruction)
        }
        self.reg_pc += 4;
    }

    fn read_word(&self, virt_addr: u64) -> u32 {
        let phys_addr = self.virt_addr_to_phys_addr(virt_addr);
        // TODO: Check endianness
        self.interconnect.read_word(phys_addr as u32)
    }


    fn virt_addr_to_phys_addr(&self, virt_addr: u64) -> u64 {
        // See Table 5-3 in the VR4300 User's Manual
        let addr_bit_values = (virt_addr >> 29) & 0b111;

        if addr_bit_values == 0b101 {
            // kseg1
            virt_addr - 0xffff_ffff_a000_0000
        } else {
            // TODO
            panic!("Unrecognized virtual address: {:#x}", virt_addr);
        }
    }

    // Escribe en el registro general con número index, el valor value
    fn write_reg_gpr(&mut self, index: usize, value: u64) {
        if index != 0 {
            self.reg_gpr[index] = value;
        }
    }

    // Devuelve el valor en el registro general del numero indicado
    fn read_reg_gpr(&self, index: usize) -> u64 {
        match index {
            0 => 0,
            _ => self.reg_gpr[index]
        }
    }
}