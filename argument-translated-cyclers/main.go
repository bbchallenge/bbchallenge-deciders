package main

import (
	"flag"
	"fmt"
	"io/ioutil"
	"os"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

const DB_PATH = "../all_5_states_undecided_machines_with_global_header"

func main() {

	DB, err := ioutil.ReadFile(DB_PATH)

	if err != nil {
		fmt.Println(err)
		os.Exit(-1)
	}

	err = bbc.TestDB(DB[:], true)
	if err != nil {
		fmt.Println(err)
		os.Exit(-1)
	}

	DB_SIZE := (len(DB) / 30) - 1
	fmt.Println(DB_SIZE)

	argTimeLimit := flag.Int("t", 1000, "time limit")
	argSpaceLimit := flag.Int("s", 500, "space limit")
	argMinIndex := flag.Int("m", 0, "min machine index to consider in seed database")
	argMaxIndex := flag.Int("M", bbc.TOTAL_UNDECIDED_TIME, "max machine index to consider in seed database")
	argNWorkers := flag.Int("n", 1000, "workers")

	flag.Parse()

	minIndex := *argMinIndex
	maxIndex := *argMaxIndex
	timeLimit := *argTimeLimit
	spaceLimit := *argSpaceLimit
	nWorkers := *argNWorkers

	fmt.Println(minIndex, maxIndex, timeLimit, spaceLimit, nWorkers)

}
