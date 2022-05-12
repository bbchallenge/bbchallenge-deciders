package main

import (
	"encoding/binary"
	"flag"
	"fmt"
	"io/ioutil"
	"os"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

const DECIDED_INDEXES = "../bb5_decided_indexes/"
const HEURISTIC_INDEXES = "../bb5_heuristically_decided_indexes/"

func main() {

	argHeuristic := flag.Bool("h", false, "apply heuristics as well as decider to get final index")

	flag.Parse()
	applyHeuristc := *argHeuristic

	var decidedIndexes map[uint32]string = make(map[uint32]string)
	var undecidedIndexByte []byte

	indexes_to_consider := []string{DECIDED_INDEXES}

	if applyHeuristc {
		indexes_to_consider = append(indexes_to_consider, HEURISTIC_INDEXES)
	}

	for i_case, index_folder := range indexes_to_consider {
		items, _ := ioutil.ReadDir(index_folder)
		for _, item := range items {
			if item.IsDir() || item.Name()[0] == '.' {
				continue
			} else {

				fmt.Println("Reading", item.Name())
				index, err := ioutil.ReadFile(index_folder + item.Name())
				fmt.Println(len(index)/4, "decided machines")
				if err != nil {
					fmt.Println(err)
					os.Exit(-1)
				}

				for i := 0; i < len(index); i += 4 {
					machineIndex := binary.BigEndian.Uint32(index[i : i+4])
					if _, ok := decidedIndexes[machineIndex]; ok && i_case == 0 {
						fmt.Println("Machine", machineIndex, "already there from", decidedIndexes[machineIndex])
						os.Exit(-1)
					}
					decidedIndexes[machineIndex] = item.Name()
				}

				fmt.Println("Done with", item.Name(), "\n")
			}
		}
	}

	fmt.Println("Decided index size", len(decidedIndexes))

	for i := uint32(0); i < bbc.TOTAL_UNDECIDED; i += 1 {
		if _, ok := decidedIndexes[i]; !ok {
			var buffer [4]byte
			binary.BigEndian.PutUint32(buffer[0:4], i)
			for _, theByte := range buffer {
				undecidedIndexByte = append(undecidedIndexByte, theByte)
			}
		}
	}

	outputFile := "output/bb5_undecided_index"
	if applyHeuristc {
		outputFile = "output/bb5_undecided_index_with_heuristics"
	}
	fmt.Println("There are", len(undecidedIndexByte)/4, "undecided machines our of", bbc.TOTAL_UNDECIDED)

	err := os.WriteFile(outputFile, undecidedIndexByte, 0644)

	if err != nil {
		fmt.Println(err)
		os.Exit(-1)
	}

}
