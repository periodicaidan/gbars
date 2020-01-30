use self::super::windows::*;

use azul::prelude::*;

use crate::emu::emulator::ROM;

pub fn gui_main(rom: &ROM) {
    let mut app = App::new(RomMetaDataModel::create(rom), AppConfig::default()).unwrap();

    let win = app.create_window(WindowCreateOptions::default(),
                                azul::css::override_native(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/interface/rom_meta.css"))).expect("CSS file not found")).unwrap();

    app.run(win).unwrap();
}