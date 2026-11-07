#!/usr/bin/env python3
"""Bounded offline pilot. Standard library only; never calls a model endpoint."""
import argparse
import csv
import hashlib
import io
import json
import pathlib
import platform
import random
import statistics
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
CONTEXTS = ['human_model', 'model_tool', 'model_model', 'model_data']
ARMS = ['conventional', 'json', 'cbor', 'records']
TIMES = ['codec_encode_validate_total_ns', 'codec_decode_validate_total_ns',
         'adapter_render_encode_validate_total_ns', 'adapter_parse_decode_validate_total_ns',
         'request_render_total_ns']
REPETITIONS = 5
SEED = 794


def digest(data):
    return hashlib.sha256(data).hexdigest()


def dumps(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'))


def cases():
    return [c['id'] for c in json.loads((ROOT / 'fixtures/golden/suite.json').read_text())['cases']]


def schedule(ids, repetitions=REPETITIONS, seed=SEED):
    if not 1 <= repetitions <= 10:
        raise ValueError('repetitions must be 1..10')
    rng = random.Random(seed)  # Python MT19937; exact order is also persisted in raw rows.
    matrix = [(case, ctx, arm) for case in ids for ctx in CONTEXTS for arm in ARMS]
    result = []
    for rep in range(repetitions):
        shuffled = matrix.copy()
        rng.shuffle(shuffled)
        for case, ctx, arm in shuffled:
            result.append({'trial_id': f'pilot/{rep}/{case}/{ctx}/{arm}',
                           'repetition': rep, 'sequence': len(result),
                           'case_id': case, 'context': ctx, 'arm': arm})
    return result


def exports(rows):
    text = ''.join(dumps(r) + '\n' for r in rows)
    out = io.StringIO(newline='')
    writer = csv.DictWriter(out, fieldnames=sorted(rows[0]), lineterminator='\n')
    writer.writeheader()
    for r in rows:
        writer.writerow({k: v if isinstance(v, str) else dumps(v) for k, v in r.items()})
    return text, out.getvalue()


def restore_csv(text, rows):
    table = list(csv.DictReader(io.StringIO(text, newline='')))
    assert len(table) == len(rows)
    for r, c in zip(rows, table):
        assert r.keys() == c.keys()
        assert {k: v if isinstance(r[k], str) else json.loads(v) for k, v in c.items()} == r


def summary(rows):
    groups = []
    for arm in ARMS:
        subset = [r for r in rows if r['arm'] == arm]
        group = {'arm': arm, 'attempted': len(subset),
                 'succeeded': sum(r['error'] is None for r in subset), 'timings_ns_per_operation': {}}
        for name in TIMES:
            values = sorted(r[name] / r['iterations'] for r in subset
                            if r['error'] is None and r[name] is not None)
            group['timings_ns_per_operation'][name] = None if not values else {
                'n': len(values), 'min': values[0], 'median': statistics.median(values),
                'p95_nearest_rank': values[-(-95 * len(values) // 100) - 1], 'max': values[-1]}
        groups.append(group)
    return groups


def run(outdir):
    outdir.mkdir(parents=True, exist_ok=False)
    binary = ROOT / 'target/release/communications-probe'
    rows, constants = [], {}
    for plan in schedule(cases()):
        row = dict(plan, error=None, returncode=None, iterations=0, warmup_iterations=0,
                   **{key: None for key in TIMES})
        args = [str(binary), 'golden', 'trial', plan['case_id'], plan['context'], plan['arm'], plan['trial_id']]
        try:
            proc = subprocess.run(args, capture_output=True, timeout=60, check=False)
            row['returncode'] = proc.returncode
            if proc.returncode:
                raise ValueError(f'exit {proc.returncode}; stdout={proc.stdout.decode(errors="replace")}; stderr={proc.stderr.decode(errors="replace")}')
            measured = json.loads(proc.stdout)
            assert measured['trial_id'] == row['trial_id']
            for key in TIMES + ['iterations', 'warmup_iterations']:
                row[key] = measured.pop(key)
            measured.pop('trial_id')
            key = '/'.join((plan['case_id'], plan['context'], plan['arm']))
            if key in constants:
                assert constants[key] == measured, 'non-timing fields changed across repetition'
            else:
                constants[key] = measured
        except (subprocess.TimeoutExpired, ValueError, AssertionError) as exc:
            row['error'] = str(exc)
        rows.append(row)  # Failed attempts stay in the denominator and the raw exports.
    j, c = exports(rows)
    (outdir / 'trials.jsonl').write_text(j)
    (outdir / 'trials.csv').write_text(c)
    (outdir / 'matrix.json').write_text(dumps(constants) + '\n')
    (outdir / 'summary.json').write_text(json.dumps(summary(rows), indent=2) + '\n')
    inputs = sorted([ROOT / 'Cargo.lock', *ROOT.glob('src/**/*.rs'), *ROOT.glob('fixtures/golden/*'),
                     *ROOT.glob('scripts/*pilot*.py'), ROOT / 'docs/CONTRACT-SCHEMA.txt',
                     ROOT / 'docs/RECORD-SCHEMA.txt'])
    manifest = {'mode': 'offline_only', 'repetitions': REPETITIONS, 'seed': SEED,
                'process_boundary': 'one fresh release child per trial; serial; parent Python not timed',
                'order': 'Python random.Random(seed) MT19937 shuffle of all 96 combinations once per repetition; actual sequence in trials',
                'warmup_iterations_per_operation': 3, 'timed_iterations_per_operation': 20,
                'timing_scope': 'per-child warmed batch totals; same fixed named-operation order inside child; startup/setup/hashes/I/O excluded; spans overlap',
                'sampling_unit': 'one warmed 20-operation batch in one fresh child for one case/context/arm; five per combination',
                'summary_scope': 'descriptive pooled batch means by arm; heterogeneous cases/contexts, not iid operation samples; no confidence intervals',
                'endpoint': {'configured': False, 'calls': 0, 'provider': None, 'budget': None,
                             'reason': 'runtime status reported no secret bindings, runtime variables or outbound identities; repository has offline clients only; no authorized free endpoint verified'},
                'model_native': 'unsupported/not testable',
                'environment': {'platform': platform.platform(), 'machine': platform.machine(),
                                'python': sys.version, 'rustc': subprocess.check_output(['rustc', '-Vv']).decode(),
                                'cargo': subprocess.check_output(['cargo', '-V']).decode().strip()},
                'base_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
                'commands': ['cargo build --release --locked', 'python3 scripts/offline-pilot.py run docs/results/pilot',
                             'target/release/communications-probe golden trial CASE CONTEXT ARM TRIAL_ID',
                             'python3 scripts/offline-pilot.py audit docs/results/pilot'],
                'inputs_sha256': {str(p.relative_to(ROOT)): digest(p.read_bytes()) for p in inputs if p.is_file()},
                'binary_sha256': digest(binary.read_bytes()),
                'outputs_sha256': {p.name: digest(p.read_bytes()) for p in sorted(outdir.iterdir())}}
    (outdir / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    audit(outdir)


def audit(outdir):
    manifest = json.loads((outdir / 'manifest.json').read_text())
    for name, expected in manifest['inputs_sha256'].items():
        assert digest((ROOT / name).read_bytes()) == expected, name
    for name, expected in manifest['outputs_sha256'].items():
        assert digest((outdir / name).read_bytes()) == expected, name
    rows = [json.loads(line) for line in (outdir / 'trials.jsonl').read_text().splitlines()]
    plans = schedule(cases(), manifest['repetitions'], manifest['seed'])
    assert len(rows) == len(plans) == 480
    assert len({r['trial_id'] for r in rows}) == 480
    restore_csv((outdir / 'trials.csv').read_text(), rows)
    matrix = json.loads((outdir / 'matrix.json').read_text())
    checked, common = set(), {}
    for row, plan in zip(rows, plans):
        assert {k: row[k] for k in plan} == plan
        if row['error'] is not None:
            assert isinstance(row['error'], str) and row['error']
            continue
        assert row['returncode'] == 0 and row['iterations'] == 20 and row['warmup_iterations'] == 3
        for name in TIMES:
            n = row[name]
            assert (n is None) == (row['arm'] == 'conventional' and name.startswith('codec_'))
            assert n is None or (type(n) is int and n >= 0)
        key = '/'.join((row['case_id'], row['context'], row['arm']))
        if key in checked:
            continue
        checked.add(key)
        fixed = matrix[key]
        assert (fixed['case_id'], fixed['context'], fixed['arm']) == (row['case_id'], row['context'], row['arm'])
        assert fixed['mode'] == 'offline_local'
        assert fixed['source_digest_verified'] and fixed['ledger_verified'] and fixed['structural_roundtrip']
        assert all(v is None for v in fixed['metrics'].values())
        assert fixed['score'] is None and fixed['response_sha256'] is None
        assert fixed['correct_assertions_per_1000_actual_input_tokens'] is None
        req = subprocess.check_output([str(ROOT / 'target/release/communications-probe'), 'golden', 'request',
                                       row['case_id'], row['context'], row['arm']])
        assert digest(req) == fixed['request_sha256'] and len(req) == fixed['final_request_bytes']
        request = json.loads(req)
        envelope = request.pop('context_envelope')
        assert len(dumps(envelope).encode()) == fixed['adapter_envelope_bytes']
        assert digest(request['response_schema'].encode()) == fixed['response_schema_sha256']
        assert len(request['response_schema'].encode()) == fixed['response_schema_text_bytes']
        assert digest(request['instructions'].encode()) == fixed['instructions_sha256']
        prior = common.setdefault(row['case_id'], request)
        assert prior == request  # identical questions/instructions/schema across all matched arms/contexts
        if row['arm'] == 'conventional':
            if row['context'] == 'human_model':
                payload = envelope['messages'][0]['content'].encode()
            elif row['context'] == 'model_model':
                payload = dumps(json.loads(envelope['messages'][0]['content'])).encode()
            elif row['context'] == 'model_tool':
                payload = dumps(envelope['params']['arguments']).encode()
            else:
                payload = dumps(envelope['documents'][0]['content']).encode()
            assert len(payload) == fixed['payload_bytes']
            assert fixed['schema_text_bytes'] == 0
        else:
            import base64
            if row['context'] in ['human_model', 'model_model']:
                packet = json.loads(envelope['messages'][0]['content'])
            elif row['context'] == 'model_tool':
                packet = envelope['params']['arguments']
            else:
                packet = envelope['documents'][0]['content']
            payload = packet['payload'].encode() if packet['transfer'] == 'utf8' else base64.b64decode(packet['payload'], validate=True)
            assert len(payload) == fixed['payload_bytes']
            assert len(packet['schema'].encode()) == fixed['schema_text_bytes']
            if packet['transfer'] == 'base64':
                assert len(packet['payload']) == 4 * ((len(payload) + 2) // 3)
        for field, path in [('source_sha256', ROOT / f"fixtures/golden/{row['case_id']}.txt"),
                            ('contract_sha256', ROOT / f"fixtures/golden/{row['case_id']}.contract.json"),
                            ('suite_sha256', ROOT / 'fixtures/golden/suite.json'),
                            ('cargo_lock_sha256', ROOT / 'Cargo.lock')]:
            assert digest(path.read_bytes()) == fixed[field]
    assert set(matrix) == checked
    assert json.loads((outdir / 'summary.json').read_text()) == summary(rows)
    failures = sum(r['error'] is not None for r in rows)
    print(f'pilot audit: {len(rows)} attempts, {len(rows)-failures} successes, {failures} failures retained; '
          f'{len(matrix)} matched combinations; order/IDs/CSV/hashes/byte arithmetic/timing and null boundaries verified')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['run', 'audit'])
    parser.add_argument('directory', type=pathlib.Path)
    args = parser.parse_args()
    (run if args.action == 'run' else audit)(args.directory)
