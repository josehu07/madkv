// Ordering graph node -- records the corresponding client response, object pool state
// after this operation, and out-going neighbor nodes.
type OrderingNode = (
    nId: int,  // node ID
    cId: int,  // client ID where this operation came from
    op: Operation,
    tsReq: int,  // request timestamp
    tsResp: int,  // response timestamp
    pool: map[int, int],  // state after this operation
    next: set[int]  // set of node IDs where I have an out-going edge to
);

// Ordering graph definition.
type OrderingGraph = (
    nodes: map[int, OrderingNode],
    head: int  // head node ID
);

// Linearizability  ===  SO + RT
fun SatisfiesLinearizability(ordering: OrderingGraph): bool {
    var visited: set[int];
    var curr: int;
    var next: int;
    curr = ordering.head;

    while (curr in ordering.nodes) {
        visited += ( curr );

        // must have exactly one outgoing edge to the next node
        if (sizeof(ordering.nodes[curr].next) > 1) {
            print format ("Node {0} has multiple outgoing edges", curr);
            return false;
        }

        // if no outgoing edge, must be the end of the linear history
        if (sizeof(ordering.nodes[curr].next) == 0) {
            break;
        }

        // check the validity between (curr, next) pair
        next = (ordering.nodes[curr]).next[0];
        if (ordering.nodes[next].pool != ApplyOpToPool(ordering.nodes[next].op,
                                                       ordering.nodes[curr].pool)) {
            print format ("Invalid pool transition from node {0} -> {1}", curr, next);
            return false;
        }
        if (ordering.nodes[next].tsResp < ordering.nodes[curr].tsReq) {
            print format ("Real-time order violated from node {0} -> {1}", curr, next);
            return false;
        }

        curr = next;
    }

    // must have visited all nodes
    if (sizeof(visited) < sizeof(ordering.nodes)) {
        print format ("Not all nodes in ordering graph were visited");
        return false;
    }

    return true;
}
