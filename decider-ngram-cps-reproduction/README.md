# n-gram Closed Position Set (NGramCPS)

This method reproduces Nathan Fenner's [n-GRAM CPS](https://github.com/Nathan-Fenner/bb-simple-n-gram-cps).

`NGramCPS` and extensions which are not yet present in this reproduction are extensively used in [Coq-BB5](https://github.com/ccz181078/Coq-BB5).

## Examples

```
python decider.py -m 1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA -r 2
Success: 1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA does not halt
```

```
python decider.py -m 1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA -r 2 --print-cert
Reachable L-ngrams: {'01', '10', '00', '11'}
Reachable R-ngrams: {'10', '01', '00'}
Reachable local contexts (41): {'01 [B1] 00', '11 [D0] 00', '11 [E1] 00', '01 [B0] 00', '11 [E0] 01', '01 [E1] 01', '10 [C1] 00', '00 [E0] 01', '10 [A0] 01', '00 [E1] 01', '01 [B0] 01', '10 [C0] 10', '10 [E1] 01', '00 [C1] 01', '00 [D0] 10', '11 [C0] 01', '10 [A0] 00', '10 [B0] 00', '01 [D0] 10', '11 [D0] 10', '01 [D1] 01', '10 [D0] 10', '10 [E0] 01', '00 [A0] 10', '01 [B1] 01', '01 [B0] 10', '00 [C0] 10', '01 [E1] 00', '11 [E1] 01', '01 [E0] 01', '10 [C1] 01', '01 [C0] 00', '11 [C0] 00', '10 [B0] 10', '10 [B0] 01', '01 [C0] 01', '01 [D1] 00', '00 [A0] 00', '11 [D0] 01', '00 [C1] 00', '10 [A0] 10'}
Success: 1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA does not halt
```

```
python decider.py -m 1RB---_1LC0RB_1RE0LD_0RC0LD_1RA1RC -r 6
Failure: 1RB---_1LC0RB_1RE0LD_0RC0LD_1RA1RC may halt
```

## Usage

```
usage: decider.py [-h] [-m TM] [-r RADIUS] [-M MAX_CONTEXT_COUNT] [--verbose | --no-verbose]
                  [--print-cert | --no-print-cert] [--print-final-result | --no-print-final-result]

n-gram Closed Position Set (NGramCPS)

options:
  -h, --help            show this help message and exit
  -m TM, --tm TM        The transition function of the Turing machine in the bbchallenge format, e.g. 1RB---
                        _0LC0RB_1RD1LD_0LE0RA_0RC0RA
  -r RADIUS, --radius RADIUS
                        Size of the ngrams on both sides of the head, e.g. 2
  -M MAX_CONTEXT_COUNT, --max-context-count MAX_CONTEXT_COUNT
                        The maximum number of visited local contexts
  --verbose, --no-verbose
                        Prints debug information
  --print-cert, --no-print-cert
                        Prints the RepWL non-halt certificate(s)
  --print-final-result, --no-print-final-result
                        Prints whether the machine does not halt or may halt
```