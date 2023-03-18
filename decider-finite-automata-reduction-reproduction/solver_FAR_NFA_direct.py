import argparse
import multiprocessing as mp
from tm_utils import load_machine_from_db

from parser_FAR_dvf import *

from joblib import Parallel, delayed


def solve_FAR_NFA_from_DFA(
    dfa_transitions, nb_nfa_states, machine: bytes, direction_right_to_left=False
):
    """Given a DFA and a Turing machine the function solves the NFA
    associated to it (direct FAR algorithm) and returns True if the finite machine
    gives a CTL non-halt proof for the Turing machine.

    Args:
        dfa_transitions: a DFA given as a list of transitions
        nb_nfa_states: the number of states in the NFA to solve
        machine (bytes): a Turing machine in bbchallenge binary format
        direction_right_to_left (bool, optional): Whether the scan direction is right to left or not. Defaults to False.

    Returns:
        (solved, R, a):
            solved (bool): True if the machine is proven non-halting by the constructed NFA
            R: transition matrices for the sub-NFA
            a: accepting states of the sub-NFA
    """
    nb_dfa_states = len(dfa_transitions)
    R = np.zeros((2, nb_nfa_states, nb_nfa_states)).astype(bool)

    # Equation 4'
    R[0, -1, -1] = True
    R[1, -1, -1] = True

    # Machine dependent static equations
    for from_state in range(5):
        for read_symbol in range(2):
            write, move_to, goto = machine[
                6 * from_state + 3 * read_symbol : 6 * from_state + 3 * read_symbol + 3
            ]
            goto -= 1

            # Symmetrising machine if scan is right to left
            if direction_right_to_left:
                move_to = 1 - move_to

            # Equation 5'
            if goto == -1:
                R[read_symbol][:-1,][from_state::5, :][:, -1] = True

            # Equation 7'
            elif move_to == 0:
                for i_dfa_state in range(nb_dfa_states):
                    goes_to_dfa_state = dfa_transitions[i_dfa_state][write]
                    nfa_state_i_f = 5 * i_dfa_state + from_state
                    nfa_state_delta_i_w_t = 5 * goes_to_dfa_state + goto
                    R[read_symbol][nfa_state_i_f, nfa_state_delta_i_w_t] = True

    # Machine dependent dynamic equations
    old_R = None
    while old_R is None or (old_R != R).any():
        old_R = np.copy(R)

        for from_state in range(5):
            for read_symbol in range(2):
                write, move_to, goto = machine[
                    6 * from_state
                    + 3 * read_symbol : 6 * from_state
                    + 3 * read_symbol
                    + 3
                ]
                goto -= 1

                if goto == -1:
                    continue

                # Symmetrising machine if scan is right to left
                if direction_right_to_left:
                    move_to = 1 - move_to

                # Equations 6' left move, iterative construction
                if move_to == 1:
                    for b in range(2):
                        RbRw = old_R[b] @ old_R[write]
                        for i_dfa_state in range(nb_dfa_states):
                            goes_to_dfa_state = dfa_transitions[i_dfa_state][b]

                            i = 5 * goes_to_dfa_state + from_state
                            j = 5 * i_dfa_state + goto

                            R[read_symbol][i, :] += RbRw[j, :]

    # Solve a
    a = np.zeros((1, nb_nfa_states)).astype(bool)
    a[0, -1] = True

    old_a = None
    while old_a is None or (a != old_a).any():
        old_a = np.copy(a)
        a = (R[0] @ old_a.T).T

    e_0 = np.zeros((1, nb_nfa_states)).astype(bool)
    e_0[0, 0] = True

    solved = not (e_0 @ a.T)[0, 0]

    return solved, R, a


def aux_check_solve_FAR_NFA_from_DFA(proof: FAR_EntryDFANFA, machine: bytes):
    solved, R, a = solve_FAR_NFA_from_DFA(
        proof.dfa_transitions,
        5 * len(proof.dfa_transitions) + 1,
        machine,
        proof.direction_right_to_left,
    )

    return (
        solved
        and (R[0] == proof.nfa_transitions[0]).all()
        and (R[1] == proof.nfa_transitions[1]).all()
        and (a == proof.accept_vector).all()
    )


if __name__ == "__main__":
    argparser = argparse.ArgumentParser()
    argparser.add_argument(
        "-d",
        "--db",
        help="path to the DB file",
        default="../all_5_states_undecided_machines_with_global_header",
    )
    argparser.add_argument(
        "--dvf",
        required=True,
        help="path to the verification file",
    )

    argparser.add_argument(
        "-c",
        "--cores",
        type=int,
        default=mp.cpu_count(),
        help=f"number of cores on which to parallelize the run, default on your machine is {mp.cpu_count()}",
    )

    argparser.add_argument(
        "-cdvf",
        "--check-dvf",
        help="solves the nfas of the dfas of a dfa-nfa dvf files and make sure they correspond to the nfas in the file",
        default=False,
        action="store_true",
    )

    argparser.add_argument(
        "--verbose",
        help="enables logging",
        default=False,
        action="store_true",
    )

    args = argparser.parse_args()

    PATH_TO_DB = args.db
    PATH_TO_DVF = args.dvf
    VERBOSE = args.verbose

    CHECK_DVf = args.check_dvf

    if CHECK_DVf:
        with open(PATH_TO_DB, "rb") as machine_db_file:
            with open(PATH_TO_DVF, "rb") as dvf_file:
                import tqdm

                dvf = FAR_DVF.from_file(PATH_TO_DVF, pre_scan=True)

                N = dvf.n_entries
                # N = 10000

                if VERBOSE:
                    print(
                        f"Solving {N} dvf entries and checking match with dvf file..."
                    )

                gen_entries = (
                    [
                        dvf.ith_entry(dvf_file, i)[1],
                        load_machine_from_db(
                            machine_db_file, dvf.ith_entry(dvf_file, i)[0].machine_id
                        ),
                    ]
                    for i in range(N)
                )

                results = Parallel(n_jobs=args.cores, prefer="processes", verbose=0)(
                    delayed(aux_check_solve_FAR_NFA_from_DFA)(entry, machine)
                    for entry, machine in tqdm.tqdm(gen_entries, total=N)
                )

                results = np.array(map(lambda x: x[0], results))

                if results.all():
                    if VERBOSE:
                        print(
                            f"All {N} entries were successfully solved and match the dvf entries!"
                        )
                    exit(0)

                if VERBOSE:
                    argwhere = np.argwhere(results == False).flatten()
                    print(
                        f"{len(argwhere)} DVF entries were solved to different NFA than in the dvf file!"
                    )
                    print(f"Here are the 10 first such entries: {argwhere[:10]}")

                exit(-1)
