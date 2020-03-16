//! File: ips.rs
//! Functions for working with IPS patch files. Useful utilities for ROM hackers and modders

use std::slice::Split;
use std::vec::Vec;
use std::fs::{File, copy, rename, remove_file};
use std::path::Path;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, SeekFrom};
use std::ffi::OsStr;

/// Reads an IPS file and extracts all the patches from it
///
/// An IPS file looks as follows (whitespace added for readability):
///
/// ```
/// PATCH
/// <offset: 3 bytes> <patch-length: 2 bytes> <patch: <patch-length> bytes>
/// <offset: 3 bytes> $00 $00 <patch-length: 2 bytes> <patch: 1 byte>
/// ...
/// EOF
///```
///
/// As you can see, there are two types of patches. Both of them start with the location in the
/// ROM that should be patched, which is 3-bytes long. For this reason, IPS is only recommended for
/// small ROMs (luckily, the largest GameBoy ROM is 1.5 MiB, which is small enough for 3 bytes).
/// This is followed by 2 bytes representing the length of the patch. If the length is greater than
/// 0, the next <length> bytes represent the actual patch. If the length is equal to 0, then the
/// next two bytes are read as the length of the patch, and then one more byte is read. This last
/// byte is to be copied into the ROM <length> times.
///
pub fn read(ips_file: &Path) -> Option<Vec<(usize, Vec<u8>)>> {
    // Check that the extension is ".ips"
    let ext = ips_file.extension().and_then(OsStr::to_str);
    if ext != Some(&"ips") {
        println!("{} is not an IPS file (extension must be \".ips\"", ips_file.display());
        return None;
    }

    let mut patches = Vec::new();

    match File::open(ips_file) {
        Ok(file) => {
            let mut buffer = Vec::new();
            BufReader::new(file)
                .read_to_end(&mut buffer)
                .expect("File Read Error");

            let mut header = &buffer[0..5];
            let mut file_pointer = 5;

            // Check that the file starts with "PATCH"
            if header != b"PATCH" {
                println!("Invalid IPS header");
                return None;
            }

            // Take the next 3 bytes
            let mut data = &buffer[file_pointer..file_pointer + 3];
            file_pointer += 3;

            // Loop until we reach the end of the file
            while data != b"" && data != b"EOF" {
                let mut patch = Vec::new();
                let mut offset = 0;

                // These three bytes represent the offset or beginning of where the patch should go
                // in the ROM
                for c in data {
                    offset = offset * 256 + (*c as usize);
                }

                // The next two bytes represent the length of the patch
                data = &buffer[file_pointer..file_pointer + 2];
                file_pointer += 2;

                let mut length = 0;
                for c in data {
                    length = length * 256 + (*c as usize);
                }

                // If these bytes are 0's, then the patch is a repeated byte
                if length == 0 {
                    // The next two bytes represent the number of times the byte should be repeated
                    data = &buffer[file_pointer..file_pointer + 2];
                    file_pointer += 2;

                    for c in data {
                        length = length * 256 + (*c as usize);
                    }

                    // Then the next byte is the byte to be copied
                    let mut byte = &buffer[file_pointer..file_pointer + 1];
                    file_pointer += 1;

                    for _ in 0..length {
                        patch.extend_from_slice(byte);
                    }
                } else {
                    // Take the next <length> bytes as the patch
                    data = &buffer[file_pointer..file_pointer + length];
                    file_pointer += length;

                    patch.extend_from_slice(data);
                }

                patches.push((offset, patch));

                // Then take the next 3 bytes as the start of the next patch
                data = &buffer[file_pointer..file_pointer + 3];
                file_pointer += 3;
            }
        },

        Err(e) => return None
    }

    Some(patches)
}

pub fn patch(rom_file: &str, ips_file: &str, backup: bool) -> Result<u64, String> {
    // "Creates" the backup file by just renaming rom_file with a .bak extension
    // This is necessary. Setting `backup` to false just deletes it after all is said and done
    let mut backup_rom_path = format!("{}.bak", rom_file);
    rename(rom_file, &backup_rom_path)
        .expect("Error writing backup file");

    // Creates a new file for the patched ROM and a writer buffer for it
    let mut patched_file = File::create(rom_file).unwrap();
    let mut writer = BufWriter::new(patched_file);

    // Get the list of patches
    let p = self::read(Path::new(ips_file));

    if let None = p {
        return Err("Error in IPS file. Aborting.".to_string());
    }

    let patches = p.unwrap();
    let mut patches_written = 0u64;

    // Now open the renamed ROM file
    match File::open(backup_rom_path) {
        Ok(file) => {
            // We are buffering the write for efficiency and safety reasons
            let mut reader = BufReader::new(file);

            // Copy the contents into the patched file's write buffer
            let mut temp: Vec<u8> = Vec::new();
            reader.read_to_end(&mut temp);
            writer.write_all(&temp).unwrap();

            // Apply the patches to the patched file
            for (offset, patch) in patches {
                writer.seek(SeekFrom::Start(offset as u64))
                    .expect(&format!("Error seeking to offset 0x{:06X}", offset));

                if let Ok(bytes_written) = writer.write(&patch) {
                    // If it only gets partially written, this is an error
                    if bytes_written < patch.len() {
                        return Err(format!("Problem writing bytes to {} at offset 0x{:06X}: {} / {} bytes written",
                            rom_file, offset, bytes_written, patch.len()));
                    }

                    println!("{} bytes written to {} starting at offset 0x{:06X}",
                             bytes_written, rom_file, offset);

                    patches_written += 1;
                } else {
                    println!("Problem writing bytes to {} at offset 0x{:06X}",
                        rom_file, offset);
                }
            }
        },

        Err(e) => return Err(format!("Error opening ROM file. Aborting.\nError: {}", e))
    }

    if !backup {
        if let Err(e) = remove_file(format!("{}.bak", rom_file)) {
            println!("Could not delete backup file, so it has been preserved.")
        }
    }

    Ok(patches_written)
}

pub fn restore(rom_file: &str, bak_file: &str, retain_backup: bool) -> Result<(), String> {
    let rom_path = Path::new(rom_file);
    let bak_path = Path::new(bak_file);

    if !rom_path.exists() {
        return Err(format!("ROM file {} does not exist", rom_file));
    }

    if !bak_path.exists() {
        return Err(format!("Backup file {} does not exist", bak_file));
    }

    remove_file(rom_path).unwrap();
    rename(bak_path, rom_path).unwrap();

    Ok(())
}
