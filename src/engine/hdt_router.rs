use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;

const NUM_NODES: usize = 6;
const MAX_HOPS: u8 = 3;
const PRIMARY_RING: [(usize, usize); 6] = [(0,1),(1,2),(2,3),(3,4),(4,5),(5,0)];
const TOROIDAL_BRIDGES: [(usize, usize); 12] = [
    (0,2),(0,3),(0,4),
    (1,3),(1,4),(1,5),
    (2,4),(2,5),(2,0),
    (3,5),(3,0),(3,1),
];

#[derive(Debug, Clone)]
pub struct HdtRoute {
    pub source: usize,
    pub target: usize,
    pub hops: u8,
    pub path: Vec<usize>,
    pub latency_us: u64,
    pub last_updated: Instant,
}

pub struct HdtRouter {
    pub nodes: Vec<HdtNode>,
    pub routes: HashMap<(usize, usize), HdtRoute>,
    pub active_nodes: usize,
}

#[derive(Debug, Clone)]
pub struct HdtNode {
    pub id: usize,
    pub name: String,
    pub neighbors: Vec<usize>,
    pub active: bool,
    pub load: f32,
}

impl HdtRouter {
    pub fn new() -> Self {
        let mut nodes = Vec::with_capacity(NUM_NODES);
        let names = ["umbra-core", "trader", "analyst", "voice", "monitor", "security"];

        for i in 0..NUM_NODES {
            let mut neighbors = Vec::new();
            for &(a, b) in &PRIMARY_RING {
                if a == i { neighbors.push(b); }
                if b == i { neighbors.push(a); }
            }
            for &(a, b) in &TOROIDAL_BRIDGES {
                if a == i && !neighbors.contains(&b) { neighbors.push(b); }
                if b == i && !neighbors.contains(&a) { neighbors.push(a); }
            }
            neighbors.sort();
            neighbors.dedup();

            nodes.push(HdtNode {
                id: i,
                name: names[i].into(),
                neighbors,
                active: true,
                load: 0.0,
            });
        }

        let mut router = Self {
            nodes,
            routes: HashMap::new(),
            active_nodes: NUM_NODES,
        };
        router.precompute_routes();
        router
    }

    fn precompute_routes(&mut self) {
        for source in 0..NUM_NODES {
            for target in 0..NUM_NODES {
                if source == target { continue; }
                if let Some((hops, path)) = self.shortest_path(source, target) {
                    self.routes.insert((source, target), HdtRoute {
                        source,
                        target,
                        hops,
                        path,
                        latency_us: 0,
                        last_updated: Instant::now(),
                    });
                }
            }
        }
    }

    fn shortest_path(&self, source: usize, target: usize) -> Option<(u8, Vec<usize>)> {
        use std::collections::VecDeque;
        let mut visited = vec![false; NUM_NODES];
        let mut parent = vec![None; NUM_NODES];
        let mut dist = vec![0u8; NUM_NODES];
        let mut queue = VecDeque::new();

        visited[source] = true;
        queue.push_back(source);

        while let Some(current) = queue.pop_front() {
            if current == target {
                let mut path = Vec::new();
                let mut node = Some(target);
                while let Some(n) = node {
                    path.push(n);
                    node = parent[n];
                }
                path.reverse();
                return Some((dist[target], path));
            }

            if dist[current] >= MAX_HOPS { continue; }

            for &neighbor in &self.nodes[current].neighbors {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    parent[neighbor] = Some(current);
                    dist[neighbor] = dist[current] + 1;
                    queue.push_back(neighbor);
                }
            }
        }
        None
    }

    pub fn route(&self, source: usize, target: usize) -> Option<&HdtRoute> {
        self.routes.get(&(source, target))
    }

    pub fn route_to_capability(&self, capability: &str, agent_map: &HashMap<String, usize>) -> Option<&HdtRoute> {
        let target = agent_map.get(capability)?;
        self.routes.get(&(0, *target))
    }

    pub fn get_node(&self, name: &str) -> Option<&HdtNode> {
        self.nodes.iter().find(|n| n.name == name)
    }

    pub fn get_node_id(&self, name: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.name == name)
    }

    pub fn simulate_failure(&mut self, node_id: usize) -> Result<()> {
        if node_id >= NUM_NODES {
            return Err(anyhow::anyhow!("Nodo inválido: {}", node_id));
        }
        self.nodes[node_id].active = false;
        self.active_nodes = self.nodes.iter().filter(|n| n.active).count();
        self.precompute_routes();
        tracing::warn!("HDT: Nodo {} caído. {} nodos activos, rutas recalculadas.", node_id, self.active_nodes);
        Ok(())
    }

    pub fn topology_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "Hexagonal Doble Teroideal",
            "nodes": NUM_NODES,
            "bridges": TOROIDAL_BRIDGES.len(),
            "max_hops": MAX_HOPS,
            "active_nodes": self.active_nodes,
            "diameter": 3,
            "redundancy": "N+2",
            "network": self.nodes.iter().map(|n| serde_json::json!({
                "id": n.id,
                "name": n.name,
                "neighbors": n.neighbors,
                "active": n.active,
                "load": n.load,
            })).collect::<Vec<_>>(),
            "total_routes": self.routes.len(),
        })
    }
}
