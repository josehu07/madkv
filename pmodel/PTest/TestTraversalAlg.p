/***************************************************************************
 Dummy test driver that tests the DFS graph traversal algorithm defined in
 `PSpec/Consistency.p` against handcrafted examples.
****************************************************************************/
machine TestTraversalAlg
{
    start state Init {
        entry {
            TestDFSTraversal();
            TestDFSTraversalNonExistNode();
            TestDFSTraversalDisconnected();
            TestDFSTraversalBadHeadNode();
            TestDFSTraversalDetectCycle();
        }
    }

    // Inner test: DFS traversal algorithm logic valid case.
    fun TestDFSTraversal() {
        var ordering: OrderingGraph;
        var nId: int;
        var node: OrderingNode;
        var op: Operation;
        var pool: map[int, int];
        var next: set[int];
        var dfs_result: (ConvergenceLevel, map[int, set[int]]);
        var convergence: ConvergenceLevel;
        var reachable: map[int, set[int]];
        var should_reach: set[(int, int)];
        var sra: int;
        var srb: int;

        // compose example graph
        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 7, ret = 0);
        pool += ( 0, 7 );
        next += ( 1 );
        next += ( 2 );
        node = ( nId = 0, cId = 0, op = op, tsReq = 1, tsResp = 2, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_RADDW, key = 0, val = 3, ret = 7);
        pool += ( 0, 10 );
        next += ( 3 );
        next += ( 5 );
        node = ( nId = 1, cId = 0, op = op, tsReq = 3, tsResp = 6, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_RADDW, key = 0, val = -3, ret = 7);
        pool += ( 0, 4 );
        next += ( 4 );
        node = ( nId = 2, cId = 0, op = op, tsReq = 4, tsResp = 7, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 9, ret = 0);
        pool += ( 0, 9 );
        next += ( 4 );
        node = ( nId = 3, cId = 0, op = op, tsReq = 5, tsResp = 8, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 4);
        pool += ( 0, 4 );
        node = ( nId = 4, cId = 0, op = op, tsReq = 9, tsResp = 11, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_RADDW, key = 0, val = 1, ret = 10);
        pool += ( 0, 11 );
        node = ( nId = 5, cId = 0, op = op, tsReq = 10, tsResp = 12, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        ordering.head = 0;

        // test traversal result on this graph
        dfs_result = ConvergenceConformity(ordering);
        convergence = dfs_result.0;
        reachable = dfs_result.1;

        assert convergence != C_INVALID, "DFS traversal failed on valid graph";
        foreach (nId in keys(ordering.nodes)) {
            assert nId in reachable,
                   format ("Node {0} not found in reachable map", nId);
        }
        should_reach += ( (0, 1) );
        should_reach += ( (0, 2) );
        should_reach += ( (0, 3) );
        should_reach += ( (0, 4) );
        should_reach += ( (0, 5) );
        should_reach += ( (1, 3) );
        should_reach += ( (1, 4) );
        should_reach += ( (1, 5) );
        should_reach += ( (2, 4) );
        should_reach += ( (3, 4) );
        foreach (sra in keys(ordering.nodes)) {
            foreach (srb in keys(ordering.nodes)) {
                if ( sra != srb ) {
                    if ( (sra, srb) in should_reach ) {
                        assert srb in reachable[sra],
                               format ("Node {0} should reach {1}", sra, srb);
                    } else {
                        assert !(srb in reachable[sra]),
                               format ("Node {0} should NOT reach {1}", sra, srb);
                    }
                }
            }
        }
    }

    // Inner test: DFS traversal algorithm logic with non-existent node.
    fun TestDFSTraversalNonExistNode() {
        var ordering: OrderingGraph;
        var node: OrderingNode;
        var op: Operation;
        var pool: map[int, int];
        var next: set[int];
        var dfs_result: (ConvergenceLevel, map[int, set[int]]);

        // compose example graph
        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 7, ret = 0);
        pool += ( 0, 7 );
        next += ( 1 );
        next += ( 2 );
        node = ( nId = 0, cId = 0, op = op, tsReq = 1, tsResp = 2, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_RADDW, key = 0, val = 3, ret = 7);
        pool += ( 0, 10 );
        node = ( nId = 1, cId = 0, op = op, tsReq = 3, tsResp = 6, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        ordering.head = 0;

        // test traversal result on this graph
        dfs_result = ConvergenceConformity(ordering);
        assert dfs_result.0 == C_INVALID,
               "DFS traversal should fail on graph with non-existent node";
    }

    // Inner test: DFS traversal algorithm logic with disconnected graph.
    fun TestDFSTraversalDisconnected() {
        var ordering: OrderingGraph;
        var node: OrderingNode;
        var op: Operation;
        var pool: map[int, int];
        var next: set[int];
        var dfs_result: (ConvergenceLevel, map[int, set[int]]);

        // compose example graph
        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 7, ret = 0);
        pool += ( 0, 7 );
        node = ( nId = 0, cId = 0, op = op, tsReq = 1, tsResp = 2, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_RADDW, key = 0, val = 3, ret = 7);
        pool += ( 0, 10 );
        node = ( nId = 1, cId = 0, op = op, tsReq = 3, tsResp = 6, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        ordering.head = 0;

        // test traversal result on this graph
        dfs_result = ConvergenceConformity(ordering);
        assert dfs_result.0 == C_INVALID,
               "DFS traversal should fail on disconnected graph";
    }

    // Inner test: DFS traversal algorithm logic with bad head node.
    fun TestDFSTraversalBadHeadNode() {
        var ordering: OrderingGraph;
        var node: OrderingNode;
        var op: Operation;
        var pool: map[int, int];
        var next: set[int];
        var dfs_result: (ConvergenceLevel, map[int, set[int]]);

        // compose example graph
        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 7, ret = 0);
        pool += ( 0, 7 );
        next += ( 1 );
        node = ( nId = 0, cId = 0, op = op, tsReq = 1, tsResp = 2, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_RADDW, key = 0, val = 3, ret = 7);
        pool += ( 0, 10 );
        next += ( 2 );
        node = ( nId = 1, cId = 0, op = op, tsReq = 3, tsResp = 6, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 10);
        pool += ( 0, 10 );
        node = ( nId = 2, cId = 0, op = op, tsReq = 7, tsResp = 8, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        ordering.head = 1;

        // test traversal result on this graph
        dfs_result = ConvergenceConformity(ordering);
        assert dfs_result.0 == C_INVALID,
               "DFS traversal should fail on graph with bad starting head node";
    }

    // Inner test: DFS traversal algorithm logic with cycle in graph.
    fun TestDFSTraversalDetectCycle() {
        var ordering: OrderingGraph;
        var node: OrderingNode;
        var op: Operation;
        var pool: map[int, int];
        var next: set[int];
        var dfs_result: (ConvergenceLevel, map[int, set[int]]);

        // compose example graph
        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 7, ret = 0);
        pool += ( 0, 7 );
        next += ( 1 );
        node = ( nId = 0, cId = 0, op = op, tsReq = 1, tsResp = 4, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_RADDW, key = 0, val = 3, ret = 7);
        pool += ( 0, 10 );
        next += ( 2 );
        node = ( nId = 1, cId = 0, op = op, tsReq = 3, tsResp = 6, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 10);
        pool += ( 0, 10 );
        next += ( 0 );
        node = ( nId = 2, cId = 0, op = op, tsReq = 2, tsResp = 5, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        ordering.head = 0;

        // test traversal result on this graph
        dfs_result = ConvergenceConformity(ordering);
        assert dfs_result.0 == C_INVALID,
               "DFS traversal should fail on graph with ordering cycles";
    }
}

// Declare the dummy test scenario.
test CheckGraphTraversalAlg [main=TestTraversalAlg]: { TestTraversalAlg };
