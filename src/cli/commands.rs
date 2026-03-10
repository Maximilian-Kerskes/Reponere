use crate::{
    build::registry::registry_handler::Registry,
    cli::{
        args::{Arg, SubArgs},
        presenter::Presenter,
    },
    handlers::{
        install_handler::{self, InstallResult},
        list_handler::{self},
        show_handler,
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
        _ => todo!(),
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
