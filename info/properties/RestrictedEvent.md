# Restricted Event

This propety aim to apply a restriction on authorized events.

## When is the restriction applied ?

The property is verified using a modify DFS algorithm.
Because we focus on deterministic Mealy machine, there is only one starting state and the exploration from the initial state.

The restriction is applied between:
- initial event: if during exploration a matching event is reached then the restriction is applied. In the case of a missing initial event, the restriction is applied at the beginning of the exploration of the Mealy machine.
- Release event: after reaching an event matching this one, the restriction is no longer applied. In case of no release event, the restriction remain after being applied.
- Cancel event: When the exploration reach a matching event the rule will no longer apply. This is true if and only if this event is seen before the starting event. This event is optional.


## Syntax of the property
The syntax is:
```
RE:PROPERTY9
    C: cancel event 
    I: initial event
    A: Authorized event (written with an EventS)
    R: release event 
:RE
```

An example is :
```
RE:PROPERTY9
    C:*/none
    I: start/ok
    A: fine / ok; acceptable/ok
    R:  */NO_CONN+DISCONNECT+KEXINIT+UA_SUCCESS
:RE
```

## What is a counterexample ?
A counter example is a transition not respecting the restriction when it is applied.

