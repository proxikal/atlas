#compdef atlas

autoload -U is-at-least

_atlas() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
":: :_atlas_commands" \
"*::: :->atlas" \
&& ret=0
    case $state in
    (atlas)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:atlas-command-$line[1]:"
        case $line[1] in
            (run)
_arguments "${_arguments_options[@]}" : \
'--json[Output diagnostics in JSON format]' \
'-w[Watch for file changes and auto-recompile]' \
'--watch[Watch for file changes and auto-recompile]' \
'--no-clear[Don'\''t clear terminal before recompilation (with --watch)]' \
'-v[Verbose output with timing information]' \
'--verbose[Verbose output with timing information]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':file -- Path to the Atlas source file:_default' \
&& ret=0
;;
(r)
_arguments "${_arguments_options[@]}" : \
'--json[Output diagnostics in JSON format]' \
'-w[Watch for file changes and auto-recompile]' \
'--watch[Watch for file changes and auto-recompile]' \
'--no-clear[Don'\''t clear terminal before recompilation (with --watch)]' \
'-v[Verbose output with timing information]' \
'--verbose[Verbose output with timing information]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':file -- Path to the Atlas source file:_default' \
&& ret=0
;;
(check)
_arguments "${_arguments_options[@]}" : \
'--json[Output diagnostics in JSON format]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':file -- Path to the Atlas source file:_default' \
&& ret=0
;;
(c)
_arguments "${_arguments_options[@]}" : \
'--json[Output diagnostics in JSON format]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':file -- Path to the Atlas source file:_default' \
&& ret=0
;;
(build)
_arguments "${_arguments_options[@]}" : \
'-p+[Build profile (dev, release, test, or custom)]:PROFILE:_default' \
'--profile=[Build profile (dev, release, test, or custom)]:PROFILE:_default' \
'--release[Build in release mode (shorthand for --profile=release)]' \
'--clean[Clean build (ignore cache)]' \
'-v[Verbose output]' \
'--verbose[Verbose output]' \
'-q[Quiet output (errors only)]' \
'--quiet[Quiet output (errors only)]' \
'--json[JSON output]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
&& ret=0
;;
(b)
_arguments "${_arguments_options[@]}" : \
'-p+[Build profile (dev, release, test, or custom)]:PROFILE:_default' \
'--profile=[Build profile (dev, release, test, or custom)]:PROFILE:_default' \
'--release[Build in release mode (shorthand for --profile=release)]' \
'--clean[Clean build (ignore cache)]' \
'-v[Verbose output]' \
'--verbose[Verbose output]' \
'-q[Quiet output (errors only)]' \
'--quiet[Quiet output (errors only)]' \
'--json[JSON output]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
&& ret=0
;;
(repl)
_arguments "${_arguments_options[@]}" : \
'--tui[Use TUI mode (ratatui) instead of line editor]' \
'--no-history[Disable history persistence (for privacy)]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
&& ret=0
;;
(ast)
_arguments "${_arguments_options[@]}" : \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':file -- Path to the Atlas source file:_default' \
&& ret=0
;;
(typecheck)
_arguments "${_arguments_options[@]}" : \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':file -- Path to the Atlas source file:_default' \
&& ret=0
;;
(fmt)
_arguments "${_arguments_options[@]}" : \
'-c+[Path to configuration file]:CONFIG:_files' \
'--config=[Path to configuration file]:CONFIG:_files' \
'--indent-size=[Indentation size in spaces (default\: 4)]:INDENT_SIZE:_default' \
'--max-width=[Maximum line width (default\: 100)]:MAX_WIDTH:_default' \
'--trailing-commas=[Enable or disable trailing commas]:TRAILING_COMMAS:(true false)' \
'--check[Check formatting without modifying files]' \
'-w[Write changes to files (explicit mode)]' \
'--write[Write changes to files (explicit mode)]' \
'-v[Verbose output with timing information]' \
'--verbose[Verbose output with timing information]' \
'-q[Suppress non-error output]' \
'--quiet[Suppress non-error output]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
'*::files -- Files or directories to format:_default' \
&& ret=0
;;
(f)
_arguments "${_arguments_options[@]}" : \
'-c+[Path to configuration file]:CONFIG:_files' \
'--config=[Path to configuration file]:CONFIG:_files' \
'--indent-size=[Indentation size in spaces (default\: 4)]:INDENT_SIZE:_default' \
'--max-width=[Maximum line width (default\: 100)]:MAX_WIDTH:_default' \
'--trailing-commas=[Enable or disable trailing commas]:TRAILING_COMMAS:(true false)' \
'--check[Check formatting without modifying files]' \
'-w[Write changes to files (explicit mode)]' \
'--write[Write changes to files (explicit mode)]' \
'-v[Verbose output with timing information]' \
'--verbose[Verbose output with timing information]' \
'-q[Suppress non-error output]' \
'--quiet[Suppress non-error output]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
'*::files -- Files or directories to format:_default' \
&& ret=0
;;
(profile)
_arguments "${_arguments_options[@]}" : \
'--threshold=[Hotspot detection threshold percentage]:THRESHOLD:_default' \
'-o+[Save profile report to this file]:OUTPUT:_default' \
'--output=[Save profile report to this file]:OUTPUT:_default' \
'--summary[Print summary only (no detailed report)]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':file -- Path to the Atlas source file:_default' \
&& ret=0
;;
(test)
_arguments "${_arguments_options[@]}" : \
'--dir=[Test directory (defaults to current directory)]:DIR:_files' \
'--sequential[Run tests sequentially instead of parallel]' \
'-v[Verbose output (show all test names)]' \
'--verbose[Verbose output (show all test names)]' \
'--no-color[Disable colored output]' \
'--json[Output in JSON format]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
'::pattern -- Filter tests by name pattern:_default' \
&& ret=0
;;
(t)
_arguments "${_arguments_options[@]}" : \
'--dir=[Test directory (defaults to current directory)]:DIR:_files' \
'--sequential[Run tests sequentially instead of parallel]' \
'-v[Verbose output (show all test names)]' \
'--verbose[Verbose output (show all test names)]' \
'--no-color[Disable colored output]' \
'--json[Output in JSON format]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
'::pattern -- Filter tests by name pattern:_default' \
&& ret=0
;;
(debug)
_arguments "${_arguments_options[@]}" : \
'*-b+[Set breakpoints at line numbers (can be repeated)]:BREAKPOINT:_default' \
'*--breakpoint=[Set breakpoints at line numbers (can be repeated)]:BREAKPOINT:_default' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':file -- Path to the Atlas source file:_default' \
&& ret=0
;;
(d)
_arguments "${_arguments_options[@]}" : \
'*-b+[Set breakpoints at line numbers (can be repeated)]:BREAKPOINT:_default' \
'*--breakpoint=[Set breakpoints at line numbers (can be repeated)]:BREAKPOINT:_default' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':file -- Path to the Atlas source file:_default' \
&& ret=0
;;
(lsp)
_arguments "${_arguments_options[@]}" : \
'--port=[Port for TCP mode]:PORT:_default' \
'--host=[Bind address for TCP mode]:HOST:_default' \
'--tcp[Use TCP mode instead of stdio]' \
'-v[Enable verbose logging]' \
'--verbose[Enable verbose logging]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
&& ret=0
;;
(completions)
_arguments "${_arguments_options[@]}" : \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':shell -- Shell to generate completions for:(bash elvish fish powershell zsh)' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_atlas__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:atlas-help-command-$line[1]:"
        case $line[1] in
            (run)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(check)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(build)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(repl)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(ast)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(typecheck)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(fmt)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(profile)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(test)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(debug)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(lsp)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(completions)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_atlas_commands] )) ||
_atlas_commands() {
    local commands; commands=(
'run:Run an Atlas source file' \
'r:Run an Atlas source file' \
'check:Type-check an Atlas source file without running' \
'c:Type-check an Atlas source file without running' \
'build:Build an Atlas project' \
'b:Build an Atlas project' \
'repl:Start an interactive REPL' \
'ast:Dump AST to JSON' \
'typecheck:Dump typecheck information to JSON' \
'fmt:Format Atlas source files' \
'f:Format Atlas source files' \
'profile:Profile an Atlas source file (VM execution analysis)' \
'test:Run tests in a directory' \
't:Run tests in a directory' \
'debug:Debug an Atlas program interactively' \
'd:Debug an Atlas program interactively' \
'lsp:Start the Atlas Language Server' \
'completions:Generate shell completions' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'atlas commands' commands "$@"
}
(( $+functions[_atlas__ast_commands] )) ||
_atlas__ast_commands() {
    local commands; commands=()
    _describe -t commands 'atlas ast commands' commands "$@"
}
(( $+functions[_atlas__build_commands] )) ||
_atlas__build_commands() {
    local commands; commands=()
    _describe -t commands 'atlas build commands' commands "$@"
}
(( $+functions[_atlas__check_commands] )) ||
_atlas__check_commands() {
    local commands; commands=()
    _describe -t commands 'atlas check commands' commands "$@"
}
(( $+functions[_atlas__completions_commands] )) ||
_atlas__completions_commands() {
    local commands; commands=()
    _describe -t commands 'atlas completions commands' commands "$@"
}
(( $+functions[_atlas__debug_commands] )) ||
_atlas__debug_commands() {
    local commands; commands=()
    _describe -t commands 'atlas debug commands' commands "$@"
}
(( $+functions[_atlas__fmt_commands] )) ||
_atlas__fmt_commands() {
    local commands; commands=()
    _describe -t commands 'atlas fmt commands' commands "$@"
}
(( $+functions[_atlas__help_commands] )) ||
_atlas__help_commands() {
    local commands; commands=(
'run:Run an Atlas source file' \
'check:Type-check an Atlas source file without running' \
'build:Build an Atlas project' \
'repl:Start an interactive REPL' \
'ast:Dump AST to JSON' \
'typecheck:Dump typecheck information to JSON' \
'fmt:Format Atlas source files' \
'profile:Profile an Atlas source file (VM execution analysis)' \
'test:Run tests in a directory' \
'debug:Debug an Atlas program interactively' \
'lsp:Start the Atlas Language Server' \
'completions:Generate shell completions' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'atlas help commands' commands "$@"
}
(( $+functions[_atlas__help__ast_commands] )) ||
_atlas__help__ast_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help ast commands' commands "$@"
}
(( $+functions[_atlas__help__build_commands] )) ||
_atlas__help__build_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help build commands' commands "$@"
}
(( $+functions[_atlas__help__check_commands] )) ||
_atlas__help__check_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help check commands' commands "$@"
}
(( $+functions[_atlas__help__completions_commands] )) ||
_atlas__help__completions_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help completions commands' commands "$@"
}
(( $+functions[_atlas__help__debug_commands] )) ||
_atlas__help__debug_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help debug commands' commands "$@"
}
(( $+functions[_atlas__help__fmt_commands] )) ||
_atlas__help__fmt_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help fmt commands' commands "$@"
}
(( $+functions[_atlas__help__help_commands] )) ||
_atlas__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help help commands' commands "$@"
}
(( $+functions[_atlas__help__lsp_commands] )) ||
_atlas__help__lsp_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help lsp commands' commands "$@"
}
(( $+functions[_atlas__help__profile_commands] )) ||
_atlas__help__profile_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help profile commands' commands "$@"
}
(( $+functions[_atlas__help__repl_commands] )) ||
_atlas__help__repl_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help repl commands' commands "$@"
}
(( $+functions[_atlas__help__run_commands] )) ||
_atlas__help__run_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help run commands' commands "$@"
}
(( $+functions[_atlas__help__test_commands] )) ||
_atlas__help__test_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help test commands' commands "$@"
}
(( $+functions[_atlas__help__typecheck_commands] )) ||
_atlas__help__typecheck_commands() {
    local commands; commands=()
    _describe -t commands 'atlas help typecheck commands' commands "$@"
}
(( $+functions[_atlas__lsp_commands] )) ||
_atlas__lsp_commands() {
    local commands; commands=()
    _describe -t commands 'atlas lsp commands' commands "$@"
}
(( $+functions[_atlas__profile_commands] )) ||
_atlas__profile_commands() {
    local commands; commands=()
    _describe -t commands 'atlas profile commands' commands "$@"
}
(( $+functions[_atlas__repl_commands] )) ||
_atlas__repl_commands() {
    local commands; commands=()
    _describe -t commands 'atlas repl commands' commands "$@"
}
(( $+functions[_atlas__run_commands] )) ||
_atlas__run_commands() {
    local commands; commands=()
    _describe -t commands 'atlas run commands' commands "$@"
}
(( $+functions[_atlas__test_commands] )) ||
_atlas__test_commands() {
    local commands; commands=()
    _describe -t commands 'atlas test commands' commands "$@"
}
(( $+functions[_atlas__typecheck_commands] )) ||
_atlas__typecheck_commands() {
    local commands; commands=()
    _describe -t commands 'atlas typecheck commands' commands "$@"
}

if [ "$funcstack[1]" = "_atlas" ]; then
    _atlas "$@"
else
    compdef _atlas atlas
fi
