use std::{
    fs,
    io::{self, Read},
};

use clap::Parser;
use hql::{html, querier};

#[derive(Debug, Parser)]
#[command(author, version, about = "A human-friendly Html Query Language\n\nIt has three possible mode to receive html, with priority from high to low: file, inline argument and stdin", long_about = None)]
struct Cli {
    /// Html Query Language
    #[arg(long, value_name = "HQL")]
    hql: String,

    /// Input HTML file needed to be searched
    #[arg(short, long, value_name = "FILE")]
    file: Option<String>,

    /// Inline HTML string
    document: Option<String>,
}

fn main() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let q = querier::Querier::try_parse(&cli.hql)
        .unwrap_or_else(|e| panic!("failed to parse hql: {}", e));

    let mut doc_str = String::new();
    if let Some(file) = cli.file {
        doc_str =
            fs::read_to_string(&file).unwrap_or_else(|e| panic!("file {} not found: {}", file, e));
    } else if let Some(doc) = cli.document {
        doc_str = doc;
    } else {
        io::stdin()
            .read_to_string(&mut doc_str)
            .unwrap_or_else(|e| panic!("failed to read stdin to string: {}", e));
    }

    let doc = html::Html::parse_document(&doc_str, false);

    q.query_document(&doc)
        .into_iter()
        .for_each(|n| println!("{}", n));
}
