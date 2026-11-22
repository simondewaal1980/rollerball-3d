Local link shim. libasound.so points at the system libasound.so.2 runtime that
is always present on Linux desktops. The header/library files in this folder
are only used so the project builds without libasound2-dev installed.
