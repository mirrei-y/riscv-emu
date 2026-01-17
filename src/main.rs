use crate::{cpu::{Cpu, Instruction}, bus::Bus, memory::Memory};

mod cpu;
mod bus;
mod memory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new(1024 * 1024);
    let mut bus = Bus::new(memory);

    let code: Vec<u8> = vec![
        0x93, 0x00, 0x10, 0x00, // addi x1, x0, 1
        0x13, 0x01, 0x20, 0x00, // addi x2, x0, 2
        0xb3, 0x81, 0x20, 0x00, // add x3, x1, x2
        0x73, 0x00, 0x10, 0x00, // ebreak
    ];
    for (i, b) in code.iter().enumerate() {
        bus.write(0x8000_0000 + i as u64, *b as u64, 1).unwrap();
    }

    let mut cpu = Cpu::new(bus);

    loop {
        let instruction = match cpu.fetch() {
            Ok(inst) => inst,
            Err(e) => {
                println!("Fetch error: {:?}", e);
                break;
            }
        };

        // 命令をデコード
        match cpu.decode(instruction) {
            Ok(inst) => {
                println!("Decoded: {:?}", inst);
                if let Instruction::EBREAK = inst {
                    println!("Program finished successfully.");
                    break;
                }
                // TODO: 実行処理
                // cpu.execute(inst);
            }
            Err(e) => {
                println!("Decode error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
