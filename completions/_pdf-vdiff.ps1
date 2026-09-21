
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'pdf-vdiff' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'pdf-vdiff'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'pdf-vdiff' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Target output PDF file path [default: <base_stem>_vs_<tailored_stem>_diff.pdf]')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Target output PDF file path [default: <base_stem>_vs_<tailored_stem>_diff.pdf]')
            [CompletionResult]::new('--theme', '--theme', [CompletionResultType]::ParameterName, 'Color palette theme (intellij, github, classic, high-contrast)')
            [CompletionResult]::new('--granularity', '--granularity', [CompletionResultType]::ParameterName, 'Diff granularity mode (word, line, character)')
            [CompletionResult]::new('--gutter-width', '--gutter-width', [CompletionResultType]::ParameterName, 'Spacing in points between left and right pages')
            [CompletionResult]::new('--max-pages', '--max-pages', [CompletionResultType]::ParameterName, 'Maximum page count threshold to prevent unbounded processing')
            [CompletionResult]::new('--generate-completions', '--generate-completions', [CompletionResultType]::ParameterName, 'Generate shell completion script to stdout (bash, zsh, fish, elvish, powershell)')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Overwrite destination output file if it already exists')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Overwrite destination output file if it already exists')
            [CompletionResult]::new('--open', '--open', [CompletionResultType]::ParameterName, 'Automatically open the generated diff in the system default PDF viewer')
            [CompletionResult]::new('--no-header', '--no-header', [CompletionResultType]::ParameterName, 'Suppress the top header/metadata banner')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose structural logging (omits sensitive raw text PII)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose structural logging (omits sensitive raw text PII)')
            [CompletionResult]::new('--generate-man', '--generate-man', [CompletionResultType]::ParameterName, 'Generate roff man page (Section 1) to stdout')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
