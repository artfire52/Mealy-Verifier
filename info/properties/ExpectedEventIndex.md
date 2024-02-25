# Expected Event Index

This propety aim to verify is an event is at a precise place in the Mealy machine.


## Syntax of the property
The syntax is:

```
ETI:rule name 
    event
    index
:ETI
```
The property verify if event is reached at any the given index-th transition.
An example is :
```
ETI:Hello_first
    hello/Ack
    0
:ETI
```

## What is a counterexample ?
A counterexample is a transition that does not respect the sequence.

