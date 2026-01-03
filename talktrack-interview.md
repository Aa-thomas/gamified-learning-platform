
Use this for ANY lab/project. Keep it truthful, demo-backed, and in your voice.

Project name: __________________________  Repo/Link: __________________________
Target role: ____________________________  Audience: ___________________________
Timebox: ☐30s ☐60s ☐2m ☐5m     Date: ____________     Version: _____

AUTHENTICITY RULES
- Only claim what you can point to: a file, test, benchmark, demo, PR, or screenshot.
- If you didn’t measure it, say “not measured yet” + your plan.
- If you’re unsure, describe the decision you made and what you’d change next.

====================================================================
1) ONE-LINER (memorize)
“I built ____________________ that ____________________ so that ____________________.”
- Built (what is it?): ______________________________________________
- Does (main capability): ___________________________________________
- So that (impact/outcome): _________________________________________

====================================================================
2) PROBLEM + CONSTRAINTS (2–4 bullets each)
Problem (what pain / need?):
- _________________________________________________________________
- _________________________________________________________________

Constraints (what rules shaped the design? check all that apply):
- ☐ Safety: no crashes/panics (or crash-safe boundaries)
- ☐ Determinism/reproducibility
- ☐ Performance/latency target
- ☐ Correctness/precision requirements
- ☐ Observability/logging/metrics
- ☐ Limited dependencies / portability
- ☐ Security / input validation
- ☐ UX constraints (CLI/GUI/API)
Other: _____________________________________________________________

Why it matters (real world mapping, 1–2 lines):
“This is useful in ____________________ because ____________________.”
- Domain/use-case: _________________________________________________

====================================================================
3) ARCHITECTURE (what parts, how they interact)
Core components (name 2–4 modules/services/layers):
- Component A: ____________________ responsibility: __________________
- Component B: ____________________ responsibility: __________________
- Component C: ____________________ responsibility: __________________
Boundaries (what’s pure logic vs I/O/integration?):
- Pure/core: _______________________________________________________
- I/O/shell: _______________________________________________________
Interfaces (public API/commands/endpoints/messages):
- _________________________________________________________________

Repo anchors (so you can point to proof fast):
- Main entrypoint: ____________________  Core module: _______________
- Design doc: _________________________  Runbook: ___________________
- Tests: ______________________________  Demo: ______________________

====================================================================
4) RELIABILITY / FAILURE MODES (2–5 bullets)
Primary invariant (the rule you enforced):
- “_______________________________________________________________”
Error/failure strategy (how failure is represented + handled):
- Errors are: ☐typed enum ☐status codes ☐exceptions ☐other: ___________
- Where failures are handled: _______________________________________
Two real failure modes you handle (be specific):
- Failure mode #1: ____________________ → outcome: _________________
- Failure mode #2: ____________________ → outcome: _________________
Recovery story (what does the operator/user do?):
- _________________________________________________________________

====================================================================
5) DATA / STATE / PERSISTENCE (skip if not relevant)
What state exists? (data structures/files/db/messages):
- _________________________________________________________________
Where it lives:
- ☐ In-memory ☐ File ☐ DB ☐ Queue ☐ Cache ☐ Other: __________________
Lifecycle (when load/init, when write/save, when cleanup):
- _________________________________________________________________
Data integrity approach (what you actually did):
- ☐ atomic write ☐ validation ☐ checksums ☐ versioning ☐ migrations
- Notes: ___________________________________________________________
Determinism note (ordering/format/reproducibility):
- _________________________________________________________________

====================================================================
6) UX / INTERFACE (CLI/HTTP/GUI/etc.) (1–3 bullets)
Interface type: ☐CLI ☐API ☐GUI ☐Library ☐Service ☐Other: _____________
User actions (commands/endpoints/workflows):
- _________________________________________________________________
How you handle invalid input gracefully:
- _________________________________________________________________
One “delight” feature (help text, ergonomics, shortcuts, etc.):
- _________________________________________________________________

====================================================================
7) TESTING / VERIFICATION (2–5 bullets)
What you proved (not “I tested it” — what property did you prove?):
- _________________________________________________________________
Test layers used:
- ☐ unit ☐ integration ☐ property-based ☐ fuzz ☐ golden tests ☐ e2e
Key edge cases covered:
- _________________________________________________________________
How to run proof (commands):
- `____________________________`   `____________________________`

====================================================================
8) PERFORMANCE / MEASUREMENT (truthful)
What matters (latency/throughput/memory/cpu/startup):
- _________________________________________________________________
How you measured (tooling + method):
- Tool: ☐criterion ☐hyperfine ☐wrk ☐perf ☐custom timer ☐other: ________
- Method (what you excluded, setup, dataset): ________________________
Results:
- If measured: p50 ______ p95 ______ p99 ______ (or avg ______)
- If not: “Not measured yet. Plan: ________________________________”

====================================================================
9) TRADEOFFS (1–3 crisp “I chose X because Y”)
Tradeoff #1:
- “I chose ____________________ over ____________________ because ____________________.”
Tradeoff #2 (optional):
- “I chose ____________________ over ____________________ because ____________________.”

====================================================================
10) NEXT STEP (what you’d do with more time)
Next step that improves the system (not “add features” — pick one theme):
- ☐ reliability ☐ performance ☐ usability ☐ observability ☐ security ☐ scale
Specific next step:
- _________________________________________________________________

====================================================================
11) CLOSE + INVITE QUESTIONS (1–2 sentences)
Close line:
- “_______________________________________________________________”
3 interviewer hooks (questions you WANT them to ask):
1) “Ask me about _________________________________________________”
2) “Ask me about _________________________________________________”
3) “Ask me about _________________________________________________”

====================================================================
TIMEBOX CUTS (pick one)
- 30s: 1 + 2 + 3 + close
- 60s: add 4 (reliability) + 1 proof anchor
- 2m: add 7 (testing) + 8 (perf plan/results) + 9 (tradeoff) + 10 (next)
- 5m: include deeper failure modes, data integrity, and 2 tradeoffs

FINAL CHECK (authenticity gate)
- ☐ Every claim has a repo anchor (file/test/demo/bench)
- ☐ I stated one real constraint
- ☐ I named two failure modes
- ☐ I gave either numbers OR a measurement plan
- ☐ I used my own words (no buzzword soup)
