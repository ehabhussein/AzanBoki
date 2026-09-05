use anyhow::{Context, Result};
use auto_launch::AutoLaunchBuilder;

pub fn set_start_on_login(enabled: bool) -> Result<()> {
    let executable = std::env::current_exe().context("could not locate the AzanBoki executable")?;
    let path = executable
        .to_str()
        .context("the AzanBoki executable path is not valid Unicode")?;
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    let launch_path = format!("\"{path}\"");
    #[cfg(target_os = "macos")]
    let launch_path = path.to_string();
    let auto_launch = AutoLaunchBuilder::new()
        .set_app_name("AzanBoki")
        .set_app_path(&launch_path)
        .set_use_launch_agent(true)
        .set_args(&["--background"])
        .build()
        .context("could not configure start-on-login")?;
    if enabled {
        auto_launch
            .enable()
            .context("could not enable start-on-login")
    } else {
        auto_launch
            .disable()
            .context("could not disable start-on-login")
    }
}
