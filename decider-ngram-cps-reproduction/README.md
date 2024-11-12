# n-gram Closed Position Set (NGramCPS)

This method reproduces Nathan Fenner's [n-GRAM CPS](https://github.com/Nathan-Fenner/bb-simple-n-gram-cps). `NGramCPS` and extensions which are not yet present in this reproduction are extensively used in [Coq-BB5](https://github.com/ccz181078/Coq-BB5).

Note that the reproduction is not 100% exact, one difference (probably the only one), was pointed out by Justin Blanchard on [bbchallenge's discord server](https://discord.com/channels/960643023006490684/1028747034066427904/1237879123498766528) ([server invite](https://discord.gg/3uqtPJA9Uv)):

```
A note about reproducing n-gram CPS: there are 2 natural approaches:

1. When w_L HEAD w_R is a local context, w_L counts as a left-tape n-gram, ditto w_R on the right.
2. The n-grams to count are those seen 1 cell away from the head; thus, they're the possible "glue" words from a local context to the rest of the tape.

Approach 1 has a minor aesthetic upside: the recorded n-gram sets are exactly the n-symbol substrings that can (potentially) appear on the respective side of the tape. This simplifies the theory and extraction of FAR certs.

Approach 2 is a completely free performance optimization, though. The decider is already tracking the info it needs, and the search/storage are smaller.  It's what Nathan's decider does and what its readme describes.
Approach 1 regains the power (set of machines decided) of approach 2 if you increase n by 1. Approach 2 still lets you extract FAR certs from an n-gram set if you take a line graph (or take local contexts into account).

A test case is https://bbchallenge.org/1RB---_0RC0LB_1RD1RA_0LE0RD_1LB1LE, solved at n=4 in approach 2 and at n=5 in approach 1.
```

This code implements `Approach 1` while Nathan Fenner's code implements `Approach 2` as confirmed by the test suggested by Justin (failure at radius 4, success at radius 5):

```
python decider.py -m 1RB---_0RC0LB_1RD1RA_0LE0RD_1LB1LE -r 4 --print-cert
Failure: 1RB---_0RC0LB_1RD1RA_0LE0RD_1LB1LE may halt
```

```
python decider.py -m 1RB---_0RC0LB_1RD1RA_0LE0RD_1LB1LE -r 5 --print-cert
Reachable L-ngrams: {'00010', '00001', '00101', '00100', '00000', '11011', '01011', '10110', '01101'}
Reachable R-ngrams: {'00010', '00110', '00101', '00100', '00000', '10010', '11001', '01100', '01001', '11000', '11011', '01000', '01011', '10000', '10110', '01101'}
Reachable local contexts (140): {'00010 [D1] 00101', '00100 [D0] 01001', '00001 [B0] 10010', '11011 [B0] 10110', '11011 [B1] 10000', '00101 [B1] 01001', '00010 [C0] 01101', '00000 [C0] 11001', '01101 [B1] 11000', '11011 [B1] 10010', '00101 [D0] 11000', '01101 [D0] 11001', '11011 [E0] 10000', '00101 [D0] 01000', '00010 [B1] 00100', '00000 [B0] 01100', '00101 [B1] 11000', '11011 [B0] 10010', '01101 [A0] 01011', '00100 [D0] 01000', '01011 [B0] 10010', '00010 [C1] 00100', '00000 [B1] 11001', '10110 [C0] 00000', '00000 [A0] 00000', '00010 [E0] 00100', '00001 [E0] 10000', '10110 [C1] 01100', '11011 [E0] 10010', '11011 [B0] 10000', '00000 [C0] 11011', '10110 [E1] 00100', '00101 [D0] 00000', '00101 [D0] 01011', '01101 [B1] 01001', '00000 [B1] 11000', '10110 [E1] 00000', '11011 [B0] 00000', '00100 [D0] 11001', '10110 [C1] 00000', '10110 [C0] 01100', '00001 [B0] 00110', '01101 [D0] 01001', '01011 [E0] 10110', '00101 [A0] 01011', '00100 [D0] 11000', '01101 [A0] 01001', '00100 [D0] 00000', '00010 [C0] 00000', '01011 [B0] 00010', '10110 [C1] 00101', '00010 [E0] 00000', '01101 [A0] 11011', '00101 [A0] 11000', '01101 [D0] 00000', '00010 [E0] 01101', '00100 [D0] 01011', '00001 [B0] 10110', '00100 [D0] 11011', '00010 [C1] 00101', '01011 [B0] 00000', '01101 [B1] 11011', '01011 [B1] 10010', '00010 [E1] 01100', '01101 [B1] 01011', '10110 [B1] 01100', '01101 [D0] 11000', '10110 [C1] 01101', '00101 [A0] 11011', '01101 [B1] 01000', '00010 [C1] 01101', '01011 [E0] 10010', '00010 [D1] 00100', '00101 [A0] 11001', '00010 [D1] 01100', '10110 [B1] 01101', '01101 [A0] 11000', '01011 [B0] 10110', '00010 [B1] 00101', '00101 [B1] 11001', '00101 [D0] 01001', '00010 [E1] 00101', '00101 [D0] 11011', '00001 [B0] 00010', '01101 [B1] 11001', '00101 [B1] 11011', '00010 [C0] 00101', '00101 [A0] 01000', '00001 [E0] 10110', '00101 [B1] 01011', '11011 [B0] 00110', '01101 [D0] 11011', '00010 [C1] 00000', '10110 [E1] 01101', '10110 [B1] 00100', '00010 [D1] 01101', '10110 [C0] 01101', '00010 [E1] 00100', '11011 [B0] 00010', '00001 [B0] 10000', '01101 [A0] 01000', '00010 [E1] 00000', '10110 [B1] 00101', '00001 [D1] 10110', '01101 [D0] 01000', '00010 [B1] 01101', '00010 [E0] 01100', '10110 [E1] 00101', '00001 [D1] 10010', '00001 [B0] 00000', '00000 [B0] 01101', '00101 [B1] 01000', '01011 [B0] 00110', '00001 [D1] 10000', '00010 [D1] 00000', '00000 [B1] 11011', '00010 [E1] 01101', '10110 [C0] 00100', '00001 [E0] 10010', '01101 [D0] 01011', '01101 [A0] 00000', '01101 [A0] 11001', '00101 [A0] 00000', '01011 [B1] 10000', '01011 [B1] 10110', '00010 [E0] 00101', '00010 [C0] 01100', '11011 [E0] 10110', '01011 [E0] 10000', '00010 [C0] 00100', '10110 [C0] 00101', '00101 [D0] 11001', '00000 [C0] 11000', '00101 [A0] 01001', '10110 [C1] 00100', '00010 [C1] 01100', '01011 [B0] 10000', '10110 [E1] 01100', '11011 [B1] 10110', '00010 [B1] 01100'}
Success: 1RB---_0RC0LB_1RD1RA_0LE0RD_1LB1LE does not halt
```

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
                        Prints the reached ngrams and local contexts
  --print-final-result, --no-print-final-result
                        Prints whether the machine does not halt or may halt
```