use crate::prelude::*;
use once_cell::sync::Lazy;
use std::fs;
use std::sync::Mutex;

#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq)]
pub(super) struct KrateId(usize);

impl KrateId {
    pub(super) fn name(&self) -> String {
        format!("crate_{}", self.0).to_string()
    }
}

#[derive(Debug)]
pub struct Workspace {
    pub(super) root: PathBuf,
    pub(super) crates: HashSet<KrateId>,
}

static WORKSPACE: Lazy<Mutex<Workspace>> = Lazy::new(|| {
    let root = PathBuf::from("/tmp/testify");
    std::fs::create_dir_all(&root).unwrap();
    let workspace = Workspace {
        root,
        crates: HashSet::new(),
    };
    workspace.collect_garbadge();
    workspace.write_workspace_manifest();
    Mutex::new(workspace)
});

pub(super) fn lock_workspace() -> std::sync::MutexGuard<'static, Workspace> {
    WORKSPACE
        .lock()
        .expect("lock_workspace: could not lock WORKSPACE")
}

impl Workspace {
    fn write_workspace_manifest(&self) {
        let contents = format!(
            r#"
[workspace]
resolver = "2"
members = {:#?}"#,
            self.crates
                .iter()
                .map(|krate| krate.name())
                .collect::<Vec<_>>()
        );
        fs::write(self.root.join("Cargo.toml"), contents).unwrap();
    }
    pub(super) fn crate_path(&self, krate_id: KrateId) -> PathBuf {
        self.root.join(&krate_id.name())
    }
    pub(super) fn write_crate_main(&self, krate_id: KrateId, source: &str) {
        fs::write(self.crate_path(krate_id).join("main.rs"), source).unwrap();
    }
    pub(super) fn write_crate_manifest(
        &self,
        krate_id: KrateId,
        dependencies: &HashMap<String, DependencySpec>,
    ) {
        let name = krate_id.name();
        let dependencies = dependencies_to_string(dependencies);
        fs::write(
            self.crate_path(krate_id).join("Cargo.toml"),
            format!(
                r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{name}"
path = "main.rs"

{dependencies}
[dependencies.marshalling]
path = "{}/marshalling"
"#,
                std::env!("CARGO_MANIFEST_DIR")
            ),
        )
        .unwrap();
    }
    pub(super) fn add_crate(&mut self) -> KrateId {
        let krate_id = (0..)
            .map(KrateId)
            .find(|id| !self.crates.contains(id))
            .unwrap();

        fs::create_dir_all(&self.crate_path(krate_id)).unwrap();

        self.crates.insert(krate_id);

        self.write_crate_manifest(krate_id, &HashMap::default());
        self.write_crate_main(krate_id, "fn main() {}");
        self.write_workspace_manifest();

        krate_id
    }
    pub(super) fn remove_crate(&mut self, krate_id: KrateId) {
        return;
        self.crates.remove(&krate_id);
        self.collect_garbadge();
    }
    fn collect_garbadge(&self) {
        return;
        let mut crates: HashSet<_> = self.crates.iter().map(|kid| kid.name()).collect();
        crates.insert("target".into());
        for dir in std::fs::read_dir(&self.root)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .filter(|path| {
                !crates.contains(
                    path.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("<never>"),
                )
            })
        {
            fs::remove_dir_all(dir).unwrap();
        }
    }
}
