## bingo

[中文](./README-ZH_CN.md)

An executable file manager written in Rust. It can add some executable files to the `$HOME/.bingo/bin` directory and invoke them by `bingo run <name>`.


## Usage

### add a new executable file

bingo can copy or link a executable file to the `$HOME/.bingo/bin` directory. if you don't specify the name, bingo will use the file name as the name

```bash
bingo cp /usr/bin/ls
bingo cp /usr/bin/ls myls
# or
bingo ln /usr/bin/ls
bingo ln /usr/bin/ls myls
```

### run a executable file

use `bingo run <name>`/`bingo r <name>` to run a executable file, or run it directly by `bingo <name>`, if `<name>` is same as bingo's subcommands, like `ls`, only `bingo run <name>` works. 

```bash
bingo ln /usr/bin/cat
bingo run cat test.txt
bingo r cat test.txt
# or run it d
bingo cat test.txt
```

### rename a executable file
```bash
bingo mv <old_name> <new_name>
```

### delete a executable file

only file in `$HOMW/.bingo/bin` can be removed, the original executable file won't be deleted.

```bash
bingo rm <name>
```

### list all executable files
```bash
bingo ls
```
