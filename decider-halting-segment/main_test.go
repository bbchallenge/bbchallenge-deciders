package main

import (
	"fmt"
	"io/ioutil"
	"testing"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

//verify for yes: 108115
func TestIndividualMachine(t *testing.T) {
	indices_to_test := []int{108115}
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
		fmt.Println(tm.ToAsciiTable(5))
		if !deciderHaltingSegment(tm, 1, 1000, true, true) {
			t.Fail()
		}
	}
}
func TestChaosMachine(t *testing.T) {
	// http://turbotm.de/~heiner/BB/TM4-proof.txt
	// Chaotic Machine [Marxen & Buntrock, 1990]
	indices_to_test := []int{76708232}
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
		fmt.Println(tm.ToAsciiTable(5))
		if !deciderHaltingSegment(tm, 2, 1000, false, true) {
			t.Fail()
		}
	}
}

func TestComplexCounter(t *testing.T) {
	// Complex Counter [Marxen & Buntrock, 1990]
	indices_to_test := []int{10936909}
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
		fmt.Println(tm.ToAsciiTable(5))
		if !deciderHaltingSegment(tm, 3, 1000, false, true) {
			t.Fail()
		}
	}
}

func TestArgumentHaltingSegment(t *testing.T) {
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}

	t.Run(fmt.Sprintf("decider_not-halting-segment_bb5_winner"), func(t *testing.T) {
		tm := bbc.GetBB5Winner()

		if deciderHaltingSegment(tm, 1, 1000, true, false) {
			fmt.Println(tm.ToAsciiTable(5))
			fmt.Println(tm)
			fmt.Println("Uh oh, expected false but got true")
			t.Fail()
		}
	})

	// Below are indices that do not work
	not_backward_reasoning_indices := []int{7410754, 3810716}
	for i := range not_backward_reasoning_indices {
		index := not_backward_reasoning_indices[i]
		t.Run(fmt.Sprintf("decider_not-halting-segment_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if deciderHaltingSegment(tm, 1, 1000, true, false) {
				fmt.Println(tm.ToAsciiTable(5))
				fmt.Println(tm)
				fmt.Println("Uh oh, expected false but got true: ", index)
				t.Fail()
			}
		})
	}

	// Below are indices that work
	backward_reasoning_indices := []int{108082, 109520, 108115}

	for i := range backward_reasoning_indices {
		index := backward_reasoning_indices[i]
		t.Run(fmt.Sprintf("decider_halting-segment_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if !deciderHaltingSegment(tm, 1, 1000, true, false) {
				fmt.Println("Uh oh, expected true but got false: ", index)
				t.Fail()
			}
		})
	}
}
