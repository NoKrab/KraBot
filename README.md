# RSBOT README

How to build the project with voice support in Windows
------
Since libsodium requires GNU ABI instead of MSVC ABI, it is needed to use the GNU toolchain of rust. You can change to the GNU ABI using rustup, e.g. for the nightly Version execute **"rustup default nightly-gnu"** in your CLI.

Getting the GNU ABI itself including dependencies for voice support
------
The easiest way is to use the package manager of **MSYS2**.  
*Attention do not try to install MSYS2 on a drive formatted with FAT\* or ReFS, because they do not support* ***hard links***
```bash
   pacman -S --needed base-devel mingw-w64-x86_64-toolchain \
   mingw-w64-x86_64-libsodium mingw-w64-x86_64-opusfile mingw-w64-x86_64-opus \
   mingw-w64-x86_64-ffmpeg mingw-w64-x86_64-sqlite3  \
   mingw-w64-x86_64-python3 mingw-w64-x86_64-python3-pip
  
   pip3.6 install -U youtube-dl
   ```
If you want to debug rust with **Visual Studio Code** you will have to switch to the 32bit toolchain, since the 64bit is unstable.  
You can get LLVM with Python support [here](https://github.com/vadimcn/llvm/releases)  
Python **3.5** is also needed since llvm is hard linked to python35.dll, be sure to install the **32bit** Version of Python
```bash
   pacman -S --needed base-devel mingw-w64-i686-toolchain mingw-w64-i686-libsodium mingw-w64-i686-opusfile mingw-w64-i686-opus mingw-w64-i686-sqlite3 
   ```
   