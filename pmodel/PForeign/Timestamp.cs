using System;
using PChecker.Runtime.Values;
using PChecker.Runtime.StateMachines;

// All foreign code for P always implemented under `PImeplementation` namespace.
namespace PImplementation
{
    // Functions not associated to any state machine as methods are always put under
    // `GlobalFunctions` static class.
    public static partial class GlobalFunctions
    {
        public static PInt GetTimestamp(StateMachine machine)
        {
            return (PInt)DateTimeOffset.UtcNow.Ticks;
        }
    }
}
