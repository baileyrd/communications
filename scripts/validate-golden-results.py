import json,csv,hashlib,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parents[1]
for name,count in [('golden-offline',96),('golden-synthetic-replay',7)]:
 rows=[json.loads(line) for line in (root/'docs/results'/f'{name}.jsonl').read_text().splitlines()]
 assert len(rows)==count
 with (root/'docs/results'/f'{name}.csv').open(newline='') as f: table=list(csv.DictReader(f))
 assert len(table)==count
 for row,record in zip(rows,table):
  assert record.keys()==row.keys()
  for key,value in row.items():
   assert (record[key] if isinstance(value,str) else json.loads(record[key]))==value,(name,key)
  assert row['ledger_verified'] and row['structural_roundtrip'] and row['source_digest_verified']
  assert all(v is None for v in row['metrics'].values())
  assert row['correct_assertions_per_1000_actual_input_tokens'] is None
  req=subprocess.check_output([str(root/'target/release/communications-probe'),'golden','request',row['case_id'],row['context'],row['arm']])
  assert len(req)==row['final_request_bytes']
  assert hashlib.sha256(req).hexdigest()==row['request_sha256']
  source=(root/'fixtures/golden'/f"{row['case_id']}.txt").read_bytes()
  assert hashlib.sha256(source).hexdigest()==row['source_sha256']
  assert hashlib.sha256((root/'fixtures/golden'/f"{row['case_id']}.contract.json").read_bytes()).hexdigest()==row['contract_sha256']
  assert hashlib.sha256((root/'fixtures/golden/suite.json').read_bytes()).hexdigest()==row['suite_sha256']
  assert hashlib.sha256((root/'Cargo.lock').read_bytes()).hexdigest()==row['cargo_lock_sha256']
  if name=='golden-offline':assert row['score'] is None and row['mode']=='offline_local'
  else:assert row['mode']=='synthetic_replay' and row['iterations']==0
 assert len({(r['case_id'],r['context'],r['arm']) for r in rows})==(96 if name=='golden-offline' else 1)
 print(f'{name}: {count} rows; CSV restores every JSONL value; all exact request/source/contract/suite/lock hashes verified; all telemetry null; modes valid')
replays=[json.loads(line) for line in (root/'fixtures/golden/synthetic-replay.jsonl').read_text().splitlines()]
outputs=[json.loads(line) for line in (root/'docs/results/golden-synthetic-replay.jsonl').read_text().splitlines()]
for i,o in zip(replays,outputs):assert hashlib.sha256(i['response_json'].encode()).hexdigest()==o['response_sha256']
assert [r['score']['fidelity'] for r in outputs]==[1,.75,.75,.75,.25,None,.5]
example=(root/'docs/results/golden-request-example.json').read_bytes()
assert hashlib.sha256(example).hexdigest()==outputs[0]['request_sha256']
print('7 synthetic response hashes and exact expected score outcomes verified; example request digest matches replay binding')
