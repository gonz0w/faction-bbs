"""Acceptance tests for issue-bound PRs; synthetic GitHub metadata, no network."""
import copy
import unittest
from check_issue_workflow import validate

REPO='gonz0w/faction-bbs'
BODY='''## Closing issues
Closes #181

## Acceptance and tests
#181: orphan/malformed closure targets and dishonest validation modes are rejected by test_issue_workflow; valid same-repository open tickets pass.

## TDD evidence
Mode: behavior
Red: python3 -m unittest discover -s tools -p test_issue_workflow.py fails because the policy module is missing, before implementing it.
Green: the same tests pass after implementing the metadata checker; repository and archive checks also pass.
Refactor: keep API reads separate from pure validation; preserve all acceptance assertions.
'''
ISSUE={'number':181,'state':'open','labels':[], 'body':'## Acceptance criteria\n- [x] Reject orphan PRs and close real issues.\n\n## Test plan\nRun policy tests with synthetic open/closed issue and PR metadata.'}
PR={'github_closing_issues':[{'number':181,'repository':REPO}], 'body':BODY,'state':'open','created_at':'2026-10-24T01:00:00Z','base':{'ref':'main','repo':{'full_name':REPO,'default_branch':'main'}}}
class IssueWorkflow(unittest.TestCase):
 def errors(self,pr=None,issues=None,files=None):
  return '\n'.join(validate(pr or copy.deepcopy(PR),issues if issues is not None else {181:copy.deepcopy(ISSUE)},files or ['tools/check_issue_workflow.py'],REPO))
 def test_behavior_with_real_issue_and_evidence_passes(self):self.assertEqual(self.errors(),'')
 def test_github_issue_form_headings_are_supported(self):
  i=copy.deepcopy(ISSUE);i['body']=i['body'].replace('## ','### ')
  self.assertEqual(self.errors(issues={181:i}),'')
 def test_every_closing_target_must_be_an_open_issue(self):
  for update in [{'state':'closed'},{'pull_request':{'url':'ignored'}},{'labels':[{'name':'type:epic'}]}]:
   with self.subTest(update=update):
    i=copy.deepcopy(ISSUE);i.update(update);self.assertNotEqual(self.errors(issues={181:i}),'')
  self.assertNotEqual(self.errors(issues={}),'')
 def test_closing_keyword_must_be_real_not_a_comment_quote_or_example(self):
  for target in ['Related #181','<!-- Closes #181 -->','```\nCloses #181\n```','> Closes #181','Closes other/repo#181','Closes https://github.com/other/repo/issues/181']:
   with self.subTest(target=target):
    p=copy.deepcopy(PR);p['body']=BODY.replace('Closes #181',target);self.assertIn('closing',self.errors(pr=p).lower())
 def test_multiple_issues_all_require_acceptance_and_validation(self):
  p=copy.deepcopy(PR);p['body']=BODY.replace('Closes #181','Closes #181\nFixes #182');i=copy.deepcopy(ISSUE);i['number']=182
  self.assertIn('#182',self.errors(pr=p,issues={181:ISSUE,182:i}))
  p['body']=p['body'].replace('#181: orphan','#181/#182: orphan')
  p['github_closing_issues'].append({'number':182,'repository':REPO})
  self.assertEqual(self.errors(pr=p,issues={181:ISSUE,182:i}),'')
  i['body']='Nothing actionable';self.assertIn('acceptance',self.errors(pr=p,issues={181:ISSUE,182:i}).lower())
 def test_only_default_branch_in_this_repository_can_close_work(self):
  for ref,repo in [('other',REPO),('main','other/repo')]:
   p=copy.deepcopy(PR);p['base']['ref']=ref;p['base']['repo']['full_name']=repo;self.assertNotEqual(self.errors(pr=p),'')
 def test_missing_acceptance_to_test_mapping_rejects_pr(self):
  p=copy.deepcopy(PR);p['body']=BODY.replace('## Acceptance and tests','## Unrelated notes');self.assertIn('Acceptance and tests',self.errors(pr=p))
 def test_red_green_refactor_evidence_is_required_for_behavior(self):
  for name in ['Red','Green','Refactor']:
   p=copy.deepcopy(PR);p['body']=BODY.replace(name+':','Other:');self.assertIn(name,self.errors(pr=p))
 def test_docs_exception_cannot_hide_executable_changes(self):
  p=copy.deepcopy(PR);p['body']=BODY.split('## TDD evidence')[0]+'## TDD evidence\nMode: docs\nReason: Only documentation links and text change; run repository link and archive checks.\n'
  self.assertEqual(self.errors(pr=p,files=['docs/board-guide.md','AGENTS.md']),'')
  for name in ['crates/faction-core/src/lib.rs','.github/ISSUE_TEMPLATE/helper.py']:
   self.assertIn('executable',self.errors(pr=p,files=[name]))
 def test_docs_and_research_accept_exact_temper_metadata(self):
  metadata=['.agent-temper/LICENSE','.agent-temper/SHA256SUMS']
  for mode in ['docs','research']:
   p=copy.deepcopy(PR);p['body']=BODY.split('## TDD evidence')[0]+f'## TDD evidence\nMode: {mode}\nReason: Documentation provenance and links are checked against the pinned upstream source.\n'
   for files in [[metadata[0]],[metadata[1]],metadata+['.agent-temper/METHOD.md']]:
    with self.subTest(mode=mode,files=files):
     self.assertEqual(self.errors(pr=p,files=files),'')
 def test_temper_metadata_cannot_hide_executable_unknown_or_lookalike_paths(self):
  rejected=[
   '.agent-temper/run.sh','.agent-temper/check.py','.agent-temper/config.yml',
   '.agent-temper/unknown','.agent-temper/nested/LICENSE',
   '.agent-temper/nested/SHA256SUMS','.agent-temper/../LICENSE',
   '../.agent-temper/LICENSE','.agent-temper/../.agent-temper/SHA256SUMS',
   '.agent-temper/./LICENSE','./.agent-temper/LICENSE',
   '/.agent-temper/LICENSE','.agent-temper//LICENSE',
   '.agent-temper/LICENSE.py','.agent-temper/SHA256SUMS.sh',
   '.agent-temper/LICENSE.bak','.agent-temper/license',
   '.agent-temper/sha256sums','.Agent-Temper/LICENSE',
   '.agent-temper-extra/LICENSE','other/.agent-temper/LICENSE',
  ]
  for mode in ['docs','research']:
   p=copy.deepcopy(PR);p['body']=BODY.split('## TDD evidence')[0]+f'## TDD evidence\nMode: {mode}\nReason: Documentation provenance and links are checked against the pinned upstream source.\n'
   for name in rejected:
    with self.subTest(mode=mode,path=name):
     self.assertIn('Docs/research exception cannot cover executable/configuration changes',self.errors(pr=p,files=['.agent-temper/LICENSE','.agent-temper/SHA256SUMS',name]))
 def test_no_fake_tdd_for_prior_implementation_but_cutoff_is_fixed(self):
  p=copy.deepcopy(PR);p['body']=BODY.split('## TDD evidence')[0]+'## TDD evidence\nMode: existing-implementation\nException: #181\nReason: This implementation and its regression evidence predate the migration; no retrospective red-first claim.\n'
  self.assertIn('predate',self.errors(pr=p));p['created_at']='2026-10-24T00:45:00Z';self.assertEqual(self.errors(pr=p),'')
 def test_actual_github_closing_links_cannot_exceed_declared_scope(self):
  for linked in [[{'number':181,'repository':REPO},{'number':185,'repository':REPO}],[{'number':181,'repository':'other/repo'}],None]:
   p=copy.deepcopy(PR);p['github_closing_issues']=linked;self.assertIn('GitHub',self.errors(pr=p))
 def test_unfilled_template_is_not_test_evidence(self):
  p=copy.deepcopy(PR);p['body']=BODY.replace('Red: python3 -m unittest discover -s tools -p test_issue_workflow.py fails because the policy module is missing, before implementing it.','Red: Replace with the actual command and observed failure before implementation.')
  self.assertIn('Red',self.errors(pr=p))
 def test_empty_changed_file_list_cannot_prove_docs_only(self):
  p=copy.deepcopy(PR);p['body']=BODY.split('## TDD evidence')[0]+'## TDD evidence\nMode: docs\nReason: Only documentation changes, checked with the repository link checker.\n'
  self.assertNotEqual(validate(p,{181:ISSUE},[],REPO),[])
 def test_placeholder_and_unfinished_acceptance_are_not_done(self):
  i=copy.deepcopy(ISSUE);i['body']=i['body'].replace('[x]','[ ]');self.assertIn('unchecked',self.errors(issues={181:i}))
  p=copy.deepcopy(PR);p['body']=BODY.split('## Acceptance and tests')[0]+'## Acceptance and tests\nTODO\n## TDD evidence\nMode: behavior\nRed: TODO\nGreen: TODO\nRefactor: TODO\n';self.assertNotEqual(self.errors(pr=p),'')
if __name__=='__main__':unittest.main()
