# What is an event ?

A Mealy machine is a labeled desterminic finite automata. An event is 
a pair **i/o** where i in an input and o an output. 

The input and the output are written using input and output alphabet of the Melay machine with an extended syntex.

# Extend syntax of event

## Wildcards

There is two authorized wildcars:
- \* is used to match any pattern.
- ? is used to match one character.

## Or operator

We can write **A+B** which mean either letter **A** or letter **B**.

## Negation

We can write **!A** to express any letter that is not **A**.

## Nor operator

To combine Or operation and negation operation we can use
**!(A#B)**. This operation means any letter that is not **A** and not **B**.

## EventS

We write several event in a rwo using a list with ';' seperating event.
EventS is a vector of event.