import pathlib, re, sys
w=(pathlib.Path(sys.argv[1])/".github/workflows/documentation.yml").read_text()
record=len(re.findall(r"^ +python3 tools/check_recorded_files\.py dev-docs/evidence/attachment-bytes/manifest\.json [a-f0-9]{64}$",w,re.MULTILINE))
unit=w.count("python3 -m unittest discover -s experiments/250-attachment-bytes -p test_proof.py")
print("CI registration counts:",record,unit)
assert (record,unit)==(1,1), "CI must pin the actual attachment manifest and execute stdlib proof contracts exactly once"
