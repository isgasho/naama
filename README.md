# naama
A VST host written in `RUST` lang for Windows.

# Prerequisites
- GTK+ [http://www.tarnyko.net/repo/gtk3_build_system/gtk+-bundle_3.6.4-20130513_win64.zip](x64 bundle for window)
- GLib
- Cario

# Toolchain
Working fine on the `nightly-x86_64-pc-windows-msvc` (2019-09-23)

# Portability
`naama` is made for Windows only but few os specific features are used in anticipation of a future macos/unix compatibility.

# Build example VST
As the project is based on the `vst` crate there is some example VST available from it, to build it (on windows)