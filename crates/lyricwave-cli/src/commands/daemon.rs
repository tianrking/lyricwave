use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use lyricwave_core::pipeline::{DaemonEvent, MockAsrProvider, MockTranslatorProvider};
use lyricwave_core::service::PipelineService;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

pub fn run(source_lang: &str, target_lang: &str, interval_ms: u64, count: u32) -> Result<()> {
    let service = PipelineService::new(MockAsrProvider, MockTranslatorProvider, 128);

    let status_evt = DaemonEvent::Status {
        message: "daemon started".to_string(),
        emitted_at_ms: DaemonEvent::now_ms(),
    };
    println!("{}", serde_json::to_string(&status_evt)?);

    for idx in 1..=count {
        let text = format!("sample line {idx}");
        let transcript = service.process_text(&text, source_lang, target_lang);
        let evt = DaemonEvent::Transcript {
            payload: transcript,
            emitted_at_ms: DaemonEvent::now_ms(),
        };
        println!("{}", serde_json::to_string(&evt)?);
        thread::sleep(Duration::from_millis(interval_ms));
    }

    let done_evt = DaemonEvent::Status {
        message: "daemon finished".to_string(),
        emitted_at_ms: DaemonEvent::now_ms(),
    };
    println!("{}", serde_json::to_string(&done_evt)?);

    Ok(())
}

pub async fn serve(
    host: &str,
    port: u16,
    source_lang: &str,
    target_lang: &str,
    interval_ms: u64,
) -> Result<()> {
    let bind_addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind daemon server at {bind_addr}"))?;

    println!("daemon tcp json stream listening on {bind_addr}");

    let (tx, _) = broadcast::channel::<String>(256);

    let producer_tx = tx.clone();
    let source = source_lang.to_string();
    let target = target_lang.to_string();
    tokio::spawn(async move {
        let service = PipelineService::new(MockAsrProvider, MockTranslatorProvider, 128);

        let started = DaemonEvent::Status {
            message: "daemon started".to_string(),
            emitted_at_ms: DaemonEvent::now_ms(),
        };
        if let Ok(line) = serde_json::to_string(&started) {
            let _ = producer_tx.send(line);
        }

        let mut idx: u64 = 1;
        loop {
            let text = format!("live line {idx}");
            let transcript = service.process_text(&text, &source, &target);
            let evt = DaemonEvent::Transcript {
                payload: transcript,
                emitted_at_ms: DaemonEvent::now_ms(),
            };
            if let Ok(line) = serde_json::to_string(&evt) {
                let _ = producer_tx.send(line);
            }
            idx = idx.saturating_add(1);
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
        }
    });

    loop {
        let (mut socket, peer) = listener.accept().await?;
        let mut rx = tx.subscribe();
        println!("client connected: {peer}");
        tokio::spawn(async move {
            while let Ok(line) = rx.recv().await {
                if socket.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                if socket.write_all(b"\n").await.is_err() {
                    break;
                }
            }
        });
    }
}
