mod tables;
use crate::tables::{BP, BX, CX, DI, SI};
use std::{path::PathBuf, str::FromStr};
use tables::{Registers, REGISTER_TABLE, WIDE_REGISTER_TABLE, ZERO_FLAG};

// A mb of memory
static mut MEM: [u8; 1024 * 1024] = [0; 1024 * 1024];

// OPs
const MOV: u8 = 0b10_0010;
const IMMEDIATE_TO_REG_OR_MEM: u8 = 0b0110_0011;
const IMMEDITAE_TO_REG: u8 = 0b1011;
const MEM_TO_ACC: u8 = 0b101;
const ACC_TO_MEM: u8 = 0b0101_0001;
const ADD: u8 = 0b0000_0000;

// d
const REG_IS_DEST: u8 = 0b0000_0010;

// w
const WIDE: u8 = 0b0000_0001;

// mod
const MEM_MODE: u8 = 0b0000_0000;
const MEM_MODE_BYTE_DIS: u8 = 0b0100_0000;
const MEM_MODE_WORD_DIS: u8 = 0b1000_0000;
const REG_MODE: u8 = 0b1100_0000;

// Given a the r/m field value, gives the string instruction
// and the actual computed values of register/s
fn rm_to_rg(rm: u8) -> (String, u16) {
    match rm {
        0b0000_0000 => (
            "[bx + si".into(),
            WIDE_REGISTER_TABLE.get(&BX).unwrap().get_value()
                + WIDE_REGISTER_TABLE.get(&SI).unwrap().get_value(),
        ),
        0b0000_0001 => (
            "[bx + di".into(),
            WIDE_REGISTER_TABLE.get(&BX).unwrap().get_value()
                + WIDE_REGISTER_TABLE.get(&DI).unwrap().get_value(),
        ),
        0b0000_0010 => (
            "[bp + si".into(),
            WIDE_REGISTER_TABLE.get(&BP).unwrap().get_value()
                + WIDE_REGISTER_TABLE.get(&SI).unwrap().get_value(),
        ),
        0b0000_0011 => (
            "[bp + di".into(),
            WIDE_REGISTER_TABLE.get(&BP).unwrap().get_value()
                + WIDE_REGISTER_TABLE.get(&DI).unwrap().get_value(),
        ),
        0b0000_0100 => (
            "[si".into(),
            WIDE_REGISTER_TABLE.get(&SI).unwrap().get_value(),
        ),
        0b0000_0101 => (
            "[di".into(),
            WIDE_REGISTER_TABLE.get(&DI).unwrap().get_value(),
        ),
        // direction address, with potential offset!
        0b0000_0110 => (
            "[bp".into(),
            WIDE_REGISTER_TABLE.get(&BP).unwrap().get_value(),
        ),
        0b0000_0111 => (
            "[bx".into(),
            WIDE_REGISTER_TABLE.get(&BX).unwrap().get_value(),
        ),
        _ => panic!("invalid rm"),
    }
}

fn get_displacement_byte(dis: i8) -> String {
    let mut buffer = String::new();
    if dis.is_negative() {
        buffer.push_str(" - ");
    } else {
        buffer.push_str(" + ");
    }
    buffer.push_str(&format!("{}", dis.abs()));

    buffer
}

fn get_displacement_word(dis: [u8; 2]) -> String {
    let mut buffer = String::new();
    let dis = i16::from_le_bytes(dis);
    if dis.is_negative() {
        buffer.push_str(" - ");
    } else {
        buffer.push_str(" + ");
    }
    buffer.push_str(&format!("{}", dis.abs()));

    buffer
}

enum Mode {
    Mem,
    MemByteDis,
    MemWordDis,
    Reg,
}

impl From<u8> for Mode {
    fn from(value: u8) -> Self {
        if value & REG_MODE == REG_MODE {
            return Mode::Reg;
        }
        if value & MEM_MODE_WORD_DIS == MEM_MODE_WORD_DIS {
            return Mode::MemWordDis;
        }
        if value & MEM_MODE_BYTE_DIS == MEM_MODE_BYTE_DIS {
            return Mode::MemByteDis;
        }
        if value & MEM_MODE == MEM_MODE {
            return Mode::Mem;
        }
        panic!("invald mode")
    }
}

fn mem_mode(buffer: &[u8], bp: &mut usize, buffer_out: &mut String) {
    let reg_is_dest = buffer[*bp] & REG_IS_DEST == REG_IS_DEST;
    let is_wide = buffer[*bp] & WIDE == WIDE;
    let reg = (buffer[*bp + 1] & 0b0011_1000) >> 3;
    let reg_mem = buffer[*bp + 1] & 0b0000_0111;
    if is_wide {
        if reg_is_dest {
            let dest = WIDE_REGISTER_TABLE.get(&reg).unwrap();
            let memory_location = rm_to_rg(reg_mem);
            buffer_out.push_str(&format!("{dest}, "));
            if reg_mem == 0b0000_0110 {
                let mem_location = u16::from_le_bytes([buffer[*bp + 2], buffer[*bp + 3]]);
                buffer_out.push_str(&format!("[{mem_location}"));
                unsafe {
                    let value = u16::from_le_bytes([
                        MEM[mem_location as usize],
                        MEM[(mem_location + 1) as usize],
                    ]);
                    dest.update_wide(value);
                }
                *bp += 4;
            } else {
                buffer_out.push_str(&memory_location.0);
                *bp += 2;
            }
            unsafe {
                let value = u16::from_le_bytes([
                    MEM[memory_location.1 as usize],
                    MEM[(memory_location.1 + 1) as usize],
                ]);
                dest.update_wide(value);
                buffer_out.push_str(&format!("] => {} = {value}", dest.to_string()));
            }
        } else {
            // special case for direct address if reg is not the dest
            // 16 bit displacement follows
            let memory_location = rm_to_rg(reg_mem);
            if reg_mem == 0b0000_0110 {
                buffer_out.push_str(&format!(
                    "[{}",
                    u16::from_le_bytes([buffer[*bp + 2], buffer[*bp + 3]])
                ));
                *bp += 4;
            } else {
                buffer_out.push_str(&memory_location.0);
                *bp += 2;
            }
            buffer_out.push_str("], ");
            let register = &WIDE_REGISTER_TABLE.get(&reg).unwrap();
            let value = register.get_value().to_le_bytes();
            unsafe {
                MEM[memory_location.1 as usize] = value[0];
                MEM[(memory_location.1 + 1) as usize] = value[1];
            }
            buffer_out.push_str(&format!("{register} => MEM{}", memory_location.0));
        }
    } else {
        if reg_is_dest {
            buffer_out.push_str(&REGISTER_TABLE.get(&reg).unwrap().to_string());
            buffer_out.push_str(", ");
            // special case for direct address
            if reg_mem == 0b0000_0110 {
                buffer_out.push_str(&format!("[{reg_mem:0}"));
            } else {
                buffer_out.push_str(&rm_to_rg(reg_mem).0);
            }
            buffer_out.push(']');
        } else {
            // special case for direct address
            if reg_mem == 0b0000_0110 {
                buffer_out.push_str(&format!("[{reg_mem:0}"));
            } else {
                buffer_out.push_str(&rm_to_rg(reg_mem).0);
            }
            buffer_out.push_str("], ");
            buffer_out.push_str(&REGISTER_TABLE.get(&reg).unwrap().to_string());
        }
        *bp += 2;
    }
}

fn mem_mode_word_dis(buffer: &[u8], bp: &mut usize, buffer_out: &mut String) {
    let reg_is_dest = buffer[*bp] & REG_IS_DEST == REG_IS_DEST;
    let is_wide = buffer[*bp] & WIDE == WIDE;
    let reg = (buffer[*bp + 1] & 0b0011_1000) >> 3;
    let reg_mem = buffer[*bp + 1] & 0b0000_0111;
    if is_wide {
        if reg_is_dest {
            buffer_out.push_str(&WIDE_REGISTER_TABLE.get(&reg).unwrap().to_string());
            buffer_out.push_str(", ");
            buffer_out.push_str(&rm_to_rg(reg_mem).0);
            buffer_out.push_str(&get_displacement_word([buffer[*bp + 2], buffer[*bp + 3]]));
            buffer_out.push(']');
        } else {
            buffer_out.push_str(&rm_to_rg(reg_mem).0);
            buffer_out.push_str(&get_displacement_word([buffer[*bp + 2], buffer[*bp + 3]]));
            buffer_out.push_str("], ");
            buffer_out.push_str(&WIDE_REGISTER_TABLE.get(&reg).unwrap().to_string());
        }
    } else if reg_is_dest {
        buffer_out.push_str(&REGISTER_TABLE.get(&reg).unwrap().to_string());
        buffer_out.push_str(", ");
        buffer_out.push_str(&rm_to_rg(reg_mem).0);
        buffer_out.push_str(&get_displacement_word([buffer[*bp + 2], buffer[*bp + 3]]));
        buffer_out.push(']');
    } else {
        buffer_out.push_str(&rm_to_rg(reg_mem).0);
        buffer_out.push_str(&get_displacement_word([buffer[*bp + 2], buffer[*bp + 3]]));
        buffer_out.push_str("], ");
        buffer_out.push_str(&REGISTER_TABLE.get(&reg).unwrap().to_string());
    }
    *bp += 4;
}

fn mem_mode_byte_dis(buffer: &[u8], bp: &mut usize, buffer_out: &mut String) {
    let reg_is_dest = buffer[*bp] & REG_IS_DEST == REG_IS_DEST;
    let is_wide = buffer[*bp] & WIDE == WIDE;
    let reg = (buffer[*bp + 1] & 0b0011_1000) >> 3;
    let reg_mem = buffer[*bp + 1] & 0b0000_0111;
    if is_wide {
        if reg_is_dest {
            buffer_out.push_str(&WIDE_REGISTER_TABLE.get(&reg).unwrap().to_string());
            buffer_out.push_str(", ");
            buffer_out.push_str(&rm_to_rg(reg_mem).0);
            if buffer[*bp + 2] != 0 {
                buffer_out.push_str(&get_displacement_byte(buffer[*bp + 2] as i8));
            }
            buffer_out.push(']');
        } else {
            let register = rm_to_rg(reg_mem);
            buffer_out.push_str(&register.0);
            buffer_out.push_str(&get_displacement_byte(buffer[*bp + 2] as i8));
            buffer_out.push_str("], ");
            let r = WIDE_REGISTER_TABLE.get(&reg).unwrap();
            buffer_out.push_str(&r.to_string());
            let location = register.1 + buffer[*bp + 2] as u16;
            let value = r.get_value().to_le_bytes();
            unsafe {
                MEM[location as usize] = value[0];
                MEM[(location + 1) as usize] = value[1];
            }
        }
    } else if reg_is_dest {
        buffer_out.push_str(&REGISTER_TABLE.get(&reg).unwrap().to_string());
        buffer_out.push_str(", ");
        buffer_out.push_str(&rm_to_rg(reg_mem).0);
        if buffer[*bp + 2] != 0 {
            buffer_out.push_str(&get_displacement_byte(buffer[*bp + 2] as i8));
        }
        buffer_out.push(']');
    } else {
        buffer_out.push_str(&rm_to_rg(reg_mem).0);
        if buffer[*bp + 2] != 0 {
            buffer_out.push_str(&get_displacement_byte(buffer[*bp + 2] as i8));
        }
        buffer_out.push_str("], ");
        buffer_out.push_str(&REGISTER_TABLE.get(&reg).unwrap().to_string());
    }
    *bp += 3;
}

fn mov(destination: &Registers, value: u16, buffer_out: &mut String) {
    destination.update_wide(value);

    buffer_out.push_str(&format!(
        "{}, {} => {}",
        destination,
        value,
        destination.updated_value()
    ));
}

fn add_wide(destination: &Registers, value: u16, buffer_out: &mut String) {
    destination.add_wide(value);

    buffer_out.push_str(&format!(
        "{}, {} => {}",
        destination,
        value,
        destination.updated_value()
    ));
}

fn sub(destination: &Registers, value: u16, buffer_out: &mut String) {
    destination.sub_wide(value);

    buffer_out.push_str(&format!(
        "{}, {} => {}",
        destination,
        value,
        destination.updated_value()
    ));
}

fn cmp(destination: &Registers, value: u16, buffer_out: &mut String) {
    destination.cmp(value);

    buffer_out.push_str(&format!(
        "{}, {} => {}",
        destination,
        value,
        destination.updated_value()
    ));
}

fn reg_auto<F>(f: F, buffer: &[u8], bp: &mut usize, buffer_out: &mut String)
where
    F: Fn(&Registers, u16, &mut String),
{
    let reg_is_dest = buffer[*bp] & REG_IS_DEST == REG_IS_DEST;
    let is_wide = buffer[*bp] & WIDE == WIDE;
    let reg = (buffer[*bp + 1] & 0b0011_1000) >> 3;
    let reg_mem = buffer[*bp + 1] & 0b0000_0111;
    if is_wide {
        if reg_is_dest {
            f(
                WIDE_REGISTER_TABLE.get(&reg).unwrap(),
                WIDE_REGISTER_TABLE.get(&reg_mem).unwrap().get_value(),
                buffer_out,
            );
        } else {
            f(
                WIDE_REGISTER_TABLE.get(&reg_mem).unwrap(),
                WIDE_REGISTER_TABLE.get(&reg).unwrap().get_value(),
                buffer_out,
            )
        }
    }
}

fn reg_auto_immediate<F>(f: F, buffer: &[u8], bp: &mut usize, buffer_out: &mut String, value: u16)
where
    F: Fn(&Registers, u16, &mut String),
{
    let is_wide = buffer[*bp] & WIDE == WIDE;
    let reg_mem = buffer[*bp + 1] & 0b0000_0111;
    if is_wide {
        f(
            WIDE_REGISTER_TABLE.get(&reg_mem).unwrap(),
            value,
            buffer_out,
        );
    }
}

pub fn disassemble(buffer: Vec<u8>, is_executing: bool, is_dumping: bool) -> String {
    let mut buffer_out = String::from("bits 16 \n\n");

    // buffer pointer.
    let mut bp = 0;

    while bp < buffer.len() {
        if buffer[bp] >> 2 == MOV {
            buffer_out.push_str("mov ");

            if buffer[bp + 1] & REG_MODE == REG_MODE {
                reg_auto(mov, &buffer, &mut bp, &mut buffer_out);
                bp += 2;
            } else if buffer[bp + 1] & MEM_MODE_BYTE_DIS == MEM_MODE_BYTE_DIS {
                mem_mode_byte_dis(&buffer, &mut bp, &mut buffer_out);
            } else if buffer[bp + 1] & MEM_MODE_WORD_DIS == MEM_MODE_WORD_DIS {
                mem_mode_word_dis(&buffer, &mut bp, &mut buffer_out);
            }
            // this must be last!!
            else if buffer[bp + 1] & MEM_MODE == MEM_MODE {
                mem_mode(&buffer, &mut bp, &mut buffer_out);
            }
        } else if buffer[bp] >> 1 == IMMEDIATE_TO_REG_OR_MEM {
            buffer_out.push_str("mov ");
            let is_wide = buffer[bp] & 1 == 1;
            let reg_mem = buffer[bp + 1] & 0b0000_0111;
            if buffer[bp + 1] & MEM_MODE_BYTE_DIS == MEM_MODE_BYTE_DIS {
                if is_wide {
                    let register = rm_to_rg(reg_mem);
                    let displacement = buffer[bp + 2];
                    let value = u16::from_le_bytes([buffer[bp + 3], buffer[bp + 4]]);
                    buffer_out.push_str(&format!(
                        "mov word {}+ {displacement}], {value}",
                        register.0
                    ));
                    let actaul_displacement = register.1 + displacement as u16;
                    unsafe {
                        MEM[actaul_displacement as usize] = buffer[bp + 3];
                        MEM[(actaul_displacement + 1) as usize] = buffer[bp + 4];
                    }
                    bp += 5;
                } else {
                    let register = rm_to_rg(reg_mem);
                    let displacement = buffer[bp + 2];
                    let value = buffer[bp + 3];
                    buffer_out.push_str(&format!("word {}+ {displacement}], {value}", register.0));
                    let actaul_displacement = register.1 + displacement as u16;
                    unsafe {
                        MEM[actaul_displacement as usize] = buffer[bp + 3];
                    }
                    bp += 4;
                }
            } else if buffer[bp + 1] & MEM_MODE_WORD_DIS == MEM_MODE_WORD_DIS {
                if is_wide {
                    buffer_out.push_str(&rm_to_rg(reg_mem).0);
                    buffer_out.push_str(&get_displacement_word([buffer[bp + 2], buffer[bp + 3]]));
                    buffer_out.push_str(&format!(
                        "], word {}",
                        i16::from_le_bytes([buffer[bp + 4], buffer[bp + 5]])
                    ));
                    bp += 6;
                } else {
                    buffer_out.push_str(&rm_to_rg(reg_mem).0);
                    buffer_out.push_str(&get_displacement_word([buffer[bp + 2], buffer[bp + 3]]));
                    buffer_out.push_str(&format!("], byte {}", buffer[bp + 4]));
                    bp += 5;
                }
            } else if buffer[bp + 1] & MEM_MODE == MEM_MODE {
                if is_wide {
                    // special case for direct address if reg is not the dest
                    // 16 bit displacement follows
                    if reg_mem == 0b0000_0110 {
                        let address = u16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]]);
                        let value = i16::from_le_bytes([buffer[bp + 4], buffer[bp + 5]]);
                        buffer_out.push_str(&format!("word [{}] {}", address, value));
                        unsafe {
                            MEM[address as usize] = buffer[bp + 4];
                            MEM[(address + 1) as usize] = buffer[bp + 5];
                        }
                        bp += 6;
                    } else {
                        buffer_out.push_str(&rm_to_rg(reg_mem).0);
                        buffer_out.push_str(&format!(
                            "], word {}",
                            i16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]])
                        ));
                        bp += 4;
                    }
                } else {
                    // special case for direct address
                    if reg_mem == 0b0000_0110 {
                        buffer_out.push_str(&format!(
                            "[{}",
                            u16::from_le_bytes([reg_mem, buffer[bp + 2]])
                        ));
                        bp += 4;
                    } else {
                        buffer_out.push_str(&rm_to_rg(reg_mem).0);
                    }
                    buffer_out.push_str(&format!("], byte {}", buffer[bp + 2]));
                    bp += 3;
                }
            }
        } else if buffer[bp] >> 4 == IMMEDITAE_TO_REG {
            buffer_out.push_str("mov ");

            // wide is different in immediate to register
            let is_wide = buffer[bp] & 0b0000_1000 == 0b0000_1000;

            let reg = buffer[bp] & 0b0000_0111;

            if is_wide {
                let destination = WIDE_REGISTER_TABLE.get(&reg).unwrap();
                let value = u16::from_le_bytes([buffer[bp + 1], buffer[bp + 2]]);

                destination.update_wide(value);

                buffer_out.push_str(&format!(
                    "{destination}, {value} => {}",
                    destination.updated_value()
                ));

                bp += 3;
            } else {
                let destination = REGISTER_TABLE.get(&reg).unwrap();
                let value = buffer[bp + 1] as u8;

                destination.update(value);

                buffer_out.push_str(&format!(
                    "{destination}, {value} => {}",
                    destination.updated_value()
                ));

                bp += 2;
            }
        } else if buffer[bp] >> 1 == ACC_TO_MEM {
            buffer_out.push_str("mov ");
            // if it's wide
            if buffer[bp] & 1 == 1 {
                buffer_out.push_str(&format!(
                    "[{}], ax",
                    u16::from_le_bytes([buffer[bp + 1], buffer[bp + 2]])
                ));
                bp += 3;
            } else {
                buffer_out.push_str(&format!("[{}], al", buffer[bp + 1]));
                bp += 2;
            }
        } else if buffer[bp] >> 5 == MEM_TO_ACC {
            // if it's wide
            if buffer[bp] & 1 == 1 {
                buffer_out.push_str(&format!(
                    "mov ax, [{}]",
                    u16::from_le_bytes([buffer[bp + 1], buffer[bp + 2]])
                ));
                bp += 3;
            } else {
                buffer_out.push_str(&format!("mov al, [{}]", buffer[bp + 1]));
                bp += 2;
            }
        } else if buffer[bp] >> 2 == 0b0
            || buffer[bp] >> 2 == 0b001010
            || buffer[bp] >> 2 == 0b001110
        {
            let mode = Mode::from(buffer[bp + 1]);

            // SUB
            if buffer[bp] >> 2 == 0b001010 {
                buffer_out.push_str("sub ");
                match mode {
                    Mode::Mem => {
                        mem_mode(&buffer, &mut bp, &mut buffer_out);
                    }
                    Mode::MemByteDis => {
                        mem_mode_byte_dis(&buffer, &mut bp, &mut buffer_out);
                    }
                    Mode::MemWordDis => {
                        mem_mode_word_dis(&buffer, &mut bp, &mut buffer_out);
                    }
                    Mode::Reg => {
                        reg_auto(sub, &buffer, &mut bp, &mut buffer_out);
                    }
                }
            }
            // CMP
            else if buffer[bp] >> 2 == 0b001110 {
                buffer_out.push_str("cmp ");
                match mode {
                    Mode::Mem => {
                        mem_mode(&buffer, &mut bp, &mut buffer_out);
                    }
                    Mode::MemByteDis => {
                        mem_mode_byte_dis(&buffer, &mut bp, &mut buffer_out);
                    }
                    Mode::MemWordDis => {
                        mem_mode_word_dis(&buffer, &mut bp, &mut buffer_out);
                    }
                    Mode::Reg => {
                        reg_auto(cmp, &buffer, &mut bp, &mut buffer_out);
                    }
                }
            }
            // ADD - Must be last! Always true essentially
            else if buffer[bp] & ADD == ADD {
                buffer_out.push_str("add ");
                match mode {
                    Mode::Mem => {
                        mem_mode(&buffer, &mut bp, &mut buffer_out);
                    }
                    Mode::MemByteDis => {
                        mem_mode_byte_dis(&buffer, &mut bp, &mut buffer_out);
                    }
                    Mode::MemWordDis => {
                        mem_mode_word_dis(&buffer, &mut bp, &mut buffer_out);
                    }
                    Mode::Reg => {
                        reg_auto(add_wide, &buffer, &mut bp, &mut buffer_out);
                    }
                }
            }
            bp += 2;
        }
        // Immediate to reg/mem
        else if buffer[bp] >> 2 == 0b0010_0000 {
            let value = if buffer[bp] & 0b0000_0011 == 0b01 {
                u16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]])
            } else {
                buffer[bp + 2] as u16
            };

            if buffer[bp + 1] >> 3 & 0b00111 == 0 {
                buffer_out.push_str("add ");
                reg_auto_immediate(add_wide, &buffer, &mut bp, &mut buffer_out, value);
            }
            if buffer[bp + 1] >> 3 & 0b00111 == 0b00101 {
                buffer_out.push_str("sub ");
                reg_auto_immediate(sub, &buffer, &mut bp, &mut buffer_out, value);
            }
            if buffer[bp + 1] >> 3 & 0b00111 == 0b00111 {
                buffer_out.push_str("cmp ");
            }
            let is_wide = buffer[bp] & 1 == 1;
            let is_signed = buffer[bp] & 0b0000_0010 == 0b0000_0010;
            let reg_mem = buffer[bp + 1] & 0b0000_0111;
            match Mode::from(buffer[bp + 1]) {
                Mode::MemWordDis => {
                    if is_wide {
                        if is_signed {
                            buffer_out.push_str("word ");
                            buffer_out.push_str(&rm_to_rg(reg_mem).0);
                            buffer_out
                                .push_str(&get_displacement_word([buffer[bp + 2], buffer[bp + 3]]));
                            buffer_out.push_str(&format!("], {}", buffer[bp + 4]));
                            bp += 5;
                        } else {
                            buffer_out.push_str("word ");
                            buffer_out.push_str(&rm_to_rg(reg_mem).0);
                            buffer_out
                                .push_str(&get_displacement_word([buffer[bp + 2], buffer[bp + 3]]));
                            buffer_out.push_str(&format!(
                                "], {}",
                                u16::from_le_bytes([buffer[bp + 4], buffer[bp + 5]])
                            ));
                            bp += 6;
                        }
                    } else {
                        // TODO: add signed variants
                        buffer_out.push_str("byte ");
                        buffer_out.push_str(&rm_to_rg(reg_mem).0);
                        buffer_out
                            .push_str(&get_displacement_word([buffer[bp + 2], buffer[bp + 3]]));
                        buffer_out.push_str(&format!("], byte {}", buffer[bp + 4]));
                        bp += 5;
                    }
                }
                Mode::MemByteDis => {
                    if is_wide {
                        buffer_out.push_str(&rm_to_rg(reg_mem).0);
                        buffer_out.push_str(&get_displacement_byte(buffer[bp + 2] as i8));
                        buffer_out.push_str(&format!(
                            "], word {}",
                            i16::from_le_bytes([buffer[bp + 4], buffer[bp + 5]])
                        ));
                        bp += 5;
                    } else {
                        buffer_out.push_str(&rm_to_rg(reg_mem).0);
                        buffer_out.push_str(&get_displacement_byte(buffer[bp + 2] as i8));
                        buffer_out.push_str(&format!("], byte {}", buffer[bp + 3]));
                        bp += 4;
                    }
                }
                Mode::Reg => {
                    let is_wide = buffer[bp] & WIDE == WIDE;
                    //let reg_mem = buffer[bp + 1] & 0b0000_0111;
                    if is_wide {
                        if is_signed {
                            cmp(
                                &WIDE_REGISTER_TABLE.get(&reg_mem).unwrap(),
                                buffer[bp + 2] as u16,
                                &mut buffer_out,
                            );
                            //buffer_out
                            //    .push_str(&WIDE_REGISTER_TABLE.get(&reg_mem).unwrap().to_string());
                            //buffer_out.push_str(", ");
                            //buffer_out.push_str(&format!("{}", buffer[bp + 2]));
                            bp += 3;
                        } else {
                            //buffer_out
                            //    .push_str(&WIDE_REGISTER_TABLE.get(&reg_mem).unwrap().to_string());
                            //buffer_out.push_str(", ");
                            //buffer_out.push_str(&format!(
                            //    "{}",
                            //    u16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]])
                            //));
                            bp += 4;
                        }
                    } else {
                        bp += 2;
                    }
                    // Advance two to account for the OP and register bytes
                }
                Mode::Mem => {
                    let is_wide = buffer[bp] & WIDE == WIDE;
                    let reg_mem = buffer[bp + 1] & 0b0000_0111;
                    if is_wide {
                        if is_signed {
                            buffer_out.push_str("word ");
                            if reg_mem == 0b0000_0110 {
                                buffer_out.push_str(&format!(
                                    "[{}], ",
                                    u16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]])
                                ));
                                buffer_out.push_str(&format!("{}", buffer[bp + 4]));
                                bp += 5;
                            } else {
                                buffer_out.push_str(&rm_to_rg(reg_mem).0);
                                buffer_out.push_str(&format!("], {}", buffer[bp + 2]));
                                bp += 3;
                            }
                        } else {
                            buffer_out.push_str("word ");
                            // special case for direct address if reg is not the dest
                            // 16 bit displacement follows
                            if reg_mem == 0b0000_0110 {
                                buffer_out.push_str(&format!(
                                    "[{}",
                                    u16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]])
                                ));
                                buffer_out.push_str(&format!(
                                    "{}",
                                    u16::from_le_bytes([buffer[bp + 4], buffer[bp + 5]])
                                ));
                                bp += 6;
                            } else {
                                buffer_out.push_str(&rm_to_rg(reg_mem).0);
                                buffer_out.push_str(&format!(
                                    "{}",
                                    u16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]])
                                ));
                                bp += 4;
                            }
                        }
                    } else if is_signed {
                        buffer_out.push_str("byte ");
                        // special case for direct address
                        if reg_mem == 0b0000_0110 {
                            buffer_out.push_str(&format!("[{reg_mem:0}"));
                        } else {
                            buffer_out.push_str(&rm_to_rg(reg_mem).0);
                        }
                        buffer_out.push_str(&format!(", {}", buffer[bp + 2] as i8));
                        bp += 3;
                    } else {
                        buffer_out.push_str("byte ");
                        // special case for direct address
                        if reg_mem == 0b0000_0110 {
                            buffer_out.push_str(&format!("[{reg_mem:0}"));
                        } else {
                            buffer_out.push_str(&rm_to_rg(reg_mem).0);
                        }
                        buffer_out.push_str(&format!("], {}", buffer[bp + 2]));
                        bp += 3;
                    }
                }
            }
        }
        // Immediate to acc
        else if buffer[bp] >> 1 == 0b0000_0010
            || buffer[bp] >> 1 == 0b0000_10110
            || buffer[bp] >> 1 == 0b0000_11110
        {
            if buffer[bp] >> 1 == 0b0000_0010 {
                buffer_out.push_str("add ");
            }
            if buffer[bp] >> 1 == 0b0001_0110 {
                buffer_out.push_str("sub ");
            }
            if buffer[bp] >> 1 == 0b0001_1110 {
                buffer_out.push_str("cmp ");
            }
            let is_wide = buffer[bp] & 1 == 1;
            if is_wide {
                buffer_out.push_str(&format!(
                    "ax, {}",
                    i16::from_le_bytes([buffer[bp + 1], buffer[bp + 2]])
                ));
                bp += 3;
            } else {
                buffer_out.push_str(&format!("al, {}", buffer[bp + 1] as i8));
                bp += 2;
            }
        } else if buffer[bp] >> 1 == 0b0001_0110 {
            buffer_out.push_str("sub ");
            let is_wide = buffer[bp] & 1 == 1;
            if is_wide {
                buffer_out.push_str(&format!(
                    "ax, {}",
                    i16::from_le_bytes([buffer[bp + 1], buffer[bp + 2]])
                ));
                bp += 3;
            } else {
                buffer_out.push_str(&format!("al, {}", buffer[bp + 1] as i8));
                bp += 2;
            }
        }
        // JNE - jump not equal
        else if buffer[bp] == 0b0111_0101 {
            buffer_out.push_str(&format!("jne {}", buffer[bp + 1] as i8 + 2));
            ZERO_FLAG.with(|flag| {
                if !*flag.borrow() {
                    let jump_value = buffer[bp + 1] as i8;
                    if jump_value.is_positive() {
                        // probably not going to happen
                        bp += jump_value as usize;
                    } else {
                        bp -= jump_value.abs() as usize;
                    }
                    bp += 2;
                } else {
                    bp += 2;
                }
            })
        }
        // JE - jump equal
        else if buffer[bp] == 0b0111_0100 {
            buffer_out.push_str(&format!("je {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1100 {
            buffer_out.push_str(&format!("jl {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1110 {
            buffer_out.push_str(&format!("jle {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0010 {
            buffer_out.push_str(&format!("jb {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0110 {
            buffer_out.push_str(&format!("jbe {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b111_1010 {
            buffer_out.push_str(&format!("jp {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0000 {
            buffer_out.push_str(&format!("jo {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1000 {
            buffer_out.push_str(&format!("js {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1101 {
            buffer_out.push_str(&format!("jnl {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1111 {
            buffer_out.push_str(&format!("jnle {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0011 {
            buffer_out.push_str(&format!("jnb {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0111 {
            buffer_out.push_str(&format!("jnbe {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1011 {
            buffer_out.push_str(&format!("jnp {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0001 {
            buffer_out.push_str(&format!("jno {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1001 {
            buffer_out.push_str(&format!("jns {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b1110_0010 {
            buffer_out.push_str(&format!("loop {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b1110_0001 {
            buffer_out.push_str(&format!("loopz {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b1110_0000 {
            buffer_out.push_str(&format!("loopnz {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b1110_0011 {
            buffer_out.push_str(&format!("jcxz {}", buffer[bp + 1] as i8));
            bp += 2;
        } else {
            // while developing to prevent endless looping when nothing matches
            bp += 2;
        }
        buffer_out.push('\n');
    }

    buffer_out.push_str(&format!("ip: {bp}"));

    if is_executing {
        Registers::print();
    }

    if is_dumping {
        unsafe {
            let _ = std::fs::write(PathBuf::from_str("sim86_memory.data").unwrap(), MEM);
        }
    }

    buffer_out
}
