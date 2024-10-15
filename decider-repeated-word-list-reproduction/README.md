# README

The Repeated Word List (RepWL) technique was introduced by mxdys in [Coq-BB5](<https://github.com/ccz181078/Coq-BB5>), README.

The technique was reproduced by savask, with more explanations in comments: https://github.com/savask/turing/blob/main/Repeat.hs

## Usage

```
python3.12 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

Decide one machine:

```
python decider.py -m 1RB1LC_1LA1RD_1LD1LA_1RA1RE_---1RB -b 6 -r 2
```

Decide several machines:

```
python decider.py -f Coq-BB5_RWL_parameters_bbchallenge_format.txt --no-print-cert
```

All commands:
```
python decider.py --help
usage: decider.py [-h] [-m TM] [-b BLOCK_SIZE] [-r PLUS_REPEAT_THRESHOLD] [-f FILE_MACHINES_LIST]
                  [-t BLOCK_SIMULATION_TIMEOUT] [-M MAX_VISITED_REGEX] [--verbose | --no-verbose]
                  [--print-cert | --no-print-cert] [--print-params-stats | --no-print-params-stats]

Repeated Word List decider (RepWL)

options:
  -h, --help            show this help message and exit
  -m TM, --tm TM        The transition function of the Turing machine in the bbchallenge format,
                        e.g. 0RB---_1LC1RC_1LD0RA_1RE0LD_1RA1RE
  -b BLOCK_SIZE, --block-size BLOCK_SIZE
                        The block size to use for the decider
  -r PLUS_REPEAT_THRESHOLD, --plus-repeat-threshold PLUS_REPEAT_THRESHOLD
                        The threshold for the plus operator
  -f FILE_MACHINES_LIST, --file-machines-list FILE_MACHINES_LIST
                        The file containing the list of Turing machines with parameters
  -t BLOCK_SIMULATION_TIMEOUT, --block-simulation-timeout BLOCK_SIMULATION_TIMEOUT
                        The block simulation timeout
  -M MAX_VISITED_REGEX, --max-visited-regex MAX_VISITED_REGEX
                        The maximum number of visited regex tapes
  --verbose, --no-verbose
                        Prints debug information
  --print-cert, --no-print-cert
                        Prints the RepWL non-halt certificate(s)
  --print-params-stats, --no-print-params-stats
                        In case of a file with Turing machines and parameters, print statistics
                        about the parameters (min, max, avg)
```

## Statistics about Coq-BB5 RepWL Parameters

    -block_size: min=1, max=57, avg=4.1
    -plus_repeat_threshold: min=2, max=4, avg=2.4