"""Focused #795 regressions. All test failures/telemetry here are invented controls."""
import importlib.util
import pathlib
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('expanded',pathlib.Path(__file__).with_name('expanded-pilot.py'))
x=importlib.util.module_from_spec(spec);spec.loader.exec_module(x)


class ExpandedTests(unittest.TestCase):
    def test_sixty_cases_and_scaling_exactly_once_per_cell_per_rep(self):
        ids=x.case_ids();self.assertEqual(len(ids),60)
        self.assertEqual(len(x.audit_fixtures()),60)
        plans=x.pilot.schedule(ids,5,795)
        self.assertEqual(len(plans),4800)
        for rep in range(5):
            cells=[(p['case_id'],p['context'],p['arm']) for p in plans if p['repetition']==rep]
            self.assertEqual(len(set(cells)),960)
        self.assertEqual(plans,x.pilot.schedule(ids,5,795))
        self.assertNotEqual(plans,x.pilot.schedule(ids,5,794))

    def test_per_cell_stats_never_pool_heterogeneous_cases_or_count_inner_loops(self):
        rows=[]
        for case in ['one','two']:
            for n in [20,40,60,80,100]:
                rows.append(dict(case_id=case,context='model_data',arm='json',error=None,iterations=20,
                                 **{name:n for name in x.pilot.TIMES}))
        rows.append(dict(case_id='one',context='model_data',arm='json',error='test timeout',iterations=0,
                         **{name:None for name in x.pilot.TIMES}))
        summary=x.cell_summary(rows)
        self.assertEqual(len(summary),2)
        s=summary['one/model_data/json']
        self.assertEqual((s['attempted'],s['succeeded']),(6,5))
        self.assertEqual(s['timings_ns_per_operation'][x.pilot.TIMES[0]],dict(n=5,min=1,median=3,p95_nearest_rank=5,max=5))
        x.audit_stats(rows,summary)
        summary['one/model_data/json']['succeeded']=6
        with self.assertRaises(AssertionError):x.audit_stats(rows,summary)

    def test_empty_failed_cell_and_even_median_are_audited(self):
        rows=[dict(case_id='one',context='model_data',arm='json',error='test error',iterations=0,
                   **{name:None for name in x.pilot.TIMES})]
        s=x.cell_summary(rows);x.audit_stats(rows,s)
        self.assertTrue(all(v is None for v in s['one/model_data/json']['timings_ns_per_operation'].values()))
        for n in [20,60]:
            rows.append(dict(case_id='one',context='model_data',arm='json',error=None,iterations=20,
                             **{name:n for name in x.pilot.TIMES}))
        s=x.cell_summary(rows);x.audit_stats(rows,s)
        self.assertEqual(s['one/model_data/json']['timings_ns_per_operation'][x.pilot.TIMES[0]]['median'],2)

    def test_normalized_join_and_csv_retain_failures_null_and_exact_text(self):
        rows=x.pilot.schedule(['one'],1,795)
        for r in rows:r.update(error=None,metadata={'unicode':'e\u0301\u0000,\n"'},measurement=None)
        rows[0]['error']='test-only timeout'
        j,c=x.pilot.exports(rows);x.pilot.restore_csv(c,rows)
        self.assertEqual(len(j.splitlines()),16)
        with self.assertRaises(AssertionError):x.pilot.restore_csv(c.replace('one','wrong',1),rows)

    def test_fixture_reader_rejects_duplicate_reference_keys(self):
        with tempfile.TemporaryDirectory() as d:
            p=pathlib.Path(d)/'bad.json';p.write_text('{"id":1,"id":2}')
            with self.assertRaises(ValueError):x.read(p)


if __name__=='__main__':unittest.main()
