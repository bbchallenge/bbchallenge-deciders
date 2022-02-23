package main

import (
	"fmt"
	"io/ioutil"
	"testing"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

func TestSimulateAndGetPassageTimes(t *testing.T) {
	tm := bbc.GetBB5Winner()
	passageTimes := simulateAndGetPassageTimes(tm, 100000)

	// These were computed from Python notebook
	// https://github.com/bbchallenge/bbchallenge-py/blob/main/Heuristic%20-%20Polynomial%20Passage%20Times.ipynb
	correctOPassageTimes := []int{0, 6, 8, 12, 18, 30, 48, 52, 68, 78, 98, 116, 222, 226, 268, 278, 324, 340, 390, 414, 880, 884, 976, 986, 1082, 1098, 1198, 1220, 1324, 1354, 3098, 3102, 3284, 3294, 3480, 3496, 3686, 3708, 3902, 3930, 4128, 4164, 9974, 9978, 10314, 10324, 10664, 10680, 11024, 11046, 11394, 11422, 11774, 11808, 12164, 12206, 30624, 30628, 31230, 31240, 31846, 31862, 32472, 32494, 33108, 33136, 33754, 33788, 34410, 34450, 35076, 35124, 90982, 90986, 92038, 92048, 93104, 93120, 94180, 94202, 95266, 95294, 96362, 96396, 97468, 97508, 98584, 98630, 99710, 99764}

	fmt.Println(passageTimes[0])

	for i, val := range passageTimes[0] {
		if correctOPassageTimes[i] != val {
			t.Fail()
		}
	}
}

func TestOnePolynomialPassageTimes(t *testing.T) {
	n := 5643626
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}
	tm, err := bbc.GetMachineI(DB[:], n, true)
	if err != nil {
		t.Fail()
	}
	if !heuristicPolynomialPassageTimes(tm, 100000, 10, 5, 40, true) {
		t.Fail()
	}
}

func TestHeuristicPolynomialPassageTimes(t *testing.T) {

	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}

	if heuristicPolynomialPassageTimes(bbc.GetBB5Winner(), 100000, 10, 5, 40, false) {
		t.Fail()
	}

	notPolynomialPassageTimes := []int{4824149, 47437564, 7431789, 10613000, 11184382, 7931183, 7695094, 8594059, 12877334, 4631164, 4807678, 467941, 11776032, 12345394, 1236862, 14009904, 12418684, 10269203, 40851850, 54221304, 11710205, 11726152, 72903966, 10039500, 7850055, 4966400, 13754164, 7658575}
	for i := range notPolynomialPassageTimes {
		index := notPolynomialPassageTimes[i]
		t.Run(fmt.Sprintf("heuristic_not-polynomial-passage-times_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if heuristicPolynomialPassageTimes(tm, 100000, 10, 5, 40, false) {
				t.Fail()
			}
		})
	}

	// unilateral and bilateral pongs
	// see https://github.com/bbchallenge/bbchallenge-py/blob/main/Heuristic%20-%20Polynomial%20Passage%20Times.ipynb
	polynomialPassageTimes := []int{5643626, 3565349, 2108244, 13474013, 9755023, 8103390, 9225630, 3169967, 10363739, 12223654, 13090023, 11225780, 11544059, 1507193, 13795582, 12244006, 13930014, 11334525, 1028021, 1566023, 269038, 828479, 8497788, 6136947, 11635490, 6398375, 9703986, 5203553, 6843551, 6964220, 11598427, 13576089, 11519070, 9913985, 11916339, 14086216, 8439154, 87940465, 11089226, 4482180, 4284, 4468110, 12189145, 12206912, 46840666, 8416617, 13440160, 2498135, 12285970, 14218336, 8634375, 1048062, 13977714, 8457777, 4745553, 1153972, 13802273, 11388271, 5383040, 1507193, 9786806, 5702127, 7883753, 4557119, 33672339, 3370060, 402959, 11515638, 6399980, 1960734, 8818662, 6300366, 8900440, 6092203, 11980798, 9813712, 4058620, 3418516, 14044691, 9312210, 172282, 582211, 12199575, 12594284, 2731122, 7716308, 12292720, 12549707, 26414953, 2479142, 6152467, 38111044, 662317, 526738, 5328524, 13649989, 4208656, 13784545, 5928209, 5532891, 12876855, 937083, 4726380, 67780089, 13104130, 1015814, 12215361, 76952003, 205766, 13394264, 13810851, 11746917, 24345867, 9551409, 5292000, 3648846, 13676337, 64805436, 619693, 11069776, 12838991, 9115769, 12558695, 13301056, 12160685, 12301430, 1051485, 4023683, 33427945, 11460162, 6389200, 8414269, 4859453, 4105843}

	for i := range polynomialPassageTimes {
		index := polynomialPassageTimes[i]
		t.Run(fmt.Sprintf("heuristic_polynomial-passage-times_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if !heuristicPolynomialPassageTimes(tm, 100000, 10, 5, 40, false) {
				t.Fail()
			}
		})
	}
}
