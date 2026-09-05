Source: NASA-STD-8739.8B Software Assurance and Software Safety Standard
Reliability: HIGH (official NASA technical standard, publicly released, approved 2022-09-08)
Files examined:
  F:\.ELAI_workflow\.claude\papers\nasa-std-8739-8b.pdf (local, fetched from https://standards.nasa.gov/system/files/tmp/NASA-STD-8739.8B.pdf, HTTP 200, 681KB)
  F:\.ELAI_workflow\.claude\papers\nasa-std-8739-8b_extracted\nasa-std-8739-8b.md (MinerU extraction, 162KB)

Standard identity:
  Document: NASA-STD-8739.8B
  Title: Software Assurance and Software Safety Standard
  Revision B approval date: 2022-09-08 (baseline 2004-07-28, Rev A 2020-06-10)
  Issuing org: NASA Office of Safety and Mission Assurance (OSMA)
  Scope: software created, acquired, provided, or maintained by or for NASA

---

DETERMINISTIC-GATE RULES
(verbatim quotes with requirement-number cites, then ELAI mapping)

(a) SOFTWARE ASSURANCE AS AN INDEPENDENT ACTIVITY SEPARATE FROM DEVELOPMENT

Section 1.1.1 (mandatory requirement context):
"The purpose of the Software Assurance and Software Safety Standard is to define the requirements
to implement a systematic approach to Software Assurance (SA), software safety, and Independent
Verification and Validation (IV&V) for software created, acquired, provided, or maintained by or
for NASA. Various personnel in the program, project, or facility, and Safety and Mission Assurance
(SMA) organizations can perform the activities required to satisfy these requirements."

Section 1.3.1 (records as quality artifacts, mandatory):
"The software assurance and software safety information and plans are quality records. [...] The
software assurance and software safety organizations keep records, reports, metrics, analyses, and
trending results and keep copies of their project plans for future reference and improvements."

Section 1.3.2 (SMA organizational authority, mandatory):
"The software assurance and software safety organization and the project manager are responsible
for developing the software assurance and software safety plan(s). The software assurance
organization contributes to developing the Software Management Plan(s) and Software Development
Plan(s). The SMA organization has signature authority on software plans and documentation."

Section 4.1.2 (management support obligations, mandatory):
"Project and SMA Management support of the software assurance function is essential for software
assurance processes to be effective. The software assurance support minimally includes:
a. Management is familiar with and understands the software assurance function's purposes,
   concepts, practices, and needs.
b. Management provides the software assurance activities with skilled resources (people, equipment,
   knowledge, methods, facilities, and tools) to accomplish their project responsibilities.
c. Management receives and acts upon information provided by the software assurance function
   throughout a project."

Section 3.2 Definitions (normative definition of "Perform"):
"Software assurance does the action specified. Perform may include making comparisons of
independent results with similar activities performed by engineering, performing audits, and
reporting results to engineering."

Section 3.2 Definitions (normative definition of "Audit"):
"(1) systematic, independent, and documented process for obtaining audit evidence and evaluating
it objectively to determine the extent to which audit criteria are fulfilled [...]
(4) systematic, independent, documented process for obtaining records, statements of fact, or other
relevant information and assessing them objectively, to determine the extent to which specified
requirements are fulfilled."

Section 3.2 Definitions (normative definition of "Assure"):
"When software assurance personnel make certain that others have performed the specified software
assurance, management, and engineering activities."

SASS-01 (section 4.3.1, the primary mandatory requirement for all SA tasks):
"The responsible project manager shall perform the software assurance and software safety
activities defined in Table 1 per the requirements marked applicable in the software engineering
requirements mapping matrix for the software component. (SASS-01)"

Table 1 mapping to SWE-039 (mandatory SA audit obligation):
Software Assurance Task for SWE-039 (monitor/review/audit developer activities):
"5. Perform audits on software development processes and practices at least once every two years.
 6. Develop and provide status reports.
 7. Develop and maintain a list of all software assurance review discrepancies, risks, issues,
    findings, and concerns.
 8. Confirm that the project manager provides responses to software assurance and software safety
    submitted issues, findings, and risks and that the project manager tracks software assurance
    and software safety issues, findings, and risks to closure."

(b) IV&V INDEPENDENCE REQUIREMENT (technical, managerial, financial)

Section 3.2 Definitions (normative definition, sourced from ISO/IEC/IEEE 24765):
"Independent Verification and Validation (IV&V). Verification and validation performed by an
organization that is technically, managerially, and financially independent of the development
organization."

Section 4.4.1.2 (three-parameter independence definition, mandatory context):
"Three parameters define the independence of IV&V: technical independence, managerial
independence, and financial independence.

a. Technical independence requires that the personnel performing the IV&V analysis are not
   involved in the development of the system or its elements. The IV&V team establishes an
   understanding of the problem and how the system addresses the problem. Through technical
   independence, the IV&V team's different perspective allows it to detect subtle errors overlooked
   by personnel focused on developing the system.

b. Managerial independence requires that the personnel performing the IV&V analysis are not in
   the same organization as the development and program management team. Managerial independence
   also means that the IV&V team makes its own decisions about which segments of the system and
   its software to analyze and test, chooses the IV&V analysis methods to apply, and defines the
   IV&V schedule of activities. While independent from the development and program management
   organization, the IV&V team provides its findings in a timely manner to both of those
   organizations. The submission of findings to the program management organization should not
   include any restrictions (e.g., requiring the approval of the development organization) or any
   other adverse pressures from the development group.

c. Financial independence requires that the control of the IV&V budget be vested in a group
   independent of the software development organization. Financial independence does not
   necessarily mean that the IV&V team controls the budget but that the finances should be
   structured so that funding is available for the IV&V team to complete its analysis or test work.
   No adverse financial pressure or influence is applied."

Section 4.4.2.1 (shall requirement for IV&V planning):
"The IV&V Provider shall conduct planning and risk assessments to determine the specific
system/software behaviors (including the software components responsible for implementing the
behaviors) to be analyzed."

Section 4.4.2.4 (shall requirement for IV&V evidence delivery):
"The IV&V Provider shall provide analysis results, risks, and assurance statements and data to the
responsible organizations' project management, engineering, and software assurance personnel."

Section 4.4.2.8 (shall requirement for IV&V evidence retention):
"The IV&V Provider shall establish, record, maintain, report, and utilize IV&V management and
technical measurements."

Section 4.4.2.14 (shall requirement for defect tracking as records):
"The IV&V Provider shall track, record, and communicate defects/issues and other results found
during the execution of IV&V analysis and independent IV&V testing to the responsible
organizations' project management, engineering, and software assurance personnel."

Section 4.4.1.5 (objective evidence principle, normative):
"The center of the IV&V effort is on identifying and generating objective evidence that supports
the correct operation of the system or refutes the correct operation of the system."

Section 4.4.1.6 (two guiding principles for objective evidence, normative):
"a. Performing IV&V throughout the entire development lifetime is the first principle; potential
   problems should be detected as early as possible in the development life-cycle.
b. The second principle is 'appropriate assurance.' Given that it is not possible to provide IV&V
   on all aspects of a project's software, the IV&V Provider and project should balance risks
   against available resources to define an IV&V program for each project that provides IV&V so
   that the software will operate correctly, safely, reliably, and securely throughout its
   operational lifetime."

(c) ASSURANCE EVIDENCE AND RECORDS MUST BE PRODUCED AND RETAINED PER OBJECTIVE

Section 1.3.1 (quality records obligation):
"The software assurance and software safety information and plans are quality records. The format
of the documentation is a program/project/facility decision. The software assurance and software
safety organizations keep records, reports, metrics, analyses, and trending results and keep copies
of their project plans for future reference and improvements."

Section 4.4.2.8 (IV&V measurement records, mandatory):
"The IV&V Provider shall establish, record, maintain, report, and utilize IV&V management and
technical measurements."
Note: "The IV&V Provider gathers and analyzes metrics on a periodic basis to perform continuous
improvement of IV&V processes and identify indicators of IV&V and project risks."

Section 4.4.2.9 (tracking actual vs plan, mandatory):
"The IV&V Provider shall assess and track software activities' actual results and performance
against the software plans and identify and report any risks or findings to the responsible
organizations' project management, engineering, and software assurance personnel."

Section 4.4.2.12 (risk records, mandatory):
"The IV&V Provider shall identify, analyze, track, communicate, and record risks to the software
and development project in accordance with NPR 8000.4, Agency Risk Management Procedural
Requirements to the responsible organizations' project management, engineering, and software
assurance personnel."

Section 4.5.7 (risk and rationale records for tailoring):
"When the requirement and software class are applicable, the projects record the risk and rationale
for any requirement not completely implemented by the project. The projects can document their
related mitigations and risk acceptance in the approved Requirements Mapping Matrix."

---

ELAI MAPPING (one line each)

auditor agent = IV&V function: the auditor agent satisfies section 4.4.1.2 technical independence
  ("personnel performing the IV&V analysis are not involved in the development") because the
  auditor runs in its own context window, never reads the executor's reasoning, and is the sole
  runner of build/test/wire-check commands; it is structurally prevented from being the same agent
  that produced the code under review.

frozen gate-manifest = configuration control of the assurance toolchain: .claude/gate-manifest.txt
  and the frozen checker scripts satisfy section 4.4.1.2b managerial independence ("IV&V team
  makes its own decisions about which segments to analyze and test, chooses the IV&V analysis
  methods") because the gate is authored once out-of-band and the executing run cannot modify
  its own evaluator (wiring-enforcement R8).

each gate verdict = a retained assurance record (the SSB row): every auditor PASS/BLOCK verdict
  written as an SSB action_type row satisfies sections 1.3.1, 4.4.2.8, and 4.4.2.14 (records
  must be established, maintained, and tracked to closure); the SSB is the per-project quality
  record of IV&V findings.

---

ASSURANCE REQUIREMENTS ELAI DOES NOT YET SATISFY

Financial independence (section 4.4.1.2c): NASA requires that IV&V budget control be vested in a
  group independent of the development organization with "no adverse financial pressure." ELAI has
  no analog: the auditor agent and executor agent are both funded from the same API token budget
  controlled by the same user/project. There is no structural mechanism preventing the orchestrator
  from curtailing auditor cycles to save costs. UNVERIFIED whether this gap matters in a
  non-mission-critical software harness context; for safety-critical classification it would be
  a formal noncompliance.

Managerial independence (section 4.4.1.2b, partial): NASA requires the IV&V team to be in a
  different organization from program management. ELAI's auditor and main orchestrator share the
  same FSM, same session, same CLAUDE.md rule set, and the orchestrator can skip or truncate the
  auditor via orchestrate-protocol exceptions (H-10). The technical no-write-before-return rule
  (H-2) partially satisfies this but the orchestrator can override. Full managerial independence
  would require the auditor to have a separate authority path that the orchestrator cannot
  short-circuit.

Section 4.4.2.3 (SMA TA review and concurrence with the IPEP): NASA requires an independent
  technical authority to concur with the IV&V plan before execution. ELAI has no equivalent of a
  Center SMA TA; all plan approval is by the same user who directs implementation. This is a
  structural gap if ELAI were applied to safety-critical software.

Section 4.4.2.46 (independent testing of requirements tracing to hazardous events): NASA requires
  the IV&V provider to verify via independent testing each requirement that traces to a hazard.
  ELAI's auditor runs test suites but does not independently derive test cases from hazard analysis
  — it runs the project's own tests. This satisfies the "independent runner" criterion but not the
  "independently designed test" criterion.

---

Public API (principal obligations of the SA/IV&V function, as "interface"):

shall-conduct-planning | 4.4.2.1 | IV&V Provider | determine scope via risk assessment
shall-develop-IPEP | 4.4.2.2 | IV&V Provider | negotiate IV&V execution plan with project
shall-review-concur-IPEP | 4.4.2.3 | SMA TA | independent TA reviews and concurs with IV&V plan
shall-provide-analysis-results | 4.4.2.4 | IV&V Provider | deliver findings to PM + engineering + SA
shall-participate-reviews | 4.4.2.5 | IV&V Provider | attend all project reviews and report status
shall-establish-record-measurements | 4.4.2.8 | IV&V Provider | gather metrics, maintain records
shall-assess-track-vs-plan | 4.4.2.9 | IV&V Provider | track actual results vs plans; report risks
shall-record-risks | 4.4.2.12 | IV&V Provider | record all risks per NPR 8000.4
shall-verify-NPR7150.2 | 4.4.2.13 | IV&V Provider | verify project implements NPR 7150.2 requirements
shall-track-record-defects | 4.4.2.14 | IV&V Provider | track defects to closure as formal records
shall-ensure-defects-addressed | 4.4.2.15 | IV&V Provider | confirm project resolves findings
shall-verify-validate-requirements | 4.4.2.22 | IV&V Provider | confirm correct/consistent/complete/testable
shall-verify-code | 4.4.2.31-39 | IV&V Provider | verify code vs architecture/requirements/standards
shall-verify-tests | 4.4.2.40-45 | IV&V Provider | verify test coverage and results
shall-independent-test-hazards | 4.4.2.46 | IV&V Provider | independently test safety-critical paths
perform-audits | SWE-039 / Table 1 | SA function | at least once every 2 years on dev processes
maintain-discrepancy-list | SWE-039 / Table 1 | SA function | running list of findings + risks to closure

Behavioral contracts:

independence-is-structural-not-procedural: NASA-STD-8739.8B defines independence via
  organizational separation (different org, no shared management, no shared budget), not via
  procedure or promise. A reviewer who is in the same organization as the developer is NOT IV&V
  regardless of instructions given; the independence property is intrinsic to the organizational
  boundary.

missing-evidence-is-noncompliance: per sections 1.3.1, 4.4.2.8, 4.4.2.14, absence of a record is
  a finding, not a SKIP. The SA organization is obligated to produce and retain records; a gate
  that emits UNVERIFIED when evidence is absent is not compliant.

findings-must-reach-pm-without-restriction: section 4.4.1.2b specifies that IV&V findings must be
  submitted to program management without requiring "approval of the development organization."
  An auditor whose output is filtered or gated by the executor before reaching the orchestrator
  violates this principle.

tailoring-requires-documented-risk-acceptance: per section 4.5.7, any requirement not fully
  implemented must be documented with risk and rationale in the approved Requirements Mapping
  Matrix. A requirement skipped without a registered record is a noncompliance, not a
  de-scoping decision.

Platform constraints:

Applies to all software regardless of lifecycle model (section 1.1.4)
Safety-critical classification is determined by hazard analysis traceable to requirements (4.2.1)
Tailoring requires TA approval and is three-level: classification -> mapping matrix -> SA TA (4.5.2)
Records are "quality records" subject to NPR 1441.1 NASA Records Management (2.2.1 reference docs)

Gotchas:

The standard uses "shall" strictly for mandatory requirements; "should" is non-mandatory (1.1.4 language convention). Do not treat "should" statements as binding.
SA function can be performed by the same organization as development (the standard explicitly says "the same organization or different organizations can implement the requirements" - 4.1.3); only IV&V has the hard three-parameter independence requirement.
The SA independence principle is functional, not organizational: SA "makes certain that others have performed" activities (definition of "Assure," section 3.2); IV&V has organizational independence. Both exist in the standard simultaneously.
Financial independence applies only to IV&V, not general SA; conflating the two overspecifies the SA requirement.
SASS-01 is the master mandatory requirement; all Table 1 tasks flow from it; tailoring is only legal with TA approval and documented rationale.

Critical assertions (testable, mapped to ELAI):

assert auditor_agent_context != executor_agent_context, "IV&V technical independence: same context window violates non-involvement requirement (4.4.1.2a)"
assert gate_manifest_hash == stored_hash_at_plan_start, "gate-manifest must be frozen; run cannot modify its own evaluator (wiring-enforcement R8 mirrors 4.4.1.2b autonomy)"
assert len(ssb_rows_for_session) > 0 or audit_not_run, "IV&V must produce records (4.4.2.8 / 4.4.2.14); a completed audit with no SSB rows is a missing-evidence noncompliance"
assert verdict in ("PASS", "BLOCK"), "IV&V verdict must be binary; UNVERIFIED treated as FAIL (4.4.1.6 objective evidence principles; missing evidence is noncompliance not skip)"
assert findings_reach_orchestrator_without_executor_filter, "IV&V findings must reach PM without restriction or approval by development org (4.4.1.2b)"
assert tailoring_has_registered_risk_rationale or requirement_fully_implemented, "skipped requirements must have documented risk acceptance (4.5.7)"
