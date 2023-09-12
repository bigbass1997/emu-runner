[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/emu-runner?style=flat-square)](https://crates.io/crates/emu-runner)
[![Documentation](https://img.shields.io/docsrs/emu-runner?style=flat-square)](https://docs.rs/emu-runner)
### Description
`emu-runner` is a command builder intended to make it easier to run a wide range of emulators using a consistent
interface. This utility attempts to handle any version differences automatically, such as determining how to execute a particular
emulator depending on the current OS, or recognizing which CLI argument to use based on the version of the emulator.

For example, for any given version of FCEUX, it has multiple build types, all which contain different executable names, and two entirely
different sets of CLI argument names. `emu-runner` simplifies this by providing an abiguous data structure:
```Rust
let ctx = FceuxContext::new("path/to/emulator")?
    .with_lua("/a/lua/script.lua")
    .with_movie("SuperMario.fm2")
    .with_rom("roms/Super Mario Bros.nes");

ctx.run();
```

Current supported emulators include: BizHawk, FCEUX, and Gens.

Other emulator contexts can be made by implementing the `EmulatorContext` trait on your own types.