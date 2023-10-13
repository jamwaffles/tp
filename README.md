# Trajectory planner experiment

## Windows setup

Follow [this guide](https://www.gtk.org/docs/installations/windows/) using MSYS2.

To get rust-analyzer in VSCode to pick up the right stuff, I added

```
PKG_CONFIG_LIBDIR=C:\tools\msys64\mingw64\lib\pkgconfig
```

to my env vars.

I also added `C:\tools\msys64\mingw64\bin;C:\tools\msys64\mingw64\lib;` to `PATH` which may or may
not help idk.
