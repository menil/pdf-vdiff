_pdf__vdiff() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="pdf__vdiff"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        pdf__vdiff)
            opts="-o -f -v -h -V --output --force --open --theme --granularity --gutter-width --no-header --max-pages --verbose --generate-completions --generate-man --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --theme)
                    COMPREPLY=($(compgen -W "intellij github classic high-contrast" -- "${cur}"))
                    return 0
                    ;;
                --granularity)
                    COMPREPLY=($(compgen -W "word line character" -- "${cur}"))
                    return 0
                    ;;
                --gutter-width)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-pages)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --generate-completions)
                    COMPREPLY=($(compgen -W "bash elvish fish powershell zsh" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _pdf__vdiff -o nosort -o bashdefault -o default pdf-vdiff
else
    complete -F _pdf__vdiff -o bashdefault -o default pdf-vdiff
fi
