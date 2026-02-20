# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_atlas_global_optspecs
	string join \n h/help V/version
end

function __fish_atlas_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_atlas_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_atlas_using_subcommand
	set -l cmd (__fish_atlas_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c atlas -n "__fish_atlas_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_needs_command" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "run" -d 'Run an Atlas source file'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "r" -d 'Run an Atlas source file'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "check" -d 'Type-check an Atlas source file without running'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "c" -d 'Type-check an Atlas source file without running'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "build" -d 'Build an Atlas project'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "b" -d 'Build an Atlas project'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "repl" -d 'Start an interactive REPL'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "ast" -d 'Dump AST to JSON'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "typecheck" -d 'Dump typecheck information to JSON'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "fmt" -d 'Format Atlas source files'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "f" -d 'Format Atlas source files'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "profile" -d 'Profile an Atlas source file (VM execution analysis)'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "test" -d 'Run tests in a directory'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "t" -d 'Run tests in a directory'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "debug" -d 'Debug an Atlas program interactively'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "d" -d 'Debug an Atlas program interactively'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "lsp" -d 'Start the Atlas Language Server'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "completions" -d 'Generate shell completions'
complete -c atlas -n "__fish_atlas_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c atlas -n "__fish_atlas_using_subcommand run" -l json -d 'Output diagnostics in JSON format'
complete -c atlas -n "__fish_atlas_using_subcommand run" -s w -l watch -d 'Watch for file changes and auto-recompile'
complete -c atlas -n "__fish_atlas_using_subcommand run" -l no-clear -d 'Don\'t clear terminal before recompilation (with --watch)'
complete -c atlas -n "__fish_atlas_using_subcommand run" -s v -l verbose -d 'Verbose output with timing information'
complete -c atlas -n "__fish_atlas_using_subcommand run" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand run" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand r" -l json -d 'Output diagnostics in JSON format'
complete -c atlas -n "__fish_atlas_using_subcommand r" -s w -l watch -d 'Watch for file changes and auto-recompile'
complete -c atlas -n "__fish_atlas_using_subcommand r" -l no-clear -d 'Don\'t clear terminal before recompilation (with --watch)'
complete -c atlas -n "__fish_atlas_using_subcommand r" -s v -l verbose -d 'Verbose output with timing information'
complete -c atlas -n "__fish_atlas_using_subcommand r" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand r" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand check" -l json -d 'Output diagnostics in JSON format'
complete -c atlas -n "__fish_atlas_using_subcommand check" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand check" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand c" -l json -d 'Output diagnostics in JSON format'
complete -c atlas -n "__fish_atlas_using_subcommand c" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand c" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand build" -s p -l profile -d 'Build profile (dev, release, test, or custom)' -r
complete -c atlas -n "__fish_atlas_using_subcommand build" -l release -d 'Build in release mode (shorthand for --profile=release)'
complete -c atlas -n "__fish_atlas_using_subcommand build" -l clean -d 'Clean build (ignore cache)'
complete -c atlas -n "__fish_atlas_using_subcommand build" -s v -l verbose -d 'Verbose output'
complete -c atlas -n "__fish_atlas_using_subcommand build" -s q -l quiet -d 'Quiet output (errors only)'
complete -c atlas -n "__fish_atlas_using_subcommand build" -l json -d 'JSON output'
complete -c atlas -n "__fish_atlas_using_subcommand build" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand build" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand b" -s p -l profile -d 'Build profile (dev, release, test, or custom)' -r
complete -c atlas -n "__fish_atlas_using_subcommand b" -l release -d 'Build in release mode (shorthand for --profile=release)'
complete -c atlas -n "__fish_atlas_using_subcommand b" -l clean -d 'Clean build (ignore cache)'
complete -c atlas -n "__fish_atlas_using_subcommand b" -s v -l verbose -d 'Verbose output'
complete -c atlas -n "__fish_atlas_using_subcommand b" -s q -l quiet -d 'Quiet output (errors only)'
complete -c atlas -n "__fish_atlas_using_subcommand b" -l json -d 'JSON output'
complete -c atlas -n "__fish_atlas_using_subcommand b" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand b" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand repl" -l tui -d 'Use TUI mode (ratatui) instead of line editor'
complete -c atlas -n "__fish_atlas_using_subcommand repl" -l no-history -d 'Disable history persistence (for privacy)'
complete -c atlas -n "__fish_atlas_using_subcommand repl" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand repl" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand ast" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand ast" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand typecheck" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand typecheck" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand fmt" -s c -l config -d 'Path to configuration file' -r -F
complete -c atlas -n "__fish_atlas_using_subcommand fmt" -l indent-size -d 'Indentation size in spaces (default: 4)' -r
complete -c atlas -n "__fish_atlas_using_subcommand fmt" -l max-width -d 'Maximum line width (default: 100)' -r
complete -c atlas -n "__fish_atlas_using_subcommand fmt" -l trailing-commas -d 'Enable or disable trailing commas' -r -f -a "true\t''
false\t''"
complete -c atlas -n "__fish_atlas_using_subcommand fmt" -l check -d 'Check formatting without modifying files'
complete -c atlas -n "__fish_atlas_using_subcommand fmt" -s w -l write -d 'Write changes to files (explicit mode)'
complete -c atlas -n "__fish_atlas_using_subcommand fmt" -s v -l verbose -d 'Verbose output with timing information'
complete -c atlas -n "__fish_atlas_using_subcommand fmt" -s q -l quiet -d 'Suppress non-error output'
complete -c atlas -n "__fish_atlas_using_subcommand fmt" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand fmt" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand f" -s c -l config -d 'Path to configuration file' -r -F
complete -c atlas -n "__fish_atlas_using_subcommand f" -l indent-size -d 'Indentation size in spaces (default: 4)' -r
complete -c atlas -n "__fish_atlas_using_subcommand f" -l max-width -d 'Maximum line width (default: 100)' -r
complete -c atlas -n "__fish_atlas_using_subcommand f" -l trailing-commas -d 'Enable or disable trailing commas' -r -f -a "true\t''
false\t''"
complete -c atlas -n "__fish_atlas_using_subcommand f" -l check -d 'Check formatting without modifying files'
complete -c atlas -n "__fish_atlas_using_subcommand f" -s w -l write -d 'Write changes to files (explicit mode)'
complete -c atlas -n "__fish_atlas_using_subcommand f" -s v -l verbose -d 'Verbose output with timing information'
complete -c atlas -n "__fish_atlas_using_subcommand f" -s q -l quiet -d 'Suppress non-error output'
complete -c atlas -n "__fish_atlas_using_subcommand f" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand f" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand profile" -l threshold -d 'Hotspot detection threshold percentage' -r
complete -c atlas -n "__fish_atlas_using_subcommand profile" -s o -l output -d 'Save profile report to this file' -r
complete -c atlas -n "__fish_atlas_using_subcommand profile" -l summary -d 'Print summary only (no detailed report)'
complete -c atlas -n "__fish_atlas_using_subcommand profile" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand profile" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand test" -l dir -d 'Test directory (defaults to current directory)' -r -F
complete -c atlas -n "__fish_atlas_using_subcommand test" -l sequential -d 'Run tests sequentially instead of parallel'
complete -c atlas -n "__fish_atlas_using_subcommand test" -s v -l verbose -d 'Verbose output (show all test names)'
complete -c atlas -n "__fish_atlas_using_subcommand test" -l no-color -d 'Disable colored output'
complete -c atlas -n "__fish_atlas_using_subcommand test" -l json -d 'Output in JSON format'
complete -c atlas -n "__fish_atlas_using_subcommand test" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand test" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand t" -l dir -d 'Test directory (defaults to current directory)' -r -F
complete -c atlas -n "__fish_atlas_using_subcommand t" -l sequential -d 'Run tests sequentially instead of parallel'
complete -c atlas -n "__fish_atlas_using_subcommand t" -s v -l verbose -d 'Verbose output (show all test names)'
complete -c atlas -n "__fish_atlas_using_subcommand t" -l no-color -d 'Disable colored output'
complete -c atlas -n "__fish_atlas_using_subcommand t" -l json -d 'Output in JSON format'
complete -c atlas -n "__fish_atlas_using_subcommand t" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand t" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand debug" -s b -l breakpoint -d 'Set breakpoints at line numbers (can be repeated)' -r
complete -c atlas -n "__fish_atlas_using_subcommand debug" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand debug" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand d" -s b -l breakpoint -d 'Set breakpoints at line numbers (can be repeated)' -r
complete -c atlas -n "__fish_atlas_using_subcommand d" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand d" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand lsp" -l port -d 'Port for TCP mode' -r
complete -c atlas -n "__fish_atlas_using_subcommand lsp" -l host -d 'Bind address for TCP mode' -r
complete -c atlas -n "__fish_atlas_using_subcommand lsp" -l tcp -d 'Use TCP mode instead of stdio'
complete -c atlas -n "__fish_atlas_using_subcommand lsp" -s v -l verbose -d 'Enable verbose logging'
complete -c atlas -n "__fish_atlas_using_subcommand lsp" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand lsp" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c atlas -n "__fish_atlas_using_subcommand completions" -s V -l version -d 'Print version'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "run" -d 'Run an Atlas source file'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "check" -d 'Type-check an Atlas source file without running'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "build" -d 'Build an Atlas project'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "repl" -d 'Start an interactive REPL'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "ast" -d 'Dump AST to JSON'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "typecheck" -d 'Dump typecheck information to JSON'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "fmt" -d 'Format Atlas source files'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "profile" -d 'Profile an Atlas source file (VM execution analysis)'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "test" -d 'Run tests in a directory'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "debug" -d 'Debug an Atlas program interactively'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "lsp" -d 'Start the Atlas Language Server'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "completions" -d 'Generate shell completions'
complete -c atlas -n "__fish_atlas_using_subcommand help; and not __fish_seen_subcommand_from run check build repl ast typecheck fmt profile test debug lsp completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
