#!/bin/bash
if [ $# -ne 1 ];then
    echo "usage: $0 output_folder"
    exit 1
fi

OUTPUT_FOLDER=$1
DIR=$(readlink -f ${0%/*})
cd $DIR

OPCUA_MODE_1_DIR=./model/ready_to_verify/opcua_model/opcua_mode_1
OPCUA_MODE_3_DIR=./model/ready_to_verify/opcua_model/opcua_mode_3
OPCUA_RULE=./rules/opcua.opcua
SSH_MODEL="./model/ready_to_verify/ssh_models/"
SSH_RULE="rules/ssh"
EXEC="target/release/mealy_verifier"

OPCUA(){
	echo "Start OPC UA analysis"
    for i in $(ls $OPCUA_MODE_1_DIR);do
        $EXEC -r $OPCUA_RULE -o $OUTPUT_FOLDER   $OPCUA_MODE_1_DIR/$i/automata.dot
    done
    for i in $(ls $OPCUA_MODE_3_DIR);do
        $EXEC -r $OPCUA_RULE -o $OUTPUT_FOLDER   $OPCUA_MODE_3_DIR/$i/automata.dot
    done
	echo "End of OPC UA analysis"
}

SSH(){
	echo "Start SSH analysis"
    for i in $(ls $SSH_MODEL/*.dot);do
        $EXEC -r $SSH_RULE -o $OUTPUT_FOLDER   $i
    done
	echo "End  of SSH analysis"
}

#Compile
cargo build --release

OPCUA
SSH
