import unittest
from main import decider_closed_states, DB_PATH
from bbchallenge_utils import get_machine_i


class TestDeciderClosedStates(unittest.TestCase):
    def test_examples(self):
        ids_to_test = [4, 9]

        for machine_id in ids_to_test:
            machine = get_machine_i(DB_PATH, machine_id)
            self.assertTrue(decider_closed_states(machine))

    def test_counterexamples(self):
        ids_to_test = [5, 207]

        for machine_id in ids_to_test:
            machine = get_machine_i(DB_PATH, machine_id)
            self.assertFalse(decider_closed_states(machine, debug=False))
