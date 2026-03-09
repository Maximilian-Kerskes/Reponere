use crate::{
    build::{
        build_step_handler::build_handler::BuildHandler,
        dependency_handler::{
            build_dependency_guard::BuildDependencyGuard, dependency_handler::DependencyHandler,
        },
        package::{
            package::{InstalledPackage, Package},
            parse::PackageParser,
        },
        package_manager::manager::PackageManager,
        package_tracker::package_tracker::PackageTracker,
        registry::registry_handler::{Registry, Release},
        source::source_handler::{GitSource, GitSourceHandler},
    },
    handlers::events::InstallEvent,
};
use tempfile::TempDir;
use thiserror::Error;

pub enum InstallResult {
    Installed,
    AlreadyInstalled,
}

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("release not found: {0}")]
    ReleaseNotFound(String),
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("package manager error: {0}")]
    PackageManagerError(String),
    #[error("Runtime dependency errors: {0}")]
    RuntimeDependencyError(String),
    #[error("Buildtime dependency errors: {0}")]
    BuildtimeDependencyError(String),
    #[error("source error: {0}")]
    SourceFetchError(String),
    #[error("build error: {0}")]
    BuildError(String),
}

pub fn run<F: FnMut(InstallEvent)>(
    reg: &Registry,
    tracker: &mut PackageTracker,
    package: &str,
    force: bool,
    progress: &mut F,
) -> Result<InstallResult, InstallError> {
    if !force && check_already_installed(package, tracker) {
        return Ok(InstallResult::AlreadyInstalled);
    }
    let release = resolve_release(package, reg)?;
    let parsed = parse_package(release)?;

    progress(InstallEvent::InstallingDependencies);
    let installed = install_dependencies(&parsed, progress)?;

    let _build_dependency_guard = BuildDependencyGuard {
        package_manager: &PackageManager::get_package_manager(true)
            .map_err(|e| InstallError::PackageManagerError(e.to_string()))?,
        installed,
    };

    progress(InstallEvent::FetchingSource);
    let source_dir = fetch_source(&parsed)?;

    progress(InstallEvent::BuildingSource);
    optional_build(&parsed, &source_dir, progress)?;

    progress(InstallEvent::Cleanup);
    track_installation(&parsed, tracker)?;
    drop(_build_dependency_guard);

    progress(InstallEvent::Finished);
    Ok(InstallResult::Installed)
}

fn check_already_installed(package: &str, tracker: &PackageTracker) -> bool {
    tracker.get_package(package).is_some()
}

fn resolve_release<'a>(package: &str, reg: &'a Registry) -> Result<&'a Release, InstallError> {
    if let Some(package) = package.split_once('@') {
        reg.resolve_release(package.0, Some(package.1))
            .ok_or(InstallError::ReleaseNotFound(package.0.to_string()))
    } else {
        reg.resolve_release(package, None)
            .ok_or(InstallError::ReleaseNotFound(package.to_string()))
    }
}

fn parse_package(release: &Release) -> Result<Package, InstallError> {
    PackageParser::new(release.build_file())
        .parse()
        .map_err(|e| InstallError::ParseError(e.to_string()))
}

fn install_dependencies<F: FnMut(InstallEvent)>(
    parsed: &Package,
    progress: &mut F,
) -> Result<Vec<String>, InstallError> {
    let package_manager = PackageManager::get_package_manager(true)
        .map_err(|e| InstallError::PackageManagerError(e.to_string()))?;

    let dependency_handler = DependencyHandler::new(&package_manager, parsed.dependencies.clone());

    progress(InstallEvent::InstallingRunTimeDependencies {
        dependencies: parsed
            .dependencies
            .runtime
            .iter()
            .map(|d| d.name.clone())
            .collect(),
    });

    let mut runtime_errors = Vec::new();
    dependency_handler.install_runtime_dependencies(&mut runtime_errors, progress);

    if !runtime_errors.is_empty() {
        return Err(InstallError::RuntimeDependencyError(
            runtime_errors
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }

    progress(InstallEvent::InstallingBuildDependencies {
        dependencies: parsed
            .dependencies
            .build
            .iter()
            .map(|d| d.name.clone())
            .collect(),
    });
    let mut build_errors = Vec::new();
    let installed = dependency_handler.install_build_dependencies(&mut build_errors, progress);

    if !build_errors.is_empty() {
        return Err(InstallError::BuildtimeDependencyError(
            build_errors
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    Ok(installed)
}

fn fetch_source(parsed: &Package) -> Result<TempDir, InstallError> {
    let source = GitSource::from_source(&parsed.source)
        .map_err(|e| InstallError::SourceFetchError(e.to_string()))?;

    GitSourceHandler::new(source)
        .fetch()
        .map_err(|e| InstallError::SourceFetchError(e.to_string()))
}

fn optional_build<F: FnMut(InstallEvent)>(
    parsed: &Package,
    source_dir: &TempDir,
    progress: &mut F,
) -> Result<(), InstallError> {
    if let Some(build) = &parsed.build {
        BuildHandler::new(build.clone())
            .run_build_steps(source_dir.path(), progress)
            .map_err(|e| InstallError::BuildError(e.to_string()))?;
    }

    Ok(())
}

fn track_installation(parsed: &Package, tracker: &mut PackageTracker) -> Result<(), InstallError> {
    let installed = InstalledPackage {
        name: parsed.name.clone(),
        version: parsed.version.clone(),
        install_path: parsed.install_path.clone().unwrap(),
        dependencies: parsed.dependencies.runtime.clone(),
    };

    tracker.add_package(installed);
    Ok(())
}
