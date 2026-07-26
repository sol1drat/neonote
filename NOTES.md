# TODO

## Important

- [X] Add command help screen
- [X] Do not render file extensions of Markdown files
- [X] When renaming file/dir start rename text box with current full file name
- [X] Implement auto-save

- [X] Cache last opened vault
- [ ] Add vault switching in app
- [ ] Editor text background color
- [ ] Make creating a file create a Markdown file by default
- [ ] Remove .md from editor in file name display

- [ ] Record a better demo for the README.md
- [ ] Deleting/renaming during vault selection

- [ ] File/dir moving
- [ ] Implement custom rendering of Markdown files
    - [ ] Syntax highlighting
    - [ ] Formatting

## Optimization

- [ ] Fix confirm prompt graphics
- [ ] Implement file explorer and editor widgets
- [ ] Rewrite prompts implementation
- [ ] Create build scripts for the majority of platforms

# Bugs

- [X] Deleting all files and leaving only dirs doesn't allow to make new files in the root dir
    - Fix Options:
        1. Create all files/dirs in root dir and implement file/dir moving
        2. [X] Render the root dir
- [X] Deleting a file keeps it buffered in the Editor

# Misc

Objects:
    - File explorer
        Needs its own implementation
    - Editor
        Needs its own implementation

Type of prompts:
    - Confirmation Y/N
        - Quit
        - Enter vault
        ...
    - Reports OK
        - Errors
        - Warnings
    - Text TYPE
        - File/directory creation, renaming
