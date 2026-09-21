complete -c pdf-vdiff -s o -l output -d 'Target output PDF file path [default: <base_stem>_vs_<tailored_stem>_diff.pdf]' -r -F
complete -c pdf-vdiff -l theme -d 'Color palette theme (intellij, github, classic, high-contrast)' -r -f -a "intellij\t'Soft IntelliJ-style red/green diff colors (Default)'
github\t'Clean GitHub-style split diff colors'
classic\t'Traditional vibrant red/green diff colors'
high-contrast\t'High-contrast palette for maximum visual accessibility'"
complete -c pdf-vdiff -l granularity -d 'Diff granularity mode (word, line, character)' -r -f -a "word\t'Diff by word tokens (Default)'
line\t'Diff by full lines'
character\t'Diff by individual characters/glyphs'"
complete -c pdf-vdiff -l gutter-width -d 'Spacing in points between left and right pages' -r
complete -c pdf-vdiff -l max-pages -d 'Maximum page count threshold to prevent unbounded processing' -r
complete -c pdf-vdiff -l generate-completions -d 'Generate shell completion script to stdout (bash, zsh, fish, elvish, powershell)' -r -f -a "bash\t''
elvish\t''
fish\t''
powershell\t''
zsh\t''"
complete -c pdf-vdiff -s f -l force -d 'Overwrite destination output file if it already exists'
complete -c pdf-vdiff -l open -d 'Automatically open the generated diff in the system default PDF viewer'
complete -c pdf-vdiff -l no-header -d 'Suppress the top header/metadata banner'
complete -c pdf-vdiff -s v -l verbose -d 'Enable verbose structural logging (omits sensitive raw text PII)'
complete -c pdf-vdiff -l generate-man -d 'Generate roff man page (Section 1) to stdout'
complete -c pdf-vdiff -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pdf-vdiff -s V -l version -d 'Print version'
