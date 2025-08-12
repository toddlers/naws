# naws - AWS What's New CLI

A simple, command-line tool written in Rust to fetch and display the latest announcements from 
the AWS "What's New" RSS feed directly in terminal.

---

## Features
- **Filtering**: Search for announcements by keyword across titles, descriptions, and categories.
- **Flexible Descriptions**: View a short summary or the full announcement description.
- **Multiple Formats**: Output as human-readable, colored text or as structured JSON.


Usage

The basic command fetches the 10 most recent announcements.naws
```bash
naws
```

Examples

Show 5 recent announcements with full descriptions:
```bash
naws -l 5 -F
```

Filter for announcements related to "Lambda" and show descriptions:

```bash
naws --filter "Lambda" --show-description
```
Get the 3 latest "EKS" announcements in JSON format:

```bash
naws -l 3 -f "EKS" -j
```
