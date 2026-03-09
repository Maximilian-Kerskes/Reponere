use crate::{
    build::registry::registry_handler::Registry,
    cli::args::{Arg, SubArgs},
    handlers::{
        install_handler::{self, InstallEvent, InstallResult},
        list_handler::{self, ListEvent},
        uninstall_handler::{self, UninstallError, UninstallEvent, UninstallPlan},
    },
    util::context::Context,
};
use clap::Parser;
use dialoguer::Confirm;

pub fn run(ctx: &mut Context) {
    let args = Arg::parse();

    match args.sub {
        SubArgs::Install { packages } => {
            install(ctx, packages);
        }
        SubArgs::Uninstall { packages } => {
            uninstall(ctx, packages);
        }
        SubArgs::Sync => {
            sync(ctx);
        }
        SubArgs::List {
            packages,
            available,
        } => {
            list(ctx, packages, available);
        }
        _ => todo!(),
    }
}

fn show_install_progress(event: InstallEvent) {
    match event {
        InstallEvent::InstallingDependencies => {
            println!("==> Installing dependencies");
        }
        InstallEvent::DependencyAlreadyInstalled { name } => {
            println!("-> dependency {name} already installed");
        }
        InstallEvent::InstallingDependency { name } => {
            println!("-> installing dependency {name}...");
        }
        InstallEvent::InstallingRunTimeDependencies { dependencies } => {
            println!("==> Installing runtime dependencies: {dependencies:?}");
        }
        InstallEvent::InstallingBuildDependencies { dependencies } => {
            println!("==> Installing build dependencies: {dependencies:?}");
        }
        InstallEvent::FetchingSource => {
            println!("==> Fetching source");
        }
        InstallEvent::BuildingSource => {
            println!("==> Building source");
        }
        InstallEvent::BuildStep { step } => {
            println!("-> {step}");
        }
        InstallEvent::Cleanup => {
            println!("==> Cleanup");
        }
        InstallEvent::Finished => {
            println!("==> Finished");
        }
    }
}

fn show_uninstall_progress(event: UninstallEvent) {
    match event {
        UninstallEvent::UninstallingDependencies => {
            println!("==> Uninstalling dependencies");
        }
        UninstallEvent::UninstallingDependency { name } => {
            println!("-> uninstalling dependency {name}...");
        }
        UninstallEvent::DependencyAlreadyUninstalled { name } => {
            println!("-> dependency {name} already uninstalled");
        }
        UninstallEvent::RemovingPackageFiles => {
            println!("==> Removing package files");
        }
        UninstallEvent::Cleanup => {
            println!("==> Cleanup");
        }
        UninstallEvent::Finished => {
            println!("==> Finished");
        }
    }
}

fn show_list_progress(event: ListEvent) {
    match event {
        ListEvent::Available => {
            println!("==> Available packages:");
        }
        ListEvent::AvailablePackage(name, version) => {
            println!("-> {name}: {version}");
        }
        ListEvent::Installed => {
            println!("==> Installed packages:");
        }
        ListEvent::InstalledPackage(name, version) => {
            println!("-> {name}: {version}");
        }
    }
}

fn install(ctx: &mut Context, packages: Vec<String>) {
    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to install the following packages?\n - {}",
            packages.join("\n - ")
        ))
        .interact()
        .unwrap()
    {
        println!("==> Aborted installation");
        return;
    }

    for package in packages {
        match install_handler::run(
            &ctx.registry,
            &mut ctx.tracker,
            &package,
            false,
            &mut show_install_progress,
        ) {
            Ok(InstallResult::Installed) => {
                println!("==> Installed {package}");
            }

            Ok(InstallResult::AlreadyInstalled) => {
                let reinstall = Confirm::new()
                    .with_prompt("Package already installed. Reinstall?")
                    .interact()
                    .unwrap();

                if !reinstall {
                    continue;
                }

                match install_handler::run(
                    &ctx.registry,
                    &mut ctx.tracker,
                    &package,
                    true,
                    &mut show_install_progress,
                ) {
                    Ok(_) => println!("==> Reinstalled {package}"),
                    Err(e) => println!("==> Failed to reinstall {package}: {e}"),
                }
            }

            Err(e) => {
                println!("==> Failed to install {package}: {e}");
            }
        }
    }
    ctx.tracker
        .save(ctx.config.packages_path.to_str().unwrap())
        .unwrap();
}

fn uninstall(ctx: &mut Context, packages: Vec<String>) {
    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to uninstall the following packages?\n - {}",
            packages.join("\n - ")
        ))
        .interact()
        .unwrap()
    {
        println!("==> Aborted uninstallation");
        return;
    }

    for package in packages {
        let plan: UninstallPlan = match uninstall_handler::plan(&ctx.tracker, &package) {
            Ok(plan) => plan,
            Err(UninstallError::AlreadyUninstalled) => {
                println!("==> Package already uninstalled");
                continue;
            }
            Err(e) => {
                println!("==> something went wrong: {e}");
                continue;
            }
        };

        if !plan.remove_dependencies.is_empty() {
            println!("==> Dependencies to remove:");
            for d in &plan.remove_dependencies {
                println!(" - {}", d.name);
            }
        }

        if !plan.keep_dependencies.is_empty() {
            println!("==> Dependencies that will remain (used by others):");
            for d in &plan.keep_dependencies {
                println!(" - {}", d.name);
            }
        }
        if Confirm::new()
            .with_prompt("Proceed with uninstall?")
            .interact()
            .unwrap()
        {
            match uninstall_handler::execute(&mut ctx.tracker, plan, &mut show_uninstall_progress) {
                Ok(_) => println!("==> Uninstalled {package}"),
                Err(e) => println!("==> Failed to uninstall {package}: {e}"),
            };
        }
        ctx.tracker
            .save(ctx.config.packages_path.to_str().unwrap())
            .unwrap();
    }
}

fn sync(ctx: &mut Context) {
    ctx.registry =
        Registry::resync_from_directory_and_save(&ctx.config.index_path, &ctx.config.registry_path);
    println!("==> Registry synced");
}

fn list(ctx: &Context, packages: Vec<String>, available: bool) {
    match list_handler::run(ctx, packages, available, &mut show_list_progress) {
        Ok(()) => (),
        Err(e) => println!("==> something went wrong: {e}"),
    }
}
