package main

import (
	"encoding/binary"
	"fmt"
	"io/ioutil"
	"os"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

const DECIDED_INDEXES = "../bb5_decided_indexes/"

func main() {

	var decidedIndexes map[uint32]bool = make(map[uint32]bool)
	var undecidedIndexByte []byte

	items, _ := ioutil.ReadDir(DECIDED_INDEXES)
	for _, item := range items {
		if item.IsDir() || item.Name()[0] == '.' {
			continue
		} else {
			fmt.Println("Reading", item.Name())
			index, err := ioutil.ReadFile(DECIDED_INDEXES + item.Name())

			if err != nil {
				fmt.Println(err)
				os.Exit(-1)
			}

			for i := 0; i < len(index); i += 4 {
				machineIndex := binary.BigEndian.Uint32(index[i : i+4])
				decidedIndexes[machineIndex] = true
			}

			fmt.Println("Done with", item.Name())
		}
	}

	for i := uint32(0); i < bbc.TOTAL_UNDECIDED; i += 1 {
		if _, ok := decidedIndexes[i]; !ok {
			var buffer [4]byte
			binary.BigEndian.PutUint32(buffer[0:4], i)
			for _, theByte := range buffer {
				undecidedIndexByte = append(undecidedIndexByte, theByte)
			}
		}
	}

	fmt.Println("There are", len(undecidedIndexByte)/4, "undecided machines")
	err := os.WriteFile("output/bb5_undecided_index", undecidedIndexByte, 0644)
	if err != nil {
		fmt.Println(err)
		os.Exit(-1)
	}
}
