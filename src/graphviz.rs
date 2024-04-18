use crate::graph::Graph;
use crate::utils::MaybeError;

pub fn output(graph: Graph) {
    let filename = construct_filename(&graph);
    let text = format_to_graphviz(graph);

    std::fs::write(filename, text).or_die("write to file");
}

fn construct_filename(graph: &Graph) -> String {
    format!("{}.dot", graph.name)
}

fn format_to_graphviz(graph: Graph) -> String {
    let mut t = String::new();

    t.push_str("digraph g {\n");

    for (fn_name, idx) in graph.indices {
        t.push_str(&format!("{idx} [label = \"{fn_name}\"]\n"));
    }

    for (src, set) in graph.adj_list.iter().enumerate() {
        for dst in set {
            t.push_str(&format!("{src} -> {dst}\n"));
        }
    }

    t.push_str("}\n");

    t
}
