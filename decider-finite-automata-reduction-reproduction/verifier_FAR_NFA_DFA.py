import argparse

from parser_FAR_dvf import *

from typing import Callable


def verify_FAR_halting_transition(proof: FAR_EntryDFANFA, from_state, read_symbol):
    # The condition is that for all u q_0 T_u T_f T_r >= s with (f,r) a halting transition
    # In our case, this amount to making sure that NFA states 0f, 1f, etc..., (l-1)f
    # (with l the number of DFA states), when they read r, reach at least all states of s
    # Here s is \bot, hence we just have to check that reading 0 from those states reaches \bot in all cases
    return proof.nfa_transitions[read_symbol][:-1,][from_state::5, :][:, -1].all()


def verify_FAR_left_transition(
    proof: FAR_EntryDFANFA, from_state, read_symbol, write, goto
):
    # The condition is:
    # For all b \in {0,1}
    # R_r >= row_{\delta(i,b)}(M_f)^T row_i(M_t) R_b R_w
    # Which amounts to checking R_b R_w at position "\delta(i,b)f", "it"

    for i_dfa_state in range(proof.nb_dfa_states):
        for b in range(2):
            goes_to_dfa_state = proof.dfa_transitions[i_dfa_state][b]
            RbRw = proof.nfa_transitions[b] @ proof.nfa_transitions[write]

            i = 5 * goes_to_dfa_state + from_state
            j = 5 * i_dfa_state + goto

            if not (proof.nfa_transitions[read_symbol][i, :] >= RbRw[j, :]).all():
                # print(i_dfa_state, b, write)
                # print(i,j, proof.nfa_transitions[read_symbol][i,j], RbRw[i,j])
                # print(RbRw.astype(int))
                return False

    return True


def verify_FAR_right_transition(proof, from_state, read_symbol, write, goto):
    # The condition is:
    # R_r >= row_i(M_f)^T row_{\delta(i,w)}(M_t)
    # Which amounts to checking R_r at position "if", "\delta(i,w)t"

    done = set({})
    for i_dfa_state in range(proof.nb_dfa_states):
        goes_to_dfa_state = proof.dfa_transitions[i_dfa_state][write]

        if goes_to_dfa_state in done:
            continue
        done.add(goes_to_dfa_state)

        nfa_state_i_f = 5 * i_dfa_state + from_state
        nfa_state_delta_i_w_t = 5 * goes_to_dfa_state + goto

        if not proof.nfa_transitions[read_symbol][nfa_state_i_f, nfa_state_delta_i_w_t]:
            return False

    return True


def verify_FAR_proof_DFA_NFA(
    header: FAR_EntryHeader,
    proof: FAR_EntryDFANFA,
    get_machine_i: Callable[[int], bytes],
):
    # Condition 1 (Leading zeros ignored)
    # The DFA's transition function \delta should verify
    # \delta(0,0) = 0
    if proof.dfa_transitions[0][0] != 0:
        return False, 1

    # Condition 2 (Trailing zeros ignored)
    # The NFA's should verify T_0a^T = a^T
    if not (
        proof.nfa_transitions[0] @ proof.accept_vector.T == proof.accept_vector.T
    ).all():
        return False, 2

    # Condition 3 (Steady state is accepting)
    # Here, the steady state is ⊥ (last index)
    if not proof.accept_vector[0, -1]:
        return False, 3

    # Condition 4 (Steady state is steady)
    # s T_0 >= s
    # s T_1 >= s
    # Here, the steady state is ⊥ (last index)
    # Hence we can simply check sT_0 and sT_1 at position -1,-1
    # And just compute this position thanks to the last columns of T_0 and T_1
    if not (
        (
            proof.steady_state
            @ proof.nfa_transitions[0][:, -1].reshape(entry.nb_nfa_states, 1)
        )[0, 0]
        and (
            proof.steady_state
            @ proof.nfa_transitions[0][:, -1].reshape(entry.nb_nfa_states, 1)
        )[0, 0]
    ):
        return False, 4

    # Condition 8 (Initial configuration rejected)
    if proof.accept_vector[0, 0]:
        return False, 8

    # There is one condition to check per machine's transition rule
    M = get_machine_i(header.machine_id)
    for from_state in range(5):
        for read_symbol in range(2):
            write, move_to, goto = M[
                6 * from_state + 3 * read_symbol : 6 * from_state + 3 * read_symbol + 3
            ]

            # Symmetrising machine if scan is right to left
            if entry.direction_right_to_left:
                move_to = 1 - move_to

            goto -= 1

            # Condition 5: halting transition
            if goto == -1:
                if not verify_FAR_halting_transition(proof, from_state, read_symbol):
                    return False, 4

            # Condition 6: left-going transition
            elif move_to == 1:
                if not verify_FAR_left_transition(
                    proof, from_state, read_symbol, write, goto
                ):
                    return False, 4

            # Condition 7: right-going transition
            elif move_to == 0:
                if not verify_FAR_right_transition(
                    proof, from_state, read_symbol, write, goto
                ):
                    return False, 4

    return True, 0


def pptm(machine, return_repr=False):
    def ithl(i):
        return chr(ord("A") + i)

    def g(move):
        if move == 0:
            return "R"
        return "L"

    from tabulate import tabulate

    headers = ["s", "0", "1"]
    table = []

    nb_states = len(machine) // 6

    for i in range(nb_states):
        row = [ithl(i)]
        for j in range(2):
            write = machine[6 * i + 3 * j]
            move = machine[6 * i + 3 * j + 1]
            goto = machine[6 * i + 3 * j + 2] - 1

            if goto == -1:
                row.append("---")
                continue

            row.append(f"{write}{g(move)}{ithl(goto)}")
        table.append(row)

    if not return_repr:
        print(tabulate(table, headers=headers))
    else:
        return tabulate(table, headers=headers)


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
        "-e",
        "--entry",
        type=int,
        help="verifies only the specified entry of the dvf file",
    )
    argparser.add_argument(
        "--graphviz",
        help="if an entry is selected with -e this will output the graphviz code of the NFA (the entry is also verified)",
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
    SELECTED_ENTRY = args.entry
    VERBOSE = args.verbose

    def load_machine_from_db(machine_db_file, i, db_has_header=True):
        c = 1 if db_has_header else 0
        machine_db_file.seek(30 * (i + c))
        return machine_db_file.read(30)

    # Verify just one machine
    if SELECTED_ENTRY is not None:
        dvf = FAR_DVF.from_file(PATH_TO_DVF, pre_scan=False)

        header, entry = dvf.ith_entry(args.entry)
        machine_db_file = open(PATH_TO_DB, "rb")

        if VERBOSE:
            print(f"Verifying machine #{header.machine_id}\n")
            machine = load_machine_from_db(machine_db_file, header.machine_id)
            pptm(machine)
            print(f"\nDVF header:\n\n{header}")
            print(f"\nDVF entry:\n\n{entry}")

        verified, error_id = verify_FAR_proof_DFA_NFA(
            header,
            entry,
            lambda machine_id: load_machine_from_db(machine_db_file, machine_id),
        )
        machine_db_file.close()

        if args.graphviz:
            print(entry.to_graphviz())

        if verified:
            if VERBOSE:
                print("\nMachine successfully verified.")
            exit(0)
        if VERBOSE:
            print(f"\nMachine not verified, failing condition {error_id}.")
        exit(-1)

    if args.graphviz and SELECTED_ENTRY is None:
        print(
            "You must select a specific entry using -e option for a graphviz representation to be outputted.\n"
        )
        argparser.print_help()
        exit(-1)
