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
const INDEX_PATH = "../../bbchallenge-undecided-index/bb5_undecided_index"

type TapeType map[int]byte

type Configuration struct {
	Tape       TapeType
	maxTapePos int
	minTapePos int
	State      byte
	Head       int
}

func (c Configuration) toString() string {
	returnString := "State: "
	if c.Head >= c.minTapePos && c.Head <= c.maxTapePos {
		returnString += string(byte('A') + c.State - 1)
	} else {
		returnString += "*"
	}
	returnString += " ; "
	if c.Head < c.minTapePos {
		returnString += "[_"
	} else {
		returnString += " _"
	}
	for i := c.minTapePos; i <= c.maxTapePos; i += 1 {
		if i == c.Head+1 {
			returnString += "]"
		} else {
			if i == c.Head {
				returnString += "["
			} else {
				returnString += " "
			}
		}
		if symbol, exists := c.Tape[i]; exists {
			returnString += fmt.Sprint(symbol)
		} else {
			returnString += "."
		}
	}
	if c.Head == c.maxTapePos {
		returnString += "]_ "
	} else {
		if c.Head > c.maxTapePos {
			returnString += "[_]"

		} else {
			returnString += " _ "
		}
	}
	return returnString
}

func (c Configuration) toIndexString() string {
	returnString := fmt.Sprint(c.Head)
	if c.Head >= c.minTapePos && c.Head <= c.maxTapePos {
		returnString += string(byte('A') + c.State - 1)
	} else {
		returnString += "*"
	}
	for i := c.minTapePos; i <= c.maxTapePos; i += 1 {
		if symbol, exists := c.Tape[i]; exists {
			returnString += fmt.Sprint(symbol)
		} else {
			returnString += "."
		}
	}
	return returnString
}

func (c Configuration) fitsStart() bool {
	if c.State == 1 || c.Head < c.minTapePos || c.Head > c.maxTapePos {
		for i := c.minTapePos; i <= c.maxTapePos; i += 1 {
			if c.Tape[i] == 1 {
				return false
			}
		}
		return true
	}
	return false
}

func backwardTransition(config Configuration, changeFromState byte, read byte, write byte, direction byte, changeToState byte) *Configuration {

	// Going in the reversed direction
	reversedHeadMoveOffest := 1
	if direction == bbc.R {
		reversedHeadMoveOffest = -1
	}

	previousHeadPosition := config.Head + reversedHeadMoveOffest

	//no predecessor can use the halting transition
	if changeToState == 0 {
		return nil
	}

	//if the head is left of the tape segment we look at, then at some point before it must have left the segment to the left, so transitions to the right can't be a relevant predecessor
	if config.Head < config.minTapePos && direction == bbc.R {
		return nil
	}

	//if the head is right of the tape segment we look at, then at some point before it must have left the segment to the right, so transitions to the left can't be a relevant predecessor
	if config.Head > config.maxTapePos && direction == bbc.L {
		return nil
	}

	//if the head is on the tape segment we look at we require the transition to lead to the correct state
	if config.Head >= config.minTapePos && config.Head <= config.maxTapePos && changeToState != config.State {
		return nil
	}

	//if the transition doesn't write the correct symbol to an already defined position on the tape segment, then it can't be a predecessor by contradiction
	if symbol, exists := config.Tape[previousHeadPosition]; exists && symbol != write {
		return nil
	}

	var newTape TapeType = make(TapeType)
	for pos, value := range config.Tape {
		newTape[pos] = value
	}

	//define the position on the tape segment where the transition left from, unless it comes from outside of the segment
	if previousHeadPosition >= config.minTapePos && previousHeadPosition <= config.maxTapePos {
		newTape[previousHeadPosition] = read
	}

	previousConfiguration := Configuration{State: changeFromState, Tape: newTape, minTapePos: config.minTapePos, maxTapePos: config.maxTapePos, Head: previousHeadPosition}

	return &previousConfiguration
}

func deciderHaltingSegment(m bbc.TM, maxDistance int, nodeLimit int, recursiveMode bool, printRunInfo bool) bool {
	var stack []Configuration
	var seenConfigurations = make(map[string]int)

	//find the configuratuons that halt the next step
	//put them on the stack to start the search
	for i := 0; i < len(m)-2; i += 3 {
		my_new_state := m[i+2]
		starting_bit := byte((i / 3) % 2)
		if my_new_state == 0 {
			var initialTape TapeType = make(TapeType)
			initialTape[0] = starting_bit
			haltingConfiguration := Configuration{Tape: initialTape, State: byte(i/6 + 1), minTapePos: -maxDistance, maxTapePos: maxDistance, Head: 0}
			stack = append(stack, haltingConfiguration)
			seenConfigurations[haltingConfiguration.toIndexString()] = 1
			continue
		}
	}

	var configuration Configuration
	var maxDepth int
	// continue until we had to check too many nodes
	for nodes := 1; nodes <= nodeLimit; nodes += 1 {

		//we have found a nonhalting machine if all configurations have been successfully checked, i.e. all possible predecessors are in the list of seenConfigurations
		if len(stack) == 0 {
			if printRunInfo {
				fmt.Println("Proved nonhalting with segment size", maxDistance*2+1, "after expanding", nodes-1, "nodes")
			}
			return true
		}

		//configuration, stack = stack[0], stack[1:] //BFS search
		configuration, stack = stack[len(stack)-1], stack[:len(stack)-1] //DFS search

		depth := seenConfigurations[configuration.toIndexString()]

		if printRunInfo {
			fmt.Println(configuration.toString(), "; Node:", nodes, "; Depth:", depth)
		}

		if depth > maxDepth {
			maxDepth = depth
		}

		//test which of the possible transitions could have lead to the current configuration
		for i := 0; i < len(m)-2; i += 3 {

			try_backwards := backwardTransition(configuration, byte(i/6+1), byte((i/3)%2), m[i], m[i+1], m[i+2])
			if try_backwards != nil {
				//check if the possible predecessor could be the starting configuration
				//if yes, we have found a path that could have lead to the halting state, so this could be a halting machine
				if try_backwards.fitsStart() {
					if printRunInfo {
						fmt.Println("Start configuration possible with path of length", depth+1, "with segment size", maxDistance*2+1)
					}
					if recursiveMode {
						return deciderHaltingSegment(m, maxDistance+1, nodeLimit-nodes, recursiveMode, printRunInfo)
					} else {
						return false
					}
				}
				// add the predecessor configuration to the stack if it wasn't seen before
				if _, exists := seenConfigurations[try_backwards.toIndexString()]; !exists {
					stack = append(stack, *try_backwards)
					seenConfigurations[try_backwards.toIndexString()] = depth + 1
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

	argNodeLimit := flag.Int("t", 100, "number of nodes of the backwards tree to examine")
	argMinIndex := flag.Int("m", 0, "min machine index to consider in seed database")
	argMaxIndex := flag.Int("M", bbc.TOTAL_UNDECIDED, "max machine index to consider in seed database")
	argNWorkers := flag.Int("n", 4, "workers")
	argIndexFile := flag.String("f", INDEX_PATH, "undecided index file to use")

	flag.Parse()

	minIndex := *argMinIndex
	maxIndex := *argMaxIndex
	indexFileName := *argIndexFile
	nodeLimit := *argNodeLimit
	nWorkers := *argNWorkers

	runName := "output/halting-segment-" + bbc.GetRunName() + "-nodes-" + fmt.Sprintf("%d", nodeLimit) + "-minIndex-" + fmt.Sprintf("%d", minIndex) + "-maxIndex-" + fmt.Sprintf("%d", maxIndex)
	f, _ := os.OpenFile(runName,
		os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)

	var wg sync.WaitGroup

	var undecidedIndex []byte
	if indexFileName != "" {
		undecidedIndex, err = ioutil.ReadFile(indexFileName)

		if err != nil {
			fmt.Println(err)
			os.Exit(-1)
		}
	}

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
					if deciderHaltingSegment(m, 0, nodeLimit, true, false) {
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
						k += 1
						continue
					}

					if err != nil {
						fmt.Println("Err:", err, n)
					}
					if deciderHaltingSegment(m, 0, nodeLimit, true, false) {
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
