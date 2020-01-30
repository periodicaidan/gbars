use azul::prelude::*;
use azul::widgets::label::Label;
use azul::widgets::button::Button;

use crate::emu::emulator::ROM;

use std::collections::HashMap;

pub struct RomMetaDataModel {
    title: String,
    version: String,
    licensee: String,
    cart_type: String,
    gbs: String,
    header_checksum: String,
    global_checksum: String
}

impl RomMetaDataModel {
    pub fn create(rom: &ROM) -> Self {
        let mut cart_type = String::new();
        for feature in &rom.cart_type {
            cart_type.push_str(&format!("{:?}", feature));
            cart_type.push('+');
        }

        cart_type.pop();

        RomMetaDataModel {
            title: rom.title.clone(),
            version: rom.version_no.to_string(),
            licensee: rom.licensee.clone(),
            cart_type: cart_type,
            gbs: String::from(if rom.gbs_compatible {"Available"} else {"Unavailable"}),
            header_checksum: format!("0x{:02X}", rom.header_checksum),
            global_checksum: format!("0x{:04X}", rom.global_checksum)
        }
    }
}

impl Layout for RomMetaDataModel {
    fn layout(&self, _model: LayoutInfo<Self>) -> Dom<Self> {
        let mut meta = Vec::<(&str, &String)>::new();

        meta.push(("Title", &self.title));
        meta.push(("Version", &self.version));
        meta.push(("Licensee", &self.licensee));
        meta.push(("Cart Type", &self.cart_type));
        meta.push(("GameBoy Super Features", &self.gbs));
        meta.push(("Checksum", &self.global_checksum));

        meta.iter().enumerate().map(|(n, (k, v))| {
            Dom::div().with_class("row")
                .with_child(Dom::label(*k))
                .with_child(Dom::label((*v).clone()))
        }).collect::<Dom<Self>>()
    }
}