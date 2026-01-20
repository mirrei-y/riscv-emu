use riscv_emu::{Bus, Cpu, Instruction, Memory, RawInstruction, RawShortInstruction};

#[test]
fn test_fibonacci() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new(1024 * 1024 * 4);
    let mut bus = Bus::new(memory);

    let code: Vec<u8> = vec![
        0x93, 0x02, 0x05, 0x00, // mv   t0, a0       (カウンタとしてnを退避)
        0x13, 0x05, 0x00, 0x00, // li   a0, 0        (current = 0)
        0x13, 0x03, 0x10, 0x00, // li   t1, 1        (next = 1)
        0x63, 0x8a, 0x02, 0x00, // beqz t0, 20       (if n == 0 goto ret)
        // loop:
        0xb3, 0x03, 0x65, 0x00, // add  t2, a0, t1   (temp = current + next)
        0x13, 0x05, 0x03, 0x00, // mv   a0, t1       (current = next)
        0x13, 0x83, 0x03, 0x00, // mv   t1, t2       (next = temp)
        0x93, 0x82, 0xf2, 0xff, // addi t0, t0, -1   (n--)
        0xe3, 0x98, 0x02, 0xfe, // bne  t0, zero, -16(if n != 0 goto loop)
        // ret:
        // 0x67, 0x80, 0x00, 0x00, // ret               (jalr x0, x1, 0)
        0x73, 0x00, 0x10, 0x00, // ebreak
    ];
    for (i, b) in code.iter().enumerate() {
        bus.write(0x8000_0000 + i as u64, *b as u64, 1).unwrap();
    }

    let mut cpu = Cpu::new(bus);
    cpu.write_register(10, 10); // a0 = 10 (フィボナッチ数列の項数)
    cpu.write_register(1, 12345678); // return address

    loop {
        cpu.cycle();
        // Fib(10) = 55
        // assert_eq!(cpu.read_register(10), 55);
    }
}
