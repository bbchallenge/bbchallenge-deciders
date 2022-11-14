# [Decider] Finite Automata Reduction

## Usage

To build it and view usage options, [install Rust](https://www.rust-lang.org/learn/get-started) and, in this directory:

```
$ cargo run --release -- --help
   ...
     Running `target/release/decider-finite-automata-reduction --help`
Usage: decider-finite-automata-reduction [-p <prover...>] [-l <limit...>] [-x <exclude...>] [-s] [--ip <ip>] [--port <port>] [-a <ad-hoc...>] [-d <db>] [-i <index>]

Decide TMs, using finite-state recognizers for their halting configurations.

Options:
  -p, --prover      prover(s) to use: see example
  -l, --limit       maximum search depth (DFA size) for corresponding prover
  -x, --exclude     exclude search depth (DFA size) for corresponding prover
  -s, --server      run as a server; clients will solve in parallel
  --ip              server IP address
  --port            server port
  -a, --ad-hoc      analyze only the given TMs/seeds and show any proofs found
  -d, --db          path to the DB file
  -i, --index       path to the undecided index file (used if present)
  --help            display usage information

Examples:
  # Analyze individual machines:
  $ decider-finite-automata-reduction -a 7410754 -a 1RB0RC_0LC1LE_0RD1LB_1RA1RC_0LB---
  # Parallel processing:
  $ decider-finite-automata-reduction --server --ip 10.0.0.1 -p direct -x 0 -l 8 -p mitm_dfa -x 8
  # And in other terminal tabs and/or computers on the network, once per CPU
  $ decider-finite-automata-reduction --ip 10.0.0.1
```

(After building, you can also pass command lines directly to the binary, e.g. `target/release/decider-finite-automata-reduction --help`.)

For BB(5), you'll want to have `../all_5_states_undecided_machines_with_global_header` and `../bb5_undecided_index` -- see `../README.md`.
(Alternate locations may be provided on the command line.)
I recommend the exact settings from the example command lines above: `--server --ip <your address> [--port <#>] -p direct -x 0 -l 8 -p mitm_dfa -x 8`.
The server command will wait for one or more client commands to start, then use them to solve machines in parallel.

In general, deeper searches succeed more often but take dramatically more time. 
(For example, `-p direct` runtimes per TM at depths 1-9  are ~2μs, ~8μs, ~50μs, ~300μs, ~4ms, ~70ms, ~1s, ~30s, ~20m.)
The program will solve most of the seed DB at low depth before going deeper.
The `mitm_dfa` prover covers a subset of the `direct` prover's search space, so it's redundant to use it at a depth where `direct` has been used.
(Up to depth ~5, it's also slower, but after depth ~8 it gains the upper hand.)

Results will be saved to the `output` subdir:

- `finite_automata_reduction.index`: solved DB indexes in [this format](https://bbchallenge.org/method#undecided-machines-index-file)
- `finite_automata_reduction.dvf`: a [Decider Verification File](https://github.com/TonyGuil/bbchallenge/blob/main/README), explained below.

### Individual machines

As in the first command-line example, the decider's `--ad-hoc` mode lets you solve one or more machines,
given as [Seed DB](https://bbchallenge.org/method#seed-database) indexes or [machine code](http://discuss.bbchallenge.org/t/standard-tm-text-format/60/17?u=uncombedcoconut) text.
It will output the data of any successful proof (explained below) as pretty-printed JSON.

### Larger BB(n) problems

The default build options work only for BB(5) and up to search depths of 12.
To double the depth limit (at the cost of some speed), add `--features u128` to the `cargo build` / `cargo run` command lines.
To change the number of TM states, edit `src/limits.rs`. The program will expect a seed DB format with a corresponding number of bytes per machine.
Beware, some unit tests assume `TM_STATES == 5`, and nearly all testing has been in BB(5) mode.

## How it works: practice

We search for the data of the following non-halting proof (which is easily checked, even if the search is complex).
In this section, I'll take it for granted that a complex definition is worth writing out. (That's the hard part.)
The next section will explain the background and motivations.

### Proof template

The proof, once we define its terms, will say: "the following finite-state machine, by construction, recognizes all halting tape configurations for this TM.
It doesn't recognize the initial configuration. Therefore the TM doesn't halt from its initial configuration."

The BB Challenge community sometimes calls this a "co-CTL" proof. More on that later.

Now for the definitions. The below is also expressed as commented Rust code, in `src/proof.rs`.

1. Define a finite-state machine of the following form, whose job is to scan TM configurations (WLOG left to right) and "recognize" some of them:
    1. Start "at the end" — on an infinite, this means an arbitrary position left of the head and the tape's finite non-zero-filled part.
    2. Read the tape as a [DFA (Q, Σ={0,1}, δ, q₀)](https://en.wikipedia.org/wiki/Deterministic_finite_automaton) up to (but excluding) the head position.
       To ensure the end state is well-defined, we require the DFA to ignore leading zeros, i.e., that δ(q₀, 0) = q₀.
    3. Transition to an [NFA (Q', Σ'={0,1}, δ', F)](https://en.wikipedia.org/wiki/Nondeterministic_finite_automata) whose state space Q' includes QˣQ™,
       i.e., has a specific NFA state (q, F) for whichever DFA end-state q and TM head-state F we get.
    4. Read the tape as an NFA, starting with the symbol under the TM head and "until the end".
       The TM configuration is *recognized* if the NFA can "end" in a state belonging to F.
       We again take the "end" to be an arbitrary point beyond all nonzero content.
       We again require that the choice of "end" doesn't matter, i.e., that the image δ(F, 0) equals F.
2. Also designate a *steady state* S⊆Q', i.e., a set of states such that δ(S, 0)⊇S and δ(S, 1)⊇S, which furthermore contains at least one state from F.
   In other words, if at any step the NFA could have reached all of S, that's also true on the next step and the configuration will ultimately be recognized.
3. Define the *closure* properties which effectively say, if the configuration after a TM step is recognized, the configuration before it is too:
    - In case of a right transition (r, F) ↦ (w, **R**, T), whose effect on the tape is to change the sequence `F@r` to `w T@`:
      ∀q∈Q: δ'((q, F), r) ∋ (δ(q, w), T).
    - In case of a left transition (r, F) ↦ (w, **L**, T), whose effect on the tape is to change the sequence `b F@r` to `T@b w`:
      ∀(q,b)∈Q×Σ: δ'((δ(q, b), F), r) ⊇ δ'(δ'((q, T), b), w).
4. In case of a halt rule for (r, F), require an NFA transition to the steady state (thus guaranteeing recognition): 
   ∀q∈Q: δ'((q, F), r) ⊇ S.
5. Finally, require the initial configuration not to be recognized: (q₀, A)∉F.

### Proof data format

In a computer representation of the above, we number the DFA states, identifying q₀ with 0, and number the NFA states, identifying (q, F) with 5q+f
(in the BB(5) case, of course; and similarly for other values of 5.)

We represent a DFA as a simple nested list, [[δ(0, 0), δ(0, 1)], [δ(1, 0), δ(1, 1)], …, [δ(n, 0), δ(n, 1)]].

In a [Decider Verification File](https://github.com/TonyGuil/bbchallenge/blob/main/README), this is flattened to a sequence of 2n bytes.

To represent transitions, sets of accepted states, and sets of reachable states, we use a well-known
[matrix characterization of automata](https://planetmath.org/matrixcharacterizationsofautomata):
they become matrices, row vectors, and column vectors, respectively.
Boolean matrices suffice. Vectors are represented as bitfields. Matrices are [row-major](https://en.wikipedia.org/wiki/Row-_and_column-major_order).

This explains the JSON objects that come back from a `--ad-hoc` command to the decider.
That's a lot for a data format, so our Verification Files actually only include the DFA.
As it turns out, the rest of the proof can be reconstructed almost instantly.

### Code and test structure

The program's code is divided into four modules:

- core: defines the data and correctness criteria of a proof, as above (plus essentials like the TM, DFA, NFA, and boolean vector/matrix representations).
  This part is extensively documented and unit tested.
- io: defines how to work with the file formats involved. This part is merely battle-tested and reasonably documented.
- provers: the secret sauce: not actually secret, obviously, but it's impossible to make these "so simple that there are obviously no deficiencies".
           Instead, this code is restricted to outputting Proof objects which are checked before the decider considers a machine solved.
- driver: utility code for handling such concerns as distributed processing and progress monitoring.

In terms of correctness, the most important part is `core/proof.rs`.
As usual for Rust programs — unit tests are in-line, so `core/proof.rs` also has the tests that bad proofs are rejected.

### Verification File

In our [Decider Verification Files](https://github.com/TonyGuil/bbchallenge/blob/main/README), we specify the following:
- `DeciderType` = 10
- `InfoLength` is variable, but corresponds to
- `DeciderSpecificInfo` contains 1 byte for the direction (0 if as above the FSM is to scan left-to-right, 1 if reversed),
  then 2n bytes for a DFA transition table as described above
- Warning: The DVF format has an `nEntries` header. This decider operates in append mode and lets that become stale.
  It may also write multiple proof records for the same `SeedDatabaseIndex`. These are quirks to fix in post-processing.

## How it works: theory

### CTL and Co-CTL
The [Closed Tape Language](https://www.sligocki.com/2022/06/10/ctl.html) technique analyzes TM behavior using regular languages.
Its potential has been well-known to the community.

Compared to the definitions above, this is *complemented* (CTL recognizes the start and avoids recognizing halting configurations).
It is also *time-reversed* (if the configuration before a TM step is recognized, the configuration after it is too).
Indeed, the complement of a language making a "co-CTL proof" work makes a "CTL proof" work, and vice-versa: this is just contraposition.
(Before implies after iff not after implies not before.)

These two techniques thus have precisely the same power, though some proof searches may be better at finding one than the other, and there may be overhead
when translating between the two proof styles.

### Automata connections

Picture a Turing Machine as a two-stack machine: a fixed head pushes and pulls on two half-tapes.
If we split the left "stack" configurations into finitely many classes, and consider the transitions between (class, head, right-tape) tuples, we get a nondeterministic stack machine.
The following paper builds a finite state machine which recognizes configurations from which a stack machine can halt:

[[BEM97](https://www.irif.fr/~abou//BEM97.pdf)] Bouajjani, A., Esparza, J., & Maler, O. (1997, July). Reachability analysis of pushdown automata: Application to model-checking.
In International Conference on Concurrency Theory (pp. 135-150). Springer, Berlin, Heidelberg.

If you have a (regular) (co-) CTL, it has a finite-state recognizer.
The recognizer classifies left half-tapes by the state reached.
The classification produces a stack machine.
Its exact solution is again a regular co-CTL, so these characterizations are also interchangeable.

### Quotients of Turing Machines
Let L be a co-CTL for a TM.

As mentioned in the "Proof Template" section — for a *tape* language to make sense we must require it to be (reverse) closed under TM transitions *and*
invariant under zero-padding.

Let \~ be the [left syntactic equivalence](https://en.wikipedia.org/wiki/Syntactic_monoid#Syntactic_equivalence) relation it induces on bit-strings.

Let `[u]` denote the \~-equivalence class of a bit-string u, and v be another bit-string.

Define TM/\~ as a machine with configurations `[u] S@v`, and transitions `[u] S@v` ↦ `[u'] S'@v'` for each valid TM step `0̅0 u S@v 0̅0` ↦ `0̅0 u S@v' 0̅0`.

Define halting for TM/\~ as for TM, and L(TM/\~) — the language TM/\~ accepts — to contain the configurations from which a halt is reachable.

When we view the `[u] S@` as states and the `v` as a stack, [BEM97] says TM/\~ is a "pushdown system" and L(TM/\~) is recognized by a certain finite automaton.

Thus, L' = { `u S@v` | TM/\~ may accept `[u] S@v 0^n` for some n } is a regular language we can recognize.

L' is also a co-CTL: if it accepts `u S@v` after one step, then TM/\~ accepts it after one step (and zero-padding), so ditto before the one step, so L' accepts `u S@v`.

L' accepts halting words. If L does too, then L'⊆L, so we've recovered an equal or finer co-CTL.

### Direct recognizer for L(TM/\~)
While [BEM97] is a nice tool, it's helpful to derive the L(TM/\~) recognizer directly.
So: let’s decide if a configuration starting with `[u] S@b₀` can lead to a halt, by considering how:
it would via a finite transition sequence, which either reads the next bit b₁ at some point or first hits a halt-transition.
If the former is possible, that’s an unconditional yes;
the latter is possible iff `[u] S@b₀v` may lead to some `[u'] S'@v` (via transitions which ignore the `v`) and `[u'] S@b₁b₂…` can lead to a halt.
This gives an inductively defined recognizer, which operates as an NFA on the state space {HALT} ∪ Q×Q™.
That last paragraph defines its transitions mathematically. We can make the definition effective by defining *them* inductively:

* In case of a halt rule for (r, F): for any `[u]`, `[u] F@, r`↦`HALT` is an `r`-transition. (`HALT` simply transitions to itself.)
* In case of a right transition (r, F) ↦ (w, **R**, T), whose effect on the tape is to change the sequence `F@r` to `w T@`:
  for any `[u]`, `[u] F@, r`↦`[uw] T@` is an `r`-transition.
* In case of a left transition (r, F) ↦ (w, **L**, T), whose effect on the tape is to change the sequence `b₀ F@r` to `T@b₀ w`:
  whenever `[u] T@, b₀, w`↦`[u'] T'@` is a composition of `b₀`- and `w`-transitions, there’s an `r`-transition from `[ub₀] F@, r`↦`[u'] T'@`.

To compute the recognizing NFA’s transition relation, we may close an initially empty relation under these rules.

This is what `direct.rs` does.

Consequently, direct.rs is also able to complete a proof from just the DFA side.
This makes it a useful partner to the verifier, even if it's untrusted: if someone claims a DFA works in a proof, the verifier can ask this algorithm to complete it, then check the completed proof.

### Meet-in-the-Middle DFAs
There is another finite state machine architecture to consider for a tape language recognizer, which is appealing because it's very simple:
Any regular tape language remains regular when reversed.
Thus, there exist DFAs recognizing the original and reversed language.
If we delete everything except the `0`- and `1`-transitions, we get finite-state classifiers for the left and right halves of the tape (in both cases excluding the bit under the head).

Here, too, we can consider the connection between DFAs and semantic equivalence:
we see that any tape configuration's membership in the language is determined by what the left-tape DFA does, what the right-tape DFA does, and the head configuration (state and bit underneath).

If we define a finite-state recognizer which operates this way — runs both tape halves through a DFA, and checks this triple against an accepted set —
we see that the relevant closure conditions are fairly easy to formulate.

There are two ways to connect this construction back to the DFA+NFA constructions discussed above: either reverse the arrows on the right-tape DFA (making it nondeterministic) and change the accept states into a bunch of transitions, or simply throw away the right-tape DFA and apply the TM/~ construction.

The decider in `mitm_dfa.rs` follows on a path first set by others — see there for details — and sets up the closure conditions for the MitM-DFA as a boolean satisfiability problem.
Thanks and credit go to:

- @djmati1111 (https://github.com/colette-b/bbchallenge)
- @Mateon1 (https://discuss.bbchallenge.org/u/mateon1)
