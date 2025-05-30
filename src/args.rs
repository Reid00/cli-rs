use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value_t = String::from("sk-xx"))]
    pub key: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = String::from("https://api.deepseek.com/beta"))]
    pub url: String,
}
