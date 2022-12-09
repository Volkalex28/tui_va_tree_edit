# tui-va-tree-edit

[tui-va-tree-edit][crate] is a editor and data display in tree style for [tui-rs] with using [tui-texarea]. The tree editor can be easily integrated into your TUI application.

## Features
 - [X] Display and edit `numbers`, `strings` and `bool` values
 - [X] Ability to add multiple trees separated by `tabs`
 - [X] Checking the correctness of the entered data
 - [X] Cancel changes without saving
 - [ ] Displaying and editing `arrays`
 - [ ] Support [crossterm] and [termion]
 - [ ] Automatic adjustment to the size of the area
 - [ ] Separate Widget and State

## Installation
Add `tui-va-tree-edit` crate to dependencies in your `Cargo.toml`.
```toml
[dependencies]
tui = "1.19"
tui-va-tree-edit = "*"
```

## Basic Usage

[crate]: https://github.com/Volkalex28/tui_va_tree_edit
[tui-rs]: https://github.com/fdehau/tui-rs
[tui-texarea]: https://github.com/rhysd/tui-textarea
[termion]: https://docs.rs/termion/latest/termion/
[crossterm]: https://docs.rs/crossterm/latest/crossterm/
