import argparse
import os
import sys
from typing import Optional

from tm_tape import TMTape, TMHasHalted
from tm_regex_tape import TMRegexTape, BlockSimulationTimeout, FacingBlock


class NoException(Exception):
    pass


class MaxVisitedRegex(Exception):
    pass


def display_graph(
    dot_content: str, output_path: Optional[str] = None, verbose: bool = False
) -> bool:
    """
    Displays the DOT graph using graphviz/pydot.
    Returns True if successful, False otherwise.

    Args:
        dot_content: The DOT graph content as a string
        output_path: Optional path to save the rendered image
        verbose: Whether to print status messages
    """
    try:
        # Try to use graphviz first
        import graphviz  # type: ignore

        # Create a Source object from the DOT content
        src = graphviz.Source(dot_content)

        # Render and view the graph
        if output_path:
            # Remove .dot extension if present
            if output_path.endswith(".dot"):
                img_path = output_path[:-4]
            else:
                img_path = output_path
            src.render(img_path, view=True, cleanup=True)
        else:
            src.view(cleanup=True)

        return True
    except ImportError:
        try:
            # Fall back to pydot if graphviz is not available
            import pydot  # type: ignore

            # Parse the DOT content
            graphs = pydot.graph_from_dot_data(dot_content)
            if not graphs:
                if verbose:
                    print("Failed to parse DOT content")
                return False

            graph = graphs[0]

            # Determine output path
            if output_path:
                if output_path.endswith(".dot"):
                    png_path = output_path[:-4] + ".png"
                else:
                    png_path = output_path + ".png"
            else:
                png_path = "tm_graph.png"

            # Write the graph to a PNG file
            graph.write_png(png_path)
            if verbose:
                print(f"Graph saved to {png_path}")

            # Try to open the PNG file
            try:
                if sys.platform == "darwin":  # macOS
                    import subprocess

                    subprocess.run(["open", png_path], check=True)
                elif sys.platform == "win32":  # Windows
                    os.startfile(png_path)
                else:  # Linux and other Unix-like
                    import subprocess

                    subprocess.run(["xdg-open", png_path], check=True)
            except Exception as e:
                if verbose:
                    print(f"Generated PNG file but couldn't open it automatically: {e}")

            return True
        except ImportError:
            if verbose:
                print("Neither graphviz nor pydot Python packages are available.")
                print(
                    "Please install one of them with: pip install graphviz or pip install pydot"
                )
            return False
        except Exception as e:
            if verbose:
                print(f"Error displaying graph with pydot: {e}")
            return False


def deciderRep_WL(
    TM: str,
    block_size: int,
    plus_threshold: int,
    max_visited_regex: int,
    block_simulation_timeout: int,
    print_cert: bool,
    verbose: bool,
    build_graph: bool = False,
    graph_output_path: str = "tm_graph.dot",
    display_graph_when_done: bool = False,
    save_graph: bool = False,
) -> tuple[bool, Exception, int, bool]:
    if print_cert:
        print(TM)

    TM_tape = TMTape(TM, "", 0, "")

    visited_regex_tapes: set[str] = set()
    regex_tapes_to_visit: list[TMRegexTape] = [
        TMRegexTape.from_tm_tape(TM_tape, block_size, plus_threshold)
    ]

    # Initialize graph
    graph: list[str] = []
    node_ids: dict[str, int] = {}
    edge_count = 0

    # Flag to track if there is at least one regex branching
    has_regex_branching = False

    if build_graph:
        graph = ["digraph TM_Execution {"]
        # Use a more space-efficient layout
        graph.append(
            "    layout=dot;"
        )  # dot layout is hierarchical and works well for state machines
        graph.append("    overlap=false;")  # prevent node overlap
        graph.append("    splines=true;")  # use curved edges
        graph.append(
            "    concentrate=true;"
        )  # merge edges going to the same destination
        graph.append("    nodesep=0.4;")  # space between nodes
        graph.append("    ranksep=0.6;")  # space between ranks
        graph.append("    ratio=fill;")  # fill the available space
        graph.append('    size="10,10";')  # default size in inches
        graph.append(
            "    node [shape=box, style=filled, fillcolor=lightblue, fontsize=10, width=0, height=0, margin=0.1];"
        )
        graph.append("    edge [fontsize=9];")

        # Add initial node
        initial_tape = str(regex_tapes_to_visit[0])
        node_ids[initial_tape] = 0
        # Truncate long labels to make the graph more compact
        truncated_label = truncate_label(initial_tape, 30)
        graph.append(f'    node{0} [label="{truncated_label}", fillcolor=lightgreen];')

    while len(regex_tapes_to_visit) > 0:
        curr_regex_tape = regex_tapes_to_visit.pop()
        curr_tape_str = str(curr_regex_tape)

        if curr_tape_str in visited_regex_tapes:
            continue

        visited_regex_tapes.add(curr_tape_str)

        # Add node to graph if not already added
        if build_graph and curr_tape_str not in node_ids:
            node_id = len(node_ids)
            node_ids[curr_tape_str] = node_id
            # Truncate long labels to make the graph more compact
            truncated_label = truncate_label(curr_tape_str, 30)
            graph.append(f'    node{node_id} [label="{truncated_label}"];')

        if len(visited_regex_tapes) > max_visited_regex:
            if build_graph:
                # Finalize and save graph before returning
                graph.append("}")
                dot_content = "\n".join(graph)

                if save_graph:
                    with open(graph_output_path, "w") as f:
                        f.write(dot_content)
                    if verbose:
                        print(f"DOT graph saved to {graph_output_path}")

                if display_graph_when_done:
                    display_graph(
                        dot_content, graph_output_path if save_graph else None, verbose
                    )
            return False, MaxVisitedRegex(), len(node_ids), has_regex_branching

        if print_cert:
            print(curr_regex_tape)

        try:
            curr_regex_tape.macro_step(block_simulation_timeout, verbose)
            next_tape_str = str(curr_regex_tape)

            # Add node and edge to graph
            if build_graph:
                if next_tape_str not in node_ids:
                    node_id = len(node_ids)
                    node_ids[next_tape_str] = node_id
                    # Truncate long labels to make the graph more compact
                    truncated_label = truncate_label(next_tape_str, 30)
                    graph.append(f'    node{node_id} [label="{truncated_label}"];')

                # Add edge for macro_step
                src_id = node_ids[curr_tape_str]
                dst_id = node_ids[next_tape_str]
                graph.append(
                    f'    node{src_id} -> node{dst_id} [label="macro_step", color=darkgreen, penwidth=1.5];'
                )
                edge_count += 1

            regex_tapes_to_visit.append(curr_regex_tape)
        except TMHasHalted as e:
            if build_graph:
                # Add halting node
                halt_id = len(node_ids)
                graph.append(
                    f'    node{halt_id} [label="HALTED", fillcolor=red, shape=doubleoctagon];'
                )

                # Add edge to halting node
                src_id = node_ids[curr_tape_str]
                graph.append(
                    f'    node{src_id} -> node{halt_id} [label="halted", color=red, penwidth=2.0];'
                )

                # Put halting node at the bottom rank
                graph.append(f"    {{ rank=sink; node{halt_id}; }}")

                # Finalize and save graph before returning
                graph.append("}")
                dot_content = "\n".join(graph)

                if save_graph:
                    with open(graph_output_path, "w") as f:
                        f.write(dot_content)
                    if verbose:
                        print(f"DOT graph saved to {graph_output_path}")

                if display_graph_when_done:
                    display_graph(
                        dot_content, graph_output_path if save_graph else None, verbose
                    )
            return False, e, len(node_ids), has_regex_branching
        except BlockSimulationTimeout as e:
            if build_graph:
                # Add timeout node
                timeout_id = len(node_ids)
                graph.append(
                    f'    node{timeout_id} [label="TIMEOUT", fillcolor=orange, shape=doubleoctagon];'
                )

                # Add edge to timeout node
                src_id = node_ids[curr_tape_str]
                graph.append(
                    f'    node{src_id} -> node{timeout_id} [label="timeout", color=orange, penwidth=2.0];'
                )

                # Put timeout node at the bottom rank
                graph.append(f"    {{ rank=sink; node{timeout_id}; }}")

                # Finalize and save graph before returning
                graph.append("}")
                dot_content = "\n".join(graph)

                if save_graph:
                    with open(graph_output_path, "w") as f:
                        f.write(dot_content)
                    if verbose:
                        print(f"DOT graph saved to {graph_output_path}")

                if display_graph_when_done:
                    display_graph(
                        dot_content, graph_output_path if save_graph else None, verbose
                    )
            return False, e, len(node_ids), has_regex_branching
        except FacingBlock:
            # Set the flag to indicate that there is at least one regex branching
            has_regex_branching = True

            plus_branches = curr_regex_tape.get_plus_branches(verbose)

            if build_graph:
                # Add edges for each branch
                src_id = node_ids[curr_tape_str]
                for i, branch in enumerate(plus_branches):
                    branch_str = str(branch)
                    if branch_str not in node_ids:
                        node_id = len(node_ids)
                        node_ids[branch_str] = node_id
                        # Truncate long labels to make the graph more compact
                        truncated_label = truncate_label(branch_str, 30)
                        graph.append(f'    node{node_id} [label="{truncated_label}"];')

                    dst_id = node_ids[branch_str]
                    graph.append(
                        f'    node{src_id} -> node{dst_id} [label="branch {i+1}", color=blue, style=dashed];'
                    )
                    edge_count += 1

            regex_tapes_to_visit += plus_branches

    if build_graph:
        # Add success node
        success_id = len(node_ids)
        graph.append(
            f'    node{success_id} [label="SUCCESS", fillcolor=green, shape=doubleoctagon];'
        )

        # Add invisible edges to improve layout
        if len(node_ids) > 2:
            # Add invisible edge from initial node to success node to encourage a top-to-bottom flow
            graph.append(f"    node0 -> node{success_id} [style=invis];")

            # Create a subgraph for special nodes to keep them at the same rank
            # This is a simplified approach that just puts the success node at the bottom
            graph.append(f"    {{ rank=sink; node{success_id}; }}")

            # Put the initial node at the top
            graph.append(f"    {{ rank=source; node0; }}")

        # Finalize and save graph
        graph.append("}")
        dot_content = "\n".join(graph)

        if save_graph:
            with open(graph_output_path, "w") as f:
                f.write(dot_content)

            if verbose:
                print(
                    f"DOT graph saved to {graph_output_path} with {len(node_ids)} nodes and {edge_count} edges"
                )
        elif verbose:
            print(f"Graph built with {len(node_ids)} nodes and {edge_count} edges")

        if display_graph_when_done:
            if display_graph(
                dot_content, graph_output_path if save_graph else None, verbose
            ):
                if verbose:
                    print("Graph displayed successfully")
            elif verbose:
                print("Failed to display graph")

    return True, NoException(), len(node_ids), has_regex_branching


def failure_reason_str(
    reason_failure: Exception, block_simulation_timeout: int, max_visited_regex: int
) -> str:
    if isinstance(reason_failure, TMHasHalted):
        return "halting configuration reached"
    if isinstance(reason_failure, BlockSimulationTimeout):
        return f"block simulation timeout `{block_simulation_timeout}` reached"
    if isinstance(reason_failure, MaxVisitedRegex):
        return f"limit of `{max_visited_regex}` visited regex tapes reached"
    return "Unknown"


def truncate_label(label: str, max_length: int = 30) -> str:
    """Truncate a label to a maximum length, adding ellipsis if needed."""
    return label


if __name__ == "__main__":

    argparser = argparse.ArgumentParser(
        description="Repeated Word List decider (RepWL)"
    )
    argparser.add_argument(
        "-m",
        "--tm",
        type=str,
        help="The transition function of the Turing machine in the bbchallenge format, e.g. 0RB---_1LC1RC_1LD0RA_1RE0LD_1RA1RE",
    )
    argparser.add_argument(
        "-b", "--block-size", type=int, help="The block size to use for the decider"
    )

    argparser.add_argument(
        "-r",
        "--plus-repeat-threshold",
        type=int,
        help="The threshold for the plus operator",
    )

    argparser.add_argument(
        "-f",
        "--file-machines-list",
        type=str,
        help="The file containing the list of Turing machines with parameters",
    )

    argparser.add_argument(
        "-t",
        "--block-simulation-timeout",
        type=int,
        help="The block simulation timeout",
        default=1000,
    )

    argparser.add_argument(
        "-M",
        "--max-visited-regex",
        type=int,
        help="The maximum number of visited regex tapes",
        default=150000,
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
        default=True,
        help="Prints the RepWL non-halt certificate(s)",
    )

    argparser.add_argument(
        "--print-params-stats",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="In case of a file with Turing machines and parameters, print statistics about the parameters (min, max, avg)",
    )

    argparser.add_argument(
        "--build-graph",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Build a DOT graph of the execution",
    )

    argparser.add_argument(
        "--graph-output",
        type=str,
        default="tm_graph.dot",
        help="Path to save the DOT graph (default: tm_graph.dot)",
    )

    argparser.add_argument(
        "--display-graph",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Display the graph when the decider finishes",
    )

    argparser.add_argument(
        "--save-graph",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Save the graph to a file when the decider finishes",
    )

    args = argparser.parse_args()

    FILE_MACHINES_LIST = None
    if args.file_machines_list is not None:
        FILE_MACHINES_LIST = args.file_machines_list
    else:
        if (
            args.tm is None
            or args.block_size is None
            or args.plus_repeat_threshold is None
        ):
            print(
                "Error: either provide a file with Turing machines and parameters or a single Turing machine with paramters"
            )
            exit(-1)
        TM = args.tm
        BLOCK_SIZE = args.block_size
        PLUS_THRESHOLD = args.plus_repeat_threshold

    BLOCK_SIMULATION_TIMEOUT = args.block_simulation_timeout
    MAX_VISITED_REGEX = args.max_visited_regex
    PRINT_CERT = args.print_cert
    VERBOSE = args.verbose
    BUILD_GRAPH = args.build_graph
    GRAPH_OUTPUT = args.graph_output
    DISPLAY_GRAPH = args.display_graph
    SAVE_GRAPH = args.save_graph

    if FILE_MACHINES_LIST is None:
        success, reason_failure, node_count, has_regex_branching = deciderRep_WL(
            TM,
            BLOCK_SIZE,
            PLUS_THRESHOLD,
            MAX_VISITED_REGEX,
            BLOCK_SIMULATION_TIMEOUT,
            PRINT_CERT,
            VERBOSE,
            BUILD_GRAPH,
            GRAPH_OUTPUT,
            DISPLAY_GRAPH,
            SAVE_GRAPH,
        )

        if success:
            print(f"Decider successful: TM does not halt")
            if has_regex_branching:
                print(f"TM has at least one regex branching")

            if BUILD_GRAPH and VERBOSE:
                print(f"Graph has {node_count} nodes")
            exit(0)

        # If we get here, success is False and there was an exception
        if isinstance(reason_failure, TMHasHalted):
            print("Decider not successful (halting configuration reached)")
            if has_regex_branching:
                print(f"TM has at least one regex branching")
            if BUILD_GRAPH and VERBOSE:
                print(f"Graph has {node_count} nodes")
            exit(-1)
        if isinstance(reason_failure, BlockSimulationTimeout):
            print(
                f"Decider not successful ({failure_reason_str(reason_failure, BLOCK_SIMULATION_TIMEOUT, MAX_VISITED_REGEX)})"
            )
            if has_regex_branching:
                print(f"TM has at least one regex branching")
            if BUILD_GRAPH and VERBOSE:
                print(f"Graph has {node_count} nodes")
            exit(-1)
        if isinstance(reason_failure, MaxVisitedRegex):
            print(
                f"Decider not successful ({failure_reason_str(reason_failure, BLOCK_SIMULATION_TIMEOUT, MAX_VISITED_REGEX)})"
            )
            if has_regex_branching:
                print(f"TM has at least one regex branching")
            if BUILD_GRAPH and VERBOSE:
                print(f"Graph has {node_count} nodes")
            exit(-1)
    else:
        import tqdm  # type: ignore

        with open(FILE_MACHINES_LIST) as f:
            file_content = f.read()

        at_least_one_failure = False

        params_stats_B = []
        params_stats_R = []
        total_nodes = 0
        num_TMs = 0
        num_TMs_with_branching = 0

        for line in tqdm.tqdm(file_content.split("\n")):
            if line.strip() == "":
                continue
            TM, BLOCK_SIZE, PLUS_THRESHOLD = line.split(" ")

            params_stats_B.append(int(BLOCK_SIZE))
            params_stats_R.append(int(PLUS_THRESHOLD))

            success, reason_failure, node_count, has_regex_branching = deciderRep_WL(
                TM,
                int(BLOCK_SIZE),
                int(PLUS_THRESHOLD),
                MAX_VISITED_REGEX,
                BLOCK_SIMULATION_TIMEOUT,
                PRINT_CERT,
                VERBOSE,
                BUILD_GRAPH,
                (
                    f"{os.path.splitext(GRAPH_OUTPUT)[0]}_{TM}.dot"
                    if BUILD_GRAPH and SAVE_GRAPH
                    else GRAPH_OUTPUT
                ),
                DISPLAY_GRAPH,
                SAVE_GRAPH,
            )

            if node_count < 10:
                print(TM, node_count)

            num_TMs += 1
            total_nodes += node_count
            if has_regex_branching:
                num_TMs_with_branching += 1

            if not success:
                at_least_one_failure = True
                print(
                    f"Failed to decide `{TM}` with parameters `block_size={BLOCK_SIZE}` and `plus_repeat_threshold={PLUS_THRESHOLD}`. Reason: {failure_reason_str(reason_failure, BLOCK_SIMULATION_TIMEOUT, MAX_VISITED_REGEX)}."
                )
                if has_regex_branching:
                    print(f"TM has at least one regex branching")
                if BUILD_GRAPH and VERBOSE:
                    print(f"Graph has {node_count} nodes")

        if args.print_params_stats:
            print(
                f"Statistics:\n\t-block_size: min={min(params_stats_B)}, max={max(params_stats_B)}, avg={round(sum(params_stats_B)/len(params_stats_B),1)}"
            )
            print(
                f"\t-plus_repeat_threshold: min={min(params_stats_R)}, max={max(params_stats_R)}, avg={round(sum(params_stats_R)/len(params_stats_R),1)}"
            )
            if BUILD_GRAPH:
                print(
                    f"\t-total graph nodes: {total_nodes}, avg per TM: {round(total_nodes/num_TMs, 1)}"
                )
            print(
                f"\t-TMs with regex branching: {num_TMs_with_branching} out of {num_TMs} ({round(100*num_TMs_with_branching/num_TMs, 1)}%)"
            )

        if at_least_one_failure:
            exit(-1)
        print(f"All {num_TMs} TMs have been decided successfully")
        exit(0)
