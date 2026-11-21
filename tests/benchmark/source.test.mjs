import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { test } from 'node:test';

const source = path => new URL(`../../${path}`, import.meta.url);
const read = path => readFileSync(source(path), 'utf8');
const authoring = 'skills/cirnet-work-prompt-authoring/SKILL.md';
const refactoring = 'skills/cirnet-refactoring/SKILL.md';
const template = 'templates/chat-to-work-prompt.md';
const newSkills = [
  'cirnet-task-continuity',
  'cirnet-plan-spec-authoring',
  'cirnet-candidate-evaluation',
];
const pairs = [
  authoring,
  refactoring,
  template,
  ...newSkills.map(name => `skills/${name}/SKILL.md`),
  'skills/README.md',
  'docs/context.md',
  'docs/behavioral-evaluation.md',
  'docs/model-compatibility.md',
  'plans/cirnet-rules/adaptive-stable/plan.md',
  'specs/cirnet-rules/adaptive-stable/spec.md',
];

test('Vietnamese footer identifies the exact English source bytes', () => {
  for (const path of pairs) {
    const digest = createHash('sha256').update(readFileSync(source(`en/${path}`))).digest('hex');
    assert.ok(read(`vi/${path}`).endsWith(
      `<!-- source: en/${path} -->\n<!-- source-sha256: ${digest} -->\n`,
    ), path);
  }
});

test('both languages expose the same 16 Skills through the fallback index', () => {
  const skillNames = lang => readdirSync(source(`${lang}/skills`), { withFileTypes: true })
    .filter(entry => entry.isDirectory() && existsSync(source(`${lang}/skills/${entry.name}/SKILL.md`)))
    .map(entry => entry.name).sort();
  const skills = skillNames('en');
  assert.equal(skills.length, 16);
  for (const name of newSkills) assert.ok(skills.includes(name), `Missing Skill: ${name}`);
  assert.deepEqual(skillNames('vi'), skills);
  for (const lang of ['en', 'vi']) {
    const links = read(`${lang}/skills/README.md`).matchAll(/\]\((cirnet-[a-z0-9-]+)\/SKILL\.md\)/g);
    assert.deepEqual(Array.from(links, match => match[1]).sort(), skills, `${lang} fallback index`);
  }
});

test('both languages keep one Work Prompt template', () => {
  for (const lang of ['en', 'vi']) {
    const templates = readdirSync(source(`${lang}/templates`)).filter(name => name.includes('work-prompt'));
    assert.deepEqual(templates, ['chat-to-work-prompt.md']);
    for (const mode of ['LIGHT', 'STANDARD', 'DEEP']) assert.ok(read(`${lang}/${template}`).includes(mode));
  }
});

test('the canonical authoring pair and evaluation guides route model changes to the compatibility procedure', () => {
  for (const lang of ['en', 'vi']) {
    const term = lang === 'en' ? /model-agnostic/i : /độc lập model/i;
    for (const path of [authoring, template]) assert.match(read(`${lang}/${path}`), term);
    for (const path of [authoring, template, 'docs/context.md', 'docs/behavioral-evaluation.md']) {
      assert.match(read(`${lang}/${path}`), /model-compatibility\.md/, `${lang}/${path}`);
    }
  }
});

test('Author stops discovery at a safe handoff and preserves boundaries on read failure', () => {
  const skill = read(`en/${authoring}`);
  const guide = read(`en/${template}`);
  assert.match(skill, /Draft the goal and granted scope before discovery/);
  assert.match(skill, /Stop discovery once you can hand off safely/);
  assert.match(guide, /Only retry with a verified\s+alternative[\s\S]*same read scope and output limits/);
  assert.match(guide, /Honor access and\s+shell-policy denials/);
  assert.match(guide, /`NOT_INSPECTED`[\s\S]*scoped Work preflight/);
  assert.match(guide, /Only block the handoff when an essential decision, authority, conflict or\s+safety condition/);
});

test('refactoring preserves contracted error behavior and reports incidental diagnostics', () => {
  const skill = read(`en/${refactoring}`);
  assert.match(skill, /Preserve error types, codes and documented message text/);
  assert.match(skill, /Report incidental runtime diagnostic wording changes without expanding the edit/);
});

test('refactoring checks the separators left behind by a removed declaration', () => {
  const skill = read(`en/${refactoring}`);
  assert.match(skill, /remove the blank separator that belonged only to the declaration/);
  assert.match(skill, /review the scoped diff including whitespace/i);
});
