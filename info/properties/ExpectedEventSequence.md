# Expected Event Sequence

This propety aim to verify is a sequence of message is respected.
The sequence start when the input of the first event is matched.


## Syntax of the property
The syntax is:

```
ETS:rule name 
        I: starting event
        E: ending event
        first event of the sequence
        second event of the sequence
        Ig: eventS ignored
:ETS
```

The initial event is optional and it indicates when the property should be verified.
Hence, the property is checked after reaching this event.
The ending event indicate when the sequence should not be verified anymore.
The ignore events are the events that would not count as violation if they are reached instead of the event (usefull when input are ignored).


An example is :
```
ETS:start connection
        Hello/ack
        KeyExchange/KeyExchange
        Ig: malformed message/no response
:ETS
```

Here is an example of a Mealy machine with events to ignore. 

```
┌──────────────────────────┐
│            0             │
└──────────────────────────┘
  │
  │ Hello/Ack
  ▼
┌──────────────────────────┐   malformed message/no response
│                          │ ────────────────────────────────┐
│            1             │                                 │
│                          │ ◀───────────────────────────────┘
└──────────────────────────┘
  │
  │ KeyExchange/KeyExchange
  ▼
┌──────────────────────────┐
│            2             │
└──────────────────────────┘
```

## What is a counterexample ?
A counterexample is a transition that does not respect the sequence.

