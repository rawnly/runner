# 🏃Runner
Hotreload for scripting

## Usage
```sh
Usage: runner [OPTIONS] [PATH]

Arguments:
  [PATH]  path to the file to watch

Options:
      --runtime <RUNTIME>  runtime to use for the file if not provided, it will be inferred from the file extension [possible values: perl, php, ruby, c, cpp, python, python3, node, go, typescript, c-sharp, java, swift, scala, rust, shell, unsupported]
      --command <COMMAND>  command to run when the file changes - if includes whitespace, it will be split and the first part will be the command when using docker you can use {entrypoint} to refer to the executable
      --image <IMAGE>      
  -e <ENV>                 environment variables to pass to the command e.g. `--env "KEY=VALUE"`
      --no-docker          do not use docker to run the code this is useful when you want to run the code on your local machine
  -h, --help               Print help
```

## Supported Runtimes 
Runtimes are automatically detected by file extension if not specified
```
  Perl
  Php
  Ruby
  C
  Cpp
  Python
  Python3
  Node
  Go
  Typescript
  CSharp
  Java
  Swift
  Scala
  Rust
  Shell
```
