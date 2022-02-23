# BBChallenge Deciders

## News

**Update 09/02/22**. 1 253 418 machines were decided thanks to the backward-reasoning decider:
[https://github.com/bbchallenge/bbchallenge-deciders/tree/main/decider-backward-reasoning](https://github.com/bbchallenge/bbchallenge-deciders/tree/main/decider-backward-reasoning). As of this day there remains **2 323 786** undecided machines in the database.

**Update 29/01/22**. 73 857 622 translated-cyclers were decided (among machines that exceeded space limit) thanks to the translated-cyclers decider:
[https://github.com/bbchallenge/bbchallenge-deciders/tree/main/decider-translated-cyclers](https://github.com/bbchallenge/bbchallenge-deciders/tree/main/decider-translated-cyclers). As of this day there remains **3 577 204** undecided machines in the database.

**Update 28/01/22**. 11 229 238 cyclers were decided (among machines that exceeded time limit) thanks to the cyclers decider: [https://github.com/bbchallenge/bbchallenge-deciders/tree/main/decider-cyclers](https://github.com/bbchallenge/bbchallenge-deciders/tree/main/decider-cyclers).


## Downloading the full database

The database computed by bbchallenge-seed is available here:

- [https://dna.hamilton.ie/tsterin/all_5_states_undecided_machines_with_global_header.zip](https://dna.hamilton.ie/tsterin/all_5_states_undecided_machines_with_global_header.zip)
- [ipfs://QmPRjcK9mJz4UMwkzLNrVG3YtAUzdRHACuHbmpf1n1bfYr](ipfs://QmPRjcK9mJz4UMwkzLNrVG3YtAUzdRHACuHbmpf1n1bfYr)
- [https://ipfs.prgm.dev/ipfs/QmPRjcK9mJz4UMwkzLNrVG3YtAUzdRHACuHbmpf1n1bfYr](https://ipfs.prgm.dev/ipfs/QmPRjcK9mJz4UMwkzLNrVG3YtAUzdRHACuHbmpf1n1bfYr)

The format of the database is described here: [https://github.com/bbchallenge/bbchallenge-seed](https://github.com/bbchallenge/bbchallenge-seed).

Database (.zip) shasum: `8ba107bf1dbd7864865260d3eb8f07580646cb8c`.

## Downloading the index of currently undecided machines

An undecided index file is an ordered binary succession of uint32 (BigEndian) which corresponds to the IDs of the currently undecided machines in the above database.

The current size of the file is **9 295 144** bytes which corresponds to **(9 295 144)/4 = 2 323 786** machines. 

- [https://dna.hamilton.ie/tsterin/bb5_undecided_index](https://dna.hamilton.ie/tsterin/bb5_undecided_index)

Index file shasum: `68830ed2ba8f249a521586cf9f6230a09dd8bc7c`.
