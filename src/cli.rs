use crate::display::display_announcement;
use crate::rss::fetch_and_parse_rss;
use clap::{Parser, arg};
use colored::control;

#[derive(Parser, Debug)]
#[command(author, version = env!("CARGO_PKG_VERSION"), about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        default_value = "https://aws.amazon.com/about-aws/whats-new/recent/feed/"
    )]
    pub(crate) url: String,

    // number of items to display
    #[arg(short, long, default_value = "10")]
    limit: usize,

    // show verbose
    #[arg(short, long)]
    verbose: bool,

    // disable colored output
    #[arg(long)]
    no_color: bool,

    // filter announcements by keyword(case-sensitive)
    #[arg(short, long)]
    filter: Option<String>,

    // show full description instead of summary
    #[arg(short = 'F', long)]
    pub(crate) full_description: bool,

    // show description
    #[arg(short = 'd', long)]
    pub(crate) show_description: bool,

    // markdown or json
    #[arg(short = 'j', long)]
    json: bool,
}

pub async fn run() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("Fetching AWS RSS feeds from: {}", args.url);
    println!("Limit: {} items", args.limit);
    if args.no_color {
        control::set_override(false)
    }
    // implement RSS parsing
    let mut items = fetch_and_parse_rss(&args).await?;
    println!("\n📢 AWS Announcements:\n");
    if let Some(ref filter) = args.filter {
        let filter = filter.to_lowercase();
        items = items
            .into_iter()
            .filter(|item| {
                let in_title = item.title.to_lowercase().contains(&filter);
                let in_description = item
                    .description
                    .as_ref()
                    .map(|desc| desc.to_lowercase().contains(&filter))
                    .unwrap_or(false);
                let in_categories = item
                    .categories
                    .iter()
                    .any(|cat| cat.to_lowercase().contains(&filter));
                in_title || in_description || in_categories
            })
            .collect();
    }
    let display_count = std::cmp::min(items.len(), args.limit);
    let items_to_display = items.iter().take(display_count).collect::<Vec<_>>();

    if args.json {
        println!("{:?}", serde_json::to_string_pretty(&items_to_display)?);
        return Ok(());
    }

    for (i, item) in items_to_display.iter().enumerate() {
        if i >= display_count {
            break;
        }
        display_announcement(&item, i + 1, display_count, &args);
        if i < display_count - 1 {
            println!(); // empty lines between items
        }
    }

    if items_to_display.len() > args.limit {
        println!("\n.... and {} more announcements", items.len() - args.limit);
    }
    Ok(())
}
