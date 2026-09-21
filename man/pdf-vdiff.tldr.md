# pdf-vdiff

> Fast CLI tool for side-by-side visual PDF diffing with vector fidelity.
> More information: <https://github.com/menil/pdf-vdiff>.

- Compare two PDF files and save the diff to a default filename:
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}}

- Compare two PDFs and specify a custom output path:
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}} -o {{path/to/diff.pdf}}

- Automatically open the resulting diff in the default PDF viewer:
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}} --open

- Diff using a specific color theme (e.g. github, intellij, classic, high-contrast):
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}} --theme {{github}}

- Adjust diff granularity to line or character level:
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}} --granularity {{line|character|word}}

- Force overwrite an existing output file:
  pdf-vdiff {{path/to/base.pdf}} {{path/to/tailored.pdf}} -f -o {{path/to/diff.pdf}}
