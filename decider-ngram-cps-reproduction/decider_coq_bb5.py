import argparse
from typing import Literal
from NGramCPS_coq_bb5 import NGramCPS_decider, TM, Transition, l2i, St

Σ_binary = Literal["0", "1"]
Σ0 = "0"


def TM_binary(tm_bbchallenge: str) -> TM[Σ_binary]:
    """Standard binary-alphabet TM transition table.
    Example:
    >>> tm = TM_binary("1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA")
    >>> tm(0, "0")
    1RB
    >>> tm(0, "1") is None
    True
    """

    def tm(state: St, symbol: Σ_binary) -> Transition[Σ_binary] | None:
        nonlocal tm_bbchallenge
        tm_bbchallenge = tm_bbchallenge.replace("_", "")
        write, move, new_state = tm_bbchallenge[
            state * 6 + 3 * int(symbol) : state * 6 + 3 * int(symbol) + 3
        ]
        if new_state == "-":
            return None
        return Transition(write, move, l2i(new_state))

    return tm


""" Here are the Coq-BB5 definitions that we reproduce for `TM_history` and `TM_history_LRU`:

Definition Σ_history:Set :=
  Σ*(list (St*Σ)).

Definition Σ_history_0:Σ_history := (Σ0,nil).

Definition TM_history(n:nat)(tm:TM Σ):TM Σ_history :=
  fun s i =>
  let (i0,i1):=i in
  match tm s i0 with
  | Some tr =>
    let (s',d,o0):=tr in
    Some {|
      nxt := s';
      dir := d;
      out := (o0,firstn n ((s,i0)::i1));
    |}
  | None => None
  end.

Definition StΣ_neb(x1 x2:St*Σ) :=
    let (s1,i1):=x1 in
    let (s2,i2):=x2 in
    if St_eqb s1 s2 then negb (Σ_eqb i1 i2) else true.

Definition TM_history_LRU(tm:TM Σ):TM Σ_history :=
  fun s i =>
  let (i0,i1):=i in
  match tm s i0 with
  | Some tr =>
    let (s',d,o0):=tr in
    Some {|
      nxt := s';
      dir := d;
      out := (o0,((s,i0)::(filter (StΣ_neb (s,i0)) i1)));
    |}
  | None => None
  end.
"""

Σ_history = tuple[Σ_binary, tuple[tuple[St, Σ_binary], ...]]
Σ_history0 = (Σ0, ())


def TM_history(tm_bbchallenge: str, history_length: int) -> TM[Σ_history]:
    """
    Using the alphabet `Σ_history`, the TM stores on each tape cell the last `history_length` state-symbol pairs seen at that cell.

    Example:
    >>> tm = TM_history("1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA", 2)
    >>> tm(0, Σ_history0)
    ('1', ((0, '0'),))RB
    >>> tm(1, ('1', ((0, '0'),)))
    ('0', ((1, '1'), (0, '0')))RB
    >>> tm(1, ('0', ((1, '1'), (0, '0'))))
    ('0', ((1, '0'), (1, '1')))LC
    >>> tm(1, ('0', ((1, '0'), (1, '1'), (0, '0'))))
    ('0', ((1, '0'), (1, '0')))LC

    >>> tm(0, ('1', ((0, '0'),))) is None
    True
    """

    def tm(state: St, symbol_history: Σ_history) -> Transition[Σ_history] | None:
        nonlocal tm_bbchallenge, history_length
        tm_bbchallenge = tm_bbchallenge.replace("_", "")

        symbol_binary: Σ_binary = symbol_history[0]

        write_binary, move, new_state = tm_bbchallenge[
            state * 6 + 3 * int(symbol_binary) : state * 6 + 3 * int(symbol_binary) + 3
        ]
        if new_state == "-":
            return None

        # (o0,firstn n ((s,i0)::i1))
        curr_history = symbol_history[1]
        new_history = ((state, symbol_binary),) + curr_history
        write_history = (write_binary, new_history[:history_length])

        return Transition(write_history, move, l2i(new_state))

    return tm


def TM_history_LRU(tm_bbchallenge: str) -> TM[Σ_history]:
    """LRU stands for Least Recent Usage.
       Using the alphabet `Σ_history`, the TM stores on each tape cell
       the set of state-symbol pairs seen at that cell,
       in order of when it was seen last, the most recent first.

    Example:
    >>> tm = TM_history_LRU("1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA")
    >>> tm(0, Σ_history0)
    ('1', ((0, '0'),))RB
    >>> tm(1, ('1', ((0, '0'),)))
    ('0', ((1, '1'), (0, '0')))RB
    >>> tm(1, ('0', ((2, '1'), (1, '0'))))
    ('0', ((1, '0'), (2, '1')))LC

    >>> tm(0, ('1', ((0, '0'),))) is None
    True
    """

    def tm(state: St, symbol_history: Σ_history) -> Transition[Σ_history] | None:
        nonlocal tm_bbchallenge
        tm_bbchallenge = tm_bbchallenge.replace("_", "")

        symbol_binary: Σ_binary = symbol_history[0]

        write_binary, move, new_state = tm_bbchallenge[
            state * 6 + 3 * int(symbol_binary) : state * 6 + 3 * int(symbol_binary) + 3
        ]
        if new_state == "-":
            return None

        def StΣ_neb(x1: tuple[St, Σ_binary], x2: tuple[St, Σ_binary]) -> bool:
            s1, i1 = x1
            s2, i2 = x2
            if s1 == s2:
                return i1 != i2
            return True

        # (o0,((s,i0)::(filter (StΣ_neb (s,i0)) i1)))
        curr_history_LRU = symbol_history[1]
        new_history = ((state, symbol_binary),) + tuple(
            filter(lambda x: StΣ_neb((state, symbol_binary), x), curr_history_LRU)
        )

        write_history = (write_binary, new_history)

        return Transition(write_history, move, l2i(new_state))

    return tm


if __name__ == "__main__":

    argparser = argparse.ArgumentParser(
        description="n-gram Closed Position Set (NGramCPS) as implemented in Coq-BB5 with history and LRU augmentations"
    )
    argparser.add_argument(
        "-m",
        "--tm",
        type=str,
        help="The transition function of the Turing machine in the bbchallenge format, e.g. 1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA",
        required=True,
    )
    argparser.add_argument(
        "-r",
        "--radius",
        type=int,
        help="Size of the ngrams on both sides of the head, e.g. 2",
        required=True,
    )

    argparser.add_argument(
        "-g",
        "--gas",
        type=int,
        help="Gas parameter, higher gas better chances of success",
        default=100,
    )

    argparser.add_argument(
        "--history",
        type=int,
        help="Length-n (state,symbol) history NGramCPS augmentation, 0 (default) means no history is used and standard NGramCPS is ran",
        default=0,
    )

    argparser.add_argument(
        "--LRU",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Least Recent Usage NGramCPS augmentation, if set, the LRU variant of NGramCPS is used (ignoring parameter --history which is used for length-n history variant)",
    )

    argparser.add_argument(
        "--verbose",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Prints debug information",
    )

    argparser.add_argument(
        "--print-cert",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Prints the reached ngrams and local contexts",
    )

    argparser.add_argument(
        "--print-final-result",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Prints whether the machine does not halt or may halt",
    )

    args = argparser.parse_args()

    if args.LRU:
        if args.verbose:
            print(f"Running NGramCPS, LRU variant on `{args.tm}`")
        tm_history_LRU = TM_history_LRU("1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA")
        result = NGramCPS_decider(
            tm_history_LRU,
            Σ_history0,
            args.radius,
            args.radius,
            args.gas,
            args.verbose,
            args.print_cert,
        )

    elif args.history > 0:
        if args.verbose:
            print(
                f"Running NGramCPS, length-{args.history} history variant on `{args.tm}`"
            )
        tm_history = TM_history(args.tm, args.history)
        result = NGramCPS_decider(
            tm_history,
            Σ_history0,
            args.radius,
            args.radius,
            args.gas,
            args.verbose,
            args.print_cert,
        )

    else:
        if args.verbose:
            print(f"Running standard NGramCPS on `{args.tm}`")

        tm_binary = TM_binary(args.tm)
        result = NGramCPS_decider(
            tm_binary,
            Σ0,
            args.radius,
            args.radius,
            args.gas,
            args.verbose,
            args.print_cert,
        )

    if result:
        if args.print_final_result:
            print(f"Success: {args.tm} does not halt")
        exit(0)

    if args.print_final_result:
        print(f"Failure: {args.tm} may halt")

    exit(-1)
