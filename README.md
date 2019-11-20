# naama
A VST host written in `RUST` lang for Windows.

# Prerequisites
- GTK+
- GLib
- Cario

# Toolchain
Working fine on the `nightly-x86_64-pc-windows-msvc` (2019-09-23)

# Portability
`naama` is made for Windows only but few os specific features are used in anticipation of a future macos/unix compatibility.

# Setup prerequisites
- Install git standalone
- Install visual studio 2017 with english language pack
- install vcpkg
```bash
git clone https://github.com/Microsoft/vcpkg
cd vcpkg
.\bootstrap-vcpkg.bat 
```
- install gtk using vcpkg
```bash
vcpkg install gtk:x64-windows
```
- WIP (this not a proper way to do this) copy every .lib and .dll from your "vcpkg/packages/ [WIP]" to your /target/deps
- Setup your path and GTK_LIB_DIR, for example
```bash
SET GTK_LIB_DIR=C:\vcpkg\packages\gtk_x64-windows\lib
SET PATH=%PATH%;C:\vcpkg\packages\gtk_x64-windows\bin
```
- Install rust nightly for windows (not wsl)

# Build the host

# Build example VST
As the project is b
ased on the `vst` crate there is some example VST available from it, to build it (on windows)