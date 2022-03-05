package main

import (
	"encoding/binary"
	"flag"
	"fmt"
	"io/ioutil"
	"os"
	"sync"
	"time"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

const DB_PATH = "../all_5_states_undecided_machines_with_global_header"
const MAX_STATES = 5

func simulateAndGetRecordTimes(tm bbc.TM, timeLimit int, recordLimit int) (recordTimes [][]int) {

	recordTimes = append(recordTimes, []int{0})
	recordTimes = append(recordTimes, []int{0})

	currPos := 0
	nextPos := 0
	write := byte(0)
	currState := byte(1)
	currTime := 0

	tape := make(map[int]byte)

	var err error

	minRecord := 0
	maxRecord := 0

	for err == nil && currState > 0 && currState <= MAX_STATES && currTime < timeLimit && len(recordTimes[0]) <= recordLimit && len(recordTimes[1]) <= recordLimit {
		if _, ok := tape[currPos]; !ok {
			tape[currPos] = byte(0)
		}

		if currPos < minRecord {
			minRecord = currPos
			recordTimes[bbc.L] = append(recordTimes[bbc.L], currTime)
		}

		if currPos > maxRecord {
			maxRecord = currPos
			recordTimes[bbc.R] = append(recordTimes[bbc.R], currTime)
		}

		read := tape[currPos]
		write, currState, nextPos = bbc.TmStep(tm, read, currState, currPos, currTime)
		tape[currPos] = write
		currPos = nextPos
		currTime += 1
	}

	return recordTimes
}

// Having too little records in a long time is a hint of an exponential counter type of
// behavior. Typical values are <= 80 records for 10M steps.
func heuristicCountRecordTimes(tm bbc.TM, timeLimit int, recordLimit int) bool {
	recordTimes := simulateAndGetRecordTimes(tm, timeLimit, recordLimit)

	return len(recordTimes[0]) <= recordLimit && len(recordTimes[1]) <= recordLimit
}

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

	argTimeLimit := flag.Int("t", 10000000, "time limit for each machine to run")
	argRecordLimit := flag.Int("r", 200, "maximum of possible records on one side within time limit")
	argIndexFile := flag.String("f", "", "undecided index file to use")
	argMinIndex := flag.Int("m", 0, "min machine index to consider in seed database")
	argMaxIndex := flag.Int("M", bbc.TOTAL_UNDECIDED, "max machine index to consider in seed database")
	argNWorkers := flag.Int("n", 10000, "workers")

	flag.Parse()

	minIndex := *argMinIndex
	maxIndex := *argMaxIndex
	indexFileName := *argIndexFile
	timeLimit := *argTimeLimit
	recordLimit := *argRecordLimit
	nWorkers := *argNWorkers

	var undecidedIndex []byte
	if indexFileName != "" {
		undecidedIndex, err = ioutil.ReadFile(indexFileName)

		if err != nil {
			fmt.Println(err)
			os.Exit(-1)
		}
	}

	var runName = "output/heuristic-polynomial-passage-times-" + bbc.GetRunName() + "-t-" + fmt.Sprintf("%d", timeLimit) + "-r-" + fmt.Sprintf("%d", recordLimit)

	f, _ := os.OpenFile(runName,
		os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)

	var wg sync.WaitGroup

	startTime := time.Now()

	for i := 0; i < nWorkers; i += 1 {
		wg.Add(1)
		go func(iWorker int, nWorkers int) {
			k := 0
			if indexFileName == "" {
				for n := minIndex + iWorker; n < maxIndex; n += nWorkers {
					if k%1000 == 0 {
						fmt.Println(time.Since(startTime), "Worker: ", iWorker, "k: ", k)
					}
					m, err := bbc.GetMachineI(DB[:], n, true)
					if err != nil {
						fmt.Println("Err:", err, n)
					}
					if heuristicCountRecordTimes(m, timeLimit, recordLimit) {
						var arr [4]byte
						binary.BigEndian.PutUint32(arr[0:4], uint32(n))
						f.Write(arr[:])
					}
					k += 1
				}
			} else {
				for n := iWorker; n < len(undecidedIndex)/4; n += nWorkers {
					if k%1000 == 0 {
						fmt.Println(time.Since(startTime), "Worker: ", iWorker, "k: ", k)
					}
					m, indexInDb, err := bbc.GetMachineIFromIndex(DB[:], n, true, undecidedIndex[:])

					if indexInDb < uint32(minIndex) || indexInDb >= uint32(maxIndex) {
						continue
					}

					if err != nil {
						fmt.Println("Err:", err, n)
					}
					if heuristicCountRecordTimes(m, timeLimit, recordLimit) {
						var arr [4]byte
						binary.BigEndian.PutUint32(arr[0:4], indexInDb)
						f.Write(arr[:])
					}
					k += 1
				}
			}
			wg.Done()
		}(i, nWorkers)
	}

	wg.Wait()
	f.Close()
}
