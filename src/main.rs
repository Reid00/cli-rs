use anyhow::Result;
use chrono::{Local, Utc};
use clap::Parser;
use llm::chat::Completion;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout};

#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
#[cfg(windows)]
use tokio::signal::windows::ctrl_c;

pub mod args;
pub mod llm;

/// 带时间戳的消息格式
fn format_message(sender: &str, message: &str) -> String {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    format!("[{}] {}: {}", now, sender, message)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::Args::parse();

    let api_key = args.key;
    let base_url = args.url;

    println!(
        "{}",
        format_message("SYSTEM", &format!("当前时间: {}", Utc::now()))
    );
    println!("{}", format_message("SYSTEM", "已连接到 DeepSeek 大模型"));
    println!(
        "{}",
        format_message("SYSTEM", "输入 'quit' 或 'q' 退出对话")
    );
    println!(
        "{}",
        format_message("SYSTEM", "--------------------------------")
    );

    // 修改信号处理部分，确保跨平台正确响应
    #[cfg(unix)]
    async fn get_signal() {
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        sigint.recv().await;
    }

    #[cfg(windows)]
    async fn get_signal() {
        ctrl_c().unwrap().recv().await;
    }

    let c = Completion::new(
        api_key.as_str(),
        base_url.as_str(),
        "ctx.txt",
        llm::chat::DumpPolicy::NeverDump,
    );

    println!("now: {}", Utc::now());

    let mut stdout = stdout();
    let stdin = stdin();
    let mut stdin = BufReader::new(stdin);
    loop {
        tokio::select! {
            _ = get_signal() => {
                println!("\n{}", format_message("SYSTEM", "检测到终止信号正在退出!"));
                break;
            }
            result = async {

                // print!("{}", format_message("YOU", ""));
                stdout.flush().await?;

                let mut input = String::new();
                match stdin.read_line(&mut input).await {
                    Ok(_) => {
                        let input = input.trim();

                        if input.is_empty(){
                            return Ok::<(), anyhow::Error>(());
                        }

                        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("q") {
                            println!("{}", format_message("SYSTEM", "正在结束对话..."));
                            std::process::exit(0);
                        }


                        println!("{}", format_message("YOU", input));
                        let resp = input.to_uppercase();
                        println!("{}", format_message("AI", &resp));

                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
                Ok(())
            } => match result {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("操作错误: {}", e);
                }
            }
        }
    }

    println!("{}", format_message("SYSTEM", "对话已结束"));
    Ok(())
}
