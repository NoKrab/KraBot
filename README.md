# RSBOT README

How to build the project with voice support in Windows
------
Since libsodium requires GNU ABI instead of MSVC ABI, it is needed to use the GNU toolchain of rust. You can change to the GNU ABI using rustup, e.g. for the nightly Version execute **"rustup default nightly-gnu"** in your CLI.

Getting the GNU ABI itself including dependencies for voice support
------
The easiest way is to use the package manager of **MSYS2**.  
*Attention do not try to install MSYS2 on a drive formatted with FAT\* or ReFS, because they do not support* ***hard links***
```bash
   pacman -S mingw-w64-x86_64-toolchain
   pacman -S base-devel
   pacman -S mingw-w64-x86_64-libsodium mingw-w64-x86_64-opusfile mingw-w64-x86_64-opus
   pacman -S mingw-w64-x86_64-ffmpeg
   pacman -S mingw-w64-x86_64-python3 mingw-w64-x86_64-python3-pip
   pip3.6 install -U youtube-dl
   ```