
use builtin;
use str;

set edit:completion:arg-completer[pdf-vdiff] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'pdf-vdiff'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'pdf-vdiff'= {
            cand -o 'Target output PDF file path [default: <base_stem>_vs_<tailored_stem>_diff.pdf]'
            cand --output 'Target output PDF file path [default: <base_stem>_vs_<tailored_stem>_diff.pdf]'
            cand --theme 'Color palette theme (intellij, github, classic, high-contrast)'
            cand --granularity 'Diff granularity mode (word, line, character)'
            cand --gutter-width 'Spacing in points between left and right pages'
            cand --max-pages 'Maximum page count threshold to prevent unbounded processing'
            cand --generate-completions 'Generate shell completion script to stdout (bash, zsh, fish, elvish, powershell)'
            cand -f 'Overwrite destination output file if it already exists'
            cand --force 'Overwrite destination output file if it already exists'
            cand --open 'Automatically open the generated diff in the system default PDF viewer'
            cand --no-header 'Suppress the top header/metadata banner'
            cand -v 'Enable verbose structural logging (omits sensitive raw text PII)'
            cand --verbose 'Enable verbose structural logging (omits sensitive raw text PII)'
            cand --generate-man 'Generate roff man page (Section 1) to stdout'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
