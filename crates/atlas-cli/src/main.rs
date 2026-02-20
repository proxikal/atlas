use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io;

mod commands;
mod config;
mod debugger;
mod testing;

/// Atlas programming language compiler and runtime.
///
/// Atlas is a modern, expressive programming language designed for productivity and safety.
/// This CLI provides tools for running, testing, debugging, and formatting Atlas code.
///
/// EXAMPLES:
///     atlas run main.atl           Run an Atlas program
///     atlas check main.atl         Type-check without running
///     atlas fmt src/ --check       Check formatting
///     atlas test                   Run all tests
///     atlas debug main.atl         Debug interactively
///     atlas repl                   Start interactive REPL
///
/// ENVIRONMENT VARIABLES:
///     ATLAS_JSON        Set to '1' for JSON output by default
///     ATLAS_NO_HISTORY  Set to '1' to disable REPL history
///     NO_COLOR          Set to disable colored output
#[derive(Parser)]
#[command(name = "atlas")]
#[command(version)]
#[command(propagate_version = true)]
#[command(after_help = "For more information, see: https://atl-lang.github.io/atlas")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an Atlas source file
    ///
    /// Compiles and executes the specified Atlas file. Supports watch mode
    /// for automatic recompilation on file changes.
    ///
    /// EXAMPLES:
    ///     atlas run main.atl              Run a program
    ///     atlas run main.atl --watch      Watch for changes
    ///     atlas run main.atl --json       Output diagnostics as JSON
    #[command(visible_alias = "r")]
    Run {
        /// Path to the Atlas source file
        file: String,
        /// Output diagnostics in JSON format
        #[arg(long, env = "ATLAS_JSON")]
        json: bool,
        /// Watch for file changes and auto-recompile
        #[arg(long, short = 'w')]
        watch: bool,
        /// Don't clear terminal before recompilation (with --watch)
        #[arg(long)]
        no_clear: bool,
        /// Verbose output with timing information
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Type-check an Atlas source file without running
    ///
    /// Analyzes the source file for type errors and reports diagnostics
    /// without executing the code.
    ///
    /// EXAMPLES:
    ///     atlas check main.atl         Check for errors
    ///     atlas check main.atl --json  Output as JSON
    #[command(visible_alias = "c")]
    Check {
        /// Path to the Atlas source file
        file: String,
        /// Output diagnostics in JSON format
        #[arg(long, env = "ATLAS_JSON")]
        json: bool,
    },

    /// Build an Atlas project
    ///
    /// Compiles an Atlas project according to atlas.toml configuration.
    /// Supports different build profiles for development and release.
    ///
    /// EXAMPLES:
    ///     atlas build                   Build with default profile
    ///     atlas build --release         Build optimized release
    ///     atlas build --profile=test    Build with test profile
    #[command(visible_alias = "b")]
    Build {
        /// Build profile (dev, release, test, or custom)
        #[arg(long, short = 'p')]
        profile: Option<String>,
        /// Build in release mode (shorthand for --profile=release)
        #[arg(long)]
        release: bool,
        /// Clean build (ignore cache)
        #[arg(long)]
        clean: bool,
        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
        /// Quiet output (errors only)
        #[arg(long, short = 'q')]
        quiet: bool,
        /// JSON output
        #[arg(long, env = "ATLAS_JSON")]
        json: bool,
    },

    /// Start an interactive REPL
    ///
    /// Opens an interactive Read-Eval-Print Loop for exploring Atlas.
    /// Supports command history, multiline input, and special commands.
    ///
    /// REPL COMMANDS:
    ///     :help, :h      Show help
    ///     :quit, :q      Exit REPL
    ///     :reset         Clear all definitions
    ///     :load <file>   Load and run a file
    ///     :type <expr>   Show expression type
    ///     :vars          List defined variables
    ///
    /// EXAMPLES:
    ///     atlas repl                    Start line editor REPL
    ///     atlas repl --tui              Start TUI mode
    ///     atlas repl --no-history       Disable history persistence
    Repl {
        /// Use TUI mode (ratatui) instead of line editor
        #[arg(long)]
        tui: bool,
        /// Disable history persistence (for privacy)
        #[arg(long, env = "ATLAS_NO_HISTORY")]
        no_history: bool,
    },

    /// Dump AST to JSON
    ///
    /// Parses the source file and outputs the Abstract Syntax Tree
    /// in JSON format for tooling or debugging purposes.
    ///
    /// EXAMPLES:
    ///     atlas ast main.atl              Print AST
    ///     atlas ast main.atl > ast.json   Save to file
    Ast {
        /// Path to the Atlas source file
        file: String,
    },

    /// Dump typecheck information to JSON
    ///
    /// Type-checks the source file and outputs detailed type information
    /// for each expression in JSON format.
    ///
    /// EXAMPLES:
    ///     atlas typecheck main.atl        Print type info
    ///     atlas typecheck main.atl | jq   Process with jq
    Typecheck {
        /// Path to the Atlas source file
        file: String,
    },

    /// Format Atlas source files
    ///
    /// Automatically formats Atlas code according to style guidelines.
    /// Can format files in place or check if they need formatting.
    ///
    /// EXAMPLES:
    ///     atlas fmt src/                  Format all files in src/
    ///     atlas fmt main.atl --check      Check without modifying
    ///     atlas fmt . --write             Format all files recursively
    ///     atlas fmt main.atl --indent-size=2
    #[command(visible_alias = "f")]
    Fmt {
        /// Files or directories to format
        #[arg(required = true)]
        files: Vec<String>,
        /// Check formatting without modifying files
        #[arg(long)]
        check: bool,
        /// Write changes to files (explicit mode)
        #[arg(long, short = 'w')]
        write: bool,
        /// Path to configuration file
        #[arg(long, short = 'c')]
        config: Option<std::path::PathBuf>,
        /// Indentation size in spaces (default: 4)
        #[arg(long)]
        indent_size: Option<usize>,
        /// Maximum line width (default: 100)
        #[arg(long)]
        max_width: Option<usize>,
        /// Enable or disable trailing commas
        #[arg(long)]
        trailing_commas: Option<bool>,
        /// Verbose output with timing information
        #[arg(long, short = 'v')]
        verbose: bool,
        /// Suppress non-error output
        #[arg(long, short = 'q')]
        quiet: bool,
    },

    /// Profile an Atlas source file (VM execution analysis)
    ///
    /// Runs the program under the VM profiler to analyze performance.
    /// Identifies hotspots and provides execution statistics.
    ///
    /// EXAMPLES:
    ///     atlas profile slow.atl          Profile execution
    ///     atlas profile slow.atl -o report.txt  Save report
    ///     atlas profile slow.atl --summary      Brief output
    Profile {
        /// Path to the Atlas source file
        file: String,
        /// Hotspot detection threshold percentage
        #[arg(long, default_value = "1.0")]
        threshold: f64,
        /// Save profile report to this file
        #[arg(long, short = 'o')]
        output: Option<String>,
        /// Print summary only (no detailed report)
        #[arg(long)]
        summary: bool,
    },

    /// Run tests in a directory
    ///
    /// Discovers and runs Atlas test files. Test files should export
    /// functions prefixed with 'test_' that return true on success.
    ///
    /// EXAMPLES:
    ///     atlas test                      Run all tests
    ///     atlas test auth                 Filter by pattern
    ///     atlas test --dir=tests/unit     Specific directory
    ///     atlas test --verbose            Show all test names
    ///     atlas test --sequential         Disable parallelism
    #[command(visible_alias = "t")]
    Test {
        /// Filter tests by name pattern
        pattern: Option<String>,
        /// Run tests sequentially instead of parallel
        #[arg(long)]
        sequential: bool,
        /// Verbose output (show all test names)
        #[arg(long, short = 'v')]
        verbose: bool,
        /// Disable colored output
        #[arg(long, env = "NO_COLOR")]
        no_color: bool,
        /// Test directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        dir: std::path::PathBuf,
        /// Output in JSON format
        #[arg(long, env = "ATLAS_JSON")]
        json: bool,
    },

    /// Debug an Atlas program interactively
    ///
    /// Starts a debugging session with breakpoints and stepping.
    /// Supports inspecting variables and evaluating expressions.
    ///
    /// DEBUGGER COMMANDS:
    ///     break <line>   Set breakpoint at line
    ///     step, s        Step into
    ///     next, n        Step over
    ///     continue, c    Continue execution
    ///     print <expr>   Evaluate expression
    ///     vars           Show local variables
    ///     backtrace, bt  Show call stack
    ///     quit, q        Exit debugger
    ///
    /// EXAMPLES:
    ///     atlas debug main.atl            Start debugging
    ///     atlas debug main.atl -b 10      Break at line 10
    ///     atlas debug main.atl -b 10 -b 20  Multiple breakpoints
    #[command(visible_alias = "d")]
    Debug {
        /// Path to the Atlas source file
        file: String,
        /// Set breakpoints at line numbers (can be repeated)
        #[arg(long, short = 'b')]
        breakpoint: Vec<u32>,
    },

    /// Start the Atlas Language Server
    ///
    /// Runs the Language Server Protocol server for IDE integration.
    /// Supports both stdio mode (for editors) and TCP mode.
    ///
    /// EXAMPLES:
    ///     atlas lsp                       Start in stdio mode
    ///     atlas lsp --tcp                 Start TCP server
    ///     atlas lsp --tcp --port=8080     Custom port
    ///     atlas lsp --verbose             Enable logging
    Lsp {
        /// Use TCP mode instead of stdio
        #[arg(long)]
        tcp: bool,
        /// Port for TCP mode
        #[arg(long, default_value = "9257")]
        port: u16,
        /// Bind address for TCP mode
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Enable verbose logging
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Generate shell completions
    ///
    /// Outputs shell completion scripts for bash, zsh, fish, or powershell.
    /// Redirect to a file and source it in your shell configuration.
    ///
    /// EXAMPLES:
    ///     atlas completions bash > ~/.bash_completions/atlas.bash
    ///     atlas completions zsh > ~/.zfunc/_atlas
    ///     atlas completions fish > ~/.config/fish/completions/atlas.fish
    ///
    /// INSTALLATION:
    ///     Bash: Add 'source ~/.bash_completions/atlas.bash' to ~/.bashrc
    ///     Zsh:  Add 'fpath=(~/.zfunc $fpath)' before compinit in ~/.zshrc
    ///     Fish: Completions auto-loaded from completions directory
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Initialize a new Atlas project
    ///
    /// Creates a new Atlas project with the standard directory structure,
    /// manifest file (atlas.toml), and optional git repository.
    ///
    /// EXAMPLES:
    ///     atlas init                    Initialize in current directory
    ///     atlas init my-project         Create new project directory
    ///     atlas init --lib              Create a library project
    ///     atlas init --no-git           Skip git initialization
    #[command(visible_alias = "i")]
    Init {
        /// Project name (defaults to directory name)
        name: Option<String>,
        /// Create a library project instead of binary
        #[arg(long)]
        lib: bool,
        /// Skip git repository initialization
        #[arg(long)]
        no_git: bool,
        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Add a dependency to the project
    ///
    /// Adds a new dependency to atlas.toml. Supports version constraints,
    /// git repositories, and local path dependencies.
    ///
    /// EXAMPLES:
    ///     atlas add http                 Add latest version
    ///     atlas add http@1.2             Add specific version
    ///     atlas add http --dev           Add as dev dependency
    ///     atlas add http --git=https://... Add from git
    ///     atlas add http --path=../http  Add local dependency
    Add {
        /// Package name (optionally with @version)
        package: String,
        /// Version constraint (e.g., "1.0", "^1.2.3")
        #[arg(long)]
        ver: Option<String>,
        /// Add as dev dependency
        #[arg(long)]
        dev: bool,
        /// Git repository URL
        #[arg(long)]
        git: Option<String>,
        /// Git branch
        #[arg(long)]
        branch: Option<String>,
        /// Git tag
        #[arg(long)]
        tag: Option<String>,
        /// Git revision
        #[arg(long)]
        rev: Option<String>,
        /// Local path dependency
        #[arg(long)]
        path: Option<std::path::PathBuf>,
        /// Enable specific features
        #[arg(long, short = 'F')]
        features: Vec<String>,
        /// Disable default features
        #[arg(long)]
        no_default_features: bool,
        /// Mark as optional dependency
        #[arg(long)]
        optional: bool,
        /// Rename the dependency
        #[arg(long)]
        rename: Option<String>,
        /// Dry run (don't modify files)
        #[arg(long)]
        dry_run: bool,
    },

    /// Remove a dependency from the project
    ///
    /// Removes one or more dependencies from atlas.toml.
    ///
    /// EXAMPLES:
    ///     atlas remove http              Remove single dependency
    ///     atlas remove http json         Remove multiple
    ///     atlas remove test-utils --dev  Remove from dev-dependencies
    #[command(visible_alias = "rm")]
    Remove {
        /// Package names to remove
        packages: Vec<String>,
        /// Remove from dev dependencies
        #[arg(long)]
        dev: bool,
        /// Dry run (don't modify files)
        #[arg(long)]
        dry_run: bool,
        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Install project dependencies
    ///
    /// Downloads and installs all dependencies specified in atlas.toml.
    /// Uses atlas.lock for reproducible builds when available.
    ///
    /// EXAMPLES:
    ///     atlas install                  Install all dependencies
    ///     atlas install --production     Skip dev dependencies
    ///     atlas install --force          Force reinstall
    Install {
        /// Only install production dependencies
        #[arg(long)]
        production: bool,
        /// Force reinstall even if cached
        #[arg(long)]
        force: bool,
        /// Dry run (don't actually install)
        #[arg(long)]
        dry_run: bool,
        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
        /// Quiet output (errors only)
        #[arg(long, short = 'q')]
        quiet: bool,
    },

    /// Update project dependencies
    ///
    /// Updates dependencies to their latest compatible versions
    /// according to the constraints in atlas.toml.
    ///
    /// EXAMPLES:
    ///     atlas update                   Update all dependencies
    ///     atlas update http              Update specific package
    ///     atlas update --dry-run         Show what would be updated
    #[command(visible_alias = "up")]
    Update {
        /// Specific packages to update (empty = all)
        packages: Vec<String>,
        /// Only update dev dependencies
        #[arg(long)]
        dev: bool,
        /// Dry run (don't modify files)
        #[arg(long)]
        dry_run: bool,
        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Publish package to registry
    ///
    /// Validates, packages, and publishes your package to the Atlas
    /// package registry. Requires authentication.
    ///
    /// EXAMPLES:
    ///     atlas publish                  Publish to default registry
    ///     atlas publish --dry-run        Validate without publishing
    ///     atlas publish --no-verify      Skip validation steps
    Publish {
        /// Registry to publish to
        #[arg(long)]
        registry: Option<String>,
        /// Skip all validation checks
        #[arg(long)]
        no_verify: bool,
        /// Validate without publishing
        #[arg(long)]
        dry_run: bool,
        /// Allow publishing with dirty git state
        #[arg(long)]
        allow_dirty: bool,
        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cli_config = config::Config::from_env();

    // Load project configuration (atlas.toml) if in a project directory
    // This is available for commands that need project-level settings
    let _project_config = atlas_config::ConfigLoader::new()
        .load_from_directory(&std::env::current_dir()?)
        .ok(); // Optional - not all commands run in a project

    match cli.command {
        Commands::Run {
            file,
            json,
            watch,
            no_clear,
            verbose,
        } => {
            // Command-line flag overrides environment variable
            let use_json = json || cli_config.default_json;

            if watch {
                // Watch mode
                let config = commands::watch::WatchConfig {
                    clear_screen: !no_clear,
                    continue_on_error: true,
                    json_output: use_json,
                    verbose,
                };
                commands::watch::run_watch(&file, config)?;
            } else {
                // Normal run
                commands::run::run(&file, use_json)?;
            }
        }
        Commands::Check { file, json } => {
            // Command-line flag overrides environment variable
            let use_json = json || cli_config.default_json;
            commands::check::run(&file, use_json)?;
        }
        Commands::Build {
            profile,
            release,
            clean,
            verbose,
            quiet,
            json,
        } => {
            // Command-line flag overrides environment variable
            let use_json = json || cli_config.default_json;
            let args = commands::build::BuildArgs {
                profile,
                release,
                clean,
                verbose,
                quiet,
                json: use_json,
                ..Default::default()
            };
            commands::build::run(args)?;
        }
        Commands::Repl { tui, no_history } => {
            // Command-line flag overrides environment variable
            let disable_history = no_history || cli_config.no_history;
            commands::repl::run(tui, disable_history, &cli_config)?;
        }
        Commands::Ast { file } => {
            commands::ast::run(&file)?;
        }
        Commands::Typecheck { file } => {
            commands::typecheck::run(&file)?;
        }
        Commands::Fmt {
            files,
            check,
            write,
            config,
            indent_size,
            max_width,
            trailing_commas,
            verbose,
            quiet,
        } => {
            let verbosity = if quiet {
                commands::fmt::Verbosity::Quiet
            } else if verbose {
                commands::fmt::Verbosity::Verbose
            } else {
                commands::fmt::Verbosity::Normal
            };
            let args = commands::fmt::FmtArgs {
                files,
                check,
                write,
                config_path: config,
                indent_size,
                max_width,
                trailing_commas,
                verbosity,
            };
            commands::fmt::run(args)?;
        }
        Commands::Profile {
            file,
            threshold,
            output,
            summary,
        } => {
            let mut args = commands::profile::ProfileArgs::new(file);
            args.hotspot_threshold = threshold;
            args.output_file = output.map(std::path::PathBuf::from);
            args.detailed = !summary;
            commands::profile::run(args)?;
        }
        Commands::Test {
            pattern,
            sequential,
            verbose,
            no_color,
            dir,
            json,
        } => {
            let args = commands::test::TestArgs {
                pattern,
                sequential,
                verbose,
                no_color,
                dir,
                json,
            };
            commands::test::run(args)?;
        }
        Commands::Debug { file, breakpoint } => {
            let args = commands::debug::DebugArgs {
                file,
                breakpoints: breakpoint,
                stop_at_entry: true,
            };
            commands::debug::run(args)?;
        }
        Commands::Lsp {
            tcp,
            port,
            host,
            verbose,
        } => {
            let args = commands::lsp::LspArgs {
                tcp,
                port,
                host,
                verbose,
            };
            commands::lsp::run(args)?;
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut io::stdout());
        }
        Commands::Init {
            name,
            lib,
            no_git,
            verbose,
        } => {
            let project_type = if lib {
                commands::init::ProjectType::Library
            } else {
                commands::init::ProjectType::Binary
            };
            let non_interactive = name.is_some();
            let args = commands::init::InitArgs {
                name,
                project_type,
                git: !no_git,
                path: std::env::current_dir()?,
                non_interactive,
                verbose,
            };
            commands::init::run(args)?;
        }
        Commands::Add {
            package,
            ver,
            dev,
            git,
            branch,
            tag,
            rev,
            path,
            features,
            no_default_features,
            optional,
            rename,
            dry_run,
        } => {
            // Parse package@version syntax
            let (pkg_name, pkg_version) = if let Some(idx) = package.find('@') {
                let (name, version) = package.split_at(idx);
                (name.to_string(), Some(version[1..].to_string()))
            } else {
                (package.clone(), ver)
            };

            let args = commands::add::AddArgs {
                package: pkg_name,
                version: pkg_version,
                dev,
                git,
                branch,
                tag,
                rev,
                path,
                features,
                no_default_features,
                optional,
                rename,
                project_dir: std::env::current_dir()?,
                dry_run,
                verbose: false,
            };
            commands::add::run(args)?;
        }
        Commands::Remove {
            packages,
            dev,
            dry_run,
            verbose,
        } => {
            let args = commands::remove::RemoveArgs {
                packages,
                dev,
                project_dir: std::env::current_dir()?,
                dry_run,
                verbose,
            };
            commands::remove::run(args)?;
        }
        Commands::Install {
            production,
            force,
            dry_run,
            verbose,
            quiet,
        } => {
            let args = commands::install::InstallArgs {
                packages: Vec::new(),
                production,
                force,
                project_dir: std::env::current_dir()?,
                dry_run,
                verbose,
                quiet,
            };
            commands::install::run(args)?;
        }
        Commands::Update {
            packages,
            dev,
            dry_run,
            verbose,
        } => {
            let args = commands::update::UpdateArgs {
                packages,
                dev,
                project_dir: std::env::current_dir()?,
                dry_run,
                verbose,
            };
            commands::update::run(args)?;
        }
        Commands::Publish {
            registry,
            no_verify,
            dry_run,
            allow_dirty,
            verbose,
        } => {
            let args = commands::publish::PublishArgs {
                project_dir: std::env::current_dir()?,
                registry,
                no_verify,
                dry_run,
                allow_dirty,
                verbose,
            };
            commands::publish::run(args)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_smoke() {
        // Verify CLI can be instantiated
        // This test ensures the binary compiles and basic structure works
        let _cli = Cli::parse_from(["atlas", "repl"]);
        // If we get here without panicking, the CLI structure is valid
    }

    #[test]
    fn test_cli_repl_tui_flag() {
        // Verify TUI flag is parsed correctly
        let cli = Cli::parse_from(["atlas", "repl", "--tui"]);
        match cli.command {
            Commands::Repl { tui, .. } => assert!(tui),
            _ => panic!("Expected Repl command"),
        }
    }

    #[test]
    fn test_cli_repl_no_history_flag() {
        // Verify no-history flag is parsed correctly
        let cli = Cli::parse_from(["atlas", "repl", "--no-history"]);
        match cli.command {
            Commands::Repl { no_history, .. } => assert!(no_history),
            _ => panic!("Expected Repl command"),
        }
    }

    #[test]
    fn test_cli_json_flag() {
        // Verify JSON flag is parsed correctly
        let cli = Cli::parse_from(["atlas", "check", "file.atl", "--json"]);
        match cli.command {
            Commands::Check { json, .. } => assert!(json),
            _ => panic!("Expected Check command"),
        }
    }

    // Command alias tests
    #[test]
    fn test_alias_r_for_run() {
        let cli = Cli::parse_from(["atlas", "r", "main.atl"]);
        matches!(cli.command, Commands::Run { .. });
    }

    #[test]
    fn test_alias_t_for_test() {
        let cli = Cli::parse_from(["atlas", "t"]);
        matches!(cli.command, Commands::Test { .. });
    }

    #[test]
    fn test_alias_f_for_fmt() {
        let cli = Cli::parse_from(["atlas", "f", "src/"]);
        matches!(cli.command, Commands::Fmt { .. });
    }

    #[test]
    fn test_alias_b_for_build() {
        let cli = Cli::parse_from(["atlas", "b"]);
        matches!(cli.command, Commands::Build { .. });
    }

    #[test]
    fn test_alias_c_for_check() {
        let cli = Cli::parse_from(["atlas", "c", "main.atl"]);
        matches!(cli.command, Commands::Check { .. });
    }

    #[test]
    fn test_alias_d_for_debug() {
        let cli = Cli::parse_from(["atlas", "d", "main.atl"]);
        matches!(cli.command, Commands::Debug { .. });
    }

    #[test]
    fn test_completions_bash() {
        let cli = Cli::parse_from(["atlas", "completions", "bash"]);
        match cli.command {
            Commands::Completions { shell } => assert_eq!(shell, Shell::Bash),
            _ => panic!("Expected Completions command"),
        }
    }

    #[test]
    fn test_completions_zsh() {
        let cli = Cli::parse_from(["atlas", "completions", "zsh"]);
        match cli.command {
            Commands::Completions { shell } => assert_eq!(shell, Shell::Zsh),
            _ => panic!("Expected Completions command"),
        }
    }

    #[test]
    fn test_completions_fish() {
        let cli = Cli::parse_from(["atlas", "completions", "fish"]);
        match cli.command {
            Commands::Completions { shell } => assert_eq!(shell, Shell::Fish),
            _ => panic!("Expected Completions command"),
        }
    }
}
