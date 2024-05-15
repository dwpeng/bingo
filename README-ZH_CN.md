一个用Rust编写的可执行文件管理器。它可以将一些可执行文件添加到`$HOME/.bingo/bin`目录，并通过`bingo run <name>`来调用它们。

## 安装

```bash
# 使用cargo安装
cargo install bingogo
```

你也可以在发布页面找到并下载它。

## 使用方法

### 添加新的可执行文件

bingo可以复制或链接一个可执行文件到`$HOME/.bingo/bin`目录。如果你没有指定名称，bingo将使用文件名作为名称。

```bash
bingo cp /usr/bin/ls
bingo cp /usr/bin/ls myls
# 或者
bingo ln /usr/bin/ls
bingo ln /usr/bin/ls myls
```

### 运行一个可执行文件

使用`bingo run <name>`/`bingo r <name>`来运行一个可执行文件，或者直接通过`bingo <name>`来运行，如果`<name>`与bingo的子命令相同，如`ls`，只有`bingo run <name>`有效。

```bash
bingo ln /usr/bin/cat
bingo run cat test.txt
bingo r cat test.txt
# 或者直接运行
bingo cat test.txt
```

### 重命名一个可执行文件
```bash
bingo mv <old_name> <new_name>
```

### 删除一个可执行文件

只有在`$HOME/.bingo/bin`中的文件可以被删除，原始的可执行文件不会被删除。

```bash
bingo rm <name>
```

### 列出所有可执行文件
```bash
bingo ls
```
