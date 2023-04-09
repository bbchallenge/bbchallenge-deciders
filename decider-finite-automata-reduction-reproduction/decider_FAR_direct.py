import argparse
import multiprocessing as mp

from joblib import Parallel, delayed

from parser_FAR_dvf import *
from solver_FAR_NFA_direct import solve_FAR_NFA_from_DFA
from tm_utils import *


def decider_FAR_direct(machine: bytes, limit_dfa_states: int) -> bool:
    """Implements the FAR direct CTL method for deciding if a Turing machine halts.

    Args:
        machine (bytes): Turing machine in bbchallenge's binary format
        limit_dfa_states (int): number of dfa states to search up to

    Returns:
        bool: True if the machine is found to halt, False if we cannot conclude
    """

    def unflatten_DFA(flat_DFA):
        unflatten_DFA: list[list[int]] = []
        for i, s in enumerate(flat_DFA):
            if i % 2 == 0:
                unflatten_DFA.append([])
            unflatten_DFA[-1].append(s)
        return unflatten_DFA

    def check_DFA(
        partial_flatten_DFA, total_dfa_states, direction_right_to_left
    ) -> bool:
        solved, R, a = solve_FAR_NFA_from_DFA(
            unflatten_DFA(partial_flatten_DFA),
            total_dfa_states,
            machine,
            direction_right_to_left,
        )
        return solved, R, a

    def search_DFA_with_n_states(total_dfa_states, direction_right_to_left) -> bool:
        solved, R, a = check_DFA([0], total_dfa_states, direction_right_to_left)

        k = 1
        t = np.zeros(2 * total_dfa_states).astype(int)
        m = np.zeros(2 * total_dfa_states).astype(int)

        while True:
            if solved and k < 2 * total_dfa_states:
                q_new = m[k - 1] + 1
                t[k] = q_new if (q_new < total_dfa_states and 2 * q_new - 1 == k) else 0
            elif not solved:
                while True:
                    if k <= 1:
                        return False, None, None, None
                    k -= 1
                    if t[k] <= m[k - 1] and t[k] < total_dfa_states - 1:
                        break
                t[k] += 1
            else:
                return True, unflatten_DFA(t), R, a
            m[k] = max(m[k - 1], t[k])
            k += 1
            solved, R, a = check_DFA(t[:k], total_dfa_states, direction_right_to_left)

    for total_dfa_states in range(1, limit_dfa_states + 1):
        for direction_right_to_left in [False, True]:
            solved, dfa, R, a = search_DFA_with_n_states(
                total_dfa_states, direction_right_to_left
            )
            if solved:
                return solved, direction_right_to_left, dfa, R, a

    return False, None, None, None, None


def aux_check_decider_FAR_direct(
    proof: FAR_EntryDFANFA, machine: bytes, limit_dfa_states: int
) -> bool:
    solved, direction_right_to_left, dfa, R, a = decider_FAR_direct(
        machine,
        limit_dfa_states,
    )

    return (
        solved
        and direction_right_to_left == proof.direction_right_to_left
        and (str(dfa) == str(proof.dfa_transitions))
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
        "-c",
        "--cores",
        type=int,
        default=mp.cpu_count(),
        help=f"number of cores on which to parallelize the run, default on your machine is {mp.cpu_count()}",
    )

    argparser.add_argument(
        "-cdvf",
        "--check-dvf",
        help="Decides all machine IDs present in a .dvf (where the number of dfa states are less than the limit given with option -l) and check that results are the same",
    )

    argparser.add_argument(
        "-l",
        "--limit-dfa-states",
        help="max number of dfa states to enumerate",
        type=int,
        default=7,
    )

    argparser.add_argument(
        "--verbose",
        help="enables logging",
        default=False,
        action="store_true",
    )

    args = argparser.parse_args()

    PATH_TO_DB = args.db
    PATH_TO_DVF = args.check_dvf
    VERBOSE = args.verbose
    LIMIT_DFA_STATES = args.limit_dfa_states

    CHECK_DVf = args.check_dvf is not None

    if CHECK_DVf:
        with open(PATH_TO_DB, "rb") as machine_db_file:
            with open(PATH_TO_DVF, "rb") as dvf_file:
                import tqdm

                dvf = FAR_DVF.from_file(PATH_TO_DVF, pre_scan=True)

                relevant_entries = []
                if VERBOSE:
                    print(
                        f"Filtering relevant dvf entries (<= {LIMIT_DFA_STATES} DFA states)"
                    )
                N = dvf.n_entries
                for i in tqdm.tqdm(range(N)):
                    header, entry = dvf.ith_entry(dvf_file, i)
                    if entry.nb_dfa_states <= LIMIT_DFA_STATES:
                        relevant_entries.append([i, header.machine_id])

                if VERBOSE:
                    print(
                        f"Deciding {len(relevant_entries)}/{dvf.n_entries} dvf entries that have <= {LIMIT_DFA_STATES} DFA states and checking match with dvf file..."
                    )

                gen_entries = (
                    [
                        dvf.ith_entry(dvf_file, i)[1],
                        load_machine_from_db(machine_db_file, machine_id),
                        LIMIT_DFA_STATES,
                    ]
                    for i, machine_id in relevant_entries
                )

                results = Parallel(n_jobs=args.cores, prefer="processes", verbose=0)(
                    delayed(aux_check_decider_FAR_direct)(entry, machine, limit)
                    for entry, machine, limit in tqdm.tqdm(
                        gen_entries, total=len(relevant_entries)
                    )
                )

                results = np.array(map(lambda x: x[0], results))

                if results.all():
                    if VERBOSE:
                        print(
                            f"All {len(relevant_entries)}/{dvf.n_entries} dvf entries that have <= {LIMIT_DFA_STATES} DFA states were successfully decided and match the dvf entries!"
                        )
                    exit(0)

                if VERBOSE:
                    argwhere = np.argwhere(results == False).flatten()
                    print(f"{len(argwhere)} DVF entries were not decided correctly!")
                    print(f"Here are the 10 first such entries: {argwhere[:10]}")

                exit(-1)
