from copy import deepcopy
from typing import Literal, TypeVar, Generic, Callable

Sigma = str
Σ0 = "0"
St = Literal["St0", "St1", "St2", "St3", "St4"]


T = TypeVar("T")


def sort_set_str(set: set) -> str:
    return str(sorted(set)).replace("[", "{").replace("]", "}")


class NGRAM(object):
    def __init__(self, l: list[Sigma]):
        self.l = l

    def __str__(self):
        return f"'{"".join(self.l)}'"

    def __repr__(self):
        return self.__str__()

    def __hash__(self) -> int:
        return hash(str(self))

    def __eq__(self, other) -> bool:
        return str(self) == str(other)

    def __lt__(self, other) -> bool:
        return str(self) < str(other)

    def __getitem__(self, key):
        return self.l[key]

    def __len__(self):
        return len(self.l)

    #     (* Compute pop_back 0 [ 1 ; 2 ; 3 ; 4 ].
    #        = [0; 1; 2; 3]
    #        Compute pop_back 0 nil.
    #        = nil
    #     *)
    #   Fixpoint pop_back{T}(x:T)(ls:list T):(list T) :=
    #   match ls with
    #   | h::t =>
    #     x::(pop_back h t)
    #   | nil => nil
    #   end.

    def pop_back(self, x: Sigma) -> "NGRAM":
        """
        Example:
            >>> ngram = NGRAM(['0','0','1'])
            >>> ngram.pop_back('1')
            '100'
        """
        return NGRAM([x] + self.l[:-1])

    def pop_back_prime(self, h: Sigma) -> tuple["NGRAM", Sigma]:
        """
        >>> ngram = NGRAM(['0','0','1'])
        >>> ngram.pop_back_prime('1')
        ('100', '1')
        >>> ngram.pop_back_prime('0')
        ('000', '1')
        >>> ngram
        '001'
        """
        if len(self.l) == 0:
            return NGRAM([]), h
        return NGRAM([h] + self.l[:-1]), self.l[-1]


class SetOfEncoding(Generic[T]):
    """
    https://github.com/ccz181078/Coq-BB5/blob/384b4eff476867c196eb00d3e6ecedd6d85da811/CoqBB5/Prelims.v#L42

    Example:

    >>> s = SetOfEncoding[Sigma]()
    >>> s.set_ins('0')
    ((['0'], {'0'}), False)
    >>> s.set_ins('0')[0].set_ins('1')
    ((['1', '0'], {'0', '1'}), False)
    >>> s.set_ins('0')[0].set_ins('0')
    ((['0'], {'0'}), True)
    >>> s
    ([], {})

    >>> s = SetOfEncoding[MidWord]()
    >>> s.set_ins(MidWord(NGRAM([]), NGRAM([]), '0', 'St0'))
    ((['' [St0 0] '' ], {'' {St0 0} '' }), False)
    >>> s.set_ins(MidWord(NGRAM(["0","1"]), NGRAM(["1"]), '0', 'St1'))
    ((['01' [St1 0] '1' ], {'01' {St1 0} '1' }), False)
    >>> s.set_ins(MidWord(NGRAM(["0","1"]), NGRAM(["1"]), '0', 'St1'))[0].set_ins(MidWord(NGRAM(["0","1"]), NGRAM(["1"]), '0', 'St1'))
    ((['01' [St1 0] '1' ], {'01' {St1 0} '1' }), True)
    >>> s.set_ins(MidWord(NGRAM(["0","1"]), NGRAM(["1"]), '0', 'St1'))[0].set_ins(MidWord(NGRAM(["0","1"]), NGRAM(["11"]), '0', 'St2'))
    ((['01' [St2 0] '11' , '01' [St1 0] '1' ], {'01' {St1 0} '1' , '01' {St2 0} '11' }), False)

    """

    def __init__(self, fst: list[T] = [], snd: set[T] = set()):
        self.fst = fst
        self.snd = snd

    def set_ins(self, x: T) -> tuple["SetOfEncoding[T]", bool]:
        if x in self.snd:
            return self, True
        fst_copy = deepcopy(self.fst)
        snd_copy = deepcopy(self.snd)
        fst_copy.insert(0, x)
        snd_copy.add(x)
        return SetOfEncoding[T](fst_copy, snd_copy), False

    def __str__(self):
        return f"({self.fst}, {sort_set_str(self.snd)})"

    def __repr__(self):
        return f"({self.fst}, {sort_set_str(self.snd)})"


class MidWord(object):
    def __init__(self, l: NGRAM, r: NGRAM, m: Sigma, s: St):
        self.l = l
        self.r = r
        self.m = m
        self.s = s

    def __str__(self):
        return f"{self.l} [{self.s} {self.m}] {self.r} "

    def __repr__(self):
        return self.__str__()

    def __eq__(self, other):
        return (
            self.l == other.l
            and self.r == other.r
            and self.m == other.m
            and self.s == other.s
        )

    def __lt__(self, other):
        return str(self) < str(other)

    def __hash__(self):
        return hash(str(self))


xset_impl = dict[NGRAM, SetOfEncoding[Sigma]]
mset_impl = SetOfEncoding[MidWord]


# here list[Sigma] is not an NGRAM
def xset_as_list(xs: xset_impl, x1: NGRAM) -> list[Sigma]:
    if x1 in xs:
        return xs[x1].fst
    return []


def xset_ins0(
    xs: xset_impl, v: SetOfEncoding[Sigma], x1: NGRAM, x2: Sigma
) -> tuple[xset_impl, bool]:
    new_v, flag = v.set_ins(x2)
    xs_copy = xs.copy()
    xs_copy[x1] = new_v
    return (xs_copy, flag)


def xset_ins(xs: xset_impl, x: NGRAM) -> tuple[xset_impl, bool]:
    """

    Example:

    >>> xs: xset_impl = {}
    >>> xset_ins(xs, NGRAM(['0','0','1']))
    ({'00': (['1'], {'1'})}, False)
    >>> xs = xset_ins(xs, NGRAM(['0','0','1']))[0]
    >>> xset_ins(xs, NGRAM(['0','0','1']))
    ({'00': (['1'], {'1'})}, True)
    >>> xset_ins(xs, NGRAM(['0','0','0']))
    ({'00': (['0', '1'], {'0', '1'})}, False)
    >>> xs = xset_ins(xs, NGRAM(['0','0','0']))[0]
    >>> xset_ins(xs, NGRAM(['1','0','1']))
    ({'00': (['0', '1'], {'0', '1'}), '10': (['1'], {'1'})}, False)
    """
    if len(x) == 0:
        return xs, False

    h = x[0]
    t = NGRAM(x[1:])

    # "001".pop_back_prime('1') -> ('100', '1')
    # "001".pop_back_prime('0') -> ('000', '1')
    x1, x2 = t.pop_back_prime(h)

    if x1 in xs:
        v = xs[x1]
        return xset_ins0(xs, v, x1, x2)
    else:
        return xset_ins0(xs, SetOfEncoding[Sigma](), x1, x2)


def mset_ins0(ms: mset_impl, mw: MidWord) -> tuple[mset_impl, bool]:
    return ms.set_ins(mw)


def mset_ins(
    q: list[MidWord],
    ms: mset_impl,
    flag: bool,
    f: Callable[[Sigma], MidWord],
    ls: list[Sigma],  # not an NGRAM but a list of symbols
) -> tuple[tuple[list[MidWord], mset_impl], bool]:
    """

    Example:
    >>> mw1 = MidWord(NGRAM(["0","0"]), NGRAM(["1"]), '0', 'St0')
    >>> mw2 = MidWord(NGRAM(["1"]), NGRAM(["1", "0"]), '1', 'St2')
    >>> ms: mset_impl = SetOfEncoding[MidWord]()
    >>> mset_ins([mw1,mw2], ms, False, lambda x: MidWord(x, NGRAM(["1"]), '0', 'St3'), NGRAM(['0','0','1']))
    (([1 [St3 0] '1' , 0 [St3 0] '1' , '00' [St0 0] '1' , '1' [St2 1] '10' ], ([1 [St3 0] '1' , 0 [St3 0] '1' ], {0 {St3 0} '1' , 1 {St3 0} '1' })), False)


    """
    if len(ls) == 0:
        return (q, ms), flag

    h = ls[0]
    t = ls[1:]

    new_ms, new_flag = mset_ins0(ms, f(h))
    q_prime = None
    if new_flag:
        q_prime = q
    else:
        q_prime = [f(h)] + q

    return mset_ins(q_prime, new_ms, flag and new_flag, f, t)


class AES_impl(object):
    def __init__(
        self,
        lset: xset_impl,
        rset: xset_impl,
        mset: mset_impl,
    ):
        self.lset = lset
        self.rset = rset
        self.mset = mset

    def __str__(self) -> str:
        return f"AES_impl({self.lset}, {self.rset}, {self.mset})"

    def __repr__(self) -> str:
        return self.__str__()

    @classmethod
    def init(cls, len_l: int, len_r: int) -> "AES_impl":

        lset = xset_ins({}, NGRAM([Σ0] * len_l))[0]
        rset = xset_ins({}, NGRAM([Σ0] * len_r))[0]

        mset = mset_ins0(
            SetOfEncoding[MidWord](),
            MidWord(NGRAM([Σ0] * len_l), NGRAM([Σ0] * len_r), Σ0, "St0"),
        )[0]

        return cls(lset, rset, mset)


def check_InitES_InAES(len_l: int, len_r: int, S: AES_impl) -> bool:
    ls, rs, ms = S.lset, S.rset, S.mset

    return (
        mset_ins0(ms, MidWord(NGRAM([Σ0] * len_l), NGRAM([Σ0] * len_r), Σ0, "St0"))[1]
        and xset_ins(ls, NGRAM([Σ0] * len_l))[1]
        and xset_ins(rs, NGRAM([Σ0] * len_r))[1]
    )


def translate_state(s: str) -> St:
    i = ord(s) - ord("A")
    return f"St{i}"


def get_transition_0(tm_bbchallenge: str, i: int, j: int) -> str:
    return tm_bbchallenge[6 * i + 3 * j : 6 * i + 3 * j + 3]


def get_transition(tm_bbchallenge: str, s: St, m: Sigma) -> St:
    """

    Example:
    >>> TM = "1RB1LE_1LC0RD_0LA1LA_0LB0RD_1LB---"
    >>> get_transition(TM, 'St0', '0')
    '1RB'
    >>> get_transition(TM, 'St1', '1')
    '0RD'
    >>> get_transition(TM, 'St4', '1')
    '---'
    """
    tm_bbchallenge = tm_bbchallenge.replace("_", "")
    i = int(str(s)[-1])
    if m == "0":
        return get_transition_0(tm_bbchallenge, i, 0)
    return get_transition_0(tm_bbchallenge, i, 1)


def update_AES_MidWord(
    tm_bbchallenge: str, q: list[MidWord], mw: MidWord, SI: AES_impl
) -> tuple[tuple[list[MidWord], AES_impl], bool]:
    print("Update mw:", mw)
    l0 = mw.l
    r0 = mw.r
    m0 = mw.m
    s0 = mw.s

    ls = SI.lset
    rs = SI.rset
    ms = SI.mset

    if len(l0) == 0 or len(r0) == 0:
        return ((q, SI), False)

    hl = l0[0]
    l1 = NGRAM(l0[1:])

    hr = r0[0]
    r1 = NGRAM(r0[1:])

    o, d, s1 = get_transition(tm_bbchallenge, s0, m0)
    s1 = translate_state(s1)

    # Machine halts
    if s1 == "-":
        return ((q, SI), False)

    if d == "R":
        # e.g. leftgram "000" gets inserted for the first time "00" -> {"0"}, False
        print("\tDir R")
        new_ls, flag_1 = xset_ins(ls, l0)
        print(f"\tFlag insert left ngram {l0}:", flag_1)

        (new_q, new_ms), flag_2 = mset_ins(
            q,
            ms,
            True,
            lambda x: MidWord(
                NGRAM([o] + l1.pop_back(hl).l), NGRAM(r1.l + [x]), hr, s1
            ),
            xset_as_list(rs, r1),
        )
        print(f"\trs xset_as_list: {r1}", xset_as_list(rs, r1))
        print("\tFlag insert midword:", flag_2)
        print("\tnew_q - q:", set(new_q) - set(q))

        return ((new_q, AES_impl(new_ls, rs, new_ms)), flag_1 and flag_2)

    print("\tDir L")
    new_rs, flag_1 = xset_ins(rs, r0)
    print(f"\tFlag insert right ngram {r0}:", flag_1)

    (new_q, new_ms), flag_2 = mset_ins(
        q,
        ms,
        True,
        lambda x: MidWord(NGRAM(l1.l + [x]), NGRAM([o] + r1.pop_back(hr).l), hl, s1),
        xset_as_list(ls, l1),
    )
    print(f"\tls xset_as_list: {l1}", xset_as_list(ls, l1))
    print("\tFlag insert midword:", flag_2)
    print("\tnew_q - q:", set(new_q) - set(q))

    return ((new_q, AES_impl(ls, new_rs, new_ms)), flag_1 and flag_2)


def update_AES(
    tm_bbchallenge: str, ms: list[MidWord], SI: AES_impl, flag: bool, n: int
) -> tuple[AES_impl, bool, int]:

    print("len(ms)", len(ms), "ms:", ms, "n:", n, "param flag:", flag)

    if n == 0:
        return SI, False, 0

    if len(ms) == 0:
        return SI, flag, n

    mw = ms[0]
    ms0 = ms[1:]

    new_S, new_flag = update_AES_MidWord(tm_bbchallenge, ms0, mw, SI)
    print("Inner Flag:", new_flag, "\n")

    new_q, new_SI = new_S

    return update_AES(tm_bbchallenge, new_q, new_SI, flag and new_flag, n - 1)


def NGramCPS_decider_0(
    len_l: int, len_r: int, m: int, n: int, tm_bbchallenge: str, S: AES_impl
) -> bool:

    print("S:", S, "\n")
    if m == 0:
        return False

    new_S, flag, n0 = update_AES(tm_bbchallenge, S.mset.fst, S, True, n)

    print("\nNew S:", new_S)
    print("\nFlag:", flag, "n0:", n0, "\n")

    if flag:
        return check_InitES_InAES(len_l, len_r, new_S)

    return NGramCPS_decider_0(len_l, len_r, m - 1, n0, tm_bbchallenge, new_S)


def NGramCPS_decider(len_l: int, len_r: int, m: int, tm_bbchallenge: str) -> bool:
    if len_l == 0 or len_r == 0:
        return False

    return NGramCPS_decider_0(
        len_l, len_r, m, m, tm_bbchallenge, AES_impl.init(len_l, len_r)
    )


def NGramCPS_decider_impl2_0(
    len_l: int, len_r: int, m: int, tm_bbchallenge: str
) -> bool:
    return NGramCPS_decider(len_l, len_r, m, tm_bbchallenge)


res = NGramCPS_decider_impl2_0(
    2, 2, 53, "1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA"
)  # <- True

print(res)
