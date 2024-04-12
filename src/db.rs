use std::collections::HashMap;
use std::path::PathBuf;

use ra_ap_ide_db::RootDatabase;
use ra_ap_load_cargo as rl;
use ra_ap_paths::AbsPathBuf;
use ra_ap_project_model as ra;

use crate::utils::MaybeError;

pub fn construct(dir: PathBuf) -> RootDatabase {
    let path = AbsPathBuf::assert_utf8(dir);
    let manifest = ra::ProjectManifest::discover_single(&path).or_die("discover project manifest");
    let cargo_config = construct_cargo_config();
    let workspace =
        ra::ProjectWorkspace::load(manifest, &cargo_config, &progress_cb).or_die("load project");
    let extra_env = HashMap::default();
    let load_config = construct_load_config();
    let (db, _vfs, _proc_macro) =
        rl::load_workspace(workspace, &extra_env, &load_config).or_die("load workspace");

    db
}

fn construct_cargo_config() -> ra::CargoConfig {
    ra::CargoConfig {
        sysroot: Some(ra::RustLibSource::Discover),
        ..Default::default()
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
fn progress_cb(_msg: String) {
    // println!("loading project: {msg}");
}
