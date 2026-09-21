use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TraceStatus {
    Pass,
    Fail,
    Skip,
    Info,
}

impl TraceStatus {
    pub fn glyph(self) -> char {
        match self {
            Self::Pass => '✓',
            Self::Fail => '✗',
            Self::Skip => '–',
            Self::Info => '·',
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceNode {
    pub kind: String,
    pub title: String,
    pub status: TraceStatus,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TraceNode>,
}

impl TraceNode {
    pub fn new(
        kind: impl Into<String>,
        title: impl Into<String>,
        status: TraceStatus,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind: kind.into(),
            title: title.into(),
            status,
            detail: detail.into(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, node: TraceNode) -> Self {
        self.children.push(node);
        self
    }

    pub fn children<I: IntoIterator<Item = TraceNode>>(mut self, nodes: I) -> Self {
        self.children.extend(nodes);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trace {
    pub nodes: Vec<TraceNode>,
}

impl Trace {
    pub fn push(&mut self, node: TraceNode) {
        self.nodes.push(node);
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        for (i, node) in self.nodes.iter().enumerate() {
            let last = i + 1 == self.nodes.len();
            render_node(&mut out, node, "", last);
        }
        out
    }
}

fn render_node(out: &mut String, node: &TraceNode, prefix: &str, last: bool) {
    let branch = if last { "└─" } else { "├─" };
    let status = node.status.glyph();
    if node.detail.is_empty() {
        out.push_str(&format!(
            "{prefix}{branch} {status} [{kind}] {title}\n",
            kind = node.kind,
            title = node.title
        ));
    } else {
        out.push_str(&format!(
            "{prefix}{branch} {status} [{kind}] {title} — {detail}\n",
            kind = node.kind,
            title = node.title,
            detail = node.detail
        ));
    }
    let child_prefix = format!("{prefix}{}", if last { "   " } else { "│  " });
    for (i, child) in node.children.iter().enumerate() {
        let child_last = i + 1 == node.children.len();
        render_node(out, child, &child_prefix, child_last);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_tree() {
        let mut t = Trace::default();
        t.push(
            TraceNode::new("prune", "28 live", TraceStatus::Pass, "guest=false").child(
                TraceNode::new("page", "audio.bump", TraceStatus::Pass, "sink"),
            ),
        );
        let s = t.render();
        assert!(s.contains("audio.bump"));
        assert!(s.contains("└─"));
    }
}
