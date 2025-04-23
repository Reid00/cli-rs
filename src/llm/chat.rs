use anyhow::{Context, Result};
use async_openai::{Client, config::OpenAIConfig, types::CreateCompletionRequestArgs};
use futures::StreamExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// An enum that determines the policy of dumping chat context into the file
#[derive(Debug, Clone)]
pub enum DumpPolicy {
    /// Never dump any change, file will always remain read-only
    NeverDump,
    AutoDump,
    DumpRelyRequest,
    PeriodicDump(Duration),
}

#[derive(Debug, Clone)]
pub struct Completion {
    /// chat context
    pub context: Vec<String>,

    /// 上下文保存到本地
    pub file: PathBuf,

    /// dump 的策略
    pub dump_policy: DumpPolicy,

    /// context 保存到上下文的时间
    pub last_dump: Instant,

    /// openai client
    pub client: Client<OpenAIConfig>,
}

impl Completion {
    pub fn new<P: AsRef<Path>>(
        api_key: &str,
        base_url: &str,
        ctx_path: P,
        dump_policy: DumpPolicy,
    ) -> Self {
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url)
            .with_org_id("CLI");

        let mut path_buf = PathBuf::new();
        path_buf.push(ctx_path);

        let client = Client::with_config(config);

        println!("cli : {:?}", client.config());

        Self {
            context: vec![],
            file: path_buf,
            dump_policy: dump_policy,
            last_dump: Instant::now(),
            client: client,
        }
    }

    pub async fn completion(
        &self,
        model: &str,
        prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<()> {
        let req = CreateCompletionRequestArgs::default()
            .model(model)
            .prompt(prompt)
            .stream(true)
            .max_tokens(max_tokens.unwrap_or(4096_u32))
            .build()
            .context("failed to build completion request")?;

        let mut stream = self.client.completions().create_stream(req).await?;

        while let Some(response) = stream.next().await {
            match response {
                Ok(ccr) => ccr.choices.iter().for_each(|c| {
                    println!("{}", c.text);
                }),
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                }
            }
        }
        Ok(())
    }
}
