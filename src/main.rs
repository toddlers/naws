mod utils;
use anyhow::{Context, Result};
use clap::{Parser, arg};
use colored::*;
use serde::{Deserialize, Serialize};
#[allow(dead_code)]
#[allow(unused_imports)]
use utils::{decode_tag, format_date};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "https://aws.amazon.com/about-aws/whats-new/recent/feed/"
    )]
    url: String,

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
    full_description: bool,

    // show description
    #[arg(short = 'd', long)]
    show_description: bool,

    // markdown or json
    #[arg(short = 'j', long)]
    json: bool,
}

#[derive(Debug, Deserialize)]
struct RssFeed {
    channel: RssChannel,
}

#[derive(Debug, Deserialize)]
struct RssChannel {
    #[serde(rename = "item")]
    items: Vec<RssItemRaw>,
}

// rss item
#[derive(Debug, Clone, Deserialize)]
struct RssItemRaw {
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    #[serde(rename = "pubDate")]
    pub_date: Option<String>,
    #[serde(default)]
    categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RssItem {
    pub title: String,
    pub link: String,
    pub description: Option<String>,
    pub pub_date: Option<String>,
    pub categories: Vec<String>,
}

impl From<RssItemRaw> for RssItem {
    fn from(raw: RssItemRaw) -> Self {
        Self {
            title: raw.title.unwrap_or_else(|| "#Untitled".into()),
            link: raw.link.unwrap_or_else(|| "#NoLinl".into()),
            description: raw.description,
            pub_date: raw.pub_date,
            categories: raw.categories,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
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

    if args.json {
        println!("{:?}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }

    for (i, mut item) in items.iter().enumerate() {
        if i >= display_count {
            break;
        }
        display_announcement(&mut item, i + 1, display_count, &args);
        if i < display_count - 1 {
            println!(); // empty lines between items
        }
    }

    if items.len() > args.limit {
        println!("\n.... and {} more announcements", items.len() - args.limit);
    }
    Ok(())
}

async fn fetch_and_parse_rss(args: &Args) -> Result<Vec<RssItem>> {
    let client = reqwest::Client::new();
    let response = client
        .get(&args.url)
        .header("User-Agent", "naws/0.1.0")
        .header(
            "User-Agent",
            "Mozilla/5.0 (compatible; naws/0.1.0; +https://github.com/yourhandle/naws)",
        )
        .send()
        .await
        .context("Failed to fetch RSS feed")?;
    if !response.status().is_success() {
        anyhow::bail!("HTTP request failed with status: {}", response.status());
    }
    let xml_content = response
        .text()
        .await
        .context("Failed to read response body")?;

    println!(
        "✅ Successfully fetched RSS feed ({} bytes)",
        xml_content.len()
    );

    // parse xml
    let items = parse_rss_xml(&xml_content)?;
    println!("📝 Found {} announcements", items.len());
    Ok(items)
}

fn parse_rss_xml(xml_content: &str) -> Result<Vec<RssItem>> {
    let feed: RssFeed = quick_xml::de::from_str(xml_content)?;
    Ok(feed.channel.items.into_iter().map(Into::into).collect())
}

fn display_announcement(item: &RssItem, index: usize, total: usize, args: &Args) {
    // link
    println!(
        "{} {}",
        "📢".bright_yellow().bold(),
        format!("[{}]", item.link).blue().underline()
    );

    //title
    println!(
        "   {} {}",
        item.title.bright_white().bold(),
        format!("({}/{})", index, total).dimmed()
    );

    if let Some(date) = &item.pub_date {
        println!("  {}", format_date(date).cyan());
    }

    // categories
    if !item.categories.is_empty() {
        let categories_str = item.categories.join(", ");
        println!("  {} {}", "🏷️".magenta(), categories_str.magenta());
    }
    if args.show_description {
        let description = format_description(&item.description, args.full_description);
        if !description.is_empty() {
            println!("  {} {}", "📄".yellow(), description.white());
        }
    }
}

fn format_description(html: &Option<String>, full_description: bool) -> String {
    let Some(html_content) = html else {
        return String::new();
    };
    if full_description {
        html2text::from_read(html_content.as_bytes(), 80).unwrap()
    } else {
        let text = html2text::from_read(html_content.as_bytes(), 80).unwrap();
        let summary = text
            .split_whitespace()
            .take(50)
            .collect::<Vec<_>>()
            .join(" ");
        if text.split_whitespace().count() > 50 {
            format!("{}...", summary)
        } else {
            summary
        }
    }
}
