# Sink as Target

This propety aim to verify if a given event is leading to sink states matching a given description.

# Sink state definition

A sink state is an state where every outgoing transitions are leadging to the state itself.
In the following exmaple, the sink state is the state labeled "sink".
```


          ┌──────────────┐
          │    start     │
          └──────────────┘
            │                 e/f        g/h
            │ start/start   ┌─────┐    ┌─────┐
            ▼               ▼     │    ▼     │
    c/d   ┌────────────────────────────────────┐   a/b
  ┌────── │                                    │ ──────┐
  │       │                sink                │       │
  └─────▶ │                                    │ ◀─────┘
          └────────────────────────────────────┘



```

A sink state is identified by its outgoing transition. 
Hence, the Mealy verifier uses a list of events as a description of a sink node.

## Syntax of the property
The syntax is:
```
ST:Rule name
        trigger event |event;event
:ST
```
**event;event** is the description of the sink state. 

An example is :
```
ST:test
        e/ok|a/no_resp;c/no_resp;e/nok
:ST
```

## What is a counterexample ?
A counter example is a transition matching the trigger event that do not lead to a sink state matching the given description.











