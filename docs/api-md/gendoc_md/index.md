# gendoc_md

Orchestration for the `gendoc-md` command line workflow.

This crate keeps IO, discovery, parsing, and rendering in separate modules so
that Python source is never imported or executed. The top-level runner
validates destructive output choices before replacing generated Markdown
files.

## Modules

### [`gendoc_md`](gendoc_md.md)

*2 functions, 6 modules*

### [`cli`](cli.md)

*1 function, 1 struct*

### [`diagnostics`](diagnostics.md)

*1 enum*

### [`discover`](discover.md)

*2 functions*

### [`model`](model.md)

*3 enums, 7 structs*

### [`python`](python.md)

*2 functions*

### [`render`](render.md)

*1 function*
