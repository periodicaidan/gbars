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
- Support for link cable play
- Debugger and disassembler
- Good documentation
- Modular and embeddable
- Minimal dependency graph

**Non-Goals**
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
+-- gbars_hardware/
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
(N.B.: Structure is not final. Intuiting package structure *a priori* is really, really hard.)

## The Road to `no_std`

Most of the emulation portion is easily `no_std`-able. The biggest thing will be
removing the dependency on `std::fs`, mainly in the code for the cartridge (since 
the hardware may not have a filesystem). One idea would be to create special traits
for reading and writing an abstract cartridge, that users of the emulator can hook up
with whatever data source they're using for cartridges.

The only collections used by the emulation are `Vec` and `String`, which can easily
be replaced with `core::alloc` equivalents, and in many cases I think they can even be
replaced by regular slices.

## Graphics

The graphics are fairly straightforward for the GameBoy: Simply copy (and maybe 
reformat) the VRAM into an OpenGL texture, and render that texture onto a frame buffer.
I won't be using a library for this because what I want is too simple; instead I'll be
writing a minimal library using raw OpenGL bindings from [the `gl` crate](https://docs.rs/gl/0.14.0/gl/).

I may also pull in [`pixels`](https://docs.rs/pixels/0.0.2/pixels/) for the frame
buffering, which advertises "an emulator for your favorite platform" as one of its 
use-cases.

For an OpenGL context and windowing, I plan to use [`glutin`](https://docs.rs/glutin/0.23.0/glutin/).

*Vulkan renderer is not planned, and is basically never going to happen.* The reason for
this is that, while Vulkan can be much more optimal than OpenGL, you can only draw a
rectangle so optimally. Whatever minuscule performance increase there would be probably
won't be worth the extra work required to get Vulkan working.

## Audio

PortAudio is a pretty arcane API that not a lot of people have written about (probably 
because there aren't nearly as many people interested in audio programming as there are
in graphics programming). So getting it working is going to take a bit of work. I
currently have no plans for it, and it'll probably be the last thing I get working.

## GUI

Taking a glance at [Are We GUI Yet?](https://areweguiyet.com/), it would appear that we are
not, in fact, GUI yet. [Azul](https://azul.rs/) is a very promising IMGUI-style library written in pure Rust. However
I don't think it is quite ready to use yet (it's not even on crates.io). If and when it is, I would love to use it, as it seems
very flexible. Another god contender is [`iced`](https://github.com/hecrj/iced), which uses an Elm-like architecture to
structure code. I've been using Elm recently and love its design so this would be a great option as well.

But after contemplating other solutions, I think the best answer is one that isn't listed on Are We GUI Yet?: Flutter. 
[`flutter-rs`](https://github.com/flutter-rs/flutter-rs) is a library that lets you write desktop
apps with Rust and Flutter, *without having to leverage an FFI*. And Flutter exposes a class that can serve as an OpenGL
context, so this may just be the perfect solution for the time being. This has a few problems though, as the library 
isn't stable yet (and currently crashes on Linux when a mouse enters the window). However it is in active development and the
devs seem really serious about fixing the issues, so I think the major problems will be hammered out by the time I get
around to designing the UI. I could probably also fix the mouse enter issue on Linux myself :p 

Failing that, there is [wxWidgets](https://www.wxwidgets.org/), which will create a GUI that uses
native API's, making the app look natural on all platforms. It also provides OpenGL contexts, so 
this is a viable solution. [There are Rust bindings for it](https://github.com/kenz-gelsoft/wxRust)
but they don't appear to be actively maintained anymore, so I may have to do it in C++ (god no) or
Python (actually viable) or Haskell (for the meme). However, doing the GUI in another language may
require significant reworking versus doing it in Rust or with `flutter-rs`.

## Some Notes on Programming Style

As this is an emulator, I want this to be as fast and performant as possible. But where performance is no object, or
where it would not impact performance, *favor pure functions*. Pure functions are those that do not mutate their input.
I'm no Haskeller but there are merits to using these kinds of functions opportunistically. Basically if it does not need
to be `&mut` do not make it `&mut`. And favor Rust's iterator utilities over explicit loops.

Unless absolutely necessary, make all struct fields public. There is no compelling reason to make a struct's field
anything more private than crate-public unless it has unsafe pointers or `PhantomData`. This is a whole application; 
there is no reason why one part of the application can't have access to any other part.

Favor `enum`s over `trait`s for polymorphism.

### Naming Conventions

- Types and traits in `PascalCase`
- Functions and local variables in `snake_case`
- Globals in `SCREAMING_SNAKE_CASE`
- Initialisms shall only have their first letter capitalized (e.g., `Http`, not `HTTP`)