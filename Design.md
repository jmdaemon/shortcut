# Design

## Functionality

### Converting into Shortcut Paths

#### File Path Type Detections

We need to detect between different types of shortcut paths,
specifically the root and children paths.

We need to treat both separately than either paths, and we can't easily
mix the two in homogeneous collections.

We do this by first filtering out the first element of our folder search.
This will be the `root` element, which we will treat specifically later.

**Basic Algorithm**

1. Classify different kinds of file paths
2. Support file path variants across platforms

To do this we define an enum `PathKind` with two states
1. `Environment`
2. `Standard`
This will allow us to aggregate variant paths in our containers,
while still managing them separately.

For cross platform support we have:

**Cases**

- Linux, Mac:
    - "~/my-folder"
    - "$HOME/my-folder"
    - "/home/user/my-folder"
- Windows
    - "C:Users\User\my-folder"
    - "$home\my-folder"
