use crate::prelude::*;
use extension_traits::extension;

#[extension(pub trait DefIdExt)]
impl hax_frontend_exporter::DefId {
    fn into_string(&self) -> String {
        use core::ops::Deref;
        DefIdContentsExt::into_string(self.deref())
    }
}

#[extension(pub trait DefIdContentsExt)]
impl hax_frontend_exporter::DefIdContents {
    fn into_string(&self) -> String {
        std::iter::once(self.krate.to_string())
            .chain(self.path.iter().flat_map(|i| {
                use hax_frontend_exporter::DefPathItem;
                match &i.data {
                    DefPathItem::TypeNs(s) | DefPathItem::ValueNs(s) => Some(s.to_string()),
                    _ => None,
                }
            }))
            .join("::")
    }
}

#[extension(pub trait StrExt)]
impl<'a> &'a str {
    /// `line` is not 0-based but 1-based: the first line of a file is denoted `1`
    fn split_at_line_col(self, line: usize, col: usize) -> (String, String) {
        let lines: Vec<_> = self.lines().collect();
        assert!(!lines.is_empty());
        assert!(line >= 1);
        let [first_lines @ .., middle_line] = &lines[..line] else {
            panic!()
        };
        let first_lines = first_lines.join("\n");
        let last_lines = if lines.len() > line {
            lines[line + 1..].join("\n")
        } else {
            "".to_string()
        };
        let middle_line_l: String = middle_line.chars().take(col).collect();
        let middle_line_r: String = middle_line.chars().skip(col).collect();
        (
            format!("{first_lines}{middle_line_l}"),
            format!("{middle_line_r}{last_lines}"),
        )
    }
    fn split_at_loc(self, loc: hax_frontend_exporter::Loc) -> (String, String) {
        self.split_at_line_col(loc.line, loc.col)
    }
}
