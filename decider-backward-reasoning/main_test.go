package main

import (
	"fmt"
	"io/ioutil"
	"math"
	"testing"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

func TestSkelet10(t *testing.T) {
	// There is a debate whether or not Skelet's 10 machine is
	// decidable by backward reasoning or not https://bbchallenge.org/3810716
	// https://discuss.bbchallenge.org/t/skelet-10-backtracking/57
	// Shawn is right, we have a bug..... This test should fail.
	indices_to_test := []int{3810716}
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}

	for i := range indices_to_test {
		index := indices_to_test[i]
		tm, err := bbc.GetMachineI(DB[:], index, true)
		if err != nil {
			t.Fail()
		}
		if deciderBackwardReasoning(tm, i, 300, true, false) {
			fmt.Println(tm.ToAsciiTable(5))
			fmt.Println(tm)

			t.Fail()
		}
	}
}

func TestIndividualMachine(t *testing.T) {
	// Machine used to expamplify the proof of https://github.com/bbchallenge/bbchallenge-proofs/blob/main/deciders/correctness-deciders.pdf
	indices_to_test := []int{55897188}
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}

	for i := range indices_to_test {
		index := indices_to_test[i]
		tm, err := bbc.GetMachineI(DB[:], index, true)
		if err != nil {
			t.Fail()
		}
		if !deciderBackwardReasoning(tm, index, 2, true, false) {
			fmt.Println(tm.ToAsciiTable(5))
			fmt.Println(tm)

			t.Fail()
		}
	}
}

func TestArgumentBackwardReasoning(t *testing.T) {
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}

	t.Run(fmt.Sprintf("decider_not-backward-reasoning_bb5_winner"), func(t *testing.T) {
		tm := bbc.GetBB5Winner()

		if deciderBackwardReasoning(tm, math.MaxInt32, 50, false, false) {
			fmt.Println(tm.ToAsciiTable(5))
			fmt.Println(tm)
			fmt.Println("Uh oh, expected false but got true")
			t.Fail()
		}
	})

	// Below are indices
	not_backward_reasoning_indices := []int{7410754, 3810716}
	for i := range not_backward_reasoning_indices {
		index := not_backward_reasoning_indices[i]
		t.Run(fmt.Sprintf("decider_not-backward-reasoning_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if deciderBackwardReasoning(tm, index, 50, false, false) {
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
			if !deciderBackwardReasoning(tm, index, 50, false, false) {
				fmt.Println("Uh oh, expected true but got false: ", index)
				t.Fail()
			}
		})
	}

}
