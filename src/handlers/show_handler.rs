use crate::{handlers::events::ShowEvent, util::context::Context};

pub fn run<F: FnMut(ShowEvent)>(ctx: &Context, package_name: &str, progress: &mut F) {
    progress(ShowEvent::LookingUpPackage {
        name: package_name.to_string(),
    });

    let pkg_entry = match ctx.registry.get_package(package_name) {
        Some(entry) => entry,
        None => {
            progress(ShowEvent::PackageNotFound {
                name: package_name.to_string(),
            });
            return;
        }
    };

    progress(ShowEvent::ShowingPackage {
        name: package_name.to_string(),
        latest: pkg_entry.latest.clone(),
    });

    let versions: Vec<String> = pkg_entry.releases.keys().cloned().collect();

    progress(ShowEvent::AvailableVersions {
        versions: versions.clone(),
    });

    if let Some(installed_pkg) = ctx.tracker.get_package(package_name) {
        progress(ShowEvent::InstalledVersion {
            version: installed_pkg.version.clone(),
        });

        progress(ShowEvent::InstalledPath {
            path: installed_pkg.install_path.to_string(),
        });

        if installed_pkg.version != pkg_entry.latest {
            progress(ShowEvent::StatusOutdated {
                latest: pkg_entry.latest.clone(),
            });
        } else {
            progress(ShowEvent::StatusUpToDate);
        }
    } else {
        progress(ShowEvent::StatusNotInstalled);
    }

    if let Some(latest_release) = pkg_entry.releases.get(&pkg_entry.latest) {
        progress(ShowEvent::BuildFile {
            path: latest_release.build_file().to_string(),
        });
    }

    progress(ShowEvent::Finished);
}
