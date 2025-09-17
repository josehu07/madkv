// Test parameter: number of clients to spawn for `TestRefObjectPool`.
param numClients: int;

/**********************************************************************************
 Test driver that uses multiple clients to issue requests against the linearizable
 object pool machine, checking that linearizability always holds.
***********************************************************************************/
machine TestRefObjectPool
{
    var service: ObjectPool;
    var clients: map[int, Client];

    var numKeys: int;
    var numVals: int;

    start state Init {
        entry {
            numKeys = 1 + choose(3);  // pick between 1 to 3 keys
            numVals = 1 + choose(5);  // pick between 1 to 5 possible initial values

            SetupService();
            SetupClients();
        }
    }

    fun SetupService() {
        var initPool: map[int, int];
        var ki: int;

        while (ki < numKeys) {
            initPool += ( ki, choose(numVals) );
            ki = ki + 1;
        }

        service = new ObjectPool(initPool);

        // make sure spec monitor is kicked off before starting any clients
        announce ServiceInitialPool, initPool;
    }

    fun SetupClients() {
        var numReqs: int;
        var ci: int;

        numReqs = 1 + choose(5);  // each client issues between 1 to 5 requests
        
        while (ci < numClients) {
            clients += ( ci, new Client(
                (cId = ci, service = service, numReqs = numReqs, numKeys = numKeys, numVals = numVals)
            ));
            ci = ci + 1;
        }
    }
}

// Declare the model checking test scenario -- with 1, or 5 clients.
test param (numClients in [1, 5]) CheckRefObjectPoolImpl [main=TestRefObjectPool]:
    assert ServiceIsLinearizable in
    (union clientMod, serviceMod, { TestRefObjectPool });
