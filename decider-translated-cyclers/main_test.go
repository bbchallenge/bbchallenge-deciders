package main

import (
	"fmt"
	"io/ioutil"
	"testing"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

func TestArgumentTranslatedCyclers(t *testing.T) {
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}
	// Non-cyclers machines (halting machines and divergent ones)
	t.Run("argument_translated-cyclers_bb5", func(t *testing.T) {
		if argumentTranslatedCyclers(bbc.GetBB5Winner(), 1000, 500) {
			t.Fail()
		}
	})

	divergent_indices := []int{7888060, 5351679, 7199289, 7177945, 12930717, 16322779, 41540523, 8852034, 14203995, 294145, 13128060, 12023841, 3666257, 56021278}

	for i := range divergent_indices {
		index := divergent_indices[i]
		t.Run(fmt.Sprintf("argument_translated-cyclers_divergent_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentTranslatedCyclers(tm, 1000, 500) {
				t.Fail()
			}
		})
	}

	translated_cyclers_indices := []int{78619822, 52297459, 37549149, 37799884, 33613794, 65712201, 73643020, 73823886, 87711504}

	for i := range translated_cyclers_indices {
		index := translated_cyclers_indices[i]
		t.Run(fmt.Sprintf("argument_translated-cyclers_translated-cyclers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if !argumentTranslatedCyclers(tm, 1000, 500) {
				t.Fail()
			}
		})
	}

	// End of Feb 22 we discovered translated cyclers that need bigger parameters to be decided
	more_complex_translated_cyclers_indices := []int{54203719, 36496615, 78264693, 88470160, 70383585, 34518122, 60197828, 81893093}

	for i := range more_complex_translated_cyclers_indices {
		index := more_complex_translated_cyclers_indices[i]
		t.Run(fmt.Sprintf("argument_translated-cyclers_complex-translated-cyclers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if !argumentTranslatedCyclers(tm, 10000, 5000) {
				t.Fail()
			}
		})
	}

}
