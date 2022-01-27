package main

import (
	"fmt"
	"io/ioutil"
	"os"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

const DB_PATH = "../all_5_states_undecided_machines_with_global_header"

const R = 0
const L = 1

const MAX_STATES = 5
const TIME_LIMIT = 47176870
const SPACE_LIMIT = 12289

const MAX_MEMORY = 40000

type SymbolAndSeen struct {
	Symbol byte
	Seen   bool
}

type Tape [MAX_MEMORY]SymbolAndSeen

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

func argumentCyclers(tm bbc.TM) bool {
	currPos := MAX_MEMORY / 2
	nextPos := 0
	write := byte(0)
	currState := byte(1)
	currTime := 0

	var tape Tape

	minPosSeen := MAX_MEMORY / 2
	maxPosSeen := MAX_MEMORY / 2

	var configSeen map[byte]map[byte]map[string]bool // [state][read][tape] -> True

	configSeen = make(map[byte]map[byte]map[string]bool)

	var err error

	for err == nil && currState > 0 && currState <= MAX_STATES {

		if _, ok := configSeen[currState]; !ok {
			configSeen[currState] = make(map[byte]map[string]bool)
		}

		minPosSeen = MinI(minPosSeen, currPos)
		maxPosSeen = MaxI(maxPosSeen, currPos)

		tape[currPos].Seen = true
		read := tape[currPos].Symbol

		if _, ok := configSeen[currState][read]; !ok {
			configSeen[currState][read] = make(map[string]bool)
		}

		tapeStr := tapeToStr(&tape)

		_, ok := configSeen[currState][read][tapeStr]

		if ok {
			fmt.Println(currTime, currState, read, tapeStr)
			return true
		}

		configSeen[currState][read][tapeStr] = true

		write, currState, nextPos = bbc.TmStep(tm, read, currState, currPos, currTime)

		tape[currPos].Symbol = write
		currPos = nextPos

		if maxPosSeen-minPosSeen > SPACE_LIMIT {
			return false
		}

		if currTime > TIME_LIMIT {
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

	fmt.Println(len(DB) / 30)

	m, err := bbc.GetMachineI(DB[:], 4888230, true)
	//m := GetBB5Winner()

	//fmt.Println(tapeToStr(&tape))
	//fmt.Println(m.ToAsciiTable(5))
	fmt.Println(argumentCyclers(m))
}
