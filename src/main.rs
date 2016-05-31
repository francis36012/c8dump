#[macro_use]
extern crate clap;

use std::io::Read;
use std::io::BufReader;
use std::io::Write;
use std::io::BufWriter;
use std::fs::File;
use std::path::Path;

use clap::{Arg, App};

fn main() {
    let matches = App::new("c8dump")
        .version(crate_version!())
        .author("Francis A. <francisagyapong2@gmail.com>")
        .about("A very simple Chip-8 binary disassembler")
        .arg(Arg::with_name("input")
             .short("i")
             .long("input")
             .value_name("FILE")
             .help("The binary file to disassemble")
             .required(true)).get_matches();

    let input_file_path = Path::new(matches.value_of("input").unwrap());
    let input_file = File::open(input_file_path).unwrap();
    disassemble(input_file, std::io::stdout());
}


enum DissasembleError {
    InputError(String),
    OutputError(String),
}

type Result = std::result::Result<(), DissasembleError>;

fn decode(instruction: u16) -> String {
    let opcode = ((instruction & 0xf000u16) >> 12) as u8;

    match opcode {
        0x0 => {
            let lnibbles = instruction & 0x0fffu16;

            if lnibbles == 0x00e0 {
                return format!("CLS");
            } else if lnibbles == 0x00ee {
                return format!("RET");
            } else {
                return format!("SYS 0x{:0>3x}", lnibbles);
            }
        },
        0x1 => {
            // JP addr:nnn
            let nnn = instruction & 0x0fff;
            return format!("JP 0x{:0>3x}", nnn);
        },
        0x2 => {
            // CALL addr:nnn
            let nnn = instruction & 0x0fff;
            return format!("CALL 0x{:0>3x}", nnn);
        },
        0x3 => {
            // 3xkk - SE Vx, byte
            let x = ((instruction >> 8u16) & 0x000fu16) as u8;
            let kk = (instruction & 0x00ffu16) as u8;
            return format!("SE V{:x}, 0x{:0>2x}", x, kk);
        },
        0x4 => {
            // 4xkk - SNE Vx, byte
            let x = ((instruction >> 8u16) & 0x000fu16) as u8;
            let kk = (instruction & 0x00ffu16) as u8;
            return format!("SNE V{:x}, 0x{:0>2x}", x, kk);
        },
        0x5 => {
            // 4xy0 - SE Vx, Vy
            let x = ((instruction >> 8u16) & 0x000fu16) as u8;
            let y = (instruction >> 4u16 & 0x000fu16) as u8;
            return format!("SE V{:x}, V{:x}", x, y);
        },
        0x6 => {
            // 6xkk - LD Vx, byte
            let x = ((instruction >> 8u16) & 0x000fu16) as u8;
            let kk = (instruction & 0x00ffu16) as u8;
            return format!("LD V{:x}, 0x{:0>2x}", x, kk);
        },
        0x7 => {
            // 7xkk - ADD Vx, byte
            let x = ((instruction >> 8u16) & 0x000fu16) as u8;
            let kk = (instruction & 0x00ffu16) as u8;
            return format!("ADD V{:x}, 0x{:0>2x}", x, kk);
        },
        0x8 => {
            let x = ((instruction >> 8u16) & 0x000fu16) as u8;
            let y = (instruction >> 4u16 & 0x000fu16) as u8;
            let n = (instruction & 0x000fu16) as u8;

            match n {
                // 8xy0 - LD Vx, Vy
                0x0 => {
                    return format!("LD V{:x}, V{:x}", x, y);
                },
                // 8xy1 - OR Vx, Vy
                0x1 => {
                    return format!("OR V{:x}, V{:x}", x, y);
                },
                // 8xy2 - AND Vx, Vy
                0x2 => {
                    return format!("AND V{:x}, V{:x}", x, y);
                },
                // 8xy3 - XOR Vx, Vy
                0x3 => {
                    return format!("XOR V{:x}, V{:x}", x, y);
                },
                // 8xy4 - ADD Vx, Vy
                0x4 => {
                    return format!("ADD V{:x}, V{:x}", x, y);
                },
                // 8xy5 - SUB Vx, Vy
                0x5 => {
                    return format!("SUB V{:x}, V{:x}", x, y);
                },
                // 8xy6 - SHR Vx {, Vy}
                0x6 => {
                    return format!("SHR V{:x}, V{:x}", x, y);
                },
                // 8xy7 - SUBN Vx ,Vy
                0x7 => {
                    return format!("SUBN V{:x}, V{:x}", x, y);
                },
                // 8xy6 - SHL Vx {, Vy}
                0xe => {
                    return format!("SHL V{:x}, V{:x}", x, y);
                },
                _ => {
                    return format!("(invalid): opcode: 8, x: {:x}, v: {:x}, sel: 0x{:>1x}", x, y, n);
                }
            }
        },
        0x9 => {
            // 9xy0 - SNE Vx, Vy
            let x = ((instruction >> 8u16) & 0x000fu16) as u8;
            let y = (instruction >> 4u16 & 0x000fu16) as u8;
            return format!("SNE V{:x}, V{:x}", x, y);
        },
        0xa => {
            // Annn - LD I, addr:nnn
            let nnn = instruction & 0x0fff;
            return format!("LD I, 0x{:0>3x}", nnn);
        },
        0xb => {
            // Bnnn - JP V0, addr:nnn
            let nnn = instruction & 0x0fff;
            return format!("LD V0, 0x{:0>3x}", nnn);
        },
        0xc => {
            // Cxkk - RND Vx, byte
            let x = ((instruction >> 8u16) & 0x000fu16) as u8;
            let kk = (instruction & 0x00ffu16) as u8;
            return format!("RND V{:x}, 0x{:0>2x}", x, kk);
        },
        0xd => {
            // Dxyn - DRW Vx, Vy, nibble
            let x = ((instruction >> 8u16) & 0x000fu16) as u8;
            let y = (instruction >> 4u16 & 0x000fu16) as u8;
            let n = (instruction & 0x000fu16) as u8;

            return format!("DRW V{:x}, V{:x}, 0x{:0>2x}", x, y, n);
        },

        0xe => {
            let x = ((instruction >> 8u16) & 0x000fu16) as u8;
            let kk = (instruction & 0x00ffu16) as u8;

            match kk {
                // Ex9e - SKP Vx
                0x9e => {
                    return format!("SKP V{:x}", x);
                },
                // Exa1 - SKNP Vx
                0xa1 => {
                    return format!("SKNP V{:x}", x);
                },
                _ => {
                    return format!("(invalid): opcode: e, x: {:x}, 0x{:0>2x}", x, kk);
                }
            }
        },

        0xf => {
            let x = ((instruction >> 8u16) & 0x000fu16) as u8;
            let kk = (instruction & 0x00ffu16) as u8;

            match kk {
                // Fx07 - LD Vx, DT
                0x07 => {
                    return format!("LD V{:x}, DT", x);
                },
                // Fx0a - LD Vx, K
                0x0a => {
                    return format!("LD V{:x}, K", x);
                },
                // Fx15 - LD  DT, Vx
                0x15 => {
                    return format!("LD DT, V{:x}", x);
                },
                // Fx18 - LD ST, Vx
                0x18 => {
                    return format!("LD ST, V{:x}", x);
                },
                // Fx1e - ADD I, Vx
                0x1e => {
                    return format!("ADD I, V{:x}", x);
                },
                // Fx29 - LD F, Vx
                0x29 => {
                    return format!("LD F, V{:x}", x);
                },
                // Fx33 - LD B, Vx
                0x33 => {
                    return format!("LD B, V{:x}", x);
                },
                // Fx55 - LD [I], Vx
                0x55 => {
                    return format!("LD [I], V{:x}", x);
                },
                // Fx65 - LD Vx, [I]
                0x65 => {
                    return format!("LD V{:x}, [I]", x);
                },
                _ => {
                    return format!("(invalid): opcode: f, x: {:x}, sel: 0x{:0>2x}", x, kk);
                }
            }
        },
        _ => {
            return format!("(invalid): opcode: {:x}, args: 0x{:0>3x}", opcode, instruction & 0x0fffu16);
        }
    }
    String::new()
}

fn disassemble<R: Read, T: Write>(input: R, output: T) -> Result {
    let mut in_reader = BufReader::new(input);
    let mut out_writer = BufWriter::new(output);
    let new_line = "\n".as_bytes();
    let mut offset = 0;

    let mut instruction_buffer: [u8; 2] = [0; 2];
    loop {
        match in_reader.read(&mut instruction_buffer) {
            Ok(n) => {
                if n != 2 {
                    return Ok(());
                }
                let instruction = ((instruction_buffer[0] as u16) << 8) | instruction_buffer[1] as u16;
                let decoded_instruction = decode(instruction);
                out_writer.write(format!("0x{:0>4x}: ", offset).as_bytes());
                out_writer.write(decoded_instruction.as_bytes()).unwrap();
                out_writer.write(&new_line).unwrap();
                out_writer.flush();
                offset += n;
            },
            Err(_) => { return Err(DissasembleError::InputError("Error reading input".to_owned())) }
        }
    }
}
