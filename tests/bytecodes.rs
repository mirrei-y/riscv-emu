use std::fs;
use std::path::Path;
use elf::endian::LittleEndian;
use elf::ElfBytes;
use elf::abi::PT_LOAD;

use riscv_emu::{Bus, Cpu, Exception, Instruction, Memory, RawInstruction, RawShortInstruction};

fn run_vm(path: &Path) -> Result<(), Exception> {
    let file_data = fs::read(path).expect("Could not read file");
    let slice = file_data.as_slice();
    let file = ElfBytes::<LittleEndian>::minimal_parse(slice).expect("Failed to parse ELF");

    // NOTE: メモリにロード
    let mut memory = Memory::new(1024 * 1024 * 128);
    for ph in file.segments().unwrap() {
        if ph.p_type == PT_LOAD {
            let offset = ph.p_offset as usize;
            let filesz = ph.p_filesz as usize;
            let vaddr = ph.p_vaddr;

            if vaddr >= 0x8000_0000 {
                let paddr = vaddr - 0x8000_0000;
                let segment_data = &slice[offset..offset + filesz];
                for (i, &byte) in segment_data.iter().enumerate() {
                    memory.write(paddr + i as u64, byte as u64, 1);
                }
            }
        }
    }

    let bus = Bus::new(memory);
    let mut cpu = Cpu::new(bus);

    let mut inst_count = 0;
    loop {
        if inst_count > 1_000_000 {
            panic!("Instruction limit reached.");
        }
        inst_count += 1;

        let instruction = cpu.fetch()?;
        let ctx = if instruction & 0b11 != 0b11 {
            cpu.decode_compressed(instruction as RawShortInstruction)
        } else {
            cpu.decode(instruction as RawInstruction)
        }?;

        println!("Execute: {:?}", ctx);

        if let Instruction::EBREAK = ctx.instruction {
            println!("EBREAK encountered. Halting execution.");
            break;
        }

        cpu.execute(ctx)?;
    }

    Ok(())
}
fn run_vm_glob(pattern: &str) -> Result<(), Exception> {
    for entry in glob::glob(pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                println!("Running test for: {:?}", path);
                run_vm(&path)?
            }
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(())
}

// #[test]
// fn test_all_bytecodes() -> Result<(), Exception> {
//     let fixtures_dir = Path::new("tests/fixtures/bytecodes");
//     let mut entries: Vec<_> = fs::read_dir(fixtures_dir)
//         .expect("Could not read fixtures directory. Please run: git submodule update --init --recursive")
//         .filter_map(|res| res.ok())
//         .map(|entry| entry.path())
//         .filter(|path| path.is_file())
//         .collect();
//     entries.sort();

//     for path in entries {
//         let filename = path.file_name().unwrap().to_str().unwrap();

//         println!("Running test for: {}", filename);
//         if let Err(e) = run_vm(&path) {
//             panic!("Test failed for {}: {:?}", filename, e);
//         }
//     }
//     Ok(())
// }

#[test]
fn test_rv64_user_integer_physical() -> Result<(), Exception> {
    run_vm_glob("tests/fixtures/bytecodes/rv64ui-p-*")
}

#[test]
fn test_rv64_user_multiply_physical() -> Result<(), Exception> {
    run_vm_glob("tests/fixtures/bytecodes/rv64um-p-*")
}

#[test]
fn test_rv64_user_compressed_physical() -> Result<(), Exception> {
    run_vm_glob("tests/fixtures/bytecodes/rv64uc-p-*")
}
