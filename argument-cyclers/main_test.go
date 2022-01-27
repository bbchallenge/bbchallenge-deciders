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
		if argumentCyclers(bbc.GetBB5Winner()) {
			t.Fail()
		}
	})

	divergent_indices := []int{7888061, 5351680, 7199290, 7177946, 12930718, 16322780, 41540524, 8852035, 14203996, 294146, 13128061, 12023842, 3666258, 56021279}

	for i := range divergent_indices {
		index := divergent_indices[i]
		t.Run(fmt.Sprintf("argument_cyclers_divergent_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentCyclers(tm) {
				t.Fail()
			}
		})
	}

	cyclers_indices := []int{5164458, 13551916, 4888230, 78619823, 52297460, 37549150, 37799885, 33613795, 65712202, 73643021, 73823887, 87711505}

	for i := range cyclers_indices {
		index := cyclers_indices[i]
		t.Run(fmt.Sprintf("argument_cyclers_cyclers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if !argumentCyclers(tm) {
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
