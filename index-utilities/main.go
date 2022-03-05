package main

import (
	"encoding/binary"
	"flag"
	"fmt"
	"io/ioutil"
	"os"
)

func main() {

	argBelong := flag.Bool("b", true, "test if machine belongs to index")
	argIndexFileName := flag.String("f", "", "index of machines indices")
	argMachineIndex := flag.Int("m", 0, "machine index to test")

	flag.Parse()

	belong := *argBelong
	indexFileName := *argIndexFileName
	machineIndex := *argMachineIndex

	var err error
	var index []byte
	if indexFileName != "" {
		index, err = ioutil.ReadFile(indexFileName)
		if err != nil {
			fmt.Println(err)
			os.Exit(-1)
		}
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

}
