// Ordering graph node -- records the corresponding client response, object pool state
// after this operation, and out-going neighbor nodes.
// Assumption: for each client (i.e., session), its operations are strictly single-threaded.
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
// NOTE: assuming there's a single head node to start from.
type OrderingGraph = (
    nodes: map[int, OrderingNode],
    head: int  // head node ID
);

// Convergence constraint level.
// NOTE: for CPO, we are currently just checking if it is a valid partial order (DAG, no cycles),
//       where at each convergence point, the pool state is a valid transtion from any one of the
//       parents. This is insufficient as user could define a custom conflict resolution function
//       and the function must deterministic at all convergence points. We leave this as a TODO
//       item for the spec.
//       Correspondingly, the NPO definition is not exactly correct either -- it should mean
//       "multiple different outcomes detected at convergence points" instead of not transitioning
//       from one of the parents.
enum ConvergenceLevel { C_SO = 3, C_CPO = 2, C_NPO = 1, C_INVALID = 0 }

// Relationship constraint level.
enum RelationshipLevel { R_RT = 3, R_CASL = 2, R_FIFO = 1, R_NONE = 0 }

// Check convergence constraint conformity level: SO, CPO, NPO, or invalid?
// Uses a colored DFS traversal algorithm. Also returns a map from node ID -> all nodes reachable
// from it, produced from this DFS traversal, useful for checking relationship constraints later.
fun ConvergenceConformity(ordering: OrderingGraph): (ConvergenceLevel, map[int, set[int]]) {
    var level: ConvergenceLevel;
    var stack: seq[int];
    var color: map[int, int];  // 0: unvisited, 1: subtree in progress, 2: fully visited
    var nId: int;
    var done: int;
    var curr: int;
    var next: int;
    var reachable: map[int, set[int]];  // node ID -> set of reachable node IDs from it
    var parents: map[int, set[int]];  // node ID -> set of immediate parent nodes of it
    var pValid: bool;

    assert ordering.head in ordering.nodes,
           format ("Ordering graph head node {0} does not exist", ordering.head);
    level = C_SO;
    stack += ( 0, ordering.head );
    foreach (nId in keys(ordering.nodes)) {
        color += ( nId, 0 );
        reachable += ( nId, default(set[int]) );
        parents += ( nId, default(set[int]) );
    }
    done = 0;

    while (sizeof(stack) > 0) {
        curr = stack[sizeof(stack) - 1];

        if (color[curr] == 0) {
            // first visit, mark as in-progress
            color[curr] = 1;

            if (level == C_SO && sizeof(ordering.nodes[curr].next) > 1) {
                // SO must have exactly one outgoing edge to the next node
                print format ("Node {0} has multiple outgoing edges", curr);
                level = C_CPO;
            }

            foreach (next in ordering.nodes[curr].next) {
                if (!(next in ordering.nodes)) {
                    // non-existing node
                    print format ("Node {0} has an edge to non-existing node {1}", curr, next);
                    return (C_INVALID, reachable);
                }

                parents[next] += ( curr );

                if (color[next] == 0) {
                    // unvisited neighbor, push to stack
                    stack += ( sizeof(stack), next );
                } else if (color[next] == 1) {
                    // back edge found, cycle detected, not DAG
                    print format ("Cycle detected at node {0} -> {1}", curr, next);
                    return (C_INVALID, reachable);
                } else {
                    // ignore fully visited nodes
                }
            }

        } else {
            if (color[curr] == 1) {
                // in-progress node poped, means its subtree is fully visited
                foreach (next in ordering.nodes[curr].next) {
                    assert color[next] == 2;
                    reachable[curr] += ( next );
                    foreach (nId in reachable[next]) {
                        reachable[curr] += ( nId );
                    }
                }

                color[curr] = 2;
                done = done + 1;
                stack -= ( sizeof(stack) - 1 );
            }
        }
    }

    // must have visited all nodes to be a connected graph from head
    if (done < sizeof(ordering.nodes)) {
        print format ("Not all nodes in ordering graph were visited");
        return (C_INVALID, reachable);
    }

    // check that pool state transitions are valid
    foreach (next in keys(parents)) {
        if (sizeof(parents[next]) == 0) {
            assert next == ordering.head;
            continue;
        }

        pValid = false;
        foreach (nId in parents[next]) {
            if (ordering.nodes[next].pool == ApplyOpToPool(ordering.nodes[next].op,
                                                           ordering.nodes[nId].pool)) {
                pValid = true;
                break;
            }
        }
        if (!pValid) {
            // impossible pool state after this node's op applied
            print format ("Invalid pool state transition at node {0}", next);
            return (C_NPO, reachable);
        }
    }

    return (level, reachable);
}

// Check relationship constraint conformity level: RT, CASL, FIFO, or less?
// We assume each client is strictly single-threaded, hence no "cluster of reads".
fun RelationshipConformity(ordering: OrderingGraph, reachable: map[int, set[int]]): RelationshipLevel {
    var level: RelationshipLevel;
    var na: int;
    var nb: int;

    level = R_RT;

    // loop through all pairs of nodes
    foreach (na in keys(ordering.nodes)) {
        foreach (nb in keys(ordering.nodes)) {
            if (na == nb) {
                continue;
            }

            // check real-time constraint
            if (level == R_RT && ordering.nodes[na].tsResp < ordering.nodes[nb].tsReq
                              && !(nb in reachable[na])) {
                // nb not ordered after na, RT violation
                print format ("RT violation: node {0} should be before {1}", na, nb);
                level = R_CASL;
            }

            // check strict session order constraint
            if (level == R_CASL && ordering.nodes[na].cId == ordering.nodes[nb].cId
                                && ordering.nodes[na].tsResp < ordering.nodes[nb].tsReq
                                && !(nb in reachable[na])) {
                // nb not ordered after na in same session, CASL violation
                print format ("CASL violation: node {0} should be before {1}", na, nb);
                level = R_FIFO;
            }

            // check session order without writes following reads
            if (level == R_FIFO && ordering.nodes[na].cId == ordering.nodes[nb].cId
                                && ordering.nodes[na].tsResp < ordering.nodes[nb].tsReq
                                && !(ordering.nodes[na].op.typ == OP_READ
                                     && ordering.nodes[nb].op.typ == OP_WRITE)
                                && !(nb in reachable[na])) {
                // nb not ordered after na in same session, and is not a write-follow-read
                // situation, FIFO violation
                print format ("FIFO violation: node {0} should be before {1}", na, nb);
                return R_NONE;
            }
        }
    }

    return level;
}

// Generic satisfaction check conjuncting convergence and relationship constraints.
fun SatisfiedConstraints(ordering: OrderingGraph): (ConvergenceLevel, RelationshipLevel) {
    var dfs_result: (ConvergenceLevel, map[int, set[int]]);
    var convergence: ConvergenceLevel;
    var relationship: RelationshipLevel;
    var reachable: map[int, set[int]];

    dfs_result = ConvergenceConformity(ordering);
    convergence = dfs_result.0;
    if (convergence == C_INVALID) {
        return (C_INVALID, R_NONE);  // no point checking relationship if graph malformed
    }

    reachable = dfs_result.1;
    relationship = RelationshipConformity(ordering, reachable);
    return (convergence, relationship);
}

// Linearizability  ===  SO + RT
fun AssertLinearizability(ordering: OrderingGraph) {
    var constraints: (ConvergenceLevel, RelationshipLevel);
    constraints = SatisfiedConstraints(ordering);
    assert constraints.0 == C_SO,
           format ("Convergence constraint != SO for linearizability: {0}", constraints.0);
    assert constraints.1 == R_RT,
           format ("Relationship constraint != RT for linearizability: {0}", constraints.1);
}

// Sequential consistency  ===  SO + CASL
fun AssertSequentialConsistency(ordering: OrderingGraph) {
    var constraints: (ConvergenceLevel, RelationshipLevel);
    constraints = SatisfiedConstraints(ordering);
    assert constraints.0 == C_SO,
           format ("Convergence constraint != SO for sequential consistency: {0}", constraints.0);
    assert constraints.1 == R_CASL,
           format ("Relationship constraint != CASL for sequential consistency: {0}", constraints.1);
}

// Causal+ consistency  ===  CPO + CASL
fun AssertCausalConsistency(ordering: OrderingGraph) {
    var constraints: (ConvergenceLevel, RelationshipLevel);
    constraints = SatisfiedConstraints(ordering);
    assert constraints.0 == C_CPO,
           format ("Convergence constraint != CPO for causal consistency: {0}", constraints.0);
    assert constraints.1 == R_CASL,
           format ("Relationship constraint != CASL for causal consistency: {0}", constraints.1);
}

// PRAM consistency  ===  NPO + FIFO
fun AssertPRAMConsistency(ordering: OrderingGraph) {
    var constraints: (ConvergenceLevel, RelationshipLevel);
    constraints = SatisfiedConstraints(ordering);
    assert constraints.0 == C_NPO,
           format ("Convergence constraint != NPO for PRAM consistency: {0}", constraints.0);
    assert constraints.1 == R_FIFO,
           format ("Relationship constraint != FIFO for PRAM consistency: {0}", constraints.1);
}

// Eventual consistency  ===  CPO + NONE
fun AssertEventualConsistency(ordering: OrderingGraph) {
    var constraints: (ConvergenceLevel, RelationshipLevel);
    constraints = SatisfiedConstraints(ordering);
    assert constraints.0 == C_CPO,
           format ("Convergence constraint != CPO for eventual consistency: {0}", constraints.0);
    assert constraints.1 == R_NONE,
           format ("Relationship constraint != NONE for eventual consistency: {0}", constraints.1);
}

// Weak consistency  ===  NPO + NONE
fun AssertWeakConsistency(ordering: OrderingGraph) {
    var constraints: (ConvergenceLevel, RelationshipLevel);
    constraints = SatisfiedConstraints(ordering);
    assert constraints.0 == C_NPO,
           format ("Convergence constraint != NPO for weak consistency: {0}", constraints.0);
    assert constraints.1 == R_NONE,
           format ("Relationship constraint != NONE for weak consistency: {0}", constraints.1);
}
