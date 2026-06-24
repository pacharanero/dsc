use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dsc")]
#[command(about = "Discourse CLI", long_about = None)]
#[command(next_display_order = None)]
pub struct Cli {
    /// Path to the config file. If omitted, `dsc` consults `$DSC_CONFIG`,
    /// then searches `./dsc.toml`, `$DSC_CONFIG_HOME/dsc.toml`
    /// (default `~/.config/dsc/dsc.toml`), then system locations.
    /// Errors if the given file does not exist (no silent fallthrough).
    /// See `dsc config` for the active selection.
    #[arg(long, short = 'c')]
    pub config: Option<PathBuf>,
    /// Describe destructive actions without sending them. Read-only commands
    /// ignore the flag.
    #[arg(long, short = 'n', global = true)]
    pub dry_run: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum Commands {
    /// List configured Discourses.
    #[command(visible_alias = "ls")]
    #[command(after_help = "Examples:
  dsc list
  dsc list --tags production -f json")]
    List {
        /// Output format for the listing.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: OutputFormat,
        /// Filter by tags (comma/semicolon separated, match-any).
        #[arg(long, value_name = "tag1,tag2")]
        tags: Option<String>,
        /// Open each listed Discourse base URL in a browser tab/window.
        #[arg(long, short = 'o')]
        open: bool,
        /// Include empty results and verbose listing details where supported.
        #[arg(long, short = 'v')]
        verbose: bool,
        #[command(subcommand)]
        command: Option<ListCommand>,
    },
    /// Add one or more Discourses to the config.
    #[command(visible_alias = "a")]
    #[command(after_help = "Examples:
  dsc add myforum
  dsc add forum-a,forum-b -i")]
    Add {
        /// Comma-separated discourse names to add.
        names: String,
        /// Prompt for additional optional fields while adding.
        #[arg(long, short = 'i')]
        interactive: bool,
    },
    /// Import Discourses from a file or stdin.
    #[command(visible_alias = "imp")]
    #[command(after_help = "Examples:
  dsc import forums.csv
  cat forums.txt | dsc import")]
    Import {
        /// Path to import input (text/CSV). Reads stdin when omitted.
        path: Option<PathBuf>,
    },
    /// Run remote OS + Discourse update workflow for one or all Discourses.
    #[command(visible_alias = "up")]
    #[command(after_help = "Examples:
  dsc update myforum
  dsc update all -p   # update every forum in parallel")]
    Update {
        /// Discourse name, or 'all' to update every configured Discourse.
        name: String,
        /// Parallel update mode for `dsc update all`.
        #[arg(long, short = 'p')]
        parallel: bool,
        /// Maximum workers when parallel mode is enabled (default: 3).
        #[arg(long, short = 'm')]
        max: Option<usize>,
        /// Disable changelog posting (posting prompt is on by default).
        #[arg(long = "no-changelog", action = ArgAction::SetFalse, default_value_t = true)]
        post_changelog: bool,
        /// Auto-confirm changelog posting prompt (non-interactive mode).
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Manage custom emoji.
    #[command(visible_alias = "em")]
    #[command(after_help = "Examples:
  dsc emoji push myforum ./emoji/    # bulk-upload a folder
  dsc emoji list myforum")]
    Emoji {
        #[command(subcommand)]
        command: EmojiCommand,
    },
    /// Pull/push/sync topics as local Markdown.
    #[command(visible_alias = "t")]
    #[command(after_help = "Examples:
  dsc topic pull myforum 123 topic.md
  dsc topic push myforum 123 topic.md
  dsc topic title myforum 123 \"A clearer title\"")]
    Topic {
        #[command(subcommand)]
        command: TopicCommand,
    },
    /// List/copy/pull/push categories.
    #[command(visible_alias = "cat")]
    #[command(after_help = "Examples:
  dsc category pull myforum 34 ./playbook/
  dsc category push -n myforum 34 ./playbook/   # -n previews the plan")]
    Category {
        #[command(subcommand)]
        command: CategoryCommand,
    },
    /// List/inspect/copy groups.
    #[command(visible_alias = "grp")]
    #[command(after_help = "Examples:
  dsc group list myforum
  dsc group info myforum staff")]
    Group {
        #[command(subcommand)]
        command: GroupCommand,
    },
    /// Operations that act from a user's perspective.
    #[command(visible_alias = "usr")]
    #[command(after_help = "Examples:
  dsc user list myforum -f json
  dsc user info myforum alice
  dsc user activity myforum alice")]
    User {
        #[command(subcommand)]
        command: UserCommand,
    },
    /// Send invites — single or bulk from a file.
    #[command(visible_alias = "inv")]
    #[command(after_help = "Examples:
  dsc invite send myforum newuser@example.com
  dsc invite bulk myforum emails.txt")]
    Invite {
        #[command(subcommand)]
        command: InviteCommand,
    },
    /// Manage API keys (admin scope).
    #[command(visible_alias = "ak")]
    #[command(after_help = "Examples:
  dsc api-key list myforum
  dsc api-key create myforum ci-bot")]
    ApiKey {
        #[command(subcommand)]
        command: ApiKeyCommand,
    },
    /// Send and list private messages.
    #[command(visible_alias = "msg")]
    #[command(after_help = "Examples:
  dsc pm list myforum alice
  dsc pm send myforum alice -t Greetings body.md")]
    Pm {
        #[command(subcommand)]
        command: PmCommand,
    },
    /// Create/list/restore backups.
    #[command(visible_alias = "bk")]
    #[command(after_help = "Examples:
  dsc backup create myforum
  dsc backup pull myforum backup.tar.gz")]
    Backup {
        #[command(subcommand)]
        command: BackupCommand,
    },
    /// List/pull/push color palettes.
    #[command(visible_alias = "pal")]
    #[command(after_help = "Examples (now lives under `dsc theme palette`):
  dsc theme palette list myforum
  dsc theme palette pull myforum 2 palette.json")]
    Palette {
        #[command(subcommand)]
        command: PaletteCommand,
    },
    /// List/install/remove plugins.
    #[command(visible_alias = "plg")]
    #[command(after_help = "Examples:
  dsc plugin list myforum
  dsc plugin install myforum https://github.com/org/plugin")]
    Plugin {
        #[command(subcommand)]
        command: PluginCommand,
    },
    /// List/install/remove/pull/push/duplicate themes.
    #[command(visible_alias = "th")]
    #[command(after_help = "Examples:
  dsc theme list myforum
  dsc theme show myforum 11
  dsc theme setting set myforum 14 links_position left")]
    Theme {
        #[command(subcommand)]
        command: ThemeCommand,
    },
    /// Get, set, list, diff, audit, and snapshot site settings.
    ///
    /// To discover what settings exist, `dsc setting pull` writes a
    /// self-documenting catalog of every setting (value, default, type,
    /// category, and Discourse's own description) - the reference guide for
    /// what is available and adjustable.
    #[command(visible_alias = "set")]
    #[command(after_help = "Examples:
  dsc setting pull myforum settings.yaml   # catalog EVERY setting + descriptions (start here)
  dsc setting get myforum title
  dsc setting set myforum login_required true
  dsc setting audit login_required         # compare one setting across all forums")]
    Setting {
        #[command(subcommand)]
        command: SettingCommand,
    },
    /// Export everything a forum holds about one person into a reviewable
    /// Subject Access Request (SAR / GDPR Art. 15) bundle. Single forum.
    #[command(after_help = "Examples:
  dsc sar myforum jane@example.com
  dsc sar myforum jane-doe --messages   # include private messages")]
    Sar {
        /// Discourse name.
        discourse: String,
        /// Subject: a username or an email address.
        user: String,
        /// Output directory (default `sar-<username>-<date>/`).
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
        /// Also collect the subject's private messages. Off by default: PMs
        /// contain third-party personal data and need a disclose/redact
        /// judgement. Written with a REVIEW REQUIRED banner when included.
        #[arg(long)]
        messages: bool,
    },
    /// Manage the tag taxonomy: list/pull/push tags and tag groups.
    #[command(visible_alias = "tg")]
    #[command(after_help = "Examples:
  dsc tag pull myforum tags.yaml
  dsc tag rename myforum old-tag new-tag")]
    Tag {
        #[command(subcommand)]
        command: TagCommand,
    },
    /// Post-level operations: edit / delete / move.
    #[command(visible_alias = "po")]
    #[command(after_help = "Examples:
  dsc post edit myforum 456 body.md
  dsc post move myforum 456 789")]
    Post {
        #[command(subcommand)]
        command: PostCommand,
    },
    /// Open a Discourse in the default browser.
    #[command(visible_alias = "o")]
    #[command(after_help = "Examples:
  dsc open myforum")]
    Open {
        /// Discourse name.
        discourse: String,
    },
    /// Harden a fresh Ubuntu server reachable via `ssh root@host`.
    ///
    /// **Stage 1 (current):** creates a non-root sudo user, installs the
    /// given pubkey to their authorized_keys, and verifies the new-user
    /// SSH login works. Does NOT yet tighten sshd_config, install Docker
    /// / fail2ban / etc — those come in follow-up releases.
    ///
    /// Defaults can be overridden in the `[harden]` block of dsc.toml;
    /// the flags below override that block on a per-run basis.
    #[command(visible_alias = "hd")]
    #[command(after_help = "Examples:
  dsc harden 203.0.113.10 --new-user discourse --pubkey-file ~/.ssh/id_ed25519.pub")]
    Harden {
        /// Target hostname or IP (reachable via SSH).
        host: String,
        /// Username to SSH in as initially. Defaults to `root`, which is
        /// what a fresh cloud-provisioned box typically has.
        #[arg(long, default_value = "root")]
        ssh_user: String,
        /// Username for the new sudo-enabled non-root account. Overrides
        /// `[harden].new_user` from dsc.toml. Built-in default: `discourse`.
        #[arg(long)]
        new_user: Option<String>,
        /// SSH port to move the daemon to in stage 2. Overrides
        /// `[harden].ssh_port`. Built-in default: 2227. Parsed now so the
        /// CLI is stable; not yet applied in stage 1.
        #[arg(long)]
        ssh_port: Option<u16>,
        /// Path to an SSH public key file whose contents will be added to
        /// the new user's authorized_keys. A typical value is
        /// `~/.ssh/<hostname>.pub` — the per-server keypair pattern in
        /// the Bawmedical hardening playbook.
        #[arg(long)]
        pubkey_file: PathBuf,
    },
    /// Community-health analytics — growth, activity, and health metrics
    /// for a Discourse, with optional period-over-period comparison.
    ///
    /// See `spec/analytics.md` for the full spec. v1 ships every metric
    /// that maps onto a single `/admin/reports/{id}.json` endpoint;
    /// derivation-heavy ones (e.g. lost regulars, top-10 share) print
    /// `— (n/i)` until follow-up implementation lands.
    #[command(visible_alias = "stats")]
    #[command(after_help = "Examples:
  dsc analytics myforum
  dsc analytics myforum --section growth --since 30d")]
    Analytics {
        /// Discourse name.
        discourse: String,
        /// Window to report on. Same syntax as `dsc user activity --since`
        /// (e.g. `7d`, `24h`, `1m`, ISO-8601). Ignored when `--snapshot`
        /// is set. Default: 30d.
        #[arg(long, short = 's', default_value = "30d")]
        since: String,
        /// Also fetch the immediately preceding window of equal length and
        /// show a delta column. Mutually exclusive with `--snapshot`.
        #[arg(long, short = 'c', conflicts_with = "snapshot")]
        compare: bool,
        /// Multi-window snapshot mode. Reports each metric across several
        /// preset windows (`--periods`) so you see growth/health trends
        /// at a glance. Replaces `--since` + `--compare`.
        #[arg(long)]
        snapshot: bool,
        /// Comma-separated periods for `--snapshot`. Default: `24h,7d,30d,1y`.
        #[arg(long, requires = "snapshot")]
        periods: Option<String>,
        /// Restrict output to one section.
        #[arg(long, value_enum, default_value = "all")]
        section: SectionArg,
        /// Output format. `table` is DuckDB-style box-drawing; falls
        /// through to `text` automatically when stdout isn't a TTY.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: AnalyticsFormat,
    },
    /// Search topics on a Discourse.
    #[command(visible_alias = "s")]
    #[command(after_help = "Examples:
  dsc search myforum \"status:open category:bugs\"
  dsc search myforum @alice -f json")]
    Search {
        /// Discourse name.
        discourse: String,
        /// Search query (passed through verbatim, including any
        /// Discourse filter syntax like `category:foo` or `@user`).
        query: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Upload a file. Prints the resulting upload:// short URL by default.
    #[command(visible_alias = "u")]
    #[command(after_help = "Examples:
  dsc upload myforum ./diagram.png")]
    Upload {
        /// Discourse name.
        discourse: String,
        /// Path to the file to upload.
        file: PathBuf,
        /// Discourse upload context. Default `composer` is correct for
        /// embedding in posts; other values include `avatar`,
        /// `profile_background`, `card_background`, `custom_emoji`.
        #[arg(long, short = 't', default_value = "composer")]
        upload_type: String,
        /// Output format. Text mode prints just the short URL.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Inspect and validate configuration.
    #[command(visible_alias = "cfg")]
    #[command(after_help = "Examples:
  dsc config         # show active config + search order
  dsc config check   # probe API auth + SSH for every forum")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommand>,
    },
    /// Generate shell completion scripts.
    #[command(visible_alias = "comp")]
    #[command(after_help = "Examples:
  dsc completions zsh > _dsc
  dsc completions bash > dsc.bash")]
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: CompletionShell,
        /// Output directory. Prints to stdout when omitted.
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
    },
    /// Generate man pages for `dsc` and every subcommand.
    ///
    /// Writes one ROFF-formatted file per (sub)command (e.g. `dsc.1`,
    /// `dsc-tag-pull.1`) into the given directory. Distro packagers
    /// install these into section 1 of the man path. Run `gzip -9` on
    /// the output if your packaging convention expects compressed pages.
    #[command(visible_alias = "manpages")]
    #[command(after_help = "Examples:
  dsc man --dir ./man")]
    Man {
        /// Output directory. Required - this command always writes to disk.
        #[arg(long, short = 'd')]
        dir: PathBuf,
    },
    /// Print the dsc version.
    #[command(visible_alias = "ver")]
    /// Print dsc's own version, or a configured forum's Discourse version + commit.
    #[command(after_help = "Examples:
  dsc version         # dsc's own version
  dsc version accm    # the forum's live Discourse version + git commit")]
    Version {
        /// Forum name. When given, print that forum's live Discourse version
        /// and git commit (from /about.json, via the configured API key)
        /// instead of dsc's own version.
        discourse: Option<String>,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum ConfigCommand {
    /// Probe each configured Discourse: API auth and (optionally) SSH reachability.
    #[command(visible_alias = "ck")]
    Check {
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
        /// Skip the SSH reachability probe.
        #[arg(long)]
        skip_ssh: bool,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum ListCommand {
    /// Sort discourse entries by name and rewrite config in-place.
    /// Also inserts placeholder values for unset template keys.
    #[command(visible_alias = "ty")]
    Tidy,
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum EmojiCommand {
    /// Pull all custom emoji from a Discourse into a local directory.
    #[command(visible_alias = "pl")]
    Pull {
        /// Discourse name.
        discourse: String,
        /// Local directory to save emoji images into.
        output_dir: PathBuf,
    },
    /// Push (upload) one emoji file, or bulk-upload from a directory (alias: add).
    #[command(visible_alias = "ps", alias = "add")]
    Push {
        /// Discourse name.
        discourse: String,
        /// Local file or directory path.
        emoji_path: PathBuf,
        /// Optional emoji name (file uploads only).
        emoji_name: Option<String>,
    },

    /// List custom emojis on a Discourse.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
        /// Include additional fields where supported.
        #[arg(long, short = 'v')]
        verbose: bool,
        /// Render inline images when terminal protocol support is available.
        #[arg(long, short = 'i')]
        inline: bool,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum TopicCommand {
    /// Pull a topic to a local Markdown file.
    #[command(visible_alias = "pl")]
    Pull {
        /// Discourse name.
        discourse: String,
        /// Topic ID.
        topic_id: u64,
        /// Destination file or directory (auto-derived when omitted).
        local_path: Option<PathBuf>,
        /// Pull the entire thread (every post) as a single Markdown file
        /// with YAML frontmatter and per-post headings. Default behaviour
        /// (no `--full`) writes only the OP, which is what `topic push`
        /// expects.
        #[arg(long, short = 'F')]
        full: bool,
    },
    /// Push a local Markdown file to a topic.
    #[command(visible_alias = "ps")]
    Push {
        /// Discourse name.
        discourse: String,
        /// Topic ID.
        topic_id: u64,
        /// Local Markdown file path.
        local_path: PathBuf,
        /// Update the post without bumping the topic in the activity feed.
        /// Use for silent maintenance edits (sends post[no_bump]=true).
        #[arg(long)]
        no_bump: bool,
        /// Update the post without recording an edit-history revision
        /// (sends post[skip_revision]=true). Suppresses the online audit
        /// trail - use sparingly.
        #[arg(long)]
        skip_revision: bool,
    },
    /// Sync a topic and local Markdown file using newest timestamp.
    #[command(visible_alias = "sy")]
    Sync {
        /// Discourse name.
        discourse: String,
        /// Topic ID.
        topic_id: u64,
        /// Local Markdown file path.
        local_path: PathBuf,
        /// Skip sync confirmation prompt.
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Reply to a topic with content from a file or stdin.
    #[command(visible_alias = "r")]
    Reply {
        /// Discourse name.
        discourse: String,
        /// Topic ID.
        topic_id: u64,
        /// Input file path. Reads stdin when omitted or `-`.
        local_path: Option<PathBuf>,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Create a new topic in a category, body from a file or stdin.
    #[command(visible_alias = "n")]
    New {
        /// Discourse name.
        discourse: String,
        /// Target category ID.
        category_id: u64,
        /// Topic title.
        #[arg(long, short = 't')]
        title: String,
        /// Input file path. Reads stdin when omitted or `-`.
        local_path: Option<PathBuf>,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Add a tag to a topic.
    Tag {
        /// Discourse name.
        discourse: String,
        /// Topic ID.
        topic_id: u64,
        /// Tag to add.
        tag: String,
    },
    /// Remove a tag from a topic.
    Untag {
        /// Discourse name.
        discourse: String,
        /// Topic ID.
        topic_id: u64,
        /// Tag to remove.
        tag: String,
    },
    /// Rename a topic's title (changes its URL slug). Honours `--dry-run`.
    Title {
        /// Discourse name.
        discourse: String,
        /// Topic ID.
        topic_id: u64,
        /// New title.
        title: String,
    },
    /// Set a topic's full tag list, replacing existing tags. Pass no tags to
    /// clear all tags. Honours `--dry-run`.
    Tags {
        /// Discourse name.
        discourse: String,
        /// Topic ID.
        topic_id: u64,
        /// Tags to set (space-separated; omit to clear all tags).
        tags: Vec<String>,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum CategoryCommand {
    /// List categories.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
        /// Include additional fields where supported.
        #[arg(long, short = 'v')]
        verbose: bool,
        /// Show category hierarchy tree.
        #[arg(long)]
        tree: bool,
    },
    /// Copy a category to another Discourse.
    #[command(visible_alias = "cp")]
    Copy {
        /// Source discourse name.
        discourse: String,
        /// Target discourse name (defaults to source when omitted).
        #[arg(long, short = 't')]
        target: Option<String>,
        /// Category ID or slug.
        category: String,
    },
    /// Pull all topics from a category into local Markdown files.
    #[command(visible_alias = "pl")]
    Pull {
        /// Discourse name.
        discourse: String,
        /// Category ID or slug.
        category: String,
        /// Destination directory (auto-derived when omitted).
        local_path: Option<PathBuf>,
    },
    /// Push local Markdown files into a category.
    #[command(visible_alias = "ps")]
    Push {
        /// Discourse name.
        discourse: String,
        /// Category ID or slug.
        category: String,
        /// Local directory containing Markdown files.
        local_path: PathBuf,
        /// Only update existing topics; error instead of creating a new topic
        /// when a local file has no remote match.
        #[arg(long)]
        updates_only: bool,
        /// Update posts without bumping their topics in the activity feed.
        /// Use for silent bulk maintenance edits (sends post[no_bump]=true).
        #[arg(long)]
        no_bump: bool,
        /// Update posts without recording edit-history revisions
        /// (sends post[skip_revision]=true). Suppresses the online audit
        /// trail - use sparingly.
        #[arg(long)]
        skip_revision: bool,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum GroupCommand {
    /// List groups.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
        /// Include additional fields where supported.
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Show group details.
    #[command(visible_alias = "i")]
    Info {
        /// Discourse name.
        discourse: String,
        /// Group ID.
        group: u64,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: StructuredFormat,
    },
    /// List members of a group.
    #[command(visible_alias = "m")]
    Members {
        /// Discourse name.
        discourse: String,
        /// Group ID.
        group: u64,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Copy a group to another Discourse.
    #[command(visible_alias = "cp")]
    Copy {
        /// Source discourse name.
        discourse: String,
        /// Target discourse name (defaults to source when omitted).
        #[arg(long, short = 't')]
        target: Option<String>,
        /// Group ID.
        group: u64,
    },
    /// Bulk add members to a group from a file (or stdin) of email addresses.
    #[command(visible_alias = "a")]
    Add {
        /// Discourse name.
        discourse: String,
        /// Group ID.
        group: u64,
        /// Path to a file of email addresses (one per line; blank
        /// lines and `#` comments are ignored). Reads stdin when
        /// omitted or `-`.
        local_path: Option<PathBuf>,
        /// Send Discourse notifications to added users.
        #[arg(long)]
        notify: bool,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum BackupCommand {
    /// Create a new backup.
    #[command(visible_alias = "cr")]
    Create {
        /// Discourse name.
        discourse: String,
    },
    /// List backups.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: OutputFormat,
        /// Include additional fields where supported.
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Pull (download) a backup to a local file.
    #[command(visible_alias = "pl")]
    Pull {
        /// Discourse name.
        discourse: String,
        /// Backup filename on the server (from `dsc backup list`).
        backup_filename: String,
        /// Local output path. Defaults to the backup filename in the current directory.
        local_path: Option<PathBuf>,
    },
    /// Push (restore) a backup on the server (alias: restore).
    #[command(visible_alias = "ps", alias = "restore")]
    Push {
        /// Discourse name.
        discourse: String,
        /// Backup filename/path on the target system.
        backup_path: String,
    },
    /// Provision an S3 backup bucket + a scoped IAM user and point Discourse at
    /// it (one command for the per-forum AWS backup runbook). Requires the
    /// `aws` CLI configured with IAM + S3 admin rights. Always preview with
    /// `-n` / `--dry-run` first.
    #[command(after_help = "Examples:
  dsc backup setup-s3 -n myforum                  # preview the full plan (review gate)
  dsc backup setup-s3 myforum --region eu-west-1
  dsc backup setup-s3 myforum --no-test")]
    SetupS3 {
        /// Discourse name.
        discourse: String,
        /// AWS region for the bucket.
        #[arg(long, default_value = "eu-west-2")]
        region: String,
        /// Bucket name (default: `<name>-discourse-backups`).
        #[arg(long)]
        bucket: Option<String>,
        /// Skip the verification backup after provisioning.
        #[arg(long)]
        no_test: bool,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum PaletteCommand {
    /// List color palettes.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
        /// Include additional fields where supported.
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Pull a palette to local JSON.
    #[command(visible_alias = "pl")]
    Pull {
        /// Discourse name.
        discourse: String,
        /// Palette ID.
        palette_id: u64,
        /// Destination file path (auto-derived when omitted).
        local_path: Option<PathBuf>,
    },
    /// Push local JSON to create or update a palette.
    #[command(visible_alias = "ps")]
    Push {
        /// Discourse name.
        discourse: String,
        /// Local JSON file path.
        local_path: PathBuf,
        /// Palette ID to update (creates a new palette when omitted).
        palette_id: Option<u64>,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum PluginCommand {
    /// List installed plugins.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
        /// Include additional fields where supported.
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Install a plugin from URL.
    #[command(visible_alias = "i")]
    Install {
        /// Discourse name.
        discourse: String,
        /// Plugin repository URL.
        url: String,
    },
    /// Remove a plugin by name.
    #[command(visible_alias = "rm")]
    Remove {
        /// Discourse name.
        discourse: String,
        /// Plugin name.
        name: String,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum ThemeCommand {
    /// List installed themes.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
        /// Include additional fields where supported.
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Install a theme from URL.
    #[command(visible_alias = "i")]
    Install {
        /// Discourse name.
        discourse: String,
        /// Theme repository URL.
        url: String,
    },
    /// Remove a theme by name.
    #[command(visible_alias = "rm")]
    Remove {
        /// Discourse name.
        discourse: String,
        /// Theme name.
        name: String,
    },
    /// Pull a theme to a local JSON file.
    #[command(visible_alias = "pl")]
    Pull {
        /// Discourse name.
        discourse: String,
        /// Theme ID (from `dsc theme list`).
        theme_id: u64,
        /// Destination file path (auto-derived from theme name when omitted).
        local_path: Option<PathBuf>,
    },
    /// Push a local JSON file to create or update a theme.
    #[command(visible_alias = "ps")]
    Push {
        /// Discourse name.
        discourse: String,
        /// Local JSON file path.
        local_path: PathBuf,
        /// Theme ID to update (creates a new theme when omitted).
        theme_id: Option<u64>,
    },
    /// Duplicate a theme and print the new theme ID.
    #[command(visible_alias = "dup")]
    Duplicate {
        /// Discourse name.
        discourse: String,
        /// Theme ID to duplicate (from `dsc theme list`).
        theme_id: u64,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Show a richer view of one theme/component than `list`.
    Show {
        /// Discourse name.
        discourse: String,
        /// Theme ID (from `dsc theme list`).
        theme_id: u64,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Read and write a theme/component's settings (not site settings).
    Setting {
        #[command(subcommand)]
        command: ThemeSettingCommand,
    },
    /// Enable a theme or component.
    Enable {
        /// Discourse name.
        discourse: String,
        /// Theme ID (from `dsc theme list`).
        theme_id: u64,
    },
    /// Disable a theme or component.
    Disable {
        /// Discourse name.
        discourse: String,
        /// Theme ID (from `dsc theme list`).
        theme_id: u64,
    },
    /// Attach a component to a parent theme (makes it active on that theme).
    Attach {
        /// Discourse name.
        discourse: String,
        /// Parent theme ID.
        parent_id: u64,
        /// Component (child theme) ID to attach.
        component_id: u64,
    },
    /// Detach a component from a parent theme.
    Detach {
        /// Discourse name.
        discourse: String,
        /// Parent theme ID.
        parent_id: u64,
        /// Component (child theme) ID to detach.
        component_id: u64,
    },
    /// Manage colour palettes (colour schemes). The canonical home for what
    /// was `dsc palette`.
    Palette {
        #[command(subcommand)]
        command: PaletteCommand,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum ThemeSettingCommand {
    /// List a theme/component's settings.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Theme ID (from `dsc theme list`).
        theme_id: u64,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Print a single setting's current value.
    Get {
        /// Discourse name.
        discourse: String,
        /// Theme ID.
        theme_id: u64,
        /// Setting key (the `setting` name from `theme setting list`).
        key: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Set a single setting. Value is sent verbatim (pass JSON text for
    /// json-schema list settings). Honours global `--dry-run`.
    Set {
        /// Discourse name.
        discourse: String,
        /// Theme ID.
        theme_id: u64,
        /// Setting key.
        key: String,
        /// New value (verbatim).
        value: String,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum PmCommand {
    /// Send a private message.
    #[command(visible_alias = "s")]
    Send {
        /// Discourse name.
        discourse: String,
        /// Recipient(s) — comma-separated usernames or group names.
        recipients: String,
        /// PM title / subject.
        #[arg(long, short = 't')]
        title: String,
        /// Input file path. Reads stdin when omitted or `-`.
        local_path: Option<PathBuf>,
    },
    /// List PMs for a user.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Username whose PMs to list.
        username: String,
        /// Direction / view: inbox | sent | archive | unread | new.
        #[arg(long, short = 'd', default_value = "inbox")]
        direction: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum ApiKeyCommand {
    /// List API keys.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Create a new API key. The secret is only shown at creation time —
    /// capture it from the output.
    #[command(visible_alias = "cr")]
    Create {
        /// Discourse name.
        discourse: String,
        /// Description / label for the key (shown in admin UI).
        description: String,
        /// Username the key acts as. Omit for a global all-users key.
        #[arg(long, short = 'u')]
        username: Option<String>,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Revoke an API key by ID.
    #[command(visible_alias = "rm")]
    Revoke {
        /// Discourse name.
        discourse: String,
        /// API key ID (from `dsc api-key list`).
        key_id: u64,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum InviteCommand {
    /// Invite a single email address.
    #[command(visible_alias = "s")]
    Send {
        /// Discourse name.
        discourse: String,
        /// Email address to invite.
        email: String,
        /// Add invitee to one or more groups on accept (repeatable).
        #[arg(long, short = 'g')]
        group: Vec<u64>,
        /// Land the invitee on a specific topic on accept.
        #[arg(long, short = 't')]
        topic: Option<u64>,
        /// Custom invitation message.
        #[arg(long, short = 'm')]
        message: Option<String>,
    },
    /// Bulk-invite from a file (or stdin) of email addresses.
    #[command(visible_alias = "b")]
    Bulk {
        /// Discourse name.
        discourse: String,
        /// Path to a file of email addresses (one per line; blank lines and
        /// `#` comments ignored). Reads stdin when omitted or `-`.
        local_path: Option<PathBuf>,
        /// Add every invitee to one or more groups on accept (repeatable).
        #[arg(long, short = 'g')]
        group: Vec<u64>,
        /// Land every invitee on a specific topic on accept.
        #[arg(long, short = 't')]
        topic: Option<u64>,
        /// Custom invitation message attached to each invite.
        #[arg(long, short = 'm')]
        message: Option<String>,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum UserCommand {
    /// List users via the admin users endpoint.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Listing type: active | new | staff | suspended | silenced | staged.
        #[arg(long, short = 'l', default_value = "active")]
        listing: String,
        /// Page number (Discourse paginates 100 per page).
        #[arg(long, short = 'p', default_value_t = 1)]
        page: u32,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Show detailed info for a user.
    #[command(visible_alias = "i")]
    Info {
        /// Discourse name.
        discourse: String,
        /// Username.
        username: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Suspend a user.
    #[command(visible_alias = "sus")]
    Suspend {
        /// Discourse name.
        discourse: String,
        /// Username.
        username: String,
        /// When the suspension ends. ISO-8601 timestamp (e.g.
        /// `2026-12-31T00:00:00Z`) or `forever`.
        #[arg(long, short = 'u', default_value = "forever")]
        until: String,
        /// Reason shown to the user and in the audit log.
        #[arg(long, short = 'r', default_value = "")]
        reason: String,
    },
    /// Remove a suspension from a user.
    #[command(visible_alias = "uns")]
    Unsuspend {
        /// Discourse name.
        discourse: String,
        /// Username.
        username: String,
    },
    /// Silence a user (prevents posting; less visible than suspend).
    #[command(visible_alias = "sil")]
    Silence {
        /// Discourse name.
        discourse: String,
        /// Username.
        username: String,
        /// When the silence ends. ISO-8601 timestamp; empty means
        /// indefinite.
        #[arg(long, short = 'u', default_value = "")]
        until: String,
        /// Reason shown to the user and in the audit log.
        #[arg(long, short = 'r', default_value = "")]
        reason: String,
    },
    /// Lift a silence on a user.
    #[command(visible_alias = "unsil")]
    Unsilence {
        /// Discourse name.
        discourse: String,
        /// Username.
        username: String,
    },
    /// Grant the user the admin or moderator role.
    #[command(visible_alias = "pr")]
    Promote {
        /// Discourse name.
        discourse: String,
        /// Username.
        username: String,
        /// Role to grant.
        #[arg(long, short = 'r', value_enum)]
        role: RoleArg,
    },
    /// Revoke the user's admin or moderator role.
    #[command(visible_alias = "de")]
    Demote {
        /// Discourse name.
        discourse: String,
        /// Username.
        username: String,
        /// Role to revoke.
        #[arg(long, short = 'r', value_enum)]
        role: RoleArg,
    },
    /// Create a new user. `--approve` also marks the account approved
    /// (needed when site requires manual approval). Password is either
    /// supplied via stdin (`--password-stdin`) or omitted — in the
    /// latter case the user will have to set one via the reset flow.
    #[command(visible_alias = "cr")]
    Create {
        /// Discourse name.
        discourse: String,
        /// New user's email address.
        email: String,
        /// New user's username.
        username: String,
        /// Display name (optional).
        #[arg(long, short = 'N')]
        name: Option<String>,
        /// Read the password from stdin instead of auto-reset.
        #[arg(long)]
        password_stdin: bool,
        /// Also mark the user approved (for sites with manual approval).
        #[arg(long)]
        approve: bool,
    },
    /// Trigger Discourse's password-reset email flow for a user.
    #[command(name = "password-reset", visible_aliases = ["pwreset", "pw-reset"])]
    PasswordReset {
        /// Discourse name.
        discourse: String,
        /// Username or email.
        username: String,
    },
    /// Set a user's primary email address. Requires admin scope.
    #[command(name = "email-set", visible_alias = "email")]
    EmailSet {
        /// Discourse name.
        discourse: String,
        /// Username.
        username: String,
        /// New email address.
        email: String,
    },
    /// Show a user's recent public activity (topics + replies by default).
    ///
    /// Built for the "archive my own activity to a journal forum" loop —
    /// pipe the markdown output straight into `dsc topic reply`/`topic new`.
    #[command(visible_alias = "act")]
    Activity {
        /// Discourse name (the *source* forum to read activity from).
        discourse: String,
        /// Username whose activity to read.
        username: String,
        /// How far back to look. Accepts `7d`, `24h`, `30m`, `1w`, `90s`, or
        /// an ISO-8601 timestamp / date. Omit to fetch everything available.
        #[arg(long, short = 's')]
        since: Option<String>,
        /// Action types to include, comma-separated. Default: topics,replies.
        /// Also recognises: mentions, quotes, likes, edits, responses.
        #[arg(long, short = 't', default_value = "topics,replies")]
        types: String,
        /// Hard cap on number of items returned.
        #[arg(long, short = 'L')]
        limit: Option<u32>,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "markdown")]
        format: ActivityFormatArg,
    },
    /// Manage a user's group memberships.
    #[command(visible_alias = "g")]
    Groups {
        #[command(subcommand)]
        command: UserGroupsCommand,
    },
}

#[derive(ValueEnum, Clone, Copy)]
pub enum SectionArg {
    All,
    Growth,
    Activity,
    Health,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum AnalyticsFormat {
    /// Plain text (default). Fixed-width columns, no borders.
    Text,
    /// DuckDB-style box-drawing table. Falls through to `text` when
    /// stdout isn't a TTY.
    Table,
    /// Pretty JSON.
    Json,
    /// YAML.
    #[value(alias = "yml")]
    Yaml,
    /// Markdown bullet list per section.
    #[value(alias = "md")]
    Markdown,
    /// Markdown table per section.
    #[value(alias = "md-table", name = "markdown-table")]
    MarkdownTable,
    /// CSV — one row per metric.
    Csv,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum ActivityFormatArg {
    Text,
    Json,
    #[value(alias = "yml")]
    Yaml,
    #[value(alias = "md")]
    Markdown,
    Csv,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum RoleArg {
    Admin,
    Moderator,
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum UserGroupsCommand {
    /// List the groups a user belongs to.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Target username.
        username: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Add a user to a group.
    #[command(visible_alias = "a")]
    Add {
        /// Discourse name.
        discourse: String,
        /// Target username.
        username: String,
        /// Group ID.
        group_id: u64,
        /// Send Discourse notification to the user.
        #[arg(long)]
        notify: bool,
    },
    /// Remove a user from a group.
    #[command(visible_alias = "rm")]
    Remove {
        /// Discourse name.
        discourse: String,
        /// Target username.
        username: String,
        /// Group ID.
        group_id: u64,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum PostCommand {
    /// Pull a post's raw Markdown to a local file.
    #[command(visible_alias = "pl")]
    Pull {
        /// Discourse name.
        discourse: String,
        /// Post ID.
        post_id: u64,
        /// Output file path. Prints to stdout when omitted.
        local_path: Option<PathBuf>,
    },
    /// Push a local file to update a post (alias: edit).
    #[command(visible_alias = "ps", alias = "edit")]
    Push {
        /// Discourse name.
        discourse: String,
        /// Post ID.
        post_id: u64,
        /// Input file path. Reads stdin when omitted or `-`.
        local_path: Option<PathBuf>,
    },
    /// Delete a post by ID.
    #[command(visible_alias = "rm")]
    Delete {
        /// Discourse name.
        discourse: String,
        /// Post ID.
        post_id: u64,
    },
    /// Move a post to a different topic.
    #[command(visible_alias = "mv")]
    Move {
        /// Discourse name.
        discourse: String,
        /// Post ID to move.
        post_id: u64,
        /// Destination topic ID.
        #[arg(long = "to-topic", short = 't')]
        to_topic: u64,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum TagCommand {
    /// List every tag on the Discourse.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
    /// Pull the tag taxonomy (tags + tag groups) to a local file.
    #[command(visible_alias = "pl")]
    Pull {
        /// Discourse name.
        discourse: String,
        /// Output file (default: tags.yaml). Extension determines format (.yaml/.json).
        #[arg(default_value = "tags.yaml")]
        local_path: PathBuf,
    },
    /// Push a local taxonomy file to the server (upsert; optionally prune).
    #[command(visible_alias = "ps")]
    Push {
        /// Discourse name.
        discourse: String,
        /// Input taxonomy file.
        local_path: PathBuf,
        /// Delete server tags/groups absent from the file.
        #[arg(long)]
        prune: bool,
    },
    /// Rename a tag, preserving topic associations.
    ///
    /// Discourse rewrites every topic's tag list in-place, so this avoids
    /// the delete-and-recreate pattern that loses topic membership.
    #[command(visible_alias = "rn")]
    Rename {
        /// Discourse name.
        discourse: String,
        /// Current tag name.
        old_name: String,
        /// New tag name.
        new_name: String,
    },
}

#[derive(Subcommand)]
#[command(next_display_order = None)]
pub enum SettingCommand {
    /// Set a site setting on a Discourse (or all tagged Discourses).
    ///
    /// Usage:
    ///   dsc setting set <discourse> <setting> <value>
    ///   dsc setting set --tags <tag1,tag2> <setting> <value>
    #[command(visible_alias = "s")]
    Set {
        /// Discourse name. Required unless `--tags` is provided.
        discourse: Option<String>,
        /// Setting key. Required.
        setting: Option<String>,
        /// Setting value. Required.
        value: Option<String>,
        /// Tag filter (comma/semicolon separated, match-any). Apply across all
        /// Discourses matching any of the tags. When set, omit `<discourse>`
        /// and pass `<setting> <value>` as the only positionals.
        #[arg(long, value_name = "tag1,tag2")]
        tags: Option<String>,
    },

    /// Get the current value of a site setting.
    #[command(visible_alias = "g")]
    Get {
        /// Discourse name.
        discourse: String,
        /// Setting key.
        setting: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },

    /// List all site settings.
    #[command(visible_alias = "ls")]
    List {
        /// Discourse name.
        discourse: String,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
        /// Show output even when list is empty.
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Snapshot every site setting to a file - the reference for what settings exist.
    ///
    /// See spec/setting-sync.md for the full schema and workflow. The
    /// generated file is a self-documenting YAML (or JSON) including each
    /// setting's value, default, type, category, and Discourse's own
    /// description - so it doubles as a catalog of available settings.
    #[command(visible_alias = "pl")]
    Pull {
        /// Discourse name.
        discourse: String,
        /// Output path. Format detected by extension (.json → JSON,
        /// otherwise YAML). Defaults to `settings.yaml`.
        #[arg(default_value = "settings.yaml")]
        local_path: PathBuf,
        /// Only include settings whose value differs from default. Produces
        /// a manageable file (~50-100 entries) suitable for version control.
        #[arg(long, short = 'c')]
        changed_only: bool,
        /// Limit to settings in this category (e.g. `required`, `email`,
        /// `security`).
        #[arg(long)]
        category: Option<String>,
    },

    /// Apply a settings snapshot file to a Discourse (idempotent).
    ///
    /// Compares each setting in the file against the server and PUTs only
    /// values that differ. Combine with `--dry-run` to preview the plan.
    #[command(visible_alias = "ph")]
    Push {
        /// Discourse name.
        discourse: String,
        /// Path to the settings snapshot file (YAML or JSON).
        local_path: PathBuf,
        /// For settings present on the server but absent from the file,
        /// reset them to their default value. Off by default (file describes
        /// only the values you care about).
        #[arg(long)]
        reset_unlisted: bool,
    },

    /// Compare site settings between two sources.
    ///
    /// Each source can be a Discourse name (live fetch) or a path to a
    /// snapshot file produced by `dsc setting pull`. Sources are detected
    /// by whether the argument refers to an existing file on disk; if not,
    /// it is treated as a Discourse name.
    #[command(visible_alias = "df")]
    Diff {
        /// First source: Discourse name or snapshot file path.
        source: String,
        /// Second source: Discourse name or snapshot file path.
        target: String,
        /// Filter to settings where at least one source differs from default.
        /// Reduces noise when most settings on both sides are still default.
        #[arg(long, short = 'c')]
        changed_only: bool,
        /// Limit to settings in this category (e.g. `required`, `email`).
        /// Only effective when both sources carry category metadata.
        #[arg(long)]
        category: Option<String>,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },

    /// Show the value of one setting across every configured forum
    /// (optionally filtered by `--tags`). Diff-friendly; distinct from `diff`,
    /// which compares two specific sources across all settings.
    Audit {
        /// Setting key.
        setting: String,
        /// Only audit forums carrying at least one of these tags
        /// (comma/semicolon-separated). Omit to audit every configured forum.
        #[arg(long, value_name = "tag1,tag2")]
        tags: Option<String>,
        /// Output format.
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: ListFormat,
    },
}

#[derive(ValueEnum, Clone, Copy)]
pub enum CompletionShell {
    /// Bash shell.
    Bash,
    /// Zsh shell.
    Zsh,
    /// Fish shell.
    Fish,
}

impl From<CompletionShell> for Shell {
    fn from(value: CompletionShell) -> Self {
        match value {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Zsh => Shell::Zsh,
            CompletionShell::Fish => Shell::Fish,
        }
    }
}

#[derive(ValueEnum, Clone)]
pub enum OutputFormat {
    /// Plain text.
    #[value(alias = "plaintext")]
    Text,
    /// Markdown list.
    Markdown,
    /// Markdown table.
    MarkdownTable,
    /// Pretty JSON.
    Json,
    /// YAML.
    #[value(alias = "yml")]
    Yaml,
    /// CSV.
    Csv,
    /// One base URL per line (pipe-friendly).
    #[value(alias = "url")]
    Urls,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum ListFormat {
    /// Plain text.
    Text,
    /// Pretty JSON.
    Json,
    /// YAML.
    #[value(alias = "yml")]
    Yaml,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum StructuredFormat {
    /// Pretty JSON.
    Json,
    /// YAML.
    #[value(alias = "yml")]
    Yaml,
}
