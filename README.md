# Trajectory planner experiment

## Windows setup

Follow this <https://github.com/wingtk/gvsbuild#development-environment>

EXCEPT you must use Python 3.9 as `distutils` was removed in Python 3.12. Install it with

```bash
# In an admin prompt
choco install python --pin --version 3.9
```

You must ALSO do the manual install from `git` of `gvsbuild`. Needs some deps though:

```bash
# In an admin prompt
choco install xsltproc
```

This all needs to be done inside `C:\gtk-build\github`, presumably for some `venv` reasons idk.

Don't forget to set your paths:

```powershell
$env:Path = "C:\gtk-build\gtk\x64\release\bin;" + $env:Path
$env:LIB = "C:\gtk-build\gtk\x64\release\lib;" + $env:LIB
$env:INCLUDE = "C:\gtk-build\gtk\x64\release\include;C:\gtk-build\gtk\x64\release\include\cairo;C:\gtk-build\gtk\x64\release\include\glib-2.0;C:\gtk-build\gtk\x64\release\include\gobject-introspection-1.0;C:\gtk-build\gtk\x64\release\lib\glib-2.0\include;" + $env:INCLUDE
```

### Sidenote on `distutils`

If you get

```
meson.build:2329:26: ERROR: <PythonExternalProgram 'python3' -> ['C:\\Users\\jamwa\\.local\\pipx\\venvs\\gvsbuild\\Scripts\\python.exe']> is not a valid python or it is missing distutils
```

You must also install `setuptools` as `distutils` was removed in Python 3.10.

Thank you to [this SO answer](https://stackoverflow.com/a/77233866/383609).
