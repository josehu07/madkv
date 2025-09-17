/***************************************************************************
 Dummy test driver that tests the consistency checking functions defined in
 `PSpec/Consistency.p` against handcrafted examples.
****************************************************************************/
machine TestConsistency
{
    start state Init {
        entry {
            TestLinearizableOrdering();
            TestSequentiallyConsistentOrdering();
            TestCausallyConsistentOrdering();
            TestEventuallyConsistentOrdering();
            TestPRAMConsistentOrdering();
            TestWeaklyConsistentOrdering();
        }
    }

    // Inner test: a linearizable ordering example.
    fun TestLinearizableOrdering() {
        var ordering: OrderingGraph;
        var node: OrderingNode;
        var op: Operation;
        var pool: map[int, int];
        var next: set[int];

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 1, ret = 0);
        pool += ( 0, 1 );
        next += ( 1 );
        node = ( nId = 0, cId = 0, op = op, tsReq = 1, tsResp = 3, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 2, ret = 0);
        pool += ( 0, 2 );
        next += ( 2 );
        node = ( nId = 1, cId = 1, op = op, tsReq = 2, tsResp = 4, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_RADDW, key = 0, val = 5, ret = 2);
        pool += ( 0, 7 );
        next += ( 3 );
        node = ( nId = 2, cId = 0, op = op, tsReq = 5, tsResp = 6, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 7);
        pool += ( 0, 7 );
        node = ( nId = 3, cId = 1, op = op, tsReq = 7, tsResp = 8, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        ordering.head = 0;

        AssertLinearizability(ordering);
    }

    // Inner test: a sequential consistency example.
    fun TestSequentiallyConsistentOrdering() {
        var ordering: OrderingGraph;
        var node: OrderingNode;
        var op: Operation;
        var pool: map[int, int];
        var next: set[int];

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 1, ret = 0);
        pool += ( 0, 1 );
        next += ( 1 );
        node = ( nId = 0, cId = 0, op = op, tsReq = 1, tsResp = 2, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_RADDW, key = 0, val = 5, ret = 1);
        pool += ( 0, 6 );
        next += ( 2 );
        node = ( nId = 1, cId = 1, op = op, tsReq = 5, tsResp = 6, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 9, ret = 0);
        pool += ( 0, 9 );
        next += ( 3 );
        node = ( nId = 2, cId = 0, op = op, tsReq = 3, tsResp = 4, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 9);
        pool += ( 0, 9 );
        node = ( nId = 3, cId = 1, op = op, tsReq = 7, tsResp = 8, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        ordering.head = 0;

        AssertSequentialConsistency(ordering);
    }

    // Inner test: a causal consistency example.
    fun TestCausallyConsistentOrdering() {
        var ordering: OrderingGraph;
        var node: OrderingNode;
        var op: Operation;
        var pool: map[int, int];
        var next: set[int];

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 0);
        pool += ( 0, 0 );
        pool += ( 1, 0 );
        next += ( 1 );
        next += ( 2 );
        node = ( nId = 0, cId = 0, op = op, tsReq = 0, tsResp = 0, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 1, ret = 0);
        pool += ( 0, 1 );
        pool += ( 1, 0 );
        next += ( 3 );
        next += ( 4 );
        node = ( nId = 1, cId = 0, op = op, tsReq = 1, tsResp = 2, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 2, ret = 1);
        pool += ( 0, 2 );
        pool += ( 1, 0 );
        next += ( 4 );
        node = ( nId = 2, cId = 1, op = op, tsReq = 3, tsResp = 4, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 1, val = 1, ret = 0);
        pool += ( 0, 1 );
        pool += ( 1, 1 );
        node = ( nId = 3, cId = 0, op = op, tsReq = 5, tsResp = 6, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 2);
        pool += ( 0, 2 );
        pool += ( 1, 0 );
        next += ( 5 );
        node = ( nId = 4, cId = 2, op = op, tsReq = 7, tsResp = 8, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_RADDW, key = 1, val = 3, ret = 0);
        pool += ( 0, 2 );
        pool += ( 1, 3 );
        node = ( nId = 5, cId = 2, op = op, tsReq = 9, tsResp = 10, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        ordering.head = 0;

        AssertCausalConsistency(ordering);
    }

    // Inner test: an eventual consistency example.
    fun TestEventuallyConsistentOrdering() {
        var ordering: OrderingGraph;
        var node: OrderingNode;
        var op: Operation;
        var pool: map[int, int];
        var next: set[int];

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 0);
        pool += ( 0, 0 );
        next += ( 1 );
        next += ( 2 );
        node = ( nId = 0, cId = 0, op = op, tsReq = 0, tsResp = 0, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 2, ret = 0);
        pool += ( 0, 2 );
        next += ( 3 );
        node = ( nId = 1, cId = 0, op = op, tsReq = 4, tsResp = 6, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 3, ret = 0);
        pool += ( 0, 3 );
        next += ( 4 );
        node = ( nId = 2, cId = 1, op = op, tsReq = 3, tsResp = 5, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 1, ret = 0);
        pool += ( 0, 1 );
        next += ( 4 );
        node = ( nId = 3, cId = 0, op = op, tsReq = 1, tsResp = 2, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 3);
        pool += ( 0, 3 );
        node = ( nId = 4, cId = 0, op = op, tsReq = 7, tsResp = 8, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        ordering.head = 0;

        AssertEventualConsistency(ordering);
	}

    // Inner test: a PRAM consistency example.
    fun TestPRAMConsistentOrdering() {
        var ordering: OrderingGraph;
        var node: OrderingNode;
        var op: Operation;
        var pool: map[int, int];
        var next: set[int];

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 0);
        pool += ( 0, 0 );
        pool += ( 1, 0 );
        next += ( 1 );
        next += ( 2 );
        node = ( nId = 0, cId = 0, op = op, tsReq = 0, tsResp = 0, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 1, ret = 0);
        pool += ( 0, 1 );
        pool += ( 1, 0 );
        next += ( 3 );
        next += ( 4 );
        node = ( nId = 1, cId = 0, op = op, tsReq = 1, tsResp = 2, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 2, ret = 1);
        pool += ( 0, 2 );
        pool += ( 1, 0 );
        next += ( 4 );
        node = ( nId = 2, cId = 1, op = op, tsReq = 3, tsResp = 4, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 1, val = 1, ret = 0);
        pool += ( 0, 1 );
        pool += ( 1, 1 );
        node = ( nId = 3, cId = 0, op = op, tsReq = 5, tsResp = 6, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 9);
        pool += ( 0, 9 );
        pool += ( 1, 0 );
        next += ( 5 );
        node = ( nId = 4, cId = 2, op = op, tsReq = 7, tsResp = 8, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_RADDW, key = 1, val = 3, ret = 0);
        pool += ( 0, 9 );
        pool += ( 1, 3 );
        next += ( 6 );
        node = ( nId = 5, cId = 2, op = op, tsReq = 9, tsResp = 11, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 1, val = 8, ret = 0);
        pool += ( 0, 9 );
        pool += ( 1, 8 );
        next += ( 7 );
        node = ( nId = 6, cId = 1, op = op, tsReq = 13, tsResp = 14, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 1, val = 0, ret = 3);
        pool += ( 0, 9 );
        pool += ( 1, 3 );
        node = ( nId = 7, cId = 1, op = op, tsReq = 10, tsResp = 12, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        ordering.head = 0;

        AssertPRAMConsistency(ordering);
	}

    // Inner test: a weak consistency example.
    fun TestWeaklyConsistentOrdering() {
        var ordering: OrderingGraph;
        var node: OrderingNode;
        var op: Operation;
        var pool: map[int, int];
        var next: set[int];

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 0);
        pool += ( 0, 0 );
        next += ( 1 );
        next += ( 2 );
        node = ( nId = 0, cId = 0, op = op, tsReq = 0, tsResp = 0, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 2, ret = 0);
        pool += ( 0, 2 );
        next += ( 3 );
        node = ( nId = 1, cId = 0, op = op, tsReq = 4, tsResp = 6, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 3, ret = 0);
        pool += ( 0, 3 );
        next += ( 4 );
        node = ( nId = 2, cId = 1, op = op, tsReq = 3, tsResp = 5, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_WRITE, key = 0, val = 1, ret = 0);
        pool += ( 0, 1 );
        next += ( 4 );
        node = ( nId = 3, cId = 0, op = op, tsReq = 1, tsResp = 2, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        pool = default(map[int, int]);
        next = default(set[int]);
        op = (typ = OP_READ, key = 0, val = 0, ret = 9);
        pool += ( 0, 9 );
        node = ( nId = 4, cId = 0, op = op, tsReq = 7, tsResp = 8, pool = pool, next = next );
        ordering.nodes += ( node.nId, node );

        ordering.head = 0;

        AssertWeakConsistency(ordering);
	}
}

// Declare the dummy test scenario.
test CheckConsistencyExamples [main=TestConsistency]: { TestConsistency };
