package main

import (
	"encoding/binary"
	"flag"
	"fmt"
	"io/ioutil"
	"math"
	"os"
	"sync"
	"time"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

const DB_PATH = "../all_5_states_undecided_machines_with_global_header"
const MAX_STATES = 5
const INDEX_PATH = "../../bbchallenge-undecided-index/bb5_undecided_index"

type TapePosition struct {
	Symbol       byte
	LastTimeSeen int
	Seen         bool
}

func tapeSegment(tape []TapePosition, start int, end int) (segment string) {
	for i := start; i <= end; i += 1 {
		if tape[i].Symbol == 0 {
			segment += "0"
		} else {
			segment += "1"
		}
	}
	return segment
}

func invertMachine(tm *bbc.TM) {
	for i, v := range tm {
		if i%3 == 1 {
			switch v {
			case bbc.L:
				tm[i] = bbc.R
			case bbc.R:
				tm[i] = bbc.L
			}
		}
	}
}

type CheckerState struct {
	Phase              int
	UturnLeftSide      string
	UturnRightSide     string
	Buffer1            string
	Buffer2            string
	Increment1         string
	Increment2         string
	State1             byte
	State2             byte
	UturnLeftSideSize  int
	UturnRightSideSize int
	BufferSize         int
	IncrementSize      int
}

func (c *CheckerState) checkRightBouncers(tape []TapePosition, currState byte, currPos int, minPos int, maxPos int) {
	switch c.Phase {
	case 0:
		if maxPos-minPos >= 3 && currPos <= (minPos+maxPos)/2 {
			//we have seen 4 or more cells and the TM head is in the first half of the visited tape
			//if the TM is a unilateral bouncer to the right that has to happen between bounces once the repeated section grows big enough
			//
			//doing it like this does mean that we pick up machines like 7866044 a bit later than we could, since we only start testing after it has been bouncing for a while
			//there are different ways we could pick starting spots as well as head and buffer size with different advantages. In practice it does not seem to matter much
			//
			c.UturnRightSideSize = (maxPos - minPos) / 3 //pick some growing value for the head and buffer size. We will pick up bouncers even if those values are bigger than necessary
			c.BufferSize = (maxPos - minPos) / 3         //here we make sure the chosen values get big enough eventually, but we can still begin testing early
			c.Phase = 1                                  //begin testing for a bounce
			//fmt.Printf("finished phase 0 --- %+v\n", c)
		}
	case 1:
		//the checker stays in phase 1 as long as the TM stays on the Base and Buffer segments of the tape
		if currPos == maxPos-c.UturnRightSideSize+1 {
			//the head entered the Head segment:
			//  ...000)(Base)(Buffer1)(Head)(000...
			//                        ^
			//                      State1
			c.UturnLeftSideSize = currPos - c.BufferSize - minPos
			c.UturnLeftSide = tapeSegment(tape, minPos, minPos+c.UturnLeftSideSize)
			c.Buffer1 = tapeSegment(tape, currPos-c.BufferSize, currPos-1)
			c.State1 = currState
			c.UturnRightSide = tapeSegment(tape, currPos, maxPos)
			c.Phase = 2
			//fmt.Printf("finished phase 1 --- %+v\n", c)
		}
	case 2:
		//the checker stays in phase 2 as long as the TM stays on the Buffer and Head segments of the tape, moving those to the right as maxPos grows
		if currPos == maxPos-c.UturnRightSideSize-c.BufferSize {
			//the head entered the Increment segment:
			//  ...000)(Base)(Increment1)(Buffer2)(Head)(000...
			//                          ^
			//                        State2
			//if this is a valid bounce then (Base) and (Head) have to be the same as before. There was no opportunity to change (Base), so just check (Head)
			if c.UturnRightSide == tapeSegment(tape, maxPos-c.UturnRightSideSize+1, maxPos) {
				c.IncrementSize = (currPos + 1) - (minPos + c.UturnLeftSideSize)
				if c.IncrementSize > 0 {
					c.Increment1 = tapeSegment(tape, minPos+c.UturnLeftSideSize, currPos)
					c.State2 = currState
					c.Buffer2 = tapeSegment(tape, currPos+1, currPos+c.BufferSize)
					c.Phase = 3
					//fmt.Printf("finished phase 2 --- %+v\n", c)
				} else {
					c.Phase = 0
					//fmt.Println("failed phase 2 --- no growth")
				}
			} else {
				c.Phase = 0
				//fmt.Printf("failed phase 2 --- %+v\n", c)
			}
		}
	case 3:
		//the checker stays in phase 3 as long as the TM stays on the Buffer and Increment segments of the tape
		if currPos == minPos+c.UturnLeftSideSize-1 {
			//the head entered the Base segment:
			//  ...000)(Base)(Buffer2)(Increment2)(Head)(000...
			//              ^
			//            State2
			//if this is a valid bounce then (Base), (Head), (Buffer2) and (State2) have to be the same as before. There was no opportunity to change (Base) or (Head)
			if c.State2 == currState && c.Buffer2 == tapeSegment(tape, currPos+1, currPos+c.BufferSize) {
				c.Increment2 = tapeSegment(tape, currPos+c.BufferSize+1, maxPos-c.UturnRightSideSize)
				c.Phase = 4
				//fmt.Printf("finished phase 3 --- %+v\n", c)
			} else {
				c.Phase = 0
				//fmt.Printf("failed phase 3 --- %+v\n", c)
			}
		}
		if currPos == maxPos-c.UturnRightSideSize+1 {
			c.Phase = 0
			//fmt.Println("failed phase 3 --- wrong exit direction")
		}
	case 4:
		//the checker stays in phase 4 as long as the TM stays on the Buffer and Base segments of the tape
		if currPos == minPos+c.UturnLeftSideSize+c.BufferSize {
			//the head entered the Increment segment:
			//  ...000)(Base)(Buffer1)(Increment2)(Head)(000...
			//                        ^
			//                      State1
			//if this is a valid bounce then (Base), (Head), (Buffer1), (Increment2) and (State1) have to be the same as before. There was no opportunity to change (Increment2) or (Head)
			if c.State1 == currState && c.UturnLeftSide == tapeSegment(tape, minPos, currPos-c.BufferSize) && c.Buffer1 == tapeSegment(tape, minPos+c.UturnLeftSideSize, currPos-1) {
				c.Phase = 5
				//fmt.Printf("finished phase 4 --- %+v\n", c)
			} else {
				c.Phase = 0
				//fmt.Printf("failed phase 4 --- %+v\n", c)
			}
		}
		if maxPos-minPos > c.UturnLeftSideSize+c.UturnRightSideSize+c.IncrementSize+c.BufferSize {
			//the visited tape is growing while we are on the left side. We do not handle that here and have to start our checks from the beginning
			c.Phase = 0
			//fmt.Println("failed phase 4 --- wrong exit direction")
		}
	case 5:
		//the checker stays in phase 5 as long as the TM stays on the Buffer and Increment segments of the tape
		if currPos == maxPos-c.UturnRightSideSize+1 {
			//the head entered the Head segment:
			//  ...000)(Base)(Increment1)(Buffer1)(Head)(000...
			//                                    ^
			//                                  State1
			//if this is a valid bounce then (Base), (Head), (Buffer1), (Increment1) and (State1) have to be the same as before. There was no opportunity to change (Base) or (Head)
			if c.State1 == currState && c.Increment1 == tapeSegment(tape, minPos+c.UturnLeftSideSize, currPos-c.BufferSize-1) && c.Buffer1 == tapeSegment(tape, currPos-c.BufferSize, currPos-1) {
				c.Phase = 6
				//fmt.Printf("finished phase 5 --- %+v\n", c)
				//fmt.Println("unilateral-bouncer-right detected")
			} else {
				c.Phase = 0
				//fmt.Printf("failed phase 5 --- %+v\n", c)
			}
		}
		if currPos == minPos+c.UturnLeftSideSize-1 {
			c.Phase = 0
			//fmt.Println("failed phase 5 --- wrong exit direction")
		}
	}
}

var irecord int
var irecord_index uint32
var trecord int
var trecord_index uint32
var srecord int
var srecord_index uint32

func argumentUnilateralBouncers(tm bbc.TM, indexInDb uint32, timeLimit int, spaceLimit int, invertTM bool, reportRecords bool) bool {

	if invertTM {
		invertMachine(&tm)
	}

	tapeMemory := 2 * spaceLimit

	var tape []TapePosition = make([]TapePosition, tapeMemory)

	currPos := tapeMemory / 2
	currState := byte(1)
	currTime := 0

	minPosSeen := tapeMemory / 2
	maxPosSeen := tapeMemory / 2

	nextPos := currPos
	write := byte(0)

	var err error

	var rightChecker CheckerState

	for err == nil && currState > 0 && currState <= MAX_STATES {

		if currTime > timeLimit {
			return false
		}

		if maxPosSeen-minPosSeen > spaceLimit || currPos < 0 || currPos >= len(tape) {
			return false
		}

		rightChecker.checkRightBouncers(tape, currState, currPos, minPosSeen, maxPosSeen)
		if rightChecker.Phase == 6 {
			if reportRecords {
				if irecord < len(rightChecker.Increment1) {
					irecord = len(rightChecker.Increment1)
					irecord_index = indexInDb
					fmt.Println("Record Increment Size:", irecord, "by", irecord_index)
				}
				if trecord < currTime {
					trecord = currTime
					trecord_index = indexInDb
					fmt.Println("Record Detection Time:", trecord, "by", trecord_index)
				}
				if srecord < maxPosSeen-minPosSeen {
					srecord = maxPosSeen - minPosSeen
					srecord_index = indexInDb
					fmt.Println("Record Detection Space:", srecord, "by", srecord_index)
				}
			}
			return true
		}

		read := tape[currPos].Symbol
		//fmt.Println(currTime, currState, read, tapeSegment(tape, (tapeMemory / 2) - 2, (tapeMemory / 2) + 23))

		tape[currPos].Seen = true
		tape[currPos].LastTimeSeen = currTime

		write, currState, nextPos = bbc.TmStep(tm, read, currState, currPos, currTime)

		tape[currPos].Symbol = write

		currPos = nextPos
		currTime += 1

		minPosSeen = bbc.MinI(minPosSeen, currPos)
		maxPosSeen = bbc.MaxI(maxPosSeen, currPos)
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

	argTimeLimit := flag.Int("t", 100000, "time limit")
	argSpaceLimit := flag.Int("s", 500, "space limit")
	argMinIndex := flag.Int("m", 0, "min machine index to consider in seed database")
	argMaxIndex := flag.Int("M", bbc.TOTAL_UNDECIDED, "max machine index to consider in seed database")
	argNWorkers := flag.Int("n", 1000, "workers")
	argIndexFile := flag.String("f", INDEX_PATH, "undecided index file to use")
	argInvertTM := flag.Bool("i", false, "invert the L and R direction of the machine to detect bouncers to the left")
	argReportRecords := flag.Bool("r", false, "report records for time, space and increment size")

	flag.Parse()

	minIndex := *argMinIndex
	maxIndex := *argMaxIndex
	timeLimit := *argTimeLimit
	spaceLimit := *argSpaceLimit
	nWorkers := *argNWorkers
	indexFileName := *argIndexFile
	invertTM := *argInvertTM
	reportRecords := *argReportRecords

	direction := "right-"
	if invertTM {
		direction = "left-"
	}
	runName := "output/unilateral-bouncers-" + direction + bbc.GetRunName() + "-time-" + fmt.Sprintf("%d", timeLimit) + "-space-" + fmt.Sprintf("%d", spaceLimit) + "-minIndex-" + fmt.Sprintf("%d", minIndex) + "-maxIndex-" + fmt.Sprintf("%d", maxIndex)
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
					if argumentUnilateralBouncers(m, math.MaxUint32, timeLimit, spaceLimit, invertTM, reportRecords) {
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
					if argumentUnilateralBouncers(m, indexInDb, timeLimit, spaceLimit, invertTM, reportRecords) {
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

	if reportRecords {
		fmt.Println("Record Increment Size: ", irecord, " by ", irecord_index)
		fmt.Println("Record Detection Time: ", trecord, " by ", trecord_index)
		fmt.Println("Record Detection Space: ", srecord, " by ", srecord_index)
	}
}
