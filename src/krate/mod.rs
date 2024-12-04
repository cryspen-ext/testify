use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

pub mod hax;
pub mod server;
mod workspace;

use workspace::lock_workspace;

#[derive(Debug)]
pub struct Krate {
    id: workspace::KrateId,
    dependencies: Vec<String>,
}

impl Krate {
    pub fn new() -> Self {
        let mut workspace = lock_workspace();
        Self {
            id: workspace.add_crate(),
            dependencies: vec![],
        }
    }

    pub fn workspace_path(&self) -> PathBuf {
        let workspace = lock_workspace();
        workspace.root.clone()
    }

    /// Create a crate by using the source of an existing crate
    pub fn duplicate_crate(
        path: &Path,
        extra_deps: impl Iterator<Item = (String, String)>,
    ) -> std::io::Result<Self> {
        use std::path::Path;
        use std::{fs, io};

        fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
            fs::create_dir_all(&dst)?;
            for entry in fs::read_dir(src)? {
                let entry = entry?;
                let ty = entry.file_type()?;
                if ty.is_dir() {
                    copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
                } else {
                    fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
                }
            }
            Ok(())
        }

        fn rename_crate(dir: &Path, name: &str) -> io::Result<()> {
            let manifest_path = dir.join("Cargo.toml");
            let manifest = fs::read_to_string(&manifest_path)?
                .lines()
                .map(|line| {
                    if line.starts_with("name = ") {
                        format!(r#"name = "{}""#, name)
                    } else {
                        line.to_string()
                    }
                })
                .join("\n");
            fs::write(&manifest_path, manifest)?;
            Ok(())
        }

        fn cargo_add(
            krate: &Krate,
            extra_deps: impl Iterator<Item = (String, String)>,
        ) -> io::Result<()> {
            let mut cmd = krate.command("cargo");
            cmd.arg("add");
            cmd.args(extra_deps.map(|(dep, version)| format!("{dep}@{version}")));
            cmd.output()?;
            Ok(())
        }

        let krate = Self::new();
        fs::remove_dir_all(krate.path())?;
        copy_dir_all(&path, krate.path())?;
        rename_crate(&krate.path(), &krate.name())?;
        cargo_add(&krate, extra_deps)?;
        Ok(krate)
    }

    pub fn path(&self) -> PathBuf {
        let workspace = lock_workspace();
        workspace.crate_path(self.id)
    }

    pub fn name(&self) -> String {
        self.id.name()
    }

    /// Constructs a command whose current directory is correctly setup
    pub fn command<S: AsRef<OsStr>>(&self, program: S) -> Command {
        let mut command = Command::new(program);
        command.current_dir(self.path());
        command
    }

    pub fn run(&self) -> std::process::Child {
        use std::process::Stdio;

        std::process::Command::new("cargo")
            .arg("run")
            .arg("--quiet")
            .arg("--release")
            .current_dir(self.path())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            // .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Couldn't run `cargo build`")
    }

    pub fn build(&self) -> Result<PathBuf, String> {
        use std::process::Stdio;

        let output = std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            // .env("RUSTFLAGS", "-Z threads=4")
            .current_dir(self.path())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
            .expect("Couldn't run `cargo build`");

        if output.status.success() {
            Ok(self
                .workspace_path()
                .join("target")
                .join("release")
                // .join("debug")
                .join(self.name()))
        } else {
            let stderr = std::str::from_utf8(&output.stderr).unwrap();
            Err(stderr.to_string())
        }
    }

    pub fn hax(
        &self,
    ) -> Result<Vec<hax_frontend_exporter::Item<hax_frontend_exporter::ThirBody>>, String> {
        use std::process::Stdio;

        let output = std::process::Command::new("cargo")
            .args(["hax", "json", "-o", "-"])
            .current_dir(self.path())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
            .expect("Couldn't run `cargo run`");

        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let stderr = std::str::from_utf8(&output.stderr).unwrap();

        if output.status.success() {
            let items: Vec<hax_frontend_exporter::Item<hax_frontend_exporter::ThirBody>> =
                serde_json::from_str(&stdout).map_err(|e| {
                    format!("Error parsing hax output: stdout:\n{stdout}\n\nstderr:{stderr}\n\nerror:{e:?}")
                })?;
            Ok(items)
        } else {
            Err(stderr.to_string())
        }
    }

    pub fn use_serde(&mut self) {
        self.add_dependency(r#"serde_json = "1""#);
        self.add_dependency(r#"serde = { version = "1.0", features = ["derive"] }"#);
    }
    pub fn add_dependency(&mut self, deps: &str) {
        let workspace = lock_workspace();
        self.dependencies.push(deps.into());
        workspace.write_crate_manifest(self.id, &self.dependencies.join("\n"))
    }
    pub fn source(&self, source: &str) {
        let source = prettyplease::unparse(&syn::parse_str(source).expect(source));
        let workspace = lock_workspace();
        workspace.write_crate_main(self.id, &source)
    }
}

impl Drop for Krate {
    fn drop(&mut self) {
        let mut workspace = lock_workspace();
        workspace.remove_crate(self.id);
    }
}

/// Runs a function `f` that takes a collection of items of type
/// `T`. The function `f` might fail, but we don't know because of
/// which item among all items `items`. `run_or_catch_error(items, f)`
/// will call `f` on `items`: if this fails, then it will repeatedly
/// call `f` to spot the first item which is yielding an error.
pub fn run_or_locate_error<'a, Item: std::fmt::Debug, Output, Error: Clone>(
    items: &'a [Item],
    mut f: impl FnMut(&'a [Item]) -> Result<Output, Error>,
) -> Result<Output, (Vec<&'a Item>, Error)> {
    assert!(!items.is_empty());
    use std::collections::VecDeque;
    let mut queue: VecDeque<_> = vec![items].into();
    let mut error: Option<(Vec<&'a Item>, Error)> = None;
    while let Some(items) = queue.pop_front() {
        match f(items) {
            Ok(value) => {
                if error.is_none() {
                    return Ok(value);
                }
            }
            Err(err) => {
                let err = (items.iter().collect(), err.clone());
                match items {
                    [] => unreachable!(),
                    [item] => {
                        return Err(err.clone());
                    }
                    _ => {
                        let (left, right) = items.split_at(items.len() / 2);
                        queue.push_front(left);
                        queue.push_front(right);
                    }
                }
                error = Some(err);
            }
        }
        eprintln!("Continue...")
    }
    Err(error.unwrap())
}
