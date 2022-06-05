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

type TapePosition struct {
	Symbol       byte
	LastTimeSeen int
	Seen         bool
}

type Record struct {
	Tape     []TapePosition
	Time     int
	Position int
}

func recordsAreEquivalent(minSide bool, pastRecord *Record, currRecord *Record) bool {
	offset := 0

	//fmt.Println("\t", tapeRepr(pastRecord.Tape, pastRecord.Position, 10))
	//fmt.Println("\t", tapeRepr(currRecord.Tape, currRecord.Position, 10))

	for {

		//fmt.Println(offset)

		if pastRecord.Position+offset < 0 || pastRecord.Position+offset >= len(currRecord.Tape) {
			break
		}

		if !currRecord.Tape[pastRecord.Position+offset].Seen || currRecord.Tape[pastRecord.Position+offset].LastTimeSeen < pastRecord.Time {
			break
		}

		if currRecord.Tape[currRecord.Position+offset].Symbol != pastRecord.Tape[pastRecord.Position+offset].Symbol {
			return false
		}

		if !minSide {
			offset -= 1
		} else {
			offset += 1
		}
	}

	return true
}

func tapeRepr(tape []TapePosition, pos int, radius int) (toRet string) {
	origin := len(tape) / 2
	for i := -1 * radius / 2; i < radius/2; i += 1 {
		if origin+i == pos {
			toRet += "->"
		}
		if tape[origin+i].Symbol == 0 {
			toRet += "0"
		} else {
			toRet += "1"
		}

	}
	return toRet
}

var mutexPS sync.Mutex
var maxValueP int
var championPID uint32
var maxValueS int
var championSID uint32

func argumentTranslatedCyclers(tm bbc.TM, indexInDb uint32, timeLimit int, spaceLimit int, reportMaxSandP bool, reportMaxSandPForAll bool) bool {

	tapeMemory := 2 * spaceLimit

	var tape []TapePosition = make([]TapePosition, tapeMemory)

	currPos := tapeMemory / 2
	nextPos := currPos
	write := byte(0)
	currState := byte(1)
	currTime := 0

	// [tapeSide][state][symbol] -> tapes
	var recordHolders map[bool]map[byte]map[byte][]Record = make(map[bool]map[byte]map[byte][]Record)

	minPosSeen := tapeMemory / 2
	maxPosSeen := tapeMemory / 2

	var err error

	for err == nil && currState > 0 && currState <= MAX_STATES {

		if currTime > timeLimit {
			return false
		}

		read := tape[currPos].Symbol
		//fmt.Println(currTime, currState, read, tapeRepr(tape, currPos, 10))
		// We are breaking a record
		if currPos < minPosSeen || currPos > maxPosSeen {
			//fmt.Println("^")
			var record Record
			record.Tape = make([]TapePosition, len(tape))
			copy(record.Tape, tape)
			record.Time = currTime
			record.Position = currPos

			minSide := currPos < minPosSeen
			if _, ok := recordHolders[minSide]; !ok {
				recordHolders[minSide] = make(map[byte]map[byte][]Record)
			}
			if _, ok := recordHolders[minSide][currState]; !ok {
				recordHolders[minSide][currState] = make(map[byte][]Record)
			}

			// The record has been broken in the past
			if _, ok := recordHolders[minSide][currState][read]; ok {
				for _, pastRecord := range recordHolders[minSide][currState][read] {
					if recordsAreEquivalent(minSide, &pastRecord, &record) {

						// See https://groups.google.com/g/busy-beaver-discuss/c/lcr_6buFz_8
						valueOfS := pastRecord.Time + 1
						valueOfP := currTime - pastRecord.Time

						if reportMaxSandP {
							mutexPS.Lock()
							if valueOfS > maxValueS {
								maxValueS = valueOfS
								championSID = indexInDb
							}
							if valueOfP > maxValueP {
								maxValueP = valueOfP
								championPID = indexInDb
							}
							mutexPS.Unlock()
						}
						if reportMaxSandPForAll {
							fmt.Println("(almost) S:", valueOfS)
							fmt.Println("P:", valueOfP)
						}
						return true
					}
				}
			}

			recordHolders[minSide][currState][read] = append(recordHolders[minSide][currState][read], record)

			minPosSeen = bbc.MinI(minPosSeen, currPos)
			maxPosSeen = bbc.MaxI(maxPosSeen, currPos)
		}

		if maxPosSeen-minPosSeen > spaceLimit || currPos < 0 || currPos >= len(tape) {
			return false
		}

		tape[currPos].Seen = true
		tape[currPos].LastTimeSeen = currTime

		write, currState, nextPos = bbc.TmStep(tm, read, currState, currPos, currTime)

		tape[currPos].Symbol = write

		currPos = nextPos
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
	argMinIndex := flag.Int("m", bbc.TOTAL_UNDECIDED_TIME, "min machine index to consider in seed database")
	argMaxIndex := flag.Int("M", bbc.TOTAL_UNDECIDED, "max machine index to consider in seed database")
	argNWorkers := flag.Int("n", 1000, "workers")
	argIndexFile := flag.String("f", "", "undecided index file to use")

	// See https://groups.google.com/g/busy-beaver-discuss/c/lcr_6buFz_8
	argReportMaxSAndP := flag.Bool("p", false, "report max S and P")

	flag.Parse()

	minIndex := *argMinIndex
	maxIndex := *argMaxIndex
	timeLimit := *argTimeLimit
	spaceLimit := *argSpaceLimit
	nWorkers := *argNWorkers
	indexFileName := *argIndexFile
	reportMaxSAndP := *argReportMaxSAndP

	// fmt.Println(minIndex, maxIndex, timeLimit, spaceLimit, nWorkers)

	// n := 7888060 // not a translated cycler
	// m, _ := bbc.GetMachineI(DB[:], n, true)

	// fmt.Println(argumentTranslatedCyclers(m, timeLimit, spaceLimit))

	runName := "output/translated-cyclers-" + bbc.GetRunName() + "-time-" + fmt.Sprintf("%d", timeLimit) + "-space-" + fmt.Sprintf("%d", spaceLimit) + "-minIndex-" + fmt.Sprintf("%d", minIndex) + "-maxIndex-" + fmt.Sprintf("%d", maxIndex)
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
					if argumentTranslatedCyclers(m, math.MaxUint32, timeLimit, spaceLimit, reportMaxSAndP, false) {
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
					if argumentTranslatedCyclers(m, indexInDb, timeLimit, spaceLimit, reportMaxSAndP, false) {
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

	if reportMaxSAndP {
		fmt.Println("Max (almost) S:", maxValueS)
		fmt.Println("S champion ID:", championSID)
		fmt.Println("Max P:", maxValueP)
		fmt.Println("P champion ID:", championPID)
	}
}
