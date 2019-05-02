use super::super::interconnect;
use super::cp0;
use super::opcode::Opcode::*;
use super::opcode::SpecialOpcode::*;
use super::opcode::RegImmOpcode::*;
use super::instruction::Instruction;

use std::fmt;


const NUM_GPR: usize = 32; // 32 registros

enum SignExtendResult {
    Yes,
    No,
}

enum WriteLink {
    Yes,
    No,
}

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

    delay_slot_pc: Option<u64>,
}


// CPU
impl Cpu {
    pub fn new(interconnect: interconnect::Interconnect) -> Cpu {
        Cpu {
            reg_gpr: [0; NUM_GPR],   // Registros de proposito general
            reg_fpr: [0.0; NUM_GPR], // Registros de coma flotante
            reg_pc: 0xffff_ffff_bfc0_0000, // Contador de programa TODO: move to const,
            reg_hi: 0,               // Registros de multiplicar y dividie
            reg_lo: 0,
            reg_llbit: false,        // Registro Load/Link
            reg_fcr0: 0,             // Registro Implementacion/revision coma flotante
            reg_fcr31: 0,            // Registro control/status coma flotante
            cp0: cp0::Cp0::default(),

            interconnect: interconnect,

            delay_slot_pc: None,

        }
    }

    pub fn reg_pc(&self) -> u64 {
        self.reg_pc
    }

    pub fn step(&mut self) {
        if let Some(pc) = self.delay_slot_pc {
            let instr = self.read_instruction(pc);
            self.execute_instruction(instr);

            self.delay_slot_pc = None;
        } else {
            let instr = self.read_instruction(self.reg_pc);

            self.reg_pc += 4;
            self.execute_instruction(instr);
        }
    }


    fn read_instruction(&self, addr: u64) -> Instruction {
        Instruction(self.read_word(addr))
    }

    fn execute_instruction(&mut self, instr: Instruction) {
        //        let opcode =  // Opcode 6 bits altos
        //        let rs =  // Registro source
        //        let instr.rt() =  // Registro destino (target)
        //        let imm =  // Valor inmediato

        // instr.rt() => reg source, rd => reg destino, instr.rt() => reg destino/fuente
        // Tipo R-> opcode  rs   instr.rt()   rd   shamt funct
        //          XXXXXX XXXXX   XXXXX     XXXXX XXXXX XXXXXX
        // Tipo I-> opcode  rs    instr.rt()   immediate(imm)
        //          XXXXXX XXXXX   XXXXX      XXXXXXXXXXXXXXXX
        // Tipo J-> opcode        address
        //          XXXXXX XXXXXXXXXXXXXXXXXXXXXXXXXX

        match instr.opcode() {
            Special => {
                match instr.special_op() {
                    Sll => self.reg_instr(instr, |_, rt, sa| rt << sa),

                    Srl =>
                        self.reg_instr(instr, |_, rt, sa| {
                            let rt = rt as u32;
                            (rt >> sa) as u64
                        }),

                    Sllv =>
                        self.reg_instr(instr, |rs, rt, _| {
                            // TODO: Check to see if this is actually sa
                            let shift = rs & 0b11111;
                            rt << shift
                        }),

                    Srlv =>
                        self.reg_instr(instr, |rs, rt, _| {
                            let rs = rs as u32;
                            let rt = rt as u32;
                            let shift = rs & 0b11111;
                            (rt >> shift) as u64
                        }),

                    Jr => {
                        let delay_slot_pc = self.reg_pc;

                        // Update PC before executing delay slot instruction
                        self.reg_pc = self.read_reg_gpr(instr.rs());

                        self.delay_slot_pc = Some(delay_slot_pc);
                    }

                    Multu => {
                        let rs = self.read_reg_gpr(instr.rs()) as u32;
                        let rt = self.read_reg_gpr(instr.rt()) as u32;

                        let res = (rs as u64) * (rt as u64);

                        // TODO: Undefined if last 2 instructions were
                        //  MFHI or MFLO
                        self.reg_lo = (res as i32) as u64;
                        self.reg_hi = ((res >> 32) as i32) as u64;
                    }

                    Mfhi => {
                        let value = self.reg_hi;
                        self.write_reg_gpr(instr.rd() as usize, value)
                    }

                    Mflo => {
                        let value = self.reg_lo;
                        self.write_reg_gpr(instr.rd() as usize, value)
                    }

                    Addu => self.reg_instr(instr, |rs, rt, _| rs.wrapping_add(rt)),
                    Subu => self.reg_instr(instr, |rs, rt, _| rs.wrapping_sub(rt)),

                    And => self.reg_instr(instr, |rs, rt, _| rs & rt),
                    Or => self.reg_instr(instr, |rs, rt, _| rs | rt),
                    Xor => self.reg_instr(instr, |rs, rt, _| rs ^ rt),

                    Sltu => self.reg_instr(instr, |rs, rt, _| if rs < rt { 1 } else { 0 })
                }
            }

            RegImm => {
                match instr.reg_imm_op() {
                    Bgezal => {
                        self.branch(instr, WriteLink::Yes, |rs, _| (rs as i64) >= 0);
                    }
                }
            }

            // Tipo I, R[rt] = R[rs] + SignExtImm
            Addi =>
                self.imm_instr(instr, SignExtendResult::Yes, |rs, _, imm_sign_extended| {
                    // TODO: Handle overflow exception
                    rs + imm_sign_extended
                }),

            // Tipo I, R[rt] = R[rs] + SignExtImm
            Addiu => self.imm_instr(instr, SignExtendResult::Yes, |rs, _, imm_sign_extended| rs.wrapping_add(imm_sign_extended)),

            // Tipo I, R[rt] = R[rs] & ZeroExtImm
            Andi => self.imm_instr(instr, SignExtendResult::No, |rs, imm, _| rs & imm),
            // Tipo I, R[rt] = R[rs] | ZeroExtImm
            Ori => self.imm_instr(instr, SignExtendResult::No, |rs, imm, _| rs | imm),
            // Tipo I, R[rt] = {imm, 16'b0}
            Lui => self.imm_instr(instr, SignExtendResult::Yes, |_, imm, _| imm << 16),
            // mtc0: El contenido del registro rt se carga en el registro rd de CP0
            Mtc0 => {
                let data = self.read_reg_gpr(instr.rt());
                self.cp0.write_reg(instr.rd(), data);
            }

            Beq => { self.branch(instr, WriteLink::No, |rs, rt| rs == rt); }
            Bne => { self.branch(instr, WriteLink::No, |rs, rt| rs != rt); }

            // Tipo I, if(R[rs] == R[rt] -> PC = PC + 4 + BranchAddr
            Beql => self.branch_likely(instr, |rs, rt| rs == rt),
            // Tipo I, if(R[rs] != R[rt] -> PC = PC + 4 + BranchAddr
            Bnel => self.branch_likely(instr, |rs, rt| rs != rt),

            // Tipo I, R[rt] = M[R[rs]+SignExtImm]
            Lw => {
                let base = instr.rs();

                let sign_extended_offset = instr.offset_sign_extended();
                let virt_addr =
                    self.read_reg_gpr(base).wrapping_add(sign_extended_offset);
                let mem = (self.read_word(virt_addr) as i32) as u64;
                self.write_reg_gpr(instr.rt(), mem);
            }


            // Tipo I, M[R[rs]+SignExtImm] = R[rt]
            Sw => {
                let base = instr.rs();

                let sign_extended_offset = instr.offset_sign_extended();
                let virt_addr =
                    self.read_reg_gpr(base).wrapping_add(sign_extended_offset);
                let mem = self.read_reg_gpr(instr.rt()) as u32;
                self.write_word(virt_addr, mem);
            }
        }
    }

    fn imm_instr<F>(&mut self, instr: Instruction, sign_extend_result: SignExtendResult, f: F)
        where F: FnOnce(u64, u64, u64) -> u64 {
        let rs = self.read_reg_gpr(instr.rs());
        let imm = instr.imm() as u64;
        let imm_sign_extended = instr.imm_sign_extended();
        let value = f(rs, imm, imm_sign_extended);
        let sign_extended_value = (value as i32) as u64;
        let value = match sign_extend_result {
            SignExtendResult::Yes => sign_extended_value,
            _ => value
        };
        self.write_reg_gpr(instr.rt(), value);
    }

    fn reg_instr<F>(&mut self, instr: Instruction, f: F) where F: FnOnce(u64, u64, u32) -> u64 {
        let rs = self.read_reg_gpr(instr.rs());
        let rt = self.read_reg_gpr(instr.rt());
        let sa = instr.sa();
        let value = f(rs, rt, sa);
        let sign_extended_value = (value as i32) as u64;
        self.write_reg_gpr(instr.rd() as usize, sign_extended_value);
    }

    fn branch<F>(&mut self, instr: Instruction, write_link: WriteLink, f: F) -> bool where F: FnOnce(u64, u64) -> bool {
        let rs = self.read_reg_gpr(instr.rs());
        let rt = self.read_reg_gpr(instr.rt());
        let is_taken = f(rs, rt);

        let delay_slot_pc = self.reg_pc;

        if let WriteLink::Yes = write_link {
            let link_address = delay_slot_pc + 4;
            self.write_reg_gpr(31, link_address);
        }

        if is_taken {
            let sign_extended_offset = instr.offset_sign_extended() << 2;
            // Update PC before executing delay slot instruction
            self.reg_pc = self.reg_pc.wrapping_add(sign_extended_offset);

            self.delay_slot_pc = Some(delay_slot_pc);
        }

        is_taken
    }

    fn branch_likely<F>(&mut self, instr: Instruction, f: F) where F: FnOnce(u64, u64) -> bool {
        if !self.branch(instr, WriteLink::No, f) {
            // Skip over delay slot instruction when not branching
            self.reg_pc = self.reg_pc.wrapping_add(4);
        }
    }


    fn store_delay_slot_pc(&mut self, delay_slot_pc: u64) {
        ;
    }

    fn read_word(&self, virt_addr: u64) -> u32 {
        let phys_addr = self.virt_addr_to_phys_addr(virt_addr);
        self.interconnect.read_word(phys_addr as u32)
    }

    fn write_word(&mut self, virt_addr: u64, value: u32) {
        let phys_addr = self.virt_addr_to_phys_addr(virt_addr);
        self.interconnect.write_word(phys_addr as u32, value);
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

    fn write_reg_gpr(&mut self, index: usize, value: u64) {
        if index != 0 {
            self.reg_gpr[index] = value;
        }
    }

    fn read_reg_gpr(&self, index: usize) -> u64 {
        match index {
            0 => 0,
            _ => self.reg_gpr[index]
        }
    }
}

//// Para presentar adecuadamente los registros, cambio el macro try! por ?
impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const REGS_PER_LINE: usize = 2;
        const REG_NAMES: [&'static str; NUM_GPR] = [
            "r0", "at", "v0", "v1", "a0", "a1", "a2", "a3",
            "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7",
            "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7",
            "t8", "t9", "k0", "k1", "gp", "sp", "s8", "ra",
        ];

        write!(f, "\nCPU General Purpose Registers:")?;
        for reg_num in 0..NUM_GPR {
            if (reg_num % REGS_PER_LINE) == 0 {
                writeln!(f, "")?;
            }
            write!(f,
                   "{reg_name}/gpr{num:02}: {value:#018X} ",
                   num = reg_num,
                   reg_name = REG_NAMES[reg_num],
                   value = self.reg_gpr[reg_num],
            )?;
        }

        write!(f, "\n\nCPU Floating Point Registers:")?;
        for reg_num in 0..NUM_GPR {
            if (reg_num % REGS_PER_LINE) == 0 {
                writeln!(f, "")?;
            }
            write!(f,
                   "fpr{num:02}: {value:21} ",
                   num = reg_num,
                   value = self.reg_fpr[reg_num], )?;
        }

        writeln!(f, "\n\nCPU Special Registers:")?;
        writeln!(f,
                 "\
            reg_pc: {:#018X}\n\
            reg_hi: {:#018X}\n\
            reg_lo: {:#018X}\n\
            reg_llbit: {}\n\
            reg_fcr0:  {:#010X}\n\
            reg_fcr31: {:#010X}\n\
            ",
                 self.reg_pc,
                 self.reg_hi,
                 self.reg_lo,
                 self.reg_llbit,
                 self.reg_fcr0,
                 self.reg_fcr31
        )?;

        writeln!(f, "{:#?}", self.cp0)?;
        writeln!(f, "{:#?}", self.interconnect)
    }
}