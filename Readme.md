# Mealy-verifier

This tool analyzes the Mealy machines of network protocol implementations.
Those Mealy machines are extracted thanks to active automata learning.
Mealy machines are expected to be dot files and be deterministic.


## Requirements

This tool is written in Rust. To install Rust please see the [official website](https://www.rust-lang.org/tools/install).

## Running the Mealy Verifier

```
Check property on transitions in mealy machine dot file

Usage: mealy_verifier [OPTIONS] --rules <RULES> [GRAPHS]...

Arguments:
  [GRAPHS]...  dot file to be verified

Options:
  -r, --rules <RULES>                  rules to check against the mealy machines
  -o, --output-folder <OUTPUT_FOLDER>  Output folder, if not provided a random name is chosen
  -h, --help                           Print help
  -V, --version                        Print version
```
For example to run the Mealy Verifier on the file *mealymachine.dot* with properties written in the file *properties*:
```sh
cargo run -r -- -r properties mealymachine.dot
```
or
```
mealy_verifier -r properties mealymachine.dot
```
A random output folder will be created. The name is printed as:
 ```
 Output folder is  result_xxx
```
To provide an output folder:
```sh
cargo run -r -- --r properties -o chosen_output_folder mealymachine.dot
```
The folder will be created by the program if it does not exist.
Moreover, Several dot files can be given at one time to the Mealy verifier.
```sh
cargo run -r -- -r properties mealymachine1.dot mealymachine2.dot
```

## Note on SSH
SSH mealy machines are extracted from [here](https://gitlab.science.ru.nl/pfiteraubrostean/Learning-SSH-Paper).
However for our tool to work on those we require to simplify graphviz options.
Also, we remove one edge to the two final states, to make them sink state
This is performed with the script in **ssh_models** and the script **clean.py**:
```
usage: clean.py [-h] path

Clean ssh dot file to make them readable by mealy verifier

positional arguments:
  path        path to dot file

optional arguments:
  -h, --help  show this help message and exit
```

## Note on OPC UA
OPC UA Mealy machines are in the folder **opcua_model**.  There are classified based on the mode used to obtain them:
- 3: Encryption and signature   
- 1: None  
The folders **opcua_mode_x** contain a list of folders that are the sha256 of the Mealy machine they contain.

```
└──  [Hash of dot file]
    ├── automata.dot
    └──  implem
```
- *automata.dot*: Mealy machine   
- *implem*: list of implementation with the same Mealy machine

## Properties
Properties are in the folder rules.

## Results
The results are in result folder within **opcua_mode_1**,**opcua_mode_3** and **result_ssh** subfolders.

## Publication
This repository is related to the publication
```
Mealy Verifier: An Automated, Exhaustive, and Explainable
Methodology for Analyzing State Machines in Protocol
Implementations
doi: 10.1145/3664476.3664506
published in: The 19th International Conference on Availability, Reliability and Security (ARES 2024), July 30-August 2, 2024, Vienna, Austria
```

## Additional information

Additional information regarding syntax and type of properties are availble in the info folder