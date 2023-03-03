package main

import (
	"encoding/binary"
	"flag"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"sort"
)

type TypeIndices []uint32

func (f TypeIndices) Len() int {
	return len(f)
}

func (f TypeIndices) Less(i, j int) bool {
	return f[i] < f[j]
}

func (f TypeIndices) Swap(i, j int) {
	f[i], f[j] = f[j], f[i]
}

func main() {

	argBelong := flag.Bool("b", false, "test if machine belongs to index")
	argIndexFileName := flag.String("f", "", "index of machines indices")
	argMachineIndex := flag.Int("m", 0, "machine index to test")
	argTestSorted := flag.Bool("ts", false, "test if machine index file is sorted")
	argSortFile := flag.Bool("s", false, "outputs sorted version of file")
	argTestSameContent := flag.String("sc", "", "tests that two files have same content modulo order")
	argToText := flag.Bool("txt", false, "outputs the machine index file in text")

	flag.Parse()

	belong := *argBelong
	indexFileName := *argIndexFileName
	machineIndex := *argMachineIndex
	testSorted := *argTestSorted
	sortFile := *argSortFile
	otherIndexFileName := *argTestSameContent
	toText := *argToText

	var err error
	var index []byte
	if indexFileName != "" {
		index, err = ioutil.ReadFile(indexFileName)
		if err != nil {
			fmt.Println(err)
			os.Exit(-1)
		}
	}

	var machineIndices TypeIndices
	for i := 0; i < len(index); i += 4 {
		machineIndices = append(machineIndices, binary.BigEndian.Uint32(index[i:i+4]))
	}

	if belong {
		for i := 0; i < len(index); i += 4 {
			if machineIndex == int(binary.BigEndian.Uint32(index[i:i+4])) {
				fmt.Println("Machine", machineIndex, "belongs to", indexFileName)
				os.Exit(0)
			}
		}
		fmt.Println("Machine", machineIndex, "does not belongs to", indexFileName)
		os.Exit(-1)
	}

	if testSorted {
		for i := 1; i < len(machineIndices); i += 1 {
			if machineIndices[i] < machineIndices[i-1] {
				fmt.Println("FAILED: index is not sorted!")
				os.Exit(-1)
			}
		}
		fmt.Println("SUCCESS: index is sorted")
		os.Exit(0)
	}

	if toText {
		for i := 0; i < len(machineIndices); i += 1 {
				fmt.Println(machineIndices[i])
		}
	}

	if sortFile {
		sort.Sort(machineIndices)
		fmt.Println("output/" + filepath.Base(indexFileName))
		f, _ := os.OpenFile("output/"+filepath.Base(indexFileName),
			os.O_CREATE|os.O_WRONLY, 0644)
		for i := 0; i < len(machineIndices); i += 1 {
			var arr [4]byte
			binary.BigEndian.PutUint32(arr[0:4], machineIndices[i])
			f.Write(arr[:])
		}
		f.Close()
	}

	if otherIndexFileName != "" {

		var other_index []byte

		other_index, err = ioutil.ReadFile(otherIndexFileName)
		if err != nil {
			fmt.Println(err)
			os.Exit(-1)
		}

		var otherMachineIndices TypeIndices
		for i := 0; i < len(other_index); i += 4 {
			otherMachineIndices = append(otherMachineIndices, binary.BigEndian.Uint32(other_index[i:i+4]))
		}

		if len(machineIndices) != len(otherMachineIndices) {
			fmt.Println("FAILED: different sizes!")
			os.Exit(-1)
		}

		var seen map[uint32]bool = make(map[uint32]bool)
		for i := 0; i < len(machineIndices); i += 1 {
			seen[machineIndices[i]] = true
		}

		for i := 0; i < len(otherMachineIndices); i += 1 {
			if ok, _ := seen[otherMachineIndices[i]]; !ok {
				fmt.Println("FAILED: content differs!")
				os.Exit(-1)
			}
		}

		fmt.Println("SUCCESS: same content!")
		os.Exit(0)

	}

}
