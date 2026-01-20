pub type NodeId = usize;

#[derive(Debug, Clone)]
pub enum NodeType {
    Element(ElementData),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
    pub node_type: NodeType,
}

#[derive(Debug)]
pub struct Dom {
    pub nodes: Vec<Node>,
}

impl Dom {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn create_element(&mut self, tag_name: &str, attrs: Vec<(String, String)>, parent: Option<NodeId>) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node {
            children: vec![],
            parent,
            node_type: NodeType::Element(ElementData {
                tag_name: tag_name.to_string(),
                attributes: attrs,
            }),
        });
        if let Some(pid) = parent {
            self.nodes[pid].children.push(id);
        }
        id
    }

    pub fn create_text(&mut self, text: &str, parent: Option<NodeId>) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node {
            children: vec![],
            parent,
            node_type: NodeType::Text(text.to_string()),
        });
        if let Some(pid) = parent {
            self.nodes[pid].children.push(id);
        }
        id
    }

    pub fn root(&self) -> NodeId {
        0
    }

    pub fn pretty_print(&self, id: NodeId, indent: usize) {
        let node = &self.nodes[id];
        println!(
            "{}{:?}",
            "  ".repeat(indent),
            node.node_type
        );
        for &child in &node.children {
            self.pretty_print(child, indent + 1);
        }
    }
}
