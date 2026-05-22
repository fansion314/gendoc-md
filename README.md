# gendoc-md

`gendoc-md` statically reads a local Python project and generates a nested Markdown API map under `docs/api-md`.

```bash
gendoc-md -p your_package
```

The generated docs are intended as a quick index for LLMs and humans. The tool does not import or execute the target Python project.

## Install From Source

```bash
python -m pip install .
```

or build a wheel with maturin:

```bash
maturin build --release
```

## Usage

```bash
gendoc-md [OPTIONS]
```

- `-i, --input <DIR>`: add an import search root. Can be repeated.
- `-p, --package <IMPORT_NAME>`: document a package and its submodules. Can be repeated.
- `-m, --module <IMPORT_NAME>`: document a single module by import name. Can be repeated.
- `-o, --output <DIR>`: output directory. Defaults to `docs/api-md`.
- `--render-toc`: include generated tables of contents.
- `-j, --jobs <N>`: number of worker threads. Defaults to logical CPU count.

When no input roots are given, `gendoc-md` searches `./src` and `.`. When no package or module is given, it discovers top-level local packages and modules from those roots.
