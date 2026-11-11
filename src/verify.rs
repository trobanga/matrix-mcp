use futures_util::StreamExt;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::encryption::verification::{
    SasState, SasVerification, Verification, VerificationRequest, VerificationRequestState,
};
use matrix_sdk::ruma::events::key::verification::request::ToDeviceKeyVerificationRequestEvent;
use matrix_sdk::Client;
use tokio::sync::mpsc;

/// Waits for one emoji verification started from another session of the same
/// account, e.g. Element, and asks on the terminal whether the emojis match.
pub async fn verify(client: Client) -> anyhow::Result<()> {
    client.sync_once(SyncSettings::default()).await?;
    let (done_tx, mut done_rx) = mpsc::unbounded_channel::<String>();
    client.add_event_handler(move |ev: ToDeviceKeyVerificationRequestEvent, client: Client| {
        let done_tx = done_tx.clone();
        async move {
            let request = client
                .encryption()
                .get_verification_request(&ev.sender, &ev.content.transaction_id)
                .await;
            if let Some(request) = request {
                tokio::spawn(handle_request(request, done_tx));
            }
        }
    });
    eprintln!(
        "Waiting. In Element, open Settings > Sessions, select this session \
         (matrix-mcp) and choose Verify."
    );
    let sync_client = client.clone();
    let sync = tokio::spawn(async move { sync_client.sync(SyncSettings::default()).await });
    let outcome = done_rx.recv().await;
    sync.abort();
    eprintln!("{}", outcome.unwrap_or_else(|| "verification stopped".into()));
    Ok(())
}

async fn handle_request(request: VerificationRequest, done: mpsc::UnboundedSender<String>) {
    if let Err(e) = request.accept().await {
        let _ = done.send(format!("cannot accept request: {e}"));
        return;
    }
    let mut changes = request.changes();
    while let Some(state) = changes.next().await {
        match state {
            VerificationRequestState::Transitioned {
                verification: Verification::SasV1(sas),
            } => {
                let _ = done.send(run_sas(sas).await);
                return;
            }
            VerificationRequestState::Done | VerificationRequestState::Cancelled(_) => {
                let _ = done.send("request ended before emoji comparison".into());
                return;
            }
            _ => {}
        }
    }
}

async fn run_sas(sas: SasVerification) -> String {
    if let Err(e) = sas.accept().await {
        return format!("cannot accept verification: {e}");
    }
    let mut changes = sas.changes();
    while let Some(state) = changes.next().await {
        match state {
            SasState::KeysExchanged {
                emojis: Some(emojis),
                ..
            } => {
                eprintln!("\nCompare these emojis with Element:");
                for emoji in emojis.emojis {
                    eprintln!("  {}  {}", emoji.symbol, emoji.description);
                }
                let confirmed = ask("Do they match? [y/N] ").await;
                let result = if confirmed {
                    sas.confirm().await
                } else {
                    sas.mismatch().await
                };
                if let Err(e) = result {
                    return format!("cannot send answer: {e}");
                }
            }
            SasState::Done { .. } => return "verified".into(),
            SasState::Cancelled(info) => return format!("cancelled: {}", info.reason()),
            _ => {}
        }
    }
    "verification stream ended".into()
}

async fn ask(prompt: &str) -> bool {
    let prompt = prompt.to_owned();
    tokio::task::spawn_blocking(move || {
        eprint!("{prompt}");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).is_ok() && line.trim().eq_ignore_ascii_case("y")
    })
    .await
    .unwrap_or(false)
}
