use std::collections::{BTreeMap, HashSet};

use ra_ap_ide as ri;
use ra_ap_syntax::ast::AstNode;
use ra_ap_syntax::{NodeOrToken, SyntaxKind, SyntaxNode, WalkEvent};
use ra_ap_vfs as vfs;

use crate::db::State;
use crate::die;
use crate::utils::MaybeError;

pub struct Graph {
    pub name: String,
    pub adj_list: Vec<HashSet<usize>>,
    pub indices: BTreeMap<String, usize>,
}

pub fn construct(state: State) -> Graph {
    let crate_name = get_crate_name(&state);
    let root_file_id = get_root_file_id(&state);
    let host = ri::AnalysisHost::with_database(state.db);
    let analysis = host.analysis();
    let source_file = analysis.parse(root_file_id).or_die("parse root file");
    let syntax_tree = source_file.syntax();

    let mut graph = Graph::new(crate_name);
    let mut seen = HashSet::new();

    for toplevel in syntax_tree.children() {
        if toplevel.kind() != SyntaxKind::FN {
            continue;
        }

        println!("processing function {}...", get_fn_name(&toplevel));

        process_fn_children(&mut graph, &mut seen, &analysis, root_file_id, &toplevel);
    }

    graph
}

fn get_crate_name(state: &State) -> String {
    state.target.name.clone()
}

fn get_root_file_id(state: &State) -> vfs::FileId {
    let path = vfs::VfsPath::from(state.target.root.clone());

    state
        .vfs
        .file_id(&path)
        .or_die("get ID corresponding to root file")
}

fn get_fn_name(fn_node: &SyntaxNode) -> String {
    // println!("fn_node={}", fn_node);
    // println!("fn_node={:#?}", fn_node);

    find_child(fn_node, SyntaxKind::NAME)
        .or_die("find NAME")
        .text()
        .to_string()
}

fn find_child(node: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxNode> {
    node.children().find(|child| child.kind() == kind)
}

impl Graph {
    fn new(name: String) -> Self {
        Self {
            name,
            adj_list: Vec::new(),
            indices: BTreeMap::new(),
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

    fn _insert_function_calls(&mut self, fn_node: &SyntaxNode, idx: usize) {
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

fn process_fn_children(
    graph: &mut Graph,
    seen: &mut HashSet<String>,
    an: &ri::Analysis,
    file_id: ri::FileId,
    fn_node: &SyntaxNode,
) {
    let fn_name = get_fn_name(fn_node);

    if !seen.insert(fn_name.clone()) {
        return;
    }

    let src_idx = graph.get_or_insert(fn_name);

    let Some(block) = find_child(fn_node, SyntaxKind::BLOCK_EXPR) else {
        println!("...can't find BLOCK_EXPR child");
        return;
    };

    for event in block.preorder() {
        let WalkEvent::Enter(e) = event else {
            continue;
        };

        if e.kind() != SyntaxKind::CALL_EXPR {
            continue;
        }

        let Some(path_expr) = find_child(&e, SyntaxKind::PATH_EXPR) else {
            println!("...can't find PATH_EXPR child, likely closure"); // TODO
            continue;
        };

        let full_path = path_expr.text().to_string();

        graph.connect(src_idx, full_path);

        process_recursively(graph, seen, an, file_id, &path_expr);
    }
}

fn process_recursively(
    graph: &mut Graph,
    seen: &mut HashSet<String>,
    an: &ri::Analysis,
    file_id: ri::FileId,
    f: &SyntaxNode,
) {
    let fn_name = f.text();

    println!("processing function {fn_name}...");

    let offset = f.text_range().end();
    let position = ri::FilePosition { file_id, offset };

    let cancellable = an.goto_definition(position);

    let Ok(definitions_opt) = cancellable else {
        die!("Unexpected cancellation of gotoDefinition");
    };

    let definitions = definitions_opt.or_die("go to definition");
    let targets = definitions.info;

    let Some(target) = targets.first() else {
        println!("...no definitions found");
        return;
    };

    let source_file = an.parse(target.file_id).or_die("parse file");
    let syntax_tree = source_file.syntax();

    let fn_tree = syntax_tree.covering_element(target.full_range);

    let NodeOrToken::Node(fn_node) = fn_tree else {
        println!("...returned token");
        return;
    };

    match fn_node.kind() {
        SyntaxKind::VARIANT => println!("...it's an enum variant"),
        SyntaxKind::STRUCT => println!("...it's a newtype struct"),
        SyntaxKind::TOKEN_TREE => println!("...it's complicated"),
        SyntaxKind::IMPL => println!("TODO"),
        SyntaxKind::FN => process_fn_children(graph, seen, an, target.file_id, &fn_node),
        _ => {
            println!("fn_node={}", fn_node);
            println!("fn_node={:#?}", fn_node);
            die!("unexpected fn_node kind {:#?}", fn_node.kind());
        }
    }
}
