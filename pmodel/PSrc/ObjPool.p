// Client operation type. `RADDW` stands for read-add-write, which serves as an example of
// a read-modify-write operation.
enum OpType {
  OP_READ,
  OP_WRITE,
  OP_RADDW
}

// Single operation structure.
type Operation = (
  typ: OpType,
  key: int,
  val: int, // used only for WRITE and RADDW
  ret: int  // used only for READ and RADDW
);

/*************************************************************************
 Machine mimicking a shared (assume replicated), linearizable object pool.
**************************************************************************/
machine ObjectPool
{
  var pool: map[int, int];

  start state Init {
    entry (initialPool: map[int, int]) {
      assert sizeof(initialPool) > 0;
      pool = initialPool;

      goto WaitForRequests;
    }

    exit {
      print format ("Object pool initialized to {0}", pool);
    }
  }

  state WaitForRequests {
    on ClientRequest do (req: (op: Operation, cli: Client, cId: int, tsReq: int)) {
      var respOp: Operation;
      var retVal: int;
      retVal = 0;
      
      assert req.op.key in pool;

      // serve the request according to its type
      if (req.op.typ == OP_READ) {
        retVal = pool[req.op.key];
      } else if (req.op.typ == OP_WRITE) {
        pool[req.op.key] = req.op.val;
      } else if (req.op.typ == OP_RADDW) {
        retVal = pool[req.op.key];
        pool[req.op.key] = retVal + req.op.val;
      } else {
        assert false, "Unknown operation type received";
      }

      respOp = (typ = req.op.typ, key = req.op.key, val = 0, ret = retVal);

      // send response to the client
      send req.cli, ClientResponse,
           (op = respOp, cli = req.cli, cId = req.cId, tsReq = req.tsReq);
    }
  }
}
