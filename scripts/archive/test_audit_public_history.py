#!/usr/bin/env python3
"""History scanning regressions use actual commits in disposable repositories."""
import os
import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from audit_public_history import audit


class HistoryAuditTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='archive-history-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.env = os.environ | {'GIT_CONFIG_GLOBAL':os.devnull,'GIT_CONFIG_NOSYSTEM':'1'}
        self.git('-c','init.templateDir=','init','-b','main')
        self.git('config','user.name','Archive Fixture')
        self.git('config','user.email','fixture@example.invalid')

    def git(self, *args):
        return subprocess.check_output(['git',*args],cwd=self.root,env=self.env,stderr=subprocess.DEVNULL)

    def commit(self, message):
        self.git('add','-A')
        self.git('commit','-m',message)

    def test_removed_content_and_old_metadata_are_still_scanned(self):
        (self.root/'notes.txt').write_text('private-fixture-person\n')
        (self.root/'.env').write_text('fixture configuration\n')
        self.commit('private-fixture-person authored this old commit')
        (self.root/'notes.txt').unlink()
        (self.root/'.env').unlink()
        (self.root/'README.md').write_text('Current snapshot is clean.\n')
        self.commit('Remove prior files')
        result = audit(self.root,['private-fixture-person'])
        self.assertEqual(result['commits'],2)
        identities = [x for x in result['findings'] if x['kind']=='identity']
        self.assertEqual({x['object_type'] for x in identities},{'blob','commit'})
        self.assertTrue(any(x['kind']=='excluded-artifact' for x in result['findings']))

    def test_clean_history_and_non_head_branch(self):
        (self.root/'README.md').write_text('Harmless archive fixture.\n')
        self.commit('Initial fixture')
        self.assertFalse(audit(self.root,['private-fixture-person'])['findings'])
        self.git('switch','-c','fixture-side-branch')
        (self.root/'hidden.txt').write_text('private-fixture-person\n')
        self.commit('Side branch change')
        self.git('switch','main')
        self.assertTrue(audit(self.root,['private-fixture-person'])['findings'])

    def test_synthetic_allowance_must_cover_every_historical_blob_path(self):
        token = 'sk-' + 'a' * 24
        (self.root/'one.txt').write_text(token)
        allow = self.root/'scripts/archive/scan_allowlist.json'
        allow.parent.mkdir(parents=True)
        allow.write_text(json.dumps([{'path':'one.txt','kind':'provider-token',
            'value_sha256':hashlib.sha256(token.encode()).hexdigest(),'reason':'Synthetic test fixture'}]))
        self.commit('One reviewed fixture path')
        self.assertFalse(audit(self.root,[])['findings'])
        (self.root/'two.txt').write_text(token)
        self.commit('Same blob at an unreviewed path')
        self.assertTrue(any(x['kind']=='provider-token' for x in audit(self.root,[])['findings']))


if __name__ == '__main__':
    unittest.main()
