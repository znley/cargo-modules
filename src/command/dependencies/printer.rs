// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Printer for displaying crate as a graoh.

use std::fmt::{self, Write};

use petgraph::{
    graph::NodeIndex,
    visit::{IntoNodeReferences, NodeRef},
};
use ra_ap_hir as hir;
use ra_ap_ide::RootDatabase;

use crate::{
    analyzer,
    graph::{Edge, EdgeKind, Graph, Node},
    item::ItemVisibility,
};

use super::{
    options::Options,
    theme::{edge_styles, node_styles},
};

const INDENTATION: &str = "    ";

pub struct Printer<'a> {
    options: &'a Options,
    member_krate: hir::Crate,
    db: &'a RootDatabase,
}

impl<'a> Printer<'a> {
    pub fn new(options: &'a Options, member_krate: hir::Crate, db: &'a RootDatabase) -> Self {
        Self {
            options,
            member_krate,
            db,
        }
    }

    pub fn fmt(
        &self,
        f: &mut dyn fmt::Write,
        graph: &Graph,
        start_node_idx: NodeIndex,
    ) -> Result<(), anyhow::Error> {
        let root_node = &graph[start_node_idx];
        let label = root_node.item.display_path(self.db);
        let layout = self.options.layout.to_string();
        let i = INDENTATION;

        writeln!(f, "digraph {{")?;

        writeln!(f)?;

        indoc::writedoc!(
            f,
            r#"
            {i}graph [
            {i}    label={label:?},
            {i}    labelloc=t,

            {i}    pad=0.4,

            {i}    // Consider rendering the graph using a different layout algorithm, such as:
            {i}    // [dot, neato, twopi, circo, fdp, sfdp]
            {i}    layout={layout},
            {i}    overlap=false,
            {i}    splines="line",
            {i}    rankdir=LR,
            
            {i}    fontname="Helvetica", 
            {i}    fontsize="36",
            {i}];
            "#,
        )?;

        writeln!(f)?;

        indoc::writedoc!(
            f,
            r#"
            {i}node [
            {i}    fontname="monospace",
            {i}    fontsize="10",
            {i}    shape="record",
            {i}    style="filled",
            {i}];
            "#,
        )?;

        writeln!(f)?;

        indoc::writedoc!(
            f,
            r#"
            {i}edge [
            {i}    fontname="monospace",
            {i}    fontsize="10",
            {i}];
            "#,
        )?;

        writeln!(f)?;

        self.fmt_nodes(f, graph)?;

        writeln!(f)?;

        self.fmt_edges(f, graph)?;

        writeln!(f)?;

        writeln!(f, "}}")?;

        Ok(())
    }

    fn fmt_nodes(&self, f: &mut dyn fmt::Write, graph: &Graph) -> fmt::Result {
        let mut lines: Vec<_> = graph
            .node_references()
            .map(|node_ref| {
                let node: &Node = node_ref.weight();

                let id = node.item.display_path(self.db);
                let kind = node.item.kind_display_name(self.db);

                let label = self.node_label(node).unwrap();
                let attributes = self.node_attributes(node);

                let i = INDENTATION;

                format!(r#"{i}{id:?} [label={label:?}{attributes}]; // {kind:?} node"#)
            })
            .collect();

        lines.sort();

        for line in lines {
            f.write_str(&line)?;
            f.write_char('\n')?;
        }

        Ok(())
    }

    fn fmt_edges(&self, f: &mut dyn fmt::Write, graph: &Graph) -> fmt::Result {
        let mut lines: Vec<_> = graph.edge_indices().map(|edge_idx| {
            let edge = &graph[edge_idx];
            let (source_idx, target_idx) = graph.edge_endpoints(edge_idx).unwrap();

            let source = graph[source_idx].item.display_path(self.db);
            let target = graph[target_idx].item.display_path(self.db);

            let kind = edge.kind.display_name();

            let label = self.edge_label(edge);
            let attributes = self.edge_attributes(edge);

            let constraint = match edge.kind {
                EdgeKind::Uses => "[constraint=false]",
                EdgeKind::Owns => "[constraint=true]",
            };

            let i = INDENTATION;

            format!(r#"{i}{source:?} -> {target:?} [label={label:?}{attributes}] {constraint}; // {kind:?} edge"#)
        }).collect();

        lines.sort();

        for line in lines {
            f.write_str(&line)?;
            f.write_char('\n')?;
        }

        Ok(())
    }

    fn node_label(&self, node: &Node) -> Result<String, fmt::Error> {
        let mut label = String::new();

        self.fmt_node_header(&mut label, node)?;
        write!(&mut label, "|")?;
        self.fmt_node_body(&mut label, node)?;

        Ok(label)
    }

    fn fmt_node_header(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let krate = analyzer::krate(node.item.hir, self.db);

        let is_external = krate != Some(self.member_krate);
        let is_crate = analyzer::moduledef_is_crate(node.item.hir, self.db);

        let visibility = if is_external {
            Some("external".to_owned())
        } else if is_crate {
            None
        } else {
            Some(format!("{}", node.item.visibility(self.db)))
        };

        let kind = node.item.kind_display_name(self.db);

        if let Some(visibility) = visibility {
            write!(f, "{visibility} ")?;
        }

        write!(f, "{kind}")
    }

    fn fmt_node_body(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let path = node.item.display_path(self.db);

        let refined_path = if self.options.selection.no_externs {
            // Try to drop the crate-name from the path if externs are being filtered:
            if let Some((_crate_name, relative_path)) = path.split_once("::") {
                relative_path.to_owned()
            } else {
                path
            }
        } else {
            path
        };

        write!(f, "{refined_path}")
    }

    fn node_attributes(&self, node: &Node) -> String {
        let styles = node_styles();

        let is_crate = analyzer::moduledef_is_crate(node.item.hir, self.db);

        let style = if is_crate {
            styles.krate
        } else {
            match &node.item.visibility(self.db) {
                ItemVisibility::Crate => styles.visibility.pub_crate,
                ItemVisibility::Module(_) => styles.visibility.pub_module,
                ItemVisibility::Private => styles.visibility.pub_private,
                ItemVisibility::Public => styles.visibility.pub_global,
                ItemVisibility::Super => styles.visibility.pub_super,
            }
        };

        format!(r#", fillcolor="{}""#, style.fill_color)
    }

    fn edge_label(&self, edge: &Edge) -> String {
        edge.kind.display_name().to_owned()
    }

    fn edge_attributes(&self, edge: &Edge) -> String {
        let styles = edge_styles();

        let style = match edge.kind {
            EdgeKind::Uses { .. } => styles.uses,
            EdgeKind::Owns => styles.owns,
        };

        format!(r#", color="{}", style="{}""#, style.color, style.stroke)
    }
}
