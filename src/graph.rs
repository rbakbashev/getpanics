use std::collections::{HashMap, HashSet};

use ra_ap_ide as ri;
use ra_ap_syntax::ast::AstNode;
use ra_ap_syntax::{SyntaxKind, SyntaxNode, WalkEvent};
use ra_ap_vfs as vfs;

use crate::db::State;
use crate::die;
use crate::utils::MaybeError;

pub struct Graph {
    adj_list: Vec<HashSet<usize>>,
    indices: HashMap<String, usize>,
}

pub fn construct(state: State) -> Graph {
    let root_file_id = get_root_file_id(&state);
    let host = ri::AnalysisHost::with_database(state.db);
    let analysis = host.analysis();
    let source_file = analysis.parse(root_file_id).or_die("parse root file");
    let syntax_tree = source_file.syntax();

    let mut graph = Graph::new();

    for toplevel in syntax_tree.children() {
        if toplevel.kind() != SyntaxKind::FN {
            continue;
        }

        let name = get_fn_name(&toplevel);
        let idx = graph.get_or_insert(name);

        graph.insert_function_calls(&toplevel, idx);
    }

    graph
}

fn get_root_file_id(state: &State) -> vfs::FileId {
    let path = vfs::VfsPath::from(state.target.root.clone());

    state
        .vfs
        .file_id(&path)
        .or_die("get ID corresponding to root file")
}

fn get_fn_name(fn_node: &SyntaxNode) -> String {
    find_child(fn_node, SyntaxKind::NAME)
        .or_die("find NAME")
        .text()
        .to_string()
}

fn find_child(node: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxNode> {
    node.children().find(|child| child.kind() == kind)
}

impl Graph {
    fn new() -> Self {
        Self {
            adj_list: Vec::new(),
            indices: HashMap::new(),
        }
    }

    fn get_or_insert(&mut self, name: String) -> usize {
        match self.indices.get(&name) {
            Some(idx) => *idx,
            None => self.insert(name),
        }
    }

    fn insert(&mut self, name: String) -> usize {
        let index = self.adj_list.len();

        self.adj_list.push(HashSet::new());
        self.indices.insert(name, index);

        index
    }

    fn insert_function_calls(&mut self, fn_node: &SyntaxNode, idx: usize) {
        let block = find_child(fn_node, SyntaxKind::BLOCK_EXPR).or_die("find BLOCK_EXPR");

        for event in block.preorder() {
            let WalkEvent::Enter(e) = event else {
                continue;
            };

            if e.kind() != SyntaxKind::CALL_EXPR {
                continue;
            }

            let path_expr = e.first_child().or_die("get CALL_EXPR's child");

            if path_expr.kind() != SyntaxKind::PATH_EXPR {
                die!("unexpected PATH_EXPR kind: {:?}", path_expr.kind());
            }

            let full_path = path_expr.text().to_string();

            self.connect(idx, full_path);
        }
    }

    fn connect(&mut self, src: usize, dst: String) {
        let dst_idx = self.get_or_insert(dst);

        self.adj_list[src].insert(dst_idx);
    }
}
