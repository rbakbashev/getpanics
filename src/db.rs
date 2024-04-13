use std::collections::HashMap;
use std::path::PathBuf;

use ra_ap_ide::RootDatabase;
use ra_ap_load_cargo as rl;
use ra_ap_paths::{AbsPathBuf, RelPath, Utf8Path};
use ra_ap_project_model as ra;

use crate::utils::MaybeError;

pub fn construct(dir: PathBuf) -> (Vec<ra::TargetData>, RootDatabase) {
    let path = AbsPathBuf::assert_utf8(dir);
    let manifest = ra::ProjectManifest::discover_single(&path).or_die("discover project manifest");
    let cargo_config = construct_cargo_config();
    let workspace =
        ra::ProjectWorkspace::load(manifest, &cargo_config, &progress_cb).or_die("load project");
    let targets = get_targets(&workspace);
    let extra_env = HashMap::default();
    let load_config = construct_load_config();
    let (db, _vfs, _proc_macro_server) =
        rl::load_workspace(workspace, &extra_env, &load_config).or_die("load workspace");

    (targets, db)
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

#[allow(clippy::needless_pass_by_value)] // it's a callback
fn progress_cb(_msg: String) {}
