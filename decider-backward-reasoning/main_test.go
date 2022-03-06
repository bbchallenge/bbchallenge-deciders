package main

import (
	"fmt"
	"io/ioutil"
	"testing"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

func TestArgumentBackwardReasoning(t *testing.T) {
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}

	// Obsolete indices since DB sorting operation 6/03/22 not_backward_reasoning_indices := []int{40851850, 11710205, 11726152, 72903966, 10039500, 4966400, 13754164}
	not_backward_reasoning_indices := []int{73261028, 8527536, 7911681, 28086713, 11059089, 11670429, 7865218}
	for i := range not_backward_reasoning_indices {
		index := not_backward_reasoning_indices[i]
		t.Run(fmt.Sprintf("decider_not-backward-reasoning_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if deciderBackwardReasoning(tm, 300, testing.Verbose()) {
				t.Fail()
			}
		})
	}

	// Obsolete indices since DB sorting operation 6/03/22 backward_reasoning_indices := []int{13955979, 54221304, 7850055, 7658575, 7274055, 620647, 392407}

	backward_reasoning_indices := []int{4843748, 58360621, 2009846, 1973992, 11176971, 4147941, 12071224}

	for i := range backward_reasoning_indices {
		index := backward_reasoning_indices[i]
		t.Run(fmt.Sprintf("decider_backward-reasoning_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if !deciderBackwardReasoning(tm, 300, testing.Verbose()) {
				t.Fail()
			}
		})
	}

}
