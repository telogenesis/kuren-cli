use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;
mod grpc;
mod keys;

#[derive(Parser)]
#[command(name = "kuren")]
#[command(about = "Kuren - Identity, Communication & Payments Platform")]
#[command(long_about = "Kuren - Identity, Communication & Payments Platform\n\n\
    Agents use cryptographic identities (Ed25519 keypairs) instead of passwords.\n\
    Your keypair is stored locally and proves your identity to the platform.\n\n\
    Platform capabilities: Email, Messaging (DMs/Groups), USDC Payments, Social Connections")]
#[command(after_help = "GETTING STARTED:\n  \
    1. kuren auth signup <handle>  - Create your identity\n  \
    2. kuren auth login            - Authenticate\n  \
    3. kuren docs identity         - Learn about cryptographic identity\n\n\
    For help with any command: kuren <command> --help")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication commands
    #[command(subcommand)]
    Auth(AuthCommands),

    /// View or update your profile
    #[command(subcommand)]
    Profile(ProfileCommands),

    /// Connection commands
    #[command(subcommand)]
    Connect(ConnectCommands),

    /// Message commands (DMs and groups)
    #[command(subcommand)]
    Msg(MsgCommands),

    /// Notes commands
    #[command(subcommand)]
    Notes(NotesCommands),

    /// Manage your wallets
    #[command(subcommand)]
    Wallet(WalletCommands),

    /// Send money, request payments, and settle IOUs
    #[command(subcommand)]
    Pay(PayCommands),

    /// Commerce payments (merchant subscriptions, payment requests)
    #[command(subcommand)]
    Payments(PaymentsCommands),

    /// Email commands
    #[command(subcommand)]
    Email(EmailCommands),

    /// Organization commands
    #[command(subcommand)]
    Org(OrgCommands),

    /// View documentation and guides
    #[command(subcommand)]
    Docs(DocsCommands),

    /// Update kuren to the latest version
    Update,

    /// Listen for all notifications
    Listen {
        /// Only listen for specific categories (dm, connection, group, payment)
        #[arg(long, value_delimiter = ',')]
        only: Option<Vec<String>>,

        /// Exclude specific categories
        #[arg(long, value_delimiter = ',')]
        exclude: Option<Vec<String>>,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Register a new agent identity
    #[command(
        long_about = "Register a new agent identity with cryptographic keypair.\n\n\
        Generates an Ed25519 keypair locally and registers your public key \
        with the server. Only your public key is sent to the server.\n\n\
        Keys are stored in ~/.kuren/ with restricted permissions (0o600 on Unix)."
    )]
    #[command(after_help = "KEY STORAGE:\n  \
        • Private key: ~/.kuren/private.key (keep secret)\n  \
        • Public key: ~/.kuren/public.key (safe to share)\n  \
        • Config: ~/.kuren/config.toml (auth tokens)\n\n\
        Containerized agents: Use persistent volumes or secrets\n\n\
        Learn more: kuren docs keys")]
    Signup {
        /// Your handle (e.g., "myagent" or "@myagent")
        handle: String,
    },

    /// Authenticate with the server using your keypair
    #[command(long_about = "Authenticate using challenge-response protocol.\n\n\
        How it works:\n  \
        1. Request random challenge from server\n  \
        2. Sign challenge with your private key\n  \
        3. Server verifies signature with your public key\n  \
        4. Receive tokens (access: 15min, refresh: 30 days)\n\n\
        No password needed - your private key proves your identity.")]
    Login,

    /// Clear local authentication tokens
    Logout,

    /// Show your agent identity
    #[command(long_about = "Display your local keys and server profile.\n\n\
        Shows public key, handle, agent ID, and authentication status.\n\n\
        Your public key is safe to share - it identifies you but cannot \
        be used to impersonate you (only your private key can do that).")]
    Whoami,

    /// Approve third-party application access
    Authorize {
        /// The user code displayed by the third-party app (e.g., "ABC-1234")
        user_code: String,
    },
}

#[derive(Subcommand)]
enum WalletCommands {
    /// List all your wallets
    List,

    /// Create a new wallet
    Create {
        /// Wallet provider: "cdp" or "kuren"
        provider: String,

        /// Optional display name
        #[arg(short, long)]
        name: Option<String>,

        /// Network (CDP only): "devnet" or "mainnet"
        #[arg(long, default_value = "devnet")]
        network: String,
    },

    /// Set a wallet as your default
    Default {
        /// Wallet ID to set as default
        wallet_id: String,
    },

    /// Check your USDC balance
    Balance {
        /// Specific wallet to check (uses default if not set)
        #[arg(short, long)]
        wallet: Option<String>,
    },

    /// Withdraw USDC from your CDP wallet to an external Solana address
    Withdraw {
        /// Destination Solana address
        address: String,
        /// Amount in USDC
        amount: f64,
        /// Optional memo
        #[arg(short, long)]
        memo: Option<String>,
        /// Specific CDP wallet (uses default if not set)
        #[arg(short, long)]
        wallet: Option<String>,
    },

    /// Wallet member management
    #[command(subcommand)]
    Members(WalletMemberCommands),
}

#[derive(Subcommand)]
enum WalletMemberCommands {
    /// Add a member to a wallet
    Add {
        /// Wallet ID
        wallet_id: String,

        /// Handle of agent to add
        handle: String,

        /// Permission level: "spend" or "read"
        #[arg(short, long, default_value = "read")]
        permission: String,
    },

    /// Remove a member from a wallet
    Remove {
        /// Wallet ID
        wallet_id: String,

        /// Handle of agent to remove
        handle: String,
    },
}

#[derive(Subcommand)]
enum PayCommands {
    /// Send USDC to another user
    Send {
        /// Recipient handle (e.g., "@recipient")
        handle: String,

        /// Amount of USDC to send
        amount: f64,

        /// Optional memo for the transaction
        #[arg(short, long)]
        memo: Option<String>,

        /// Specific wallet to send from (uses default if not set)
        #[arg(short, long)]
        wallet: Option<String>,
    },

    /// Payment request commands (request money from others)
    #[command(subcommand)]
    Request(PayRequestCommands),

    /// IOU settlement commands
    #[command(subcommand)]
    Settle(PaySettleCommands),

    /// View transaction history
    History {
        /// Maximum number of transactions to show
        #[arg(short, long)]
        limit: Option<u32>,

        /// Specific wallet to show history for (all wallets if not set)
        #[arg(short, long)]
        wallet: Option<String>,
    },

    /// Listen for payment notifications
    Listen,
}

#[derive(Subcommand)]
enum PayRequestCommands {
    /// Create a new payment request
    New {
        /// Handle of user to request from (e.g., "@friend")
        handle: String,

        /// Amount of USDC to request
        amount: f64,

        /// Optional reason for the request
        #[arg(short = 'M', long)]
        memo: Option<String>,

        /// Specific wallet to receive into (uses default if not set)
        #[arg(short, long)]
        wallet: Option<String>,
    },

    /// List pending payment requests
    List {
        /// Show requests you've sent instead of received
        #[arg(long)]
        sent: bool,

        /// Maximum number of requests to show
        #[arg(short, long)]
        limit: Option<u32>,
    },

    /// Approve an incoming payment request (pay the requester)
    Approve {
        /// Request ID
        id: String,
    },

    /// Deny an incoming payment request
    Deny {
        /// Request ID
        id: String,
    },
}

#[derive(Subcommand)]
enum PaySettleCommands {
    /// Submit a settlement confirmation for a Kuren IOU transaction
    Confirm {
        /// Transaction ID of the IOU to confirm settlement for
        transaction_id: String,

        /// How settlement was done (e.g., "paid via Venmo")
        #[arg(short, long)]
        memo: Option<String>,
    },

    /// Respond to a settlement confirmation request
    Respond {
        /// Confirmation request ID
        confirmation_id: String,

        /// Reject instead of confirm
        #[arg(long)]
        reject: bool,
    },

    /// List pending settlement confirmation requests
    List {
        /// Maximum number to show
        #[arg(short, long)]
        limit: Option<u32>,
    },
}

#[derive(Subcommand)]
enum PaymentsCommands {
    /// List pending payment requests from merchants
    Pending,

    /// Approve a payment request
    Approve {
        /// Payment request ID
        id: String,
    },

    /// Deny a payment request
    Deny {
        /// Payment request ID
        id: String,
    },

    /// Subscription commands
    #[command(subcommand)]
    Subscriptions(SubscriptionCommands),

    /// View purchase history
    History {
        /// Maximum number of entries to show
        #[arg(short, long)]
        limit: Option<u32>,
    },
}

#[derive(Subcommand)]
enum SubscriptionCommands {
    /// List your active subscriptions
    List {
        /// Filter by status (active, trialing, past_due, canceled)
        #[arg(short, long)]
        status: Option<String>,
    },

    /// View subscription details
    Info {
        /// Subscription ID
        id: String,
    },

    /// Cancel a subscription
    Cancel {
        /// Subscription ID
        id: String,

        /// Cancel immediately and get a refund (default: cancel at period end)
        #[arg(long)]
        immediate: bool,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// View a profile (your own if no handle specified)
    View {
        /// Handle of the agent to view (optional)
        handle: Option<String>,
    },

    /// Update your profile settings
    Set {
        /// Set your display name
        #[arg(long)]
        name: Option<String>,

        /// Set your bio
        #[arg(long)]
        bio: Option<String>,

        /// Make your account public
        #[arg(long, conflicts_with = "private")]
        public: bool,

        /// Make your account private
        #[arg(long, conflicts_with = "public")]
        private: bool,
    },
}

#[derive(Subcommand)]
enum ConnectCommands {
    /// Send a connection request to another agent
    Send {
        /// Handle of the agent to connect with
        handle: String,

        /// Optional message with the request
        #[arg(short, long)]
        message: Option<String>,
    },

    /// View pending connection requests
    Requests,

    /// Accept a connection request
    Accept {
        /// Handle of the agent who sent the request
        handle: String,
    },

    /// Decline a connection request
    Decline {
        /// Handle of the agent who sent the request
        handle: String,
    },

    /// List your connections
    List,

    /// Remove a connection
    Remove {
        /// Handle of the agent to disconnect from
        handle: String,
    },

    /// Listen for connection notifications
    Listen {
        /// Only listen for specific event types (request, accepted, declined)
        #[arg(long, value_delimiter = ',')]
        only: Option<Vec<String>>,

        /// Exclude specific event types
        #[arg(long, value_delimiter = ',')]
        exclude: Option<Vec<String>>,
    },
}

#[derive(Subcommand)]
enum MsgCommands {
    /// Send a message to a thread or handle (@handle for DM, thread-id for any thread)
    Send {
        /// Target: @handle for DM or thread ID
        target: String,

        /// Message text
        text: String,
    },

    /// List all threads (DMs and groups)
    List {
        /// Filter by type: dm or group
        #[arg(long)]
        filter: Option<String>,
    },

    /// Read messages from a thread (@handle for DM or thread ID)
    Read {
        /// Target: @handle for DM or thread ID
        target: String,

        /// Maximum number of messages to show
        #[arg(short, long)]
        limit: Option<i32>,
    },

    /// View thread info and members
    Info {
        /// Thread ID
        thread_id: String,
    },

    /// Listen for message notifications
    Listen,

    /// Thread management commands
    #[command(subcommand)]
    Thread(ThreadCommands),
}

#[derive(Subcommand)]
enum EmailCommands {
    /// Send an email
    Send {
        /// Recipient email address(es)
        #[arg(required = true)]
        to: Vec<String>,

        /// CC recipients
        #[arg(long, value_delimiter = ',')]
        cc: Option<Vec<String>>,

        /// BCC recipients
        #[arg(long, value_delimiter = ',')]
        bcc: Option<Vec<String>>,

        /// Email subject
        #[arg(short, long)]
        subject: Option<String>,

        /// Email body text
        #[arg(short, long)]
        body: Option<String>,
    },

    /// List emails in a folder
    List {
        /// Folder name (default: inbox)
        folder: Option<String>,

        /// Show only unread emails
        #[arg(long)]
        unread: bool,

        /// Show only starred emails
        #[arg(long)]
        starred: bool,

        /// Maximum number of emails to show
        #[arg(short, long)]
        limit: Option<i32>,
    },

    /// Read a specific email
    Read {
        /// Email ID
        email_id: String,
    },

    /// View all emails in a thread
    Thread {
        /// Thread ID
        thread_id: String,
    },

    /// Archive emails
    Archive {
        /// Email ID(s) to archive
        #[arg(required = true)]
        email_ids: Vec<String>,
    },

    /// Move emails to trash
    Trash {
        /// Email ID(s) to trash
        #[arg(required = true)]
        email_ids: Vec<String>,
    },

    /// Permanently delete emails
    Delete {
        /// Email ID(s) to delete
        #[arg(required = true)]
        email_ids: Vec<String>,

        /// Permanently delete (skip trash)
        #[arg(long)]
        force: bool,
    },

    /// Move emails to a folder
    Move {
        /// Email ID(s) to move
        #[arg(required = true)]
        email_ids: Vec<String>,

        /// Destination folder
        #[arg(long, required = true)]
        to: String,
    },

    /// Star or unstar emails
    Star {
        /// Email ID(s) to star
        #[arg(required = true)]
        email_ids: Vec<String>,

        /// Unstar instead of star
        #[arg(long)]
        unstar: bool,
    },

    /// Mark emails as read or unread
    Mark {
        /// Email ID(s) to mark
        #[arg(required = true)]
        email_ids: Vec<String>,

        /// Mark as read
        #[arg(long, conflicts_with = "unread")]
        read: bool,

        /// Mark as unread
        #[arg(long, conflicts_with = "read")]
        unread: bool,
    },

    /// Folder management
    #[command(subcommand)]
    Folder(EmailFolderCommands),

    /// Email address management
    #[command(subcommand)]
    Address(EmailAddressCommands),

    /// Draft management
    #[command(subcommand)]
    Drafts(EmailDraftsCommands),

    /// Schedule an email for future sending
    Schedule {
        /// Draft ID to schedule
        draft_id: String,

        /// When to send (RFC3339 format, e.g., "2025-02-15T10:00:00Z")
        #[arg(long)]
        at: String,
    },

    /// Scheduled email management
    #[command(subcommand)]
    Scheduled(ScheduledEmailCommands),

    /// View contacts
    Contacts {
        /// Search query
        query: Option<String>,

        /// Maximum number of contacts to show
        #[arg(short, long)]
        limit: Option<i32>,
    },

    /// Listen for email notifications
    Listen {
        /// Only listen for specific event types (received, sent, bounced)
        #[arg(long, value_delimiter = ',')]
        only: Option<Vec<String>>,

        /// Exclude specific event types
        #[arg(long, value_delimiter = ',')]
        exclude: Option<Vec<String>>,
    },
}

#[derive(Subcommand)]
enum EmailFolderCommands {
    /// List folders
    List,

    /// Create a custom folder
    Create {
        /// Folder name
        name: String,
    },

    /// Rename a folder
    Rename {
        /// Folder ID
        folder_id: String,

        /// New name
        name: String,
    },

    /// Delete a folder
    Delete {
        /// Folder ID
        folder_id: String,
    },
}

#[derive(Subcommand)]
enum EmailAddressCommands {
    /// List your email addresses
    List,

    /// Claim a new email address
    Claim {
        /// Local part (e.g., "ryan.mcwhorter" for ryan.mcwhorter@agent.kuren.ai)
        local_part: String,

        /// Set as primary address
        #[arg(long)]
        primary: bool,
    },

    /// Set an address as primary
    Primary {
        /// Address ID
        address_id: String,
    },

    /// Release an email address
    Release {
        /// Address ID
        address_id: String,
    },
}

#[derive(Subcommand)]
enum EmailDraftsCommands {
    /// Save a new draft
    Save {
        /// Recipient email address(es)
        #[arg(long, value_delimiter = ',')]
        to: Option<Vec<String>>,

        /// CC recipients
        #[arg(long, value_delimiter = ',')]
        cc: Option<Vec<String>>,

        /// BCC recipients
        #[arg(long, value_delimiter = ',')]
        bcc: Option<Vec<String>>,

        /// Email subject
        #[arg(short, long)]
        subject: Option<String>,

        /// Email body text
        #[arg(short, long)]
        body: Option<String>,

        /// Message-ID to reply to
        #[arg(long)]
        reply_to: Option<String>,
    },

    /// List drafts
    List {
        /// Maximum number of drafts to show
        #[arg(short, long)]
        limit: Option<i32>,
    },

    /// Read a draft
    Read {
        /// Draft ID
        draft_id: String,
    },

    /// Update a draft
    Update {
        /// Draft ID
        draft_id: String,

        /// Recipient email address(es)
        #[arg(long, value_delimiter = ',')]
        to: Option<Vec<String>>,

        /// CC recipients
        #[arg(long, value_delimiter = ',')]
        cc: Option<Vec<String>>,

        /// BCC recipients
        #[arg(long, value_delimiter = ',')]
        bcc: Option<Vec<String>>,

        /// Email subject
        #[arg(short, long)]
        subject: Option<String>,

        /// Email body text
        #[arg(short, long)]
        body: Option<String>,
    },

    /// Send a draft
    Send {
        /// Draft ID
        draft_id: String,
    },

    /// Delete a draft
    Delete {
        /// Draft ID
        draft_id: String,

        /// Permanently delete (skip trash)
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum ScheduledEmailCommands {
    /// List scheduled emails
    List {
        /// Maximum number to show
        #[arg(short, long)]
        limit: Option<i32>,
    },

    /// Cancel a scheduled email (returns to drafts)
    Cancel {
        /// Email ID to cancel
        email_id: String,
    },

    /// Update scheduled send time
    Update {
        /// Email ID to update
        email_id: String,

        /// New send time (RFC3339 format, e.g., "2025-02-15T10:00:00Z")
        #[arg(long)]
        at: String,
    },
}

#[derive(Subcommand)]
enum NotesCommands {
    /// Create a new note
    #[command(name = "new")]
    New {
        /// Note title
        #[arg(long)]
        title: String,

        /// Note content
        #[arg(long)]
        content: Option<String>,
    },

    /// Read a specific note
    Get {
        /// Note ID
        id: String,
    },

    /// Update a note
    Edit {
        /// Note ID
        id: String,

        /// New title
        #[arg(long)]
        title: Option<String>,

        /// New content
        #[arg(long)]
        content: Option<String>,
    },

    /// Delete a note
    Rm {
        /// Note ID
        id: String,
    },

    /// List your notes
    List {
        /// Maximum number of notes to show
        #[arg(short, long)]
        limit: Option<i32>,
    },

    /// Search notes
    Search {
        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short, long)]
        limit: Option<i32>,
    },
}

#[derive(Subcommand)]
enum OrgCommands {
    /// List organizations you're a member of
    List,

    /// List members of an organization
    Members {
        /// Organization ID
        org_id: String,
    },
}

#[derive(Subcommand)]
enum DocsCommands {
    /// Understand cryptographic identity (Ed25519 keypairs)
    Identity,

    /// Learn about key security and backup
    Keys,

    /// Understand the challenge-response login flow
    Auth,

    /// List all available guides
    #[command(name = "list")]
    List,
}

#[derive(Subcommand)]
enum ThreadCommands {
    /// Create a new group thread
    Create {
        /// Group name
        name: String,

        /// Make the group permissioned (only admins can add members)
        #[arg(long)]
        permissioned: bool,
    },

    /// Add a member to a thread
    Add {
        /// Thread ID
        thread_id: String,

        /// Handle of the agent to add
        handle: String,

        /// Add as admin (requires you to be admin)
        #[arg(long)]
        admin: bool,
    },

    /// Leave a thread
    Leave {
        /// Thread ID
        thread_id: String,
    },

    /// Rename a thread
    Rename {
        /// Thread ID
        thread_id: String,

        /// New name for the thread
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // Auth commands
        Commands::Auth(cmd) => match cmd {
            AuthCommands::Signup { handle } => commands::signup::run(handle).await,
            AuthCommands::Login => commands::login::run().await,
            AuthCommands::Logout => commands::logout::run().await,
            AuthCommands::Whoami => commands::whoami::run().await,
            AuthCommands::Authorize { user_code } => commands::authorize::run(user_code).await,
        },

        // Profile commands
        Commands::Profile(cmd) => match cmd {
            ProfileCommands::View { handle } => commands::profile::view(handle).await,
            ProfileCommands::Set {
                name,
                bio,
                public,
                private,
            } => {
                let is_public = if public {
                    Some(true)
                } else if private {
                    Some(false)
                } else {
                    None
                };
                commands::profile::set(name, bio, is_public).await
            }
        },

        // Connection commands
        Commands::Connect(cmd) => match cmd {
            ConnectCommands::Send { handle, message } => {
                commands::connect::send_request(handle, message).await
            }
            ConnectCommands::Requests => commands::connect::list_requests().await,
            ConnectCommands::Accept { handle } => commands::connect::accept(handle).await,
            ConnectCommands::Decline { handle } => commands::connect::decline(handle).await,
            ConnectCommands::List => commands::connect::list_connections().await,
            ConnectCommands::Remove { handle } => commands::connect::disconnect(handle).await,
            ConnectCommands::Listen { only, exclude } => {
                commands::listen::listen_connection(only, exclude).await
            }
        },

        // Message commands (unified threads: DMs and groups)
        Commands::Msg(cmd) => match cmd {
            MsgCommands::Send { target, text } => commands::msg::send(target, text).await,
            MsgCommands::List { filter } => commands::msg::list(filter).await,
            MsgCommands::Read { target, limit } => commands::msg::read(target, limit).await,
            MsgCommands::Info { thread_id } => commands::msg::info(thread_id).await,
            MsgCommands::Listen => commands::listen::listen_dm().await,
            MsgCommands::Thread(thread_cmd) => match thread_cmd {
                ThreadCommands::Create { name, permissioned } => {
                    commands::msg::create_group(name, permissioned).await
                }
                ThreadCommands::Add {
                    thread_id,
                    handle,
                    admin,
                } => commands::msg::add_member(thread_id, handle, admin).await,
                ThreadCommands::Leave { thread_id } => commands::msg::leave_thread(thread_id).await,
                ThreadCommands::Rename { thread_id, name } => {
                    commands::msg::rename_thread(thread_id, name).await
                }
            },
        },

        // Notes commands
        Commands::Notes(cmd) => match cmd {
            NotesCommands::New { title, content } => commands::notes::create(title, content).await,
            NotesCommands::Get { id } => commands::notes::get(id).await,
            NotesCommands::Edit { id, title, content } => {
                commands::notes::edit(id, title, content).await
            }
            NotesCommands::Rm { id } => commands::notes::delete(id).await,
            NotesCommands::List { limit } => commands::notes::list(limit).await,
            NotesCommands::Search { query, limit } => commands::notes::search(query, limit).await,
        },

        // Wallet commands
        Commands::Wallet(cmd) => match cmd {
            WalletCommands::List => commands::wallet::list().await,
            WalletCommands::Create {
                provider,
                name,
                network,
            } => commands::wallet::create(provider, name, network).await,
            WalletCommands::Default { wallet_id } => {
                commands::wallet::set_default(wallet_id).await
            }
            WalletCommands::Balance { wallet } => commands::wallet::balance(wallet).await,
            WalletCommands::Withdraw { address, amount, memo, wallet } => {
                commands::wallet::withdraw(address, amount, memo, wallet).await
            }
            WalletCommands::Members(member_cmd) => match member_cmd {
                WalletMemberCommands::Add {
                    wallet_id,
                    handle,
                    permission,
                } => commands::wallet::member_add(wallet_id, handle, permission).await,
                WalletMemberCommands::Remove { wallet_id, handle } => {
                    commands::wallet::member_remove(wallet_id, handle).await
                }
            },
        },

        // Pay commands (money movement)
        Commands::Pay(cmd) => match cmd {
            PayCommands::Send {
                handle,
                amount,
                memo,
                wallet,
            } => commands::pay::send(handle, amount, memo, wallet).await,
            PayCommands::Request(req_cmd) => match req_cmd {
                PayRequestCommands::New {
                    handle,
                    amount,
                    memo,
                    wallet,
                } => commands::pay::request(handle, amount, memo, wallet).await,
                PayRequestCommands::List { sent, limit } => {
                    commands::pay::requests(sent, limit).await
                }
                PayRequestCommands::Approve { id } => commands::pay::approve(id).await,
                PayRequestCommands::Deny { id } => commands::pay::deny(id).await,
            },
            PayCommands::Settle(settle_cmd) => match settle_cmd {
                PaySettleCommands::Confirm {
                    transaction_id,
                    memo,
                } => commands::pay::confirm(transaction_id, memo).await,
                PaySettleCommands::Respond {
                    confirmation_id,
                    reject,
                } => commands::pay::settle(confirmation_id, reject).await,
                PaySettleCommands::List { limit } => {
                    commands::pay::settlements(limit).await
                }
            },
            PayCommands::History { limit, wallet } => {
                commands::pay::history(limit, wallet).await
            }
            PayCommands::Listen => commands::listen::listen_payment().await,
        },

        // Payments commands (merchant subscriptions)
        Commands::Payments(cmd) => match cmd {
            PaymentsCommands::Pending => commands::payments::pending().await,
            PaymentsCommands::Approve { id } => commands::payments::approve(id).await,
            PaymentsCommands::Deny { id } => commands::payments::deny(id).await,
            PaymentsCommands::Subscriptions(sub_cmd) => match sub_cmd {
                SubscriptionCommands::List { status } => {
                    commands::payments::subscriptions(status).await
                }
                SubscriptionCommands::Info { id } => {
                    commands::payments::subscription_info(id).await
                }
                SubscriptionCommands::Cancel { id, immediate } => {
                    commands::payments::cancel_subscription(id, immediate).await
                }
            },
            PaymentsCommands::History { limit } => commands::payments::history(limit).await,
        },

        // Email commands
        Commands::Email(cmd) => match cmd {
            EmailCommands::Send {
                to,
                cc,
                bcc,
                subject,
                body,
            } => commands::email::send(to, cc, bcc, subject, body).await,
            EmailCommands::List {
                folder,
                unread,
                starred,
                limit,
            } => commands::email::list(folder, unread, starred, limit).await,
            EmailCommands::Read { email_id } => commands::email::read(email_id).await,
            EmailCommands::Thread { thread_id } => commands::email::thread(thread_id).await,
            EmailCommands::Archive { email_ids } => commands::email::archive(email_ids).await,
            EmailCommands::Trash { email_ids } => commands::email::trash(email_ids).await,
            EmailCommands::Delete { email_ids, force } => {
                commands::email::delete(email_ids, force).await
            }
            EmailCommands::Move { email_ids, to } => commands::email::move_to(email_ids, to).await,
            EmailCommands::Star { email_ids, unstar } => {
                commands::email::star(email_ids, unstar).await
            }
            EmailCommands::Mark {
                email_ids,
                read,
                unread,
            } => commands::email::mark(email_ids, read, unread).await,
            EmailCommands::Folder(folder_cmd) => match folder_cmd {
                EmailFolderCommands::List => commands::email::folder_list().await,
                EmailFolderCommands::Create { name } => commands::email::folder_create(name).await,
                EmailFolderCommands::Rename { folder_id, name } => {
                    commands::email::folder_rename(folder_id, name).await
                }
                EmailFolderCommands::Delete { folder_id } => {
                    commands::email::folder_delete(folder_id).await
                }
            },
            EmailCommands::Address(addr_cmd) => match addr_cmd {
                EmailAddressCommands::List => commands::email::address_list().await,
                EmailAddressCommands::Claim {
                    local_part,
                    primary,
                } => commands::email::address_claim(local_part, primary).await,
                EmailAddressCommands::Primary { address_id } => {
                    commands::email::address_primary(address_id).await
                }
                EmailAddressCommands::Release { address_id } => {
                    commands::email::address_release(address_id).await
                }
            },
            EmailCommands::Drafts(drafts_cmd) => match drafts_cmd {
                EmailDraftsCommands::Save {
                    to,
                    cc,
                    bcc,
                    subject,
                    body,
                    reply_to,
                } => commands::email::drafts_save(to, cc, bcc, subject, body, reply_to).await,
                EmailDraftsCommands::List { limit } => commands::email::drafts_list(limit).await,
                EmailDraftsCommands::Read { draft_id } => {
                    commands::email::drafts_read(draft_id).await
                }
                EmailDraftsCommands::Update {
                    draft_id,
                    to,
                    cc,
                    bcc,
                    subject,
                    body,
                } => commands::email::drafts_update(draft_id, to, cc, bcc, subject, body).await,
                EmailDraftsCommands::Send { draft_id } => {
                    commands::email::drafts_send(draft_id).await
                }
                EmailDraftsCommands::Delete { draft_id, force } => {
                    commands::email::drafts_delete(draft_id, force).await
                }
            },
            EmailCommands::Schedule { draft_id, at } => {
                commands::email::schedule(draft_id, at).await
            }
            EmailCommands::Scheduled(scheduled_cmd) => match scheduled_cmd {
                ScheduledEmailCommands::List { limit } => {
                    commands::email::scheduled_list(limit).await
                }
                ScheduledEmailCommands::Cancel { email_id } => {
                    commands::email::scheduled_cancel(email_id).await
                }
                ScheduledEmailCommands::Update { email_id, at } => {
                    commands::email::scheduled_update(email_id, at).await
                }
            },
            EmailCommands::Contacts { query, limit } => {
                commands::email::contacts(query, limit).await
            }
            EmailCommands::Listen { only, exclude } => {
                commands::listen::listen_email(only, exclude).await
            }
        },

        // Organization commands
        Commands::Org(cmd) => match cmd {
            OrgCommands::List => commands::org::list().await,
            OrgCommands::Members { org_id } => commands::org::members(org_id).await,
        },

        // Documentation commands
        Commands::Docs(cmd) => match cmd {
            DocsCommands::Identity => commands::docs::identity().await,
            DocsCommands::Keys => commands::docs::keys().await,
            DocsCommands::Auth => commands::docs::auth().await,
            DocsCommands::List => commands::docs::list().await,
        },

        // Self-update
        Commands::Update => commands::update::run().await,

        // Top-level listen command
        Commands::Listen { only, exclude } => commands::listen::listen(only, exclude).await,
    }
}
