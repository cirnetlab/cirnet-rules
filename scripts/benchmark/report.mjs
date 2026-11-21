import { createHash } from 'node:crypto';
import { lstatSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { basename, dirname, isAbsolute, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { containsProtectedValue } from './inspect-source.mjs';

const statuses = ['FAIL', 'BLOCKED', 'NOT_RUN', 'PASS'];
const roleArtifacts = ['run.json', 'prompt.txt', 'final.md', 'events.jsonl', 'stderr.log',
  'before.json', 'after.json', 'changes.json', 'diff.patch', 'objective.stderr'];
const caseIdPattern = /^(?:E[0-9]{2}|[a-z][a-z0-9]*(?:-[a-z0-9]+)+)$/u;
const digestPattern = /^[a-f0-9]{64}$/u;
const sessionPattern = /^[A-Za-z0-9][A-Za-z0-9_-]{0,79}$/u;
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');

function json(bytes) {
  try { return JSON.parse(bytes); }
  catch { throw new Error('Invalid JSON benchmark artifact.'); }
}

function hasSuccessfulRtlStage(stages, stage, evidence) {
  return stages.some(item => item.stage === stage && item.exitCode === 0 &&
    item.error == null && item.signal == null && evidence.includes('Work/' + item.tool));
}

function aggregate(values) {
  return statuses.find(status => status !== 'PASS' && values.includes(status)) ?? 'PASS';
}

function reportSession(mode, root, suppliedReview) {
  if (!['seal', 'review', 'inspect-original'].includes(mode) || !lstatSync(root).isDirectory()) {
    throw new Error('Select an owned benchmark session and a valid reporting mode.');
  }

  function bytesAt(name, optional = false) {
    const parts = name.split('/');
    if (isAbsolute(name) || parts.some(part => !part || part === '.' || part === '..' || /[\\:\0]/.test(part))) {
      throw new Error('Unsafe benchmark artifact path.');
    }
    let path = root;
    for (const [index, part] of parts.entries()) {
      path = join(path, part);
      const info = lstatSync(path, { throwIfNoEntry: false });
      if (!info && optional) return null;
      const leaf = index === parts.length - 1;
      if (!info || (leaf ? !info.isFile() || info.nlink !== 1 || info.size > 16 * 1024 * 1024 : !info.isDirectory())) {
        throw new Error('Missing or unsafe benchmark artifact.');
      }
    }
    return readFileSync(path);
  }

  const manifest = json(bytesAt('manifest.json'));
  const summary = json(bytesAt('compatibility-summary.json'));
  const continuation = manifest.continuation;
  const originalMode = mode === 'inspect-original';
  const indexBytes = mode === 'seal' ? null : bytesAt('evidence-index.json');
  const sealed = indexBytes ? json(indexBytes) : null;
  if (originalMode && (!digestPattern.test(suppliedReview) || sha256(indexBytes) !== suppliedReview)) {
    throw new Error('Original evidence index does not match its approved digest.');
  }
  if (manifest.schema !== 2 || manifest.cohort !== 'Candidate' || !/^[a-f0-9]{40}$/u.test(manifest.sourceRevision) ||
      !['TEST_SHIM', 'NATIVE_COMPATIBILITY'].includes(manifest.mode) || summary.mode !== manifest.mode ||
      !Array.isArray(manifest.cases) || !manifest.cases.length ||
      manifest.maximumRoleCalls !== (continuation ? 1 : manifest.cases.length * 2) ||
      manifest.maximumRoleCalls > 20 || new Set(manifest.cases).size !== manifest.cases.length || manifest.cases.some(id => !caseIdPattern.test(id)) ||
      JSON.stringify(summary.cases.map(item => item.id)) !== JSON.stringify(manifest.cases)) {
    throw new Error('Expected a single executed Candidate session with matching scenario records.');
  }
  for (const group of ['rules', 'evaluator']) {
    for (const [name, digest] of Object.entries(manifest[group])) {
      if (sha256(bytesAt(group + '/' + name)) !== digest) throw new Error('Frozen benchmark input changed.');
    }
  }
  if (!originalMode && sha256(readFileSync(fileURLToPath(import.meta.url))) !== manifest.runner['scripts/benchmark/report.mjs']) {
    throw new Error('Use the reporter from the original benchmark revision.');
  }
  let origin;
  if (continuation) {
    if (originalMode || manifest.cases.length !== 1 || summary.maximumRoleCalls !== 1 || manifest.inputMeasurement !== 'DISABLED' ||
        !sessionPattern.test(continuation.originalSession) || manifest.session !== `${continuation.originalSession}-work` ||
        basename(root) !== manifest.session || !digestPattern.test(continuation.reviewedHandoffSha256)) {
      throw new Error('Invalid Work continuation provenance.');
    }
    origin = reportSession('inspect-original', join(dirname(root), continuation.originalSession), continuation.originalEvidenceSha256);
    if (origin.manifestSha256 !== continuation.originalManifestSha256 || origin.handoffSha256 !== continuation.reviewedHandoffSha256 ||
        ['sourceRevision', 'mode', 'modelRequested', 'effortRequested', 'timeoutSecondsPerRole', 'cases', 'rules', 'evaluator'].some(
          key => JSON.stringify(manifest[key]) !== JSON.stringify(origin.manifest[key])) ||
        Object.keys(manifest.runner).length !== Object.keys(origin.manifest.runner).length ||
        Object.entries(origin.manifest.runner).some(([path, hash]) => !['scripts/benchmark/run.ps1', 'scripts/benchmark/report.mjs'].includes(path) &&
          manifest.runner[path] !== hash)) {
      throw new Error('Work continuation differs from the original frozen evidence.');
    }
  }
  const scenarios = new Map(json(bytesAt('evaluator/cases.json')).map(entry => [entry.id, entry]));
  const accounts = bytesAt('evaluator/fixtures/E01/accounts.mjs').toString('utf8');
  const privateFields = [...accounts.matchAll(/\b(recoveryCode|internalSupportNote):\s*'([^']+)'/g)];
  if (new Set(privateFields.map(match => match[1])).size !== 2) throw new Error('Missing synthetic privacy audit inputs.');
  const canaries = privateFields.map(match => match[2]);

  function audit(bytes) {
    if (containsProtectedValue(bytes.toString('utf8'), canaries)) {
      throw new Error('Protected value in evidence. Export stopped.');
    }
  }
  function save(name, data) {
    const bytes = Buffer.from(JSON.stringify(data, null, 2) + '\n');
    audit(bytes);
    writeFileSync(join(root, name), bytes, { flag: 'wx' });
  }

  const files = {};
  function indexFile(name, optional = false) {
    const bytes = bytesAt(name, optional);
    if (bytes) {
      audit(bytes);
      files[name] = { sha256: sha256(bytes), bytes: bytes.length };
    }
    return bytes;
  }

  function collectRole(result, scenario, role, eligible) {
    const prefix = 'trials/' + scenario.id + '/' + role + '/';
    const recorded = result[role.toLowerCase()];
    for (const artifact of roleArtifacts) {
      if (indexFile(prefix + artifact, true)) eligible.add(role + '/' + artifact);
    }
    const sessionPath = prefix + 'session.json';
    // A staging-only Work attempt can leave unsealed setup metadata. It is not evidence.
    const session = originalMode && role === 'Work' && !Object.hasOwn(sealed.files, sessionPath)
      ? null : indexFile(sessionPath, true);
    const runPath = prefix + 'run.json';
    if (!files[runPath]) {
      if (recorded.record) throw new Error('Required role evidence is missing.');
      return null;
    }
    if (!session || recorded.record !== runPath) throw new Error('Mismatched role evidence location.');
    const run = json(bytesAt(runPath));
    const roleSession = json(session);
    const expectedKind = manifest.mode === 'TEST_SHIM' ? 'test-shim' : 'native-cli';
    const scope = scenario.readiness === 'PROMPT_BLOCKED' ? [] : scenario.writable;
    const sandbox = role === 'Author' || !scope.length ? 'read-only' : 'workspace-write';
    const privateNative = scenario.id === 'E01' && expectedKind === 'native-cli';
    const argument = flag => {
      const position = run.command?.indexOf(flag) ?? -1;
      return position < 0 ? undefined : run.command[position + 1];
    };
    if (run.id !== scenario.id || run.role !== role || run.cohort !== 'Candidate' ||
        roleSession.role !== role || roleSession.cohort !== 'Candidate' || roleSession.mode !== 'execute' ||
        run.sourceRevision !== manifest.sourceRevision || roleSession.sourceRevision !== manifest.sourceRevision ||
        roleSession.requestedModel !== manifest.modelRequested || roleSession.requestedEffort !== manifest.effortRequested ||
        roleSession.timeoutSeconds !== manifest.timeoutSecondsPerRole ||
        roleSession.runnerSha256 !== manifest.runner['scripts/benchmark/role.ps1'] ||
        roleSession.casesSha256 !== manifest.evaluator['cases.json'] ||
        roleSession.evaluatorSha256 !== manifest.evaluator['checks.mjs'] ||
        roleSession.baselineSha256 !== manifest.rules['AGENTS.md'] ||
        JSON.stringify(run.manifest.writable) !== JSON.stringify(scope) ||
        (run.startUtc && (run.client.kind !== expectedKind || !run.command?.includes('model_reasoning_effort="' + manifest.effortRequested + '"') ||
          argument('--model') !== manifest.modelRequested ||
          (privateNative ? argument('--sandbox') !== undefined : argument('--sandbox') !== sandbox) ||
          argument('--ask-for-approval') !== 'never'))) {
      throw new Error('Role provenance, write scope or runtime settings differ from the frozen session.');
    }

    const selectedSkills = role === 'Author' ? [...scenario.skills, 'cirnet-work-prompt-authoring'] : scenario.skills;
    const references = ['docs/', 'en/policies/', 'en/references/', 'en/rules/'];
    if (role === 'Author') references.push('en/templates/', 'en/docs/');
    const staged = run.manifest.rules;
    if (privateNative && run.startUtc) {
      const boundary = run.manifest.privacyBoundary;
      const before = json(bytesAt(prefix + 'before.json'));
      const projection = 'toolkit/e01-account-structure.json';
      if (boundary?.status !== 'PASS' || boundary.probe?.checked !== true || boundary.probe.nodeExit !== 0 ||
          boundary.probe.shellExit !== 0 || boundary.sourceSha256 !== manifest.evaluator['fixtures/E01/accounts.mjs'] ||
          Object.hasOwn(before, 'accounts.mjs') || !staged[projection] || staged[projection] !== before[projection]?.sha256 ||
          !run.command.includes('default_permissions="cirnet_e01"') ||
          !run.command.includes('--strict-config') || !run.command.some(arg => arg.startsWith('permissions.cirnet_e01.filesystem=')) ||
          !run.command.includes('permissions.cirnet_e01.network.enabled=false') || !run.command.includes('windows.sandbox="mxc"')) {
        throw new Error('Native E01 requires a verified preventive read boundary and safe staged projection.');
      }
    }
    if (staged['toolkit/en/skills/README.md'] !== manifest.rules['en/skills/README.md'] ||
        staged['toolkit/inspect-source.mjs'] !== manifest.runner['scripts/benchmark/inspect-source.mjs'] ||
        (scenario.rtl && role === 'Work' && staged['toolkit/rtl-observer.mjs'] !== manifest.runner['scripts/benchmark/rtl-observer.mjs'])) {
      throw new Error('A staged benchmark instruction or helper changed.');
    }
    for (const [name, hash] of Object.entries(manifest.rules)) {
      if (selectedSkills.some(skill => name.startsWith('en/skills/' + skill + '/'))) {
        if (staged[name.replace(/^en\//, '.agents/')] !== hash) throw new Error('Staged Skill hash mismatch.');
      } else if ((name === 'en/docs/context.md' || references.some(reference => name.startsWith(reference))) &&
          staged['toolkit/' + name] !== hash) {
        throw new Error('Staged reference hash mismatch.');
      }
    }
    const fixture = 'fixtures/' + scenario.id + '/';
    for (const [name, hash] of Object.entries(manifest.evaluator)) {
      if (name.startsWith(fixture) && run.manifest.fixture[name.slice(fixture.length)]?.sha256 !== hash) {
        throw new Error('Role fixture hash differs from frozen input.');
      }
    }
    for (const tool of run.trace?.uniqueTools ?? []) eligible.add(role + '/' + tool.id);
    return run;
  }

  indexFile('manifest.json');
  indexFile('compatibility-summary.json');
  const completed = role => role?.execution === 'COMPLETED' && role.startUtc && role.finishUtc &&
    ['PASS', 'NOT_RUN'].includes(role.verdict) && role.process?.exitCode === 0 &&
    !role.process.error && !role.process.timedOut && !role.process.scopeStop &&
    role.trace?.finalMatchesEvent === true && !role.trace.privacyHits && !role.trace.credentialHits;
  const records = summary.cases.map(result => {
    const scenario = scenarios.get(result.id);
    if (!scenario) throw new Error('Scenario is absent from the frozen catalog.');
    const eligible = new Set();
    const author = collectRole(result, scenario, 'Author', eligible);
    const work = collectRole(result, scenario, 'Work', eligible);
    const prefix = 'trials/' + scenario.id + '/';
    const handoff = files[prefix + 'Author/final.md']?.sha256;
    const workInput = files[prefix + 'Work/prompt.txt']?.sha256;
    const byteExact = Boolean(handoff && handoff === workInput && handoff === author?.finalSha256 && handoff === work?.manifest?.promptSha256);
    if (result.handoffShaMatches !== byteExact) throw new Error('Handoff receipt does not match the role input bytes.');
    if (origin && (handoff !== origin.handoffSha256 ||
        Object.keys(files).filter(name => name.startsWith(prefix + 'Author/')).length !== Object.keys(origin.authorFiles).length ||
        Object.entries(origin.authorFiles).some(([name, entry]) => files[name]?.sha256 !== entry.sha256 || files[name]?.bytes !== entry.bytes) ||
        (work?.startUtc && work.handoffValidation?.status !== (scenario.readiness === 'PROMPT_BLOCKED' ? 'BLOCKED' : 'PASS')) ||
        (work?.startUtc && scenario.readiness === 'PROMPT_BLOCKED' && work.handoffValidation?.clarificationApproved !== true))) {
      throw new Error('Work continuation lacks preserved Author evidence or reviewed Work intake.');
    }
    if (['OBJECTIVE_PASS_REVIEW_REQUIRED', 'TEST_SHIM_OBJECTIVE_PASS'].includes(result.status)) {
      if (result.status !== (manifest.mode === 'TEST_SHIM' ? 'TEST_SHIM_OBJECTIVE_PASS' : 'OBJECTIVE_PASS_REVIEW_REQUIRED') ||
          !byteExact || !completed(author) || !completed(work) || work.objective?.status !== 'PASS' ||
          work.objectiveProcess?.exitCode !== 0 || work.executionScope?.preflightStatus !== 'PASS' ||
          work.executionScope?.auditStatus !== 'PASS') {
        throw new Error('Objective success conflicts with role evidence.');
      }
    }
    const checks = {
      taskSuccess: scenario.rubric,
      architectureCompliance: scenario.architecture,
      verificationCoverage: scenario.verification,
      privacy: ['data and access boundaries'],
    };
    return { result, scenario, eligible, author, work, byteExact, checks };
  });

  const blank = criterion => ({ criterion, status: 'NOT_RUN', reason: '', evidence: [] });
  if (mode === 'seal') {
    save('evidence-index.json', { schema: 1, files });
    const evidenceIndexSha256 = sha256(bytesAt('evidence-index.json'));
    save('review-template.json', {
      reviewer: '', evidenceIndexSha256,
      cases: Object.fromEntries(records.map(({ scenario, checks }) => [scenario.id,
        Object.fromEntries(Object.entries(checks).map(([key, expected]) => [key, expected.map(blank)]))])),
    });
    console.log('Evidence index SHA-256: ' + evidenceIndexSha256);
    return 0;
  }

  if (sealed.schema !== 1 || Object.keys(files).length !== Object.keys(sealed.files).length ||
      Object.entries(files).some(([name, file]) => file.sha256 !== sealed.files[name]?.sha256 || file.bytes !== sealed.files[name]?.bytes)) {
    throw new Error('Reviewed artifacts differ from the sealed evidence index.');
  }
  if (originalMode) {
    const record = records[0];
    const prefix = `trials/${record.scenario.id}/`;
    const originalWork = join(root, prefix, 'Work');
    const workSetup = lstatSync(originalWork, { throwIfNoEntry: false });
    const setupNames = new Set(['fixture', 'preflight', 'pristine', 'profile', 'tools', 'rtl-preflight', 'session.json']);
    const setupBytes = bytesAt(prefix + 'Work/session.json', true);
    const setup = setupBytes ? json(setupBytes) : null;
    if (records.length !== 1 ||
        manifest.session !== basename(root) || !['DISABLED', 'INITIAL'].includes(manifest.inputMeasurement) ||
        record.result.status !== 'BLOCKED' || record.result.author.invoked !== true || record.result.work.invoked !== false ||
        record.result.work.execution !== 'NOT_RUN' || record.work ||
        (workSetup && (!workSetup.isDirectory() || readdirSync(originalWork).some(name => !setupNames.has(name)))) ||
        (setup && (setup.role !== 'Work' || setup.finishUtc != null || !Array.isArray(setup.cases) || setup.cases.length)) ||
        lstatSync(join(originalWork, 'profile', 'native-final.md'), { throwIfNoEntry: false }) ||
        !completed(record.author) || record.author.handoffValidation?.status !== 'REVIEW_REQUIRED' ||
        files[prefix + 'Author/final.md']?.sha256 !== record.author.finalSha256 ||
        !files[prefix + 'Author/events.jsonl'] || !files[prefix + 'Author/prompt.txt']) {
      throw new Error('Original evidence must prove a completed Author and undispatched Work.');
    }
    return { manifest, manifestSha256: files['manifest.json'].sha256, handoffSha256: record.author.finalSha256,
      authorFiles: Object.fromEntries(Object.entries(files).filter(([name]) => name.startsWith(prefix + 'Author/'))) };
  }
  if (!suppliedReview) throw new Error('Provide a review file inside the owned session.');
  const reviewBytes = bytesAt(relative(root, resolve(suppliedReview)).replaceAll('\\', '/'));
  audit(reviewBytes);
  const review = json(reviewBytes);
  const evidenceIndexSha256 = sha256(indexBytes);
  if (review.evidenceIndexSha256 !== evidenceIndexSha256 || typeof review.reviewer !== 'string' || !review.reviewer.trim() ||
      !review.cases || Object.keys(review.cases).length !== records.length || records.some(record => !Object.hasOwn(review.cases, record.scenario.id))) {
    throw new Error('Review must identify its reviewer, sealed evidence digest and every selected scenario.');
  }

  const cases = records.map(({ result, scenario, eligible, author, work, byteExact, checks }) => {
    const reviewed = review.cases[scenario.id];
    const commands = new Map((work?.trace?.commands ?? []).filter(command =>
      command.event === 'item.completed' && Number.isInteger(command.exitCode)).map(command => ['Work/' + command.id, command.exitCode]));
    const criteria = Object.fromEntries(Object.entries(checks).map(([key, expected]) => {
      if (!Array.isArray(reviewed[key]) || reviewed[key].length !== expected.length) {
        throw new Error('Review must cover every required criterion in catalog order.');
      }
      return [key, expected.map((criterion, index) => {
        const check = reviewed[key][index];
        if (check.criterion !== criterion || !statuses.includes(check.status) || typeof check.reason !== 'string' || !check.reason.trim() ||
            !Array.isArray(check.evidence) || (['PASS', 'FAIL'].includes(check.status) && !check.evidence.length) ||
            check.evidence.some(locator => !eligible.has(locator))) {
          throw new Error('Each criterion needs a verdict, reason and valid case-relative evidence.');
        }
        if (key === 'verificationCoverage' && check.status === 'PASS') {
          if (scenario.id === 'E07' && criterion === 'partner-integration' &&
              !Object.hasOwn(manifest.evaluator, 'fixtures/E07/inputs/partner-batch.json')) {
            throw new Error('Partner integration cannot pass without the pinned E07 partner input.');
          }
          if (!check.evidence.some(locator => commands.has(locator) &&
              (criterion === 'actual-red' ? commands.get(locator) !== 0 :
                criterion === 'pin-decoder-release-inspection' || commands.get(locator) === 0))) {
            throw new Error('Verification PASS requires a cited completed Work command with the expected exit code.');
          }
          if (scenario.rtl && ['compile', 'simulation'].includes(criterion) &&
              !hasSuccessfulRtlStage(work?.trace?.rtlStages ?? [], criterion, check.evidence)) {
            throw new Error('RTL PASS requires the actual compiler or simulator child exit evidence.');
          }
        }
        const required = key !== 'verificationCoverage' || !scenario.optionalVerification?.includes(criterion);
        return { criterion, status: check.status, reason: check.reason, evidence: check.evidence, required };
      })];
    }));
    const quality = Object.fromEntries(Object.entries(criteria).map(([key, values]) =>
      [key, { status: aggregate(values.filter(check => check.required || check.status === 'FAIL').map(check => check.status)), checks: values }]));
    const privacyFailure = [author, work].some(role => role?.trace?.privacyHits || role?.trace?.credentialHits);
    const scopeFailure = [author, work].some(role => role?.process?.scopeStop) ||
      work?.objective?.checks?.some(check => check.status === 'FAIL' && ['protected fixture paths', 'regular fixture files only'].includes(check.name));
    if (privacyFailure) quality.privacy.status = 'FAIL';
    if (scopeFailure) quality.architectureCompliance.status = 'FAIL';
    const execution = result.status === 'FAIL' || privacyFailure || scopeFailure ? 'FAIL' :
      result.status === 'BLOCKED' ? 'BLOCKED' :
      manifest.mode !== 'NATIVE_COMPATIBILITY' || result.status !== 'OBJECTIVE_PASS_REVIEW_REQUIRED' ? 'NOT_RUN' : 'PASS';
    return { id: scenario.id, language: scenario.language, status: aggregate([execution, ...Object.values(quality).map(item => item.status)]),
      execution: result.status, handoffShaMatches: byteExact, quality };
  });
  const status = aggregate(cases.map(record => record.status));
  save('quality-report.json', {
    schema: 1, scope: 'Selected scenarios in one Candidate session',
    source: manifest.mode, sourceRevision: manifest.sourceRevision,
    manifestSha256: files['manifest.json'].sha256, evidenceIndexSha256, reviewSha256: sha256(reviewBytes),
    reviewer: review.reviewer, status, cases,
    ...(continuation ? { continuation } : {}),
  });
  console.log('Candidate quality report: ' + status);
  return status === 'PASS' ? 0 : status === 'FAIL' ? 1 : 2;
}

try {
  const [mode, directory, review] = process.argv.slice(2);
  if (!directory) throw new Error('Provide the benchmark session directory.');
  const result = reportSession(mode, resolve(directory), review);
  if (mode === 'inspect-original') console.log(JSON.stringify(result));
  else process.exitCode = result;
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
