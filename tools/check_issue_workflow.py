#!/usr/bin/env python3
"""Read-only PR/issue closure policy. Pure validation is tested without GitHub."""
from __future__ import annotations
import argparse,json,os,re,sys
from datetime import datetime
from pathlib import Path
from urllib.request import Request,urlopen

LEGACY_CUTOFF='2026-10-24T00:51:14Z'  # Issue #181 opened; never advance this cutoff.

def prose(text):
    text=re.sub(r'<!--.*?-->', '', text or '', flags=re.S)
    text=re.sub(r'(?ms)^\s*(`{3,}|~{3,}).*?^\s*\1[^\n]*$', '', text)
    return '\n'.join(line for line in text.splitlines() if not line.lstrip().startswith('>'))

def closing_numbers(body):
    return sorted({int(n) for n in re.findall(r'^\s*(?:close[sd]?|fix(?:es|ed)?|resolve[sd]?)\s*:?\s+#([1-9]\d*)\s*\.?\s*$',prose(body),re.I|re.M)})

def section(body,title):
    match=re.search(r'^#{2,3} '+re.escape(title)+r'\s*\n(.*?)(?=^#{2,3} |\Z)',prose(body),re.M|re.S)
    return match.group(1).strip() if match else ''

def substantive(text):
    return len(text.strip())>=30 and not re.search(r'\b(?:TODO|TBD|PLACEHOLDER)\b|Replace with|Map every closing issue',text,re.I)

def doc_file(name):
    return name.endswith(('.md','.rst','.txt','.svg','.png','.jpg','.jpeg','.webp')) or name in {'.gitignore','.gitattributes','.editorconfig','LICENSE','.agent-temper/LICENSE','.agent-temper/SHA256SUMS'} or (name.startswith('.github/ISSUE_TEMPLATE/') and name.endswith(('.yml','.yaml')))

def validate(pr,issues,files,repository):
    errors=[];base=pr.get('base',{});repo=base.get('repo',{})
    if repo.get('full_name')!=repository or base.get('ref')!=repo.get('default_branch','main'):
        errors.append('PR must target this repository default branch for automatic issue closure')
    body=pr.get('body') or '';numbers=closing_numbers(body)
    if not numbers:errors.append('Missing real closing directive: one Closes #NUMBER line per issue')
    linked=pr.get('github_closing_issues')
    if not isinstance(linked,list) or any(x.get('repository')!=repository for x in linked) or {x.get('number') for x in linked}!=set(numbers):
        errors.append('GitHub-recognized closing links must exactly match declared same-repository issue scope')
    for n in numbers:
        issue=issues.get(n)
        if not issue or issue.get('number')!=n or issue.get('pull_request') or issue.get('state')!='open':
            errors.append(f'Closing target #{n} must be an existing open issue, not a PR');continue
        labels={x if isinstance(x,str) else x.get('name') for x in issue.get('labels',[])}
        if 'type:epic' in labels:errors.append(f'Closing target #{n} is an epic; close a finite executable issue instead')
        acceptance=section(issue.get('body',''),'Acceptance criteria')
        if not re.search(r'^- \[[xX]\] .+',acceptance,re.M):errors.append(f'Issue #{n} needs completed acceptance criteria')
        if re.search(r'^- \[ \] ',acceptance,re.M):errors.append(f'Issue #{n} has unchecked acceptance criteria')
        if not substantive(section(issue.get('body',''),'Test plan')):errors.append(f'Issue #{n} needs a concrete Test plan')
    if not substantive(section(body,'Acceptance and tests')):errors.append('PR needs substantive Acceptance and tests mapping')
    for n in numbers:
        if not re.search(r'#'+str(n)+r'\b',section(body,'Acceptance and tests')):errors.append(f'Acceptance and tests must map closing issue #{n}')
    evidence=section(body,'TDD evidence');mode=re.search(r'^Mode: (behavior|docs|research|existing-implementation)\s*$',evidence,re.M)
    if not mode:errors.append('TDD evidence needs Mode: behavior, docs, research, or existing-implementation');return errors
    mode=mode.group(1)
    if mode=='behavior':
        for field in ['Red','Green','Refactor']:
            m=re.search(r'^'+field+r':\s*(.+)$',evidence,re.M)
            if not m or not substantive(m.group(1)):errors.append(f'TDD evidence needs concrete {field} command/outcome')
    else:
        m=re.search(r'^Reason:\s*(.+)$',evidence,re.M)
        if not m or not substantive(m.group(1)):errors.append('TDD exception needs a concrete Reason and validation')
        if mode in {'docs','research'} and (not files or not all(doc_file(n) for n in files)):
            errors.append('Docs/research exception cannot cover executable/configuration changes or unknown file sets')
        if mode=='existing-implementation':
            try:old=datetime.fromisoformat(pr['created_at'].replace('Z','+00:00'))<datetime.fromisoformat(LEGACY_CUTOFF.replace('Z','+00:00'))
            except (KeyError,ValueError):old=False
            if not old or not re.search(r'^Exception: #181\s*$',evidence,re.M):errors.append('Existing implementation must predate #181 with its explicit Exception; never invent retrospective TDD')
    return errors

def github(path,data=None):
    token=os.environ.get('GH_TOKEN') or os.environ.get('GITHUB_TOKEN')
    if not token:raise ValueError('Read-only GitHub token is required for PR metadata validation')
    req=Request('https://api.github.com/'+path,data=None if data is None else json.dumps(data).encode(),headers={'Authorization':'Bearer '+token,'Accept':'application/vnd.github+json','X-GitHub-Api-Version':'2022-11-28','Content-Type':'application/json'})
    with urlopen(req,timeout=20) as response:return json.load(response)

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--event',required=True,type=Path);a=p.parse_args();event=json.loads(a.event.read_text())
    if 'pull_request' not in event:print('Issue closure gate: not a pull-request event.');return 0
    repository=os.environ.get('GITHUB_REPOSITORY','')
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+',repository):raise ValueError('Invalid repository context')
    number=event.get('number');assert isinstance(number,int) and number>0
    pr=github(f'repos/{repository}/pulls/{number}')
    owner,name=repository.split('/')
    query='query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){pullRequest(number:$number){closingIssuesReferences(first:100){pageInfo{hasNextPage} nodes{number repository{nameWithOwner}}}}}}'
    response=github('graphql',{'query':query,'variables':{'owner':owner,'name':name,'number':number}})
    if response.get('errors'):raise ValueError('GitHub closing-link lookup failed')
    linked=response['data']['repository']['pullRequest']['closingIssuesReferences']
    if linked['pageInfo']['hasNextPage']:raise ValueError('Closing-issue set exceeds finite 100-issue budget')
    pr['github_closing_issues']=[{'number':x['number'],'repository':x['repository']['nameWithOwner']} for x in linked['nodes']]
    issues={n:github(f'repos/{repository}/issues/{n}') for n in closing_numbers(pr.get('body'))};files=[]
    for page in range(1,32):
        batch=github(f'repos/{repository}/pulls/{number}/files?per_page=100&page={page}');files.extend(x['filename'] for x in batch)
        if len(batch)<100:break
    else:raise ValueError('Changed-file enumeration exceeded fixed 3000-file budget')
    errors=validate(pr,issues,files,repository)
    if errors:
        for error in errors:print('ERROR:',error,file=sys.stderr)
        return 1
    print('Issue closure and acceptance/TDD evidence passed for:',', '.join('#'+str(n) for n in issues));return 0
if __name__=='__main__':
    try:sys.exit(main())
    except Exception as exc:print('Issue closure gate failed:',type(exc).__name__,file=sys.stderr);sys.exit(1)
