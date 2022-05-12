package main

import (
	"encoding/binary"
	"flag"
	"fmt"
	"io/ioutil"
	"os"
	"strconv"
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

type Configuration struct {
	Tape  string
	State byte
	Head  int
}

func (c Configuration) toString() string {
	return c.Tape + string(c.State) + strconv.Itoa(c.Head)
}

func backwardTransition(config Configuration, write byte, read byte, direction byte, state byte) *Configuration {
	var new_tape string
	var new_head int
	if config.Head == 0 && direction == 0 {
		new_tape = string('0'+read) + config.Tape
		new_head = 0
	} else if config.Head == len(config.Tape)-1 && direction == 1 {
		new_tape = config.Tape + string('0'+read)
		new_head = config.Head + 1
	} else {
		new_head = config.Head - 1 + 2*int(direction)
		if config.Tape[new_head] != write {
			return nil
		}
		new_tape = config.Tape[:new_head] + string('0'+read) + config.Tape[new_head+1:]
	}
	return &Configuration{
		Tape:  new_tape,
		State: state,
		Head:  new_head,
	}
}

func deciderBackwardReasoning(m bbc.TM, transitionTreeDepthLimit int, debug bool) bool {
	var stack []Configuration
	var depthStack []int

	// map from state-1 to the mask of the ten Turing machine transitions that go to it
	var predecessors [5][10]bool

	// populate predecessors
	for i := byte(0); i < 10; i += 1 {
		my_new_state := m[3*i+2]
		starting_bit := string('0' + (i % 2))
		if my_new_state == 0 {
			stack = append(stack, Configuration{Tape: starting_bit, State: byte(i/2) + 1, Head: 0})
			depthStack = append(depthStack, 0)
			continue
		}
		predecessors[my_new_state-1][i] = true
	}

	var configuration Configuration

	seenConfigurations := map[string]bool{}
	// continue until all configurations have contradicted or one branch is too long
	for branches_searched := 0; len(stack) != 0; branches_searched += 1 {
		configuration, stack = stack[len(stack)-1], stack[:len(stack)-1]
		depth, depthStack := depthStack[len(depthStack)-1], depthStack[:len(depthStack)-1]

		// Fail if a branch gets longer than `transitionTreeDepthLimit`
		if depth > transitionTreeDepthLimit {
			return false
		}

		if _, found := seenConfigurations[configuration.toString()]; found {
			continue
		}

		seenConfigurations[configuration.toString()] = true

		my_preds := predecessors[configuration.State-1]

		for i := 0; i < 10; i += 1 {
			if !my_preds[i] {
				continue
			}

			transition := i * 3
			read := i % 2
			try_backwards := backwardTransition(configuration, m[transition], byte(read), m[transition+1], byte(i/2)+1)

			// add the predecessor configuration to the stack if valid
			if try_backwards != nil {
				stack = append(stack, *try_backwards)
				depthStack = append(depthStack, depth+1)
			}
		}
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

	debug := false

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
					if deciderBackwardReasoning(m, transitionTreeDepth, debug) {
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
					if deciderBackwardReasoning(m, transitionTreeDepth, debug) {
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
