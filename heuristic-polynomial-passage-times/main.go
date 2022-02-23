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

func simulateAndGetPassageTimes(tm bbc.TM, timeLimit int) (passageTimes map[int][]int) {
	passageTimes = make(map[int][]int)
	currPos := 0
	nextPos := 0
	write := byte(0)
	currState := byte(1)
	currTime := 0

	tape := make(map[int]byte)

	var err error

	for err == nil && currState > 0 && currState <= MAX_STATES && currTime < timeLimit {
		if _, ok := tape[currPos]; !ok {
			tape[currPos] = byte(0)
		}

		passageTimes[currPos] = append(passageTimes[currPos], currTime)

		read := tape[currPos]
		write, currState, nextPos = bbc.TmStep(tm, read, currState, currPos, currTime)
		tape[currPos] = write
		currPos = nextPos
		currTime += 1
	}
	return passageTimes
}

func heuristicPolynomialPassageTimes(tm bbc.TM, timeLimit int, nbPointsToConclude int,
	maxTimeBehaviorBegin int, maxSamplingStep int, debug bool) bool {

	passageTimes := simulateAndGetPassageTimes(tm, timeLimit)

	for _, passage := range passageTimes {
		for samplingStep := 1; samplingStep < maxSamplingStep; samplingStep += 1 {
			if samplingStep >= len(passage) {
				break
			}
			subseq := bbc.SampleList(passage, 0, samplingStep)
			if subseq[0] < maxTimeBehaviorBegin {
				thirdDerivative := bbc.DiscreteDifference(subseq, 3)
				if len(thirdDerivative) > nbPointsToConclude {
					if bbc.AllZero(thirdDerivative[len(thirdDerivative)-1-nbPointsToConclude:]) {
						if debug {
							fmt.Println(samplingStep, thirdDerivative)
						}
						return true
					}
				}
			}
		}
	}

	return false
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

	argTimeLimit := flag.Int("t", 100000, "time limit for each machine to run")
	argNbPointsToConclude := flag.Int("c", 10, "numbers of points needed for the heuristic to conclude")
	argMaxTimeBehaviorBegin := flag.Int("b", 5, "maximum time for the polynomial behavior to begin")
	argMaxSamplingStep := flag.Int("k", 40, "maximum subsampling step used by the heuristic")
	argIndexFile := flag.String("f", "", "undecided index file to use")
	argMinIndex := flag.Int("m", 0, "min machine index to consider in seed database")
	argMaxIndex := flag.Int("M", bbc.TOTAL_UNDECIDED, "max machine index to consider in seed database")
	argNWorkers := flag.Int("n", 10000, "workers")

	flag.Parse()

	minIndex := *argMinIndex
	maxIndex := *argMaxIndex
	indexFileName := *argIndexFile
	timeLimit := *argTimeLimit
	nbPointsToConclude := *argNbPointsToConclude
	maxTimeBehaviorBegin := *argMaxTimeBehaviorBegin
	maxSamplingStep := *argMaxSamplingStep
	nWorkers := *argNWorkers

	var undecidedIndex []byte
	if indexFileName != "" {
		undecidedIndex, err = ioutil.ReadFile(indexFileName)

		if err != nil {
			fmt.Println(err)
			os.Exit(-1)
		}
	}

	var runName = "output/heuristic-polynomial-passage-times-" + bbc.GetRunName() + "-t-" + fmt.Sprintf("%d", timeLimit) + "-c-" + fmt.Sprintf("%d", nbPointsToConclude) +
		"-b-" + fmt.Sprintf("%d", maxTimeBehaviorBegin) + "-k-" + fmt.Sprintf("%d", maxSamplingStep)

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
					if heuristicPolynomialPassageTimes(m, timeLimit, nbPointsToConclude, maxTimeBehaviorBegin, maxSamplingStep, false) {
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
					if heuristicPolynomialPassageTimes(m, timeLimit, nbPointsToConclude, maxTimeBehaviorBegin, maxSamplingStep, false) {
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
