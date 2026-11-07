#!/usr/bin/env python3
"""Run the unchanged #794 audit against its exact original input snapshot.

Source-hashing historical evidence must not be relabelled as a run of new code.
Current release binary still regenerates all old requests in the original audit.
"""
import hashlib
import json
import pathlib
import subprocess
import tempfile

ROOT=pathlib.Path(__file__).resolve().parents[1]
SNAPSHOT='bcac61c82540c2a7f18d584ebad85ffde869f4e4'


def main():
    evidence=ROOT/'docs/results/pilot'
    manifest=json.loads((evidence/'manifest.json').read_text())
    with tempfile.TemporaryDirectory(prefix='communications-pinned-pilot-') as name:
        temp=pathlib.Path(name)
        for path,h in manifest['inputs_sha256'].items():
            content=subprocess.check_output(['git','show',f'{SNAPSHOT}:{path}'],cwd=ROOT)
            assert hashlib.sha256(content).hexdigest()==h,path
            dest=temp/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(content)
        for p in evidence.iterdir():
            dest=temp/'docs/results/pilot'/p.name
            dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(p.read_bytes())
        binary=temp/'target/release/communications-probe'
        binary.parent.mkdir(parents=True,exist_ok=True)
        binary.symlink_to(ROOT/'target/release/communications-probe')
        subprocess.run(['python3','scripts/offline-pilot.py','audit','docs/results/pilot'],cwd=temp,check=True)
    print(f'Original #794 input hashes verified against immutable commit {SNAPSHOT}; exact requests verified with current release binary. Historical timings remain historical.')


if __name__=='__main__':main()
