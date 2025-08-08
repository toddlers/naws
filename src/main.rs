mod utils;

#[allow(dead_code)]
#[allow(unused_imports)]
use utils::{decode_tag, format_date};
use anyhow::{Result,Context};
use clap::{arg, Parser};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use colored::*;

#[derive(Parser,Debug)]
#[command(author, version, about, long_about = None)]
struct Args{
    #[arg(short, long,
        default_value = "https://aws.amazon.com/about-aws/whats-new/recent/feed/")
    ]
    url: String,

    // number of items to display
    #[arg(short, long,default_value = "10")]
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
    #[arg(short='F', long)]
    full_description: bool,

    // show description
    #[arg(short='d', long)]
    show_description: bool,

}

// rss item
#[derive(Debug,Clone)]
struct RssItem {
    title: String,
    link: String,
    description: Option<String>,
    pub_date: Option<String>,
    categories: Vec<String>,
}
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("Fetching AWS RSS feeds from: {}", args.url);
    println!("Limit: {} items", args.limit);

    // implement RSS parsing
    let mut items = fetch_and_parse_rss(&args).await?;
    println!("\n📢 AWS Announcements:\n");
    if let Some(ref filter) = args.filter {
        let filter = filter.to_lowercase();
        items = items
            .into_iter()
            .filter(|item|{
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
    let display_count  = std::cmp::min(items.len(), args.limit);

    for (i, mut item) in items.iter().enumerate(){
        if i >= display_count {
            break;
        }
        display_announcement(&mut item, i + 1, display_count, &args);
        if i < display_count - 1 {
            println!(); // empty lines between items
        }
    }

    if items.len() > args.limit {
        println!("\n.... and {} more announcements", items.len()-args.limit);
    }
    Ok(())
}


async fn fetch_and_parse_rss(args: &Args) -> Result<Vec<RssItem>> {
    let client = reqwest::Client::new();
    let response = client
        .get(&args.url)
        .header("User-Agent","naws/0.1.0")
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

    println!("✅ Successfully fetched RSS feed ({} bytes)", xml_content.len());

    // parse xml
    let items = parse_rss_xml(&xml_content, &args)?;
    println!("📝 Found {} announcements", items.len());
    Ok(items)
}



fn parse_rss_xml(xml_content :&str, args: &Args) -> Result<Vec<RssItem>> {
    let mut reader = Reader::from_str(xml_content);
    // seems like func doesn't exist
    reader.config_mut().trim_text(true);
    let mut items = Vec::new();
    let mut buf = Vec::new();
    let mut current_item: Option<HashMap<String,String>> = None;
    let mut current_tag = String::new();
    let mut inside_item = false;
    let mut current_categories: Vec<String> = Vec::new();
    loop {
       match reader.read_event_into(&mut buf) {
           Ok(Event::Start(ref e)) => {
               let tag_name = decode_tag(&reader, e.name().as_ref())?.to_ascii_lowercase();
               if tag_name == "item" {
                   inside_item = true;
                   current_item = Some(HashMap::new());
                   current_categories.clear();
               } else if inside_item {
                   current_tag = tag_name
               }
           }
           Ok(Event::Text(e)) => {
               if inside_item && !current_tag.is_empty() {
                   if let Some(ref mut item) = current_item {
                       let reader_decoder = reader.decoder();
                       // let text = match std::str::from_utf8(&*e){
                       //     Ok(s) => s.to_string(),
                       //     Err(_) => String::from_utf8_lossy(&*e).to_string(),
                       // };
                       let text = reader_decoder.decode(e.as_ref())?.to_string();
                       if current_tag == "category" {
                           current_categories.push(text);
                       } else {
                           // handle multiple text nodes for same tag
                           match item.get(&current_tag) {
                               Some(existing) => {
                                   item.insert(
                                       current_tag.clone(), format!("{} {}", existing, text));
                               }
                               None => {
                                   item.insert(current_tag.clone(), text);
                               }
                           }
                       }
                   }
               }
           }
           Ok(Event::End(ref e)) => {
               let tag_name = decode_tag(&reader, e.name().as_ref())?.to_ascii_lowercase();
               if tag_name == "item" {
                   inside_item = false;
               if let Some(item_data) = current_item.take() {
                   // create RssItem from collected data
                   if args.show_description {
                       let rss_item = RssItem {
                           title: item_data
                               .get("title")
                               .map_or_else(|| "#Untitled".to_string(), String::clone),
                           link: item_data
                               .get("link")
                               .map_or_else(|| "#Untitled".to_string(), String::clone),
                           description: Option::from(item_data
                               .get("description")
                               .map_or_else(|| "#".to_string(), String::clone)),
                           pub_date: item_data.get("pubdate").cloned(),
                           categories: current_categories.clone(),
                       };
                       items.push(rss_item);
                   } else {
                       let rss_item = RssItem {
                           title: item_data
                               .get("title")
                               .map_or_else(|| "#Untitled".to_string(), String::clone),
                           link: item_data
                               .get("link")
                               .map_or_else(|| "#Untitled".to_string(), String::clone),
                           description: None,
                           pub_date: item_data.get("pubdate").cloned(),
                           categories: current_categories.clone(),
                       };
                       items.push(rss_item);
                   }

               }
               } else if inside_item && tag_name == current_tag {
               current_tag.clear();
           }
       }
        Ok(Event::Eof) => break,
        Err(e) => return Err(anyhow::anyhow!("Error parsing XML: {}" ,e)),
        _ => {}
       }
        buf.clear()
    }
    Ok(items)
}


fn display_announcement(item: &RssItem, index: usize, total: usize, args: &Args) {
    // link
    println!("{} {}",
        "📢".bright_yellow().bold(),
             format!("[{}]", item.link).blue().underline());

    //title
    println!("   {} {}",
             item.title.bright_white().bold(),
             format!("({}/{})", index, total).dimmed());

    if let Some(date) = &item.pub_date{
        println!("  {}",
                 format_date(date).cyan());
    }

    // categories
    if !item.categories.is_empty() {
        let categories_str = item.categories.join(", ");
        println!("  {} {}",
                 "🏷️".magenta(),
            categories_str.magenta()
        );
    }
    if args.show_description {
        let description = if args.full_description {
            clean_html_tags_for_terminal(&item.description)
        } else {
            create_summary(&item.description)
        };

        println!("  {} {}",
                 "📄".yellow(), description.white());
    }
}


fn create_summary(html: &Option<String>) -> String {
    let cleaned = clean_html_tags_for_terminal(html);

    let words: Vec<&str> = cleaned.split_whitespace().collect();
    if words.len() <= 50 {
        cleaned
    } else {
        // find a breaking point
        let first_part: String = words.iter().take(50)
            .map(|&s| s)
            .collect::<Vec<_>>().join(" ");
        if let Some(pos) = first_part.rfind('.'){
            format!("{}...", &first_part[..pos+1])
        } else {
            format!("{}...", first_part)
        }

    }
}


// just don't get into html parsing
// take a lot of code
fn clean_html_tags_for_terminal(html: &Option<String>) -> String {
    let html_str = match html {
        Some(s) => s,
        None => return String::new(),
    };

    // More comprehensive HTML cleaning
    let text = html_str
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("<p>", "")
        .replace("</p>", " ")
        .replace("<br>", " ")
        .replace("<br/>", " ")
        .replace("<br />", " ")
        .replace("<div>", "")
        .replace("</div>", " ")
        .replace("<span>", "")
        .replace("</span>", "")
        .replace("<strong>", "")
        .replace("</strong>", "")
        .replace("<b>", "")
        .replace("</b>", "")
        .replace("<i>", "")
        .replace("</i>", "")
        .replace("<em>", "")
        .replace("</em>", "")
        .replace("\n", " ")
        .replace("\r", "")
        .replace("\t", " ");

    // Remove any remaining HTML tags using a more robust approach
    let mut result = String::new();
    let mut inside_tag = false;

    for ch in text.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => result.push(ch),
            _ => {}
        }
    }

    // Clean up whitespace
    let words: Vec<&str> = result.split_whitespace().collect();
    words.join(" ")
}
