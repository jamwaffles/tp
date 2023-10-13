# Trajectory planner experiment

## Windows setup

### Basically just don't bother.

Follow [this guide](https://www.gtk.org/docs/installations/windows/) using MSYS2, tl;dr
`pacman -S mingw-w64-x86_64-gtk3`.

Then download <https://gstreamer.freedesktop.org/download/> and install it so we get MSVC-built zlib
packages.

To get rust-analyzer in VSCode to pick up the right stuff, I created

```
PKG_CONFIG_LIBDIR=C:\tools\msys64\mingw64\lib\pkgconfig
```

in my env vars.
