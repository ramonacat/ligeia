# Long term todo

- support for more than one address space
- add more verification around the InstructionBuilder so we can catch issues before it gets to LLVM's module verification (do we really wanna do it? or is it enough to let LLVM do its thing?)
- allow creating modules that aren't assigned to a specific package? there's no LLVM limitation, it's all on our side, so we could
- add optional target type to the Pointer type (LLVM IR doesn't have pointer types, so maybe figure out why first, and if it's a good idea to add them for this wrapper)
