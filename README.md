## Hello

And welcome to moonside edisnoom ot emoclew dnA.

## Dependencies

 - Rust 1.30 or later
 - Cargo
 - SDL2
 - SDL2 Image
 - SDL2 TTF

### Arch

```
sudo pacman -S rust cargo sdl2 sdl2_image sdl2_ttf
cargo run
```

### Ubuntu

```
curl -sSf https://static.rust-lang.org/rustup.sh | sh
sudo apt-get install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev
cargo run
```

### OSX

```
brew install rust sdl2 sdl2_image sdl2_ttf
cargo run
```

### Windows

Install Rust with rustup and select the MSVC toolchain. Also install the
MSVC toolchain, e.g. by installing Visual Studio. Download the development
libraries for SDL, SDL_ttf, and SDL_image:

* https://www.libsdl.org/release/SDL2-devel-2.0.9-VC.zip
* https://www.libsdl.org/projects/SDL_ttf/release/SDL2_ttf-devel-2.0.14-VC.zip
* https://www.libsdl.org/projects/SDL_image/release/SDL2_image-devel-2.0.4-VC.zip

Extract these and stuff all the .lib and .dll files in your toolchain's lib folder, e.g.,
`$HOME\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\lib`.
(There's probably a better way to do this.)

Make sure your MSVC tools are availabile in your path, and `cargo run`!