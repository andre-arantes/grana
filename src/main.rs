mod format;
mod model;
mod storage;
mod subs;

use anyhow::Result;
use clap::{Parser, Subcommand};

use model::Cycle;

/// grana: subscriptions and who owes whom, from your terminal.
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage subscriptions
    Subs {
        #[command(subcommand)]
        command: SubsCommand,
    },
}

#[derive(Subcommand)]
enum SubsCommand {
    /// Add a subscription (monthly unless --yearly)
    Add {
        name: String,
        price: String,
        #[arg(long)]
        yearly: bool,
    },
    /// List subscriptions with monthly and yearly totals
    List {
        /// Include cancelled subscriptions
        #[arg(long)]
        all: bool,
    },
    /// Record a new price (keeps the old one in history)
    Price { name: String, new_price: String },
    /// Show a subscription's price history
    History { name: String },
    /// Cancel a subscription (kept in history, removed from totals)
    Cancel { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = storage::data_path()?;
    let mut data = storage::load(&path)?;
    let today = chrono::Local::now().date_naive();

    let (output, changed) = match cli.command {
        Command::Subs { command } => match command {
            SubsCommand::Add { name, price, yearly } => {
                let cycle = if yearly { Cycle::Yearly } else { Cycle::Monthly };
                (subs::add(&mut data, &name, format::parse_money(&price)?, cycle, today)?, true)
            }
            SubsCommand::List { all } => (subs::list(&data, all), false),
            SubsCommand::Price { name, new_price } => {
                (subs::change_price(&mut data, &name, format::parse_money(&new_price)?, today)?, true)
            }
            SubsCommand::History { name } => (subs::history(&data, &name)?, false),
            SubsCommand::Cancel { name } => (subs::cancel(&mut data, &name)?, true),
        },
    };

    if changed {
        storage::save(&path, &data)?;
    }
    println!("{output}");
    Ok(())
}
