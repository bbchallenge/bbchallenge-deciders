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

	not_backward_reasoning_indices := []int{40851850, 11710205, 11726152, 72903966, 10039500, 4966400, 13754164}
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

	backward_reasoning_indices := []int{13955979, 54221304, 7850055, 7658575, 7274055, 620647, 392407}

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
