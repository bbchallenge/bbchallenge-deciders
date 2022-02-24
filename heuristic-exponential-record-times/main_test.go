package main

import (
	"fmt"
	"io/ioutil"
	"testing"

	bbc "github.com/bbchallenge/bbchallenge-go"
)

func TestSimulateAndGetPassageTimes(t *testing.T) {
	tm := bbc.GetBB5Winner()
	recordTimes := simulateAndGetRecordTimes(tm, 10000000, 200)

	// These were computed from Python notebook
	// https://github.com/bbchallenge/bbchallenge-py/blob/main/Heuristic%20-%20Exponential%20Record%20Times.ipynb
	correctRecordTimesR := []int{0, 1, 2, 3, 22, 23, 24, 105, 106, 107, 400, 401, 402, 1337, 1338, 1339, 4144, 4145, 4146, 12183, 12184, 12185}

	correctRecordTimesL := []int{0, 7, 14, 15, 34, 41, 42, 59, 60, 87, 88, 127, 130, 131, 144, 145, 168, 169, 202, 203, 246, 247, 300, 301, 364, 365, 440, 443, 444, 457, 458, 481, 482, 515, 516, 559, 560, 613, 614, 677, 678, 751, 752, 835, 836, 929, 930, 1033, 1034, 1147, 1148, 1271, 1272, 1407, 1410, 1411, 1424, 1425, 1448, 1449, 1482, 1483, 1526, 1527, 1580, 1581, 1644, 1645, 1718, 1719, 1802, 1803, 1896, 1897, 2000, 2001, 2114, 2115, 2238, 2239, 2372, 2373, 2516, 2517, 2670, 2671, 2834, 2835, 3008, 3009, 3192, 3193, 3386, 3387, 3590, 3591, 3804, 3805, 4028, 4029, 4264, 4271, 4272, 4289, 4290, 4317, 4318, 4355, 4356, 4403, 4404, 4461, 4462, 4529, 4530, 4607, 4608, 4695, 4696, 4793, 4794, 4901, 4902, 5019, 5020, 5147, 5148, 5285, 5286, 5433, 5434, 5591, 5592, 5759, 5760, 5937, 5938, 6125, 6126, 6323, 6324, 6531, 6532, 6749, 6750, 6977, 6978, 7215, 7216, 7463, 7464, 7721, 7722, 7989, 7990, 8267, 8268, 8555, 8556, 8853, 8854, 9161, 9162, 9479, 9480, 9807, 9808, 10145, 10146, 10493, 10494, 10851, 10852, 11219, 11220, 11597, 11598, 11985, 11986, 12385, 12388, 12389, 12402, 12403, 12426, 12427, 12460, 12461, 12504, 12505, 12558, 12559, 12622, 12623, 12696, 12697, 12780, 12781, 12874, 12875, 12978}

	for i, time := range correctRecordTimesR {
		if i >= len(recordTimes[bbc.R]) || time != recordTimes[bbc.R][i] {
			t.Fail()
		}
	}

	for i, time := range correctRecordTimesL {
		if i >= len(recordTimes[bbc.L]) || time != recordTimes[bbc.L][i] {
			t.Fail()
		}
	}
}

func TestOneExponentialRecordTimes(t *testing.T) {
	n := 12345394
	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}
	tm, err := bbc.GetMachineI(DB[:], n, true)
	if err != nil {
		t.Fail()
	}
	if !heuristicExponentialPassageTimes(tm, 10000000, 200, 8, 5, 5, true) {
		t.Fail()
	}
}

func TestHeuristicExponentialRecordTimes(t *testing.T) {

	DB, err := ioutil.ReadFile(DB_PATH)
	if err != nil {
		t.Fail()
	}

	if heuristicExponentialPassageTimes(bbc.GetBB5Winner(), 10000000, 200, 8, 5, 5, false) {
		t.Fail()
	}

	notExponentialRecordTimes := []int{5643626, 3565349, 2108244, 13474013, 9755023, 8103390, 9225630, 3169967, 10363739, 12223654, 13090023, 11225780, 11544059, 1507193, 13795582, 12244006, 13930014, 11334525, 1028021, 1566023, 269038, 828479, 8497788, 6136947, 11635490, 6398375, 9703986, 5203553, 6843551, 6964220, 11598427, 13576089, 11519070, 9913985, 11916339, 14086216, 8439154, 87940465, 11089226, 4482180, 4284, 4468110, 12189145, 12206912, 46840666, 8416617, 13440160, 2498135, 12285970, 14218336, 8634375, 1048062, 13977714, 8457777, 4745553, 1153972, 13802273, 11388271, 5383040, 1507193, 9786806, 5702127, 7883753, 4557119, 33672339, 3370060, 402959, 11515638, 6399980, 1960734, 8818662, 6300366, 8900440, 6092203, 11980798, 9813712, 4058620, 3418516, 14044691, 9312210, 172282, 582211, 12199575, 12594284, 2731122, 7716308, 12292720, 12549707, 26414953, 2479142, 6152467, 38111044, 662317, 526738, 5328524, 13649989, 4208656, 13784545, 5928209, 5532891, 12876855, 937083, 4726380, 67780089, 13104130, 1015814, 12215361, 76952003, 205766, 13394264, 13810851, 11746917, 24345867, 9551409, 5292000, 3648846, 13676337, 64805436, 619693, 11069776, 12838991, 9115769, 12558695, 13301056, 12160685, 12301430, 1051485, 4023683, 33427945, 11460162, 6389200, 8414269, 4859453, 4105843, 4824149, 47437564, 7431789, 10613000, 40851850, 54221304, 11710205, 11726152, 72903966, 10039500, 7850055, 4966400, 13754164, 7658575}
	for i := range notExponentialRecordTimes {
		index := notExponentialRecordTimes[i]
		t.Run(fmt.Sprintf("heuristic_not-exponential-record-times_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if heuristicExponentialPassageTimes(tm, 10000000, 200, 8, 5, 5, false) {
				t.Fail()
			}
		})
	}

	// smart counters expect 11776032
	// see https://github.com/bbchallenge/bbchallenge-py/blob/main/Heuristic%20-%20Exponential%20Record%20Times.ipynb
	exponentialRecordTimes := []int{11184382, 7931183, 7695094, 8594059, 12877334, 4631164, 4807678, 467941, 12345394, 1236862, 14009904, 12418684, 10269203}

	for i := range exponentialRecordTimes {
		index := exponentialRecordTimes[i]
		t.Run(fmt.Sprintf("heuristic_polynomial-exponential-record-times_%d", index), func(t *testing.T) {
			tm, err := bbc.GetMachineI(DB[:], index, true)
			if err != nil {
				t.Fail()
			}
			if !heuristicExponentialPassageTimes(tm, 10000000, 200, 8, 5, 5, false) {
				t.Fail()
			}
		})
	}
}
