use crate::cli::Args;
use crate::rss::RssItem;
use crate::utils::format_date;
use colored::Colorize;

pub fn display_announcement(item: &RssItem, index: usize, total: usize, args: &Args) {
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

pub fn format_description(html: &Option<String>, full_description: bool) -> String {
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
