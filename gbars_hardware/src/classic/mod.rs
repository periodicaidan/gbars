// cartridge depends on std::fs, std::io, and std::error
#[cfg(feature = "std")] pub mod cartridge;
pub mod cpu;
pub mod instruction;
pub mod memory;
pub mod registers;
pub mod console;
pub(crate) mod utils;

#[cfg(test)]
mod test {
    use super::cartridge::Cartridge;
    use super::cpu::{Cpu, CpuState, OpRead, DataRead};
    use super::memory::{MBC, ROM};
    use crate::classic::console::Console;

    #[test]
    fn cartridge_loads_and_parses_header_correctly() {
        let cartridge = Cartridge::load("src/test_roms/pokeblue.gbc").unwrap();

        assert_eq!(cartridge.title, "POKEMON BLUE");
        assert_eq!(cartridge.rom_size, 1_048_576);
    }

    #[test]
    fn cartridge_is_valid() {
        let cartridge = Cartridge::load("src/test_roms/pokeblue.gbc").unwrap();

        // If the cartridge is invalid, this will panic and the test will fail
        cartridge.validate().unwrap();

        // If we've gotten here the following should be true
        assert!(cartridge.is_valid());
    }

    // #[test]
    // fn test_cpu() {
    //     let mut cpu = Cpu::init();
    //
    //     // This is a short program to test some instructions. Eventually I'll write something to
    //     // test all of them.
    //     let program = vec![
    //         0x00,               // nop
    //         0x04,               // inc b
    //         0x0E, 0x39,         // ld C, $39
    //         0xC3, 0x0B, 0x00,   // jp $000B
    //         0x15,               // dec D
    //         0x78,               // ld A, B
    //         0xD6, 0x01,         // sub 1
    //         0xCA, 0x07, 0x00,   // jp z $0007
    //         0xC3, 0x08, 0x00    // jp $0008
    //     ];
    //
    //     let mut memory_controller = MBC::RomOnly(ROM::new(program));
    //
    //     assert_eq!(cpu.state, CpuState::OpRead(OpRead::General));
    //
    //     // nop
    //     cpu.step(&mut memory_controller);
    //     assert_eq!(cpu.state, CpuState::Exec);
    //     cpu.step(&mut memory_controller);
    //     assert_eq!(cpu.state, CpuState::OpRead(OpRead::General));
    //
    //     // inc b
    //     cpu.step(&mut memory_controller);
    //     assert_eq!(cpu.state, CpuState::Exec);
    //     cpu.step(&mut memory_controller);
    //     assert_eq!(cpu.state, CpuState::OpRead(OpRead::General));
    //     assert_eq!(cpu.registers.b.0, 1);
    //
    //     // ld C, $39
    //     cpu.step(&mut memory_controller);
    //     assert_eq!(cpu.state, CpuState::DataRead(DataRead::Byte));
    //     cpu.step(&mut memory_controller);
    //     assert_eq!(cpu.state, CpuState::Exec);
    //     cpu.step(&mut memory_controller);
    //     assert_eq!(cpu.state, CpuState::OpRead(OpRead::General));
    //     assert_eq!(cpu.registers.c.0, 0x39);
    //
    //     // jp $000B
    //     cpu.step(&mut memory_controller);
    //     assert_eq!(cpu.state, CpuState::DataRead(DataRead::ShortLo));
    //     cpu.step(&mut memory_controller);
    //     assert_eq!(cpu.state, CpuState::DataRead(DataRead::ShortHi));
    //     cpu.step(&mut memory_controller);
    //     assert_eq!(cpu.state, CpuState::Exec);
    //     cpu.step(&mut memory_controller);
    //     assert_eq!(cpu.state, CpuState::OpRead(OpRead::General));
    //     assert_eq!(cpu.registers.pc, 0x000B);
    //
    //     // jp z $0007
    //     cpu.step(&mut memory_controller); // DataRead::ShortHi
    //     cpu.step(&mut memory_controller); // DataRead::ShortLo
    //     cpu.step(&mut memory_controller); // Exec
    //     cpu.step(&mut memory_controller); // OpRead::General
    //     assert_eq!(cpu.registers.pc, 0x000E);
    //
    //     // jp $0008
    //     cpu.step(&mut memory_controller); // DataRead::ShortHi
    //     cpu.step(&mut memory_controller); // DataRead::ShortLo
    //     cpu.step(&mut memory_controller); // Exec
    //     cpu.step(&mut memory_controller); // OpRead::General
    //     assert_eq!(cpu.registers.pc, 0x0008);
    //
    //     // ld A, B
    //     cpu.step(&mut memory_controller); // Exec
    //     cpu.step(&mut memory_controller); // OpRead::General
    //     assert_eq!(cpu.registers.a.0, 1);
    //
    //     // sub 1
    //     cpu.step(&mut memory_controller); // DataRead::Byte
    //     cpu.step(&mut memory_controller); // Exec
    //     cpu.step(&mut memory_controller); // OpRead::General
    //     assert_eq!(cpu.registers.a.0, 0);
    //     assert!(cpu.registers.zero());
    //
    //     // jp z $0007
    //     cpu.step(&mut memory_controller); // DataRead::ShortHi
    //     cpu.step(&mut memory_controller); // DataRead::ShortLo
    //     cpu.step(&mut memory_controller); // Exec
    //     cpu.step(&mut memory_controller); // OpRead::General
    //     assert_eq!(cpu.registers.pc, 0x0007);
    //
    //     // dec D
    //     cpu.step(&mut memory_controller); // Exec
    //     cpu.step(&mut memory_controller); // OpRead::General
    //     assert_eq!(cpu.registers.d.0, 0xFF);
    // }

    #[test]
    fn test_multiplication() {
        // This is a program that just multiplies 2 by 4
        let program = vec![
            0x3E, 0x02,         // ld A, $02
            0x4F,               // ld C, A
            0x06, 0x04,         // ld B, $04
            0x05,               // dec B
            // loop:
            0x81,               // add C
            0x05,               // dec B
            0xC2, 0x06, 0x00    // jp nz, loop
        ];

        let cartridge = Cartridge {
            title: "".to_string(),
            mbc: MBC::RomOnly(ROM::new(program.clone())),
            features: vec![],
            rom_size: 0,
            rom_banks: 0,
            ram_size: 0,
            ram_banks: 0,
            locale: "".to_string(),
            header_checksum: 0,
            global_checksum: 0
        };

        let mut cpu = Cpu::init();

        let mut console = Console::start(Some(cartridge));

        while (cpu.registers.pc as usize) < program.len() || cpu.state == CpuState::Exec {
            cpu.step(&mut console);
        }

        assert_eq!(cpu.registers.a.0, 8);
    }

    // #[test]
    // fn test_division() {
    //     let mut cpu = Cpu::init();
    //
    //     // This is a program that divides 8 by 2
    //     let program = vec![
    //         0x3E, 0x08,         // ld A, $08
    //         0x06, 0x02,         // ld B, $02
    //         0x0E, 0x00,         // ld C, $00
    //                             // loop:
    //         0x0C,               // inc C
    //         0x90,               // sub B
    //         0xC2, 0x06, 0x00,   // jp nz, loop
    //         0x79                // ld A, C
    //     ];
    //
    //     let mut memory = MBC::RomOnly(ROM::new(program.clone()));
    //
    //     while (cpu.registers.pc as usize) < program.len() || cpu.state == CpuState::Exec {
    //         cpu.step(&mut memory);
    //     }
    //
    //     assert_eq!(cpu.registers.a.0, 4);
    // }
}