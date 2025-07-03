# Long term todo

- support for more than one address space
- add more verification around the InstructionBuilder so we can catch issues before it gets to LLVM's module verification (do we really wanna do it? or is it enough to let LLVM do its thing?)
- allow creating modules that aren't assigned to a specific package? there's no LLVM limitation, it's all on our side, so we could
