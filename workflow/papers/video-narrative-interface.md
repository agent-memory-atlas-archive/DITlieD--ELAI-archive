Source: Kun's agentic engineering workflow video (transcript)
Reliability: MEDIUM-doc (author design claims, not measured by us)
Files examined:
  C:\Users\user\Downloads\Hi everyone, welcome to this video.txt  (original transcript, full read)
  F:\.ELAI_workflow\docs\.source\video-narrative.txt  (copy)

Author background (from transcript):
  "My name is Kun. I was previously an L8 principal engineer worked at Meta, Microsoft and Atlassian
  on many large scale systems like the Bing search engine, Windows and Facebook games. In the recent
  couple of years, I have been building frontier coding agents at Atlassian and helped many engineering
  teams figure out how to use them effectively."

Central framing (captain/crew/firstmate mental model):
  AUTHOR DESIGN CLAIM: "By the end of this video, I want you to feel like a captain that can sail a
  large ship with a crew of agents working for you and do so in a stress-free and satisfying way."
  - You are the captain: set direction, hold quality bar, manage agents
  - Agents are crew mates: execute implementation autonomously
  - Firstmate (firstmate tool): manages crew mates on your behalf so you focus on the big picture
  - Terminal + tmux: the ship (persistent, multiplexed, cross-device)

Core thesis -- "spend time at START and END, free the MIDDLE":
  AUTHOR DESIGN CLAIM: "The place where I spend time is towards the beginning and the end of the task.
  At the beginning, I would spend time in lavish to plan the requirements more clearly.
  At the end, I would come in and hold a bar on quality.
  All these parts in the middle is done by AI which frees me up to spin up other tasks."
  AUTHOR DESIGN CLAIM: "I can keep going because none of these sessions will need my attention
  anytime soon."

"Treat yourself as an eng-manager not a diff-reviewer" claim:
  AUTHOR DESIGN CLAIM: "My advice here is that in order to really scale ourselves with AI,
  we have to think of ourselves more as an engineering manager or engineering director.
  Your directors most likely don't review any PRs. Yet, they can influence the quality of their
  team's software by creating good culture and processes and rely on the team to carry them out."
  AUTHOR DESIGN CLAIM: "I never stare at this screen. I would go spin up other tasks.
  I come back only when no mistakes says all checks passed."
  AUTHOR DESIGN CLAIM: "For low-risk changes, I don't really look at the diff at all because I have
  validated time and time again for low-risk changes. Any problem I could catch is very likely
  already caught by the pipeline."

Memory file vs skill (progressive disclosure onboarding model):
  AUTHOR DESIGN CLAIM: "Global memory file: everything in this file gets loaded into the system
  prompt of every single agent session across all our projects. If we have too much content in this
  file, it will silently use a lot of our tokens."
  - Global file: personal preferences only (e.g. "never use mdash", "don't give too much weight to
    development cost", "when doing bug fixes always start with reproducing the bug end-to-end")
  - Project file: collective learning from all agent sessions on that project
  - Building the project file: "every time I saw the agent doing something wrong, I would correct it
    and ask it to remember to not make the same mistake again by storing the learning into this memory file"
  AUTHOR DESIGN CLAIM: "Over time, our crew mates working on this project get smarter and more experienced.
  You don't need any fancy memory system to do that. This markdown file is all it takes."
  Skills as overflow: "converting this kind of conditionally useful information from the memory file
  into skills... only when it actually decides that it needs to use a certain skill, it will then
  read the rest of this file. This allows you to store a lot of the knowledge about how to do various
  kind of things without blowing up your system prompt."

Voice input productivity claim (Stanford paper):
  AUTHOR DESIGN CLAIM: "There's an actual paper from Stanford that seriously compared the efficiency
  and basically talking is three times faster than typing."
  NOTE: he cites this as a Stanford paper; names Dario Amodei as appearing in that paper's references
  for prior speech recognition work from 2016. Paper title NOT stated; we have NOT verified this
  citation independently. UNVERIFIED: exact paper, exact 3x figure, sample size, methodology.

Explicit warning against popular-but-unevaluated skills:
  AUTHOR DESIGN CLAIM: "you should generally avoid installing random skills from the internet
  even the ones that have a lot of GitHub stars."
  AUTHOR DESIGN CLAIM: "First of all, these skills can instruct your agent to run pretty much
  anything on your machine. This is a very risky thing to do because the agent can leak your API keys
  or even credentials to your bank account to untrusted third parties without you knowing."
  Karpathy skills repo benchmark result:
  AUTHOR DESIGN CLAIM: "I actually evaluated the skill in this repo [andrekarpathy/agi-skills or similar,
  177,000 GitHub stars] with program bench, which tests the agent's ability to build programs end to end.
  And the result shows that by using this skill, the agent will use 5% more tokens while making the
  results worse."
  NOTE: repo name not stated precisely in transcript; "Andre Kaparthi skills" is the phonetic transcription.
  UNVERIFIED: exact repo name, programbench methodology, 5% token increase number, "worse results" delta.
  "if you look closely, this skill is not even written by Andre Kapathy himself"
  "A lot of the skills being widely shared today have not been rigorously evaluated"
  "Their GitHub stars only tell you how popular they are and not whether they are actually helpful"
  AUTHOR DESIGN CLAIM: "I recommend that you do not install any skill from the internet that claims
  to magically make your agent perform better, but hasn't published anything rigorous that proves its claim."

Terminal + tmux + worktree parallelism model:
  Terminal choice: Wezterm (26k stars, cross-platform Mac/Win/Linux, Lua config, hot-reload)
  Multiplexer: tmux (splits, tabs/windows, persistent sessions, SSH-accessible from phone)
  Editor: Neovim (keyboard-only, relative line numbers, muscle memory nav)
  Parallel agents: each agent in its own tmux tab/window
  Worktrees for isolation: "if I spin up another agent working in the same directory, they will step
    on each other's toes and cause conflicts. The default solution here is git work tree."
  Worktree overhead: "this work tree basically becomes a debt" -- requires manual tracking and removal

Tool inventory (every tool named + one-line purpose per the author):
  treehouse:       "drop me into a fresh work tree where I can start doing whatever I want...
                   Next time I ask for a work tree, it will try to reuse one of the idle work trees"
  good night have fun (gnhf): "a free and open-source tool I built specifically for long-running tasks...
                   Just give it an objective and it will keep going until it meets some stop condition"
                   Exposes: moon-count per iteration, commit count, token usage monitoring
                   AUTHOR DESIGN CLAIM: prevents quota from being consumed overnight unlike /go in codex/claude-code
  firstmate:       "someone I can talk to as a captain that will carry out all my directions and
                   manage all the crew mates for me so I can focus on the big picture stuff"
                   Mechanism: talks to it, it spawns tmux tabs + treehouse worktrees + runs agents autonomously
                   "It realizes that this is not one task, but three parallel tasks"
  no-mistakes:     "orchestrates your agent to execute a series of steps that takes this first pass
                   code all the way through to a clean PR"
                   Pipeline steps (AUTHOR DESIGN CLAIM, verbatim order):
                     1. create branch if not exists
                     2. commit
                     3. isolated work tree (nothing affects current repo during validation)
                     4. understand original intent by analyzing agent session
                     5. rebase on latest main + resolve merge conflicts
                     6. adversarial review in fresh context window (obvious problems self-corrected,
                        ambiguous ones escalated to human)
                     7. end-to-end test against original intent + record evidence
                     8. documentation pass (update all relevant docs)
                     9. check no linking problems
                     10. push branch to remote + raise PR
                     11. babysit PR (merge conflicts, CI failures) until merged
                   PR output: original intent summary, what changed, how tested, evidence (screenshot/video/log),
                     problems found+fixed, risk assessment
                   AUTHOR DESIGN CLAIM: "This is how I scale up the volume of code changes I do every day
                     through a large crew of agents without losing control on quality."
  lavish (lavish axi):  "Lavish editor basically instructed the agent to create an HTML artifact to
                   visualize what we need to discuss. It always uses the same design system as the
                   current project being worked on."
                   Named "richer than a rich editor" (almost named "filthy rich editor")
                   Supports: annotations + comments on specific artifact regions, click to make decisions,
                     send feedback back to agent without returning to terminal
                   AUTHOR DESIGN CLAIM: "I can never go back to reading text in the terminal anymore.
                   This is just like way too much more efficient."
  axi (and axi.mmd catalog):  "a set of design standards I authored after discovering the huge upside
                   we can have by designing our tools to treat agents as a first class citizen and
                   optimize for agent ergonomics. I created 10 principles for how to make a tool
                   highly efficient for agents."
                   GitHub axi benchmark result:
                   AUTHOR DESIGN CLAIM: "Using GitHub MCP server will cost you to spend three times more
                   on token cost and more than double the latency compared to using the CLI."
                   AUTHOR DESIGN CLAIM: "Using token efficient output format can save about 40% tokens
                   compared to using JSON."
                   Includes: GitHub axi, ChromeDevTools axi
                   UNVERIFIED: 3x MCP vs CLI claim, 40% token format claim, benchmark methodology
  open super whisper (OpenSuperWhisper):  "completely free and open source... runs the whisper model
                   locally on your machine... the quality is like really really good"
                   Custom vocab: "Open super whisperer actually supports this customization through
                   system prompt. So what we can do here is in this model menu in the transcription menu
                   there is an initial prompt and we can put in some common vocabulary that we use"
                   AUTHOR DESIGN CLAIM: "this prompt is what makes the transcription really good"
                   AUTHOR DESIGN CLAIM: "I have pretty much completely moved to voice input now"
                   Fall-back to typing: URLs, file paths ("Trust me, you don't want to speak a URL out loud")
  skills (npx skills / bunx skills):  "my main tool for installing and managing skills...
                   it supports pretty much any agent"
                   skill-creator: "a skill called skill creator which was written by Anthropic...
                   you can install a skill called skill creator... Once this skill is installed,
                   your agent will be able to follow the rules and create new skills for you"

Harness comparison (four agents Kun uses regularly):
  Claude Code:   "the only practical choice if you are using the subscription from Anthropic.
                 most sensible default experience out of the box. Pretty rich feature set.
                 downside: sometimes a little buggy, not as customizable."
  Codex CLI:     "written in Rust, feels smoother. Open source so Codeex can inspect its own source.
                 lacking in bells and whistles, not very customizable."
  Pi:            "philosophy is to be minimal and highly extensible. Great if you don't want bloat."
  OpenCode:      "buttery smooth TUI. Good integration with pretty much every model.
                 More complete out-of-the-box than Pi. Model agnostic, grab from shelf and go."
  AUTHOR DESIGN CLAIM: "I have been very strict about making my workflow agent agnostic because
  the landscape is changing very very fast."

Agent bias fix (training memory bug):
  AUTHOR DESIGN CLAIM: "If we ask a Frontier model to estimate the development cost of a project...
  the estimate is in days and weeks and months. But if we ask the agent to actually build it now,
  I can guarantee it will come back with a playable version in just a few minutes."
  AUTHOR DESIGN CLAIM: "This mismatch is happening because the models were trained from human data
  and that is what a typical human developer would give as the estimate. AI doesn't seem to know
  it can code much faster than humans yet."
  Fix: "When making technical decisions, don't give too much weight to development cost" in global
  memory file.

End-to-end testing preference:
  AUTHOR DESIGN CLAIM: "AI models today by default like to write unit tests which are often not
  sufficient and not really covering the product behaviors we want to guard. I found that leaning into
  end to end testing is a lot more reliable."

Productivity claim:
  AUTHOR DESIGN CLAIM: "I have been shipping 40 to 50 PRs almost every day, sometimes more.
  And these are all well-tested and shipped to production, not those Minecraft demos you see people
  vibe code on social media."

UNVERIFIED claims:
  Stanford "3x faster" paper: title, authors, exact metric, sample size not stated
  Karpathy skills repo: exact repo URL/name, "5% more tokens, worse results" benchmark numbers
  GitHub MCP "3x token cost, 2x latency" vs CLI: methodology not described
  "40% token savings" from non-JSON output format: methodology not described
  "40-50 PRs per day": personal claim, no external verification
