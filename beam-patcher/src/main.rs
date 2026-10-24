#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use beam_core::Config;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.yml")]
    config: String,
    
    #[arg(long)]
    headless: bool,
    
    #[arg(short, long)]
    manual_patch: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Velopack launches the main EXE with --veloapp-* lifecycle hooks.
    // A preview-only build must exit immediately rather than showing the
    // Beam window during install/uninstall. Normal Beam remains unchanged.
    #[cfg(feature = "preview-only")]
    if std::env::args_os()
        .skip(1)
        .any(|arg| arg.to_string_lossy().starts_with("--veloapp-"))
    {
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // Preview builds never read/write config.yml or invoke the headless patcher.
    // The packaged visual UI is distinct from the normal Beam release.
    #[cfg(feature = "preview-only")]
    {
        let mut config = Config::default();
        config.app.name = "Ragnar Classic Beam Preview".to_string();
        config.app.window_title = "Ragnar Classic — UI Preview Only".to_string();
        config.app.server_name = Some("Ragnar Ragnarok Online".to_string());
        config.app.video_background_enabled = Some(false);
        config.app.bgm_autoplay = Some(false);
        config.ui.theme = "ragnar-classic".to_string();
        config.ui.news_feed_url = None;
        config.ui.server_status_url = None;
        config.ui.status_image_url = None;
        config.app.game_directory = None;
        beam_ui::run_ui(config)?;
        return Ok(());
    }

    #[cfg(not(feature = "preview-only"))]
    { 
    let args = Args::parse();
    
    let config_path = if std::path::Path::new(&args.config).is_absolute() {
        std::path::PathBuf::from(&args.config)
    } else {
        let exe_dir = std::env::current_exe()?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get executable directory"))?
            .to_path_buf();
        exe_dir.join(&args.config)
    };
    
    let config = if config_path.exists() {
        Config::load(&config_path)?
    } else {
        tracing::warn!("Config file not found at {:?}, creating default config", config_path);
        let config = Config::default();
        config.save(&config_path)?;
        config
    };
    
    if args.headless {
        run_headless(config, args.manual_patch).await?;
    } else {
        beam_ui::run_ui(config)?;
    }
    
    Ok(())
    }
}

#[cfg(not(feature = "preview-only"))]
async fn run_headless(config: Config, manual_patch: Option<String>) -> Result<()> {
    let patcher = beam_core::Patcher::new(config)?;
    
    if let Some(patch_path) = manual_patch {
        tracing::info!("Applying manual patch: {}", patch_path);
        patcher.manual_patch(std::path::Path::new(&patch_path)).await?;
    } else {
        tracing::info!("Starting full patch process");
        patcher.run_full_patch().await?;
    }
    
    tracing::info!("Patching completed successfully");
    Ok(())
}
