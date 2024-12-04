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

        // println!("Building...");
        // self.build().unwrap();
        // println!("Building... done!");

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
        eprintln!("running f for {} item", items.len());
        match f(items) {
            Ok(value) => {
                if error.is_none() {
                    return Ok(value);
                }
                eprintln!("Ok!")
            }
            Err(err) => {
                let err = (items.iter().collect(), err.clone());
                match items {
                    [] => unreachable!(),
                    [item] => {
                        eprintln!("Err, only one item!");
                        return Err(err.clone());
                    }
                    _ => {
                        let (left, right) = items.split_at(items.len() / 2);
                        eprintln!(
                            "Err, pushing more items: left={}, right={}",
                            left.len(),
                            right.len()
                        );
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
