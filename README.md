# Plume
An LLVM-based language that aims to be an in-between of JavaScript and C.

This is just a for-fun project, so don't expect anything serious out of it. I've been wanting to make a compiled language for a while now and decided to finally do it.

## What does it look like?
A lot like C and TypeScript. For now it's very C-like, but as the language develops it will be nicely in the middle:

```plume
declare function putchar(ch: char) -> i32;

// putchar usage is temporary until strings are implemented in llvm
function main(argc: i32, argv: string*) -> i32 {
  putchar('h');
  putchar('i');
  // newline (parser doesn't account for escapes yet)
  putchar(10); 
  return 0;
}
```

## Usage
(replace `plume` with `cargo run -- {command}` if running from source).
``` shell
# this is not implemented yet, just create a project.json file in the root of your project. (see examples folder)
$ plume init my-project && cd my-project
$ plume build
```

## Goals
- Target WebAssembly nicely with a minimal runtime.
- Be able to link and use any C library.
- Provide a clean and simple language for beginners and experienced developers.
- Implement compile time guarantees for things like memory saftey, à la Rust.
- Provide a very good stdlib, like Go.
- Reach self-hosting language status (can compile itself).

## To-Do
- [ ] `plume init` command to create a plume project.
- [ ] `plume compile` command to directly compile source files.
- [ ] File validation (ensure that code will work with LLVM)
- [ ] Determine types for values at parsing/verification time.
- [ ] Clean up Lexer code
- [ ] Compile loops, conditions, etc.
- [ ] Run a linker on the outputted object files
- [ ] Optionally export object files, bytecode, and LLVM IR,
- [ ] A JIT mode.
- [ ] A Language Server, for support in most code editors.
- [ ] AST subcommand should export a JSON representation of the code that can be loaded into the compiler.
