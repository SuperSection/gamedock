use gamedock_core::AppConfig;
use gamedock_optimizer::Optimizer;
use console::style;

pub async fn system_info() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let optimizer = Optimizer::new(config.clone())?;
    let sys_info = optimizer.get_system_info().await?;

    println!("{}", style("System").cyan().bold());
    println!("  CPU:        {}", sys_info.cpu_model);
    println!("  Cores:      {}", sys_info.cpu_cores);
    println!("  Memory:     {} MB / {} MB", sys_info.available_memory_mb, sys_info.total_memory_mb);
    println!("  Kernel:     {}", sys_info.kernel_version);
    if let Some(ref distro) = sys_info.distro {
        println!("  Distro:     {}", distro);
    }
    println!("  Display:    {}", sys_info.display_server);
    if let Some(ref compositor) = sys_info.compositor {
        println!("  Compositor: {}", compositor);
    }
    println!();

    println!("{}", style("Tools").cyan().bold());
    println!("  GameMode:   {}", if sys_info.has_gamemode { style("yes").green().to_string() } else { style("no").red().to_string() });
    println!("  MangoHUD:   {}", if sys_info.has_mangohud { style("yes").green().to_string() } else { style("no").red().to_string() });
    println!("  Waydroid:   {}", if sys_info.has_waydroid { style("yes").green().to_string() } else { style("no").red().to_string() });

    if let Some(ref gpu) = sys_info.gpu_name {
        println!();
        println!("{}", style("GPU").cyan().bold());
        println!("  Device:     {}", gpu);
        if let Some(ref driver) = sys_info.gpu_driver {
            println!("  Driver:     {}", driver);
        }
    }

    Ok(())
}
