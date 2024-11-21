from typing import TypeVar, Generic, Callable, Literal


def i2l(i: int) -> str:
    return chr(i + ord("A"))


def l2i(l: str) -> int:
    return ord(l) - ord("A")


St = int
Σ = TypeVar("Σ")


class Transition(Generic[Σ]):
    def __init__(self, write: Σ, move: Literal["L", "R"], new_state: St):
        self.write: Σ = write
        self.move: Literal["L", "R"] = move
        self.new_state: St = new_state

    def __iter__(self):
        return iter((self.write, self.move, self.new_state))

    def __str__(self):
        return f"{self.write}{self.move}{i2l(self.new_state)}"


TM = Callable[[St, Σ], Transition | None]
NGRAM = tuple[Σ]


class LocalContext(Generic[Σ]):
    def __init__(self, left: NGRAM[Σ], right: NGRAM[Σ], center: Σ, state: St):
        self.left: NGRAM[Σ] = left
        self.right: NGRAM[Σ] = right
        self.center: Σ = center
        self.state: int = state

    def __str__(self):
        return f"{self.left} [{i2l(self.state)} {self.center}] {self.right}"

    def __hash__(self) -> int:
        return hash((self.left, self.right, self.center, self.state))

    def __eq__(self, other) -> int:
        return (
            self.left == other.left
            and self.right == other.right
            and self.center == other.center
            and self.state == other.state
        )


# set of ngrams {'000','001'} is represented as
# {NGRAM['00']: {'0', '1'}}
ngram_set = dict[NGRAM[Σ], set[Σ]]


class AbstractExecState(Generic[Σ]):
    def __init__(
        self,
        left_ngrams: ngram_set,
        right_ngrams: ngram_set,
        local_contexts: set[LocalContext[Σ]],
    ):
        self.left_ngrams: ngram_set = left_ngrams
        self.right_ngrams: ngram_set = right_ngrams
        self.local_contexts: set[LocalContext[Σ]] = local_contexts

    def ins_left_ngram(self, ngram: NGRAM[Σ]):
        suffix = ngram[1:]
        if suffix in self.left_ngrams:
            self.left_ngrams[suffix].add(ngram[0])
        else:
            self.left_ngrams[suffix] = {ngram[0]}

    def ins_right_ngram(self, ngram: NGRAM[Σ]):
        prefix = ngram[:-1]
        if prefix in self.right_ngrams:
            self.right_ngrams[prefix].add(ngram[-1])
        else:
            self.right_ngrams[prefix] = {ngram[-1]}

    @classmethod
    def initial(cls, Σ0: Σ, len_l: int, len_r) -> "AbstractExecState":
        return cls[Σ](
            {NGRAM([Σ0] * (len_l - 1)): {Σ0}},
            {NGRAM([Σ0] * (len_r - 1)): {Σ0}},
            {LocalContext(NGRAM([Σ0] * len_l), NGRAM([Σ0] * len_r), Σ0, 0)},
        )


def expand_local_context(
    tm: Callable[[Σ, St], Transition | None],
    lc: LocalContext[Σ],
    S: AbstractExecState[Σ],
) -> tuple[list[LocalContext[Σ]], bool]:
    state = lc.state
    symbol = lc.center

    transition = tm(state, symbol)

    if transition is None:
        return [], True

    write, move, new_state = transition

    to_return: list[LocalContext[Σ]] = []

    if move == "R":
        falling_left_ngram = lc.left
        S.ins_left_ngram(falling_left_ngram)
        possible_new_right_symbols: set[Σ] = S.right_ngrams.get(lc.right[1:], set())
        for s in possible_new_right_symbols:
            new_lc = LocalContext(
                lc.left[1:] + (write,),
                lc.right[1:] + (s,),
                lc.right[0],
                new_state,
            )

            if new_lc not in S.local_contexts:
                to_return.append(new_lc)
                S.local_contexts.add(new_lc)

    elif move == "L":
        falling_right_ngram = lc.right
        S.ins_right_ngram(falling_right_ngram)
        possible_new_left_symbols: set[Σ] = S.left_ngrams.get(lc.left[:-1], set())
        for s in possible_new_left_symbols:
            new_lc = LocalContext(
                (s,) + lc.left[:-1],
                (write,) + lc.right[:-1],
                lc.left[-1],
                new_state,
            )
            if new_lc not in S.local_contexts:
                to_return.append(new_lc)
                S.local_contexts.add(new_lc)

    return to_return, False


def NGramCPS_decider(
    tm: Callable[[Σ, St], Σ],
    Σ0: Σ,
    len_l: int,
    len_r: int,
    gas: int,
    verbose: bool = False,
) -> bool:
    S: AbstractExecState[Σ] = AbstractExecState.initial(Σ0, len_l, len_r)

    while gas > 0:
        to_visit = list[LocalContext[Σ]](S.local_contexts)
        any_updates = False
        while len(to_visit) > 0 and gas > 0:
            curr_local_context = to_visit[0]

            if verbose:
                print("Visiting", curr_local_context)

            to_visit = to_visit[1:]
            new_contexts_to_visit, halting_met = expand_local_context(
                tm, curr_local_context, S
            )

            if halting_met:
                return False

            if len(new_contexts_to_visit) > 0:
                any_updates = True

            to_visit = new_contexts_to_visit + to_visit
            gas -= 1
        if not any_updates:
            return True

    return False


Σ_binary = Literal["0", "1"]
Σ0 = "0"


def TM_binary(tm_bbchallenge: str) -> TM[Σ_binary]:
    """
    Example:
    >>> tm = tm_binary("1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA")
    >>> str(tm(0, "0"))
    '1RB'
    >>> str(tm(0, "1"))
    'None'
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


def TM_history(tm_bbchallenge: str, history_length: int) -> TM[Σ_binary]:
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


def TM_history_LRU(tm_bbchallenge: str) -> TM[Σ_binary]:
    """LRU stands for Least Recent Usage"""

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
        new_history = ((state, symbol_binary),) + curr_history_LRU
        new_history = tuple(
            filter(lambda x: StΣ_neb((state, symbol_binary), x), curr_history_LRU)
        )
        write_history = (write_binary, new_history)

        return Transition(write_history, move, l2i(new_state))

    return tm


if __name__ == "__main__":
    tm_binary = TM_binary("1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA")
    print(NGramCPS_decider(tm_binary, Σ0, 2, 2, 100))

    tm_history = TM_history("1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA", 4)
    print(NGramCPS_decider(tm_history, Σ_history0, 2, 2, 200))

    tm_history_LRU = TM_history_LRU("1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA")
    print(NGramCPS_decider(tm_history_LRU, Σ_history0, 2, 2, 100))