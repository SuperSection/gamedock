use crate::cli::OptimizeArgs;
use console::style;
use gamedock_core::AppConfig;
use gamedock_optimizer::Optimizer;

pub async fn optimize(args: OptimizeArgs) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let optimizer = Optimizer::new(config)?;

    if args.reset {
        optimizer.reset_all().await?;
        println!("{} Reset", style("✓").green());
        return Ok(());
    }

    if args.gamemode {
        optimizer.enable_gamemode().await?;
        println!("{} GameMode enabled", style("✓").green());
    }

    if args.mangohud {
        optimizer.enable_mangohud(None).await?;
        println!("{} MangoHUD enabled", style("✓").green());
    }

    if let Some(governor) = &args.cpu_governor {
        optimizer.set_cpu_governor(governor).await?;
        println!("{} CPU governor: {}", style("✓").green(), governor);
    }

    if !args.gamemode && !args.mangohud && args.cpu_governor.is_none() {
        optimizer.optimize_all().await?;
        println!("{} All optimizations applied", style("✓").green());
    }

    Ok(())
}
