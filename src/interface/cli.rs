use clap::{App, Arg, SubCommand};

use crate::emu::emulator::{ROM, Emulator};

use crate::ips;

pub fn cli_main() {
    let yaml = load_yaml!("cli.yaml");

    let mut cli = App::from_yaml(yaml);

    let matches = cli.get_matches();

    let dump = matches.subcommand_matches("dump");
    let patch = matches.subcommand_matches("patch");
    let debug = matches.subcommand_matches("debug");
    let disas = matches.subcommand_matches("disas");
    let as_ = matches.subcommand_matches("as");

    if let Some(d) = dump {
        let rom = d.subcommand_matches("rom");

        if let Some(r) = rom {
            let rom_to_dump = r.value_of("ROM").unwrap();
            ROM::new(rom_to_dump).dump();

            return;
        }
    }

//    if let Some(p) = patch {
//        let restore = p.subcommand_matches("restore");
//
//        if let Some(rest) = restore {
//            let rom = rest.value_of("ROM").unwrap();
//            let bak = rest.value_of("BACKUP").unwrap_or(format!("{}.bak", rom));
//            let retain_backup = match rest.value_of("retain-backup").unwrap() {
//                "true" => true,
//                "false" => false,
//                _ => true,
//            };
//
//            ips::restore(rom, bak, retain_backup);
//
//            return;
//        }
//
//        let rom = p.value_of("rom").unwrap();
//        let ips = p.value_of("ips").unwrap();
//        let backup = match p.value_of("backup").unwrap() {
//            "true" => true,
//            "false" => false,
//            _=> true
//        };
//
//        ips::patch(rom, ips, backup);
//
//        return;
//    }

    let rom = matches.value_of("rom");
    let emu = Emulator::start(rom);

    if let Err(e) = emu {
        println!("Error starting emulator: {}", e);
    }

    emu.unwrap();
}