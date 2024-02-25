# Conditional Event

This property aims to verify if a given event is reached after some prerequisite are respected.
The event that should be met only when prerequisite are respected is called the action event.

## Prerequisite

### Definition

A prerequisite is a condition that must be respected.
It is composed of two events. 
The first one change the state of the prerequisites from False to True.
The second one is the opposite and is called counter event.
We note it:
```
event|counter event
```

### Several prerequisites

If more than one prerequisite is required to specify a property, the order is important. A prerequisite can become true only if previous ones are true. Furthermore, if a prerequisite is false, all following prerequisites become false.


## Syntax of the property


The syntax is:

```
CT:rule name
    first prerequisite
    second prerequisite
    action event
:CT
```
The property verify if event is reached at any the given index-th transition.
An example is :
```
CT:activation_after_creation
    create_session/*CreSesResOK*|close_session/*CloSesResOK*
    active_session_cert+active_session/*AcSesResOK*
:CT
```


## What is a counterexample ?

A counter example is a path on the Mealy machine that do not respect the prerequisite when the action is reached

## Output of the property

It may have several graphs as output, one per state that has an outgoing edge with the label corresponding to the action. Moreover, to ensure that every path from a state without ingoing edges (except cycle) to the state where the action is observed in those graphs is a counterexample, a state can be split into several states with the same name.


