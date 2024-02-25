#!/usr/bin/env python3
import argparse
import parse

def clean(lines):
    clean_lines=[]
    clean_lines.append("digraph G{")
    for line in lines:
        if line=="""label=""""":
            continue
        if not ( "[" in line):
            continue
        label=''.join(r[0] for r in parse.findall("<td>{}</td>", line))
        #It is a transition
        if label!="":
            index=line.find("=")
            clean_line=line[:index+1]+'"'+label.replace("+","_")+'"'+"];"
            clean_lines.append(clean_line)
        else:
            clean_lines.append("")
            if parse.search("color=\"{}\"",line):
                index=line.find(",")
                clean_line=line[:index]+"];"
                clean_lines.append(clean_line)
            else:
                clean_lines.append(line.strip())
    clean_lines.append("}")
    return clean_lines



def read(path):
    try:
        with open(path,'r') as f:
            return f.readlines()
    except Exception as e:
        print("failed to read file:"+str(e))
        exit(1)

def merge_sink_states(lines):
    sinks_states=[]
    cpt=2
    for line in lines:
        if "->" not in line and "{" not in line  and "}"!=line.strip() and ""!=line:
            if cpt<=1:
                sinks_states.append(state)
            cpt=0
            state=parse.search("{}[",line)[0].strip()
        if "->" in line:
            temp=parse.search("{} -> {}[",line)
            src=temp[0].strip()
            dst=temp[1].strip()
            if src!=dst:
                cpt+=1
    ret_lines=["digraph G {"]
    inputs=set()
    sink_done=False
    for line in lines:
        if "->" in line:
            temp=parse.search("{} -> {}[",line)
            src=temp[0].strip()
            dst=temp[1].strip()
            if src in sinks_states:
                input_=parse.search("[label=\"{}/",line)[0].strip()
                if input_ in inputs:
                    continue
                else:
                    inputs.add(input_)
                if input_=="CH_OPEN":
                    index=line.find("/")
                    line=line[:index+1]+"CH_MAX\"];"
                if input_=="CH_CLOSE":
                    index=line.find("/")
                    line=line[:index+1]+"CH_NONE\"];"
                line=line.replace(src,"sink")
                line=line.replace(dst,"sink")
                ret_lines.append(line)
            elif dst in sinks_states:
                dst_index=line.find(">")
                line=line[:dst_index]+line[dst_index:].replace(dst,"sink")
                ret_lines.append(line)
            else:
                ret_lines.append(line)
        if "->" not in line and "{" not in line  and "}"!=line.strip() and ""!=line:
            state=parse.search("{}[",line)[0].strip()
            if state in sinks_states and len(inputs)==0:
                ret_lines.append("sink [label=\"sink\"];")
            elif state not in sinks_states:
                ret_lines.append(line)
    ret_lines.append("}")
    return ret_lines



def main(path):
    lines=read(path)
    clean_lines=clean(lines)
    merged_and_clean_lines=merge_sink_states(clean_lines)
    for line in merged_and_clean_lines:
        print(line)

if __name__=="__main__":
    
    parser=argparse.ArgumentParser(description="Clean ssh dot file to make them readable by mealy verifier")
    parser.add_argument('path', type=str,help="path to dot file")
    args=parser.parse_args()
    if args.path!=None:
        main(args.path)
    else:
        parser.print_help()
