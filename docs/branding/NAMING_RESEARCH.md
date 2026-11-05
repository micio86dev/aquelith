# AQELITH naming research

Date: 2026-11-05. Codename: AQELITH (provisional, programming language). This is a factual availability and risk report. It is NOT legal clearance and must not be cited as such.

## Method and limitations

- Read-only lookups only: `gh` CLI, `curl` against public registry/RDAP endpoints, web search, and fetches of public trademark UIs. No accounts, purchases, reservations, contacts, or publications.
- Variants searched: aqelith, aquelith, aquelit, aqualith, aquilith, aquolith, acquelith, aqelit, aqulith, aquelyth, aqe-lith (not every variant on every source; see table).
- Limitations:
  - Registry 404 means "no package of that exact name today", not "name is safe". Registries can hold deleted/reserved names, and exact-name checks miss similar names.
  - RDAP "not found" is an indication of non-registration, not proof of availability (reserved/premium/blocked names, registry lag, registrar holds).
  - GitHub search is lexical; the shared API rate limit (5000/hr) was hit mid-run, so some variant user searches (aqulith, aquelyth, aqe-lith) are not completed.
  - Web search is a general engine (US-only), not an exhaustive index; absence of hits is weak evidence.
  - Trademark registries could NOT be queried reliably in an automated way (see Trademark registries). No trademark conclusion is possible from this run.
  - Phonetic/visual similarity analysis is my judgment, not an examiner's.

## Results by source

| source | query | date | result | limitation |
|---|---|---|---|---|
| GitHub repos (`gh search repos`) | aqelith, aquelith, aquilith, aquolith, acquelith, aqelit, aqulith, aquelyth, aqe-lith | 2026-11-05 | 0 results each | lexical search; exact tokens only |
| GitHub repos | aquelit | 2026-11-05 | 3 unrelated repos (user `aquelito` gist forks) | name differs by a letter; no language |
| GitHub repos | aqualith | 2026-11-05 | Several: `hergezod/Aqualith` (monophonic synth), `shn3vnn/aqualith`, `arjun-aa-arora/Aqualith-Wallpapers`, a French narrative/puzzle game "AQUALITH" | first 5 results only; none are programming languages |
| GitHub users/orgs (`search/users`) | aqelith, aquelith, aquolith, acquelith, aqelit | 2026-11-05 | 0 results | login-substring search only |
| GitHub users/orgs | aqualith / aquilith / aquelit | 2026-11-05 | aqualith: `Aqualith`, `aqualithmedia`; aquilith: `aquilithtechnologies`; aquelit: `aquelito`, `aquelity`, `Aquelity-Tech` | not inspected in depth |
| GitHub direct URL | github.com/aqelith, github.com/aquelith | 2026-11-05 | HTTP 404 both (handles appear unclaimed) | 404 on a profile URL is indicative; reserved names possible |
| GitHub code search | aqelith | 2026-11-05 | only bioinformatics/data files containing the token as noise (e.g. alignment data) | not meaningful |
| npm registry `registry.npmjs.org/<name>` | all 11 variants | 2026-11-05 | 404 all | exact-name only |
| npm search API | aqelith, aquelith | 2026-11-05 | 0 objects | |
| PyPI `pypi.org/pypi/<name>/json` | all 11 variants | 2026-11-05 | 404 all | PyPI normalizes names; exact only |
| crates.io API | all 11 variants + search aqelith/aquelith | 2026-11-05 | 404 / 0 results | |
| RubyGems | aqelith, aquelith | 2026-11-05 | 404 | |
| Go module proxy | github.com/aqelith/aqelith, github.com/aquelith/aquelith | 2026-11-05 | 404 | only those two paths |
| Homebrew formulae API | aqelith, aquelith | 2026-11-05 | 404 | formulae only, not casks/taps |
| Maven Central search | aqelith, aquelith | 2026-11-05 | numFound 0 (spellcheck suggests aquality, eulith, modulith...) | |
| Docker Hub `v2/repositories/<ns>/` | aqelith, aquelith | 2026-11-05 | count 0, but a known-nonexistent namespace gives the same response | INCONCLUSIVE: endpoint cannot distinguish an unused namespace from an empty one |
| Web search | "aqelith" / "aquelith" programming language | 2026-11-05 | no result for the names; nearest unrelated "AQ" language (aq-org/AQ) | search engine coverage |
| Web search | "Aqelith" | 2026-11-05 | no exact hits; neighbours: Aqualith (fictional race in a tabletop forum post), Shaqilath, Aqeel (Arabic name) | |
| Web search | Aqualith/Aquilith/Aquolith + software trademark | 2026-11-05 | no marks for those exact strings; nearby marks: AQUILON (US Cl.9 software, live), AQUILIUM (dead), AQUALX | third-party aggregator sites, not authoritative |
| RDAP (see Domains) | 11 domain/TLD combinations | 2026-11-05 | see Domains | |
| TMview / EUIPO eSearch | aqelith | 2026-11-05 | blocked ("Request Rejected" WAF) | not queried |
| WIPO Global Brand Database | aqelith | 2026-11-05 | altcha proof-of-work challenge page | not queried |
| USPTO trademark search | aqelith | 2026-11-05 | JS single-page app shell, no API results | not queried |
| UIBM banca dati | aqelith | 2026-11-05 | legacy JS UI returned, no results retrievable | not queried |

## Similar names found

Exact AQELITH / AQUELITH: nothing found in any source checked (GitHub, npm, PyPI, crates.io, RubyGems, Go, Homebrew, Maven, web search).

Neighbours (none is a programming language as far as seen):

- **Aqualith** (edit distance 2-3): GitHub org `Aqualith`, `aqualithmedia`; repos for a synth, wallpapers, a perfume brand, a French puzzle game. Domain aqualith.com registered since 2018-07-12. Fictional race name in a forum thread.
- **Aquilith**: GitHub org `aquilithtechnologies`; aquilith.com registered (created 2023-09-15).
- **Aquolith**: aquolith.com registered (created 2026-05-14, i.e. recent; possibly someone targeting the same coinage space).
- **Aquelit / aquelito / Aquelity-Tech**: GitHub accounts only; Aquelity-Tech is a tech-named org (not inspected).
- **AQ** (aq-org/AQ): an unrelated interpreted language with a very short, partially overlapping prefix; low confusion but same field.
- Marks in the wider "Aqu-" software space: AQUILON (live US Class 9), AQUILIUM (dead), AQUALX. Typical crowded "Aqua-/Aqui-" prefix field, so a likelihood-of-confusion analysis against Class 9/42 marks is plausible and must be done by the registries/counsel.
- Component words: "lith" = stone (Greek lithos), as in monolith/megalith (very common suffix: Eclipse "Modulith", Spring Modulith, "Eulith" are in Maven spellcheck neighbours). "Aqe"/"Aque" has no clear meaning.

## Domains

RDAP queried via authoritative registry endpoints (Verisign for .com/.net, Google Registry for .dev/.app, PIR for .org, Identity Digital for .io/.ai). All five endpoints were sanity-checked with a known registered domain (google.com/.dev/.app/.io/.ai, example.org returned 200), so a 404 is a valid "not found" for that registry. The generic rdap.org proxy rate-limited (429) and was only used for early probing.

| domain | RDAP result 2026-11-05 | interpretation |
|---|---|---|
| aqelith.com / .net / .dev / .app / .org / .io / .ai | 404 (not found) | appears unregistered; NOT proof it can be registered |
| aquelith.com / .net / .dev / .app / .org / .io / .ai | 404 | same |
| aquelit.com, acquelith.com, aqelit.com, aqulith.com | 404 (.com only) | appear unregistered |
| aqualith.com | 200, created 2018-07-12 | registered |
| aquilith.com | 200, created 2023-09-15 | registered |
| aquolith.com | 200, created 2026-05-14 | registered |
| "lang-style" TLDs (.lang, .xyz, .tech) | `.lang` does not exist as a TLD (rdap.org 404 is meaningless); .xyz/.tech not authoritatively checked | unknown |

Caveats: not found does not exclude premium/reserved/blocked labels, registry-lock, or a grace/pending-delete state. Check with a registrar search before relying on this. Nothing was bought or reserved. Also GitHub handles `aqelith` and `aquelith` returned 404 (unclaimed as of now).

## Trademark registries

Actually checked (and what came out):

- EUIPO eSearch plus / TMview: automated access rejected by WAF ("Request Rejected"). NOT checked.
- WIPO Global Brand Database: proof-of-work challenge page. NOT checked.
- USPTO trademark search: JS SPA, no unauthenticated API. NOT checked.
- UIBM (Italy): legacy UI, results not retrievable automatically. NOT checked.
- Indirect only: general web search and third-party aggregator pages (Justia, trademarkdir, markinton) surfaced no marks for AQELITH, AQUELITH, AQUALITH, AQUILITH or AQUOLITH. This is weak evidence (aggregators are incomplete, delayed, and not authoritative). Nothing found for the exact name does NOT mean clear.

Conclusion: trademark status of AQELITH is UNKNOWN. No claim of clearance is made.

## Risk assessment

| area | level | reasons |
|---|---|---|
| Exact-name collision in software/language ecosystems | low | zero hits on GitHub, 5 package registries, web for AQELITH and AQUELITH |
| Package/handle availability (npm, PyPI, crates, GitHub) | low | all names free today; but not reserved, so first-mover squatting is possible until claimed (Docker Hub inconclusive) |
| Domains | low-medium | .com/.dev/.app/.org/.io/.ai/.net not found for AQELITH; typo-variants (aqualith, aquilith, aquolith .com) taken so look-alike squatting/confusion exists; not proof of registrability |
| Phonetic/visual confusion with existing names | medium | Aqualith/Aquilith/Aquolith/Aquelit exist as orgs/products (synth, game, perfume, tech companies); "Aqu-" software marks exist (AQUILON live US Cl.9). Spelling is distinct but pronunciation "ah-keh-lith" vs "aqua-lith" could be close in speech |
| Trademark (EU/US/IT/WIPO) | unknown | no authoritative registry could be queried. Class 9/42 (software, SaaS, dev tools) conflicts cannot be excluded; this is the main open risk |
| Meaning/connotation | low | no known meaning in English, Italian, Spanish, French, German; "lith" = stone is neutral; "aquel" (Spanish/Portuguese "that one") is only a faint echo of AQUELITH, not AQELITH. Spelling "Aqe" is hard to type/remember (usability risk, not legal). Not verified by native speakers or for Arabic/Chinese/Japanese transliteration |
| Descriptiveness / distinctiveness | low risk | coined word, inherently distinctive (good for registrability) |
| Searchability / SEO | low-medium | few exact hits now (good), but misspellings are heavily attracted to Aqualith/Aquelit results |

Ranked real risks: (1) unknown trademark status in Classes 9/42 (EU, US, IT); (2) confusion with Aqualith/Aquilith/Aquolith-type names in speech and search; (3) unreserved handles/domains/packages can be taken before launch; (4) typing/spelling difficulty ("qe").

## Manual steps for the user

1. EUIPO eSearch plus (https://euipo.europa.eu/eSearch/): Trade marks > Advanced search; word mark "AQELITH", then repeat with AQUELITH, AQUALITH, AQUILITH, AQUOLITH, AQELIT; use "contains" and "similar/phonetic" if offered; filter Nice classes 9, 42 (and 35/41 optional).
2. TMview (https://www.tmdn.org/tmview/): same terms, offices EUIPO + UIBM + USPTO + WIPO + UK; classes 9 and 42.
3. WIPO Global Brand Database (https://branddb.wipo.int/): Brand name = AQELITH; complete the challenge in a browser; also search by phonetic option; classes 9, 42.
4. USPTO (https://tmsearch.uspto.gov/): search "AQELITH" and wildcard `AQ*LITH*`, `AQU*LITH*`; Class 009/042; include dead records to see abandoned marks.
5. UIBM Italy (https://www.uibm.gov.it/bancadati/): marchi > ricerca per denominazione "AQELITH"; Classi 9, 42.
6. Registrar check (e.g. Namecheap/Cloudflare registrar search, no purchase) for aqelith.{com,dev,io,ai,app,org,net}; confirm not premium/reserved. Check whois history for squatting.
7. Also check in a browser: github.com/aqelith (org/user), npm/PyPI/crates web UIs for exact and `@aqelith` scope, Docker Hub namespace, X/Bluesky/YouTube/Mastodon handles.
8. Before any public use, have a trademark attorney (EU + Italy + US) perform a formal clearance with similarity analysis; this report cannot substitute for it.
9. Optional: have native speakers (Spanish/Portuguese, Arabic, Chinese, Japanese) sanity-check the pronunciation and transliteration.
