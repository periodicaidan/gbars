pub mod cartridge;
pub mod memory;
pub mod instruction;
pub mod registers;
pub mod cpu;

#[cfg(test)]
mod test {
    use super::cartridge::Cartridge;

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
}