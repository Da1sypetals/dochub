~/.dochub/hub.toml
each entry:
key: hub.<skill_name>
value: <path>

dochub add <skill_name> <path>
- verify path exist and is a dir
- skill_name not duplicate
- add that entry

dochub prune
- remove all path-non-existence entries, report each.

dochub sanity
- check all path associated with an entry is not bigger than deafault 16MB. This size can be configured with the "sane-size" key (in MB).
- Report if any exceed.

dochub cp <skill_name> <dest>

copy recursively the path associated with <skill_name> to <dest>. Resulting structure should be <dest>/<skill_name>/content, i.e. copy the directory, NOT the content in the directory, use skill_name as the dir name

dochub rm <skill_name>
- prompt to show details and ask for confirmation.

dochub ls (list) <optional:skill_name>

if no skill_name specified, list all (skill_name map to path)

if skill_name specified, found -> show it, if not, report.


dochub use <skill_name> [dest]

hub.toml can have a key "skill-dir", list of string. list the relative path where skill should reside, e.g. ".claude/skill/" , ".cursur/skill/" , ".codebuddy/skill" . apply the logic of "dochub cp" to <dest>.join(each of skill-dir) ; dest defaults to . if not specified. ignore applies. 
Report to user all copy destinations.


Add comprehensive tests.