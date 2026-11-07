"""Regression tests for schedule, failure denominators, summaries and CSV export."""
import importlib.util
import json
import pathlib
import unittest

spec = importlib.util.spec_from_file_location('pilot', pathlib.Path(__file__).with_name('offline-pilot.py'))
pilot = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pilot)


class PilotTests(unittest.TestCase):
    def test_schedule_covers_each_cell_once_per_rep_and_is_reproducible(self):
        plans = pilot.schedule(['a', 'b', 'c'])
        self.assertEqual(plans, pilot.schedule(['a', 'b', 'c']))
        self.assertNotEqual(plans, pilot.schedule(['a', 'b', 'c'], seed=795))
        self.assertEqual(len({p['trial_id'] for p in plans}), 240)
        for rep in range(5):
            cells = [(p['case_id'], p['context'], p['arm']) for p in plans if p['repetition'] == rep]
            self.assertEqual(len(cells), 48)
            self.assertEqual(len(set(cells)), 48)
        self.assertEqual([p['sequence'] for p in plans], list(range(240)))
        for count in [0, 11]:
            with self.assertRaises(ValueError):
                pilot.schedule(['a'], repetitions=count)

    def test_export_preserves_nested_null_unicode_quotes_and_failed_attempt(self):
        rows = [{'trial_id': 'one', 'error': None, 'value': {'n': None}, 'text': 'café, "\n'},
                {'trial_id': 'two', 'error': 'failed\n"', 'value': None, 'text': ''}]
        j, c = pilot.exports(rows)
        self.assertEqual([json.loads(line) for line in j.splitlines()], rows)
        pilot.restore_csv(c, rows)
        with self.assertRaises(AssertionError):
            pilot.restore_csv(c.replace('two', 'changed'), rows)

    def test_summary_retains_failures_and_uses_batch_means_not_inner_samples(self):
        rows = []
        for i, n in enumerate([20, 40, 100]):
            rows.append({'arm': 'json', 'error': None, 'iterations': 20,
                         **{key: n for key in pilot.TIMES}})
        rows.append({'arm': 'json', 'error': 'test-only failure', 'iterations': 0,
                     **{key: None for key in pilot.TIMES}})
        group = pilot.summary(rows)[1]
        self.assertEqual((group['attempted'], group['succeeded']), (4, 3))
        for stats in group['timings_ns_per_operation'].values():
            self.assertEqual(stats, {'n': 3, 'min': 1, 'median': 2, 'p95_nearest_rank': 5, 'max': 5})
        empty = pilot.summary([])[0]
        self.assertEqual(empty['attempted'], 0)
        self.assertTrue(all(v is None for v in empty['timings_ns_per_operation'].values()))


if __name__ == '__main__':
    unittest.main()
