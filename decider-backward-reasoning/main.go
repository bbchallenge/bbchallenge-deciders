package main

import (
	"encoding/binary"
	"errors"
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

const MAX_MEMORY = 1000

type SymbolAndSeen struct {
	Symbol byte
	Seen   bool
}

type Tape [MAX_MEMORY]SymbolAndSeen

// Example:
// s    0    1
// ---  ---  ---
// A    1RB  0RC
// B    1LC  1LB
// C    1RA  1LD
// D    0RE  1LA
// E    ???  0LB

// E . . >0 . .
// D . >0 0 . .
// C . 0 >1 . . Contradiction!

// 0 1 2 3 4
// A B C D E

func deciderBackwardReasoningRec(m bbc.TM, transitionTreeDepth int, currDepth int, currState byte, currRead byte, currPos int, currTape Tape, printTrace bool) (bool, error) {
	if printTrace {
		fmt.Println(currDepth, string('A'+(currState-1)), currRead, currPos, currDepth)
	}

	if currDepth == transitionTreeDepth {
		return false, nil
	}

	if currPos < 0 || currPos >= len(currTape) {
		return false, errors.New("tape memory exceeded")
	}

	// We spotted a backward reasoning contradiction
	if currTape[currPos].Seen == true && currTape[currPos].Symbol != currRead {
		return true, nil
	}

	currTape[currPos].Symbol = currRead
	currTape[currPos].Seen = true

	for iTransition := byte(0); iTransition < 10; iTransition += 1 {
		//write := m[3*iTransition]
		move := m[3*iTransition+1]
		gotoState := m[3*iTransition+2]

		// That transition points to us eg. (A,0) is "1LE" and currState is E
		if gotoState == currState {
			newState := iTransition / 2
			newRead := iTransition % 2
			newPos := currPos + 1
			// Moving backward so in opposite direction
			if move == bbc.R {
				newPos = currPos - 1
			}
			result, err := deciderBackwardReasoningRec(m, transitionTreeDepth, currDepth+1, newState+1, newRead, newPos, currTape, printTrace)

			if !result || err != nil {
				return false, err
			}
		}
	}

	return true, nil
}

func deciderBackwardReasoning(m bbc.TM, transitionTreeDepth int, printTrace bool) bool {

	for iTransition := byte(0); iTransition < 10; iTransition += 1 {
		gotoState := m[3*iTransition+2]

		if gotoState == 0 {
			newState := iTransition / 2
			newRead := iTransition % 2
			var tape Tape
			result, err := deciderBackwardReasoningRec(m, transitionTreeDepth, 0, newState+1, newRead, MAX_MEMORY/2, tape, printTrace)

			if !result || err != nil {
				return false
			}
		}
	}

	return true
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

	argTransitionTreeDepth := flag.Int("d", 5, "depth of backward reasoning tree")
	argIndexFile := flag.String("f", "", "undecided index file to use")
	argMinIndex := flag.Int("m", 0, "min machine index to consider in seed database")
	argMaxIndex := flag.Int("M", bbc.TOTAL_UNDECIDED, "max machine index to consider in seed database")
	argNWorkers := flag.Int("n", 10000, "workers")

	flag.Parse()

	minIndex := *argMinIndex
	maxIndex := *argMaxIndex
	indexFileName := *argIndexFile
	transitionTreeDepth := *argTransitionTreeDepth
	nWorkers := *argNWorkers

	var undecidedIndex []byte
	if indexFileName != "" {
		undecidedIndex, err = ioutil.ReadFile(indexFileName)

		if err != nil {
			fmt.Println(err)
			os.Exit(-1)
		}
	}

	// fmt.Println(minIndex, maxIndex, timeLimit, spaceLimit, nWorkers)

	// n := 7888060 // not a translated cycler
	// m, _ := bbc.GetMachineI(DB[:], n, true)

	// fmt.Println(argumentTranslatedCyclers(m, timeLimit, spaceLimit))

	var runName string

	runName = "output/backward-reasoning-" + bbc.GetRunName() + "-depth-" + fmt.Sprintf("%d", transitionTreeDepth) + "-minIndex-" + fmt.Sprintf("%d", minIndex) + "-maxIndex-" + fmt.Sprintf("%d", maxIndex)

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
					if deciderBackwardReasoning(m, transitionTreeDepth, false) {
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
					if deciderBackwardReasoning(m, transitionTreeDepth, false) {
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
