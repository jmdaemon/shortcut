# README

## Usage

- shortcut: `gen [dir] [format] [options]`
    - `[format]`
    - `bash`: Generate bash path shortcuts
    - `powershell`: Generate powershell shortcuts
    - `shortpath`: Generate a shortpath file
    - `[options]`
    - `-d | --dest`: Generate to destination filepath

## Functionality

1. For a directory, recursively iterate through it
2. Yield every file path
3. For every file path,
    1. Get the parent folder name
    2. Join the path with the child's name
    3. Convert to ENV variable by converting to string with $ prefix
4. Convert to file format of choice

## Features
