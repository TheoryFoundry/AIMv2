# Problem Clarifier

Use when the user's input is exploratory, ambiguous, or not yet a checkable mathematical proposition. Convert rough ideas, examples, and goals into a small set of concrete candidate mathematical problems, help the user pick or refine one, and only then hand off to proof search.

## Goal

Turn an underspecified math request into a precise problem statement that AIM can responsibly solve.

Do not start theorem proving immediately when the user has not yet provided a clear mathematical target. First make the target explicit enough that success and failure are checkable.

## When To Use

Use this skill when one or more of the following is true:

- The user gives a topic, intuition, example, or research direction, but not a formal question.
- The user asks for "ideas", "a framework", "what can be studied here", or something similarly open-ended.
- The input mixes several possible goals, such as proving, classifying, constructing, optimizing, or finding counterexamples.
- Important objects, assumptions, domains, quantifiers, or desired outputs are missing.
- The request sounds mathematical, but it is still unclear what a complete answer should look like.

Do not use this skill when the user has already given a well-formed mathematical statement or a clearly checkable task.

## Clarity Test

Before solving, check whether the current request already determines all of the following:

1. The main mathematical objects.
2. The setting and assumptions.
3. The task type.
4. The success condition.

Treat the problem as clear only if you can restate it as a concrete proposition or task such as:

- prove that `...`
- determine whether `...`
- classify all `...`
- construct an example of `...`
- compute `...`
- give a counterexample to `...`

If any of these are still materially unclear, stay in clarification mode.

## Clarification Workflow

1. Restate the user's current input in compact mathematical language.
2. Say briefly why it is not yet fully well-posed.
3. Propose 2 to 4 concrete candidate problem statements derived from the user's input.
4. For each candidate, give:
   - the precise task
   - the missing assumptions you are choosing
   - why this is a natural reading of the user's request
5. Prefer specific options over open-ended questions.
6. Ask the user to choose one option or refine one of them.
7. Once the user confirms, restate the chosen problem in a final precise form before solving.

## Proposal Style

Keep proposals grounded in the user's input. Do not invent a completely unrelated theorem just to make the problem look formal.

When proposing candidates:

- Start with the most conservative interpretation first.
- If useful, include one easier local version and one stronger or more general version.
- Make hidden assumptions explicit.
- Keep notation stable across options.
- Avoid long essays; aim for compact, high-signal alternatives.

Use a numbered list in this exact style:

1. `Problem statement`
   Assumptions: `...`
   Why this fits: `...`

2. `Problem statement`
   Assumptions: `...`
   Why this fits: `...`

## After Confirmation

Once a clear problem has been selected:

1. Restate the final problem precisely.
2. If the clarified problem contains important source information from the user, record it as theorem-graph context before using it.
3. Switch out of clarification mode and continue with normal proof exploration.
4. Do not keep re-proposing alternatives unless the current formalization fails or the user asks to broaden or change it.

## Safety Notes

- Do not pretend that a vague research direction is already a theorem.
- Do not claim success criteria that the user did not agree to.
- If multiple formalizations are plausible, surface the ambiguity instead of silently choosing a high-risk interpretation.
- If the user explicitly asks for brainstorming rather than proof, you may stay at the proposal stage and not force a theorem.
