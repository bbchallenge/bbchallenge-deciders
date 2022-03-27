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

	// Obsolete indices since DB sorting operation 6/03/22 
	// divergent_indices := []int{7888060, 5351679, 7199289, 7177945, 12930717, 16322779, 41540523, 8852034, 14203995, 294145, 13128060, 12023841, 3666257, 56021278}
	// Below are new indices
	divergent_indices := []int{14017021,
		13206000,
		8107478,
		14053644,
		14276172,
		78082807,
		83293270,
		1201055,
		9354848,
		6369968,
		5795478,
		12745999,
		13578663,
		23400034}

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

	// Obsolete indices since DB sorting operation 6/03/22 
	// translated_cyclers_indices := []int{78619822, 52297459, 37549149, 37799884, 33613794, 65712201, 73643020, 73823886, 87711504}
	// Below are new indices
	translated_cyclers_indices := []int{32510779,
		45010518,
		14427007,
		14643029,
		15167997,
		50491158,
		59645887,
		31141863,
		28690248}

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
	// Obsolete indices since DB sorting operation 6/03/22 
	// more_complex_translated_cyclers_indices := []int{54203719, 36496615, 78264693, 88470160, 70383585, 34518122, 60197828, 81893093}
	// Below are new indices
	more_complex_translated_cyclers_indices := []int{46965866,
		74980673,
		88062418,
		59090563,
		76989562,
		46546554,
		36091834,
		58966114}

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
