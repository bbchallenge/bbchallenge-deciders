import argparse

parser = argparse.ArgumentParser()
parser.add_argument(
    "-d",
    "--db",
    help="path to the DB file",
    default="../all_5_states_undecided_machines_with_global_header",
)
parser.add_argument(
    "--dvf",
    required=True,
    help="path to the verification file",
)
parser.add_argument(
    "-e",
    "--entry",
    type=int,
    help="verifies only the specified entry of the dvf file",
)
parser.add_argument(
    "--graphviz",
    help="if an entry is selected with -e this will output the graphviz code of the NFA",
)

args = parser.parse_args()

print(args)
