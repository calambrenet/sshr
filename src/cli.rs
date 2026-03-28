use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
/// sshr — A modern SSH connection manager
///
/// Manage your SSH connections with style. Connect directly with
/// `sshr <host>` or use subcommands for advanced features.
#[derive(Parser, Debug)]
#[command(
    name = "sshr",
    version,                      // Usa la versión de Cargo.toml
    about,                        // Usa el doc comment de arriba
    long_about = None,
    arg_required_else_help = true, // Muestra ayuda si no hay argumentos
    propagate_version = true,      // --version funciona en subcomandos
)]
pub struct Cli {
    /// Path to SSH config file
    #[arg(
        short = 'F',              // -F (igual que ssh)
        long = "config-file",
        global = true,             // Disponible en todos los subcomandos
        env = "SSHR_CONFIG",      // También configurable por env var
        default_value = "~/.ssh/config"
    )]
    pub config_file: PathBuf,
    /// Disable colored output
    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,
    /// Output format
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = OutputFormat::Text
    )]
    pub format: OutputFormat,
    #[command(subcommand)]
    pub command: Command,
}
/// Formatos de salida soportados
#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum OutputFormat {
    /// Salida formateada para humanos (por defecto)
    Text,
    /// JSON para scripting y pipes
    Json,
    /// CSV para importar en hojas de cálculo
    Csv,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List all configured SSH hosts
    #[command(alias = "ls")]
    List(ListArgs),
    /// Connect to an SSH host
    #[command(alias = "c")]
    Connect(ConnectArgs),
    /// Add a new SSH host to config
    Add(AddArgs),
    /// Remove an SSH host from config
    #[command(alias = "rm")]
    Remove(RemoveArgs),
    /// Fuzzy search across all hosts
    #[command(alias = "s")]
    Search(SearchArgs),
    /// Show detailed information for a host
    Show(ShowArgs),
    /// Lint and validate SSH config for errors
    #[command(alias = "check")]
    Lint(LintArgs),
    /// Manage and audit SSH keys
    Keys(KeysArgs),
    /// Manage known_hosts entries
    Trust(TrustArgs),
    /// Manage SSH port-forwarding tunnels
    Tunnel(TunnelArgs),
    /// Show connection history
    #[command(alias = "log")]
    History(HistoryArgs),
    /// Copy files via SCP/SFTP
    #[command(alias = "cp")]
    Transfer(TransferArgs),
    /// Show active SSH connections and tunnels
    Status(StatusArgs),
    /// Generate shell completions
    Completions(CompletionsArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Show verbose output with all host details
    #[arg(short, long)]
    pub verbose: bool,
    /// Filter by tag (can be used multiple times)
    #[arg(short, long)]
    pub tag: Vec<String>,
    /// Sort by field
    #[arg(long, value_enum, default_value_t = SortField::Name)]
    pub sort: SortField,
    /// Reverse sort order
    #[arg(short, long)]
    pub reverse: bool,
}
#[derive(ValueEnum, Clone, Debug)]
pub enum SortField {
    Name,
    Hostname,
    User,
    Port,
    LastUsed,
}

#[derive(Args, Debug)]
pub struct ConnectArgs {
    /// Host name or alias to connect to
    pub host: String,
    /// Override the SSH port
    #[arg(short, long)]
    pub port: Option<u16>,
    /// Override the SSH user
    #[arg(short, long)]
    pub user: Option<String>,
    /// Enable verbose SSH output (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Enable persistent connection with auto-reconnect
    #[arg(long)]
    pub persistent: bool,
    /// Accept changed host key without prompting
    #[arg(long)]
    pub trust: bool,
    /// Additional SSH arguments passed directly to ssh
    /// Example: sshr connect prod -- -L 8080:localhost:80
    #[arg(last = true)]
    pub ssh_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// Name/alias for the host
    pub name: String,
    /// Hostname or IP address
    pub hostname: String,
    /// SSH user
    #[arg(short, long)]
    pub user: Option<String>,
    /// SSH port (default: 22)
    #[arg(short, long, default_value_t = 22)]
    pub port: u16,
    /// Path to identity file (SSH key)
    #[arg(short, long)]
    pub identity_file: Option<PathBuf>,
    /// Jump host (ProxyJump)
    #[arg(short = 'J', long)]
    pub jump: Option<String>,
    /// Tags for organizing (comma-separated: web,production,eu)
    #[arg(short, long, value_delimiter = ',')]
    pub tags: Vec<String>,
    /// ServerAliveInterval in seconds (default: 60)
    #[arg(long, default_value_t = 60)]
    pub keep_alive: u64,
    /// Additional SSH options as key=value pairs
    /// Example: --option Compression=yes --option Ciphers=aes256-ctr
    #[arg(short = 'o', long = "option")]
    pub extra_options: Vec<String>,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Host name to remove
    pub name: String,
    /// Skip confirmation prompt
    #[arg(short, long)]
    pub force: bool,
}
#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query (fuzzy matched against all fields)
    pub query: String,
    /// Maximum number of results to show
    #[arg(short, long, default_value_t = 10)]
    pub limit: usize,
}
#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Host name to show details for
    pub name: String,
}

#[derive(Args, Debug)]
pub struct KeysArgs {
    #[command(subcommand)]
    pub action: KeysAction,
}
#[derive(Subcommand, Debug)]
pub enum KeysAction {
    /// List all SSH keys with their associated hosts
    #[command(alias = "ls")]
    List,
    /// Audit keys for security issues
    Audit {
        /// Automatically fix issues when possible
        #[arg(long)]
        fix: bool,
    },
    /// Show which hosts use a specific key
    Which {
        /// Path to the key file
        key: PathBuf,
    },
    /// Generate a new Ed25519 key pair
    Generate {
        /// Name for the new key (will be saved as ~/.ssh/<name>)
        name: String,
        /// Comment embedded in the key
        #[arg(short, long)]
        comment: Option<String>,
        /// Don't set a passphrase
        #[arg(long)]
        no_passphrase: bool,
    },
}
#[derive(Args, Debug)]
pub struct TunnelArgs {
    #[command(subcommand)]
    pub action: TunnelAction,
}
#[derive(Subcommand, Debug)]
pub enum TunnelAction {
    /// Create a new SSH tunnel
    Add {
        /// Host to tunnel through
        host: String,
        /// Local port to bind
        #[arg(short, long)]
        local: u16,
        /// Remote target (host:port format)
        #[arg(short, long)]
        remote: String,
        /// Run the tunnel in background
        #[arg(short, long)]
        background: bool,
        /// Name for this tunnel (for easier management)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// List all active tunnels
    #[command(alias = "ls")]
    List,
    /// Stop a specific tunnel
    Stop {
        /// Tunnel name or host
        target: String,
    },
    /// Stop all active tunnels
    StopAll,
}

#[derive(Args, Debug)]
pub struct LintArgs {
    /// Show warnings in addition to errors
    #[arg(short, long)]
    pub warnings: bool,
    /// Only check, don't suggest fixes
    #[arg(long)]
    pub strict: bool,
}
#[derive(Args, Debug)]
pub struct TrustArgs {
    /// Host to manage trust for
    pub host: String,
    /// Remove old key and accept new one
    #[arg(long)]
    pub reset: bool,
    /// Show fingerprint history for this host
    #[arg(long)]
    pub history: bool,
}
#[derive(Args, Debug)]
pub struct HistoryArgs {
    /// Number of entries to show
    #[arg(short, long, default_value_t = 20)]
    pub limit: usize,
    /// Filter by host name
    #[arg(short = 'H', long)]
    pub host: Option<String>,
    /// Show entries from the last N days
    #[arg(long)]
    pub days: Option<u64>,
    /// Clear all history
    #[arg(long)]
    pub clear: bool,
}
#[derive(Args, Debug)]
pub struct TransferArgs {
    /// Source path (local or remote: host:/path)
    pub source: String,
    /// Destination path (local or remote: host:/path)
    pub destination: String,
    /// Recursive copy for directories
    #[arg(short, long)]
    pub recursive: bool,
    /// Show transfer progress
    #[arg(short, long)]
    pub progress: bool,
    /// Resume interrupted transfer
    #[arg(long)]
    pub resume: bool,
}
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Show detailed status for each connection
    #[arg(short, long)]
    pub verbose: bool,
}
#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}
