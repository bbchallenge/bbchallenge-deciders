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

const MAX_MEMORY = 40000

type SymbolAndSeen struct {
	Symbol byte
	Seen   bool
}

type Tape [MAX_MEMORY]SymbolAndSeen

func tapeToStr(tape *Tape) (toRet string) {

	for i := MAX_MEMORY / 2; i >= 0; i -= 1 {
		if !tape[i].Seen {
			break
		}

		if tape[i].Symbol == 0 {
			toRet = "0" + toRet
		} else {
			toRet = "1" + toRet
		}
	}

	for i := MAX_MEMORY/2 + 1; i < len(tape); i += 1 {
		if !tape[i].Seen {
			break
		}

		if tape[i].Symbol == 0 {
			toRet = "0" + toRet
		} else {
			toRet = "1" + toRet
		}
	}

	return toRet
}

func argumentCyclers(tm bbc.TM, timeLimit int, spaceLimit int) bool {
	currPos := MAX_MEMORY / 2
	nextPos := 0
	write := byte(0)
	currState := byte(1)
	currTime := 0

	var tape Tape

	minPosSeen := MAX_MEMORY / 2
	maxPosSeen := MAX_MEMORY / 2

	// [state][read][tape][pos] -> bool
	var configSeen map[byte]map[byte]map[string]map[int]bool = make(map[byte]map[byte]map[string]map[int]bool)

	var err error

	for err == nil && currState > 0 && currState <= MAX_STATES {

		if _, ok := configSeen[currState]; !ok {
			configSeen[currState] = make(map[byte]map[string]map[int]bool)
		}

		minPosSeen = bbc.MinI(minPosSeen, currPos)
		maxPosSeen = bbc.MaxI(maxPosSeen, currPos)

		tape[currPos].Seen = true
		read := tape[currPos].Symbol

		if _, ok := configSeen[currState][read]; !ok {
			configSeen[currState][read] = make(map[string]map[int]bool)
		}

		tapeStr := tapeToStr(&tape)

		if _, ok := configSeen[currState][read][tapeStr]; !ok {
			configSeen[currState][read][tapeStr] = make(map[int]bool)
		}

		//fmt.Println(currState, read, tapeStr)
		_, ok := configSeen[currState][read][tapeStr][currPos]

		if ok {
			//fmt.Println(currTime, currState, read, tapeStr)
			return true
		}

		configSeen[currState][read][tapeStr][currPos] = true

		write, currState, nextPos = bbc.TmStep(tm, read, currState, currPos, currTime)

		tape[currPos].Symbol = write
		currPos = nextPos

		if maxPosSeen-minPosSeen > spaceLimit {
			return false
		}

		if currTime > timeLimit {
			return false
		}

		currTime += 1

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

	runName := "output/" + bbc.GetRunName() + "-time-" + fmt.Sprintf("%d", timeLimit) + "-space-" + fmt.Sprintf("%d", spaceLimit) + "-minIndex-" + fmt.Sprintf("%d", minIndex) + "-maxIndex-" + fmt.Sprintf("%d", maxIndex)
	f, _ := os.OpenFile(runName,
		os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)

	var wg sync.WaitGroup

	startTime := time.Now()

	for i := 0; i < nWorkers; i += 1 {
		wg.Add(1)
		go func(iWorker int, nWorkers int) {
			k := 0
			for n := minIndex + iWorker; n < maxIndex; n += nWorkers {
				if k%1000 == 0 {
					fmt.Println(time.Since(startTime), "Worker: ", iWorker, "k: ", k)
				}
				m, err := bbc.GetMachineI(DB[:], n, true)
				if err != nil {
					fmt.Println("Err:", err, n)
				}
				if argumentCyclers(m, timeLimit, spaceLimit) {
					var arr [4]byte
					binary.BigEndian.PutUint32(arr[0:4], uint32(n))
					f.Write(arr[:])
				}
				k += 1
			}
			wg.Done()
		}(i, nWorkers)
	}

	wg.Wait()
	f.Close()
}
