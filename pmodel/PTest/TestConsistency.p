/***************************************************************************
 Dummy test driver that tests the consistency checking functions defined in
 `PSpec/Consistency.p` against handcrafted examples.
****************************************************************************/
machine TestConsistency
{
    start state Init {
        entry {
            assert true;  // TODO:
        }
    }
}
