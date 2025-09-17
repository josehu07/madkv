// Event to initialize the spec monitor with initial object pool.
event ServiceInitialPool: map[int, int];

// Event announced when a client observes a response from the object pool.
event ClientSeesResponse: (op: Operation, cId: int, tsReq: int, tsResp: int);

/***************************************************************
 Checks that the service always yields a linearizable history.
 Since the service is literally a single atomic process, this
 abstract spec is easy to write: just check upon every announced
 response and see if it continues the linear history.
****************************************************************/
spec ServiceIsLinearizable
observes ServiceInitialPool, ClientSeesResponse
{
    // record the linear history of operations and the state of the object pool
    // after each operation
    var history: OrderingGraph;
    var nodeId: int;  // next node id to assign

    start state Init {
        on ServiceInitialPool goto WaitForResponses with (initialPool: map[int, int]) {
            // the history starts with a dummy response with the initial pool state
            history.nodes += ( nodeId,
                (nId = nodeId,
                 cId = 0,
                 op = default(Operation),
                 tsReq = 0,
                 tsResp = 0,
                 pool = initialPool,
                 next = default(set[int]))
            );

            history.head = nodeId;
            nodeId = nodeId + 1;
        }
    }

    state WaitForResponses {
        on ClientSeesResponse do (resp: (op: Operation, cId: int, tsReq: int, tsResp: int)) {
            AppendResponse(resp);
            AssertLinearizability(history);

            nodeId = nodeId + 1;
        }
    }

    // Append an incoming response to the end of the linear history.
    fun AppendResponse(resp: (op: Operation, cId: int, tsReq: int, tsResp: int)) {
        var node: OrderingNode;
        node.nId = nodeId;
        node.cId = resp.cId;
        node.op = resp.op;
        node.tsReq = resp.tsReq;
        node.tsResp = resp.tsResp;

        // since we expect a strictly linear, real-time history, must append
        // this node to the end of the linear history
        node.pool = ApplyOpToPool(resp.op, history.nodes[nodeId - 1].pool);
        node.next = default(set[int]);

        // also remember to set the `next` edge from previous node to me
        history.nodes[nodeId - 1].next += ( nodeId );

        history.nodes += ( nodeId, node );
    }
}

// Expected mutation that happens when applying an operation to a pool state.
fun ApplyOpToPool(op: Operation, pool: map[int, int]): map[int, int] {
    var newPool: map[int, int];
    newPool = pool;

    assert op.key in pool;

    if (op.typ == OP_READ) {
        // no state change
    } else if (op.typ == OP_WRITE) {
        newPool[op.key] = op.val;
    } else if (op.typ == OP_RADDW) {
        newPool[op.key] = pool[op.key] + op.val;
    } else {
        assert false, "Unknown operation type received";
    }

    return newPool;
}
