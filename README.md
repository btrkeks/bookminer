## Overview

This is a work-in-progress desktop application designed for quickly creating Anki cards 
containing LaTeX, with PDF viewer integration. 
It's inspired by the concept of
[Sentence Mining](https://refold.la/roadmap/stage-2/a/basic-sentence-mining/).

A (supported) PDF viewer is not necessary, but it will provide the file name
and the current page number for later reference.

**Note:** Currently, the application is only tested on Linux systems and probably won't work on Windows.

(I am open for better names for this application)

## Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) and [Cargo](https://github.com/rust-lang/cargo) installed on your system

### Steps
1. Clone this repository:
   ```
   git clone https://github.com/yourusername/bookminer.git
   cd bookminer
   ```

2. Build the application:
   ```
   cargo build --release
   ```

3. Add the compiled binary to your PATH:
   ```
   sudo cp ./target/release/bookminer /usr/local/bin/
   ```

## Usage

BookMiner uses the `EDITOR` and `TERMINAL` environment variables to choose the terminal and editor.
Ensure these are set in your environment. \
Then map the binary `bookminer` binary to a key in your window manager. \
You can also specify a custom terminal and editor for a single session by using:
```bash
TERMINAL=st EDITOR=nvim bookminer
```

### Supported PDF viewers

#### Sioyek
1. Open Sioyek's `prefs_user.config` file
2. Add the following command:
   ```
   new_command    _add_to_anki bookminer --book-filename %{file_name} --page-number %{page_number}
   ```
3. In `keys_user.config`, bind the command to a key (e.g., `U`):
   ```
   _add_to_anki       U
   ```

#### Zathura
Add the following to your Zathura config, replacing `<key>` with your preferred key:
```
map <key> exec "bookminer --book-filename \"$FILE\" --page-number \"$PAGE\""
```

### TODO
- [ ] Display the currently selected Anki config in the final menu
- [ ] Integrate with Okular using their [D-Bus API](https://docs.kde.org/trunk5/en/kid3/kid3/dbus-api.html)
- [ ] Pressing space in the tag selection should advance to the next tag
- [ ] Try to detect wrong Anki config (parse AnkiConnect error and act accordingly)
- [ ] LaTeX lint on the fly?
- [ ] LaTeX live preview?
- [ ] Define special keywords on the front and back? E.g. !PROOF! or !REMARK!
      and put everything after that in a different field. Also use for adding images?