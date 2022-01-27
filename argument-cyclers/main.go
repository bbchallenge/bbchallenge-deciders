package main

import (
	"errors"
	"fmt"
	"io/ioutil"
	"os"
	"strconv"

	tabulate "github.com/rgeoghegan/tabulate"
)

type TM [2 * 5 * 3]byte

func tmTransitionToStr(b1 byte, b2 byte, b3 byte) (toRet string) {

	if b3 == 0 {
		return "???"
	}

	toRet = strconv.Itoa(int(b1))

	if b2 == 0 {
		toRet += "R"
	} else {
		toRet += "L"
	}

	toRet += string(rune(int('A') + int(b3) - 1))

	return toRet
}

func (tm TM) ToAsciiTable(nbStates byte) (toRet string) {

	var table [][]string

	for i := byte(0); i < nbStates; i += 1 {

		table = append(table, []string{string(rune(int('A') + int(i))),
			tmTransitionToStr(tm[6*i], tm[6*i+1], tm[6*i+2]),
			tmTransitionToStr(tm[6*i+3], tm[6*i+4], tm[6*i+5])})
	}

	layout := &tabulate.Layout{Headers: []string{"-", "0", "1"}, Format: tabulate.SimpleFormat}
	asText, _ := tabulate.Tabulate(
		table, layout,
	)

	return asText
}

const DB_PATH = "../all_5_states_undecided_machines_with_global_header"

const R = 0
const L = 1

const MAX_STATES = 5
const TIME_LIMIT = 47176870
const SPACE_LIMIT = 12289

const MAX_MEMORY = 40000

type SymbolTimeSeen struct {
	Symbol byte
	Time   int
	Seen   bool
}

type Tape [MAX_MEMORY]SymbolTimeSeen

func tmStep(tm TM, tape *Tape, currState byte, currPos int, currTime int) (nextState byte, nextPos int, err error) {
	read := tape[currPos]
	tmTransition := 6*(currState-1) + 3*read.Symbol
	write := tm[tmTransition]

	tape[currPos].Symbol = write
	tape[currPos].Time = currTime

	move := tm[tmTransition+1]
	nextState = tm[tmTransition+2]

	if move == R {
		nextPos = currPos + 1

		if nextPos >= len(tape) {
			err = errors.New("max memory exceeded")
		}

	} else {
		nextPos = currPos - 1

		if nextPos < 0 {
			err = errors.New("max memory exceeded")
		}
	}

	return nextState, nextPos, err
}

func MaxI(a int, b int) int {
	if a > b {
		return a
	}
	return b
}

func MinI(a int, b int) int {
	if a < b {
		return a
	}
	return b
}

func simulate(tm TM) (int, error) {
	currPos := MAX_MEMORY / 2
	currState := byte(1)
	currTime := 0
	var tape Tape

	minPosSeen := MAX_MEMORY
	maxPosSeen := 0

	var err error

	for err == nil && currState > 0 && currState <= MAX_STATES {
		currState, currPos, err = tmStep(tm, &tape, currState, currPos, currTime)
		minPosSeen := MinI(minPosSeen, currPos)
		maxPosSeen := MaxI(maxPosSeen, currPos)

		if maxPosSeen-minPosSeen > SPACE_LIMIT {
			err = errors.New("space limit bb5_space exceeded")
			return currTime, err
		}

		if currTime > TIME_LIMIT {
			err = errors.New("time limit bb5 exceeded")
			return currTime, err
		}

		currTime += 1

	}

	return currTime, err
}

func identicalLocalNeighborsThroughTime(tapeBefore *Tape, tapeNow *Tape, posBefore int, posNow int, timeBefore int) bool {

	if timeBefore == 12 {
		fmt.Println(posBefore, posNow, timeBefore)
	}

	offset := 0
	for posBefore-offset >= 0 && posNow-offset >= 0 && tapeNow[posBefore-offset].Seen && tapeNow[posBefore-offset].Time >= timeBefore {
		if timeBefore == 12 {
			fmt.Print(offset, posBefore-offset, tapeNow[posBefore-offset], "  ")
		}
		if tapeNow[posNow-offset].Symbol != tapeBefore[posBefore-offset].Symbol {
			return false
		}
		offset += 1

		if !(posBefore-offset >= 0 && posNow-offset >= 0) {
			return false // not necessarily false but not bothered to check
		}
	}

	//fmt.Println()

	offset = 0
	for posBefore+offset < MAX_MEMORY && posNow+offset < MAX_MEMORY && tapeNow[posBefore+offset].Seen && tapeNow[posBefore+offset].Time >= timeBefore {
		if tapeNow[posNow+offset].Symbol != tapeBefore[posBefore+offset].Symbol {
			return false
		}
		offset += 1

		if !(posBefore+offset < MAX_MEMORY && posNow+offset < MAX_MEMORY) {
			return false // not necessarily false but not bothered to check
		}
	}

	return true
}

func tapeToStr(tape *Tape, currPos int, radius int) (toRet string) {
	pos := MAX_MEMORY / 2
	for i := -1 * radius / 2; i < radius/2; i += 1 {
		if pos+i == MAX_MEMORY/2 {
			toRet += string(".")
		}
		if pos+i == currPos {
			toRet += string("->")
		}
		toRet += string('0' + tape[pos+i].Symbol)

	}
	return toRet
}

func argumentCyclers(tm TM) bool {
	currPos := MAX_MEMORY / 2
	currState := byte(1)
	currTime := 0
	var tape Tape

	minPosSeen := MAX_MEMORY / 2
	maxPosSeen := MAX_MEMORY / 2

	var err error

	var tapeSnapshots [2][10]Tape
	var transitionSeen [2][10]bool
	var posSnapshot [2][10]int
	var timeSnapshot [2][10]int

	for err == nil && currState > 0 && currState <= MAX_STATES {

		minPosSeen = MinI(minPosSeen, currPos)
		maxPosSeen = MaxI(maxPosSeen, currPos)

		isRecord := false
		recordPositive := 0
		if currPos >= maxPosSeen {
			isRecord = true
			recordPositive = 1
		} else if currPos <= minPosSeen {
			isRecord = true
			recordPositive = 0
		}

		tape[currPos].Seen = true
		read := tape[currPos].Symbol

		//fmt.Println(currPos-MAX_MEMORY/2, minPosSeen-MAX_MEMORY/2, maxPosSeen-MAX_MEMORY/2)
		fmt.Println(currTime, currState, read, tapeToStr(&tape, currPos, 10))

		if isRecord && transitionSeen[recordPositive][2*(currState-1)+read] && (currPos == minPosSeen || currPos == maxPosSeen) {
			if identicalLocalNeighborsThroughTime(&tapeSnapshots[recordPositive][2*(currState-1)+read], &tape, posSnapshot[recordPositive][2*(currState-1)+read], currPos, timeSnapshot[recordPositive][2*(currState-1)+read]) {

				fmt.Println(tapeToStr(&tapeSnapshots[recordPositive][2*(currState-1)+read], posSnapshot[recordPositive][2*(currState-1)+read], 10), timeSnapshot[recordPositive][2*(currState-1)+read], currTime)
				return true
			}
		}

		if isRecord {
			//fmt.Println(currTime, currPos, minPosSeen, maxPosSeen, "oo", tapeToStr(&tape, currPos, 10))
			transitionSeen[recordPositive][2*(currState-1)+read] = true
			timeSnapshot[recordPositive][2*(currState-1)+read] = currTime
			posSnapshot[recordPositive][2*(currState-1)+read] = currPos
			tapeSnapshots[recordPositive][2*(currState-1)+read] = tape
		}

		currState, currPos, err = tmStep(tm, &tape, currState, currPos, currTime)

		if maxPosSeen-minPosSeen > SPACE_LIMIT {
			return false
		}

		if currTime > TIME_LIMIT {
			return false
		}
		if currTime%1000000 == 0 {
			fmt.Println(currTime)
		}
		currTime += 1

		if currTime > 20 {
			return false
		}

	}

	return false
}

func getMachineI(db []byte, i int) (tm TM, err error) {

	if i <= 0 || i > 88664064 {
		err := errors.New("invalid db index")
		return tm, err
	}

	copy(tm[:], db[30*i:30*(i+1)])
	return tm, nil
}

func GetBB5Winner() TM {
	// +---+-----+-----+
	// | - |  0  |  1  |
	// +---+-----+-----+
	// | A | 1RB | 1LC |
	// | B | 1RC | 1RB |
	// | C | 1RD | 0LE |
	// | D | 1LA | 1LD |
	// | E | 1RH | 0LA |
	// +---+-----+-----+

	return TM{
		1, R, 2, 1, L, 3,
		1, R, 3, 1, R, 2,
		1, R, 4, 0, L, 5,
		1, L, 1, 1, L, 4,
		1, R, 6, 0, L, 1}

}

func main() {

	DB, err := ioutil.ReadFile(DB_PATH)

	if err != nil {
		fmt.Println(err)
		os.Exit(-1)
	}

	fmt.Println(len(DB) / 30)

	m, err := getMachineI(DB[:], 13551916)
	//m := GetBB5Winner()

	fmt.Println(m.ToAsciiTable(5))
	fmt.Println(argumentCyclers(m))
}
