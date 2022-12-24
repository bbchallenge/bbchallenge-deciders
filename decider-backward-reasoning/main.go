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

const MAX_MEMORY = 1000

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

type TapeType map[int]byte

type ConfigurationAndDepth struct {
	Tape       TapeType
	maxTapePos int
	minTapePos int
	State      byte
	Head       int
	Depth      int
}

func backwardTransition(config ConfigurationAndDepth, write byte, read byte, direction byte, state byte) *ConfigurationAndDepth {

	// Going in the reversed direction
	reversedHeadMoveOffest := 1
	if direction == bbc.R {
		reversedHeadMoveOffest = -1
	}

	previousHeadPosition := config.Head + reversedHeadMoveOffest

	maxHeadPos := config.maxTapePos
	minHeadPos := config.minTapePos

	if previousHeadPosition < minHeadPos {
		minHeadPos = previousHeadPosition
	} else if previousHeadPosition > maxHeadPos {
		maxHeadPos = previousHeadPosition
	} else {
		if config.Tape[previousHeadPosition] != write {
			return nil
		}
	}

	var newTape TapeType = make(TapeType)
	for pos, value := range config.Tape {
		newTape[pos] = value
	}
	newTape[previousHeadPosition] = read
	previousConfiguration := ConfigurationAndDepth{State: state, Tape: newTape, minTapePos: minHeadPos, maxTapePos: maxHeadPos, Head: previousHeadPosition, Depth: config.Depth + 1}

	return &previousConfiguration
}

var globalMaxDepth int
var globalMaxDepthMachineId int
var mutexMaxDepth sync.Mutex

func deciderBackwardReasoning(m bbc.TM, machineId int, transitionTreeDepthLimit int, printRunInfo bool, computeGlobalMaxDepth bool) bool {
	var stack []ConfigurationAndDepth

	// map from state-1 to the mask of the ten Turing machine transitions that go to it
	var predecessors [5][10]bool

	// populate predecessors
	for i := byte(0); i < 10; i += 1 {
		my_new_state := m[3*i+2]
		starting_bit := i % 2
		var initialTape TapeType = make(TapeType)
		initialTape[0] = starting_bit
		if my_new_state == 0 {
			stack = append(stack, ConfigurationAndDepth{Tape: initialTape, State: byte(i/2) + 1, Head: 0, Depth: 0})
			continue
		}
		predecessors[my_new_state-1][i] = true
	}

	var configurationAndDepth ConfigurationAndDepth
	var maxDepth int

	// continue until all configurations have contradicted or one branch is too long
	for branches_searched := 0; len(stack) != 0; branches_searched += 1 {

		configurationAndDepth, stack = stack[len(stack)-1], stack[:len(stack)-1]
		depth := configurationAndDepth.Depth

		if printRunInfo {
			fmt.Println("State:", configurationAndDepth.State, ";", "Head:", configurationAndDepth.Head, ";")
		}

		if depth > maxDepth {
			maxDepth = depth
		}

		// Fail if a branch gets longer than `transitionTreeDepthLimit`
		if depth > transitionTreeDepthLimit {
			return false
		}

		my_preds := predecessors[configurationAndDepth.State-1]

		for i := 0; i < 10; i += 1 {
			if !my_preds[i] {
				continue
			}

			transition := i * 3
			read := i % 2
			try_backwards := backwardTransition(configurationAndDepth, m[transition], byte(read), m[transition+1], byte(i/2)+1)

			// add the predecessor configuration to the stack if valid
			if try_backwards != nil {
				stack = append(stack, *try_backwards)
			}
		}
	}

	if printRunInfo {
		fmt.Println(maxDepth)
	}

	if computeGlobalMaxDepth && len(stack) == 0 && maxDepth > globalMaxDepth {
		mutexMaxDepth.Lock()
		globalMaxDepth = maxDepth
		globalMaxDepthMachineId = machineId
		mutexMaxDepth.Unlock()
	}

	return len(stack) == 0
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
	argNWorkers := flag.Int("n", 1000, "workers")
	argReportMaxDepth := flag.Bool("r", false, "report max bactracking depth met in this batch of machines")

	flag.Parse()

	minIndex := *argMinIndex
	maxIndex := *argMaxIndex
	indexFileName := *argIndexFile
	transitionTreeDepth := *argTransitionTreeDepth
	nWorkers := *argNWorkers
	reportMaxDepth := *argReportMaxDepth

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
					if deciderBackwardReasoning(m, n, transitionTreeDepth, false, reportMaxDepth) {
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
					if deciderBackwardReasoning(m, int(indexInDb), transitionTreeDepth, false, reportMaxDepth) {
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

	if reportMaxDepth {
		fmt.Println("Max depth:", globalMaxDepth)
		fmt.Println("Reach by machine with id:", globalMaxDepthMachineId)
	}
}
