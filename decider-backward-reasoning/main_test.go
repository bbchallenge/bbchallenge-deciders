package main

import (
	"fmt"
	"io/ioutil"
	"testing"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

// func TestIndividualMachine(t *testing.T) {
// 	//DB, _ := ioutil.ReadFile(DB_PATH)
// 	// Machine 27879939
// 	// tm := bbc.TM{1, 0, 2, 0, 0, 0, 1, 1, 3, 0, 0, 0, 0, 1, 4,
// 	// 	0, 1, 3, 1, 0, 4, 0, 0, 5, 1, 1, 2, 0, 0, 5}

// 	// Machine 7410754
// 	tm := bbc.TM{1, 2, 0, 1, 4, 1, 1, 3, 1, 0, 3, 1, 1, 1, 0, 0, 3, 0, 0, 0, 0, 1, 5, 0, 0, 2, 1, 1, 4}

// 	//tm, _ := bbc.GetMachineI(DB[:], 27879939, true)
// 	//fmt.Println(tm.ToAsciiTable(5))
// 	//fmt.Print(tm)
// 	deciderBackwardReasoning(tm, 300, true)
// }

func TestArgumentBackwardReasoning(t *testing.T) {
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}

	t.Run(fmt.Sprintf("decider_not-backward-reasoning_bb5_winner"), func(t *testing.T) {
		tm := bbc.GetBB5Winner()

		if deciderBackwardReasoning(tm, 300, false) {
			fmt.Println(tm.ToAsciiTable(5))
			fmt.Println(tm)
			fmt.Println("Uh oh, expected false but got true")
			t.Fail()
		}
	})

	// Below are indices
	not_backward_reasoning_indices := []int{7410754}
	for i := range not_backward_reasoning_indices {
		index := not_backward_reasoning_indices[i]
		t.Run(fmt.Sprintf("decider_not-backward-reasoning_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if deciderBackwardReasoning(tm, 300, false) {
				fmt.Println(tm.ToAsciiTable(5))
				fmt.Println(tm)
				fmt.Println("Uh oh, expected false but got true: ", index)
				t.Fail()
			}
		})
	}

	// Below are new indices
	backward_reasoning_indices := []int{55897188, 27879939, 2713328, 10817532}

	for i := range backward_reasoning_indices {
		index := backward_reasoning_indices[i]
		t.Run(fmt.Sprintf("decider_backward-reasoning_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if !deciderBackwardReasoning(tm, 300, false) {
				fmt.Println("Uh oh, expected true but got false: ", index)
				t.Fail()
			}
		})
	}

}
