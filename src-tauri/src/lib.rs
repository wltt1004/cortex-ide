#![allow(dead_code)]
//! Cortex Desktop - Tauri application backend
//!
//! This module provides the Rust backend for the Cortex Desktop application.
//! Command registration is split into feature-grouped modules under `app/`.

mod acp;
mod action_log;
mod activity;
mod ai;
mod app;
mod auto_update;
mod batch;
mod batch_ipc;
mod browser;
mod collab;
mod commands;
mod context_server;
mod cortex_engine;
mod cortex_protocol;
mod cortex_storage;
mod dap;
mod deep_link;
mod diagnostics;
mod editor;
pub mod error;
mod extensions;
mod factory;
mod formatter;
mod fs;
mod fs_commands;
mod git;
mod i18n;
mod keybindings;
mod language_selector;
mod lsp;
mod mcp;
mod models;
mod notebook;
mod process;
mod process_utils;
mod project;
mod prompt_store;
mod remote;
mod repl;
mod rules_library;
mod sandbox;
mod search;
mod settings;
mod settings_sync;
#[cfg(feature = "remote-ssh")]
mod ssh_terminal;
mod system_specs;
mod tasks;
mod terminal;
mod testing;
mod themes;
mod timeline;
mod toolchain;
mod window;
mod workspace;
mod workspace_settings;
mod wsl;

use std::sync::{Arc, OnceLock};

use tracing::{error, info};

pub use error::CortexError;

/// Lazy initialization wrapper for heavy state managers.
/// Uses `OnceLock` to defer initialization until first access.
pub struct LazyState<T> {
    inner: OnceLock<T>,
    init: fn() -> T,
}

impl<T> LazyState<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            inner: OnceLock::new(),
            init,
        }
    }

    pub fn get(&self) -> &T {
        self.inner.get_or_init(self.init)
    }

    pub fn is_initialized(&self) -> bool {
        self.inner.get().is_some()
    }
}

impl<T: Clone> Clone for LazyState<T> {
    fn clone(&self) -> Self {
        let new_state = Self::new(self.init);
        if let Some(value) = self.inner.get() {
            let _ = new_state.inner.set(value.clone());
        }
        new_state
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::{Manager, WindowEvent};

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .try_init();

    let _startup_span = tracing::info_span!("startup").entered();
    info!("Starting Cortex Desktop with optimized startup...");
    let startup_time = std::time::Instant::now();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init());

    // Desktop-only plugins: single instance guard, auto-updater, deep links.
    // Registered before MCP bridge so they initialize first — the MCP bridge
    // injects JavaScript into the WebView during init, so placing it last
    // avoids interfering with earlier plugin setup on Windows.
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_deep_link::init());

    // MCP Bridge plugin — debug builds only.
    // On Windows, the plugin's WebView2 JavaScript injection during init can
    // trigger a STATUS_ACCESS_VIOLATION in certain environments (containers,
    // headless, missing WebView2 runtime). Set CORTEX_SKIP_MCP_BRIDGE=1 to
    // disable the plugin if you hit this crash.
    #[cfg(debug_assertions)]
    let builder = if std::env::var("CORTEX_SKIP_MCP_BRIDGE").is_ok() {
        info!("Skipping MCP Bridge plugin (CORTEX_SKIP_MCP_BRIDGE is set)");
        builder
    } else {
        builder.plugin(
            tauri_plugin_mcp_bridge::Builder::new()
                .bind_address("127.0.0.1")
                .build(),
        )
    };

    let remote_manager = Arc::new(remote::RemoteManager::new());

    let builder = app::register_state(builder, remote_manager);

    let app = match builder
        .invoke_handler(app::cortex_commands!())
        .setup(move |tauri_app| app::setup_app(tauri_app, startup_time))
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                let label = window.label();
                let app_handle = window.app_handle();
                crate::window::remove_window_session(app_handle, label);
            }
        })
        .build(tauri::generate_context!())
    {
        Ok(app) => app,
        Err(e) => {
            error!("Failed to build Tauri application: {}", e);
            std::process::exit(1);
        }
    };

    app.run(|app, event| {
        app::handle_run_event(app, event);
    });
}
