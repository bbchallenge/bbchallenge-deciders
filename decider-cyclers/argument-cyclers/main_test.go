package main

import (
	"fmt"
	"io/ioutil"
	"testing"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

func TestArgumentCyclers(t *testing.T) {
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}
	// Non-cyclers machines (halting machines and divergent ones)
	t.Run("argument_cyclers_bb5", func(t *testing.T) {
		if argumentCyclers(bbc.GetBB5Winner(), 1000, 500) {
			t.Fail()
		}
	})

	divergent_indices := []int{7888060, 5351679, 7199289, 7177945, 12930717, 16322779, 41540523, 8852034, 14203995, 294145, 13128060, 12023841, 3666257, 56021278}

	for i := range divergent_indices {
		index := divergent_indices[i]
		t.Run(fmt.Sprintf("argument_cyclers_divergent_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentCyclers(tm, 1000, 500) {
				t.Fail()
			}
		})
	}

	cyclers_indices := []int{5164457, 13551915, 4888229}

	for i := range cyclers_indices {
		index := cyclers_indices[i]
		t.Run(fmt.Sprintf("argument_cyclers_cyclers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if !argumentCyclers(tm, 1000, 500) {
				t.Fail()
			}
		})
	}

}

func TestTapeToStr(t *testing.T) {
	var tape Tape

	tape[MAX_MEMORY/2].Symbol = 0
	tape[MAX_MEMORY/2].Seen = true
	tape[MAX_MEMORY/2+1].Symbol = 0
	tape[MAX_MEMORY/2+1].Seen = true

	if tapeToStr(&tape) != "00" {
		t.Fail()
	}
}
