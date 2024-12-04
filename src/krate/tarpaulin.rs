use crate::krate::Krate;
use crate::prelude::*;
use hax_frontend_exporter::Loc;
use hax_frontend_exporter::Span;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stats {
    #[serde(rename = "Line")]
    pub line: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trace {
    pub line: usize,
    pub stats: Stats,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileReport {
    pub path: Vec<String>,
    pub traces: Vec<Trace>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TarpaulinReport {
    pub files: Vec<FileReport>,
}

#[derive(Copy, Clone, Debug)]
pub struct LineReport {
    pub line: usize,
    pub covered: bool,
}

impl From<Trace> for LineReport {
    fn from(trace: Trace) -> Self {
        Self {
            line: trace.line,
            covered: trace.stats.line > 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BadCoverageReport {
    pub relative_path: PathBuf,
    pub item_path: String,
    pub lines: Vec<(usize, String, Option<bool>)>,
}

impl TarpaulinReport {
    pub fn lines_for_file(&self, file: &Path) -> Vec<LineReport> {
        self.files
            .iter()
            .flat_map(|file_report| {
                file_report.traces.iter().map(move |r| {
                    (
                        file_report.path.iter().collect::<PathBuf>(),
                        LineReport::from(r.clone()),
                    )
                })
            })
            .filter(|(path, _)| path == &file)
            .map(|(_, line)| line)
            .collect()
    }
    pub fn coverage_for_span(
        &self,
        item_path: String,
        file: &Path,
        span: Span,
    ) -> Option<BadCoverageReport> {
        let within_range = |line| line >= span.lo.line && line <= span.hi.line;
        let line_reports = self.lines_for_file(&file);
        let lines_status: HashMap<_, _> = line_reports
            .iter()
            .filter(|lr| within_range(lr.line))
            .map(|lr| (lr.line, lr.covered))
            .collect();
        if lines_status.is_empty() {
            return None;
        }
        let contents = std::fs::read_to_string(&file).unwrap();
        let lines: Vec<_> = contents.lines().collect();
        let lines = (span.lo.line..=span.hi.line).map(|line| {
            (
                line,
                lines[line - 1].to_string(),
                lines_status.get(&line).copied(),
            )
        });
        Some(BadCoverageReport {
            lines: lines.collect(),
            item_path,
            relative_path: span.filename.to_path().unwrap_or(file).to_path_buf(),
        })
    }
}

impl fmt::Display for BadCoverageReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "\n{}",
            format!(
                " ╭ ⚠ Item `{}` was not entirely covered.\n │ ↳ You need more contracts or more tests.",
                format!("{}", self.item_path).reversed()
            )
            .bold()
        )?;
        let width = self
            .lines
            .iter()
            .map(|(n, _, _)| *n)
            .max()
            .unwrap_or(4)
            .to_string()
            .len();
        let filler = std::iter::repeat_n(" ", width).collect::<String>();
        for (i, (n, s, c)) in self.lines.iter().enumerate() {
            let n = format!("{filler}{n}");
            let n: String = format!("{filler}{n}")
                .chars()
                .skip(n.len() - width)
                .collect();
            let (c, s) = match c {
                None => (" ".into(), s.dimmed()),
                Some(true) => ("✓".green(), s.green()),
                Some(false) => ("⨯".red(), s.red()),
            };
            let box_char = if i + 1 == self.lines.len() {
                "╰"
            } else {
                "│"
            };
            writeln!(f, " {box_char} {} {c} {s}", n.dimmed())?;
        }
        Ok(())
    }
}

impl Krate {
    pub fn tarpaulin(&self) -> TarpaulinReport {
        let mut tarpaulin = self.command("cargo");
        tarpaulin.args(&["tarpaulin", "--out", "Json"]);
        tarpaulin.arg("--output-dir");
        tarpaulin.arg(self.path());
        tarpaulin.args(&["--", "testify_test"]);
        let output = tarpaulin.output().unwrap();
        let path = self.path().join("tarpaulin-report.json");
        use std::fs::File;
        use std::io::BufReader;
        let file = File::open(path).unwrap_or_else(|e| panic!("{e}: {output:#?}"));
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).unwrap()
    }
}
