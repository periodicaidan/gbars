GBARS Design Document
===

Aidan T. Manning; Lead Developer

### Abstract
GBARS is an experimental GameBoy emulator and debugger written in Rust. The goal
is *playability*, and ideally it should be able to play as many GameBoy games as
possible. Longer-term goals will be to make it embeddable (i.e., to add no\_std 
support) and to add GameBoy Advance games. You should be able to load this onto a 
Raspberry Pi and make a little hacked-together GameBoy.

**Goals**
- Support as many games as possible, as well as possible
- GameBoy, GameBoy Color, and (eventually) GameBoy Advance
- Debugger and disassembler
- Good documentation
- Modular and embeddable

**Non-Goals**
- Cycle-accuracy
- Peripheral support (GB Printer, GB Camera, etc.)

## Package Structure

GBARS is divided into 3 major parts. First there's the low-level hardware emulation
crates, which model the CPU, the APU, the RAM, cartridges, etc. These crates shall be
no\_std to facilitate easy porting. The second is the high-level user interface things
like the audio, visuals, and CLI and GUI. The third will be utilities for debugging, 
patching, disassembling, and so on. 

```
gbars/
|
+-- hardware/
|   |
|   +-- src/
|   |   |
|   |   +-- classic/
|   |   |
|   |   +-- advanced/
|   |   |
|   |   +-- lib.rs
|   |
|   +-- Cargo.toml
|
+-- src/
|   |
|   +-- audio/
|   |
|   +-- graphics/
|   |
|   +-- cli/
|   |
|   +-- utils/
|   |
|   +-- debugging/
|   |
|   +-- console/
|   |
|   +-- main.rs
|
+-- Cargo.toml
|
+-- LICENSE
|
+-- README.md
|
+-- DESIGNDOC.md
```

## The Road to `no_std`

Most of the emulation portion is easily `no_std`-able. The biggest thing will be
removing the dependency on `std::fs`, mainly in the code for the cartridge (since 
the hardware may not have a filesystem). One idea would be to create special traits
for reading and writing an abstract cartridge, that users of the emulator can hook up
with whatever data source they're using for cartridges.

The only collections used by the emulation are `Vec` and `String`, which can easily
be replaced with `core::alloc` equivalents, and in many cases I think they can even be
replaced by regular slices.
