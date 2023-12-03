use crate::db_manager::DBManager;

// Signal sender is non-clonable therefore we need to create a new one for each server.
// https://github.com/rust-lang/futures-rs/issues/1971
pub async fn http_sigint() {
    wait_for_signal().await;

    println!("http shutdown complete");
}

pub async fn grpc_sigint(dbm: DBManager) {
    wait_for_signal().await;

    // Shutdown the DB connection.
    dbm.close_db()
        .await
        .expect("Failed to close database connection");
    println!("DB connection closed");

    println!("gRPC shutdown complete");
}
/// Registers signal handlers and waits for a signal that
/// indicates a shutdown request.
pub(crate) async fn wait_for_signal() {
    wait_for_signal_impl().await
}

/// Waits for a signal that requests a graceful shutdown, like SIGTERM, SIGINT (Ctrl-C), or SIGQUIT.
#[cfg(unix)]
async fn wait_for_signal_impl() {
    use tokio::signal::unix::{signal, SignalKind};

    // https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html
    let mut signal_terminate = signal(SignalKind::terminate()).unwrap();
    let mut signal_interrupt = signal(SignalKind::interrupt()).unwrap();
    let mut signal_quit = signal(SignalKind::quit()).unwrap();

    tokio::select! {
        _ = signal_terminate.recv() => {println!("Received SIGTERM")},
        _ = signal_interrupt.recv() => println!("Received SIGINT"),
        _ = signal_quit.recv() => println!("Received SIGQUIT"),
    }
}

/// Waits for a signal that requests a graceful shutdown, Ctrl-C (SIGINT).
#[cfg(windows)]
async fn wait_for_signal_impl() {
    use tokio::signal::windows;

    // Infos here:
    // https://learn.microsoft.com/en-us/windows/console/handlerroutine
    let mut signal_c = windows::ctrl_c().unwrap();
    let mut signal_break = windows::ctrl_break().unwrap();
    let mut signal_close = windows::ctrl_close().unwrap();
    let mut signal_shutdown = windows::ctrl_shutdown().unwrap();

    tokio::select! {
        _ = signal_c.recv() => println!("Received CTRL_C."),
        _ = signal_break.recv() => println!("Received CTRL_BREAK."),
        _ = signal_close.recv() => println!("Received CTRL_CLOSE."),
        _ = signal_shutdown.recv() => println!("Received CTRL_SHUTDOWN."),
    }
}
