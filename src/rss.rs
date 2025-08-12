use crate::cli::Args;
use anyhow::Context;
use serde::{Deserialize, Serialize};

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

pub fn parse_rss_xml(xml_content: &str) -> anyhow::Result<Vec<RssItem>> {
    let feed: RssFeed = quick_xml::de::from_str(xml_content)?;
    Ok(feed.channel.items.into_iter().map(Into::into).collect())
}

pub async fn fetch_and_parse_rss(args: &Args) -> anyhow::Result<Vec<RssItem>> {
    let client = reqwest::Client::new();
    let response = client
        .get(&args.url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (compatible; naws/0.1.0; +https://github.com/toddlers/naws)",
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
