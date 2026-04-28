use crate::{
    build::registry::registry_handler::Registry,
    cli::{
        args::{Arg, SubArgs},
        presenter::Presenter,
    },
    handlers::{
        events::UpdateEvent,
        install_handler::{self, InstallResult},
        list_handler::{self},
        show_handler,
        update_handler::{self, UpdateStatus},
        uninstall_handler::{self, UninstallError, UninstallPlan},
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
        SubArgs::Update { packages } => {
            update(ctx, packages);
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
        SubArgs::Show { package } => {
            show(ctx, &package);
        }
    }
}

fn install(ctx: &mut Context, packages: Vec<String>) {
    let mut presenter = |event| Presenter::display(&event);

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
            &mut presenter,
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
                    &mut presenter,
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
    let mut presenter = |event| Presenter::display(&event);

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
            match uninstall_handler::execute(&mut ctx.tracker, plan, &mut presenter) {
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

fn update(ctx: &mut Context, packages: Vec<String>) {
    let targets = if packages.is_empty() {
        let mut installed = ctx
            .tracker
            .get_packages()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        installed.sort();
        installed
    } else {
        packages
    };

    if targets.is_empty() {
        println!("==> No installed packages to update");
        return;
    }

    let mut plans = Vec::new();

    for package in targets {
        let mut update_presenter = |event| Presenter::display(&event);
        match update_handler::run(ctx, &package, &mut update_presenter) {
            Ok(UpdateStatus::UpdateAvailable(plan)) => plans.push(plan),
            Ok(UpdateStatus::AlreadyUpToDate) => {}
            Ok(UpdateStatus::AheadOfRegistry) => {}
            Err(update_handler::UpdateError::PackageNotInstalled(_)) => {
                Presenter::display(&UpdateEvent::PackageNotInstalled { name: package })
            }
            Err(update_handler::UpdateError::PackageNotFound(_)) => {
                Presenter::display(&UpdateEvent::PackageNotFound { name: package })
            }
        }
    }

    if plans.is_empty() {
        println!("==> No packages need updates");
        return;
    }

    let prompt = plans
        .iter()
        .map(|plan| {
            format!(
                "{} ({} -> {})",
                plan.name, plan.installed_version, plan.latest_version
            )
        })
        .collect::<Vec<_>>()
        .join("\n - ");

    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to update the following packages?\n - {prompt}"
        ))
        .interact()
        .unwrap()
    {
        println!("==> Aborted update");
        return;
    }

    for plan in plans {
        println!(
            "==> Updating {} ({} -> {})",
            plan.name, plan.installed_version, plan.latest_version
        );

        let uninstall_plan = match uninstall_handler::plan(&ctx.tracker, &plan.name) {
            Ok(plan) => plan,
            Err(UninstallError::AlreadyUninstalled) => {
                println!("==> Package already uninstalled");
                continue;
            }
            Err(e) => {
                println!("==> Failed to prepare uninstall for {}: {e}", plan.name);
                continue;
            }
        };

        let mut uninstall_presenter = |event| Presenter::display(&event);
        if let Err(e) = uninstall_handler::execute(
            &mut ctx.tracker,
            uninstall_plan,
            &mut uninstall_presenter,
        ) {
            println!("==> Failed to uninstall {}: {e}", plan.name);
            ctx.tracker
                .save(ctx.config.packages_path.to_str().unwrap())
                .unwrap();
            continue;
        }

        let package_spec = format!("{}@{}", plan.name, plan.latest_version);
        let mut install_presenter = |event| Presenter::display(&event);
        match install_handler::run(
            &ctx.registry,
            &mut ctx.tracker,
            &package_spec,
            false,
            &mut install_presenter,
        ) {
            Ok(InstallResult::Installed) => {
                println!("==> Updated {}", plan.name);
            }
            Ok(InstallResult::AlreadyInstalled) => {
                println!("==> Failed to update {}: package still marked as installed", plan.name);
            }
            Err(e) => {
                println!("==> Failed to install updated {}: {e}", plan.name);
            }
        }

        ctx.tracker
            .save(ctx.config.packages_path.to_str().unwrap())
            .unwrap();
    }
}

fn list(ctx: &Context, packages: Vec<String>, available: bool) {
    let mut presenter = |event| Presenter::display(&event);

    match list_handler::run(ctx, packages, available, &mut presenter) {
        Ok(()) => (),
        Err(e) => println!("==> something went wrong: {e}"),
    }
}

fn show(ctx: &Context, package_name: &str) {
    let mut presenter = |event| Presenter::display(&event);

    show_handler::run(ctx, package_name, &mut presenter);
}
