#!/usr/bin/env python3
"""#795 extension of the bounded offline pilot; no network or model client."""
import argparse
import base64
import importlib.util
import json
import math
import pathlib
import platform
import re
import subprocess
import sys

spec = importlib.util.spec_from_file_location('pilot', pathlib.Path(__file__).with_name('offline-pilot.py'))
pilot = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pilot)
ROOT = pilot.ROOT
BINARY = ROOT / 'target/release/communications-probe'
SUITE = ROOT / 'fixtures/golden/expanded-suite.json'
REPS, SEED = 5, 795


def read(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f'duplicate key: {key}')
            result[key] = value
        return result
    return json.loads(path.read_text(), object_pairs_hook=unique)


def fixture_path(id, suffix):
    directory = 'expanded/' if (ROOT / f'fixtures/golden/expanded/{id}.txt').exists() else ''
    return ROOT / f'fixtures/golden/{directory}{id}{suffix}'


def case_ids():
    ids = [c['id'] for c in read(SUITE)['cases']]
    if len(ids) != 60 or len(set(ids)) != 60:
        raise ValueError('expected 60 unique case ids')
    return ids


def cell_summary(rows):
    groups = {}
    for row in rows:
        key = '/'.join(row[k] for k in ['case_id', 'context', 'arm'])
        groups.setdefault(key, []).append(row)
    return {key: pilot.summary(group)[pilot.ARMS.index(group[0]['arm'])]
            for key, group in sorted(groups.items())}


def write(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + '\n')


def run(out):
    ids = case_ids()  # Freeze design before observation.
    if not BINARY.is_file():
        raise ValueError('build release binary before running')
    display_out = str(out.relative_to(ROOT)) if out.is_relative_to(ROOT) else str(out)
    plans = pilot.schedule(ids, REPS, SEED)
    out.mkdir(parents=True, exist_ok=False)
    write(out / 'plan.json', plans)
    rows, matrix = [], {}
    for plan in plans:
        row = dict(plan, error=None, returncode=None, iterations=0, warmup_iterations=0,
                   **{key: None for key in pilot.TIMES})
        try:
            proc = subprocess.run([str(BINARY), 'expanded', 'trial', plan['case_id'], plan['context'],
                                   plan['arm'], plan['trial_id']], capture_output=True, timeout=60, check=False)
            row['returncode'] = proc.returncode
            if proc.returncode:
                raise ValueError(f'exit {proc.returncode}: {proc.stderr.decode(errors="replace")}')
            measured = json.loads(proc.stdout)
            assert measured.pop('trial_id') == plan['trial_id']
            for key in pilot.TIMES + ['iterations', 'warmup_iterations']:
                row[key] = measured.pop(key)
            key = '/'.join(plan[k] for k in ['case_id', 'context', 'arm'])
            assert key not in matrix or matrix[key] == measured
            matrix[key] = measured
        except (subprocess.TimeoutExpired, ValueError, AssertionError) as exc:
            row['error'] = str(exc) or type(exc).__name__
        rows.append(row)
        if len(rows) % 960 == 0:
            print(f'expanded pilot: {len(rows)}/{len(plans)} attempts retained', flush=True)
    j, c = pilot.exports(rows)
    (out / 'trials.jsonl').write_text(j)
    (out / 'trials.csv').write_text(c)
    write(out / 'matrix.json', matrix)
    write(out / 'summary-by-cell.json', cell_summary(rows))
    write(out / 'summary-pooled-by-arm.json', pilot.summary(rows))
    capabilities = [json.loads(line) for line in subprocess.check_output(
        [str(BINARY), 'expanded', 'capabilities']).splitlines()]
    write(out / 'capabilities.json', capabilities)
    references = read(SUITE)['cases']
    write(out / 'case-manifest.json', {c['id']: {
        'source_sha256': pilot.digest(fixture_path(c['id'], '.txt').read_bytes()),
        'extraction_sha256': pilot.digest(fixture_path(c['id'], '.contract.json').read_bytes()),
        'reference_sha256': pilot.digest(pilot.dumps(c).encode())} for c in references})
    inputs = sorted(set([ROOT / 'Cargo.lock', ROOT / 'Cargo.toml', *ROOT.glob('src/**/*.rs'),
                         *ROOT.glob('fixtures/golden/**/*'), *ROOT.glob('scripts/*.py'),
                         *ROOT.glob('docs/*SCHEMA.txt')]))
    manifest = dict(mode='offline_only', repetitions=REPS, seed=SEED, cases=ids,
                    contexts=pilot.CONTEXTS, arms=pilot.ARMS, attempted=len(plans),
                    sampling_unit='one fresh child warmed batch per case/context/arm/repetition; 20 inner operations are not 20 samples',
                    design='serial; Python MT19937 seeded shuffle of all 960 cells per repetition; exact pre-run order retained in plan.json',
                    warmups=3, iterations=20, timeout_seconds=60,
                    statistics='5 batch observations per cell; descriptive min/median/P95/max only; pooled arm summaries mix heterogeneous cases/contexts; no iid/population confidence or inference claims',
                    process_boundary='fresh child per attempt; host/OS caches shared; fixed operation order within child; startup/setup/fixture checks/hashes/I/O excluded',
                    endpoint=dict(configured=False, calls=0, provider=None, model=None, budget=None,
                                  reason='current cloud metadata has no secrets/runtime variables/outbound identities; repository has offline clients only; no authorized documented endpoint established'),
                    model_native='unsupported: no authorized documented tensor/hidden-state interface',
                    missing_live_configuration='authorized provider endpoint + two exact compatible model revisions + request/response interface documentation + bounded budget/data-sharing permission + genuine usage/run evidence',
                    environment=dict(platform=platform.platform(), python=sys.version,
                                     rustc=subprocess.check_output(['rustc', '-Vv']).decode(),
                                     cargo=subprocess.check_output(['cargo', '-V']).decode().strip()),
                    base_commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
                    binary_sha256=pilot.digest(BINARY.read_bytes()),
                    commands=['cargo build --release --locked', f'python3 scripts/expanded-pilot.py run {display_out}',
                              'target/release/communications-probe expanded trial CASE CONTEXT ARM TRIAL_ID',
                              f'python3 scripts/expanded-pilot.py audit {display_out}'],
                    inputs_sha256={str(p.relative_to(ROOT)): pilot.digest(p.read_bytes()) for p in inputs if p.is_file()},
                    outputs_sha256={p.name: pilot.digest(p.read_bytes()) for p in sorted(out.iterdir())})
    write(out / 'manifest.json', manifest)
    audit(out)


def audit_fixtures():
    suite = read(SUITE)
    original = read(ROOT / 'fixtures/golden/suite.json')['cases']
    assert suite['version'] == 1 and suite['cases'][:6] == original
    ids = case_ids()
    coverage = read(ROOT / 'fixtures/golden/coverage.json')
    assert coverage['original_cases'] == ids[:6]
    assert [c['id'] for c in coverage['new_cases']] == ids[6:]
    assert len({c['distinction'] for c in coverage['new_cases']}) == 54
    for c in suite['cases']:
        source = fixture_path(c['id'], '.txt').read_bytes()
        assert pilot.digest(source) == c['source_sha256']
        contract = read(fixture_path(c['id'], '.contract.json'))
        assert contract['version'] == '0.1-draft'
        assert {k:v['kind'] for k,v in contract['entities'].items()} == c['entity_kinds']
        assert {k:[v['subject_kind'],v['object_kind']] for k,v in contract['relations'].items()} == c['relation_kinds']
        assert contract['evidence'] == {'e1': {'sha256':c['source_sha256'], 'locator':f"fixture:golden/{c['id']}.txt"}}
        fs = c['facts']
        assert len({f['id'] for f in fs}) == len(fs) == len(contract['assertions'])
        for f, a in zip(fs, contract['assertions']):
            assert a['id'] == f['id'] and a['subject'] == f['subject']
            assert a['predicate'] == f['relation'] and a['object'] == f['value']
            assert a['provenance'] == dict(asserted_by=f['by'],method=f['method'],evidence=f['evidence'])
            assert a['times'] == dict(event_ms=f['event_ms'],record_ms=f['recorded_ms'],interpretation_ms=f['interpreted_ms'])
            assert a['uncertainty'] == (dict(kind='unknown') if f['confidence_permille'] is None else dict(kind='confidence',permille=f['confidence_permille']))
        if c['id'] in ids[6:]:
            # Independently parse the authored source notation only for evidence audit.
            # This is never a runtime source extractor or a tested wire renderer.
            lines = source.decode().splitlines()
            assert json.loads(lines[2].removeprefix('Local entities: ')[:-1]) == c['entity_kinds']
            assert json.loads(lines[3].removeprefix('Relation catalogue: ')[:-1]) == c['relation_kinds']
            assertions = lines[4:-1]
            assert len(assertions) == len(fs)
            for line, f in zip(assertions, fs):
                match = re.fullmatch(r'(f[0-9]+): ([^ ]+) \((observed|reported|inferred)\) states ([^ ]+) ([^ ]+) = (.*); event (null|-?[0-9]+), record (-?[0-9]+), interpretation (-?[0-9]+), confidence permille (null|[0-9]+); cites e1\.', line)
                assert match, line
                id, by, method, subject, relation, obj, event, record, interpretation, confidence = match.groups()
                assert dict(id=id,by=by,method=method,subject=subject,relation=relation,value=json.loads(obj),
                            event_ms=json.loads(event),recorded_ms=int(record),interpreted_ms=int(interpretation),
                            confidence_permille=json.loads(confidence),evidence=['e1']) == f
        qs = c['questions']
        assert {q['task'] for q in qs} == {'comprehension','application'}
        assert len({q['id'] for q in qs}) == len(qs) and all('expected' in q and q['prompt'] for q in qs)
    return suite['cases']


def audit_stats(rows, summaries):
    # Independent arithmetic, without calling the producing summary functions.
    groups = {}
    for r in rows:
        groups.setdefault('/'.join(r[k] for k in ['case_id','context','arm']), []).append(r)
    assert summaries.keys() == groups.keys()
    for key, rs in groups.items():
        s = summaries[key]
        assert s['attempted'] == len(rs)
        assert s['succeeded'] == sum(r['error'] is None for r in rs)
        for name, stats in s['timings_ns_per_operation'].items():
            xs = sorted(r[name]/r['iterations'] for r in rs if r['error'] is None and r[name] is not None)
            if not xs:
                assert stats is None
                continue
            n = len(xs)
            median = xs[n//2] if n % 2 else (xs[n//2-1]+xs[n//2])/2
            assert stats == dict(n=n,min=xs[0],median=median,p95_nearest_rank=xs[math.ceil(.95*n)-1],max=xs[-1])


def audit(out):
    refs = {c['id']:c for c in audit_fixtures()}
    m = read(out / 'manifest.json')
    for name, h in m['inputs_sha256'].items():
        assert pilot.digest((ROOT / name).read_bytes()) == h, name
    for name, h in m['outputs_sha256'].items():
        assert pilot.digest((out / name).read_bytes()) == h, name
    assert m['cases'] == case_ids() and m['contexts'] == pilot.CONTEXTS and m['arms'] == pilot.ARMS
    assert (m['repetitions'],m['seed'],m['warmups'],m['iterations']) == (5,795,3,20)
    plans = pilot.schedule(m['cases'],m['repetitions'],m['seed'])
    assert read(out / 'plan.json') == plans
    rows = [json.loads(line) for line in (out / 'trials.jsonl').read_text().splitlines()]
    assert len(rows) == len(plans) == m['attempted'] == 4800
    assert len({r['trial_id'] for r in rows}) == 4800
    pilot.restore_csv((out / 'trials.csv').read_text(),rows)
    matrix = read(out / 'matrix.json'); checked = set(); common = {}
    for row, plan in zip(rows,plans):
        assert {k:row[k] for k in plan} == plan
        if row['error'] is not None:
            assert isinstance(row['error'],str) and row['error']
            continue
        assert row['returncode'] == 0 and row['iterations'] == 20 and row['warmup_iterations'] == 3
        for name in pilot.TIMES:
            value = row[name]
            assert (value is None) == (row['arm']=='conventional' and name.startswith('codec_'))
            assert value is None or type(value) is int and value >= 0
        key = '/'.join(row[k] for k in ['case_id','context','arm'])
        if key in checked:
            continue
        checked.add(key); f = matrix[key]; id = row['case_id']
        assert (f['case_id'],f['context'],f['arm']) == (id,row['context'],row['arm'])
        assert f['mode'] == 'offline_local' and f['source_digest_verified'] and f['ledger_verified'] and f['structural_roundtrip']
        assert all(v is None for v in f['metrics'].values())
        assert f['score'] is None and f['response_sha256'] is None and f['correct_assertions_per_1000_actual_input_tokens'] is None
        for field,path in [('source_sha256',fixture_path(id,'.txt')),('contract_sha256',fixture_path(id,'.contract.json')),('suite_sha256',SUITE),('cargo_lock_sha256',ROOT/'Cargo.lock')]:
            assert f[field] == pilot.digest(path.read_bytes())
        req = subprocess.check_output([str(BINARY),'expanded','request',id,row['context'],row['arm']])
        assert pilot.digest(req)==f['request_sha256'] and len(req)==f['final_request_bytes']
        request = json.loads(req); env=request.pop('context_envelope')
        assert len(pilot.dumps(env).encode()) == f['adapter_envelope_bytes']
        assert common.setdefault(id,request) == request
        assert request['questions'] == [{k:v for k,v in q.items() if k!='expected'} for q in refs[id]['questions']]
        for field,name in [('response_schema','response_schema'),('instructions','instructions')]:
            assert pilot.digest(request[field].encode())==f[name+'_sha256']
        assert len(request['response_schema'].encode())==f['response_schema_text_bytes']
        body = env['messages'][0]['content'] if row['context'] in ['human_model','model_model'] else (env['params']['arguments'] if row['context']=='model_tool' else env['documents'][0]['content'])
        if row['arm']=='conventional':
            payload = body.encode() if row['context']=='human_model' else pilot.dumps(json.loads(body) if isinstance(body,str) else body).encode()
            assert f['schema_text_bytes']==0
        else:
            packet = json.loads(body) if isinstance(body,str) else body
            assert packet['encoding']==row['arm']
            payload = packet['payload'].encode() if packet['transfer']=='utf8' else base64.b64decode(packet['payload'],validate=True)
            assert len(packet['schema'].encode()) == f['schema_text_bytes']
            if packet['transfer']=='base64':
                assert len(packet['payload'])==4*((len(payload)+2)//3)
        assert len(payload)==f['payload_bytes']
    assert set(matrix)==checked
    audit_stats(rows, read(out/'summary-by-cell.json'))
    pooled = read(out/'summary-pooled-by-arm.json')
    assert [s['arm'] for s in pooled] == pilot.ARMS
    audit_stats([dict(r,case_id='pooled',context='pooled') for r in rows],
                {'pooled/pooled/'+s['arm']:s for s in pooled})
    cm = read(out/'case-manifest.json')
    assert cm.keys()==refs.keys()
    for id,c in refs.items():
        assert cm[id] == dict(source_sha256=pilot.digest(fixture_path(id,'.txt').read_bytes()),extraction_sha256=pilot.digest(fixture_path(id,'.contract.json').read_bytes()),reference_sha256=pilot.digest(pilot.dumps(c).encode()))
    caps=read(out/'capabilities.json')
    assert [c['context'] for c in caps]==pilot.CONTEXTS+['model_native']
    assert all(c['live_endpoint'] is False for c in caps)
    assert [c['offline_roundtrip'] for c in caps]==[True]*4+[False]
    assert m['endpoint']['calls']==0 and m['endpoint']['configured'] is False
    failures=sum(r['error'] is not None for r in rows)
    print(f'expanded audit: {len(rows)} attempts; {len(rows)-failures} successes; {failures} failures retained; {len(matrix)} cells; 60 sources/references/extractions; order, exports, request bytes/hashes, per-cell stats, capability and null boundaries verified')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action',choices=['run','audit','fixtures'])
    parser.add_argument('directory',nargs='?',type=pathlib.Path)
    args=parser.parse_args()
    if args.action=='fixtures':
        audit_fixtures(); print('60 expanded fixtures audited')
    elif args.directory is None:
        parser.error('run/audit requires a directory')
    else:
        (run if args.action=='run' else audit)(args.directory.resolve())
