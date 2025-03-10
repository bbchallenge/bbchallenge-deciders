"""

This code has been generated by ChatGPT given as prompt:

'''
    Can you translate this Coq code into typed python:

    Record ListES := {
    l: list Σ;
    r: list Σ;
    m: Σ;
    s: St;
    }.
    
    [Content of implementations in `Decider_Loop.v`]
'''

Then `please keep the recursive style`.

The code has been modified a bit, including because there was "bug" in the Coq code due to overshadowing (which resulted in a bug in the GPT-trsanslated code):
https://github.com/tcosmo/Coq-BB5/commit/866a9b228c92670d517d94106c84ab95265826e9

"""

import sys

sys.setrecursionlimit(10000)

from dataclasses import dataclass
from typing import List, Tuple, Optional, Union, Callable

# Definitions for Σ, St, and related equality functions (to be implemented based on your application)
Σ = int  # Example type for Σ; update as needed
Σ0: Σ = 0  # Example default value for Σ; update as needed
St = int  # Example type for St; update as needed
Dir = int  # Example type for Dir; represents head movement direction (-1, 0, 1, etc.)
nat = int
Z0 = 0
St0 = 0
Z = int


@dataclass
class Trans:
    """Turing machine transition.

    Attributes:
        nxt (St): The next state.
        dir (Dir): The direction of the head move.
        out (Σ): The symbol to be written.
    """

    nxt: St
    dir: Dir
    out: Σ


TM = Callable[[St, Σ], Optional[Trans]]


def St_eqb(s1: St, s2: St) -> bool:
    return s1 == s2


def Σ_eqb(m1: Σ, m2: Σ) -> bool:
    return m1 == m2


@dataclass
class ListES:
    l: List[Σ]
    r: List[Σ]
    m: Σ
    s: St


def ListES_step_prime(tr: Trans, x: ListES) -> ListES:
    """Executes a single step of the Turing machine on a ListES configuration.

    Args:
        tr (Trans): The transition to apply.
        x (ListES): The current ListES configuration.
        Σ0 (Σ): The default symbol for empty tape cells.

    Returns:
        ListES: The updated ListES configuration after applying the transition.
    """
    l0, r0, m0, s0 = x.l, x.r, x.m, x.s
    s1, d, o = tr.nxt, tr.dir, tr.out

    if d > 0:  # Dpos (Move right)
        if r0:
            m1, *r1 = r0
            return ListES(l=[o] + l0, r=r1, m=m1, s=s1)
        else:
            return ListES(l=[o] + l0, r=[], m=Σ0, s=s1)

    elif d < 0:  # Dneg (Move left)
        if l0:
            m1, *l1 = l0
            return ListES(l=l1, r=[o] + r0, m=m1, s=s1)
        else:
            return ListES(l=[], r=[o] + r0, m=Σ0, s=s1)

    # Handle no movement case (optional, if Dpos/Dneg are exhaustive)
    return ListES(l=l0, r=r0, m=o, s=s1)


def print_listES(x: ListES) -> str:
    l_str = "".join(map(str, x.l))
    r_str = "".join(map(str, x.r))
    m_str = str(x.m)
    s_str = chr(x.s + ord("A"))
    return f"{l_str} [{s_str}{m_str}] {r_str}"


# Enum-like class for HaltDecideResult
from enum import Enum


class HaltDecideResult(Enum):
    Result_Halt = "Result_Halt"
    Result_NonHalt = "Result_NonHalt"
    Result_Unknown = "Result_Unknown"


def Z_ltb(d1: int, d2: int) -> bool:
    return d1 < d2


def Z_eqb(d1: int, d2: int) -> bool:
    return d1 == d2


# Define verify_loop1
def verify_loop1(
    h0: Tuple[ListES, Z],
    h1: Tuple[ListES, Z],
    ls0: List[Tuple[ListES, Z]],
    ls1: List[Tuple[ListES, Z]],
    n: nat,
    dpos: Z,
) -> bool:
    es0, d0 = h0
    es1, d1 = h1

    if not (St_eqb(es0.s, es1.s) and Σ_eqb(es0.m, es1.m)):
        return False

    # print("Verify plausible Loop", n)
    # print(print_listES(es0), d0, dpos)
    # print(print_listES(es1), d1, dpos)

    val = False

    if n == 0:
        if dpos == Z0:
            val = Z_eqb(d1, d0)
        elif dpos > 0:
            if not es1.r:
                val = Z_ltb(d1, d0)
            else:
                val = False
        elif dpos < 0:
            if not es1.l:
                val = Z_ltb(d0, d1)
            else:
                val = False

    if val:
        return True

    if ls0 and ls1:
        h0_prime, *ls0_prime = ls0
        h1_prime, *ls1_prime = ls1
        return verify_loop1(
            h0_prime, h1_prime, ls0_prime, ls1_prime, max(0, n - 1), dpos
        )
    else:
        return False


# Define find_loop1
def find_loop1(
    h0: Tuple[ListES, Z],
    h1: Tuple[ListES, Z],
    h2: Tuple[ListES, Z],
    ls0: List[Tuple[ListES, Z]],
    ls1: List[Tuple[ListES, Z]],
    ls2: List[Tuple[ListES, Z]],
    n: nat,
) -> bool:
    es0, d0 = h0
    es1, d1 = h1
    es2, d2 = h2

    # print("Find Loop")
    # for hhh in [h0, h1, h2]:
    #     print(print_listES(hhh[0]), hhh[1], n)
    # print()

    if (
        St_eqb(es0.s, es1.s)
        and St_eqb(es0.s, es2.s)
        and Σ_eqb(es0.m, es1.m)
        and Σ_eqb(es0.m, es2.m)
        and verify_loop1(h0, h1, ls0, ls1, n + 1, d0 - d1)
    ):
        return True

    if ls2 and ls1:
        h3, h2_prime, *ls2_prime = ls2
        h1_prime, *ls1_prime = ls1
        return find_loop1(h0, h1_prime, h2_prime, ls0, ls1_prime, ls2_prime, n + 1)
    else:
        return False


# Define find_loop1_0
def find_loop1_0(
    h0: Tuple[ListES, Z], h1: Tuple[ListES, Z], ls: List[Tuple[ListES, Z]]
) -> bool:
    # print("Find Loop1 0")
    if ls:
        h2, *ls_prime = ls
        return find_loop1(h0, h1, h2, [h1] + ls, ls, ls_prime, 0)
    else:
        return False


# Define loop1_decider0
def loop1_decider0(
    tm: TM, n: nat, es: ListES, d: Z, ls: List[Tuple[ListES, Z]]
) -> HaltDecideResult:
    if n == 0:
        return HaltDecideResult.Result_Unknown
    else:
        tr = tm(es.s, es.m)
        if tr is None:
            return HaltDecideResult.Result_Halt
        else:
            es_prime = ListES_step_prime(tr, es)
            d_prime = d + tr.dir
            # print(print_listES(es_prime), d_prime)
            ls_prime = [(es, d)] + ls
            if n > 1:
                return loop1_decider0(tm, n - 1, es_prime, d_prime, ls_prime)
            else:
                if find_loop1_0((es_prime, d_prime), (es, d), ls):
                    return HaltDecideResult.Result_NonHalt
                else:
                    return loop1_decider0(tm, n - 1, es_prime, d_prime, ls_prime)


# Define loop1_decider
def loop1_decider(n: nat, tm: TM) -> HaltDecideResult:
    initial_es = ListES(l=[], r=[], m=Σ0, s=St0)
    return loop1_decider0(tm, n, initial_es, Z0, [])


if __name__ == "__main__":

    def TM_from_bbchallenge(tm_bbchallenge: str) -> TM:
        """E.g. 1RB1LE_1LC0RD_0LA1LA_0LB0RD_1LB---"""
        tm_bbchallenge = tm_bbchallenge.replace("_", "")

        def TM(s: St, m: Σ) -> Optional[Trans]:
            trans_str = tm_bbchallenge[6 * s + 3 * m : 6 * s + 3 * m + 3]
            if trans_str[-1] == "-":
                return None
            nxt = ord(trans_str[-1]) - ord("A")
            dir = 1 if trans_str[1] == "R" else -1
            out = int(trans_str[0])
            return Trans(nxt, dir, out)

        return TM

    k = 0
    with open("bb5_verified_enumeration.csv") as infile:
        for line in infile:
            machine, status, decider = line.split(",")
            if not "LOOP1" in decider:
                continue
            gas_param = int(decider.split("_")[-2])
            res = loop1_decider(
                gas_param,
                TM_from_bbchallenge(machine),
            )

            if (res == HaltDecideResult.Result_NonHalt and status != "nonhalt") or (
                res == HaltDecideResult.Result_Halt and status != "halt"
            ):
                print("error")
                print(machine, status, decider)
                print(res)

            if k % 1_000_000 == 0:
                print(k)
            k += 1

    # loops_130_512_halt = [
    #     # "0RB0LC_1LA1RB_1LB0LD_0RA1RE_0LE---",
    #     # "0RB0LC_1LA1RB_1LB0LD_0RA1RE_1LE---",
    # ]
    # loops_130_512_nonhalt = [
    #     # "1RB---_1RC---_1RD0RC_1RE1LC_1LE1RD",
    #     # "1RB---_1RC---_1RD0LE_1RE1LC_1LE1RD",
    #     "0RB0LC_1LA1RB_1LB1RB_------_------",
    # ]

    # # BB5_CHAMPION = "1RB1LC_1RC1RB_1RD0LE_1LA1LD_---0LA"
    # # tm = TM_from_bbchallenge(BB5_CHAMPION)
    # # es = ListES([], [], 0, 0)
    # # k = 0
    # # while True:
    # #     aux = tm(es.s, es.m)
    # #     if aux is None:
    # #         break

    # #     es = ListES_step_prime(aux, es)
    # #     # print(print_listES(es))
    # #     if k % 1_000_000 == 0:
    # #         print(k)
    # #     k += 1
    # # print("Halt", k)

    # for machine in loops_130_512_halt:
    #     print(machine)
    #     res = loop1_decider(
    #         130,
    #         TM_from_bbchallenge(machine),
    #     )
    #     assert res == HaltDecideResult.Result_Halt

    # for machine in loops_130_512_nonhalt:
    #     print(machine)
    #     res = loop1_decider(
    #         130,
    #         TM_from_bbchallenge(machine),
    #     )
    #     assert res == HaltDecideResult.Result_NonHalt
