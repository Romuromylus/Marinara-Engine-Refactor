# Bug And Work Updates

Use this folder to report bugs, track active bug ownership, current work, and status updates.

New bugs start in `unowned-bugs.md` when no one owns them yet. When someone starts fixing a bug, move the bug entry into that person's file under `people/`.

Use the user's GitHub name to determine who is working on a bug. If the user asks for their identity or how to track their bugs, check `git config user.name`, `git config user.email`, and `gh auth status` when GitHub CLI is logged in.

After identifying the user, compare that identity against the files already listed in `people/` and choose the closest matching owner file. For example, local Git user `munimunigamer` should be tracked in `people/muni.md` because `muni.md` is the matching person file.

Known mappings:

- `Coda` -> Chai

## Files

- `unowned-bugs.md`: Bugs that do not have a current owner.
- `people/mari.md`: Mari's owned bugs and current work.
- `people/deci.md`: Deci's owned bugs and current work.
- `people/romu.md`: Romu's owned bugs and current work.
- `people/jorge.md`: Jorge's owned bugs and current work.
- `people/promansis.md`: Promansis's owned bugs and current work.
- `people/chai.md`: Chai's owned bugs and current work.
- `people/xel.md`: Xel's owned bugs and current work.
- `people/sunny.md`: Sunny's owned bugs and current work.
- `people/tld.md`: TLD's owned bugs and current work.
- `people/muni.md`: Muni's owned bugs and current work.

## Bug Entry Template

```md
## Bug title

- Status: Unowned | Investigating | In progress | Blocked | In review | Done
- Owner: Name or Unowned
- Impact area: UI | engine | shared/api | Rust capability | docs | unknown
- Reported:
- Last updated:

### Steps

1.

### Expected


### Actual


### Notes


```

## Status Rules

- Move bugs out of `unowned-bugs.md` when someone takes ownership.
- Keep the owner file updated with current status, next step, and blockers.
- Mark resolved bugs as `Done` before removing them from active tracking.
