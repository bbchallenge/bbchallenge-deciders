package main

import (
	"fmt"
	"io/ioutil"
	"math"
	"testing"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

func TestIndividualMachines(t *testing.T) {
	DB, _ := ioutil.ReadFile(DB_PATH)

	//this one takes bit longer to detect
	indices := []int{7866044}
	for i := range indices {
		index := indices[i]
		tm, _ := bbc.GetMachineI(DB[:], index, true)
		if !argumentUnilateralBouncers(tm, uint32(index), 10000, 500, false, true, false) {
			t.Fail()
		}
	}
}

func TestArgumentUnilateralBouncers(t *testing.T) {
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}

	// halting machines
	t.Run("argument_translated-cyclers_bb5", func(t *testing.T) {
		if argumentUnilateralBouncers(bbc.GetBB5Winner(), math.MaxUint32, 1000, 500, false, false, false) {
			t.Fail()
		}
	})

	cyclers_indices := []int{279081, 4231819, 279081}
	for i := range cyclers_indices {
		index := cyclers_indices[i]
		t.Run(fmt.Sprintf("argument_unilateral-bouncers_cyclers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, false, false, false) {
				t.Fail()
			}
		})
	}

	translated_cyclers_indices := []int{59645887, 15167997, 59090563}
	for i := range translated_cyclers_indices {
		index := translated_cyclers_indices[i]
		t.Run(fmt.Sprintf("argument_unilateral-bouncers_translated-cyclers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, false, false, false) {
				t.Fail()
			}
		})
	}

	unilateral_bouncers_right_indices := []int{4175994, 9281450, 7122624, 8693297, 6521893, 3859367, 12342257, 3192958, 6986652, 61599536, 731, 186028, 508512, 22925328, 2119, 94290, 81882, 3169076, 6235165, 43021946, 8038262}
	for i := range unilateral_bouncers_right_indices {
		index := unilateral_bouncers_right_indices[i]
		t.Run(fmt.Sprintf("argument_unilateral-bouncers_unilateral-bouncers-right_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if !argumentUnilateralBouncers(tm, uint32(index), 1000, 500, false, false, false) {
				t.Fail()
			}
		})
	}

	unilateral_bouncers_left_indices := []int{6048289, 7651640, 7640327, 297868, 7505844, 872906, 2482815}
	for i := range unilateral_bouncers_left_indices {
		index := unilateral_bouncers_left_indices[i]
		t.Run(fmt.Sprintf("argument_unilateral-bouncers_unilateral-bouncers-left_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, false, false, false) {
				t.Fail()
			}
		})
	}

	//strange left bouncers
	strange_unilateral_bouncers_left_indices := []int{6538740, 7159661, 5030335}
	for i := range strange_unilateral_bouncers_left_indices {
		index := strange_unilateral_bouncers_left_indices[i]
		t.Run(fmt.Sprintf("argument_unilateral-bouncers_strange-unilateral-bouncers-left_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, false, false, false) {
				t.Fail()
			}
		})
	}
	bilateral_bouncers_indices := []int{12785688, 8929416, 76727755}
	for i := range bilateral_bouncers_indices {
		index := bilateral_bouncers_indices[i]
		t.Run(fmt.Sprintf("argument_unilateral-bouncers_bilateral-bouncers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, false, false, false) {
				t.Fail()
			}
		})
	}

	translated_unilateral_bouncers_indices := []int{6164147, 31837821, 20076854}
	for i := range translated_unilateral_bouncers_indices {
		index := translated_unilateral_bouncers_indices[i]
		t.Run(fmt.Sprintf("argument_unilateral-bouncers_translated-unilateral-bouncers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, false, false, false) {
				t.Fail()
			}
		})
	}

	exponential_counters_indices := []int{14244805, 10936909, 3840180}
	for i := range exponential_counters_indices {
		index := exponential_counters_indices[i]
		t.Run(fmt.Sprintf("argument_unilateral-bouncers_exponential-counters_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, false, false, false) {
				t.Fail()
			}
		})
	}

	bells_indices := []int{8527536, 73261028, 63938734}
	for i := range bells_indices {
		index := bells_indices[i]
		t.Run(fmt.Sprintf("argument_unilateral-bouncers_bells_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, false, false, false) {
				t.Fail()
			}
		})
	}

	non_classified_indices := []int{6490892, 11018350, 9390305}
	for i := range non_classified_indices {
		index := non_classified_indices[i]
		t.Run(fmt.Sprintf("argument_unilateral-bouncers_non-classified_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, false, false, false) {
				t.Fail()
			}
		})
	}

}

func TestInvertedArgumentUnilateralBouncers(t *testing.T) {
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}

	// halting machines
	t.Run("argument_translated-cyclers_bb5", func(t *testing.T) {
		if argumentUnilateralBouncers(bbc.GetBB5Winner(), math.MaxUint32, 1000, 500, true, false, false) {
			t.Fail()
		}
	})

	cyclers_indices := []int{279081, 4231819, 279081}
	for i := range cyclers_indices {
		index := cyclers_indices[i]
		t.Run(fmt.Sprintf("argument_inverted-unilateral-bouncers_cyclers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, true, false, false) {
				t.Fail()
			}
		})
	}

	translated_cyclers_indices := []int{59645887, 15167997, 59090563}
	for i := range translated_cyclers_indices {
		index := translated_cyclers_indices[i]
		t.Run(fmt.Sprintf("argument_inverted-unilateral-bouncers_translated-cyclers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, true, false, false) {
				t.Fail()
			}
		})
	}

	unilateral_bouncers_right_indices := []int{4175994, 9281450, 7122624, 8693297, 6521893, 3859367, 12342257, 3192958, 6986652, 61599536, 731, 186028, 508512, 22925328, 2119, 94290, 81882, 3169076, 6235165, 43021946, 8038262}
	for i := range unilateral_bouncers_right_indices {
		index := unilateral_bouncers_right_indices[i]
		t.Run(fmt.Sprintf("argument_inverted-unilateral-bouncers_unilateral-bouncers-right_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, true, false, false) {
				t.Fail()
			}
		})
	}

	unilateral_bouncers_left_indices := []int{6048289, 7651640, 7640327, 297868, 7505844, 872906, 2482815}
	for i := range unilateral_bouncers_left_indices {
		index := unilateral_bouncers_left_indices[i]
		t.Run(fmt.Sprintf("argument_inverted-unilateral-bouncers_unilateral-bouncers-left_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if !argumentUnilateralBouncers(tm, uint32(index), 1000, 500, true, false, false) {
				t.Fail()
			}
		})
	}

	//strange left bouncers:
	//6538740 and 5030335 flip between 2 different bases
	//7159661 flips between 2 different heads
	//as is those are not detected
	//should they be classified as unilateral bouncers?
	strange_unilateral_bouncers_left_indices := []int{6538740, 7159661, 5030335}
	for i := range strange_unilateral_bouncers_left_indices {
		index := strange_unilateral_bouncers_left_indices[i]
		t.Run(fmt.Sprintf("argument_inverted-unilateral-bouncers_strange-unilateral-bouncers-left_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, true, false, false) {
				t.Fail()
			}
		})
	}

	bilateral_bouncers_indices := []int{12785688, 8929416, 76727755}
	for i := range bilateral_bouncers_indices {
		index := bilateral_bouncers_indices[i]
		t.Run(fmt.Sprintf("argument_inverted-unilateral-bouncers_bilateral-bouncers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, true, false, false) {
				t.Fail()
			}
		})
	}

	translated_unilateral_bouncers_indices := []int{6164147, 31837821, 20076854}
	for i := range translated_unilateral_bouncers_indices {
		index := translated_unilateral_bouncers_indices[i]
		t.Run(fmt.Sprintf("argument_inverted-unilateral-bouncers_translated-unilateral-bouncers_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, true, false, false) {
				t.Fail()
			}
		})
	}

	exponential_counters_indices := []int{14244805, 10936909, 3840180}
	for i := range exponential_counters_indices {
		index := exponential_counters_indices[i]
		t.Run(fmt.Sprintf("argument_inverted-unilateral-bouncers_exponential-counters_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, true, false, false) {
				t.Fail()
			}
		})
	}

	bells_indices := []int{8527536, 73261028, 63938734}
	for i := range bells_indices {
		index := bells_indices[i]
		t.Run(fmt.Sprintf("argument_inverted-unilateral-bouncers_bells_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, true, false, false) {
				t.Fail()
			}
		})
	}

	non_classified_indices := []int{6490892, 11018350, 9390305}
	for i := range non_classified_indices {
		index := non_classified_indices[i]
		t.Run(fmt.Sprintf("argument_inverted-unilateral-bouncers_non-classified_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if argumentUnilateralBouncers(tm, uint32(index), 1000, 500, true, false, false) {
				t.Fail()
			}
		})
	}

}
