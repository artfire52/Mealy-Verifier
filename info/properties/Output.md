# Output

This propety aim to restrict the outputs authorized with a given input.

## Syntax of the property
The syntax is:

```
OR:rule name 
    I:input
    O:authorized output
    O:authorized output 
:OR
```

An example is :
```
OR:malformed message
    I:attack
    O:Error
    O:Eof
:OR
```

## What is a counterexample ?
A counter example is a transition with an input matching the input of the property with an output not matching any given output in the property.

