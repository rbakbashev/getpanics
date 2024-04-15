use std::collections::HashMap;
use std::path::Path;

use ra_ap_ide::RootDatabase;
use ra_ap_load_cargo as rl;
use ra_ap_paths::{AbsPathBuf, RelPath, Utf8Path};
use ra_ap_project_model as ra;

use crate::utils::MaybeError;

pub fn construct(dir: &Path) -> (Vec<ra::TargetData>, RootDatabase) {
    let path = AbsPathBuf::assert_utf8(dir.to_path_buf());
    let manifest = ra::ProjectManifest::discover_single(&path).or_die("discover project manifest");
    let workspace = load_manifest(manifest);
    let targets = get_targets(&workspace);
    let extra_env = HashMap::default();
    let load_config = construct_load_config();
    let (db, _vfs, _proc_macro_server) =
        rl::load_workspace(workspace, &extra_env, &load_config).or_die("load workspace");

    (targets, db)
}

fn load_manifest(manifest: ra::ProjectManifest) -> ra::ProjectWorkspace {
    let cargo_config = construct_cargo_config();
    let progress_cb = |_msg: String| {};

    ra::ProjectWorkspace::load(manifest, &cargo_config, &progress_cb).or_die("load project")
}

fn construct_cargo_config() -> ra::CargoConfig {
    ra::CargoConfig {
        sysroot: Some(ra::RustLibSource::Discover),
        ..Default::default()
    }
}

fn get_targets(workspace: &ra::ProjectWorkspace) -> Vec<ra::TargetData> {
    match workspace {
        ra::ProjectWorkspace::Cargo { cargo, .. } => {
            let mut targets = vec![];

            for package_idx in cargo.packages() {
                let package = &cargo[package_idx];

                if !package.is_local {
                    continue;
                }

                for target_idx in &package.targets {
                    targets.push(cargo[*target_idx].clone());
                }
            }

            targets
        }
        ra::ProjectWorkspace::Json { project, .. } => {
            let _root_path = project.path();
            todo!()
        }
        ra::ProjectWorkspace::DetachedFiles { files, .. } => {
            let lib_suffix = RelPath::new_unchecked(Utf8Path::new("lib.rs"));
            let bin_suffix = RelPath::new_unchecked(Utf8Path::new("main.rs"));
            let _root_paths = files
                .iter()
                .filter(|p| p.ends_with(lib_suffix) || p.ends_with(bin_suffix));
            todo!()
        }
    }
}

fn construct_load_config() -> rl::LoadCargoConfig {
    rl::LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: rl::ProcMacroServerChoice::None,
        prefill_caches: false,
    }
}
