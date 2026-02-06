# docs

This is the first file in the project to be written using demys! This means that the editor is probably at a point where it needs a usage guide.

# starting demys

`demys`  

Starting with no arguments will open an explorer tab.

`demys [file1] [file2] [file...]`  

Passing file paths as arguments will open all the files in separate tabs.

# navigation

To cycle through tabs, press `Tab`.\
When on a tab, press `Ctrl+Right` to split into a new window.\
To cycle through windows, press `Ctrl-l`.\
To close a window, press `Ctrl-X`.\
Press `Esc` or type `:qall` to exit.

Global commands:
- `:x` open explorer in new window

# text editing

demys is a modal editor!
- `i` to go into insert mode at cursor
- `Esc` or `Ctrl+[` to leave

Additional insert mode keys:
- `I` beginning of line
- `a` after cursor
- `A` end of line
- `o` new line below cursor

Text editor commands:
- `:tl` toggle line numbers
- `:w` save work (also ctrl+s)
- `:q` try quit
- `:q!` force quit
- `:wq` write and quit

# explorer tab

Use this tab to open files within your current directory.\
Press (j/Down) or (k/Up) to move your cursor, and enter to open/close directories or open a file.