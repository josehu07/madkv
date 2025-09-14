/// Client request event, annotated with client source.
event ClientRequest: (op: Operation, cli: Client, cId: int, tsReq: int);

// Client response event, annotated with client destination.
event ClientResponse: (op: Operation, cli: Client, cId: int, tsReq: int);

/*****************************************************
 Machine mimicking a client of the shared object pool.
******************************************************/
machine Client
{
    var cId: int;
    var service: ObjectPool;
    var numReqs: int;

    var numKeys: int;  // number of keys to consider
    var numVals: int;  // range of values to consider
    var opTypes: set[OpType];  // valid operation types

    start state Init {
        entry (input: (cId: int, service: ObjectPool, numReqs: int, numKeys: int, numVals: int)) {
            assert input.numKeys > 0;

            cId = input.cId;
            service = input.service;
            numReqs = input.numReqs;
            numKeys = input.numKeys;

            opTypes += ( OP_READ );
            opTypes += ( OP_WRITE );
            opTypes += ( OP_RADDW );

            goto MakeRequests;
        }

        exit {
            print format ("Client {0} starting {1} requests", cId, numReqs);
        }
    }

    state MakeRequests {
        entry {
            if (numReqs <= 0)
                goto Finished;

            send service, ClientRequest,
                 (op = GenerateOp(), cli = this, cId = cId, tsReq = GetTimestamp());
        }

        on ClientResponse do (resp: (op: Operation, cli: Client, cId: int, tsReq: int)) {
            assert this == resp.cli;
            assert resp.op.key >= 0 && resp.op.key < numKeys;

            // notify the spec monitor that I observed this response
            announce ClientSeesResponse,
                     (op = resp.op, cId = cId, tsReq = resp.tsReq, tsResp = GetTimestamp());

            numReqs = numReqs - 1;
            goto MakeRequests;
        }
    }

    // generate a random operation for request
    fun GenerateOp(): Operation {
        var typ: OpType;
        var key: int;
        var val: int;

        typ = choose(opTypes);
        key = choose(numKeys);
        val = choose(numVals);

        return (typ = typ, key = key, val = val, ret = 0);
    }

    state Finished {
        entry {
            assert numReqs == 0, "Client entered finished state too early";
            print format ("Client {0} finished all requests", cId);
        }
    }
}