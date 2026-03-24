~/.dochub/hub.toml
each entry:
key: hub.<name>
value: <path>

dochub add <name> <path>
- verify path exist and is a dir
- name not duplicate
- add that entry

dochub prune
- remove all path-non-existence entries, report each.

dochub sanity
- check all path associated with an entry is not bigger than deafault 16MB. This size can be configured with the "sane-size" key (in MB).
- Report if any exceed.

dochub cp <name> <dest>

copy recursively the path associated with <name> to <dest>. Resulting structure should be <dest>/<name>/content, i.e. copy the directory, NOT the content in the directory, use name as the dir name

dochub rm <name>
- prompt to show details and ask for confirmation.

dochub ls (list) <optional:name>

if no name specified, list all (name map to path)

if name specified, found -> show it, if not, report.


dochub skill cp <name> <dest>

hub.toml can have a key "skill-dir", list of string. list the relative path where skill should reside, e.g. ".claude/skill/" , ".cursur/skill/" , ".codebuddy/skill" . apply the logic of "dochub cp" to <dest>.join(each of skill-dir) ; dest defaults to . if not specified. ignore applies. 
Report to user all copy destinations.


Add comprehensive tests.