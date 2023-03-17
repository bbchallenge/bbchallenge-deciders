# Finite Automata Reduction (reproduction)

This reproduces [the work of Justin Blanchard](https://github.com/UncombedCoconut/bbchallenge-deciders/tree/finite-automata-reduction/decider-finite-automata-reduction) (and also [here](https://github.com/UncombedCoconut/bbchallenge-nfa-verification)) on Finite Automata Reduction, a technique for automatically deciding whether some Turing machines halt or not.

For more details on the technique please refer to [bbchallenge's paper](https://github.com/bbchallenge/bbchallenge-proofs) (you currently have to select the FAR branch of the github repo). 

Please also refer to:

- [https://github.com/UncombedCoconut/bbchallenge-deciders/tree/finite-automata-reduction/decider-finite-automata-reduction](https://github.com/UncombedCoconut/bbchallenge-deciders/tree/finite-automata-reduction/decider-finite-automata-reduction)
- [https://github.com/UncombedCoconut/bbchallenge-nfa-verification](https://github.com/UncombedCoconut/bbchallenge-nfa-verification)
- [https://github.com/TonyGuil/bbchallenge/tree/main/FAR](https://github.com/TonyGuil/bbchallenge/tree/main/FAR)

You can feed to the verifier DVF files (see this [README](https://github.com/UncombedCoconut/bbchallenge-deciders/tree/finite-automata-reduction/decider-finite-automata-reduction)) containing NFA-DFA Finite Automata Reduction (FAR) proofs, such as the uncompressed version of [**this file**](https://github.com/UncombedCoconut/bbchallenge-nfa-verification/blob/4da6899808be8922fb6872b48fd17c35856858fb/dfa_nfa_proofs.dvf.zst) (you need to use facebook's `zstd` for decompressing: [https://github.com/facebook/zstd](https://github.com/facebook/zstd)).

## Setup

```
python3.10 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

## Verifier usage

Verify machines in a DVF file:

- `python verifier_FAR_NFA_DFA.py --dvf resources/dfa_nfa_proofs.dvf --verbose`: verifies all the proofs of `dfa_nfa_proofs.dvf`
- `python verifier_FAR_NFA_DFA.py --dvf resources/dfa_nfa_proofs.dvf -e 6`: verifies only the 6th entry of the dvf file (counting from 0)
- `python verifier_FAR_NFA_DFA.py --dvf resources/dfa_nfa_proofs.dvf -e 6 --graphviz | dot -Tsvg -o output/nfa.svg`: outputs an SVG image of the NFA of the 6th entry of the dvf file (the entry is also verified)

```
usage: verifier_FAR_NFA_DFA.py [-h] [-d DB] --dvf DVF [-e ENTRY] [--graphviz] [--verbose]

options:
  -h, --help            show this help message and exit
  -d DB, --db DB        path to the DB file
  --dvf DVF             path to the verification file
  -e ENTRY, --entry ENTRY
                        verifies only the specified entry of the dvf file
  --graphviz            if an entry is selected with -e this will output the graphviz code of the NFA (the
                        entry is also verified)
  --verbose             enables logging
```

## Solver usage

`solver_FAR_NFA_direct.py` contains the function `solve_FAR_NFA_from_DFA` which implements the direct FAR algorithm (see references in introduction) for getting the full NFA associated to a machine from its left-hand side or right-hand side DFA.

Whereas this function is mainly used by the (yet to come) decider, the script can be used on its own to check that all the NFAs present in a DFA-NFA file are correct (i.e. each NFA is computed from its DFA and the results are compared with the NFA in the dvf file):

- `python solver_FAR_NFA_direct.py --dvf resources/dfa_nfa_proofs.dvf --check-dvf --verbose`

```
usage: solver_FAR_NFA_direct.py [-h] [-d DB] --dvf DVF [-c CORES] [-cdvf] [--verbose]

options:
  -h, --help            show this help message and exit
  -d DB, --db DB        path to the DB file
  --dvf DVF             path to the verification file
  -c CORES, --cores CORES
                        number of cores on which to parallelize the run, default on your machine is 8
  -cdvf, --check-dvf    solves the nfas of the dfas of a dfa-nfa dvf files and make sure they correspond to the nfas in the file
  --verbose             enables logging
```
